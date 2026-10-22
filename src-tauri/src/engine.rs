//! Supervised single Xray process. Only verified provider servers can reach this API.
//! No Windows route, DNS, WFP or Proxifier settings are changed.
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub const SOCKS_HOST: Ipv4Addr = Ipv4Addr::LOCALHOST;
pub const DEFAULT_SOCKS_PORT: u16 = 10808;
static FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnginePhase {
    Disconnected,
    Starting,
    Running,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub phase: EnginePhase,
    pub pid: Option<u32>,
    pub socks_address: String,
    pub socks_ready: bool,
    pub verified: bool,
    pub egress_ip: Option<String>,
    pub message: String,
}

struct Inner {
    child: Option<Child>,
    config_path: Option<PathBuf>,
    phase: EnginePhase,
    verified: bool,
    egress_ip: Option<String>,
    message: String,
}

pub struct Engine {
    // Serializes connect / disconnect, including slow network verification.
    operations: Mutex<()>,
    inner: Mutex<Inner>,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            operations: Mutex::new(()),
            inner: Mutex::new(Inner {
                child: None,
                config_path: None,
                phase: EnginePhase::Disconnected,
                verified: false,
                egress_ip: None,
                message: "No Xray process started by OneTunnel".into(),
            }),
        }
    }
}

fn delete_config(inner: &mut Inner) {
    if let Some(path) = inner.config_path.take() {
        let _ = fs::remove_file(path);
    }
}

fn reap_process(inner: &mut Inner) -> Result<(), String> {
    if let Some(mut child) = inner.child.take() {
        if child.try_wait().map_err(|e| format!("Cannot inspect Xray: {e}"))?.is_none() {
            child.kill().map_err(|e| format!("Cannot stop Xray: {e}"))?;
            child.wait().map_err(|e| format!("Cannot reap Xray: {e}"))?;
        }
    }
    delete_config(inner);
    inner.verified = false;
    inner.egress_ip = None;
    Ok(())
}

fn write_config(config: &serde_json::Value) -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System clock unavailable")?
        .as_nanos();
    let sequence = FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    // Windows' per-user TEMP directory is used; create_new prevents overwrites.
    let path = std::env::temp_dir().join(format!(
        "onetunnel-{}-{timestamp}-{sequence}.json",
        std::process::id()
    ));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .map_err(|_| "Cannot create temporary Xray configuration")?;
    let written = serde_json::to_writer(&mut file, config)
        .and_then(|_| file.flush().map_err(serde_json::Error::io));
    if written.is_err() {
        let _ = fs::remove_file(&path);
        return Err("Cannot write temporary Xray configuration".into());
    }
    Ok(path)
}

impl Engine {
    fn status_locked(inner: &Inner) -> EngineStatus {
        let ready = inner.child.is_some()
            && crate::health::socks5_ready(DEFAULT_SOCKS_PORT).is_ok();
        EngineStatus {
            phase: inner.phase.clone(),
            pid: inner.child.as_ref().map(Child::id),
            socks_address: format!("{SOCKS_HOST}:{DEFAULT_SOCKS_PORT}"),
            socks_ready: ready,
            verified: ready && inner.verified,
            egress_ip: if ready && inner.verified {
                inner.egress_ip.clone()
            } else {
                None
            },
            message: inner.message.clone(),
        }
    }

    pub fn status(&self) -> Result<EngineStatus, String> {
        let mut guard = self.inner.lock().map_err(|_| "Engine state unavailable")?;
        if let Some(child) = guard.child.as_mut() {
            match child.try_wait() {
                Ok(Some(exit)) => {
                    guard.child = None;
                    delete_config(&mut guard);
                    guard.verified = false;
                    guard.egress_ip = None;
                    guard.phase = EnginePhase::Error;
                    guard.message = format!("Xray exited unexpectedly: {exit}");
                }
                Err(_) => {
                    guard.verified = false;
                    guard.egress_ip = None;
                    guard.phase = EnginePhase::Error;
                    guard.message = "Could not inspect Xray process".into();
                }
                Ok(None) => {}
            }
        }
        let state = Self::status_locked(&guard);
        if guard.verified && !state.socks_ready {
            guard.verified = false;
            guard.egress_ip = None;
            guard.phase = EnginePhase::Error;
            guard.message = "SOCKS listener stopped responding; reconnect required".into();
            return Ok(Self::status_locked(&guard));
        }
        Ok(state)
    }

    fn mark_failed(&self, reason: &str) -> Result<(), String> {
        let mut guard = self.inner.lock().map_err(|_| "Engine state unavailable")?;
        let cleanup = reap_process(&mut guard);
        guard.phase = EnginePhase::Error;
        guard.message = reason.to_string();
        cleanup
    }

    /// The caller must verify the Xray binary SHA-256 and derive the server
    /// exclusively from authenticated 1VPN provider data.
    pub fn connect(
        &self,
        executable: &Path,
        server: &crate::config::RealityServer,
    ) -> Result<EngineStatus, String> {
        let _operation = self.operations.lock().map_err(|_| "Engine is unavailable")?;
        {
            let guard = self.inner.lock().map_err(|_| "Engine state unavailable")?;
            if guard.child.is_some() {
                return Err("Disconnect the current tunnel before connecting another server".into());
            }
        }
        if !executable.is_file() {
            return Err("Verified Xray executable was not found".into());
        }
        let config = crate::config::build_config(server, DEFAULT_SOCKS_PORT)
            .map_err(str::to_owned)?;
        let path = write_config(&config)?;
        // Refuse malformed configurations before opening any local listener.
        let test = Command::new(executable)
            .args(["run", "-test", "-config"])
            .arg(&path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if !matches!(test, Ok(status) if status.success()) {
            let _ = fs::remove_file(&path);
            return Err("Xray rejected the Reality configuration".into());
        }
        let address = SocketAddr::new(IpAddr::V4(SOCKS_HOST), DEFAULT_SOCKS_PORT);
        let listener = TcpListener::bind(address)
            .map_err(|e| {
                let _ = fs::remove_file(&path);
                format!("Local SOCKS port {DEFAULT_SOCKS_PORT} is unavailable: {e}")
            })?;
        drop(listener);

        let child = match Command::new(executable)
            .args(["run", "-config"])
            .arg(&path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => {
                let _ = fs::remove_file(&path);
                return Err("Could not launch verified Xray executable".into());
            }
        };
        {
            let mut guard = self.inner.lock().map_err(|_| "Engine state unavailable")?;
            guard.child = Some(child);
            guard.config_path = Some(path);
            guard.phase = EnginePhase::Starting;
            guard.message = "Checking SOCKS5 listener and HTTPS exit".into();
            guard.verified = false;
            guard.egress_ip = None;
        }
        let mut ready = false;
        for _ in 0..40 {
            let state = self.status()?;
            if state.pid.is_none() {
                return Err("Xray exited while starting; inspect your server settings".into());
            }
            if state.socks_ready {
                ready = true;
                break;
            }
            thread::sleep(Duration::from_millis(200));
        }
        if !ready {
            let reason = "Xray started but SOCKS5 did not become ready";
            self.mark_failed(reason)?;
            return Err(reason.into());
        }
        let ip = match crate::health::verify_https_egress(DEFAULT_SOCKS_PORT) {
            Ok(ip) => ip,
            Err(error) => {
                let reason = format!("SOCKS5 is ready but the HTTPS VPN verification failed: {error}");
                self.mark_failed(&reason)?;
                return Err(reason);
            }
        };
        let mut guard = self.inner.lock().map_err(|_| "Engine state unavailable")?;
        guard.phase = EnginePhase::Running;
        guard.verified = true;
        guard.egress_ip = Some(ip);
        guard.message = "SOCKS5 and HTTPS exit verified through Xray".into();
        Ok(Self::status_locked(&guard))
    }

    pub fn stop(&self) -> Result<EngineStatus, String> {
        let _operation = self.operations.lock().map_err(|_| "Engine is unavailable")?;
        let mut guard = self.inner.lock().map_err(|_| "Engine state unavailable")?;
        reap_process(&mut guard)?;
        guard.phase = EnginePhase::Disconnected;
        guard.message = "Xray stopped".into();
        Ok(Self::status_locked(&guard))
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.inner.lock() {
            let _ = reap_process(&mut guard);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_state_is_disconnected_and_unverified() {
        let engine = Engine::default();
        let status = engine.status().unwrap();
        assert!(matches!(status.phase, EnginePhase::Disconnected));
        assert!(status.pid.is_none());
        assert!(!status.verified);
    }
    #[test]
    fn stop_without_start_is_safe() {
        let status = Engine::default().stop().unwrap();
        assert!(matches!(status.phase, EnginePhase::Disconnected));
    }
    #[test]
    fn transient_config_is_removed_on_cleanup() {
        let config = serde_json::json!({"test":true});
        let path = write_config(&config).unwrap();
        assert!(path.exists());
        let mut inner = Inner {
            child: None,
            config_path: Some(path.clone()),
            phase: EnginePhase::Starting,
            verified: false,
            egress_ip: None,
            message: String::new(),
        };
        reap_process(&mut inner).unwrap();
        assert!(!path.exists());
    }
}

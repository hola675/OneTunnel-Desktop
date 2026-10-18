//! A single supervised Xray process. No route, DNS, or Proxifier changes.
//! This component deliberately does not accept executable paths from the UI.
use serde::Serialize;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
};

pub const SOCKS_HOST: Ipv4Addr = Ipv4Addr::LOCALHOST;
pub const DEFAULT_SOCKS_PORT: u16 = 10808;

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
    pub message: String,
}

struct Inner {
    child: Option<Child>,
    config_path: Option<PathBuf>,
    phase: EnginePhase,
    message: String,
}

pub struct Engine {
    inner: Mutex<Inner>,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            inner: Mutex::new(Inner {
                child: None,
                config_path: None,
                phase: EnginePhase::Disconnected,
                message: "No Xray process started by OneTunnel".into(),
            }),
        }
    }
}

impl Engine {
    fn status_locked(inner: &Inner) -> EngineStatus {
        EngineStatus {
            phase: inner.phase.clone(),
            pid: inner.child.as_ref().map(Child::id),
            socks_address: format!("{SOCKS_HOST}:{DEFAULT_SOCKS_PORT}"),
            message: inner.message.clone(),
        }
    }

    pub fn status(&self) -> Result<EngineStatus, String> {
        let mut guard = self.inner.lock().map_err(|_| "Engine state poisoned")?;
        if let Some(child) = guard.child.as_mut() {
            match child.try_wait() {
                Ok(Some(exit)) => {
                    guard.child = None;
                    guard.config_path = None;
                    guard.phase = EnginePhase::Error;
                    guard.message = format!("Xray exited unexpectedly: {exit}");
                }
                Err(error) => {
                    guard.phase = EnginePhase::Error;
                    guard.message = format!("Unable to inspect Xray process: {error}");
                }
                Ok(None) => {}
            }
        }
        Ok(Self::status_locked(&guard))
    }

    /// Start ONLY after the caller has verified integrity and written a validated config.
    /// Does not imply a working VPN; subsequent SOCKS/HTTPS verification is mandatory.
    pub fn start(&self, executable: &Path, config: &Path) -> Result<EngineStatus, String> {
        if !executable.is_file() || !config.is_file() {
            return Err("Verified executable and configuration files are required".into());
        }
        // Fail closed when the default SOCKS port is already occupied.
        let address = SocketAddr::new(IpAddr::V4(SOCKS_HOST), DEFAULT_SOCKS_PORT);
        let listener = TcpListener::bind(address)
            .map_err(|e| format!("Local SOCKS port is unavailable: {e}"))?;
        drop(listener);

        let mut guard = self.inner.lock().map_err(|_| "Engine state poisoned")?;
        if guard.child.is_some() {
            return Err("An Xray process is already owned by OneTunnel".into());
        }
        guard.phase = EnginePhase::Starting;
        let child = match Command::new(executable)
            .arg("run")
            .arg("-config")
            .arg(config)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                guard.phase = EnginePhase::Error;
                guard.message = format!("Cannot launch Xray: {error}");
                return Err(guard.message.clone());
            }
        };
        guard.config_path = Some(config.to_path_buf());
        guard.child = Some(child);
        guard.phase = EnginePhase::Running;
        guard.message = "Xray launched; tunnel NOT yet verified".into();
        Ok(Self::status_locked(&guard))
    }

    pub fn stop(&self) -> Result<EngineStatus, String> {
        let mut guard = self.inner.lock().map_err(|_| "Engine state poisoned")?;
        if let Some(child) = guard.child.as_mut() {
            if child.try_wait().map_err(|e| e.to_string())?.is_none() {
                child.kill().map_err(|e| format!("Cannot stop owned Xray: {e}"))?;
                child.wait().map_err(|e| format!("Cannot reap owned Xray: {e}"))?;
            }
        }
        guard.child = None;
        guard.config_path = None;
        guard.phase = EnginePhase::Disconnected;
        guard.message = "Xray stopped".into();
        Ok(Self::status_locked(&guard))
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(mut child) = guard.child.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_state_is_disconnected() {
        let engine = Engine::default();
        let state = engine.status().unwrap();
        assert!(matches!(state.phase, EnginePhase::Disconnected));
        assert!(state.pid.is_none());
    }
    #[test]
    fn stop_without_start_is_safe() {
        assert!(matches!(Engine::default().stop().unwrap().phase, EnginePhase::Disconnected));
    }
    #[test]
    fn missing_runtime_fails_closed() {
        let engine = Engine::default();
        assert!(engine.start(Path::new("missing-xray.exe"), Path::new("missing.json")).is_err());
        assert!(engine.status().unwrap().pid.is_none());
    }
}

//! F1 read-only Xray runtime preflight. No processes are started in this phase.
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

const EXPECTED_VERSION: &str = "26.9.9";
const MAX_OUTPUT_BYTES: usize = 4096;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayPreflight {
    pub found: bool,
    pub supported: bool,
    pub executable: String,
    pub reported_version: Option<String>,
    pub issue: Option<String>,
}

/// Resolve a trusted application-local executable only. Never use PATH,
/// shell commands, network downloads, or a caller-provided binary path.
pub fn xray_path(resource_dir: &Path) -> PathBuf {
    resource_dir.join("xray.exe")
}

pub fn preflight(resource_dir: &Path) -> XrayPreflight {
    let executable = xray_path(resource_dir);
    let display = executable.display().to_string();
    if !executable.is_file() {
        return XrayPreflight {
            found: false,
            supported: false,
            executable: display,
            reported_version: None,
            issue: Some("XRAY_NOT_BUNDLED".into()),
        };
    }
    // The CLI must never block the UI if a bad binary is bundled.
    // Process spawning is intentionally deferred to the F1 engine lifecycle.
    XrayPreflight {
        found: true,
        supported: false,
        executable: display,
        reported_version: None,
        issue: Some("XRAY_NOT_VERIFIED".into()),
    }
}

/// This function is for native tests and the later F1 executable verifier.
/// It must not be called on the Tauri UI thread.
#[allow(dead_code)]
pub fn check_version(executable: &Path) -> Result<String, String> {
    if !executable.is_file() {
        return Err("XRAY_NOT_FOUND".into());
    }
    let mut child = Command::new(executable)
        .arg("version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "XRAY_START_FAILED".to_string())?;
    let started = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut output = Vec::new();
                use std::io::Read;
                if let Some(mut stdout) = child.stdout.take() {
                    stdout
                        .take(MAX_OUTPUT_BYTES as u64)
                        .read_to_end(&mut output)
                        .map_err(|_| "XRAY_READ_FAILED".to_string())?;
                }
                if !status.success() {
                    return Err("XRAY_VERSION_FAILED".into());
                }
                let version = String::from_utf8_lossy(&output).into_owned();
                if !version.contains(EXPECTED_VERSION) {
                    return Err("XRAY_VERSION_MISMATCH".into());
                }
                return Ok(version.lines().next().unwrap_or("").to_string());
            }
            Ok(None) if started.elapsed() < Duration::from_secs(3) => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("XRAY_VERSION_TIMEOUT".into());
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("XRAY_VERSION_FAILED".into());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xray_path_is_not_resolved_from_path_env() {
        let path = xray_path(Path::new("C:/OneTunnel/resources"));
        assert!(path.ends_with("xray.exe"));
        assert!(path.to_string_lossy().contains("OneTunnel"));
    }
    #[test]
    fn missing_runtime_is_not_reported_as_connected() {
        let result = preflight(Path::new("Z:/not-a-real-onetunnel-location"));
        assert!(!result.found);
        assert!(!result.supported);
        assert_eq!(result.issue.as_deref(), Some("XRAY_NOT_BUNDLED"));
    }
}

//! F1: local Xray runtime discovery and non-invasive validation.
//! This module never changes Windows routes, DNS, or Proxifier settings.
use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCheck {
    pub ready: bool,
    pub executable: Option<String>,
    pub version_output: Option<String>,
    pub message: String,
}

fn runtime_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(path) = std::env::var("ONETUNNEL_XRAY_PATH") {
        if !path.is_empty() {
            paths.push(PathBuf::from(path));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("xray.exe"));
            paths.push(dir.join("runtime").join("xray.exe"));
            paths.push(dir.join("resources").join("xray.exe"));
        }
    }
    paths
}

pub fn inspect_runtime() -> RuntimeCheck {
    for candidate in runtime_candidates() {
        if !candidate.is_file() {
            continue;
        }
        let expected = match std::env::var("ONETUNNEL_XRAY_SHA256") {
            Ok(hash) => hash,
            Err(_) => {
                return RuntimeCheck {
                    ready: false,
                    executable: Some(candidate.display().to_string()),
                    version_output: None,
                    message: "Expected Xray SHA-256 not configured; refusing execution".into(),
                }
            }
        };
        if let Err(error) = crate::integrity::verify_sha256(&candidate, &expected) {
            return RuntimeCheck {
                ready: false,
                executable: Some(candidate.display().to_string()),
                version_output: None,
                message: error,
            };
        }
        match check_version(&candidate) {
            Ok(output) => {
                return RuntimeCheck {
                    ready: true,
                    executable: Some(candidate.display().to_string()),
                    version_output: Some(output),
                    message: "Xray SHA-256 verified and version check passed; no tunnel started."
                        .into(),
                }
            }
            Err(error) => {
                return RuntimeCheck {
                    ready: false,
                    executable: Some(candidate.display().to_string()),
                    version_output: None,
                    message: format!("Xray runtime verification failed: {error}"),
                }
            }
        }
    }
    RuntimeCheck {
        ready: false,
        executable: None,
        version_output: None,
        message: "xray.exe not found beside the app or in its runtime folder.".into(),
    }
}

fn check_version(path: &Path) -> Result<String, String> {
    // No shell invocation. The known local executable is invoked directly.
    let output = Command::new(path)
        .arg("version")
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!("xray version exited with {}", output.status));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !stdout.starts_with("Xray ") {
        return Err("Unexpected version response".into());
    }
    Ok(stdout.lines().next().unwrap_or_default().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_executable_is_not_ready() {
        assert!(check_version(Path::new("does-not-exist-onetunnel-xray.exe")).is_err());
    }
    #[test]
    fn runtime_result_is_json_serializable() {
        let result = RuntimeCheck {
            ready: false,
            executable: None,
            version_output: None,
            message: "missing".into(),
        };
        assert!(!result.ready);
    }
}

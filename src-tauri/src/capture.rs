//! WFP detection and fail-closed availability reporting.
//! Detecting the Windows Base Filtering Engine does not imply a redirect driver exists.
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureCapability {
    pub platform: &'static str,
    pub wfp_engine_running: bool,
    pub driver_installed: bool,
    pub interception_active: bool,
    pub redirect_supported: bool,
    pub blocking_supported: bool,
    pub administrator_approval_required: bool,
    pub supported_transport: &'static str,
    pub reason: &'static str,
}

#[cfg(target_os = "windows")]
fn bfe_running() -> bool {
    let output = std::process::Command::new("sc.exe")
        .args(["query", "BFE"])
        .output();
    match output {
        Ok(result) if result.status.success() => {
            let text = String::from_utf8_lossy(&result.stdout);
            text.lines().any(|line| {
                line.trim_start().starts_with("STATE")
                    && line.contains("RUNNING")
            })
        }
        _ => false,
    }
}

#[cfg(not(target_os = "windows"))]
fn bfe_running() -> bool {
    false
}

pub fn capability() -> CaptureCapability {
    CaptureCapability {
        platform: if cfg!(target_os = "windows") { "windows" } else { "unsupported" },
        wfp_engine_running: bfe_running(),
        driver_installed: false,
        interception_active: false,
        redirect_supported: false,
        blocking_supported: false,
        administrator_approval_required: true,
        supported_transport: "none",
        reason: "Windows WFP detection only. No signed OneTunnel redirect callout driver or audited TCP forwarding service is shipped.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn feature_is_fail_closed_even_if_wfp_is_running() {
        let status = capability();
        assert!(!status.driver_installed);
        assert!(!status.interception_active);
        assert!(!status.redirect_supported);
        assert!(!status.blocking_supported);
    }
}

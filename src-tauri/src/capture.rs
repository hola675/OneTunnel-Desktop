//! Windows interception capability boundary.
//! This reports readiness only. Installing or enabling a WFP driver is explicitly unsupported.
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureCapability {
    pub platform: &'static str,
    pub driver_installed: bool,
    pub interception_active: bool,
    pub redirect_supported: bool,
    pub blocking_supported: bool,
    pub administrator_approval_required: bool,
    pub supported_transport: &'static str,
    pub reason: &'static str,
}

pub fn capability() -> CaptureCapability {
    CaptureCapability {
        platform: if cfg!(target_os = "windows") { "windows" } else { "unsupported" },
        driver_installed: false,
        interception_active: false,
        redirect_supported: false,
        blocking_supported: false,
        administrator_approval_required: true,
        supported_transport: "none",
        reason: "No signed WFP callout driver or audited forwarding service is bundled",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn feature_is_fail_closed() {
        let status = capability();
        assert!(!status.driver_installed);
        assert!(!status.interception_active);
        assert!(!status.redirect_supported);
        assert!(!status.blocking_supported);
    }
}

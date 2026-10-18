//! OneTunnel routing policy V0. The engine evaluates only supplied metadata.
//! It does NOT capture, redirect, block or monitor Windows network connections.
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteAction {
    Tunnel,
    Direct,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteRule {
    pub id: String,
    pub enabled: bool,
    pub application: Option<String>,
    pub host_pattern: Option<String>,
    pub port_start: Option<u16>,
    pub port_end: Option<u16>,
    pub action: RouteAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionMeta {
    pub application: String,
    pub destination_host: String,
    pub destination_port: u16,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteDecision {
    pub action: RouteAction,
    pub matched_rule: String,
    pub applied: bool,
    pub explanation: &'static str,
}

fn matches_glob(pattern: &str, input: &str) -> bool {
    let pattern = pattern.as_bytes();
    let input = input.as_bytes();
    let (mut p, mut i, mut star, mut after_star) = (0, 0, None, 0);
    while i < input.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p].eq_ignore_ascii_case(&input[i])) {
            p += 1;
            i += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            after_star = i;
        } else if let Some(previous_star) = star {
            p = previous_star + 1;
            after_star += 1;
            i = after_star;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

fn application_matches(pattern: &str, application: &str) -> bool {
    let normalized = application.replace('/', "\\");
    let name = normalized.rsplit('\\').next().unwrap_or(&normalized);
    if pattern.contains('/') || pattern.contains('\\') {
        matches_glob(pattern, &normalized)
    } else {
        matches_glob(pattern, name)
    }
}

fn internal_process(application: &str) -> bool {
    let normalized = application.replace('/', "\\").to_ascii_lowercase();
    let filename = normalized.rsplit('\\').next().unwrap_or(&normalized);
    matches!(
        filename,
        "xray.exe" | "onetunnel-desktop.exe" | "onetunnel.exe"
    )
}

fn localhost(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

pub fn validate(rules: &[RouteRule]) -> Result<(), String> {
    if rules.len() > 200 {
        return Err("Maximum 200 routing rules".into());
    }
    let mut ids = std::collections::HashSet::new();
    for rule in rules {
        if rule.id.is_empty() || rule.id.len() > 80 || !ids.insert(rule.id.as_str()) {
            return Err("Routing rule IDs must be unique and nonempty".into());
        }
        for value in [rule.application.as_deref(), rule.host_pattern.as_deref()]
            .into_iter()
            .flatten()
        {
            if value.is_empty() || value.len() > 255 || value.contains(|c: char| c.is_control()) {
                return Err("Invalid routing match pattern".into());
            }
        }
        match (rule.port_start, rule.port_end) {
            (None, None) => {}
            (Some(start), Some(end)) if start > 0 && end >= start => {}
            _ => return Err("Invalid routing port range".into()),
        }
    }
    Ok(())
}

/// First matching enabled rule wins. Localhost bypass is an immutable guard.
pub fn evaluate(
    rules: &[RouteRule],
    default: RouteAction,
    meta: &ConnectionMeta,
) -> Result<RouteDecision, String> {
    validate(rules)?;
    if meta.destination_port == 0
        || meta.application.trim().is_empty()
        || meta.destination_host.trim().is_empty()
    {
        return Err("Invalid connection metadata".into());
    }
    if internal_process(&meta.application) {
        return Ok(RouteDecision {
            action: RouteAction::Direct,
            matched_rule: "system:internal-process".into(),
            applied: false,
            explanation:
                "Simulation only; core runtime processes are excluded to avoid redirect loops",
        });
    }
    if localhost(&meta.destination_host) {
        return Ok(RouteDecision {
            action: RouteAction::Direct,
            matched_rule: "system:loopback".into(),
            applied: false,
            explanation: "Simulation only; localhost is never redirected",
        });
    }
    for rule in rules.iter().filter(|rule| rule.enabled) {
        if rule
            .application
            .as_deref()
            .is_some_and(|p| !application_matches(p, &meta.application))
        {
            continue;
        }
        if rule
            .host_pattern
            .as_deref()
            .is_some_and(|p| !matches_glob(p, &meta.destination_host))
        {
            continue;
        }
        if let (Some(start), Some(end)) = (rule.port_start, rule.port_end) {
            if !(start..=end).contains(&meta.destination_port) {
                continue;
            }
        }
        return Ok(RouteDecision {
            action: rule.action,
            matched_rule: rule.id.clone(),
            applied: false,
            explanation: "Decision preview only; no Windows traffic has been intercepted",
        });
    }
    Ok(RouteDecision {
        action: default,
        matched_rule: "system:default".into(),
        applied: false,
        explanation: "Default policy preview only; no Windows traffic has been intercepted",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn meta(host: &str) -> ConnectionMeta {
        ConnectionMeta {
            application: "firefox.exe".into(),
            destination_host: host.into(),
            destination_port: 443,
        }
    }
    fn rule(pattern: &str, action: RouteAction) -> RouteRule {
        RouteRule {
            id: "browser".into(),
            enabled: true,
            application: Some("fire*.exe".into()),
            host_pattern: Some(pattern.into()),
            port_start: Some(443),
            port_end: Some(443),
            action,
        }
    }
    #[test]
    fn wildcard_and_case_insensitive_rules() {
        assert_eq!(
            evaluate(
                &[rule("*.example.com", RouteAction::Tunnel)],
                RouteAction::Direct,
                &meta("WWW.EXAMPLE.COM")
            )
            .unwrap()
            .action,
            RouteAction::Tunnel
        );
    }
    #[test]
    fn executable_rule_matches_full_windows_path() {
        let request = ConnectionMeta {
            application: "C:\\Program Files\\Mozilla Firefox\\FIREFOX.EXE".into(),
            destination_host: "www.example.com".into(),
            destination_port: 443,
        };
        assert_eq!(
            evaluate(
                &[rule("*.example.com", RouteAction::Tunnel)],
                RouteAction::Direct,
                &request
            )
            .unwrap()
            .action,
            RouteAction::Tunnel
        );
    }
    #[test]
    fn localhost_is_always_direct() {
        assert_eq!(
            evaluate(
                &[rule("*", RouteAction::Block)],
                RouteAction::Block,
                &meta("127.0.0.1")
            )
            .unwrap()
            .action,
            RouteAction::Direct
        );
    }
    #[test]
    fn non_matching_uses_default() {
        assert_eq!(
            evaluate(
                &[rule("*.invalid", RouteAction::Block)],
                RouteAction::Direct,
                &meta("example.com")
            )
            .unwrap()
            .action,
            RouteAction::Direct
        );
    }
    #[test]
    fn no_rule_is_applied_to_real_traffic() {
        assert!(
            !evaluate(&[], RouteAction::Block, &meta("example.com"))
                .unwrap()
                .applied
        );
    }
    #[test]
    fn xray_core_is_excluded_from_redirect_and_block() {
        for name in [
            "xray.exe",
            "C:\\Program Files\\OneTunnel\\xray.exe",
            "ONETUNNEL-DESKTOP.EXE",
        ] {
            let mut request = meta("example.com");
            request.application = name.into();
            let result = evaluate(
                &[rule("*", RouteAction::Block)],
                RouteAction::Tunnel,
                &request,
            )
            .unwrap();
            assert_eq!(result.action, RouteAction::Direct);
            assert_eq!(result.matched_rule, "system:internal-process");
        }
    }
    #[test]
    fn reject_invalid_port_range() {
        let mut r = rule("*", RouteAction::Tunnel);
        r.port_start = Some(9000);
        r.port_end = Some(8000);
        assert!(validate(&[r]).is_err());
    }
}

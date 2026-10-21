//! Strict VLESS Reality configuration model; no secrets are embedded in source control.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealityServer {
    pub address: String,
    pub port: u16,
    pub uuid: String,
    pub public_key: String,
    pub short_id: String,
    pub server_name: String,
    pub fingerprint: String,
}

fn is_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && [8, 13, 18, 23].iter().all(|&n| bytes[n] == b'-')
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| [8, 13, 18, 23].contains(&i) || b.is_ascii_hexdigit())
}

pub fn validate_server(server: &RealityServer) -> Result<(), &'static str> {
    let host = server.address.trim();
    if host.is_empty()
        || host.len() > 253
        || host
            .chars()
            .any(|c| c.is_control() || matches!(c, '/' | '\\' | '@' | ' ' | ':'))
    {
        return Err("Invalid server address");
    }
    if host
        .parse::<IpAddr>()
        .map(|ip| ip.is_loopback() || ip.is_unspecified() || ip.is_multicast())
        .unwrap_or(false)
    {
        return Err("Forbidden destination address");
    }
    if server.port != 443 {
        return Err("V1 requires port 443");
    }
    if !is_uuid(&server.uuid) {
        return Err("Invalid UUID");
    }
    if server.public_key.len() != 43
        || !server
            .public_key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("Invalid Reality public key");
    }
    if server.short_id.len() > 16
        || server.short_id.len() % 2 != 0
        || !server.short_id.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("Invalid Reality short ID");
    }
    if server.server_name.is_empty()
        || server.server_name.len() > 253
        || server
            .server_name
            .chars()
            .any(|c| c.is_control() || matches!(c, '/' | '\\' | ' ' | ':'))
    {
        return Err("Invalid Reality SNI");
    }
    if !matches!(
        server.fingerprint.as_str(),
        "chrome" | "firefox" | "safari" | "edge"
    ) {
        return Err("Unsupported fingerprint");
    }
    Ok(())
}

pub fn build_config(server: &RealityServer, socks_port: u16) -> Result<Value, &'static str> {
    validate_server(server)?;
    if socks_port < 1024 {
        return Err("SOCKS port outside allowed range");
    }
    Ok(json!({
        "log": {"loglevel": "warning"},
        "inbounds": [{
            "tag": "onetunnel-socks",
            "listen": "127.0.0.1",
            "port": socks_port,
            "protocol": "socks",
            "settings": {"auth": "noauth", "udp": false}
        }],
        "outbounds": [{
            "tag": "onetunnel-proxy",
            "protocol": "vless",
            "settings": {"vnext": [{"address": server.address, "port": server.port,
                "users": [{"id": server.uuid, "encryption": "none", "flow": "xtls-rprx-vision"}]}]},
            "streamSettings": {
                "network": "tcp",
                "security": "reality",
                "realitySettings": {
                    "show": false, "fingerprint": server.fingerprint,
                    "serverName": server.server_name,
                    "publicKey": server.public_key,
                    "shortId": server.short_id,
                    "spiderX": ""
                }
            }
        }],
        "routing": {"domainStrategy": "AsIs", "rules": []}
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> RealityServer {
        RealityServer {
            address: "vpn.example.org".into(),
            port: 443,
            uuid: "b4c7e30a-8f42-4a9f-a7b6-c168507c0457".into(),
            public_key: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into(),
            short_id: "abcdef01".into(),
            server_name: "www.example.org".into(),
            fingerprint: "chrome".into(),
        }
    }
    #[test]
    fn valid_config_is_local_socks_only() {
        let conf = build_config(&fixture(), 10808).unwrap();
        assert_eq!(conf["inbounds"][0]["listen"], "127.0.0.1");
        assert_eq!(conf["inbounds"][0]["settings"]["udp"], false);
        assert_eq!(
            conf["outbounds"][0]["streamSettings"]["realitySettings"]["spiderX"],
            ""
        );
        assert_eq!(
            conf["outbounds"][0]["streamSettings"]["security"],
            "reality"
        );
    }
    #[test]
    fn invalid_uuid_is_rejected() {
        let mut s = fixture();
        s.uuid = "invalid".into();
        assert!(build_config(&s, 10808).is_err());
    }
    #[test]
    fn rejects_invalid_reality_public_key() {
        let mut server = fixture();
        server.public_key = "invalid".into();
        assert!(build_config(&server, 10808).is_err());
    }
    #[test]
    fn rejects_invalid_reality_short_id() {
        let mut server = fixture();
        server.short_id = "abc".into();
        assert!(build_config(&server, 10808).is_err());
    }
    #[test]
    fn configuration_contains_no_system_proxy_or_route_changes() {
        let config = build_config(&fixture(), 10808).unwrap();
        assert_eq!(config["inbounds"][0]["listen"], "127.0.0.1");
        assert!(config.get("dns").is_none());
        assert!(config.get("tun").is_none());
        assert_eq!(config["outbounds"][0]["protocol"], "vless");
    }
    #[test]
    fn privileged_port_is_rejected() {
        assert!(build_config(&fixture(), 80).is_err());
    }
}

//! Interoperability data models independently implemented from the public 1VPN Android API.
//! No network requests or credential storage in this module.
use serde::{Deserialize, Serialize};
use crate::config::{RealityServer, validate_server};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiServer {
    pub host: String,
    #[serde(default)]
    pub reality_server_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiLocation {
    pub city: Option<String>,
    pub city_code: Option<String>,
    pub country: String,
    pub country_code: String,
    pub servers: Option<Vec<ApiServer>>,
    #[serde(default)]
    pub is_premium: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub username: String,
    pub email: String,
    pub is_premium: bool,
    pub session_auth_token: String,
    pub uuid: String,
    pub public_key: String,
    pub short_id: String,
    pub locations: Vec<ApiLocation>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicAccountSummary {
    pub username: String,
    pub is_premium: bool,
    pub location_count: usize,
}

impl UserData {
    pub fn summary(&self) -> PublicAccountSummary {
        PublicAccountSummary {
            username: self.username.clone(),
            is_premium: self.is_premium,
            location_count: self.locations.len(),
        }
    }

    /// Location permissions come from the authenticated API response, not caller flags.
    /// Unknown/missing SNI is rejected rather than guessed.
    pub fn to_reality_server(&self, location_index: usize, server_index: usize) -> Result<RealityServer, &'static str> {
        let location = self.locations.get(location_index).ok_or("Unknown location")?;
        if location.is_premium && !self.is_premium { return Err("Premium location not authorized"); }
        let selected = location.servers.as_ref().and_then(|s| s.get(server_index))
            .ok_or("Unknown or empty server")?;
        let sni = selected.reality_server_name.as_deref().filter(|s| !s.is_empty())
            .ok_or("Missing Reality server name")?;
        let candidate = RealityServer {
            address: selected.host.clone(),
            port: 443,
            uuid: self.uuid.clone(),
            public_key: self.public_key.clone(),
            short_id: self.short_id.clone(),
            server_name: sni.to_owned(),
            fingerprint: "chrome".into(),
        };
        validate_server(&candidate)?;
        Ok(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn user(is_premium: bool) -> UserData {
        let sample = format!(r#"{{
            "username":"tester","email":"tester@example.invalid","isPremium":{is_premium},
            "sessionAuthToken":"private-token",
            "uuid":"b4c7e30a-8f42-4a9f-a7b6-c168507c0457",
            "publicKey":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "shortId":"abcdef01",
            "locations":[{{"country":"Netherlands","countryCode":"NL","isPremium":true,
                "servers":[{{"host":"vpn.example.org","realityServerName":"www.example.org"}}]}}]
        }}"#);
        serde_json::from_str(&sample).unwrap()
    }
    #[test] fn premium_location_is_rejected_for_free_account() {
        assert!(user(false).to_reality_server(0, 0).is_err());
    }
    #[test] fn premium_location_is_available_to_premium_account() {
        let account = user(true);
        assert_eq!(account.to_reality_server(0, 0).unwrap().port, 443);
        let json = serde_json::to_string(&account.summary()).unwrap();
        assert!(!json.contains("private-token"));
        assert!(!json.contains("publicKey"));
    }
    #[test] fn server_without_sni_is_rejected() {
        let mut account = user(true);
        account.locations[0].servers.as_mut().unwrap()[0].reality_server_name = None;
        assert!(account.to_reality_server(0, 0).is_err());
    }
}

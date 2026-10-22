//! 1VPN account login. Credentials and tokens remain exclusively in Rust memory.
use crate::provider::{PublicAccountSummary, UserData};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

const DOMAINS: [&str; 4] = ["1vpn.org", "1vpn.co", "onevpn.com", "cloudlogcdn.com"];
const MAX_REPLY_BYTES: usize = 2 * 1024 * 1024;

#[derive(Default)]
pub struct AuthState {
    account: Mutex<Option<UserData>>,
}

#[derive(Deserialize)]
struct ApiError {
    #[serde(default)]
    code: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum LoginResult {
    #[serde(rename = "authenticated")]
    Authenticated { account: PublicAccountSummary },
    #[serde(rename = "requires_2fa")]
    Requires2fa,
}

impl AuthState {
    pub fn locations(&self) -> Result<Vec<crate::provider::PublicLocationChoice>, String> {
        let guard = self.account.lock().map_err(|_| "Account state unavailable")?;
        Ok(guard.as_ref().map(UserData::available_locations).unwrap_or_default())
    }

    pub fn selected_server(
        &self,
        location_index: usize,
        server_index: usize,
    ) -> Result<crate::config::RealityServer, String> {
        let guard = self.account.lock().map_err(|_| "Account state unavailable")?;
        guard.as_ref()
            .ok_or_else(|| "Sign in to 1VPN before choosing a server".to_owned())?
            .to_reality_server(location_index, server_index)
            .map_err(str::to_owned)
    }

    pub fn summary(&self) -> Result<Option<PublicAccountSummary>, String> {
        let guard = self
            .account
            .lock()
            .map_err(|_| "Account state unavailable")?;
        Ok(guard.as_ref().map(UserData::summary))
    }
    pub fn logout(&self) -> Result<(), String> {
        *self
            .account
            .lock()
            .map_err(|_| "Account state unavailable")? = None;
        Ok(())
    }

    pub async fn login(
        &self,
        email: String,
        password: String,
        token: Option<String>,
    ) -> Result<LoginResult, String> {
        if email.len() > 254
            || !email.contains('@')
            || email.trim() != email
            || email.contains(char::is_whitespace)
        {
            return Err("Invalid email address".into());
        }
        if password.is_empty() || password.len() > 1024 {
            return Err("Password is required".into());
        }
        if token.as_ref().is_some_and(|t| {
            t.is_empty() || t.len() > 64 || !t.chars().all(|c| c.is_ascii_alphanumeric())
        }) {
            return Err("Invalid verification code".into());
        }

        // Never persist passwords or log request/response bodies.
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(12))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "Cannot initialize secure HTTP client")?;
        let body = serde_json::json!({"email": email, "password": password, "token": token});

        for domain in DOMAINS {
            let url = format!("https://{domain}/api/login/?type=xray");
            let response = match client
                .post(&url)
                .header("Client-Type", "app")
                .json(&body)
                .send()
                .await
            {
                Ok(r) => r,
                Err(_) => continue,
            };
            let status = response.status();
            // Do not attempt another host after a definitive account response (incl. 2FA).
            if status.is_server_error() {
                continue;
            }
            if response
                .content_length()
                .is_some_and(|length| length > MAX_REPLY_BYTES as u64)
            {
                return Err("API response too large".into());
            }
            let mut response = response;
            let mut bytes = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|_| "Could not read authentication response")?
            {
                if chunk.len() > MAX_REPLY_BYTES.saturating_sub(bytes.len()) {
                    return Err("API response too large".into());
                }
                bytes.extend_from_slice(&chunk);
            }
            if status.is_success() {
                let account: UserData = serde_json::from_slice(&bytes)
                    .map_err(|_| "Unexpected 1VPN account response")?;
                if account.session_auth_token.is_empty() {
                    return Err("1VPN response contained no session token".into());
                }
                let summary = account.summary();
                *self
                    .account
                    .lock()
                    .map_err(|_| "Account state unavailable")? = Some(account);
                return Ok(LoginResult::Authenticated { account: summary });
            }
            if serde_json::from_slice::<ApiError>(&bytes)
                .ok()
                .and_then(|e| e.code)
                == Some(1001)
            {
                return Ok(LoginResult::Requires2fa);
            }
            return Err(match status.as_u16() {
                400 | 401 | 403 => "Invalid credentials or authentication denied",
                429 => "Too many login attempts; try again later",
                _ => "1VPN authentication rejected the request",
            }
            .into());
        }
        Err("Could not reach the 1VPN authentication service over HTTPS".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_account_and_logout() {
        let state = AuthState::default();
        assert!(state.summary().unwrap().is_none());
        assert!(state.logout().is_ok());
    }
    #[test]
    fn two_factor_response_contract() {
        let response = r#"{"error":"verification required","code":1001}"#;
        let parsed: ApiError = serde_json::from_str(response).unwrap();
        assert_eq!(parsed.code, Some(1001));
    }
}

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    app_name: &'static str,
    version: &'static str,
    platform: &'static str,
    core_status: &'static str,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        app_name: "OneTunnel",
        version: env!("CARGO_PKG_VERSION"),
        platform: "windows",
        core_status: "runtime_preflight",
    }
}

#[tauri::command]
pub fn inspect_xray_runtime() -> crate::xray::RuntimeCheck {
    crate::xray::inspect_runtime()
}

#[tauri::command]
pub fn get_engine_status(
    engine: tauri::State<'_, crate::engine::Engine>,
) -> Result<crate::engine::EngineStatus, String> {
    engine.status()
}

#[tauri::command]
pub async fn login_1vpn(
    auth: tauri::State<'_, crate::auth::AuthState>,
    email: String,
    password: String,
    token: Option<String>,
) -> Result<crate::auth::LoginResult, String> {
    auth.login(email, password, token).await
}

#[tauri::command]
pub fn get_account(
    auth: tauri::State<'_, crate::auth::AuthState>,
) -> Result<Option<crate::provider::PublicAccountSummary>, String> {
    auth.summary()
}

#[tauri::command]
pub fn logout_1vpn(auth: tauri::State<'_, crate::auth::AuthState>) -> Result<(), String> {
    auth.logout()
}

#[tauri::command]
pub fn list_1vpn_servers(
    auth: tauri::State<'_, crate::auth::AuthState>,
) -> Result<Vec<crate::selection::ServerCandidate>, String> {
    auth.server_catalog()
}

/// This is a TCP-only preflight; do not label the winner "connected" or "best VPN".
#[tauri::command]
pub async fn scan_1vpn_servers(
    auth: tauri::State<'_, crate::auth::AuthState>,
) -> Result<Vec<crate::selection::ServerMeasurement>, String> {
    let candidates = auth.server_catalog()?;
    tauri::async_runtime::spawn_blocking(move || crate::selection::scan(candidates))
        .await
        .map_err(|_| "Server evaluation worker failed".into())
}

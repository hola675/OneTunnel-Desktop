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

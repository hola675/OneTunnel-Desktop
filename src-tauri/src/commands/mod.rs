use serde::Serialize;
use std::sync::Arc;

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
    engine: tauri::State<'_, Arc<crate::engine::Engine>>,
) -> Result<crate::engine::EngineStatus, String> {
    engine.status()
}

#[tauri::command]
pub async fn login_1vpn(
    engine: tauri::State<'_, Arc<crate::engine::Engine>>,
    auth: tauri::State<'_, crate::auth::AuthState>,
    email: String,
    password: String,
    token: Option<String>,
) -> Result<crate::auth::LoginResult, String> {
    if engine.status()?.pid.is_some() {
        return Err("Disconnect the VPN before changing accounts".into());
    }
    auth.login(email, password, token).await
}

#[tauri::command]
pub fn get_account(
    auth: tauri::State<'_, crate::auth::AuthState>,
) -> Result<Option<crate::provider::PublicAccountSummary>, String> {
    auth.summary()
}

#[tauri::command]
pub async fn logout_1vpn(
    auth: tauri::State<'_, crate::auth::AuthState>,
    engine: tauri::State<'_, Arc<crate::engine::Engine>>,
) -> Result<(), String> {
    let owned = Arc::clone(&*engine);
    tauri::async_runtime::spawn_blocking(move || owned.stop())
        .await
        .map_err(|_| "Could not stop Xray when logging out")??;
    auth.logout()
}

#[tauri::command]
pub fn list_1vpn_locations(
    auth: tauri::State<'_, crate::auth::AuthState>,
) -> Result<Vec<crate::provider::PublicLocationChoice>, String> {
    auth.locations()
}

/// Only provider-authorized Reality servers may be used. No raw
/// executable path, credentials or Reality JSON cross this IPC boundary.
#[tauri::command]
pub async fn connect_1vpn(
    auth: tauri::State<'_, crate::auth::AuthState>,
    engine: tauri::State<'_, Arc<crate::engine::Engine>>,
    location_index: usize,
    server_index: usize,
) -> Result<crate::engine::EngineStatus, String> {
    let server = auth.selected_server(location_index, server_index)?;
    let runtime = crate::xray::inspect_runtime();
    if !runtime.ready {
        return Err(runtime.message);
    }
    let executable = std::path::PathBuf::from(
        runtime.executable.ok_or("Verified Xray path not available")?,
    );
    let expected_hash = std::env::var("ONETUNNEL_XRAY_SHA256")
        .map_err(|_| "Trusted Xray SHA-256 must be configured before connecting")?;
    let owned = Arc::clone(&*engine);
    tauri::async_runtime::spawn_blocking(move || {
        // Re-verify immediately before executing (preflight can be stale).
        crate::integrity::verify_sha256(&executable, &expected_hash)?;
        owned.connect(&executable, &server)
    })
    .await
    .map_err(|_| "Xray connection task failed")?
}

#[tauri::command]
pub async fn disconnect_1vpn(
    engine: tauri::State<'_, Arc<crate::engine::Engine>>,
) -> Result<crate::engine::EngineStatus, String> {
    let owned = Arc::clone(&*engine);
    tauri::async_runtime::spawn_blocking(move || owned.stop())
        .await
        .map_err(|_| "Xray disconnection task failed")?
}

#[tauri::command]
pub fn preview_traffic_rule(
    rules: Vec<crate::routing::RouteRule>,
    default_action: crate::routing::RouteAction,
    connection: crate::routing::ConnectionMeta,
) -> Result<crate::routing::RouteDecision, String> {
    crate::routing::evaluate(&rules, default_action, &connection)
}

#[tauri::command]
pub fn get_traffic_policy(
    policies: tauri::State<'_, crate::policy::PolicyStore>,
) -> Result<crate::policy::PolicySnapshot, String> {
    policies.snapshot()
}

#[tauri::command]
pub fn save_traffic_policy(
    policies: tauri::State<'_, crate::policy::PolicyStore>,
    rules: Vec<crate::routing::RouteRule>,
    default_action: crate::routing::RouteAction,
    expected_revision: u64,
) -> Result<crate::policy::PolicySnapshot, String> {
    policies.replace(rules, default_action, expected_revision)
}

#[tauri::command]
pub fn preview_saved_traffic_policy(
    policies: tauri::State<'_, crate::policy::PolicyStore>,
    connection: crate::routing::ConnectionMeta,
) -> Result<crate::routing::RouteDecision, String> {
    policies.preview(&connection)
}

#[tauri::command]
pub fn get_capture_capability() -> crate::capture::CaptureCapability {
    crate::capture::capability()
}

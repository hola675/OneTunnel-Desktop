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
pub async fn probe_remote_tcp(
    engine: tauri::State<'_, crate::engine::Engine>,
) -> Result<crate::egress::EgressProbe, String> {
    let status = engine.status()?;
    if status.pid.is_none() || !status.socks_ready {
        return Err("Owned Xray SOCKS5 listener is not ready".into());
    }
    let pid = status.pid;
    let result = tauri::async_runtime::spawn_blocking(|| crate::egress::probe(crate::engine::DEFAULT_SOCKS_PORT))
        .await
        .map_err(|_| "Remote TCP probe failed to complete".to_string())?;
    let current = engine.status()?;
    if current.pid != pid || !current.socks_ready {
        return Err("Xray changed or stopped during the remote probe; result discarded".into());
    }
    Ok(result)
}

#[tauri::command]
pub fn stop_engine(
    engine: tauri::State<'_, crate::engine::Engine>,
) -> Result<crate::engine::EngineStatus, String> {
    engine.stop()
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
pub fn list_1vpn_servers(
    auth: tauri::State<'_, crate::auth::AuthState>,
) -> Result<Vec<crate::provider::PublicServerOption>, String> {
    auth.server_options()
}

#[tauri::command]
pub fn logout_1vpn(auth: tauri::State<'_, crate::auth::AuthState>) -> Result<(), String> {
    auth.logout()
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

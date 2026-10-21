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

/// Starts only an authenticated, eligible 1VPN server. SOCKS readiness is not HTTPS verification.
#[tauri::command]
pub fn connect_1vpn(
    app: tauri::AppHandle,
    auth: tauri::State<'_, crate::auth::AuthState>,
    engine: tauri::State<'_, crate::engine::Engine>,
    location_index: usize,
    server_index: usize,
) -> Result<crate::engine::EngineStatus, String> {
    use std::io::Write;
    use tauri::Manager;
    let selected = auth.selected_server(location_index, server_index)?;
    let config = crate::config::build_config(&selected, crate::engine::DEFAULT_SOCKS_PORT)
        .map_err(str::to_owned)?;
    let runtime = crate::xray::inspect_runtime();
    if !runtime.ready {
        return Err(runtime.message);
    }
    let executable = runtime.executable.ok_or("Verified Xray executable unavailable")?;
    let directory = app.path().app_local_data_dir().map_err(|_| "Cannot locate app data directory")?;
    std::fs::create_dir_all(&directory).map_err(|_| "Cannot prepare app data directory")?;
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| "System clock unavailable")?.as_nanos();
    let path = directory.join(format!("onetunnel-session-{}-{nonce}.json", std::process::id()));
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true)
        .open(&path).map_err(|_| "Cannot create session config")?;
    let result = (|| {
        serde_json::to_writer(&mut file, &config).map_err(|_| "Cannot serialize Xray config")?;
        file.flush().map_err(|_| "Cannot flush Xray config")?;
        drop(file);
        engine.start(std::path::Path::new(&executable), &path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&path);
    }
    result
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
pub fn logout_1vpn(
    auth: tauri::State<'_, crate::auth::AuthState>,
    engine: tauri::State<'_, crate::engine::Engine>,
) -> Result<(), String> {
    // Invalidate the session first, even if the owned Xray process cannot be stopped.
    // Never leave a valid authenticated session after an attempted logout.
    auth.logout()?;
    engine.stop()?;
    Ok(())
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

#[tauri::command]
pub fn get_diagnostic_events(logs: tauri::State<'_, crate::diagnostics::Diagnostics>) -> Result<Vec<crate::diagnostics::Event>, String> {
    logs.recent()
}

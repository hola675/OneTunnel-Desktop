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

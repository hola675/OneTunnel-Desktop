use serde::Serialize;
use tauri::Manager;

use crate::engine::{self, XrayPreflight};

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
        core_status: "not_initialized",
    }
}

#[tauri::command]
pub fn get_xray_preflight(app: tauri::AppHandle) -> XrayPreflight {
    match app.path().resource_dir() {
        Ok(resources) => engine::preflight(&resources),
        Err(_) => XrayPreflight {
            found: false,
            supported: false,
            executable: String::new(),
            reported_version: None,
            issue: Some("RESOURCE_DIR_UNAVAILABLE".into()),
        },
    }
}

mod commands;
mod config;
mod engine;
mod health;
mod integrity;
mod xray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(engine::Engine::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::inspect_xray_runtime,
            commands::get_engine_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running OneTunnel Desktop");
}

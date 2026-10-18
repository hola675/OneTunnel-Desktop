mod auth;
mod commands;
mod config;
mod engine;
mod health;
mod integrity;
mod provider;
mod routing;
mod xray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(engine::Engine::default())
        .manage(auth::AuthState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::inspect_xray_runtime,
            commands::get_engine_status,
            commands::login_1vpn,
            commands::get_account,
            commands::logout_1vpn,
            commands::preview_traffic_rule
        ])
        .run(tauri::generate_context!())
        .expect("error while running OneTunnel Desktop");
}

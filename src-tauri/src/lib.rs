mod auth;
mod capture;
mod commands;
mod config;
mod engine;
mod health;
mod integrity;
mod policy;
mod provider;
mod recovery;
mod routing;
mod xray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(engine::Engine::default())
        .manage(auth::AuthState::default())
        .manage(policy::PolicyStore::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::inspect_xray_runtime,
            commands::get_engine_status,
            commands::stop_engine,
            commands::login_1vpn,
            commands::get_account,
            commands::list_1vpn_servers,
            commands::logout_1vpn,
            commands::preview_traffic_rule,
            commands::get_traffic_policy,
            commands::save_traffic_policy,
            commands::preview_saved_traffic_policy,
            commands::get_capture_capability
        ])
        .run(tauri::generate_context!())
        .expect("error while running OneTunnel Desktop");
}

mod commands;
mod xray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::get_app_info, commands::inspect_xray_runtime])
        .run(tauri::generate_context!())
        .expect("error while running OneTunnel Desktop");
}

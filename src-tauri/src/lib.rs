use std::sync::Mutex;
use tauri::Manager;

mod storage;
mod crypto;
mod state;
mod commands;
mod totp;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(state::AppState::default()));
            Ok(())
        });
    #[cfg(mobile)]
    builder.plugin(tauri_plugin_barcode_scanner::init());
        
    builder
        .invoke_handler(tauri::generate_handler![
            commands::remove_service,
            commands::add_service,
            commands::setup_storage_keys,
            commands::get_services_tokens
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

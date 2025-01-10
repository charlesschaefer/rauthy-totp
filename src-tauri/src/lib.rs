use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_fs::FsExt;


mod commands;
mod crypto;
mod state;
mod storage;
mod totp;
mod brandfetch;

#[cfg(mobile)]
const IS_MOBILE: bool = true;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();
    builder = builder
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            app.manage(Mutex::new(state::AppState::default()));
            let path = app.path().app_local_data_dir().expect("Couldn't resolve app local data dir");//.join("Rauthy.bin");
            let scope = app.fs_scope();
            // scope.allow_file(path.clone()).unwrap();
            dbg!(scope.is_allowed(path.clone()));

            let app_state = app.state::<Mutex<state::AppState>>();
            let mut state = app_state.lock().unwrap();
            state.storage_path = path;

            Ok(())
        });

    #[cfg(mobile)]
    if IS_MOBILE {
        builder = builder.plugin(tauri_plugin_barcode_scanner::init());
    }

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

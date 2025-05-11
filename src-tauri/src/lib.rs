use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_fs::FsExt;

mod biometric;
mod brandfetch;
mod commands;
mod crypto;
#[cfg(desktop)]
mod desktop;
mod state;
mod storage;
mod totp;

#[cfg(mobile)]
const IS_MOBILE: bool = true;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();
    builder = builder.plugin(tauri_plugin_fs::init()).setup(|app| {
        #[cfg(desktop)]
        {
            desktop::setup_system_tray_icon(app);
        }

        let salt_path = app.
            path()
            .app_local_data_dir()
            .expect("Couldn't resolve app local data dir")
            .join("salt.txt");
        
        app.manage(Mutex::new(state::AppState::default()));
        let path = app
            .path()
            .app_local_data_dir()
            .expect("Couldn't resolve app local data dir"); //.join("Rauthy.bin");
        let scope = app.fs_scope();
        // scope.allow_file(path.clone()).unwrap();
        dbg!(scope.is_allowed(path.clone()));

        let app_state = app.state::<Mutex<state::AppState>>();
        let mut state = app_state.lock().unwrap();
        state.storage_path = path;

        #[cfg(debug_assertions)] // only include this code on debug builds
        {
            //let window = app.get_webview_window("main").unwrap();
            //window.open_devtools();
            //window.close_devtools();
        }

        Ok(())
    });

    #[cfg(mobile)]
    if IS_MOBILE {
        builder = builder
            .plugin(tauri_plugin_barcode_scanner::init())
            .plugin(tauri_plugin_biometric::init());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            commands::remove_service,
            commands::add_service,
            commands::setup_storage_keys,
            commands::get_services_tokens,
            commands::update_service,
            commands::delete_service,
            commands::get_service_icon,
            #[cfg(mobile)]
            commands::fetch_without_pass,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

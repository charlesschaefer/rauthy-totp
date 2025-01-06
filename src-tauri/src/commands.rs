use tauri::State;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::crypto::*;
use crate::storage::*;
use crate::state::AppState;


#[tauri::command]
pub fn setup_storage_keys(app_state: State<'_, Mutex<AppState>>, user_pass: &str) -> ServiceMap {
    let key = derive_key_from_password(user_pass).unwrap();
    let mut storage = Storage::new(key.to_vec());
    if storage.file_exists() {
        storage.read_from_file().unwrap();
    }

    let mut state = app_state.lock().unwrap();
    state.storage = storage;
    
    let services = state.storage.services().clone();

    services
}

#[tauri::command]
pub fn add_service(app_state: State<'_, Mutex<AppState>>, totp_uri: &str) -> ServiceMap {
    let mut state = app_state.lock().unwrap();

    state.storage.add_service(Service::from(totp_uri));

    let services = state.storage.services().clone();

    services
}

#[tauri::command]
pub fn remove_service(app_state: State<'_, Mutex<AppState>>, service_id: String) -> ServiceMap {
    let mut state = app_state.lock().unwrap();
    state.storage.remove_service(service_id);

    let services = state.storage.services().clone();

    services
}

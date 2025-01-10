use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

use crate::crypto::*;
use crate::state::AppState;
use crate::storage::*;
use crate::totp::*;

#[tauri::command]
pub fn setup_storage_keys(
    app_state: State<'_, Mutex<AppState>>,
    user_pass: &str,
) -> Result<ServiceMap, Error> {
    let key = derive_key_from_password(user_pass)?;

    let mut state = app_state.lock().unwrap();
    
    let mut storage = Storage::new(key.to_vec());
    storage.set_base_path(state.storage_path.clone());

    if storage.file_exists() {
        match storage.read_from_file() {
            Err(_) => return Err("Couldn't decrypt the storage file"),
            Ok(_) => {}
        }
    }
    
    state.storage = storage;

    let services = state.storage.services().clone();
    // println!("Services: {:?}", services);
    Ok(services)
}

#[tauri::command]
pub fn add_service(
    app_state: State<'_, Mutex<AppState>>,
    totp_uri: &str,
) -> Result<ServiceMap, ()> {
    let mut state = app_state.lock().unwrap();

    if let Ok(service) = Service::try_from(totp_uri) {
        state.storage.add_service(service);
        state.storage.save_to_file()?;

        let services = state.storage.services().clone();
        // println!("Services: {:?}", services);

        return Ok(services);
    } else {
        // println!("Couldn't create a service from the url");
        return Ok(std::collections::HashMap::new());
    }
}

#[tauri::command]
pub fn remove_service(
    app_state: State<'_, Mutex<AppState>>,
    service_id: String,
) -> Result<ServiceMap, ()> {
    let mut state = app_state.lock().unwrap();

    state.storage.remove_service(service_id);
    state.storage.save_to_file()?;

    let services = state.storage.services().clone();

    Ok(services)
}

#[tauri::command]
pub fn get_services_tokens(
    app_state: State<'_, Mutex<AppState>>,
) -> Result<HashMap<String, TotpToken>, ()> {
    let state = app_state.lock().unwrap();
    match state.storage.services_tokens() {
        Ok(tokens) => Ok(tokens),
        Err(_) => Err(()),
    }
}

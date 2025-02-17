use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;


use crate::crypto::*;
use crate::state::AppState;
use crate::storage::*;
use crate::totp::*;
#[cfg(mobile)]
use crate::biometric::*;

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
    match Service::try_from(totp_uri)  {
        Ok(service) => {
            state.storage.add_service(service);
            state.storage.save_to_file()?;
    
            let services = state.storage.services().clone();
            return Ok(services);
        },
        Err(_) => {
            return Ok(std::collections::HashMap::new());
        }
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

#[tauri::command]
pub fn update_service(
    app_state: State<'_, Mutex<AppState>>,
    service: Service,
) -> Result<(), ()> {
    let mut state = app_state.lock().unwrap();
    
    state.storage.update_service(service);
    state.storage.save_to_file()?;

    Ok(())
}


#[cfg(mobile)] #[tauri::command]
pub fn fetch_without_pass(
    app_state: State<'_, Mutex<AppState>>,
    app_handle: tauri::AppHandle,
    reason: String,
    options: AuthOptions
) -> Result<ServiceMap, Error> {
    use tauri_plugin_biometric::BiometricExt;
    let mut state = app_state.lock().unwrap();
    match app_handle.biometric().biometric_cipher(reason, options.try_into().unwrap()) {
        Ok(data) => {
            state.storage.set_key_access_pass(data.data)
        },
        Err(err) => {
            dbg!("Can't load biometric decrypted data.");
        }
    }

    let key = derive_key_from_password(state.storage.get_key_access_pass().as_str())?;
    let mut storage = Storage::new(key.to_vec());
    storage.set_base_path(state.storage_path.clone());

    if storage.file_exists() {
        match storage.read_from_file() {
            Err(_) => return Err("Couldn't decrypt the storage file"),
            Ok(_) => {}
        }
    }
    
    let services = storage.services().clone();
    state.storage = storage;
    Ok(services.clone())
}

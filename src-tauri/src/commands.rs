use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;
use zeroize::Zeroize;
use std::env;

use crate::brandfetch::search_brand;
use crate::crypto::*;
use crate::state::AppState;
use crate::storage::*;
use crate::totp::*;

#[cfg(mobile)]
use crate::biometric::*;

#[tauri::command]
pub fn setup_storage_keys(
    app_handle: tauri::AppHandle,
    app_state: State<'_, Mutex<AppState>>,
    user_pass: String,
) -> Result<ServiceMap, Error> {
    fetch_services_with_pass(app_handle, app_state, user_pass)
}

#[tauri::command]
pub fn add_service(
    app_handle: tauri::AppHandle,
    app_state: State<'_, Mutex<AppState>>,
    totp_uri: &str,
) -> Result<ServiceMap, ()> {
    let mut state = app_state.lock().unwrap();
    match Service::try_from(totp_uri) {
        Ok(service) => {
            state.storage.add_service(service);
            state.storage.save_to_file(&app_handle)?;

            let services = state.storage.services().clone();
            return Ok(services);
        }
        Err(_) => {
            return Ok(std::collections::HashMap::new());
        }
    }
}

#[tauri::command]
pub fn remove_service(
    app_handle: tauri::AppHandle,
    app_state: State<'_, Mutex<AppState>>,
    service_id: String,
) -> Result<ServiceMap, ()> {
    let mut state = app_state.lock().unwrap();

    state.storage.remove_service(service_id);
    state.storage.save_to_file(&app_handle)?;

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
    app_handle: tauri::AppHandle,
    app_state: State<'_, Mutex<AppState>>,
    service: Service,
) -> Result<(), ()> {
    let mut state = app_state.lock().unwrap();

    state.storage.update_service(service);
    state.storage.save_to_file(&app_handle)?;

    Ok(())
}

#[tauri::command]
pub fn delete_service(
    app_handle: tauri::AppHandle,
    app_state: State<'_, Mutex<AppState>>,
    service_id: String,
) -> Result<(), String> {
    let mut state = app_state.lock().unwrap();
    if state.storage.remove_service(service_id) {
        state
            .storage
            .save_to_file(&app_handle)
            .map_err(|_| "Failed to save storage".to_string())?;
        Ok(())
    } else {
        Err("Service not found".to_string())
    }
}

#[tauri::command]
pub fn get_service_icon(
    app_handle: tauri::AppHandle,
    app_state: State<'_, Mutex<AppState>>,
    service_id: String,
) -> Result<String, ()> {
    let mut state = app_state.lock().unwrap();
    let cloned_services = state.storage.services().clone();
    let mut service = cloned_services.get(service_id.as_str()).unwrap().clone();

    let client_id = env!(
        "BRANDFETCH_USER_ID",
        "Brandfetch user_id env var not defined"
    );

    match search_brand(&service.issuer.as_str(), client_id) {
        Ok(brands) => {
            if brands.len() > 0 {
                service.icon = brands.first().unwrap().icon.clone();
            }
        }
        Err(err) => {
            service.icon = "".to_string();
            dbg!("Error searching brand logo: {}", err);
        }
    }
    state.storage.add_service(service.clone());
    state.storage.save_to_file(&app_handle).ok();

    Ok(service.icon.clone())
}

#[tauri::command]
pub fn export_services_csv(
    app_state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let state = app_state.lock().unwrap();
    let services = state.storage.services();
    
    if services.is_empty() {
        return Err("No services to export".to_string());
    }

    let mut csv_content = String::new();
    
    // CSV header
    csv_content.push_str("Issuer,Name,Secret,Algorithm,Digits,Period,Icon\n");
    
    // CSV data rows
    for service in services.values() {
        let algorithm_str = match service.algorithm {
            totp_rs::Algorithm::SHA1 => "SHA1",
            totp_rs::Algorithm::SHA256 => "SHA256", 
            totp_rs::Algorithm::SHA512 => "SHA512",
        };
        
        // Escape CSV fields that contain commas or quotes
        let issuer = escape_csv_field(&service.issuer);
        let name = escape_csv_field(&service.name);
        let secret = escape_csv_field(&service.secret);
        let icon = escape_csv_field(&service.icon);
        
        csv_content.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            issuer,
            name,
            secret,
            algorithm_str,
            service.digits,
            service.period,
            icon
        ));
    }
    
    Ok(csv_content)
}

fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        let escaped = field.replace("\"", "\"\"");
        format!("\"{}\"", escaped)
    } else {
        field.to_string()
    }
}

#[tauri::command]
pub fn import_services_csv(
    app_handle: tauri::AppHandle,
    app_state: State<'_, Mutex<AppState>>,
    csv_content: String,
) -> Result<ServiceMap, String> {
    let mut state = app_state.lock().unwrap();
    let mut imported_count = 0;
    let mut errors = Vec::new();

    let lines: Vec<&str> = csv_content.lines().collect();
    if lines.is_empty() {
        return Err("CSV file is empty".to_string());
    }

    // Skip header line
    for (line_num, line) in lines.iter().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }

        let fields: Vec<String> = parse_csv_line(line);
        if fields.len() < 7 {
            errors.push(format!("Line {}: Invalid CSV format, expected 7 fields", line_num + 1));
            continue;
        }

        // Parse algorithm
        let algorithm = match fields[3].to_uppercase().as_str() {
            "SHA1" => totp_rs::Algorithm::SHA1,
            "SHA256" => totp_rs::Algorithm::SHA256,
            "SHA512" => totp_rs::Algorithm::SHA512,
            _ => {
                errors.push(format!("Line {}: Invalid algorithm '{}'", line_num + 1, fields[3]));
                continue;
            }
        };

        // Parse digits
        let digits = match fields[4].parse::<usize>() {
            Ok(d) => d,
            Err(_) => {
                errors.push(format!("Line {}: Invalid digits '{}'", line_num + 1, fields[4]));
                continue;
            }
        };

        // Parse period
        let period = match fields[5].parse::<u64>() {
            Ok(p) => p,
            Err(_) => {
                errors.push(format!("Line {}: Invalid period '{}'", line_num + 1, fields[5]));
                continue;
            }
        };

        // Create service
        let mut service = crate::storage::Service::default();
        service.issuer = fields[0].clone();
        service.name = fields[1].clone();
        service.secret = fields[2].clone();
        service.algorithm = algorithm;
        service.digits = digits;
        service.period = period;
        service.icon = fields[6].clone();
        service.id = format!("{}{}", service.issuer, service.name);

        state.storage.add_service(service);
        imported_count += 1;
    }

    if imported_count == 0 {
        return Err(format!("No valid services imported. Errors: {}", errors.join("; ")));
    }

    // Save the updated storage
    state.storage.save_to_file(&app_handle).map_err(|_| "Failed to save storage".to_string())?;

    let services = state.storage.services().clone();
    Ok(services)
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    // Escaped quote
                    current_field.push('"');
                    chars.next(); // Skip the second quote
                } else {
                    // Toggle quote state
                    in_quotes = !in_quotes;
                }
            }
            ',' => {
                if in_quotes {
                    current_field.push(c);
                } else {
                    fields.push(current_field.trim().to_string());
                    current_field.clear();
                }
            }
            _ => {
                current_field.push(c);
            }
        }
    }
    
    // Add the last field
    fields.push(current_field.trim().to_string());
    fields
}

#[tauri::command]
pub fn change_password(
    app_handle: tauri::AppHandle,
    app_state: State<'_, Mutex<AppState>>,
    mut new_password: String,
) -> Result<(), String> {
    let mut state = app_state.lock().unwrap();
    
    // Get current services from already decrypted storage
    let services = state.storage.services().clone();
    
    if services.is_empty() {
        return Err("No services to migrate".to_string());
    }

    // Create backup of current file
    let current_path = state.storage.storage_path(&app_handle);
    let backup_path = current_path.with_extension("bin.backup");
    
    if current_path.exists() {
        std::fs::copy(&current_path, &backup_path)
            .map_err(|_| "Failed to create backup file".to_string())?;
    }

    // Generate new salt and key
    let new_salt = generate_salt();
    let new_key = derive_key_from_password_and_salt(&new_password, Some(&new_salt))
        .map_err(|_| "Failed to generate new key".to_string())?;

    // Create new storage with new password
    let mut new_storage = Storage::new(new_key.to_vec(), Some(new_salt));
    
    // Copy all services to new storage
    for (_, service) in services {
        new_storage.add_service(service);
    }

    // Save with new password
    new_storage.save_to_file(&app_handle)
        .map_err(|_| "Failed to save with new password".to_string())?;

    // Update app state
    state.storage = new_storage;

    // Clear password from memory
    new_password.clear();

    Ok(())
}

#[tauri::command]
pub fn close_services_file(
    app_state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let mut state = app_state.lock().unwrap();
    
    // Clear the storage and reset to default state
    state.storage = crate::storage::Storage::new(Vec::new(), None);
    
    Ok(())
}

#[cfg(mobile)]
#[tauri::command]
pub fn fetch_without_pass(
    app_handle: tauri::AppHandle,
    app_state: State<'_, Mutex<AppState>>,
    reason: String,
    options: AuthOptions,
) -> Result<ServiceMap, Error> {
    use tauri_plugin_biometric::BiometricExt;
    match app_handle
        .biometric()
        .biometric_cipher(reason, options.try_into().unwrap())
    {
        Ok(data) => fetch_services_with_pass(app_handle, app_state, data.data),
        Err(_) => {
            dbg!("Can't load biometric decrypted data.");
            Err("Can't load biometric decrypted data.".into())
        }
    }

}

pub fn fetch_services_with_pass(app_handle: tauri::AppHandle, app_state: State<'_, Mutex<AppState>>, mut user_pass: String) -> Result<ServiceMap, Error> {
    let mut state = app_state.lock().unwrap();

    // Tries to generate a key in the old format, with hardcoded salt
    // This is for backwards compatibility with older versions
    let key = derive_key_from_password_and_salt(user_pass.as_str(), None)?;
    let mut storage = Storage::new(key.to_vec(), None);

    if storage.file_exists(&app_handle) {
    
        match storage.read_from_file(&app_handle) {
            Err(_) => {
                // As we couldn't read the file using the old format, we will try to read it using the new format
                // Reading the salt from the file
                let salt = storage.read_salt_from_file(&app_handle).unwrap();
                let key = derive_key_from_password_and_salt(user_pass.as_str(), Some(&salt))?;
                storage = Storage::new(key.to_vec(), Some(salt));

                match storage.read_from_file(&app_handle) {
                    Err(_) => return Err("Couldn't decrypt the storage file".into()),
                    Ok(_) => {}                
                }
            }
            Ok(_) => {}
        }

        // Now that we already read the file, we will generate a new salt and key, and change the key for encryption
        let new_salt = crate::crypto::generate_salt();
        let new_key = crate::crypto::derive_key_from_password_and_salt(user_pass.as_str(), Some(&new_salt))?;
        storage.set_new_key_and_salt(new_key.to_vec(), new_salt);
        storage.save_to_file(&app_handle).unwrap();

    } else {
        // If this is a new file, we will generate a new salt
        let salt = crate::crypto::generate_salt();
        let key = crate::crypto::derive_key_from_password_and_salt(user_pass.as_str(), Some(&salt))?;
        storage = Storage::new(key.to_vec(), Some(salt));
    }

    // clear the user_pass in memory
    // This is important for security reasons
    // to prevent the password from being stored in memory longer than necessary
    user_pass.zeroize();

    state.storage = storage;

    let services = state.storage.services().clone();
    Ok(services)
}

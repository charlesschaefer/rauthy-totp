use bincode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;
use totp_rs::{Algorithm, Secret, TOTP};
use url::Url;

use crate::crypto::*;
use crate::totp::*;
use crate::brandfetch::*;

const STORAGE_FILE: &str = "Rauthy.bin";
type Error = &'static str;

// TOTP URI format:
// otpauth://totp/{issuer}:{displayUserName}?secret={secretKey}&issuer={issuer}&algorithm={algorithm}&digits={digits}&period={period}
// where:
// {issuer}, {displayUserName}, {algorithm}, {digits} and {period} are optional fields.
// {secretKey} -> a required base32-encoded String with at least 128bits (16bytes) to be used as
//                a shared secret between the service and the app
// {algorithm} -> sha1, sha256 or sha512
// {digits}    -> 6 or 8. The number of characters (digits) the generated OTP code must have
// {period}    -> The number of seconds during which the OTP code will be valid. DEFAULT: 30, but it can be any valid integer value.
//

// otpauth://totp/csinfotest?secret=ZEH7IWIVJ7Q65KF7EQPEVDQ5JTATNNPM&issuer=Namecheap+-+PHX01BSB137, results in:
// {issuer}     -> csinforest first, then Namecheap+-+PHX01BSB137
// {displayUserName} -> Nothing
// {secretKey}  -> ZEH7IWIVJ7Q65KF7EQPEVDQ5JTATNNPM
// {algorithm}  -> sha1 (the default value)
// {digits}     -> 6 (the default value)
// {period}     -> 30 (the default value)

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Service {
    pub id: String,
    pub issuer: String,
    pub secret: String,
    pub name: String,
    pub algorithm: Algorithm,
    pub digits: usize,
    pub period: u64,
    pub icon: String, // icon url
}

impl Default for Service {
    fn default() -> Self {
        Self {
            id: String::from(""),
            issuer: String::from(""),
            secret: String::from(""),
            name: String::from(""),
            algorithm: Algorithm::SHA1,
            digits: 6,
            period: 30,
            icon: String::from("")
        }
    }
}

impl Service {
    pub fn new(parsable_uri: &str) -> Result<Self, ()> {
        match Url::parse(parsable_uri) {
            Ok(uri) => match Self::try_from(uri) {
                Ok(result) => Ok(result),
                Err(_) => Err(()),
            },
            Err(_) => Err(()),
        }
    }
}

impl TryFrom<TOTP> for Service {
    type Error = Error;

    fn try_from(totp: TOTP) -> Result<Self, Self::Error> {
        if totp.account_name.len() == 0 {
            return Err("Invalid TOTP instance");
        }
        let mut service = Service::default();

        service.name = totp.account_name.clone();
        service.issuer = totp.issuer.clone().unwrap();

        service.id = service.issuer.clone();
        service.id.push_str(service.name.as_str());

        service.secret = Secret::Raw(totp.secret.clone()).to_encoded().to_string();

        service.algorithm = totp.algorithm;
        service.digits = totp.digits;
        service.period = totp.step;

        // @TODO: set the client_id here
        let client_id = env!("BRANDFETCH_USER_ID", "Brandfetch user_id env var not defined");
        match search_brand(service.issuer.as_str(), client_id) {
            Ok(brands) => {
                if brands.len() > 0 {
                    service.icon = brands.first().unwrap().icon.clone();
                }
            },
            Err(err) => {
                dbg!("Error searching brand logo: {}", err);
            }
        }

        Ok(service)
    }
}

impl TryFrom<Url> for Service {
    type Error = Error;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        match TOTP::from_url(url.as_str()) {
            Ok(totp) => Service::try_from(totp),
            Err(_) => Err("Couldn't create a service from this URL"),
        }
    }
}

impl TryFrom<&str> for Service {
    type Error = Error;

    fn try_from(url: &str) -> Result<Self, Self::Error> {
        match TOTP::from_url(url) {
            Ok(totp) => return Service::try_from(totp),
            Err(_) => {
                match TOTP::from_url_unchecked(url) {
                    Ok(totp) => return Service::try_from(totp),
                    Err(_) => {
                        return Err("Couldn't parse the provided URL as a TOTP URL");
                    }
                }
            }
        }
    }
}

impl ServiceToken for Service {
    fn current_totp(&self) -> Result<TotpToken, Error> {
        let totp;
        match TOTP::new(
            self.algorithm,
            self.digits,
            1,
            self.period,
            Secret::Encoded(self.secret.clone()).to_bytes().unwrap(),
            Some(self.issuer.clone()),
            self.name.clone(),
        ) {
            Err(err) => {
                dbg!("Error trying to create normal TOTP. We'll try to create unchecked", err);
                totp = TOTP::new_unchecked(
                    self.algorithm,
                    self.digits,
                    1,
                    self.period,
                    Secret::Encoded(self.secret.clone()).to_bytes().unwrap(),
                    Some(self.issuer.clone()),
                    self.name.clone(),
                );
            }
            Ok(_totp) => totp = _totp
        }

        match totp.generate_current() {
            Ok(token) => Ok(TotpToken {
                token,
                next_step_time: totp.next_step_current().unwrap(),
            }),
            Err(_) => Err("Couldn't generate a token based on the current time"),
        }
    }
}

pub type ServiceMap = HashMap<String, Service>;

#[derive(Default)]
pub struct Storage {
    services: ServiceMap,
    signing_key: Vec<u8>,
    file_path: String,
    key_access_pass: String,
}

impl Storage {
    pub fn new(key: Vec<u8>) -> Self {
        if key.len() == 0 {
            panic!("Key for data storing can't be empty");
        }

        // let mut file_path = std::env::current_dir().unwrap();
        // file_path.push(STORAGE_FILE);
        let file_path = tauri_plugin_fs::FilePath::from_str("$APPLOCALDATA").unwrap().as_path().unwrap().join(STORAGE_FILE);
        
        let file_path_str = file_path.to_string_lossy().to_string();

        Self {
            services: HashMap::new(),
            signing_key: key,
            file_path: file_path_str,
            key_access_pass: String::new(),
        }
    }

    pub fn file_exists(&self) -> bool {
        std::fs::exists(self.file_path.clone()).unwrap()
    }

    pub fn save_to_file(&self) -> Result<(), ()> {
        let serialized_services = bincode::serialize(&self.services).unwrap(); // Serialize the services
        let key = self.signing_key.clone();
        match encrypt_data(serialized_services, key.as_slice()) {
            // Encrypt the serialized data
            Ok(encrypted_data) => {
                let mut file = File::create(&self.file_path).unwrap(); // Create or open the file
                file.write_all(&encrypted_data).unwrap_or_default(); // Write the encrypted data to the file
                Ok(())
            }
            Err(_) => Err(()),
        }
    }

    pub fn read_from_file(&mut self) -> Result<(), ()> {
        let mut file = File::open(&self.file_path).unwrap(); // Open the file
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data).unwrap_or_default(); // Read the encrypted data

        let key = self.signing_key.clone();
        match decrypt_data(encrypted_data, key.as_slice()) {
            // Decrypt the data
            Ok(decrypted_data) => {
                self.services = bincode::deserialize(&decrypted_data).unwrap();
                return Ok(());
            }
            Err(_) => Err(()),
        }
    }

    pub fn services(&self) -> &ServiceMap {
        &self.services
    }
    
    pub fn set_base_path(&mut self, path: std::path::PathBuf) {
        self.file_path = String::from(path.join(STORAGE_FILE).to_str().unwrap());
    }
    
    pub fn add_service(&mut self, service: Service) {
        self.services.insert(service.id.clone(), service); // Add the new service to the inner vector
    }
    
    pub fn update_service(&mut self, service: Service) {
        self.services.insert(service.id.clone(), service);
    }
    
    pub fn remove_service(&mut self, id: String) -> bool {
        if let Some(_) = self.services.remove(&id) {
            return true;
        }
        return false;
    }

    pub fn set_key_access_pass(&mut self, code: String) {
        self.key_access_pass = code;
    }

    pub fn get_key_access_pass(&self) -> String {
        self.key_access_pass.clone()
    }
}

impl ServicesTokens for Storage {
    fn services_tokens(&self) -> Result<HashMap<String, TotpToken>, ()> {
        let mut tokens = std::collections::HashMap::new();
        for (key, val) in self.services.iter() {
            tokens.insert(key.clone(), val.current_totp().unwrap());
        }
        if tokens.len() == self.services.len() {
            return Ok(tokens);
        }
        return Err(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_storage() -> Storage {
        let key = vec![0; 32]; // Example key
        Storage::new(key)
    }

    #[test]
    fn test_add_service() {
        let mut storage = setup_storage();
        let service = Service::default();
        storage.add_service(service.clone());
        assert_eq!(storage.services.len(), 1);
        assert!(storage.services.contains_key(&service.id));
        // Clean up the created file
        let _ = fs::remove_file(&storage.file_path);
    }

    #[test]
    fn test_remove_service() {
        let mut storage = setup_storage();
        let service = Service::default();
        storage.add_service(service.clone());
        assert!(storage.remove_service(service.id.clone()));
        assert!(!storage.services.contains_key(&service.id));
        // Clean up the created file
        let _ = fs::remove_file(&storage.file_path);
    }

    #[test]
    fn test_file_exists() {
        let storage = setup_storage();
        assert!(!storage.file_exists()); // Should be false since the file does not exist yet
                                         // Clean up the created file
        let _ = fs::remove_file(&storage.file_path);
    }

    #[test]
    fn test_save_to_file() {
        let mut storage = setup_storage();
        let service = Service::default();
        storage.add_service(service.clone());
        let result = storage.save_to_file();
        assert!(result.is_ok());
        println!("Path of file saved: {:?}", storage.file_path);
        println!(
            "File exists: {}",
            std::fs::exists(storage.file_path.to_string()).unwrap()
        );

        // Check if the file exists after saving
        assert!(storage.file_exists());

        // Clean up the created file
        let _ = fs::remove_file(&storage.file_path);
    }

    #[test]
    fn test_read_from_file() {
        let mut storage = setup_storage();
        let service = Service::default();
        storage.add_service(service.clone());
        let result = storage.save_to_file();
        assert!(result.is_ok());
        println!("Path of file saved: {:?}", storage.file_path);

        let mut new_storage = setup_storage();
        println!("Path of file to test: {:?}", new_storage.file_path);
        println!(
            "File exists: {}",
            std::fs::exists(new_storage.file_path.to_string()).unwrap()
        );

        // Check if the file exists after saving
        assert!(new_storage.file_exists());

        new_storage.read_from_file().unwrap();
        assert_eq!(new_storage.services.len(), 1);
        assert!(new_storage.services.contains_key(&service.id));

        // Clean up the created file
        let _ = fs::remove_file(&new_storage.file_path);
    }

    #[test]
    fn test_service_from_url() {
        // otpauth://totp/csinfotest?secret=ZEH7IWIVJ7Q65KF7EQPEVDQ5JTATNNPM&issuer=Namecheap+-+PHX01BSB137
        //let url = "otpauth://totp/testIssuer:testName?secret=ZEH7IWIVJ7Q65KF7EQPEVDQ5JTATNNPM";
        let url = "otpauth://totp/GitHub:constantoine@github.com?secret=KRSXG5CTMVRXEZLUKN2XAZLSKNSWG4TFOQ&issuer=GitHub";
        let service = Service::try_from(url).unwrap();
        assert_eq!(service.issuer, "GitHub");
        assert_eq!(service.name, "constantoine@github.com");
        assert_eq!(service.secret, "KRSXG5CTMVRXEZLUKN2XAZLSKNSWG4TFOQ");
    }

    #[test]
    fn test_service_default() {
        let service = Service::default();
        assert_eq!(service.algorithm, Algorithm::SHA1);
        assert_eq!(service.digits, 6);
        assert_eq!(service.period, 30);
    }
}

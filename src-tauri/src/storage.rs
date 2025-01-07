use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use url::Url;
use std::fs::File;
use std::io::{self, Write, Read};
use bincode;

use crate::crypto::*;


const STORAGE_FILE: &str = "Rauthy.bin";

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

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub enum Algorithm {
    #[default]
    SHA1,
    SHA256,
    SHA512,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Service {
    id: String,
    issuer: String,
    secret: String,
    name: String,
    algorithm: Algorithm,
    digits: u16,
    period: isize
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
            period: 30
        }
    }
}

impl Service {
    pub fn new(parsable_uri: &str) -> Result<Self, ()> {
        match Url::parse(parsable_uri) {
            Ok(uri) => {
                
                match Self::try_from(uri) {
                    Ok(result) => Ok(result),
                    Err(err) => Err(())
                }
            }
            Err(_) => {
                Err(())
            }
        }
    }
}

impl TryFrom<Url> for Service {
    type Error = &'static str;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        let mut service = Service::default();
        
        let path_parts: Vec<&str> = url.path().split(':').collect();
        let mut issuer = path_parts[0];
        if issuer.chars().nth(0) == Some('/') {
            let mut issuer_chars = issuer.chars();
            issuer_chars.next();
            issuer = issuer_chars.as_str();
        }
        println!("Parts: {:?}", path_parts);
        if path_parts.len() > 1 {
            service.name = String::from(path_parts[1]);
            service.issuer = String::from(issuer);
        } else {
            service.name = String::from(issuer);
        }
        service.id = service.issuer.clone();
        service.id.push_str(service.name.as_str());

        let mut query = url.query_pairs();
        if let Some((Cow::Borrowed(key), Cow::Borrowed(secret_key))) = query.next() {
            if key == "secret" {
                service.secret = String::from(secret_key);
            }
        }

        

        // TODO: capture period, digits and algorithm

        Ok(service)
    }
}

impl TryFrom<&str> for Service {
    type Error = &'static str;

    fn try_from(url: &str) -> Result<Self, Self::Error> {
        if let Ok(url) = Url::parse(url) {
            return Service::try_from(url);
        } else {
            return Err("Couldn't parse the provided URL");
        }
    }
}

pub type ServiceMap = HashMap<String, Service>;

#[derive(Default)]
pub struct Storage {
    services: ServiceMap,
    signing_key: Vec<u8>,
    file_path: String,
}

impl Storage {
    pub fn new(key: Vec<u8>) -> Self {
        if key.len() == 0 {
            panic!("Key for data storing can't be empty");
        }
        
        let mut file_path = std::env::current_dir().unwrap();
        file_path.push(STORAGE_FILE);

        Self {
            services: HashMap::new(),
            signing_key: key,
            file_path: String::from(file_path.to_str().unwrap())
        }
    }

    pub fn file_exists(&self) -> bool {
        std::fs::exists(self.file_path.to_string()).unwrap()
    }

    pub fn save_to_file(&self) -> Result<(), ()> {
        let serialized_services = bincode::serialize(&self.services).unwrap(); // Serialize the services
        let key = self.signing_key.clone();
        match encrypt_data(serialized_services, key.as_slice()) { // Encrypt the serialized data
            Ok(encrypted_data) => {
                let mut file = File::create(&self.file_path).unwrap(); // Create or open the file
                file.write_all(&encrypted_data).unwrap_or_default(); // Write the encrypted data to the file
                Ok(())
            },
            Err(_) => Err(())
        }
    }

    pub fn read_from_file(&mut self) -> Result<(), ()> {
        let mut file = File::open(&self.file_path).unwrap(); // Open the file
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data).unwrap_or_default(); // Read the encrypted data

        let key = self.signing_key.clone();
        match decrypt_data(encrypted_data, key.as_slice()) { // Decrypt the data
            Ok(decrypted_data) => {
                self.services = bincode::deserialize(&decrypted_data).unwrap();
                return Ok(());
            },
            Err(_) => Err(())
        }
    }

    pub fn add_service(&mut self, service: Service) {
        self.services.insert(service.id.clone(), service); // Add the new service to the inner vector
    }

    pub fn remove_service(&mut self, id: String) -> bool {
        if let Some(_) = self.services.remove(&id) {
            return true;
        } 
        return false;
    }

    pub fn services(&self) -> &ServiceMap {
        &self.services
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
    }

    #[test]
    fn test_remove_service() {
        let mut storage = setup_storage();
        let service = Service::default();
        storage.add_service(service.clone());
        assert!(storage.remove_service(service.id.clone()));
        assert!(!storage.services.contains_key(&service.id));
    }

    #[test]
    fn test_file_exists() {
        let storage = setup_storage();
        assert!(!storage.file_exists()); // Should be false since the file does not exist yet
    }

    #[test]
    fn test_save_to_file() {
        let mut storage = setup_storage();
        let service = Service::default();
        storage.add_service(service.clone());
        let result = storage.save_to_file();
        assert!(result.is_ok());
        println!("Path of file saved: {:?}", storage.file_path);
        println!("File exists: {}", std::fs::exists(storage.file_path.to_string()).unwrap());

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
        println!("File exists: {}", std::fs::exists(new_storage.file_path.to_string()).unwrap());
        
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
        let url = "otpauth://totp/testIssuer:testName?secret=TESTSECRET";
        let service = Service::try_from(url).unwrap();
        assert_eq!(service.issuer, "testIssuer");
        assert_eq!(service.name, "testName");
        assert_eq!(service.secret, "TESTSECRET");
    }

    #[test]
    fn test_service_default() {
        let service = Service::default();
        assert_eq!(service.algorithm, Algorithm::SHA1);
        assert_eq!(service.digits, 6);
        assert_eq!(service.period, 30);
    }
}

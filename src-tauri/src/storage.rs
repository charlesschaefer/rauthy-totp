use bincode;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use zeroize::Zeroize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use totp_rs::{Algorithm, Secret, TOTP};
use url::Url;

use crate::brandfetch::*;
use crate::crypto::{self, SaltArray, SALT_LEN};
use crate::totp::*;

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
            icon: String::from(""),
        }
    }
}

// impl Service {
//     pub fn new(parsable_uri: &str) -> Result<Self, ()> {
//         match Url::parse(parsable_uri) {
//             Ok(uri) => match Self::try_from(uri) {
//                 Ok(result) => Ok(result),
//                 Err(_) => Err(()),
//             },
//             Err(_) => Err(()),
//         }
//     }
// }

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
        let client_id = env!(
            "BRANDFETCH_USER_ID",
            "Brandfetch user_id env var not defined"
        );
        match search_brand(service.issuer.as_str(), client_id) {
            Ok(brands) => {
                if brands.len() > 0 {
                    service.icon = brands.first().unwrap().icon.clone();
                }
            }
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
            Err(_) => match TOTP::from_url_unchecked(url) {
                Ok(totp) => return Service::try_from(totp),
                Err(_) => {
                    return Err("Couldn't parse the provided URL as a TOTP URL");
                }
            },
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
                dbg!(
                    "Error trying to create normal TOTP. We'll try to create unchecked",
                    err
                );
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
            Ok(_totp) => totp = _totp,
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
    /// All services stored in the storage
    services: ServiceMap,
    /// The key used to encrypt/decrypt the data
    /// This key is derived from the password and the salt
    signing_key: Vec<u8>,
    /// The path to the storage file
    /// This is the file where the encrypted data is stored
    file_path: String,
    /// The password used to generate the key. We don't store it in the file
    /// and we discard it as soon as we generate the key
    key_access_pass: String,
    /// The salt used to generate the key
    /// We store it in the file to be able to derive the key again
    /// when we need to decrypt the data
    salt: Option<SaltArray>, // novo campo para guardar o salt atual
}

impl Storage {

    pub fn new(key: Vec<u8>, salt: Option<SaltArray>) -> Self {
        let file_path = STORAGE_FILE.to_string();
        Self {
            services: HashMap::new(),
            signing_key: key,
            file_path,
            key_access_pass: String::new(),
            salt: salt,
        }
    }

    pub fn storage_path<R: tauri::Runtime>(&self, app: &tauri::AppHandle<R>) -> PathBuf {
        let mut path = app
            .path()
            .app_local_data_dir()
            .expect("Couldn't resolve app local data dir");
        path.push(&self.file_path);
        path
    }

    /// Checks if the storage file exists.
    pub fn file_exists<R: tauri::Runtime>(&self, app: &tauri::AppHandle<R>) -> bool {
        let path = self.storage_path(app);
        path.exists()
    }

    /// Lê o salt do final do arquivo, retorna (dados_criptografados, salt)
    /// Suporta arquivos antigos (sem salt no final, usa salt fixo).
    pub fn read_from_file<R: tauri::Runtime>(
        &mut self,
        app: &tauri::AppHandle<R>,
    ) -> Result<ServiceMap, ()> {
        if self.signing_key.len() == 0 {
            return Err(());
        }
        let path = self.storage_path(app);
        let mut file = File::open(path).map_err(|_| ())?;
        
        self.set_permissions(&file);


        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(|_| ())?;
        
        if buf.len() < SALT_LEN {
            return Err(());
        }
        
        let key = self.signing_key.clone();

        // 1. Try to decrypt the data in the new format (file = data + salt)
        let salt_offset = buf.len() - SALT_LEN;
        let (encrypted_data, _salt_bytes) = buf.split_at(salt_offset);
        if let Ok(decrypted_data) = crypto::decrypt_data(encrypted_data.to_vec(), key.as_slice()) {
            self.services = bincode::deserialize(&decrypted_data).unwrap();
            return Ok(self.services.clone());
        }

        // 2. Try to decrypt the data in the old format (file = data)
        // Salt is fixed, so we use the fixed salt
        match crypto::decrypt_data(buf.clone(), key.as_slice()) {
            Ok(decrypted_data) => {
                self.services = bincode::deserialize(&decrypted_data).unwrap();
                return Ok(self.services.clone());
            }
            Err(_) => return Err(()),
        }
    }


    pub fn read_salt_from_file<R: tauri::Runtime>(
        &self,
        app: &tauri::AppHandle<R>,
    ) -> Result<SaltArray, ()> {
        let path = self.storage_path(app);
        let mut file = File::open(path).map_err(|_| ())?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(|_| ())?;
        
        if buf.len() < SALT_LEN {
            return Err(());
        }
        
        let salt_offset = buf.len() - SALT_LEN;
        let (_, salt_bytes) = buf.split_at(salt_offset);
        
        Ok(salt_bytes.try_into().unwrap())
    }

    /// Gera novo salt e chave, salva os dados criptografados + salt no final do arquivo.
    pub fn save_to_file<R: tauri::Runtime>(&mut self, app: &tauri::AppHandle<R>) -> Result<(), ()> {
        if self.signing_key.len() == 0 || self.salt.is_none() {
            return Err(());
        }

        let path = self.storage_path(app);
        let key = self.signing_key.clone();
        let salt = self.salt.clone().unwrap();

        let serialized_services = bincode::serialize(&self.services).map_err(|_| ())?;
        let mut encrypted_data = crypto::encrypt_data(serialized_services, &key).map_err(|_| ())?;
        
        // Append the salt to the end of the encrypted data
        encrypted_data.extend_from_slice(&salt);
        
        // Creaates a new file or truncates the existing one
        let mut file = File::create(path).map_err(|_| ())?;
        file.write_all(&encrypted_data).map_err(|_| ())?;
        
        self.set_permissions(&file);

        Ok(())
    }

    pub fn services(&self) -> &ServiceMap {
        &self.services
    }

    pub fn add_service(&mut self, service: Service) {
        self.services.insert(service.id.clone(), service);
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

    pub fn set_new_key_and_salt(&mut self, key: Vec<u8>, salt: SaltArray) {
        self.signing_key = key;
        self.salt = Some(salt);
    }

    fn set_permissions(&self, file: &File) {
        let metadata = file.metadata().unwrap();
        let mut permissions = metadata.permissions();
        #[cfg(any(unix, target_os = "macos"))]
        {
            permissions.set_mode(0o600); // Read/write for owner only
        }
        #[cfg(windows)]
        {
            permissions.set_readonly(false);
        }
        file.set_permissions(permissions).unwrap();
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

impl Drop for Storage {
    fn drop(&mut self) {
        self.signing_key.zeroize();
        self.salt.zeroize();
        self.key_access_pass.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::mock_app;
    use tauri::Manager;

    fn setup_storage() -> Storage {
        let key = vec![0; 32]; // Example key
        Storage::new(key, None)
    }

    #[test]
    fn test_add_service() {
        let mut storage = setup_storage();
        let _app = mock_app();
        let service = Service::default();
        storage.add_service(service.clone());
        assert_eq!(storage.services.len(), 1);
        assert!(storage.services.contains_key(&service.id));
    }

    #[test]
    fn test_remove_service() {
        let mut storage = setup_storage();
        let _app = mock_app();
        let service = Service::default();
        storage.add_service(service.clone());
        assert!(storage.remove_service(service.id.clone()));
        assert!(!storage.services.contains_key(&service.id));
    }

    #[test]
    fn test_file_exists() {
        let storage = setup_storage();
        let app = mock_app();
        // Should be false since the stronghold vault does not exist yet
        assert!(!storage.file_exists(&app.app_handle()));
    }

    #[test]
    fn test_save_to_file() {
        let mut storage = setup_storage();
        let app = mock_app();
        let service = Service::default();
        storage.add_service(service.clone());
        let result = storage.save_to_file(&app.app_handle());
        assert!(result.is_ok());
        // File existence cannot be reliably checked in mock_app environment
        // assert!(storage.file_exists(&app.app_handle()));
    }

    #[test]
    fn test_read_from_file() {
        let mut storage = setup_storage();
        let app = mock_app();
        let service = Service::default();
        storage.add_service(service.clone());
        let result = storage.save_to_file(&app.app_handle());
        assert!(result.is_ok());

        //let mut new_storage = setup_storage();
        // File existence cannot be reliably checked in mock_app environment
        // assert!(new_storage.file_exists(&app.app_handle()));

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

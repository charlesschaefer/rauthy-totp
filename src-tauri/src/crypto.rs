use aes_gcm::{
    aead::{Aead, AeadCore, OsRng, Payload},
    Aes256Gcm, Key, KeyInit, Nonce,
};

use ring::{digest, pbkdf2};
use rand::RngCore;
use std::num::NonZeroU32;

const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
pub const SALT_LEN: usize = 32;
pub const SALT: &str = "E3D0C30656C194272C7B6AD2ED0B7F8078FF2921F777A142A045D45931BC2771";
pub type SaltArray = [u8; SALT_LEN];
pub type KeyArray = [u8; CREDENTIAL_LEN];
pub type Error = &'static str;

/// Gera um salt aleatório de 32 bytes.
pub fn generate_salt() -> SaltArray {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Derivates a key from the user pass and a salt
///
/// # Arguments
///
/// * `user_pass` - A string slice that holds the user's password.
/// * `salt` - Some byte slice that holds the salt. If None, the default OLD hardcoded SALT is used.
///
/// # Examples
///
///
/// let user_pass = "password";
/// let salt = generate_salt();
/// let key = derive_key_from_password_and_salt(user_pass, &salt).unwrap();
///
pub fn derive_key_from_password_and_salt(user_pass: &str, salt: Option<&[u8]>) -> Result<KeyArray, Error> {
    let salt = salt.unwrap_or(&SALT.as_bytes());
    let n_iter = NonZeroU32::new(100_000).unwrap();
    let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        n_iter,
        salt,
        user_pass.as_bytes(),
        &mut pbkdf2_hash,
    );
    Ok(pbkdf2_hash)
}

/// Encrypts the given data using AES-256-GCM with a random nonce.
///
/// This function encrypts the provided data using the AES-256-GCM algorithm with
/// a 96bits random nonce generated for each encryption operation.
/// The nonce is prepended to the encrypted data to ensure it can be correctly decrypted later.
///
/// # Arguments
///
/// * `data` - The data to be encrypted as a vector of bytes.
/// * `key` - The encryption key as a 32-byte array.
///
/// # Returns
///
/// A vector of bytes containing the encrypted data with the nonce prepended.
///
/// # Errors
///
/// This function will panic if the encryption operation fails.
pub fn encrypt_data(data: Vec<u8>, key: &[u8]) -> Result<Vec<u8>, Error> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let encrypted_data = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: data.as_ref(),
                aad: b"",
            },
        )
        .unwrap(); // Encrypt the data using GCM
                   // println!("Encrypted data: {:?}, Len: {:?}", encrypted_data, encrypted_data.len());
                   // println!("Nonce: {:?}, Len: {:?}", nonce, nonce.len());
    Ok([nonce.to_vec(), encrypted_data].concat()) // Prepend nonce to the encrypted data
}

/// Decrypts the given data using AES-256-GCM with the provided key.
///
/// This function decrypts the provided data using the AES-256-GCM algorithm with the provided key.
/// The nonce is extracted from the data and the encrypted data is decrypted using the key and nonce.
///
/// # Arguments
///
/// * `data` - A vector of bytes with the 96bits nonce + the emcrypted data.
/// * `key` - The decryption key as a 32-byte array.
///
/// # Returns
///
/// A vector of bytes containing the decrypted data.
///
/// # Errors
///
/// This function will panic if the decryption operation fails.
pub fn decrypt_data(data: Vec<u8>, key: &[u8]) -> Result<Vec<u8>, Error> {
    // println!("Data to decrypt: {:?}, Len: {:?}", data, data.len());
    let data = data.split_at(12);
    let nonce = Nonce::from_slice(data.0);
    let encrypted_data = data.1; // Split the Nonce and encrypted data for GCM
                                 // println!("Data to decrypt: {:?}, Len: {:?}", encrypted_data, encrypted_data.len());

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    match cipher.decrypt(&nonce, encrypted_data) {
        // Decrypt the data using GCM
        Ok(result) => Ok(result),
        Err(_) => Err("Couldn't decrypt the text"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_from_password() {
        let user_pass = "test_password";
        let salt = generate_salt();
        let key = derive_key_from_password_and_salt(user_pass, Some(&salt)).unwrap();
        assert_eq!(key.len(), CREDENTIAL_LEN);
    }

    #[test]
    fn test_encrypt_decrypt_data() {
        let data = b"Hello, world!".to_vec();
        let user_pass = "test_password";
        let salt = generate_salt();
        let key = derive_key_from_password_and_salt(user_pass, Some(&salt)).unwrap();

        let encrypted_data = encrypt_data(data.clone(), &key).unwrap();
        let decrypted_data = decrypt_data(encrypted_data.clone(), &key).unwrap();

        assert_eq!(data, decrypted_data);
    }

    #[test]
    fn test_encrypt_data_length() {
        let data = b"Hello, world!".to_vec();
        let user_pass = "test_password";
        let salt = generate_salt();
        let key = derive_key_from_password_and_salt(user_pass, Some(&salt)).unwrap();

        let encrypted_data = encrypt_data(data.clone(), &key).unwrap();
        assert!(encrypted_data.len() > data.len()); // Encrypted data should be longer than original
    }
}

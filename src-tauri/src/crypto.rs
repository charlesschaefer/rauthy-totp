use aes_gcm::{
    aead::{Aead, AeadCore, OsRng, Payload}, Aes256Gcm, Key, KeyInit, Nonce
};

use ring::{pbkdf2, digest, rand::{SystemRandom, SecureRandom}};
use std::num::NonZeroU32;

const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
type KeyArray = [u8;CREDENTIAL_LEN];

/// Derives a key from a password using PBKDF2 with HMAC-SHA512.
/// 
/// # Arguments
/// 
/// * `user_pass` - A string slice that holds the user's password.
/// 
/// # Examples
/// 
/// 
/// let user_pass = "password";
/// let key = derive_key_from_password(user_pass).unwrap();
/// 
pub fn derive_key_from_password(user_pass: &str) -> Result<KeyArray, ()> {
    let n_iter = NonZeroU32::new(100_000).unwrap();
    let rng = SystemRandom::new();

    let mut salt = [0u8; CREDENTIAL_LEN];
    rng.fill(&mut salt);

    let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        n_iter,
        &salt,
        user_pass.as_bytes(),
        &mut pbkdf2_hash,
    );
    /* println!("Salt: {}", HEXUPPER.encode(&salt));
    println!("PBKDF2 hash: {}", HEXUPPER.encode(&pbkdf2_hash)); */

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
pub fn encrypt_data(data: Vec<u8>, key: &[u8]) -> Vec<u8> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let encrypted_data = cipher.encrypt(&nonce, Payload {
        msg: data.as_ref(),
        aad: b""
    }).unwrap(); // Encrypt the data using GCM
    [nonce.to_vec(), encrypted_data].concat() // Prepend nonce to the encrypted data
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
pub fn decrypt_data(data: Vec<u8>, key: &[u8]) -> Vec<u8> {
    println!("Data to decrypt: {:?}, Len: {:?}", data, data.len());
    let data = data.split_at(12);
    let nonce = Nonce::from_slice(data.0);
    let encrypted_data = data.1; // Split the Nonce and encrypted data for GCM
    
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    cipher.decrypt(&nonce, encrypted_data).unwrap() // Decrypt the data using GCM
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_derive_key_from_password() {
        let user_pass = "test_password";
        let key = derive_key_from_password(user_pass).unwrap();
        assert_eq!(key.len(), CREDENTIAL_LEN);
    }

    #[test]
    fn test_encrypt_decrypt_data() {
        let data = b"Hello, world!".to_vec();
        let user_pass = "test_password";
        let key = derive_key_from_password(user_pass).unwrap();

        let encrypted_data = encrypt_data(data.clone(), &key);
        let decrypted_data = decrypt_data(encrypted_data.clone(), &key);

        assert_eq!(data, decrypted_data);
    }

    #[test]
    fn test_encrypt_data_length() {
        let data = b"Hello, world!".to_vec();
        let user_pass = "test_password";
        let key = derive_key_from_password(user_pass).unwrap();

        let encrypted_data = encrypt_data(data.clone(), &key);
        assert!(encrypted_data.len() > data.len()); // Encrypted data should be longer than original
    }
}
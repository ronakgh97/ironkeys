use crate::error::{Error, Result};
use ring::rand::SecureRandom;
use ring::{aead, pbkdf2, rand};

const PBKDF2_ITERATIONS: u32 = 100_000;
const NONCE_LENGTH: usize = 12;
const SALT_LENGTH: usize = 32;
const KEY_LENGTH: usize = 32;

/// Encrypted data with its nonce
#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Generate a random salt for key derivation
pub fn generate_salt() -> Result<Vec<u8>> {
    let rng = rand::SystemRandom::new();
    let mut salt = vec![0u8; SALT_LENGTH];
    rng.fill(&mut salt)
        .map_err(|e| Error::KeyDerivationFailed(format!("Failed to generate salt: {e:?}")))?;
    Ok(salt)
}

/// Derive an encryption key from a password using PBKDF2
pub fn derive_key(password: &str, salt: &[u8], iterations: u32) -> Result<Vec<u8>> {
    if password.is_empty() {
        return Err(Error::EmptyPassword);
    }

    let mut key = vec![0u8; KEY_LENGTH];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(iterations)
            .ok_or_else(|| Error::KeyDerivationFailed("Invalid iterations".to_string()))?,
        salt,
        password.as_bytes(),
        &mut key,
    );

    Ok(key)
}

/// Hash a password for verification (same as derive_key, but semantically different)
pub fn hash_password(password: &str, salt: &[u8], iterations: u32) -> Result<Vec<u8>> {
    derive_key(password, salt, iterations)
}

/// Verify a password against a stored hash
pub fn verify_password(password: &str, salt: &[u8], hash: &[u8], iterations: u32) -> Result<bool> {
    let result = pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(iterations)
            .ok_or_else(|| Error::KeyDerivationFailed("Invalid iterations".to_string()))?,
        salt,
        password.as_bytes(),
        hash,
    );

    Ok(result.is_ok())
}

/// Encrypt data using AES-256-GCM
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<EncryptedData> {
    if key.len() != KEY_LENGTH {
        return Err(Error::EncryptionFailed(format!(
            "Invalid key length: expected {}, got {}",
            KEY_LENGTH,
            key.len()
        )));
    }

    // Create encryption key
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key)
        .map_err(|e| Error::EncryptionFailed(format!("Failed to create key: {e:?}")))?;
    let sealing_key = aead::LessSafeKey::new(unbound_key);

    // Generate random nonce
    let rng = rand::SystemRandom::new();
    let mut nonce_bytes = vec![0u8; NONCE_LENGTH];
    rng.fill(&mut nonce_bytes)
        .map_err(|e| Error::EncryptionFailed(format!("Failed to generate nonce: {e:?}")))?;
    let nonce = aead::Nonce::assume_unique_for_key(
        nonce_bytes
            .clone()
            .try_into()
            .map_err(|_| Error::EncryptionFailed("Invalid nonce length".to_string()))?,
    );

    // Encrypt the data
    let mut ciphertext = plaintext.to_vec();
    sealing_key
        .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut ciphertext)
        .map_err(|e| Error::EncryptionFailed(format!("Encryption failed: {e:?}")))?;

    Ok(EncryptedData {
        ciphertext,
        nonce: nonce_bytes,
    })
}

/// Decrypt data using AES-256-GCM
pub fn decrypt(encrypted: &EncryptedData, key: &[u8]) -> Result<Vec<u8>> {
    if key.len() != KEY_LENGTH {
        return Err(Error::DecryptionFailed(format!(
            "Invalid key length: expected {}, got {}",
            KEY_LENGTH,
            key.len()
        )));
    }

    if encrypted.nonce.len() != NONCE_LENGTH {
        return Err(Error::DecryptionFailed(format!(
            "Invalid nonce length: expected {}, got {}",
            NONCE_LENGTH,
            encrypted.nonce.len()
        )));
    }

    // Create decryption key
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key)
        .map_err(|e| Error::DecryptionFailed(format!("Failed to create key: {e:?}")))?;
    let opening_key = aead::LessSafeKey::new(unbound_key);

    // Create nonce
    let nonce = aead::Nonce::assume_unique_for_key(
        encrypted
            .nonce
            .clone()
            .try_into()
            .map_err(|_| Error::DecryptionFailed("Invalid nonce length".to_string()))?,
    );

    // Decrypt the data
    let mut ciphertext = encrypted.ciphertext.clone();
    let plaintext = opening_key
        .open_in_place(nonce, aead::Aad::empty(), &mut ciphertext)
        .map_err(|e| Error::DecryptionFailed(format!("Decryption failed: {e:?}")))?;

    Ok(plaintext.to_vec())
}

/// Get the default number of PBKDF2 iterations
pub fn default_iterations() -> u32 {
    PBKDF2_ITERATIONS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let password = "test_password";
        let salt = generate_salt().unwrap();
        let key = derive_key(password, &salt, default_iterations()).unwrap();

        let plaintext = b"Hello, IronKey!";
        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_password_verification() {
        let password = "my_secure_password";
        let salt = generate_salt().unwrap();
        let hash = hash_password(password, &salt, default_iterations()).unwrap();

        assert!(verify_password(password, &salt, &hash, default_iterations()).unwrap());
        assert!(!verify_password("wrong_password", &salt, &hash, default_iterations()).unwrap());
    }

    #[test]
    fn test_different_nonces() {
        let key = vec![0u8; 32];
        let plaintext = b"test";

        let encrypted1 = encrypt(plaintext, &key).unwrap();
        let encrypted2 = encrypt(plaintext, &key).unwrap();

        // Same plaintext should produce different ciphertexts (different nonces)
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    }
}

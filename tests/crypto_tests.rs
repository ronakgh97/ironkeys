// Crypto module tests

use ironkey::crypto::{
    decrypt, derive_key, encrypt, generate_salt, hash_password, verify_password,
};

const TEST_ITERATIONS: u32 = 100_000;

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let password = "test_password";
    let salt = generate_salt().unwrap();
    let key = derive_key(password, &salt, TEST_ITERATIONS).unwrap();

    let plaintext = b"Hello, IronKey!";
    let encrypted = encrypt(plaintext, &key).unwrap();
    let decrypted = decrypt(&encrypted, &key).unwrap();

    assert_eq!(plaintext, decrypted.as_slice());
}

#[test]
fn test_password_verification() {
    let password = "my_secure_password";
    let salt = generate_salt().unwrap();
    let hash = hash_password(password, &salt, TEST_ITERATIONS).unwrap();

    assert!(verify_password(password, &salt, &hash, TEST_ITERATIONS).unwrap());
    assert!(!verify_password("wrong_password", &salt, &hash, TEST_ITERATIONS).unwrap());
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

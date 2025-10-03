// Integration tests for end-to-end workflows
// These tests verify the complete data flow from encryption to storage

use ironkey::crypto::{
    decrypt, derive_key, encrypt, generate_salt, hash_password, verify_password,
};
use ironkey::storage::{Database, Entry};

const TEST_ITERATIONS: u32 = 100_000;

#[test]
fn test_complete_password_flow() {
    let password = "master_password_123";

    // Step 1: Generate salt
    let salt = generate_salt().unwrap();

    // Step 2: Derive encryption key
    let key = derive_key(password, &salt, TEST_ITERATIONS).unwrap();

    // Step 3: Hash password for verification
    let password_hash = hash_password(password, &salt, TEST_ITERATIONS).unwrap();

    // Step 4: Verify password
    assert!(verify_password(password, &salt, &password_hash, TEST_ITERATIONS).unwrap());
    assert!(!verify_password("wrong_password", &salt, &password_hash, TEST_ITERATIONS).unwrap());

    // Step 5: Encrypt data
    let secret_data = b"My secret API key: sk_test_123456";
    let encrypted = encrypt(secret_data, &key).unwrap();

    // Step 6: Decrypt data
    let decrypted = decrypt(&encrypted, &key).unwrap();

    assert_eq!(secret_data.to_vec(), decrypted);
}

#[test]
fn test_database_entry_roundtrip() {
    let password = "test_password";

    // Initialize database
    let salt = generate_salt().unwrap();
    let key = derive_key(password, &salt, TEST_ITERATIONS).unwrap();
    let password_hash = hash_password(password, &salt, TEST_ITERATIONS).unwrap();

    let mut db = Database::new(salt.clone(), password_hash, TEST_ITERATIONS);

    // Encrypt and store entry
    let plaintext = b"secret_value_12345";
    let encrypted = encrypt(plaintext, &key).unwrap();
    let entry = Entry::new(encrypted.ciphertext, encrypted.nonce, false);

    db.entries.insert("my_key".to_string(), entry);

    // Retrieve and decrypt entry
    let retrieved_entry = db.entries.get("my_key").unwrap();
    let retrieved_encrypted = ironkey::crypto::EncryptedData {
        ciphertext: retrieved_entry.get_encrypted_value().unwrap(),
        nonce: retrieved_entry.get_nonce().unwrap(),
    };

    let decrypted = decrypt(&retrieved_encrypted, &key).unwrap();

    assert_eq!(plaintext.to_vec(), decrypted);
}

#[test]
fn test_multiple_entries_different_passwords() {
    let salt = generate_salt().unwrap();

    let password1 = "password_one";
    let password2 = "password_two";

    let key1 = derive_key(password1, &salt, TEST_ITERATIONS).unwrap();
    let key2 = derive_key(password2, &salt, TEST_ITERATIONS).unwrap();

    // Encrypt with different keys
    let secret1 = b"Secret for key 1";
    let secret2 = b"Secret for key 2";

    let encrypted1 = encrypt(secret1, &key1).unwrap();
    let encrypted2 = encrypt(secret2, &key2).unwrap();

    // Decrypt with correct keys
    let decrypted1 = decrypt(&encrypted1, &key1).unwrap();
    let decrypted2 = decrypt(&encrypted2, &key2).unwrap();

    assert_eq!(secret1.to_vec(), decrypted1);
    assert_eq!(secret2.to_vec(), decrypted2);

    // Wrong key should fail
    let wrong_decrypt = decrypt(&encrypted1, &key2);
    assert!(wrong_decrypt.is_err());
}

#[test]
fn test_locked_entry_workflow() {
    let password = "test_password";
    let salt = generate_salt().unwrap();
    let key = derive_key(password, &salt, TEST_ITERATIONS).unwrap();

    // Create locked entry
    let plaintext = b"locked secret";
    let encrypted = encrypt(plaintext, &key).unwrap();
    let mut entry = Entry::new(encrypted.ciphertext.clone(), encrypted.nonce.clone(), true);

    // Verify it's locked
    assert!(entry.is_locked);

    // Simulate unlocking
    entry.is_locked = false;

    // Now we can "access" it (decrypt)
    let encrypted_data = ironkey::crypto::EncryptedData {
        ciphertext: entry.get_encrypted_value().unwrap(),
        nonce: entry.get_nonce().unwrap(),
    };
    let decrypted = decrypt(&encrypted_data, &key).unwrap();

    assert_eq!(plaintext.to_vec(), decrypted);
}

#[test]
fn test_database_with_mixed_locked_states() {
    let salt = generate_salt().unwrap();
    let hash = hash_password("test", &salt, TEST_ITERATIONS).unwrap();

    let mut db = Database::new(salt, hash, TEST_ITERATIONS);

    // Add mix of locked and unlocked entries
    for i in 0..5 {
        let encrypted = vec![i; 16];
        let nonce = vec![i + 10; 12];
        let is_locked = i % 2 == 0; // Even indices are locked

        let entry = Entry::new(encrypted, nonce, is_locked);
        db.entries.insert(format!("key_{i}"), entry);
    }

    // Verify lock states
    assert!(db.entries.get("key_0").unwrap().is_locked);
    assert!(!db.entries.get("key_1").unwrap().is_locked);
    assert!(db.entries.get("key_2").unwrap().is_locked);
    assert!(!db.entries.get("key_3").unwrap().is_locked);
    assert!(db.entries.get("key_4").unwrap().is_locked);
}

#[test]
fn test_empty_database_creation() {
    let salt = generate_salt().unwrap();
    let hash = vec![1, 2, 3, 4];
    let db = Database::new(salt.clone(), hash.clone(), TEST_ITERATIONS);

    assert!(db.entries.is_empty());
    assert_eq!(db.iterations, TEST_ITERATIONS);
    assert_eq!(db.get_salt().unwrap(), salt);
    assert_eq!(db.get_hash().unwrap(), hash);
}

#[test]
fn test_special_characters_in_plaintext() {
    let password = "test_password";
    let salt = generate_salt().unwrap();
    let key = derive_key(password, &salt, TEST_ITERATIONS).unwrap();

    let special_plaintext = b"!@#$%^&*()_+-=[]{}|;':\",./<>?`~\n\r\t";
    let encrypted = encrypt(special_plaintext, &key).unwrap();
    let decrypted = decrypt(&encrypted, &key).unwrap();

    assert_eq!(special_plaintext.to_vec(), decrypted);
}

#[test]
fn test_unicode_in_plaintext() {
    let password = "test_password";
    let salt = generate_salt().unwrap();
    let key = derive_key(password, &salt, TEST_ITERATIONS).unwrap();

    let unicode_plaintext = "Hello 世界 🔐 Ñoño".as_bytes();
    let encrypted = encrypt(unicode_plaintext, &key).unwrap();
    let decrypted = decrypt(&encrypted, &key).unwrap();

    assert_eq!(unicode_plaintext.to_vec(), decrypted);
}

#[test]
fn test_large_plaintext() {
    let password = "test_password";
    let salt = generate_salt().unwrap();
    let key = derive_key(password, &salt, TEST_ITERATIONS).unwrap();

    // 1KB of data
    let large_plaintext = vec![42u8; 1024];
    let encrypted = encrypt(&large_plaintext, &key).unwrap();
    let decrypted = decrypt(&encrypted, &key).unwrap();

    assert_eq!(large_plaintext, decrypted);
}

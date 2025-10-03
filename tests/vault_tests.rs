// Vault unit tests
// Note: Integration tests that modify the actual database should be run manually
// These tests focus on internal logic without touching the filesystem

use ironkey::crypto::{derive_key, encrypt, generate_salt};
use ironkey::storage::{Database, Entry};

const TEST_ITERATIONS: u32 = 100_000;

#[test]
fn test_database_structure() {
    let salt = generate_salt().unwrap();
    let hash = vec![1, 2, 3, 4];
    let db = Database::new(salt.clone(), hash.clone(), TEST_ITERATIONS);

    assert_eq!(db.get_salt().unwrap(), salt);
    assert_eq!(db.get_hash().unwrap(), hash);
    assert_eq!(db.iterations, TEST_ITERATIONS);
    assert!(db.entries.is_empty());
}

#[test]
fn test_entry_encryption_structure() {
    let password = "test_password";
    let salt = generate_salt().unwrap();
    let key = derive_key(password, &salt, TEST_ITERATIONS).unwrap();

    let plaintext = b"secret value";
    let encrypted_data = encrypt(plaintext, &key).unwrap();

    // Create entry with encrypted data
    let entry = Entry::new(
        encrypted_data.ciphertext.clone(),
        encrypted_data.nonce.clone(),
        false,
    );

    // Verify entry structure
    assert!(!entry.is_locked);
    assert_eq!(
        entry.get_encrypted_value().unwrap(),
        encrypted_data.ciphertext
    );
    assert_eq!(entry.get_nonce().unwrap(), encrypted_data.nonce);
}

#[test]
fn test_locked_entry_flag() {
    let encrypted_data = vec![1, 2, 3];
    let nonce = vec![4, 5, 6];

    let locked_entry = Entry::new(encrypted_data.clone(), nonce.clone(), true);
    let unlocked_entry = Entry::new(encrypted_data.clone(), nonce.clone(), false);

    assert!(locked_entry.is_locked);
    assert!(!unlocked_entry.is_locked);
}

#[test]
fn test_multiple_entries_in_database() {
    let salt = generate_salt().unwrap();
    let hash = vec![1, 2, 3];
    let mut db = Database::new(salt, hash, TEST_ITERATIONS);

    // Add multiple entries
    for i in 0..5 {
        let entry = Entry::new(vec![i], vec![i + 10], false);
        db.entries.insert(format!("key_{i}"), entry);
    }

    assert_eq!(db.entries.len(), 5);
    assert!(db.entries.contains_key("key_0"));
    assert!(db.entries.contains_key("key_4"));

    // Verify lock states
    assert!(!db.entries.get("key_0").unwrap().is_locked);
}

#[test]
fn test_password_derivation_consistency() {
    let password = "test_password";
    let salt = generate_salt().unwrap();

    let key1 = derive_key(password, &salt, TEST_ITERATIONS).unwrap();
    let key2 = derive_key(password, &salt, TEST_ITERATIONS).unwrap();

    assert_eq!(key1, key2);
}

#[test]
fn test_encryption_produces_different_ciphertexts() {
    let password = "test_password";
    let salt = generate_salt().unwrap();
    let key = derive_key(password, &salt, TEST_ITERATIONS).unwrap();

    let plaintext = b"same message";
    let encrypted1 = encrypt(plaintext, &key).unwrap();
    let encrypted2 = encrypt(plaintext, &key).unwrap();

    // Different nonces should produce different ciphertexts
    assert_ne!(encrypted1.nonce, encrypted2.nonce);
    assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
}

// TODO: Full integration tests with temporary databases
// These would require:
// - Vault::init() with custom path support
// - Vault::unlock() with custom path support
// - Test vault operations: create, get, update, delete, lock/unlock
// - Proper cleanup of test databases
//
// For now, vault operations are tested manually and through the CLI

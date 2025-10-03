// Storage module tests

use ironkey::storage::{Database, Entry};

#[test]
fn test_database_creation() {
    let salt = vec![1, 2, 3, 4];
    let hash = vec![5, 6, 7, 8];
    let iterations = 100_000;

    let db = Database::new(salt.clone(), hash.clone(), iterations);

    // Database stores base64-encoded values, need to decode for comparison
    assert_eq!(db.get_salt().unwrap(), salt);
    assert_eq!(db.get_hash().unwrap(), hash);
    assert_eq!(db.iterations, iterations);
    assert!(db.entries.is_empty());
}

#[test]
fn test_entry_creation() {
    let nonce = vec![1, 2, 3];
    let encrypted_value = vec![4, 5, 6];
    let is_locked = false;

    let entry = Entry::new(encrypted_value.clone(), nonce.clone(), is_locked);

    // Entry stores base64-encoded strings, not raw bytes
    assert_eq!(entry.is_locked, is_locked);

    // Verify base64 encoding is actually done
    assert_eq!(entry.get_nonce().unwrap(), nonce);
    assert_eq!(entry.get_encrypted_value().unwrap(), encrypted_value);
}

#[test]
fn test_database_add_entry() {
    let mut db = Database::new(vec![1, 2, 3], vec![4, 5, 6], 100_000);
    let entry = Entry::new(vec![10, 11, 12], vec![7, 8, 9], false);

    db.entries.insert("test_key".to_string(), entry.clone());

    assert_eq!(db.entries.len(), 1);
    assert!(db.entries.contains_key("test_key"));
    assert!(!db.entries.get("test_key").unwrap().is_locked);
}

#[test]
fn test_entry_locked_flag() {
    let entry_unlocked = Entry::new(vec![2], vec![1], false);
    let entry_locked = Entry::new(vec![4], vec![3], true);

    assert!(!entry_unlocked.is_locked);
    assert!(entry_locked.is_locked);
}

#[test]
fn test_database_multiple_entries() {
    let mut db = Database::new(vec![1], vec![2], 100_000);

    for i in 0..5 {
        let entry = Entry::new(vec![i + 10], vec![i], false);
        db.entries.insert(format!("key_{i}"), entry);
    }

    assert_eq!(db.entries.len(), 5);
    assert!(db.entries.contains_key("key_0"));
    assert!(db.entries.contains_key("key_4"));
}

#[test]
fn test_entry_base64_decoding() {
    let original_nonce = vec![255, 128, 64, 32, 16, 8, 4, 2, 1];
    let original_encrypted = vec![11, 22, 33, 44, 55, 66, 77, 88, 99];

    let entry = Entry::new(original_encrypted.clone(), original_nonce.clone(), true);

    // Decode and verify using entry methods
    let decoded_nonce = entry.get_nonce().unwrap();
    let decoded_encrypted = entry.get_encrypted_value().unwrap();

    assert_eq!(decoded_nonce, original_nonce);
    assert_eq!(decoded_encrypted, original_encrypted);
}

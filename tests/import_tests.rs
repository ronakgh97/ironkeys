//! Import Functionality Tests
//!
//! Tests for importing vault entries from encrypted .ik files
//! NOTE: These tests must run serially because they share the same database file.
//! Run with: cargo test --test import_tests -- --test-threads=1

use ironkey::error::Result;
use ironkey::storage;
use ironkey::vault::Vault;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: Create a test vault with specific entries
fn create_test_vault_with_entries(
    master_password: &str,
    entries: Vec<(&str, &str, bool)>,
) -> Result<Vault> {
    // Clean up any existing test database
    let db_path = storage::get_database_path()?;
    let _ = fs::remove_file(&db_path);

    // Create vault
    let mut vault = Vault::init(master_password.to_string())?;

    // Add entries (key, value, locked)
    for (key, value, locked) in entries {
        vault.create_entry(key.to_string(), value.to_string())?;
        if locked {
            vault.toggle_lock(key)?;
        }
    }

    Ok(vault)
}

/// Cleanup function
fn cleanup_test_files(export_path: Option<&PathBuf>) {
    if let Ok(db_path) = storage::get_database_path() {
        let _ = fs::remove_file(db_path);
    }
    if let Some(path) = export_path {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn test_import_full_vault_merge_mode() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("export.ik");

    // Create source vault with 3 entries and export it
    {
        let vault = create_test_vault_with_entries(
            "master123",
            vec![
                ("github", "ghp_token123", false),
                ("aws", "aws_secret456", true),
                ("db", "postgres_pass789", false),
            ],
        )
        .unwrap();

        vault
            .export_to_file(&export_path, "export123".to_string())
            .unwrap();
    }

    // Create destination vault with 1 existing entry
    let mut vault =
        create_test_vault_with_entries("master456", vec![("existing", "value", false)]).unwrap();

    // Import in merge mode (should add 3 new entries, keep 1 existing)
    let result = vault.import_from_file(&export_path, "export123".to_string(), true, false, false);

    assert!(result.is_ok());

    // Verify entries
    let entries = vault.list_entries(None, None).unwrap();
    assert_eq!(entries.len(), 4); // 1 existing + 3 imported

    // Check all keys exist
    let keys: Vec<&String> = entries.iter().map(|(k, _)| *k).collect();
    assert!(keys.contains(&&"existing".to_string()));
    assert!(keys.contains(&&"github".to_string()));
    assert!(keys.contains(&&"aws".to_string()));
    assert!(keys.contains(&&"db".to_string()));

    cleanup_test_files(Some(&export_path));
}

#[test]
fn test_import_replace_mode_overwrites_existing() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("export.ik");

    // Create source vault and export
    {
        let vault =
            create_test_vault_with_entries("master123", vec![("github", "new_token_999", false)])
                .unwrap();

        vault
            .export_to_file(&export_path, "export123".to_string())
            .unwrap();
    }

    // Create destination vault with same key but different value
    let mut vault =
        create_test_vault_with_entries("master456", vec![("github", "old_token_123", false)])
            .unwrap();

    // Import in replace mode (should overwrite existing entry)
    let result = vault.import_from_file(&export_path, "export123".to_string(), false, true, false);

    assert!(result.is_ok());

    // Verify the value was replaced
    let new_value = vault.get_entry("github").unwrap();
    assert_eq!(new_value, "new_token_999");

    cleanup_test_files(Some(&export_path));
}

#[test]
fn test_import_merge_mode_skips_existing() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("export.ik");

    // Create source vault and export
    {
        let vault =
            create_test_vault_with_entries("master123", vec![("github", "new_token_999", false)])
                .unwrap();

        vault
            .export_to_file(&export_path, "export123".to_string())
            .unwrap();
    }

    // Create destination vault with same key but different value
    let mut vault =
        create_test_vault_with_entries("master456", vec![("github", "old_token_123", false)])
            .unwrap();

    // Import in merge mode (should skip existing entry)
    let result = vault.import_from_file(&export_path, "export123".to_string(), true, false, false);

    assert!(result.is_ok());

    // Verify the value was NOT replaced (old value preserved)
    let value = vault.get_entry("github").unwrap();
    assert_eq!(value, "old_token_123");

    cleanup_test_files(Some(&export_path));
}

#[test]
fn test_import_preserves_lock_status() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("export.ik");

    // Create source vault with locked entry and export
    {
        let vault =
            create_test_vault_with_entries("master123", vec![("locked_key", "secret", true)])
                .unwrap();

        vault
            .export_to_file(&export_path, "export123".to_string())
            .unwrap();
    }

    // Create empty destination vault
    let mut vault = create_test_vault_with_entries("master456", vec![]).unwrap();

    // Import
    let result = vault.import_from_file(&export_path, "export123".to_string(), true, false, false);

    assert!(result.is_ok());

    // Verify lock status is preserved
    let entries = vault.list_entries(None, Some(true)).unwrap(); // Filter locked only
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, "locked_key");
    assert!(entries[0].1); // locked status

    cleanup_test_files(Some(&export_path));
}

#[test]
fn test_import_wrong_export_password() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("export.ik");

    // Create and export vault
    {
        let vault =
            create_test_vault_with_entries("master123", vec![("key", "value", false)]).unwrap();

        vault
            .export_to_file(&export_path, "correct_password".to_string())
            .unwrap();
    }

    // Try to import with wrong password
    let mut vault = create_test_vault_with_entries("master456", vec![]).unwrap();

    let result = vault.import_from_file(
        &export_path,
        "wrong_password".to_string(),
        true,
        false,
        false,
    );

    // Should fail with decryption error
    assert!(result.is_err());

    cleanup_test_files(Some(&export_path));
}

#[test]
fn test_import_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let non_existent_path = temp_dir.path().join("does_not_exist.ik");

    let mut vault = create_test_vault_with_entries("master456", vec![]).unwrap();

    let result = vault.import_from_file(
        &non_existent_path,
        "export123".to_string(),
        true,
        false,
        false,
    );

    // Should fail with IO error
    assert!(result.is_err());

    cleanup_test_files(None);
}

#[test]
fn test_import_diff_mode_does_not_modify() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("export.ik");

    // Create and export vault
    {
        let vault = create_test_vault_with_entries(
            "master123",
            vec![
                ("new_key", "new_value", false),
                ("github", "new_token", false),
            ],
        )
        .unwrap();

        vault
            .export_to_file(&export_path, "export123".to_string())
            .unwrap();
    }

    // Create destination vault with one existing entry
    let mut vault =
        create_test_vault_with_entries("master456", vec![("github", "old_token", false)]).unwrap();

    // Import in diff mode (dry-run)
    let result = vault.import_from_file(&export_path, "export123".to_string(), false, false, true);

    assert!(result.is_ok());

    // In diff mode, the vault should be unchanged
    let entries = vault.list_entries(None, None).unwrap();
    assert_eq!(entries.len(), 1); // Still only 1 entry

    let value = vault.get_entry("github").unwrap();
    assert_eq!(value, "old_token"); // Still old value

    cleanup_test_files(Some(&export_path));
}

#[test]
fn test_import_empty_export() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("export.ik");

    // Create and export empty vault
    {
        let vault = create_test_vault_with_entries("master123", vec![]).unwrap();

        vault
            .export_to_file(&export_path, "export123".to_string())
            .unwrap();
    }

    // Create destination vault with entries
    let mut vault =
        create_test_vault_with_entries("master456", vec![("existing", "value", false)]).unwrap();

    // Import empty vault
    let result = vault.import_from_file(&export_path, "export123".to_string(), true, false, false);

    assert!(result.is_ok());

    // Should still have the existing entry
    let entries = vault.list_entries(None, None).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, "existing");

    cleanup_test_files(Some(&export_path));
}

#[test]
fn test_import_into_empty_vault() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("export.ik");

    // Create and export vault with entries
    {
        let vault = create_test_vault_with_entries(
            "master123",
            vec![("key1", "value1", false), ("key2", "value2", true)],
        )
        .unwrap();

        vault
            .export_to_file(&export_path, "export123".to_string())
            .unwrap();
    }

    // Create empty destination vault
    let mut vault = create_test_vault_with_entries("master456", vec![]).unwrap();

    // Import into empty vault
    let result = vault.import_from_file(&export_path, "export123".to_string(), true, false, false);

    assert!(result.is_ok());

    // Should have both entries
    let entries = vault.list_entries(None, None).unwrap();
    assert_eq!(entries.len(), 2);

    // Verify lock status
    let locked_entries = vault.list_entries(None, Some(true)).unwrap();
    assert_eq!(locked_entries.len(), 1);
    assert_eq!(locked_entries[0].0, "key2");

    let unlocked_entries = vault.list_entries(None, Some(false)).unwrap();
    assert_eq!(unlocked_entries.len(), 1);
    assert_eq!(unlocked_entries[0].0, "key1");

    cleanup_test_files(Some(&export_path));
}

#[test]
fn test_import_validates_format_version() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("export.ik");

    // Create a malformed export file with unsupported version
    let malformed_json = r#"{
        "format_version": "99.0.0",
        "exported_at": "2024-01-01T00:00:00Z",
        "entry_count": 0,
        "encryption": {
            "algorithm": "AES-256-GCM",
            "salt": "AAAA",
            "nonce": "BBBB",
            "iterations": 100000
        },
        "encrypted_data": "CCCC",
        "metadata": {
            "exported_from": "ironkey"
        }
    }"#;

    fs::write(&export_path, malformed_json).unwrap();

    // Try to import
    let mut vault = create_test_vault_with_entries("master456", vec![]).unwrap();

    let result = vault.import_from_file(&export_path, "export123".to_string(), true, false, false);

    // Should fail due to unsupported format version
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("format version") || err_msg.contains("99.0.0"));

    cleanup_test_files(Some(&export_path));
}

#[test]
fn test_import_malformed_json() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("export.ik");

    // Create a malformed JSON file
    fs::write(&export_path, "{ this is not valid json }").unwrap();

    // Try to import
    let mut vault = create_test_vault_with_entries("master456", vec![]).unwrap();

    let result = vault.import_from_file(&export_path, "export123".to_string(), true, false, false);

    // Should fail with deserialization error
    assert!(result.is_err());

    cleanup_test_files(Some(&export_path));
}

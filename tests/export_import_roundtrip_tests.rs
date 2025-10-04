//! Round-Trip Export/Import Integration Tests
//!
//! Tests complete export → import workflows
//! NOTE: These tests must run serially because they share the same database file.

use ironkey::storage;
use ironkey::vault::Vault;
use std::fs;
use tempfile::TempDir;

/// Cleanup function
fn cleanup() {
    if let Ok(db_path) = storage::get_database_path() {
        let _ = fs::remove_file(db_path);
    }
}

#[test]
fn test_export_import_roundtrip_preserves_all_data() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("roundtrip.ik");

    // Phase 1: Create and populate source vault
    {
        cleanup();
        let mut vault = Vault::init("source_master".to_string()).unwrap();

        vault
            .create_entry("github".to_string(), "ghp_token123".to_string())
            .unwrap();
        vault
            .create_entry("aws".to_string(), "aws_secret456".to_string())
            .unwrap();
        vault
            .create_entry("db".to_string(), "postgres_pass789".to_string())
            .unwrap();

        // Lock one entry
        vault.toggle_lock("db").unwrap();

        // Export
        vault
            .export_to_file(&export_path, "export_password".to_string())
            .unwrap();
    }

    // Phase 2: Import into new vault with different master password
    {
        cleanup();
        let mut vault = Vault::init("dest_master".to_string()).unwrap();

        // Import in merge mode
        let result = vault
            .import_from_file(
                &export_path,
                "export_password".to_string(),
                true,
                false,
                false,
            )
            .unwrap();

        // Verify import results
        assert_eq!(result.added.len(), 3);
        assert_eq!(result.updated.len(), 0);
        assert_eq!(result.skipped.len(), 0);

        // Verify all entries exist and values are correct
        assert_eq!(vault.get_entry("github").unwrap(), "ghp_token123");
        assert_eq!(vault.get_entry("aws").unwrap(), "aws_secret456");

        // Entry 'db' is locked, need to unlock it first
        vault.toggle_lock("db").unwrap(); // Unlock
        assert_eq!(vault.get_entry("db").unwrap(), "postgres_pass789");
        vault.toggle_lock("db").unwrap(); // Lock it again

        // Verify lock status preserved
        let entries = vault.list_entries(None, Some(true)).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "db");
    }

    cleanup();
}

#[test]
fn test_export_import_merge_preserves_existing() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("merge.ik");

    // Phase 1: Export from source vault
    {
        cleanup();
        let mut vault = Vault::init("source_master".to_string()).unwrap();

        vault
            .create_entry("key1".to_string(), "value1".to_string())
            .unwrap();
        vault
            .create_entry("key2".to_string(), "value2".to_string())
            .unwrap();

        vault
            .export_to_file(&export_path, "export_password".to_string())
            .unwrap();
    }

    // Phase 2: Import into vault with existing entry
    {
        cleanup();
        let mut vault = Vault::init("dest_master".to_string()).unwrap();

        // Add an existing entry with same key but different value
        vault
            .create_entry("key1".to_string(), "existing_value".to_string())
            .unwrap();

        // Import in merge mode (should skip key1, add key2)
        let result = vault
            .import_from_file(
                &export_path,
                "export_password".to_string(),
                true,
                false,
                false,
            )
            .unwrap();

        assert_eq!(result.added.len(), 1);
        assert_eq!(result.added[0], "key2");
        assert_eq!(result.skipped.len(), 1);
        assert_eq!(result.skipped[0], "key1");

        // Verify key1 has original value (not replaced)
        assert_eq!(vault.get_entry("key1").unwrap(), "existing_value");
        // Verify key2 was added
        assert_eq!(vault.get_entry("key2").unwrap(), "value2");
    }

    cleanup();
}

#[test]
fn test_export_import_replace_overwrites_existing() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("replace.ik");

    // Phase 1: Export from source vault
    {
        cleanup();
        let mut vault = Vault::init("source_master".to_string()).unwrap();

        vault
            .create_entry("key1".to_string(), "new_value".to_string())
            .unwrap();
        vault
            .create_entry("key2".to_string(), "value2".to_string())
            .unwrap();

        vault
            .export_to_file(&export_path, "export_password".to_string())
            .unwrap();
    }

    // Phase 2: Import into vault with existing entry (replace mode)
    {
        cleanup();
        let mut vault = Vault::init("dest_master".to_string()).unwrap();

        // Add an existing entry with same key but different value
        vault
            .create_entry("key1".to_string(), "old_value".to_string())
            .unwrap();

        // Import in replace mode (should overwrite key1, add key2)
        let result = vault
            .import_from_file(
                &export_path,
                "export_password".to_string(),
                false,
                true,
                false,
            )
            .unwrap();

        assert_eq!(result.added.len(), 1);
        assert_eq!(result.added[0], "key2");
        assert_eq!(result.updated.len(), 1);
        assert_eq!(result.updated[0], "key1");

        // Verify key1 was replaced with new value
        assert_eq!(vault.get_entry("key1").unwrap(), "new_value");
        // Verify key2 was added
        assert_eq!(vault.get_entry("key2").unwrap(), "value2");
    }

    cleanup();
}

#[test]
fn test_export_import_diff_mode_no_changes() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("diff.ik");

    // Phase 1: Export from source vault
    {
        cleanup();
        let mut vault = Vault::init("source_master".to_string()).unwrap();

        vault
            .create_entry("key1".to_string(), "value1".to_string())
            .unwrap();
        vault
            .create_entry("key2".to_string(), "value2".to_string())
            .unwrap();

        vault
            .export_to_file(&export_path, "export_password".to_string())
            .unwrap();
    }

    // Phase 2: Import in diff mode (no changes should be made)
    {
        cleanup();
        let mut vault = Vault::init("dest_master".to_string()).unwrap();

        vault
            .create_entry("existing".to_string(), "value".to_string())
            .unwrap();

        let entries_before = vault.list_entries(None, None).unwrap();
        assert_eq!(entries_before.len(), 1);

        // Import in diff mode
        let _result = vault
            .import_from_file(
                &export_path,
                "export_password".to_string(),
                false,
                false,
                true,
            )
            .unwrap();

        // Verify no changes were made
        let entries_after = vault.list_entries(None, None).unwrap();
        assert_eq!(entries_after.len(), 1); // Still only 1 entry
        assert_eq!(entries_after[0].0, "existing");
    }

    cleanup();
}

#[test]
fn test_multiple_export_import_cycles() {
    let temp_dir = TempDir::new().unwrap();
    let export1_path = temp_dir.path().join("export1.ik");
    let export2_path = temp_dir.path().join("export2.ik");

    // Cycle 1: Create → Export
    {
        cleanup();
        let mut vault = Vault::init("master1".to_string()).unwrap();

        vault
            .create_entry("original".to_string(), "data".to_string())
            .unwrap();

        vault
            .export_to_file(&export1_path, "pass1".to_string())
            .unwrap();
    }

    // Cycle 2: Import → Add → Export
    {
        cleanup();
        let mut vault = Vault::init("master2".to_string()).unwrap();

        vault
            .import_from_file(&export1_path, "pass1".to_string(), true, false, false)
            .unwrap();

        vault
            .create_entry("added".to_string(), "new_data".to_string())
            .unwrap();

        vault
            .export_to_file(&export2_path, "pass2".to_string())
            .unwrap();
    }

    // Cycle 3: Import and verify both entries exist
    {
        cleanup();
        let mut vault = Vault::init("master3".to_string()).unwrap();

        vault
            .import_from_file(&export2_path, "pass2".to_string(), true, false, false)
            .unwrap();

        let entries = vault.list_entries(None, None).unwrap();
        assert_eq!(entries.len(), 2);

        assert_eq!(vault.get_entry("original").unwrap(), "data");
        assert_eq!(vault.get_entry("added").unwrap(), "new_data");
    }

    cleanup();
}

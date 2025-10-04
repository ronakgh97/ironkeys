//! Export Functionality Tests
//!
//! Tests for exporting vault entries to encrypted .ik files
//! NOTE: These tests must run serially because they share the same database file.
//! Run with: cargo test --test export_tests -- --test-threads=1

use ironkey::error::Result;
use ironkey::storage;
use ironkey::vault::Vault;
use std::fs;
use std::path::PathBuf;

/// Helper function to create a test vault with sample entries
fn setup_test_vault() -> Result<Vault> {
    // Clean up any existing test database
    let db_path = storage::get_database_path()?;
    let _ = fs::remove_file(&db_path);

    // Create initial database
    let master_password = "test_master_password".to_string();
    let mut vault = Vault::init(master_password)?;

    // Add test entries
    vault.create_entry("github_token".to_string(), "ghp_test123".to_string())?;
    vault.create_entry("aws_api_key".to_string(), "AKIA_test456".to_string())?;
    vault.create_entry("database_password".to_string(), "db_pass789".to_string())?;

    // Lock one entry
    vault.toggle_lock("database_password")?;

    Ok(vault)
}

/// Cleanup function to remove test database and export files
fn cleanup_test_files() {
    if let Ok(db_path) = storage::get_database_path() {
        let _ = fs::remove_file(db_path);
    }
    // Clean up any .ik files in current directory
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "ik" {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
}

#[test]
fn test_export_full_vault() {
    let vault = setup_test_vault().unwrap();
    let output_path = PathBuf::from("test_export.ik");

    let export_password = "export_pass_123".to_string();
    let result = vault.export_to_file(&output_path, export_password);

    assert!(result.is_ok(), "Export should succeed");
    assert!(output_path.exists(), "Export file should be created");

    // Verify file is valid JSON
    let content = fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Check required fields
    assert!(json.get("format_version").is_some());
    assert!(json.get("exported_at").is_some());
    assert!(json.get("entry_count").is_some());
    assert!(json.get("encryption").is_some());
    assert!(json.get("encrypted_data").is_some());

    assert_eq!(json["entry_count"], 3);

    cleanup_test_files();
}

#[test]
fn test_export_empty_vault() {
    // Clean up and create empty vault
    let db_path = storage::get_database_path().unwrap();
    let _ = fs::remove_file(&db_path);

    let master_password = "test_master_password".to_string();
    let vault = Vault::init(master_password).unwrap();

    let output_path = PathBuf::from("test_empty_export.ik");
    let export_password = "export_pass_123".to_string();

    let result = vault.export_to_file(&output_path, export_password);

    assert!(result.is_ok(), "Exporting empty vault should succeed");
    assert!(output_path.exists(), "Export file should be created");

    // Verify entry count is 0
    let content = fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(json["entry_count"], 0);

    cleanup_test_files();
}

#[test]
fn test_export_preserves_lock_status() {
    let vault = setup_test_vault().unwrap();
    let output_path = PathBuf::from("test_lock_export.ik");
    let export_password = "export_pass_123".to_string();

    vault
        .export_to_file(&output_path, export_password.clone())
        .unwrap();

    // We'll verify lock status is preserved in import tests
    // For now, just verify export succeeds
    assert!(output_path.exists());

    cleanup_test_files();
}

#[test]
fn test_export_file_already_exists() {
    let vault = setup_test_vault().unwrap();
    let output_path = PathBuf::from("test_existing.ik");

    // Create the file first
    fs::write(&output_path, "existing content").unwrap();

    let export_password = "export_pass_123".to_string();
    let result = vault.export_to_file(&output_path, export_password);

    // Should fail because file exists (without --force flag)
    assert!(result.is_err(), "Should fail when file exists");

    cleanup_test_files();
}

#[test]
fn test_export_with_force_overwrite() {
    let vault = setup_test_vault().unwrap();
    let output_path = PathBuf::from("test_force_export.ik");

    // Create the file first
    fs::write(&output_path, "existing content").unwrap();

    let export_password = "export_pass_123".to_string();
    let result = vault.export_to_file_force(&output_path, export_password);

    assert!(result.is_ok(), "Should succeed with force flag");

    // Verify it's our new export, not old content
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("format_version"));
    assert!(!content.contains("existing content"));

    cleanup_test_files();
}

#[test]
fn test_export_invalid_path() {
    let vault = setup_test_vault().unwrap();

    // Try to export to invalid path (directory that doesn't exist)
    let output_path = PathBuf::from("/nonexistent/directory/export.ik");
    let export_password = "export_pass_123".to_string();

    let result = vault.export_to_file(&output_path, export_password);

    assert!(result.is_err(), "Should fail with invalid path");

    cleanup_test_files();
}

#[test]
fn test_export_includes_metadata() {
    let vault = setup_test_vault().unwrap();
    let output_path = PathBuf::from("test_metadata.ik");
    let export_password = "export_pass_123".to_string();

    vault.export_to_file(&output_path, export_password).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Check metadata structure
    assert!(json["metadata"].is_object());
    assert!(json["metadata"]["exported_from"].is_string());

    // TODO fields should be null for now
    assert!(json["metadata"]["vault_name"].is_null());
    assert!(json["metadata"]["tags"].is_null());

    cleanup_test_files();
}

#[test]
fn test_export_encryption_fields() {
    let vault = setup_test_vault().unwrap();
    let output_path = PathBuf::from("test_encryption.ik");
    let export_password = "export_pass_123".to_string();

    vault.export_to_file(&output_path, export_password).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Verify encryption metadata
    let encryption = &json["encryption"];
    assert_eq!(encryption["algorithm"], "AES-256-GCM");
    assert!(encryption["salt"].is_string());
    assert!(encryption["nonce"].is_string());
    assert_eq!(encryption["iterations"], 100000);

    // Verify encrypted data is base64 string
    assert!(json["encrypted_data"].is_string());
    let encrypted_data = json["encrypted_data"].as_str().unwrap();
    assert!(!encrypted_data.is_empty());

    cleanup_test_files();
}

#[test]
fn test_export_different_passwords_produce_different_output() {
    let vault = setup_test_vault().unwrap();

    let output_path1 = PathBuf::from("test_pass1.ik");
    let output_path2 = PathBuf::from("test_pass2.ik");

    vault
        .export_to_file(&output_path1, "password1".to_string())
        .unwrap();
    vault
        .export_to_file(&output_path2, "password2".to_string())
        .unwrap();

    let content1 = fs::read_to_string(&output_path1).unwrap();
    let content2 = fs::read_to_string(&output_path2).unwrap();

    // Files should be different (different salts/nonces)
    assert_ne!(
        content1, content2,
        "Different passwords should produce different outputs"
    );

    cleanup_test_files();
}

#[test]
fn test_export_format_version() {
    let vault = setup_test_vault().unwrap();
    let output_path = PathBuf::from("test_version.ik");
    let export_password = "export_pass_123".to_string();

    vault.export_to_file(&output_path, export_password).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Should have semantic version format
    let version = json["format_version"].as_str().unwrap();
    assert!(version.starts_with("1.0"));

    cleanup_test_files();
}

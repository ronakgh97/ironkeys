//! Search and Filter Tests
//!
//! Tests for the search and filter functionality of list_entries.
//! NOTE: These tests must run serially because they share the same database file.
//! Run with: cargo test --test search_filter_tests -- --test-threads=1

use ironkey::error::Result;
use ironkey::storage;
use ironkey::vault::Vault;
use std::fs;

/// Helper function to create a test vault with sample entries
/// Note: Tests must run serially due to shared database file
fn setup_test_vault() -> Result<Vault> {
    // Clean up any existing test database
    let db_path = storage::get_database_path()?;
    let _ = fs::remove_file(&db_path);

    // Create initial database
    let master_password = "test_master_password".to_string();
    let mut vault = Vault::init(master_password)?;

    // Add diverse test entries
    vault.create_entry("github_token".to_string(), "ghp_test123".to_string())?;
    vault.create_entry("aws_api_key".to_string(), "AKIA_test456".to_string())?;
    vault.create_entry("database_password".to_string(), "db_pass789".to_string())?;
    vault.create_entry("email_password".to_string(), "email123".to_string())?;
    vault.create_entry("GitHub_Personal".to_string(), "ghp_personal".to_string())?; // Mixed case
    vault.create_entry("api_secret".to_string(), "secret999".to_string())?;

    // Lock some entries for testing filter
    // toggle_lock returns true if now locked
    vault.toggle_lock("aws_api_key")?;
    vault.toggle_lock("database_password")?;

    Ok(vault)
}

/// Cleanup function to remove test database
fn cleanup_test_vault() {
    if let Ok(db_path) = storage::get_database_path() {
        let _ = fs::remove_file(db_path);
    }
}

#[test]
fn test_list_all_entries_no_filter() {
    let vault = setup_test_vault().unwrap();

    let entries = vault.list_entries(None, None).unwrap();

    assert_eq!(entries.len(), 6, "Should return all 6 entries");

    // Verify all entries are present
    let keys: Vec<&str> = entries.iter().map(|e| e.0.as_str()).collect();
    assert!(keys.contains(&"github_token"));
    assert!(keys.contains(&"aws_api_key"));
    assert!(keys.contains(&"database_password"));
    assert!(keys.contains(&"email_password"));
    assert!(keys.contains(&"GitHub_Personal"));
    assert!(keys.contains(&"api_secret"));

    cleanup_test_vault();
}

#[test]
fn test_search_by_exact_match() {
    let vault = setup_test_vault().unwrap();

    let entries = vault.list_entries(Some("github_token"), None).unwrap();

    assert_eq!(entries.len(), 1, "Should find exact match");
    assert_eq!(entries[0].0, "github_token");

    cleanup_test_vault();
}

#[test]
fn test_search_case_insensitive() {
    let vault = setup_test_vault().unwrap();

    // Search with different case
    let entries = vault.list_entries(Some("GITHUB"), None).unwrap();

    assert_eq!(entries.len(), 2, "Should find both github entries");

    let keys: Vec<&str> = entries.iter().map(|e| e.0.as_str()).collect();
    assert!(keys.contains(&"github_token"));
    assert!(keys.contains(&"GitHub_Personal"));

    cleanup_test_vault();
}

#[test]
fn test_search_partial_match() {
    let vault = setup_test_vault().unwrap();

    // Search for partial string
    let entries = vault.list_entries(Some("api"), None).unwrap();

    assert_eq!(entries.len(), 2, "Should find both API entries");

    let keys: Vec<&str> = entries.iter().map(|e| e.0.as_str()).collect();
    assert!(keys.contains(&"aws_api_key"));
    assert!(keys.contains(&"api_secret"));

    cleanup_test_vault();
}

#[test]
fn test_search_no_results() {
    let vault = setup_test_vault().unwrap();

    let entries = vault.list_entries(Some("nonexistent"), None).unwrap();

    assert_eq!(entries.len(), 0, "Should return empty list for no matches");

    cleanup_test_vault();
}

#[test]
fn test_filter_locked_only() {
    let vault = setup_test_vault().unwrap();

    let entries = vault.list_entries(None, Some(true)).unwrap();

    assert_eq!(entries.len(), 2, "Should return only locked entries");

    let keys: Vec<&str> = entries.iter().map(|e| e.0.as_str()).collect();
    assert!(keys.contains(&"aws_api_key"));
    assert!(keys.contains(&"database_password"));

    // Verify all returned entries are locked
    for entry in &entries {
        assert!(entry.1, "All entries should be locked");
    }

    cleanup_test_vault();
}

#[test]
fn test_filter_unlocked_only() {
    let vault = setup_test_vault().unwrap();

    let entries = vault.list_entries(None, Some(false)).unwrap();

    assert_eq!(entries.len(), 4, "Should return only unlocked entries");

    let keys: Vec<&str> = entries.iter().map(|e| e.0.as_str()).collect();
    assert!(keys.contains(&"github_token"));
    assert!(keys.contains(&"email_password"));
    assert!(keys.contains(&"GitHub_Personal"));
    assert!(keys.contains(&"api_secret"));

    // Verify all returned entries are unlocked
    for entry in &entries {
        assert!(!entry.1, "All entries should be unlocked");
    }

    cleanup_test_vault();
}

#[test]
fn test_search_and_filter_locked() {
    let vault = setup_test_vault().unwrap();

    // Search for "password" AND filter locked only
    let entries = vault.list_entries(Some("password"), Some(true)).unwrap();

    assert_eq!(entries.len(), 1, "Should find only locked password entry");
    assert_eq!(entries[0].0, "database_password");
    assert!(entries[0].1);

    cleanup_test_vault();
}

#[test]
fn test_search_and_filter_unlocked() {
    let vault = setup_test_vault().unwrap();

    // Search for "password" AND filter unlocked only
    let entries = vault.list_entries(Some("password"), Some(false)).unwrap();

    assert_eq!(entries.len(), 1, "Should find only unlocked password entry");
    assert_eq!(entries[0].0, "email_password");
    assert!(!entries[0].1);

    cleanup_test_vault();
}

#[test]
fn test_search_and_filter_no_results() {
    let vault = setup_test_vault().unwrap();

    // Search for "github" but filter locked only (github entries are unlocked)
    let entries = vault.list_entries(Some("github"), Some(true)).unwrap();

    assert_eq!(
        entries.len(),
        0,
        "Should return empty - no locked github entries"
    );

    cleanup_test_vault();
}

#[test]
fn test_empty_search_string() {
    let vault = setup_test_vault().unwrap();

    // Empty string should match all entries
    let entries = vault.list_entries(Some(""), None).unwrap();

    assert_eq!(entries.len(), 6, "Empty search should return all entries");

    cleanup_test_vault();
}

#[test]
fn test_search_with_special_characters() {
    let vault = setup_test_vault().unwrap();

    // Search for underscore
    let entries = vault.list_entries(Some("_"), None).unwrap();

    // Should find entries with underscores (github_token, aws_api_key, etc.)
    assert!(entries.len() >= 4, "Should find entries with underscores");

    cleanup_test_vault();
}

#[test]
fn test_list_entries_preserves_alphabetical_order() {
    let vault = setup_test_vault().unwrap();

    let entries = vault.list_entries(None, None).unwrap();

    // Entries should be returned in alphabetical order
    let keys: Vec<&str> = entries.iter().map(|e| e.0.as_str()).collect();

    let mut sorted_keys = keys.clone();
    sorted_keys.sort();

    assert_eq!(
        keys, sorted_keys,
        "Entries should be returned in alphabetical order"
    );

    cleanup_test_vault();
}

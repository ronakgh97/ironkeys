use crate::error::{Error, Result};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Entry stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub encrypted_value: String, // Base64-encoded
    pub nonce: String,           // Base64-encoded
    pub is_locked: bool,
}

/// Database file structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    pub master_salt: String, // Base64-encoded
    pub master_hash: String, // Base64-encoded
    pub iterations: u32,
    pub entries: HashMap<String, Entry>,
}

impl Database {
    /// Create a new database with master key info
    pub fn new(salt: Vec<u8>, hash: Vec<u8>, iterations: u32) -> Self {
        Self {
            master_salt: general_purpose::STANDARD.encode(&salt),
            master_hash: general_purpose::STANDARD.encode(&hash),
            iterations,
            entries: HashMap::new(),
        }
    }

    /// Get the decoded salt
    pub fn get_salt(&self) -> Result<Vec<u8>> {
        general_purpose::STANDARD
            .decode(&self.master_salt)
            .map_err(|e| Error::DatabaseLoadFailed(format!("Invalid salt: {e}")))
    }

    /// Get the decoded hash
    pub fn get_hash(&self) -> Result<Vec<u8>> {
        general_purpose::STANDARD
            .decode(&self.master_hash)
            .map_err(|e| Error::DatabaseLoadFailed(format!("Invalid hash: {e}")))
    }
}

impl Entry {
    /// Create a new entry from encrypted data
    pub fn new(encrypted_value: Vec<u8>, nonce: Vec<u8>, is_locked: bool) -> Self {
        Self {
            encrypted_value: general_purpose::STANDARD.encode(&encrypted_value),
            nonce: general_purpose::STANDARD.encode(&nonce),
            is_locked,
        }
    }

    /// Get the decoded encrypted value
    pub fn get_encrypted_value(&self) -> Result<Vec<u8>> {
        general_purpose::STANDARD
            .decode(&self.encrypted_value)
            .map_err(|e| Error::DecryptionFailed(format!("Invalid encrypted value: {e}")))
    }

    /// Get the decoded nonce
    pub fn get_nonce(&self) -> Result<Vec<u8>> {
        general_purpose::STANDARD
            .decode(&self.nonce)
            .map_err(|e| Error::DecryptionFailed(format!("Invalid nonce: {e}")))
    }
}

/// Get the database file path
pub fn get_database_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| Error::Io("Could not find config directory".to_string()))?
        .join("ironkey");

    Ok(config_dir.join("ironkey.json"))
}

/// Check if the database exists
pub fn exists() -> Result<bool> {
    let path = get_database_path()?;
    Ok(path.exists())
}

/// Load the database from disk
pub fn load() -> Result<Database> {
    let path = get_database_path()?;

    if !path.exists() {
        return Err(Error::DatabaseNotFound);
    }

    let content =
        fs::read_to_string(&path).map_err(|e| Error::DatabaseLoadFailed(e.to_string()))?;

    let database: Database =
        serde_json::from_str(&content).map_err(|e| Error::DatabaseLoadFailed(e.to_string()))?;

    Ok(database)
}

/// Save the database to disk
pub fn save(database: &Database) -> Result<()> {
    let path = get_database_path()?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::DatabaseSaveFailed(e.to_string()))?;
    }

    let content = serde_json::to_string_pretty(database)
        .map_err(|e| Error::DatabaseSaveFailed(e.to_string()))?;

    fs::write(&path, content).map_err(|e| Error::DatabaseSaveFailed(e.to_string()))?;

    Ok(())
}

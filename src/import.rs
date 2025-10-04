//! Import Module
//!
//! Handles importing vault entries from encrypted .ik export files.
//! Supports merge, replace, and diff (dry-run) strategies.

use crate::crypto::{self, EncryptedData};
use crate::error::{Error, Result};
use crate::export::{EXPORT_FORMAT_VERSION, ExportEntry, ExportFile};
use crate::storage::{Database, Entry};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use std::fs;
use std::path::Path;

/// Import Strategy Result
/// Contains information about what happened during import
#[derive(Debug)]
pub struct ImportResult {
    pub added: Vec<String>,
    pub updated: Vec<String>,
    pub skipped: Vec<String>,
    pub total_in_export: usize,
}

impl ImportResult {
    fn new(total_in_export: usize) -> Self {
        Self {
            added: Vec::new(),
            updated: Vec::new(),
            skipped: Vec::new(),
            total_in_export,
        }
    }
}

/// Import vault entries from an encrypted .ik file
///
/// # Arguments
/// * `import_path` - Path to the .ik file to import
/// * `import_password` - Password used to encrypt the export file
/// * `current_db` - Current database (will be modified based on strategy)
/// * `master_key` - Master key for encrypting entries in the destination vault
/// * `merge` - If true, add new entries but skip existing ones
/// * `replace` - If true, overwrite existing entries with imported ones
/// * `diff` - If true, dry-run mode (show what would be imported without making changes)
///
/// # Returns
/// * `Ok(ImportResult)` - Information about what was imported
/// * `Err(Error)` - If import fails
pub fn import_vault(
    import_path: &Path,
    import_password: String,
    current_db: &mut Database,
    master_key: &[u8],
    merge: bool,
    replace: bool,
    diff: bool,
) -> Result<ImportResult> {
    // Read and parse the export file
    let export_data = fs::read_to_string(import_path)
        .map_err(|e| Error::Io(format!("Failed to read import file: {e}")))?;

    let export_file: ExportFile = serde_json::from_str(&export_data)
        .map_err(|e| Error::Io(format!("Failed to parse import file: {e}")))?;

    // Validate format version
    if export_file.format_version != EXPORT_FORMAT_VERSION {
        return Err(Error::Io(format!(
            "Unsupported export format version: {} (expected {})",
            export_file.format_version, EXPORT_FORMAT_VERSION
        )));
    }

    // Derive encryption key from import password
    let salt = BASE64
        .decode(&export_file.encryption.salt)
        .map_err(|e| Error::Io(format!("Failed to decode salt: {e}")))?;

    let import_key =
        crypto::derive_key(&import_password, &salt, export_file.encryption.iterations)?;

    // Decrypt the exported data
    let nonce = BASE64
        .decode(&export_file.encryption.nonce)
        .map_err(|e| Error::Io(format!("Failed to decode nonce: {e}")))?;

    let ciphertext = BASE64
        .decode(&export_file.encrypted_data)
        .map_err(|e| Error::Io(format!("Failed to decode encrypted data: {e}")))?;

    let encrypted_data = EncryptedData { ciphertext, nonce };

    let decrypted_bytes = crypto::decrypt(&encrypted_data, &import_key)
        .map_err(|_| Error::Io("Failed to decrypt import file (wrong password?)".to_string()))?;

    // Parse the decrypted entries
    let decrypted_str = String::from_utf8(decrypted_bytes)
        .map_err(|e| Error::Io(format!("Failed to decode decrypted data: {e}")))?;

    let entries: Vec<ExportEntry> = serde_json::from_str(&decrypted_str)
        .map_err(|e| Error::Io(format!("Failed to parse decrypted entries: {e}")))?;

    // Initialize import result
    let mut result = ImportResult::new(entries.len());

    // Process each entry based on strategy
    for entry in entries {
        let key_exists = current_db.entries.contains_key(&entry.key);

        if key_exists {
            if merge {
                // Merge mode: skip existing entries
                result.skipped.push(entry.key.clone());
                continue;
            } else if replace {
                // Replace mode: update existing entry
                result.updated.push(entry.key.clone());
            }
        } else {
            // New entry
            result.added.push(entry.key.clone());
        }

        // If diff mode, don't actually modify the database
        if diff {
            continue; // Skip the actual encryption and insertion
        }

        // Encrypt the value with the destination vault's master key
        let encrypted_data = crypto::encrypt(entry.value.as_bytes(), master_key)?;

        // Encode to base64 for storage
        let encrypted_value_b64 = BASE64.encode(&encrypted_data.ciphertext);
        let nonce_b64 = BASE64.encode(&encrypted_data.nonce);

        // Create the entry
        let db_entry = Entry {
            encrypted_value: encrypted_value_b64,
            nonce: nonce_b64,
            is_locked: entry.locked,
        };

        // Insert or update the entry
        current_db.entries.insert(entry.key, db_entry);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_result_creation() {
        let result = ImportResult::new(5);
        assert_eq!(result.total_in_export, 5);
        assert_eq!(result.added.len(), 0);
        assert_eq!(result.updated.len(), 0);
        assert_eq!(result.skipped.len(), 0);
    }

    #[test]
    fn test_unsupported_format_version() {
        // This will be tested in integration tests
        // as it requires actual file I/O
    }
}

use crate::crypto;
use crate::error::{Error, Result};
use crate::storage::Database;
use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Format version for export files
pub const EXPORT_FORMAT_VERSION: &str = "1.0.0";

/// Encryption metadata for export file
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportEncryption {
    pub algorithm: String,
    pub salt: String,  // Base64-encoded
    pub nonce: String, // Base64-encoded
    pub iterations: u32,
}

/// Metadata about the export
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub exported_from: String,
    pub vault_name: Option<String>, // TODO: Multiple vaults support
    pub tags: Option<Vec<String>>,  // TODO: Tag filtering support
}

/// Export file structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportFile {
    pub format_version: String,
    pub exported_at: String, // ISO 8601 timestamp
    pub entry_count: usize,
    pub encryption: ExportEncryption,
    pub encrypted_data: String, // Base64-encoded encrypted entry data
    pub metadata: ExportMetadata,
}

/// Entry data in the decrypted export (internal)
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportEntry {
    pub key: String,
    pub value: String, // Decrypted value
    pub locked: bool,
}

/// Export vault entries to encrypted file
///
/// # Arguments
/// * `db` - Database to export
/// * `master_key` - Master key to decrypt entries
/// * `output_path` - Path where export file will be written
/// * `export_password` - Password to encrypt the export file
/// * `force` - Whether to overwrite existing file
///
/// # Security
/// - Uses same PBKDF2 + AES-256-GCM as vault
/// - Export password is independent of master password
/// - Each export has unique salt and nonce
pub fn export_vault(
    db: &Database,
    master_key: &[u8],
    output_path: &Path,
    export_password: String,
    force: bool,
) -> Result<()> {
    // Check if file exists (unless force is true)
    if !force && output_path.exists() {
        return Err(Error::Io(format!(
            "File '{}' already exists. Use --force to overwrite",
            output_path.display()
        )));
    }

    // Decrypt all entries from the vault
    let mut export_entries: Vec<ExportEntry> = Vec::new();

    for (key, entry) in &db.entries {
        // Decrypt the entry value using master key
        let encrypted_value = entry.get_encrypted_value()?;
        let nonce_bytes = entry.get_nonce()?;

        // Create EncryptedData struct for decryption
        let encrypted_data = crypto::EncryptedData {
            ciphertext: encrypted_value,
            nonce: nonce_bytes,
        };

        let decrypted_value = crypto::decrypt(&encrypted_data, master_key)?;
        let value = String::from_utf8(decrypted_value)
            .map_err(|e| Error::DecryptionFailed(format!("Invalid UTF-8: {e}")))?;

        export_entries.push(ExportEntry {
            key: key.clone(),
            value,
            locked: entry.is_locked,
        });
    }

    // Serialize entries to JSON
    let entries_json = serde_json::to_string(&export_entries)
        .map_err(|e| Error::Io(format!("Failed to serialize entries: {e}")))?;

    // Generate salt for export encryption
    let export_salt = crypto::generate_salt()?;
    let iterations = crypto::default_iterations();

    // Derive key from export password
    let export_key = crypto::derive_key(&export_password, &export_salt, iterations)?;

    // Encrypt the entries JSON
    let entries_bytes = entries_json.as_bytes();
    let encrypted = crypto::encrypt(entries_bytes, &export_key)?;

    // Create export file structure
    let export_file = ExportFile {
        format_version: EXPORT_FORMAT_VERSION.to_string(),
        exported_at: Utc::now().to_rfc3339(),
        entry_count: export_entries.len(),
        encryption: ExportEncryption {
            algorithm: "AES-256-GCM".to_string(),
            salt: general_purpose::STANDARD.encode(&export_salt),
            nonce: general_purpose::STANDARD.encode(&encrypted.nonce),
            iterations,
        },
        encrypted_data: general_purpose::STANDARD.encode(&encrypted.ciphertext),
        metadata: ExportMetadata {
            exported_from: format!("ironkey v{}", env!("CARGO_PKG_VERSION")),
            vault_name: None, // TODO: Multiple vaults
            tags: None,       // TODO: Tag filtering
        },
    };

    // Serialize to JSON and write to file
    let export_json = serde_json::to_string_pretty(&export_file)
        .map_err(|e| Error::Io(format!("Failed to serialize export file: {e}")))?;

    fs::write(output_path, export_json)
        .map_err(|e| Error::Io(format!("Failed to write export file: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_version() {
        assert_eq!(EXPORT_FORMAT_VERSION, "1.0.0");
    }

    #[test]
    fn test_export_entry_serialization() {
        let entry = ExportEntry {
            key: "test".to_string(),
            value: "password123".to_string(),
            locked: false,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: ExportEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.key, "test");
        assert_eq!(deserialized.value, "password123");
        assert!(!deserialized.locked);
    }

    #[test]
    fn test_export_file_structure() {
        let export_file = ExportFile {
            format_version: "1.0.0".to_string(),
            exported_at: "2025-10-03T10:00:00Z".to_string(),
            entry_count: 2,
            encryption: ExportEncryption {
                algorithm: "AES-256-GCM".to_string(),
                salt: "dGVzdHNhbHQ=".to_string(),
                nonce: "dGVzdG5vbmNl".to_string(),
                iterations: 100000,
            },
            encrypted_data: "ZW5jcnlwdGVkZGF0YQ==".to_string(),
            metadata: ExportMetadata {
                exported_from: "ironkey v0.1.0".to_string(),
                vault_name: None,
                tags: None,
            },
        };

        let json = serde_json::to_string_pretty(&export_file).unwrap();
        let deserialized: ExportFile = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.format_version, "1.0.0");
        assert_eq!(deserialized.entry_count, 2);
        assert_eq!(deserialized.encryption.algorithm, "AES-256-GCM");
        assert!(deserialized.metadata.vault_name.is_none());
    }
}

use crate::crypto::{self, EncryptedData};
use crate::error::{Error, Result};
use crate::storage::{self, Database, Entry};
use zeroize::Zeroize;

/// The Vault manages all password entries and master key operations
pub struct Vault {
    db: Database,
    master_key: Vec<u8>,
}

impl Vault {
    /// Initialize a new vault with a master password
    pub fn init(master_password: String) -> Result<Self> {
        // Check if database already exists
        if storage::exists()? {
            return Err(Error::MasterKeyAlreadyExists);
        }

        if master_password.trim().is_empty() {
            return Err(Error::EmptyPassword);
        }

        // Generate salt and derive key
        let salt = crypto::generate_salt()?;
        let iterations = crypto::default_iterations();
        let master_key = crypto::derive_key(&master_password, &salt, iterations)?;

        // Hash password for verification
        let master_hash = crypto::hash_password(&master_password, &salt, iterations)?;

        // Create database
        let db = Database::new(salt, master_hash, iterations);

        // Save to disk
        storage::save(&db)?;

        Ok(Self { db, master_key })
    }

    /// Unlock an existing vault with master password
    pub fn unlock(mut master_password: String) -> Result<Self> {
        // Load database
        let db = storage::load()?;

        // Get salt and hash
        let salt = db.get_salt()?;
        let stored_hash = db.get_hash()?;

        // Verify password
        let is_valid =
            crypto::verify_password(&master_password, &salt, &stored_hash, db.iterations)?;

        if !is_valid {
            master_password.zeroize();
            return Err(Error::InvalidMasterPassword);
        }

        // Derive encryption key
        let master_key = crypto::derive_key(&master_password, &salt, db.iterations)?;

        // Zeroize password
        master_password.zeroize();

        Ok(Self { db, master_key })
    }

    /// Verify that a master password is correct (for init command)
    pub fn verify_master_password(mut master_password: String) -> Result<bool> {
        let db = storage::load()?;
        let salt = db.get_salt()?;
        let stored_hash = db.get_hash()?;

        let result = crypto::verify_password(&master_password, &salt, &stored_hash, db.iterations)?;
        master_password.zeroize();

        Ok(result)
    }

    /// Create a new entry
    pub fn create_entry(&mut self, key: String, value: String) -> Result<()> {
        // Check if key already exists
        if self.db.entries.contains_key(&key) {
            return Err(Error::EntryAlreadyExists(key));
        }

        // Encrypt the value
        let encrypted = crypto::encrypt(value.as_bytes(), &self.master_key)?;

        // Create entry
        let entry = Entry::new(encrypted.ciphertext, encrypted.nonce, false);

        // Add to database
        self.db.entries.insert(key.clone(), entry);

        // Save to disk
        storage::save(&self.db)?;

        Ok(())
    }

    /// Get an entry's value
    pub fn get_entry(&self, key: &str) -> Result<String> {
        // Check if entry exists
        let entry = self
            .db
            .entries
            .get(key)
            .ok_or_else(|| Error::EntryNotFound(key.to_string()))?;

        // Check if entry is locked - prevent decryption if locked
        if entry.is_locked {
            return Err(Error::EntryLocked(key.to_string()));
        }

        // Decrypt the value
        let encrypted = EncryptedData {
            ciphertext: entry.get_encrypted_value()?,
            nonce: entry.get_nonce()?,
        };

        let decrypted = crypto::decrypt(&encrypted, &self.master_key)?;
        let value = String::from_utf8(decrypted)?;

        Ok(value)
    }

    /// Update an existing entry's value
    pub fn update_entry(&mut self, key: String, new_value: String) -> Result<()> {
        // Check if entry exists
        let entry = self
            .db
            .entries
            .get(&key)
            .ok_or_else(|| Error::EntryNotFound(key.to_string()))?;

        // Check if entry is locked - prevent updates if locked
        if entry.is_locked {
            return Err(Error::EntryLocked(key.to_string()));
        }

        // Encrypt the new value
        let encrypted = crypto::encrypt(new_value.as_bytes(), &self.master_key)?;

        // Update entry with new encrypted value
        let updated_entry = Entry::new(encrypted.ciphertext, encrypted.nonce, false);

        // Replace in database
        self.db.entries.insert(key, updated_entry);

        // Save to disk
        storage::save(&self.db)?;

        Ok(())
    }

    /// List entry keys with optional search and lock status filter
    ///
    /// # Arguments
    /// * `search` - Optional search string (case-insensitive, partial match)
    /// * `lock_filter` - Optional filter: Some(true) for locked only, Some(false) for unlocked only, None for all
    ///
    /// # Returns
    /// A Result containing a vector of tuples (key, is_locked) sorted alphabetically by key
    pub fn list_entries(
        &self,
        search: Option<&str>,
        lock_filter: Option<bool>,
    ) -> Result<Vec<(&String, bool)>> {
        let mut results: Vec<(&String, bool)> = self
            .db
            .entries
            .iter()
            .filter(|(key, entry)| {
                // Apply search filter (case-insensitive)
                let search_match = if let Some(search_term) = search {
                    key.to_lowercase().contains(&search_term.to_lowercase())
                } else {
                    true // No search filter, match all
                };

                // Apply lock status filter
                let lock_match = if let Some(required_lock_status) = lock_filter {
                    entry.is_locked == required_lock_status
                } else {
                    true // No lock filter, match all
                };

                // Entry must match both filters
                search_match && lock_match
            })
            .map(|(key, entry)| (key, entry.is_locked))
            .collect();

        // Sort alphabetically by key
        results.sort_by(|a, b| a.0.cmp(b.0));

        Ok(results)
    }

    /// Delete an entry
    pub fn delete_entry(&mut self, key: &str) -> Result<()> {
        // Check if entry exists
        let entry = self
            .db
            .entries
            .get(key)
            .ok_or_else(|| Error::EntryNotFound(key.to_string()))?;

        // Check if entry is locked - prevent deletion if locked
        if entry.is_locked {
            return Err(Error::EntryLocked(key.to_string()));
        }

        // Remove from database
        self.db.entries.remove(key);

        // Save to disk
        storage::save(&self.db)?;

        Ok(())
    }

    /// Toggle lock status of an entry
    pub fn toggle_lock(&mut self, key: &str) -> Result<bool> {
        // Check if entry exists
        let entry = self
            .db
            .entries
            .get_mut(key)
            .ok_or_else(|| Error::EntryNotFound(key.to_string()))?;

        // Toggle lock status
        entry.is_locked = !entry.is_locked;
        let new_status = entry.is_locked;

        // Save to disk
        storage::save(&self.db)?;

        Ok(new_status)
    }

    /// Save the vault (useful after multiple operations)
    #[allow(dead_code)] // Public API - may be used by external consumers
    pub fn save(&self) -> Result<()> {
        storage::save(&self.db)
    }
}

impl Drop for Vault {
    fn drop(&mut self) {
        // Zeroize master key when vault is dropped
        self.master_key.zeroize();
    }
}

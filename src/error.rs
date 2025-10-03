use std::fmt;

#[derive(Debug)]
pub enum Error {
    // Entry errors
    EntryNotFound(String),
    EntryAlreadyExists(String),
    EntryLocked(String),

    // Master password errors
    InvalidMasterPassword,
    #[allow(dead_code)] // Reserved for future use
    MasterKeyNotInitialized,
    MasterKeyAlreadyExists,
    EmptyPassword,

    // Crypto errors
    EncryptionFailed(String),
    DecryptionFailed(String),
    KeyDerivationFailed(String),

    // Storage errors
    DatabaseNotFound,
    DatabaseLoadFailed(String),
    DatabaseSaveFailed(String),

    // I/O errors
    Io(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EntryNotFound(key) => write!(f, "Entry '{key}' not found"),
            Error::EntryAlreadyExists(key) => write!(f, "Entry '{key}' already exists"),
            Error::EntryLocked(key) => write!(f, "Entry '{key}' is locked"),

            Error::InvalidMasterPassword => write!(f, "Invalid master password"),
            Error::MasterKeyNotInitialized => {
                write!(f, "Master key not initialized. Run 'ik init' first")
            }
            Error::MasterKeyAlreadyExists => {
                write!(f, "Master key already exists. Use 'ik init' to verify")
            }
            Error::EmptyPassword => write!(f, "Password cannot be empty"),

            Error::EncryptionFailed(msg) => write!(f, "Encryption failed: {msg}"),
            Error::DecryptionFailed(msg) => write!(f, "Decryption failed: {msg}"),
            Error::KeyDerivationFailed(msg) => write!(f, "Key derivation failed: {msg}"),

            Error::DatabaseNotFound => write!(f, "Database not found. Run 'ik init' first"),
            Error::DatabaseLoadFailed(msg) => write!(f, "Failed to load database: {msg}"),
            Error::DatabaseSaveFailed(msg) => write!(f, "Failed to save database: {msg}"),

            Error::Io(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

// Conversions from other error types
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::DatabaseLoadFailed(err.to_string())
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Self {
        Error::DecryptionFailed(format!("Base64 decode error: {err}"))
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::DecryptionFailed(format!("UTF-8 decode error: {err}"))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

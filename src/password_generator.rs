use crate::error::{Error, Result};
use ring::rand::{SecureRandom, SystemRandom};

/// Character sets for password generation
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

/// Build character set based on selected options
///
/// Returns a string containing all allowed characters based on the flags.
pub fn build_charset(
    use_lowercase: bool,
    use_uppercase: bool,
    use_numbers: bool,
    use_symbols: bool,
) -> String {
    let mut charset = String::new();

    if use_lowercase {
        charset.push_str(LOWERCASE);
    }
    if use_uppercase {
        charset.push_str(UPPERCASE);
    }
    if use_numbers {
        charset.push_str(NUMBERS);
    }
    if use_symbols {
        charset.push_str(SYMBOLS);
    }

    charset
}

/// Generate a cryptographically secure random password
///
/// # Arguments
/// * `length` - Length of the password to generate (must be > 0)
/// * `use_lowercase` - Include lowercase letters (a-z)
/// * `use_uppercase` - Include uppercase letters (A-Z)
/// * `use_numbers` - Include numbers (0-9)
/// * `use_symbols` - Include symbols (!@#$%^&*()_+-=[]{}|;:,.<>?)
///
/// # Returns
/// A randomly generated password string
///
/// # Errors
/// Returns an error if:
/// - `length` is 0
/// - No character types are selected
/// - Random number generation fails
///
/// # Examples
/// ```
/// use ironkey::password_generator;
///
/// // Generate a 16-character password with all character types
/// let password = password_generator::generate(16, true, true, true, true).unwrap();
/// assert_eq!(password.len(), 16);
///
/// // Generate a 20-character alphanumeric password (no symbols)
/// let password = password_generator::generate(20, true, true, true, false).unwrap();
/// assert_eq!(password.len(), 20);
/// ```
pub fn generate(
    length: usize,
    use_lowercase: bool,
    use_uppercase: bool,
    use_numbers: bool,
    use_symbols: bool,
) -> Result<String> {
    // Validate length
    if length == 0 {
        return Err(Error::InvalidInput(
            "Password length must be greater than 0".to_string(),
        ));
    }

    // Build character set
    let charset = build_charset(use_lowercase, use_uppercase, use_numbers, use_symbols);

    // Validate character set
    if charset.is_empty() {
        return Err(Error::InvalidInput(
            "At least one character type must be selected".to_string(),
        ));
    }

    // Generate random password
    let charset_bytes: Vec<u8> = charset.bytes().collect();
    let charset_len = charset_bytes.len();

    let rng = SystemRandom::new();
    let mut password = String::with_capacity(length);

    // Generate random bytes and map to characters from charset
    let mut random_bytes = vec![0u8; length];
    rng.fill(&mut random_bytes)
        .map_err(|_| Error::Io("Failed to generate random bytes".to_string()))?;

    for byte in random_bytes {
        // Map random byte to charset index
        let index = (byte as usize) % charset_len;
        password.push(charset_bytes[index] as char);
    }

    Ok(password)
}

/// Generate a password with default settings (16 characters, all types)
///
/// Convenience function for generating a password with sensible defaults.
#[allow(dead_code)] // Reserved for library API users
pub fn generate_default() -> Result<String> {
    generate(16, true, true, true, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charset_building() {
        let all = build_charset(true, true, true, true);
        assert!(all.contains('a'));
        assert!(all.contains('Z'));
        assert!(all.contains('5'));
        assert!(all.contains('!'));

        let alpha_only = build_charset(true, true, false, false);
        assert_eq!(alpha_only.len(), 52);
        assert!(!alpha_only.contains('5'));

        let empty = build_charset(false, false, false, false);
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_generate_basic() {
        let password = generate(16, true, true, true, true).unwrap();
        assert_eq!(password.len(), 16);
    }

    #[test]
    fn test_generate_default_convenience() {
        let password = generate_default().unwrap();
        assert_eq!(password.len(), 16);
    }
}

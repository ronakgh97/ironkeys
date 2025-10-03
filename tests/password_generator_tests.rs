use ironkey::password_generator;

#[test]
fn test_generate_default_password() {
    let password = password_generator::generate(16, true, true, true, true)
        .expect("Failed to generate password");

    assert_eq!(password.len(), 16, "Password should be 16 characters");
    assert!(!password.is_empty(), "Password should not be empty");
}

#[test]
fn test_generate_different_lengths() {
    let lengths = vec![8, 12, 16, 20, 32, 64];

    for length in lengths {
        let password = password_generator::generate(length, true, true, true, true)
            .expect("Failed to generate password");

        assert_eq!(
            password.len(),
            length,
            "Password length should match requested length"
        );
    }
}

#[test]
fn test_generate_lowercase_only() {
    let password = password_generator::generate(20, true, false, false, false)
        .expect("Failed to generate password");

    assert_eq!(password.len(), 20);
    assert!(
        password.chars().all(|c| c.is_ascii_lowercase()),
        "Password should only contain lowercase letters"
    );
}

#[test]
fn test_generate_uppercase_only() {
    let password = password_generator::generate(20, false, true, false, false)
        .expect("Failed to generate password");

    assert_eq!(password.len(), 20);
    assert!(
        password.chars().all(|c| c.is_ascii_uppercase()),
        "Password should only contain uppercase letters"
    );
}

#[test]
fn test_generate_numbers_only() {
    let password = password_generator::generate(20, false, false, true, false)
        .expect("Failed to generate password");

    assert_eq!(password.len(), 20);
    assert!(
        password.chars().all(|c| c.is_ascii_digit()),
        "Password should only contain numbers"
    );
}

#[test]
fn test_generate_symbols_only() {
    let password = password_generator::generate(20, false, false, false, true)
        .expect("Failed to generate password");

    assert_eq!(password.len(), 20);
    // Symbols are: !@#$%^&*()_+-=[]{}|;:,.<>?
    let is_symbol = |c: char| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c);
    assert!(
        password.chars().all(is_symbol),
        "Password should only contain symbols"
    );
}

#[test]
fn test_generate_mixed_character_types() {
    let password = password_generator::generate(100, true, true, true, true)
        .expect("Failed to generate password");

    assert_eq!(password.len(), 100);

    // With 100 characters and all types enabled, we should have at least one of each
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let is_symbol = |c: char| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c);
    let has_symbol = password.chars().any(is_symbol);

    assert!(
        has_lowercase,
        "Password should contain at least one lowercase letter"
    );
    assert!(
        has_uppercase,
        "Password should contain at least one uppercase letter"
    );
    assert!(has_digit, "Password should contain at least one digit");
    assert!(has_symbol, "Password should contain at least one symbol");
}

#[test]
fn test_generate_uniqueness() {
    // Generate multiple passwords and ensure they're different
    let mut passwords = std::collections::HashSet::new();

    for _ in 0..100 {
        let password = password_generator::generate(16, true, true, true, true)
            .expect("Failed to generate password");
        passwords.insert(password);
    }

    // All 100 passwords should be unique
    assert_eq!(passwords.len(), 100, "Generated passwords should be unique");
}

#[test]
fn test_generate_minimum_length() {
    // Test minimum length (1 character)
    let password =
        password_generator::generate(1, true, false, false, false).expect("Failed to generate");

    assert_eq!(password.len(), 1);
    assert!(password.chars().all(|c| c.is_ascii_lowercase()));
}

#[test]
fn test_generate_error_no_character_types() {
    // Should fail if no character types are selected
    let result = password_generator::generate(16, false, false, false, false);

    assert!(
        result.is_err(),
        "Should return error when no character types selected"
    );

    if let Err(e) = result {
        assert!(
            e.to_string().contains("character type"),
            "Error should mention character types"
        );
    }
}

#[test]
fn test_generate_error_zero_length() {
    // Should fail if length is 0
    let result = password_generator::generate(0, true, true, true, true);

    assert!(result.is_err(), "Should return error when length is 0");

    if let Err(e) = result {
        assert!(
            e.to_string().contains("length"),
            "Error should mention length"
        );
    }
}

#[test]
fn test_generate_large_password() {
    // Test generating a very large password (1024 characters)
    let password = password_generator::generate(1024, true, true, true, true)
        .expect("Failed to generate large password");

    assert_eq!(password.len(), 1024);
}

#[test]
fn test_generate_alphanumeric_only() {
    // Test with lowercase + uppercase + numbers (no symbols)
    let password = password_generator::generate(50, true, true, true, false)
        .expect("Failed to generate password");

    assert_eq!(password.len(), 50);

    let is_alphanumeric = |c: char| c.is_ascii_alphanumeric();
    assert!(
        password.chars().all(is_alphanumeric),
        "Password should only contain alphanumeric characters"
    );
}

#[test]
fn test_character_set_building() {
    // Test that character sets are built correctly
    let charset_all = password_generator::build_charset(true, true, true, true);
    let charset_alpha = password_generator::build_charset(true, true, false, false);
    let charset_none = password_generator::build_charset(false, false, false, false);

    // All character types (26 + 26 + 10 + 26 = 88)
    assert_eq!(
        charset_all.len(),
        88,
        "Full charset should have exactly 88 characters"
    );

    // Alpha only (26 + 26 = 52)
    assert_eq!(
        charset_alpha.len(),
        52,
        "Alpha charset should have 52 chars"
    );

    // None
    assert_eq!(charset_none.len(), 0, "Empty charset should be empty");
}

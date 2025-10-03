// Clipboard functionality tests

use ironkey::clipboard::{copy_to_clipboard, get_from_clipboard};

#[test]
fn test_clipboard_copy_and_retrieve() {
    let test_text = "test_password_123";

    // Copy to clipboard
    let copy_result = copy_to_clipboard(test_text);
    assert!(copy_result.is_ok(), "Failed to copy to clipboard");

    // Retrieve from clipboard
    let retrieved = get_from_clipboard();
    assert!(retrieved.is_ok(), "Failed to retrieve from clipboard");
    assert_eq!(retrieved.unwrap(), test_text);
}

#[test]
fn test_clipboard_with_special_characters() {
    let special_text = "p@ssw0rd!#$%^&*()_+-=[]{}|;':\",./<>?`~";

    copy_to_clipboard(special_text).unwrap();
    let retrieved = get_from_clipboard().unwrap();

    assert_eq!(retrieved, special_text);
}

#[test]
fn test_clipboard_with_unicode() {
    let unicode_text = "🔐 секретный_пароль 密码 🔑";

    copy_to_clipboard(unicode_text).unwrap();
    let retrieved = get_from_clipboard().unwrap();

    assert_eq!(retrieved, unicode_text);
}

#[test]
fn test_clipboard_with_multiline() {
    let multiline_text = "line1\nline2\nline3\r\ntab\there";

    copy_to_clipboard(multiline_text).unwrap();
    let retrieved = get_from_clipboard().unwrap();

    assert_eq!(retrieved, multiline_text);
}

#[test]
fn test_clipboard_with_empty_string() {
    let empty_text = "";

    copy_to_clipboard(empty_text).unwrap();
    let retrieved = get_from_clipboard().unwrap();

    assert_eq!(retrieved, empty_text);
}

#[test]
fn test_clipboard_large_text() {
    // Test with 1KB of text
    let large_text = "a".repeat(1024);

    copy_to_clipboard(&large_text).unwrap();
    let retrieved = get_from_clipboard().unwrap();

    assert_eq!(retrieved, large_text);
}

#[test]
fn test_clipboard_overwrites_previous() {
    let first_text = "first_password";
    let second_text = "second_password";

    // Copy first text
    copy_to_clipboard(first_text).unwrap();
    let first_retrieved = get_from_clipboard().unwrap();
    assert_eq!(first_retrieved, first_text);

    // Copy second text (should overwrite)
    copy_to_clipboard(second_text).unwrap();
    let second_retrieved = get_from_clipboard().unwrap();
    assert_eq!(second_retrieved, second_text);
}

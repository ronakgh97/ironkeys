use ironkey::clipboard;
use std::thread;
use std::time::Duration;

#[test]
fn test_auto_clear_after_timeout() {
    // Copy text to clipboard
    let test_value = "auto_clear_test_123";
    clipboard::copy_to_clipboard(test_value).expect("Failed to copy");

    // Verify it's there
    let clipboard_content = clipboard::get_from_clipboard().expect("Failed to read clipboard");
    assert_eq!(clipboard_content, test_value);

    // Auto-clear with 1 second timeout
    clipboard::auto_clear_clipboard(test_value, Duration::from_secs(1)).expect("Auto-clear failed");

    // Wait for timeout
    thread::sleep(Duration::from_millis(1200));

    // Clipboard should be cleared (empty or different content)
    let clipboard_after = clipboard::get_from_clipboard().unwrap_or_default();
    assert_ne!(
        clipboard_after, test_value,
        "Clipboard should be cleared after timeout"
    );
}

#[test]
fn test_auto_clear_only_if_matches() {
    // Copy original text
    let original_value = "original_clipboard_content";
    clipboard::copy_to_clipboard(original_value).expect("Failed to copy");

    // Start auto-clear for different value
    let expected_value = "different_value_to_monitor";
    clipboard::auto_clear_clipboard(expected_value, Duration::from_secs(1))
        .expect("Auto-clear failed");

    // Wait for timeout
    thread::sleep(Duration::from_millis(1200));

    // Original clipboard content should still be there (not cleared)
    let clipboard_after = clipboard::get_from_clipboard().expect("Failed to read clipboard");
    assert_eq!(
        clipboard_after, original_value,
        "Should not clear clipboard if value doesn't match"
    );
}

#[test]
fn test_auto_clear_with_zero_timeout() {
    // Copy text to clipboard
    let test_value = "instant_clear_test";
    clipboard::copy_to_clipboard(test_value).expect("Failed to copy");

    // Verify it's there
    let clipboard_content = clipboard::get_from_clipboard().expect("Failed to read clipboard");
    assert_eq!(clipboard_content, test_value);

    // Auto-clear with 0 second timeout (immediate)
    clipboard::auto_clear_clipboard(test_value, Duration::from_secs(0)).expect("Auto-clear failed");

    // Small delay to let thread execute
    thread::sleep(Duration::from_millis(100));

    // Clipboard should be cleared
    let clipboard_after = clipboard::get_from_clipboard().unwrap_or_default();
    assert_ne!(
        clipboard_after, test_value,
        "Clipboard should be cleared immediately with 0 timeout"
    );
}

#[test]
fn test_auto_clear_with_user_overwrite() {
    // Copy text to clipboard
    let original_value = "will_be_overwritten";
    clipboard::copy_to_clipboard(original_value).expect("Failed to copy");

    // Start auto-clear with 2 second timeout
    clipboard::auto_clear_clipboard(original_value, Duration::from_secs(2))
        .expect("Auto-clear failed");

    // Simulate user copying something else after 500ms
    thread::sleep(Duration::from_millis(500));
    let user_value = "user_copied_this";
    clipboard::copy_to_clipboard(user_value).expect("Failed to copy user content");

    // Wait for original timeout to expire
    thread::sleep(Duration::from_millis(1700));

    // User's clipboard content should still be there (not cleared)
    let clipboard_after = clipboard::get_from_clipboard().expect("Failed to read clipboard");
    assert_eq!(
        clipboard_after, user_value,
        "Should not clear user's clipboard content"
    );
}

#[test]
fn test_multiple_auto_clear_operations() {
    // Test that multiple auto-clear operations don't interfere

    // First operation
    let value1 = "first_value_12345";
    clipboard::copy_to_clipboard(value1).expect("Failed to copy value1");
    clipboard::auto_clear_clipboard(value1, Duration::from_millis(800))
        .expect("Auto-clear 1 failed");

    // Wait a bit, then start second operation
    thread::sleep(Duration::from_millis(200));
    let value2 = "second_value_67890";
    clipboard::copy_to_clipboard(value2).expect("Failed to copy value2");
    clipboard::auto_clear_clipboard(value2, Duration::from_millis(800))
        .expect("Auto-clear 2 failed");

    // Wait for first timeout to pass
    thread::sleep(Duration::from_millis(700));

    // Clipboard should still have value2 (first timeout should not clear it)
    let clipboard_mid = clipboard::get_from_clipboard().expect("Failed to read clipboard");
    assert_eq!(
        clipboard_mid, value2,
        "Second value should still be in clipboard"
    );

    // Wait for second timeout
    thread::sleep(Duration::from_millis(400));

    // Now clipboard should be cleared
    let clipboard_final = clipboard::get_from_clipboard().unwrap_or_default();
    assert_ne!(
        clipboard_final, value2,
        "Clipboard should be cleared after second timeout"
    );
}

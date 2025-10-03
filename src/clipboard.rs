use crate::error::{Error, Result};
use arboard::Clipboard;
use std::thread;
use std::time::Duration;

/// Copy text to the system clipboard
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard =
        Clipboard::new().map_err(|e| Error::Io(format!("Failed to access clipboard: {e}")))?;

    clipboard
        .set_text(text.to_string())
        .map_err(|e| Error::Io(format!("Failed to copy to clipboard: {e}")))?;

    Ok(())
}

/// Get text from the system clipboard
///
/// **Note**: Currently used for testing clipboard functionality.
/// May be useful for future features (e.g., paste command).
#[allow(dead_code)]
pub fn get_from_clipboard() -> Result<String> {
    let mut clipboard =
        Clipboard::new().map_err(|e| Error::Io(format!("Failed to access clipboard: {e}")))?;

    clipboard
        .get_text()
        .map_err(|e| Error::Io(format!("Failed to read from clipboard: {e}")))
}

/// Auto-clear clipboard after timeout if it still contains the expected value
///
/// This function spawns a background thread that waits for the specified duration,
/// then clears the clipboard ONLY if it still contains the original value.
/// This prevents accidentally clearing user's clipboard if they copied something else.
///
/// # Arguments
/// * `expected_value` - The value that should be cleared (won't clear if clipboard changed)
/// * `timeout` - Duration to wait before clearing
///
/// # Security Note
/// This prevents clipboard persistence of sensitive data while respecting user's clipboard usage.
pub fn auto_clear_clipboard(expected_value: &str, timeout: Duration) -> Result<()> {
    let expected = expected_value.to_string();

    // Spawn background thread to clear after timeout
    thread::spawn(move || {
        // Wait for timeout
        thread::sleep(timeout);

        // Only clear if clipboard still contains our value
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Ok(current_value) = clipboard.get_text() {
                if current_value == expected {
                    // Clear clipboard by setting empty string
                    let _ = clipboard.set_text(String::new());
                }
            }
        }
    });

    Ok(())
}

use crate::error::{Error, Result};
use arboard::Clipboard;

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

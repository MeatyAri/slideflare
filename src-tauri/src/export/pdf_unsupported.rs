//! PDF export fallback for platforms with no webview print pipeline wired up.
//!
//! The HTML export still works everywhere, so this reports rather than panics
//! and the frontend offers that instead.

use std::path::PathBuf;

use tauri::WebviewWindow;

pub fn print_to_pdf(_window: &WebviewWindow, _path: PathBuf) -> Result<(), String> {
    Err("PDF export is not supported on this platform. Export to HTML instead.".to_string())
}

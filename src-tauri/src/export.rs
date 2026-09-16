//! Deck export: self-contained HTML and PDF.
//!
//! The two exports take deliberately different routes.
//!
//! **HTML** is built entirely in the frontend — only the frontend can read the
//! live stylesheets that Tailwind's browser build generates for the dynamic
//! frontmatter classes. Rust just writes the finished string to disk, which is
//! why [`write_export`] is the whole of it.
//!
//! **PDF** drives the platform webview's own print pipeline against the *live*
//! deck. Printing an offscreen or freshly-created webview would be tidier, but
//! nothing guarantees such a widget has been realized and laid out, and WebKit
//! in particular gives no assurance that it prints correctly before then. The
//! visible deck webview is already realized, loaded, and laid out with fonts
//! resolved and MathML measured, so it produces the highest-fidelity output.
//!
//! The frontend puts the deck into print mode (fixed paper-sized slides, nav
//! hidden) before invoking [`export_pdf`], and restores it when one of the two
//! completion events arrives.

use std::fs;
use std::path::PathBuf;

use tauri::Emitter;

use crate::watcher::AppState;

#[cfg_attr(
    any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ),
    path = "export/pdf_linux.rs"
)]
#[cfg_attr(windows, path = "export/pdf_windows.rs")]
#[cfg_attr(target_os = "macos", path = "export/pdf_macos.rs")]
#[cfg_attr(
    not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        windows,
        target_os = "macos"
    )),
    path = "export/pdf_unsupported.rs"
)]
mod pdf;

/// Emitted once the PDF has been written. Payload is the output path.
pub const EVENT_PDF_DONE: &str = "pdf-export-done";
/// Emitted when the print pipeline refused or failed. Payload is a message.
pub const EVENT_PDF_FAILED: &str = "pdf-export-failed";

/// Slide paper size, in PostScript points.
///
/// The deck is laid out at a fixed 1280x720 CSS px design resolution (see
/// `DESIGN_W`/`DESIGN_H` in the frontend). Expressing the page in points keeps
/// that exact: 960pt / 72 * 96 = 1280px and 540pt = 720px, with no rounding, so
/// a slide fills its page precisely and never spills a blank page after itself.
/// Inches are derived from these rather than written out, for the same reason.
pub const PAGE_WIDTH_PT: f64 = 960.0;
pub const PAGE_HEIGHT_PT: f64 = 540.0;

/// Write an already-rendered export to disk.
///
/// The path comes from the frontend's save dialog, so it is a location the user
/// picked themselves.
#[tauri::command]
pub async fn write_export(path: String, contents: String) -> Result<(), String> {
    let path = PathBuf::from(path);

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Could not create {}: {}", parent.display(), e))?;
        }
    }

    fs::write(&path, contents).map_err(|e| format!("Could not write {}: {}", path.display(), e))
}

/// Path of the Markdown file currently open, if any.
///
/// Used by the frontend to name the export and title the generated document.
#[tauri::command]
pub async fn current_file_path(
    app_state: tauri::State<'_, AppState>,
) -> Result<Option<String>, String> {
    Ok(app_state.file_path())
}

/// Print the live deck webview to a PDF at `path`.
///
/// Returns as soon as the print has been *started*; on every platform the
/// outcome arrives later as [`EVENT_PDF_DONE`] or [`EVENT_PDF_FAILED`]. An
/// `Err` here means the print could not be started at all.
#[tauri::command]
pub async fn export_pdf(window: tauri::WebviewWindow, path: String) -> Result<(), String> {
    let path = PathBuf::from(path);

    // GTK's print backend writes to the output URI without creating anything
    // along the way, and fails late and quietly if the directory is missing.
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Could not create {}: {}", parent.display(), e))?;
        }
    }

    pdf::print_to_pdf(&window, path)
}

/// Report a finished print back to the frontend.
///
/// Failures are surfaced as an event rather than swallowed, because by this
/// point the deck is still sitting in print mode and only the frontend can take
/// it back out.
pub(crate) fn emit_result(window: &tauri::WebviewWindow, result: Result<String, String>) {
    let _ = match result {
        Ok(path) => window.emit(EVENT_PDF_DONE, path),
        Err(message) => window.emit(EVENT_PDF_FAILED, message),
    };
}

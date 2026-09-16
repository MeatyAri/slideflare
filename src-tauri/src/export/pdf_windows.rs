//! PDF export on WebView2.
//!
//! WebView2 has a first-class `PrintToPdf`, so this is the most direct of the
//! three backends. Two capability notes:
//!
//!   - `PrintToPdf` arrived in `ICoreWebView2_7` and `CreatePrintSettings` in
//!     `ICoreWebView2Environment6`, so both are reached by `cast()` from the
//!     handles Tauri hands out. An older WebView2 runtime fails that cast, and
//!     that is reported as such rather than as a generic failure.
//!   - Backgrounds are off by default when printing. Slide background colours
//!     come from frontmatter and are the whole point, so they are switched on
//!     explicitly here as well as in the print stylesheet.

use std::path::PathBuf;

use tauri::WebviewWindow;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2Environment6, ICoreWebView2PrintSettings, ICoreWebView2_7,
    COREWEBVIEW2_PRINT_ORIENTATION_PORTRAIT,
};
use webview2_com::PrintToPdfCompletedHandler;
use windows_core::{Interface, HSTRING, PCWSTR};

use super::{emit_result, PAGE_HEIGHT_PT, PAGE_WIDTH_PT};

/// WebView2 measures pages in inches.
const POINTS_PER_INCH: f64 = 72.0;

pub fn print_to_pdf(window: &WebviewWindow, path: PathBuf) -> Result<(), String> {
    let reported_path = path.to_string_lossy().into_owned();
    let window = window.clone();
    // A second handle: the closure below takes ownership of `window`.
    let handle = window.clone();

    handle
        .with_webview(move |platform| {
            let started = unsafe { start(&platform, &window, &reported_path) };
            // A failure to *start* still has to reach the frontend, which is
            // holding the deck in print mode waiting for an event either way.
            if let Err(message) = started {
                emit_result(&window, Err(message));
            }
        })
        .map_err(|e| format!("Could not reach the webview to print: {}", e))
}

unsafe fn start(
    platform: &tauri::webview::PlatformWebview,
    window: &WebviewWindow,
    path: &str,
) -> Result<(), String> {
    let core = platform
        .controller()
        .CoreWebView2()
        .map_err(|e| format!("Could not get the WebView2 core: {}", e))?;

    let webview: ICoreWebView2_7 = core.cast().map_err(|_| {
        "This WebView2 runtime is too old to export PDFs. Update Microsoft Edge WebView2 \
         (runtime 1.0.1108.44 or newer) and try again."
            .to_string()
    })?;

    let environment: ICoreWebView2Environment6 = platform.environment().cast().map_err(|_| {
        "This WebView2 runtime is too old to configure PDF page size. Update Microsoft Edge \
         WebView2 (runtime 1.0.1108.44 or newer) and try again."
            .to_string()
    })?;

    let settings = environment
        .CreatePrintSettings()
        .map_err(|e| format!("Could not create print settings: {}", e))?;
    configure(&settings).map_err(|e| format!("Could not configure print settings: {}", e))?;

    let completion_window = window.clone();
    let completion_path = path.to_string();
    let handler = PrintToPdfCompletedHandler::create(Box::new(move |result, success| {
        let outcome = match result {
            Err(e) => Err(format!("Printing failed: {}", e)),
            Ok(()) if success => Ok(completion_path.clone()),
            Ok(()) => Err("Printing failed: WebView2 declined to produce the PDF.".to_string()),
        };
        emit_result(&completion_window, outcome);
        Ok(())
    }));

    let path = HSTRING::from(path);
    webview
        .PrintToPdf(PCWSTR(path.as_ptr()), &settings, &handler)
        .map_err(|e| format!("Could not start printing: {}", e))
}

unsafe fn configure(settings: &ICoreWebView2PrintSettings) -> windows_core::Result<()> {
    // The page is wider than it is tall. Rather than relying on the landscape
    // orientation flag to swap the two, the dimensions are given in the order
    // they are wanted and the orientation left at portrait.
    settings.SetOrientation(COREWEBVIEW2_PRINT_ORIENTATION_PORTRAIT)?;
    settings.SetPageWidth(PAGE_WIDTH_PT / POINTS_PER_INCH)?;
    settings.SetPageHeight(PAGE_HEIGHT_PT / POINTS_PER_INCH)?;
    settings.SetMarginTop(0.0)?;
    settings.SetMarginBottom(0.0)?;
    settings.SetMarginLeft(0.0)?;
    settings.SetMarginRight(0.0)?;
    settings.SetScaleFactor(1.0)?;
    settings.SetShouldPrintBackgrounds(true)?;
    settings.SetShouldPrintHeaderAndFooter(false)?;
    settings.SetShouldPrintSelectionOnly(false)?;
    Ok(())
}

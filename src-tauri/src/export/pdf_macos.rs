//! PDF export on WKWebView.
//!
//! Note which API this uses. `WKWebView.createPDF(configuration:)` looks like
//! the obvious choice and is the wrong one: `WKPDFConfiguration` carries only a
//! capture rect and a transparency flag, so it snapshots the content area into
//! a single continuous page and ignores CSS `@page` and print stylesheets
//! entirely. A deck needs one page per slide, so this goes through
//! `printOperationWithPrintInfo:` instead — WebKit's real print pipeline, the
//! same one Safari's File > Print drives, which paginates properly.
//!
//! Saving straight to a file without a panel is an `NSPrintInfo` job
//! disposition of `NSPrintSaveJob` plus an `NSPrintJobSavingURL` attribute.

use std::path::PathBuf;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::{NSPrintInfo, NSPrintJobSavingURL, NSPrintSaveJob, NSPrintingPaginationMode};
use objc2_foundation::{NSSize, NSString, NSURL};
use objc2_web_kit::WKWebView;
use tauri::WebviewWindow;

use super::{emit_result, PAGE_HEIGHT_PT, PAGE_WIDTH_PT};

pub fn print_to_pdf(window: &WebviewWindow, path: PathBuf) -> Result<(), String> {
    let reported_path = path.to_string_lossy().into_owned();
    let window = window.clone();
    // A second handle: the closure below takes ownership of `window`.
    let handle = window.clone();

    handle
        .with_webview(move |platform| {
            let outcome = unsafe { run(platform.inner(), &reported_path) };
            emit_result(&window, outcome);
        })
        .map_err(|e| format!("Could not reach the webview to print: {}", e))
}

unsafe fn run(webview: *mut std::ffi::c_void, path: &str) -> Result<String, String> {
    let webview = Retained::retain(webview.cast::<WKWebView>())
        .ok_or_else(|| "Could not get the WKWebView to print.".to_string())?;

    let print_info = NSPrintInfo::new();
    print_info.setPaperSize(NSSize::new(PAGE_WIDTH_PT, PAGE_HEIGHT_PT));
    print_info.setTopMargin(0.0);
    print_info.setBottomMargin(0.0);
    print_info.setLeftMargin(0.0);
    print_info.setRightMargin(0.0);
    print_info.setHorizontallyCentered(false);
    print_info.setVerticallyCentered(false);
    // Slides are already sized to exactly one page each, so let the content
    // paginate as laid out rather than rescaling it to fit.
    print_info.setHorizontalPagination(NSPrintingPaginationMode::Automatic);
    print_info.setVerticalPagination(NSPrintingPaginationMode::Automatic);

    // Save to a file instead of spooling to a printer.
    print_info.setJobDisposition(NSPrintSaveJob);
    let url = NSURL::fileURLWithPath(&NSString::from_str(path));
    let target: &AnyObject = &url;
    print_info
        .dictionary()
        .setObject_forKey(target, ProtocolObject::from_ref(NSPrintJobSavingURL));

    let operation = webview.printOperationWithPrintInfo(&print_info);
    operation.setShowsPrintPanel(false);
    operation.setShowsProgressPanel(false);
    operation.setJobTitle(Some(&NSString::from_str("SlideFlare deck")));

    if operation.runOperation() {
        Ok(path.to_string())
    } else {
        Err("macOS refused to produce the PDF.".to_string())
    }
}

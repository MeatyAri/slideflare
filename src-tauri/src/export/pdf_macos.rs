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
//!
//! The operation is started with `runOperationModalForWindow:` rather than the
//! simpler `runOperation()`. That is not a style choice — `runOperation()` was
//! measured hanging indefinitely on a CI runner. It blocks the calling thread in
//! a nested run loop while WebKit asks its web content process for a page count,
//! and that reply was never getting serviced, so the call never returned. The
//! modal form schedules the job and returns immediately, leaving the main run
//! loop free to deliver the reply, and reports the outcome through a delegate
//! callback. Everything else about this file was ruled out first: the runner
//! resolves a real `NSPrinter`, the save URL and job disposition are both set
//! correctly, and pinning the job to the calling thread changed nothing.

use std::ffi::c_void;
use std::path::PathBuf;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{define_class, msg_send, sel, AnyThread, DefinedClass};
use objc2_app_kit::{NSPrintInfo, NSPrintJobSavingURL, NSPrintSaveJob, NSPrintingPaginationMode};
use objc2_foundation::{NSObject, NSObjectProtocol, NSSize, NSString, NSURL};
use objc2_web_kit::WKWebView;
use tauri::WebviewWindow;

use super::{emit_result, PAGE_HEIGHT_PT, PAGE_WIDTH_PT};

/// What the delegate needs in order to report the outcome.
struct PrintDelegateIvars {
    window: WebviewWindow,
    path: String,
}

define_class!(
    // SAFETY:
    // - `NSObject` imposes no subclassing requirements.
    // - `PrintDelegate` does not implement `Drop`.
    #[unsafe(super(NSObject))]
    #[name = "SlideFlarePrintDelegate"]
    #[ivars = PrintDelegateIvars]
    struct PrintDelegate;

    impl PrintDelegate {
        /// AppKit's completion callback for `runOperationModalForWindow:`.
        ///
        /// The selector name and signature are fixed by AppKit; getting either
        /// wrong means this is simply never called and the export hangs until
        /// the watchdog fires.
        #[unsafe(method(printOperationDidRun:success:contextInfo:))]
        fn print_operation_did_run(
            &self,
            _operation: *mut AnyObject,
            success: bool,
            _context: *mut c_void,
        ) {
            let ivars = self.ivars();

            let result = if success {
                Ok(ivars.path.clone())
            } else {
                Err("macOS refused to produce the PDF.".to_string())
            };

            eprintln!("slideflare: macOS print operation finished, success={success}");
            emit_result(&ivars.window, result);
        }
    }

    unsafe impl NSObjectProtocol for PrintDelegate {}
);

pub fn print_to_pdf(window: &WebviewWindow, path: PathBuf) -> Result<(), String> {
    let reported_path = path.to_string_lossy().into_owned();
    let window = window.clone();

    // A second handle: the closure below takes ownership of `window`.
    let handle = window.clone();

    handle
        .with_webview(move |platform| {
            // Unlike the other two backends this reports nothing here on
            // success: the print is asynchronous now, and only the delegate
            // knows whether it worked. A failure to *start* is still immediate.
            if let Err(message) = unsafe { start(platform.inner(), window.clone(), &reported_path) }
            {
                emit_result(&window, Err(message));
            }
        })
        .map_err(|e| format!("Could not reach the webview to print: {}", e))
}

unsafe fn start(
    webview: *mut std::ffi::c_void,
    window: WebviewWindow,
    path: &str,
) -> Result<(), String> {
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

    // The window the sheet would attach to. With both panels suppressed nothing
    // is actually presented, but AppKit still requires one.
    let doc_window = webview
        .window()
        .ok_or_else(|| "The webview is not in a window, so it cannot be printed.".to_string())?;

    let delegate = PrintDelegate::alloc().set_ivars(PrintDelegateIvars {
        window,
        path: path.to_string(),
    });
    let delegate: Retained<PrintDelegate> = msg_send![super(delegate), init];

    // Bound explicitly rather than coerced inline, so the delegate argument
    // cannot silently resolve to the wrong thing.
    let delegate_ref: &AnyObject = &delegate;

    eprintln!("slideflare: macOS print operation starting");
    operation.runOperationModalForWindow_delegate_didRunSelector_contextInfo(
        &doc_window,
        Some(delegate_ref),
        Some(sel!(printOperationDidRun:success:contextInfo:)),
        std::ptr::null_mut(),
    );

    // AppKit does not retain the delegate, and the callback lands long after
    // this function returns, so the delegate has to outlive this scope. One
    // small leak per export is the cheapest correct answer: a PDF export is
    // rare and deliberate, and in CLI mode the process exits immediately after.
    std::mem::forget(delegate);

    Ok(())
}

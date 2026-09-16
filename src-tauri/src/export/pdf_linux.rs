//! PDF export on GTK/WebKitGTK.
//!
//! WebKitGTK exposes no direct "render to PDF" call. Instead the GTK print
//! backend is told to print to a file: setting `output-uri` on the print
//! settings is the documented way to reach the "Print to file" printer without
//! enumerating printers or showing a dialog, and `output-file-format` picks PDF
//! over PostScript. `print()` (as opposed to `run_dialog()`) then runs the job
//! straight through with no UI.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use tauri::WebviewWindow;
use webkit2gtk::{PrintOperation, PrintOperationExt};

use super::{emit_result, PAGE_HEIGHT_PT, PAGE_WIDTH_PT};

/// GtkPrintSettings keys. These are plain dictionary entries rather than GObject
/// properties, so they are set through the generic key/value setter. Mirrors
/// `GTK_PRINT_SETTINGS_OUTPUT_URI` / `GTK_PRINT_SETTINGS_OUTPUT_FILE_FORMAT`.
const KEY_OUTPUT_URI: &str = "output-uri";
const KEY_OUTPUT_FILE_FORMAT: &str = "output-file-format";

thread_local! {
    /// Parks the in-flight print operation so it outlives the call that started
    /// it — `print()` is asynchronous and the job is abandoned if the operation
    /// is dropped before it finishes.
    ///
    /// A thread-local rather than Tauri managed state because GTK objects are
    /// neither `Send` nor `Sync`. Everything that touches this — the
    /// `with_webview` closure and both signal handlers — runs on the main
    /// thread, so the operation never leaves the thread that created it.
    static IN_FLIGHT: RefCell<Option<PrintOperation>> = const { RefCell::new(None) };
}

pub fn print_to_pdf(window: &WebviewWindow, path: PathBuf) -> Result<(), String> {
    let uri = file_uri(&path);
    let reported_path = path.to_string_lossy().into_owned();
    let window = window.clone();
    // A second handle: the closure below takes ownership of `window`.
    let handle = window.clone();

    handle
        .with_webview(move |platform| {
            let operation = PrintOperation::new(&platform.inner());

            let settings = gtk::PrintSettings::new();
            settings.set_printer(&print_to_file_printer());
            settings.set(KEY_OUTPUT_URI, Some(uri.as_str()));
            settings.set(KEY_OUTPUT_FILE_FORMAT, Some("pdf"));
            operation.set_print_settings(&settings);

            let paper = gtk::PaperSize::new_custom(
                "slideflare-slide",
                "SlideFlare slide",
                PAGE_WIDTH_PT,
                PAGE_HEIGHT_PT,
                gtk::Unit::Points,
            );
            let page_setup = gtk::PageSetup::new();
            // Deliberately not `set_paper_size_and_default_margins`: that would
            // reapply the paper's default margins and inset every slide.
            page_setup.set_paper_size(&paper);
            page_setup.set_top_margin(0.0, gtk::Unit::Points);
            page_setup.set_bottom_margin(0.0, gtk::Unit::Points);
            page_setup.set_left_margin(0.0, gtk::Unit::Points);
            page_setup.set_right_margin(0.0, gtk::Unit::Points);
            operation.set_page_setup(&page_setup);

            // WebKitGTK emits `failed` *and then* `finished` when a job goes
            // wrong, so the error would otherwise be immediately overwritten by
            // a success. First one to fire wins.
            let reported = Rc::new(Cell::new(false));

            let failed_window = window.clone();
            let failed_flag = Rc::clone(&reported);
            operation.connect_failed(move |_, error| {
                if !failed_flag.replace(true) {
                    emit_result(&failed_window, Err(error.to_string()));
                }
            });

            operation.connect_finished(move |_| {
                if !reported.replace(true) {
                    emit_result(&window, Ok(reported_path.clone()));
                }
                release();
            });

            IN_FLIGHT.with(|slot| *slot.borrow_mut() = Some(operation.clone()));
            operation.print();
        })
        .map_err(|e| format!("Could not reach the webview to print: {}", e))
}

/// Drop our reference to the finished operation, but not from inside its own
/// signal handler — do it on the next main loop turn instead.
fn release() {
    glib::idle_add_local_once(|| {
        IN_FLIGHT.with(|slot| *slot.borrow_mut() = None);
    });
}

/// GTK accepts only `file://` URIs here, and the path must be absolute.
fn file_uri(path: &Path) -> String {
    format!("file://{}", path.display())
}

/// Name of GTK's virtual "Print to File" printer.
///
/// Setting `output-uri` alone is not enough: that key is only honoured by the
/// file print backend, and without naming its printer GTK falls through to the
/// default backend and tries to spawn `lpr`, which fails outright on machines
/// with no CUPS.
///
/// GTK registers that printer under a *translated* name (`_("Print to File")`
/// from the `gtk30` domain), so the name is looked up through the same catalogue
/// rather than hardcoded in English. When the catalogue is missing, `dgettext`
/// hands back the untranslated string, which is exactly the fallback wanted.
fn print_to_file_printer() -> String {
    glib::dgettext(Some("gtk30"), "Print to File").to_string()
}

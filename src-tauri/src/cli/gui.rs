//! Booting the Tauri app, for both presentation and CLI export.
//!
//! Both modes build the *same* app — same plugins, same commands, same
//! frontend. Export mode differs in exactly two ways, and both are deliberate:
//!
//! 1. The deck is rendered out of the user's face, by a route that depends on
//!    the platform's webview. See `configure_export_window`, and
//!    `docs/headless-export.md` for the measurements behind the split.
//!
//!    On GTK there is **no window at all**: `render_offscreen` moves the webview
//!    into a `GtkOffscreenWindow` and hides the toplevel, so nothing is ever
//!    mapped on the compositor. This is verified on Linux only; a display server
//!    is still required, since `gtk_init` fails without one.
//!
//!    Windows still parks a realized, undecorated window far offscreen.
//!    Positioning there is best-effort. macOS cannot do even that: AppKit treats
//!    a fully offscreen window as occluded and WebKit then suspends rendering,
//!    so the print waits forever for pages that never come — the window stays
//!    where `tauri.conf.json` centres it, visible for the render.
//!
//!    What rules out simply hiding the window is **not** the print. WebKitGTK
//!    prints correctly from a webview that was never even realized. It is the
//!    deck measuring itself beforehand: the engine treats a hidden container's
//!    content as hidden, replaced elements never settle their layout, and every
//!    slide then prints over-sized and clipped at exit code 0.
//! 2. `AppState` carries a [`CliExportRequest`], which the frontend picks up and
//!    acts on once the deck reports itself laid out.
//!
//! Everything else — the parse, the watcher, the exporters — is shared, which is
//! the point: CI that runs `slideflare export` is testing what users run.

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{Manager, RunEvent};
// Sizing the window is for platforms that still have one; on GTK the offscreen
// container is sized instead.
#[cfg(not(gtk_platform))]
use tauri::LogicalSize;
// Only the offscreen move needs this, and that is Windows-only now.
#[cfg(windows)]
use tauri::LogicalPosition;

use super::{exit, fail, resolve_deck, resolve_output, ExportArgs};
use crate::export::CliExportRequest;
use crate::watcher::AppState;

/// Where the export window is parked.
///
/// Far enough off any plausible desktop to be invisible, while still being a
/// real mapped window that the compositor will realize and lay out. Windows
/// only: GTK renders into an offscreen window and needs no toplevel, and macOS
/// cannot move the window at all — see `configure_export_window`.
#[cfg(windows)]
const OFFSCREEN: f64 = -10_000.0;

/// Window size for export mode, in CSS pixels.
///
/// Matches `DESIGN_W`/`DESIGN_H` in `src/routes/view-slides/shared.svelte.ts`.
/// Logical rather than physical units, so a HiDPI runner does not end up with a
/// half-size viewport.
const EXPORT_WIDTH: f64 = 1280.0;
const EXPORT_HEIGHT: f64 = 720.0;

/// Set once the export outcome has been reported, so the watchdog stays quiet.
static EXPORT_SETTLED: AtomicBool = AtomicBool::new(false);

#[cfg(gtk_platform)]
thread_local! {
    /// Holds the offscreen window the export webview is rendered into.
    ///
    /// It owns the webview once [`render_offscreen`] has reparented it, so
    /// dropping it would destroy the very thing being printed. A thread-local
    /// rather than Tauri managed state because GTK objects are neither `Send`
    /// nor `Sync`, and everything that touches it runs on the main thread —
    /// the same reasoning as `IN_FLIGHT` in `export/pdf_linux.rs`.
    static EXPORT_CONTAINER: std::cell::RefCell<Option<gtk::OffscreenWindow>> =
        const { std::cell::RefCell::new(None) };
}

/// Build the app with every command registered.
///
/// Shared by both modes so a command can never be available in the GUI but
/// missing from a CLI export, which would fail only at runtime and only on the
/// path CI depends on.
fn builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            crate::watcher::start_file_watcher,
            crate::watcher::reparse_document,
            crate::watcher::initial_file_path,
            crate::updater::check_updates,
            crate::updater::install_skill,
            crate::export::write_export,
            crate::export::current_file_path,
            crate::export::export_pdf,
            crate::export::cli_export_request,
            crate::export::cli_export_finish,
        ])
}

/// Run the presentation window, optionally opening a deck straight away.
///
/// `None` is the untouched original behaviour: the drag-and-drop screen.
pub fn run(deck: Option<PathBuf>) -> i32 {
    run_app(AppState::new(deck.map(path_to_string), None), false)
}

/// Start the app, in whichever mode.
///
/// Both modes go through here for one concrete reason: `tauri::generate_context!`
/// must be expanded **exactly once per crate**. On macOS every expansion emits an
/// `_EMBED_INFO_PLIST` symbol, so a second one anywhere fails the link with
/// "symbol `_EMBED_INFO_PLIST` is already defined" — and only on macOS, so
/// neither a Linux nor a Windows build will warn you about it first.
fn run_app(state: AppState, export_mode: bool) -> i32 {
    let builder = builder().manage(state).setup(move |app| {
        if export_mode {
            configure_export_window(app)?;
        }
        Ok(())
    });

    // The one and only expansion. Read the note above before adding another.
    let context = tauri::generate_context!();

    if !export_mode {
        builder
            .run(context)
            .expect("error while running tauri application");
        return exit::OK;
    }

    let app = builder
        .build(context)
        .expect("error while building tauri application");

    // Tauri exits the process with 0 when the last window closes. Left alone
    // that would turn "the user closed the render window" — or a webview that
    // died on startup — into a silent success, which is precisely the blank-PDF
    // failure mode `docs/testing-platform-exports.md` warns is the dangerous
    // one. Nothing but `finish_export` is allowed to report success.
    app.run(|_app, event| {
        if matches!(event, RunEvent::Exit) && !EXPORT_SETTLED.load(Ordering::SeqCst) {
            eprintln!("slideflare: the render window closed before the export finished");
            let _ = std::io::stderr().flush();
            std::process::exit(exit::FAILURE);
        }
    });

    // Unreachable in practice: the callback above exits first.
    fail("export ended without reporting a result")
}

/// Put the deck where a batch job belongs: out of sight.
///
/// On GTK that means no window at all; elsewhere, a realized window kept off the
/// user's desktop. See the module comment.
fn configure_export_window(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // No dock icon or app switcher entry for what is a batch job. Must happen
    // before the window is shown or macOS activates the app.
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let window = app
        .get_webview_window("main")
        .expect("the main window is declared in tauri.conf.json");

    // On GTK there need be no window at all.
    #[cfg(gtk_platform)]
    return render_offscreen(&window);

    // Everywhere else, keep one realized but out of sight.
    #[cfg(not(gtk_platform))]
    {
        window.set_decorations(false)?;
        window.set_size(LogicalSize::new(EXPORT_WIDTH, EXPORT_HEIGHT))?;

        // On Windows, park it far off any plausible desktop.
        //
        // macOS is excluded deliberately. AppKit reports a fully offscreen
        // window as occluded, and WebKit suspends rendering in the web content
        // process for an occluded WKWebView. The print pipeline then waits
        // forever for pages that are never drawn, which is exactly how this
        // presented: readiness reached, then the print hanging until the
        // watchdog fired. It is the same trap as the animation frames an
        // unpainted window never delivers — see `nextFrames` in
        // `src/routes/view-slides/+page.svelte`.
        //
        // So on macOS the window stays where `tauri.conf.json` centres it,
        // visible for the few seconds a render takes. The activation policy
        // above already keeps it out of the Dock and the app switcher.
        #[cfg(windows)]
        {
            window.set_position(LogicalPosition::new(OFFSCREEN, OFFSCREEN))?;
            window.set_skip_taskbar(true)?;
        }

        Ok(())
    }
}

/// Move the export webview out of its toplevel and into an offscreen window.
///
/// WebKitGTK does not need a window to print, and does not even need a realized
/// widget: an unparented `WebKitWebView` prints correct, paginated, selectable
/// PDF. What it does need is a container the *engine* considers visible, which
/// is a different thing. `visibilityState` follows the container, and a webview
/// the engine calls hidden never settles the layout of its replaced elements —
/// images in particular. The deck measures its own slides to derive one
/// `fitScale` for the whole document, so it would measure short and then print
/// every slide over-sized and clipped, at exit code 0. That was observed, not
/// theorised; `docs/headless-export.md` has the evidence.
///
/// A `GtkOffscreenWindow` satisfies both halves: the web content sees a normal,
/// visible 1280x720 page and keeps receiving animation frames, while nothing is
/// ever mapped on the compositor. That makes it strictly better than the
/// toplevel it replaces, which had to be parked at (-10000, -10000) — a move
/// Wayland compositors ignore, leaving the window plainly visible for the whole
/// render.
#[cfg(gtk_platform)]
fn render_offscreen(window: &tauri::WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    use gtk::prelude::*;

    // The toplevel tao insists on creating stays, unmapped and now empty:
    // Tauri's window bookkeeping and the `RunEvent::Exit` guard in `run_app`
    // both key off it still existing.
    window.hide()?;

    window.with_webview(|platform| {
        let view = platform.inner();

        if let Some(parent) = view.parent() {
            if let Some(container) = parent.downcast_ref::<gtk::Container>() {
                container.remove(&view);
            }
        }

        let offscreen = gtk::OffscreenWindow::new();
        offscreen.set_default_size(EXPORT_WIDTH as i32, EXPORT_HEIGHT as i32);
        offscreen.add(&view);
        // Realizes and allocates it, which is what gives the page its viewport.
        // Without it `innerWidth`/`innerHeight` are 0 and the deck lays out
        // against nothing.
        offscreen.show_all();

        EXPORT_CONTAINER.with(|slot| *slot.borrow_mut() = Some(offscreen));
    })?;

    Ok(())
}

/// Render a deck to PDF or HTML and exit with the result.
///
/// Returns only if the app could not be started at all; otherwise the process
/// ends in [`finish_export`] or in the watchdog.
pub fn run_export(args: ExportArgs, quiet: bool) -> i32 {
    let deck = match resolve_deck(&args.deck, true) {
        Ok(deck) => deck,
        Err(message) => return fail(&message),
    };
    let output = match resolve_output(&args.output) {
        Ok(output) => output,
        Err(message) => return fail(&message),
    };

    // Without `custom-protocol` the webview loads `devUrl` — the Vite dev server
    // — instead of the bundled frontend, so unless `bun run dev` happens to be
    // running the export can only fail, and it fails as a bare "Connection
    // refused" in an empty window. Note this is NOT about `debug_assertions`:
    // `tauri-macros` selects dev mode from this feature alone, so even a
    // `--release` build without it points at localhost.
    if !cfg!(feature = "custom-protocol") {
        eprintln!(
            "slideflare: warning: built without the `custom-protocol` feature, so the \
             frontend is loaded from the dev server rather than the bundle. Rebuild \
             with `--features custom-protocol` (or run `bun run dev` alongside this)."
        );
    }

    if !quiet {
        eprintln!(
            "slideflare: rendering {} to {}",
            deck.display(),
            output.display()
        );
    }

    let request = CliExportRequest {
        kind: args.format.as_str().to_string(),
        out_path: path_to_string(output),
    };

    start_watchdog(args.timeout);

    run_app(
        AppState::new(Some(path_to_string(deck)), Some(request)),
        true,
    )
}

/// Print the outcome of a CLI export and end the process with the right code.
///
/// Exits hard rather than unwinding: by the time this is called the file has
/// already been written, and a one-shot batch render has nothing worth tearing
/// down. Buffers are flushed explicitly first, because `std::process::exit` does
/// not flush Rust's own — which would lose the output path whenever stdout is a
/// pipe rather than a terminal, i.e. exactly when a CI script is reading it.
pub fn finish_export(ok: bool, message: Option<&str>) -> ! {
    EXPORT_SETTLED.store(true, Ordering::SeqCst);

    let code = if ok {
        // The path written, so `out=$(slideflare export ...)` is useful.
        if let Some(message) = message {
            println!("{message}");
        }
        exit::OK
    } else {
        eprintln!("slideflare: {}", message.unwrap_or("export failed"));
        exit::FAILURE
    };

    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    std::process::exit(code);
}

/// Give up if the export never reports back.
///
/// Without this a print backend that silently drops its completion signal would
/// leave an invisible window running forever — in CI, until the job timed out
/// with no useful diagnosis.
fn start_watchdog(seconds: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(seconds));

        if EXPORT_SETTLED.load(Ordering::SeqCst) {
            return;
        }

        eprintln!("slideflare: export timed out after {seconds}s");
        let _ = std::io::stderr().flush();
        std::process::exit(exit::TIMEOUT);
    });
}

/// Paths reach the frontend as strings; they are already absolute by here.
fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().to_string()
}

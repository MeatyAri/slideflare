//! Booting the Tauri app, for both presentation and CLI export.
//!
//! Both modes build the *same* app — same plugins, same commands, same
//! frontend. Export mode differs in exactly two ways, and both are deliberate:
//!
//! 1. The window is moved offscreen and stripped of decorations rather than
//!    being hidden. `docs/testing-platform-exports.md` is explicit that hiding
//!    it is the risky choice: PDF export prints the *live, realized* webview
//!    because layout, font resolution, and MathML measurement are settled there,
//!    and on GTK an unmapped window may never realize at all. Offscreen keeps it
//!    realized while keeping it out of the user's face.
//!
//!    Positioning is best-effort, and skipped entirely on macOS. Wayland has no
//!    notion of global window coordinates, so compositors there ignore the move
//!    and the window is simply visible for the duration of the render — cosmetic
//!    rather than harmful, and irrelevant to CI, which runs under `xvfb`, an X
//!    server, where the move works. macOS is worse than cosmetic: AppKit treats
//!    a fully offscreen window as occluded and WebKit then suspends rendering,
//!    so the print waits forever for pages that never come. See
//!    `configure_export_window`.
//! 2. `AppState` carries a [`CliExportRequest`], which the frontend picks up and
//!    acts on once the deck reports itself laid out.
//!
//! Everything else — the parse, the watcher, the exporters — is shared, which is
//! the point: CI that runs `slideflare export` is testing what users run.

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{LogicalSize, Manager, RunEvent};
// Only the offscreen move needs this, and that is skipped on macOS.
#[cfg(not(target_os = "macos"))]
use tauri::LogicalPosition;

use super::{exit, fail, resolve_deck, resolve_output, ExportArgs};
use crate::export::CliExportRequest;
use crate::watcher::AppState;

/// Where the export window is parked.
///
/// Far enough off any plausible desktop to be invisible, while still being a
/// real mapped window that the compositor and GTK will realize and lay out.
/// Not used on macOS — see `configure_export_window`.
#[cfg(not(target_os = "macos"))]
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

/// Put the render window where a batch job belongs: realized, but out of sight.
fn configure_export_window(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // No dock icon or app switcher entry for what is a batch job. Must happen
    // before the window is shown or macOS activates the app.
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let window = app
        .get_webview_window("main")
        .expect("the main window is declared in tauri.conf.json");

    // Realized but out of sight — see the module comment for why this is not
    // `set_visible(false)`.
    window.set_decorations(false)?;
    window.set_size(LogicalSize::new(EXPORT_WIDTH, EXPORT_HEIGHT))?;

    // Everywhere but macOS, park it far off any plausible desktop.
    //
    // macOS is excluded deliberately. AppKit reports a fully offscreen window as
    // occluded, and WebKit suspends rendering in the web content process for an
    // occluded WKWebView. The print pipeline then waits forever for pages that
    // are never drawn, which is exactly how this presented: readiness reached,
    // then `runOperation` hanging until the watchdog fired. It is the same trap
    // as the animation frames an unpainted window never delivers — see
    // `nextFrames` in `src/routes/view-slides/+page.svelte`.
    //
    // So on macOS the window stays where `tauri.conf.json` centres it, visible
    // for the few seconds a render takes. The activation policy above already
    // keeps it out of the Dock and the app switcher.
    #[cfg(not(target_os = "macos"))]
    {
        window.set_position(LogicalPosition::new(OFFSCREEN, OFFSCREEN))?;
        window.set_skip_taskbar(true)?;
    }

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

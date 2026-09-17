pub mod cli;
pub mod export;
pub mod incremental;
pub mod parser;
pub mod updater;
mod watcher;

/// Mobile entry point.
///
/// Mobile has no command line to read, so it goes straight to the presentation
/// window with no deck preloaded. Desktop launches come through
/// [`cli::dispatch`] instead, which decides between this, an export, and the
/// headless subcommands.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    cli::gui::run(None);
}

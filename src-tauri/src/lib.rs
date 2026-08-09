pub mod incremental;
pub mod parser;
pub mod updater;
mod watcher;

use crate::updater::{check_updates, install_skill};
use crate::watcher::{reparse_document, start_file_watcher, AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            start_file_watcher,
            reparse_document,
            check_updates,
            install_skill
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

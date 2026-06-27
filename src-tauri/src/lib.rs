pub mod incremental;
pub mod parser;
mod watcher;

use crate::watcher::{start_file_watcher, reparse_document, AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![start_file_watcher, reparse_document])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

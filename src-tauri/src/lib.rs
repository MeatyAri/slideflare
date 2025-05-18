use std::path::PathBuf;

use negahban::{Negahban, HookType, EventType};

#[tauri::command]
fn start_file_watcher(file_path: String) {
    tokio::spawn(async move {
        let _ = Negahban{
            path: PathBuf::from(file_path),
            hook: HookType::IndefiniteHook(
                Box::new(|event| {
                    if event.kind == EventType::Modify {
                        println!("File modified: {:?}", event.paths);
                    }
                })
            ),
            ..Negahban::default() // sets rest of them to default
        }.watch();
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_file_watcher])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

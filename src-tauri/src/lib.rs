use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::fs;
use tauri::Emitter;

#[tauri::command]
fn start_file_watcher(window: tauri::Window, file_path: String) {
    // Spawn a new thread to handle file change events
    std::thread::spawn(move || {
        let path = PathBuf::from(file_path);
        let (tx, rx) = channel();

        // Initialize the file watcher
        let mut watcher: RecommendedWatcher = Watcher::new(tx, notify::Config::default())
            .expect("Failed to create watcher");

        // Start watching the file
        watcher
            .watch(&path, RecursiveMode::NonRecursive)
            .expect("Failed to watch file");
        
        for res in rx {
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(notify::event::ModifyKind::Data(_))) {
                        // Read the updated file content
                        if let Ok(content) = fs::read_to_string(&path) {
                            // Emit the content to the frontend
                            window
                                .emit("markdown-updated", content)
                                .expect("Failed to emit event");
                        }
                    }
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        }
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

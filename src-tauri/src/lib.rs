mod watcher;
mod parser;

use std::path::PathBuf;
use std::ops::ControlFlow;
use std::sync::{Arc, Mutex};

use tauri::{Emitter, Listener};
use negahban::{Negahban, HookType, EventType};

use watcher::send_new_file;

#[tauri::command]
async fn start_file_watcher(window: tauri::Window, file_path: String) {
    //check if the file exists
    let file_path_buf = PathBuf::from(&file_path);
    if !file_path_buf.exists() {
        window
            .emit("file-not-found", "File not found")
            .expect("Failed to emit event");
        return;
    }

    // set the window title
    let file_name = file_path_buf.file_name().and_then(|name| name.to_str()).unwrap();
    let title = file_name.trim_end_matches(".md"); // Remove ".md" extension if present
    window.set_title(&format!("SlideFlare: {}", title)).expect("Failed to set window title");

    // create a shared variable to store the last hash
    let last_hash = Arc::new(Mutex::new(0 as u64));
    let last_hash_clone = Arc::clone(&last_hash);

    // Send the initial content of the file
    let _ = send_new_file(&window, &file_path, &last_hash_clone);
    
    let terminate = Arc::new(Mutex::new(false));
    let terminate_clone = Arc::clone(&terminate);

    // Terminate watcher logic
    window.listen("terminate-event", move |_event| {
        let mut terminate_lock = terminate_clone.lock().unwrap();
        *terminate_lock = true;
    });

    let terminate_clone = Arc::clone(&terminate);
    
    let _ = Negahban{
        path: file_path_buf,
        hook: HookType::ControledHook(
            Box::new(move |event| {
                let terminate_lock = terminate_clone.lock().unwrap();
                if *terminate_lock {
                    return ControlFlow::Break(());
                }

                if event.kind == EventType::Modify {
                    let _ = send_new_file(&window, &file_path, &last_hash_clone);
                }
                return ControlFlow::Continue(());
            })
        ),
        ..Negahban::default() // sets rest of them to default
    }.watch();
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

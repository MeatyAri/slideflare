use std::path::PathBuf;
use std::fs;
use tauri::{Emitter, Listener};
use std::ops::ControlFlow;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use negahban::{Negahban, HookType, EventType};

fn send_new_file(window: &tauri::Window, file_path: &str) {
    if let Ok(content) = fs::read_to_string(file_path) {
        // Emit the content to the frontend
        window
            .emit("markdown-updated", content)
            .expect("Failed to emit event");
    }
}

#[tauri::command]
fn start_file_watcher(window: tauri::Window, file_path: String) {
    tokio::spawn(async move {
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

        // Send the initial content of the file
        send_new_file(&window, &file_path);

        let _ = Negahban{
            path: file_path_buf,
            hook: HookType::ControledHook(
                Box::new(|event| {
                    let should_terminate = Arc::new(AtomicBool::new(false));
                    let should_terminate_clone = should_terminate.clone();
                    
                    window.listen("terminate-event", move |_event| {
                        should_terminate_clone.store(true, Ordering::SeqCst);
                    });
                    if should_terminate.load(Ordering::SeqCst) {
                        return ControlFlow::Break(());
                    }

                    if event.kind == EventType::Modify {
                        send_new_file(&window, &file_path);
                    }
                    return ControlFlow::Continue(());
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

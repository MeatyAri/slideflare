use std::fs;

use tauri::Emitter;

pub fn send_new_file(window: &tauri::Window, file_path: &str) {
    if let Ok(content) = fs::read_to_string(file_path) {
        // Emit the content to the frontend
        window
            .emit("markdown-updated", content)
            .expect("Failed to emit event");
    }
}

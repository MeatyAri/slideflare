use std::fs;

use tauri::Emitter;
use twox_hash::XxHash64;

pub fn send_new_file(window: &tauri::Window, file_path: &str, last_hash: &std::sync::Arc<std::sync::Mutex<u64>>) {
    if let Ok(content) = fs::read_to_string(file_path) {
        // Check if the content has changed
        let hash = XxHash64::oneshot(42, content.as_bytes());
        let mut last_hash_lock = last_hash.lock().unwrap();
        if *last_hash_lock == hash {
            return;
        }
        *last_hash_lock = hash;
        println!("File modified: {:?}", file_path);
        
        // Emit the content to the frontend
        window
            .emit("markdown-updated", content)
            .expect("Failed to emit event");
    }
}

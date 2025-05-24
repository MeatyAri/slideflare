use std::{error::Error, fs};

use tauri::Emitter;
use twox_hash::XxHash64;

use crate::parser::parse_markdown_with_frontmatter;

pub fn send_new_file(window: &tauri::Window, file_path: &str, last_hash: &std::sync::Arc<std::sync::Mutex<u64>>) -> Result<(), Box<dyn Error>> {
    if let Ok(content) = fs::read_to_string(file_path) {
        // Check if the content has changed
        let hash = XxHash64::oneshot(42, content.as_bytes());
        let mut last_hash_lock = last_hash.lock().unwrap();
        if *last_hash_lock == hash {
            return Ok(());
        }
        *last_hash_lock = hash;

        // Parse the markdown file with frontmatter
        let res = parse_markdown_with_frontmatter(&content)?;
        
        // Convert to prettified JSON string
        let json_string = serde_json::to_string_pretty(&res)?;

        // Emit the content to the frontend
        window
            .emit("markdown-updated", json_string)
            .expect("Failed to emit event");

        return Ok(());
    }

    Err("Failed to read file".into())
}

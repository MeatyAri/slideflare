use std::{error::Error, fs, path::PathBuf};

use std::ops::ControlFlow;
use std::sync::{Arc, Mutex};
use twox_hash::XxHash64;

use negahban::{EventType, HookType, Negahban};
use tauri::{Emitter, Listener};

use crate::parser::parse_markdown_with_frontmatter;

fn send_new_file(
    window: &tauri::Window,
    file_path: &str,
    last_hash: &std::sync::Arc<std::sync::Mutex<u64>>,
) -> Result<(), Box<dyn Error>> {
    if let Ok(content) = fs::read_to_string(file_path) {
        // Check if the content has changed
        let hash = XxHash64::oneshot(42, content.as_bytes());
        let mut last_hash_lock = last_hash.lock().unwrap();
        if *last_hash_lock == hash {
            return Ok(());
        }
        *last_hash_lock = hash;

        // Get the directory of the markdown file for relative path resolution
        let base_dir = PathBuf::from(file_path)
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        // Parse the markdown file with frontmatter
        let res = parse_markdown_with_frontmatter(&content, &base_dir)?;

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

#[tauri::command]
pub async fn start_file_watcher(window: tauri::Window, file_path: String) {
    //check if the file exists
    let file_path_buf = PathBuf::from(&file_path);
    if !file_path_buf.exists() {
        window
            .emit("file-not-found", "File not found")
            .expect("Failed to emit event");
        return;
    }

    // set the window title
    let file_name = file_path_buf
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap();
    let title = file_name.trim_end_matches(".md"); // Remove ".md" extension if present
    window
        .set_title(&format!("SlideFlare: {}", title))
        .expect("Failed to set window title");

    // create a shared variable to store the last hash
    let last_hash = Arc::new(Mutex::new(0_u64));
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

    let _ = Negahban {
        path: file_path_buf,
        hook: HookType::ControledHook(Box::new(move |event| {
            let terminate_lock = terminate_clone.lock().unwrap();
            if *terminate_lock {
                return ControlFlow::Break(());
            }

            if event.kind == EventType::Modify {
                let _ = send_new_file(&window, &file_path, &last_hash_clone);
            }

            ControlFlow::Continue(())
        })),
        ..Negahban::default() // sets rest of them to default
    }
    .watch();
}

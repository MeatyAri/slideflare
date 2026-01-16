use std::{error::Error, fs, path::PathBuf};

use std::ops::ControlFlow;
use std::sync::{Arc, Mutex};
use twox_hash::XxHash64;

use negahban::{EventType, HookType, Negahban};
use tauri::{Emitter, Listener};

use crate::incremental::{
    compute_slide_hashes, create_slide_change_events, detect_slide_changes, SlideChangeEvent,
    VecSlideHashes,
};
use crate::parser::parse_markdown_with_frontmatter;

/// State for incremental slide processing
#[derive(Debug)]
struct IncrementalState {
    last_file_hash: u64,
    last_slide_hashes: VecSlideHashes,
}

impl IncrementalState {
    fn new() -> Self {
        Self {
            last_file_hash: 0,
            last_slide_hashes: VecSlideHashes::new(),
        }
    }
}

fn send_new_file(
    window: &tauri::Window,
    file_path: &str,
    incremental_state: &Arc<Mutex<IncrementalState>>,
) -> Result<(), Box<dyn Error>> {
    if let Ok(content) = fs::read_to_string(file_path) {
        // Check if the content has changed
        let file_hash = XxHash64::oneshot(42, content.as_bytes());

        let mut state_lock = incremental_state.lock().unwrap();

        // If file hasn't changed, no action needed
        if state_lock.last_file_hash == file_hash {
            return Ok(());
        }

        // Get the directory of the markdown file for relative path resolution
        let base_dir = PathBuf::from(file_path)
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        // For first load, use legacy full processing
        if state_lock.last_slide_hashes.data.is_empty() {
            let res = parse_markdown_with_frontmatter(&content, &base_dir)?;
            let json_string = serde_json::to_string(&res)?;

            window
                .emit("markdown-updated", json_string)
                .expect("Failed to emit event");
        } else {
            // Use incremental processing
            let new_hashes = compute_slide_hashes(&content)?;
            let slide_changes = detect_slide_changes(&state_lock.last_slide_hashes, &new_hashes);

            if slide_changes.hunks().next().is_none() {
                return Ok(());
            }

            let change_events = create_slide_change_events(
                &state_lock.last_slide_hashes,
                &content,
                slide_changes,
                &base_dir,
            )?;

            let slide_change_event = SlideChangeEvent {
                changes: change_events,
            };

            let json_string = serde_json::to_string(&slide_change_event)?;

            window
                .emit("slide-changed", json_string)
                .expect("Failed to emit slide change event");
        }

        // Update state
        let new_metadata = compute_slide_hashes(&content)?;
        state_lock.last_file_hash = file_hash;
        state_lock.last_slide_hashes = new_metadata;

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

    // create a shared state for incremental processing
    let incremental_state = Arc::new(Mutex::new(IncrementalState::new()));

    // Send the initial content of the file
    let _ = send_new_file(&window, &file_path, &incremental_state);

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
                let _ = send_new_file(&window, &file_path, &incremental_state);
            }

            ControlFlow::Continue(())
        })),
        ..Negahban::default() // sets rest of them to default
    }
    .watch();
}

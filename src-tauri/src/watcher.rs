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

/// State shared between the file watcher and the reparse command.
/// Allows `reparse_document` to access the watcher's incremental state
/// and file path so it can trigger a full reparse.
pub struct AppState {
    file_path: Mutex<Option<String>>,
    incremental_state: Mutex<Option<Arc<Mutex<IncrementalState>>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            file_path: Mutex::new(None),
            incremental_state: Mutex::new(None),
        }
    }
}

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

    /// Reset to initial state so the next `send_new_file` call takes the
    /// full-reparse path instead of the incremental diffing path.
    fn reset(&mut self) {
        self.last_file_hash = 0;
        self.last_slide_hashes = VecSlideHashes::new();
    }
}

fn send_new_file(
    window: &tauri::Window,
    file_path: &str,
    incremental_state: &Arc<Mutex<IncrementalState>>,
) -> Result<(), Box<dyn Error>> {
    if let Ok(content) = fs::read_to_string(file_path) {
        let file_hash = XxHash64::oneshot(42, content.as_bytes());

        let mut state_lock = incremental_state.lock().unwrap();

        if state_lock.last_file_hash == file_hash {
            return Ok(());
        }

        let base_dir = PathBuf::from(file_path)
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        if state_lock.last_slide_hashes.data.is_empty() {
            match parse_markdown_with_frontmatter(&content, &base_dir) {
                Ok(res) => {
                    let json_string = serde_json::to_string(&res)?;
                    window
                        .emit("markdown-updated", json_string)
                        .expect("Failed to emit event");

                    let new_slide_hashes = compute_slide_hashes(&content)?;
                    state_lock.last_file_hash = file_hash;
                    state_lock.last_slide_hashes = new_slide_hashes;
                }
                Err(e) => {
                    let parse_error = crate::parser::ParseError {
                        message: e.to_string(),
                        line: None,
                    };
                    let json_string = serde_json::to_string(&parse_error)?;
                    window
                        .emit("parse-error", json_string)
                        .expect("Failed to emit parse error event");
                    state_lock.last_file_hash = file_hash;
                    state_lock.last_slide_hashes = VecSlideHashes::new();
                }
            }
            return Ok(());
        }

        let new_slide_hashes = match compute_slide_hashes(&content) {
            Ok(hashes) => hashes,
            Err(e) => {
                let parse_error = crate::parser::ParseError {
                    message: e.to_string(),
                    line: None,
                };
                let json_string = serde_json::to_string(&parse_error)?;
                window
                    .emit("parse-error", json_string)
                    .expect("Failed to emit parse error event");
                state_lock.last_file_hash = file_hash;
                state_lock.last_slide_hashes = VecSlideHashes::new();
                return Ok(());
            }
        };

        let slide_changes = detect_slide_changes(&state_lock.last_slide_hashes, &new_slide_hashes);

        if slide_changes.hunks().next().is_none() {
            state_lock.last_file_hash = file_hash;
            return Ok(());
        }

        match create_slide_change_events(
            &state_lock.last_slide_hashes,
            &content,
            slide_changes,
            &base_dir,
        ) {
            Ok(change_events) => {
                let slide_change_event = SlideChangeEvent {
                    changes: change_events,
                };
                let json_string = serde_json::to_string(&slide_change_event)?;
                window
                    .emit("slide-changed", json_string)
                    .expect("Failed to emit slide change event");
            }
            Err(e) => {
                let parse_error = crate::parser::ParseError {
                    message: e.to_string(),
                    line: None,
                };
                let json_string = serde_json::to_string(&parse_error)?;
                window
                    .emit("parse-error", json_string)
                    .expect("Failed to emit parse error event");
                state_lock.last_file_hash = file_hash;
            }
        }

        state_lock.last_file_hash = file_hash;
        state_lock.last_slide_hashes = new_slide_hashes;

        return Ok(());
    }

    Err("Failed to read file".into())
}

#[tauri::command]
pub async fn start_file_watcher(
    window: tauri::Window,
    app_state: tauri::State<'_, AppState>,
    file_path: String,
) -> Result<(), String> {
    //check if the file exists
    let file_path_buf = PathBuf::from(&file_path);
    if !file_path_buf.exists() {
        window
            .emit("file-not-found", "File not found")
            .expect("Failed to emit event");
        return Ok(());
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

    // Store the file path and incremental state in app state so that the
    // `reparse_document` command can trigger a full reparse later.
    {
        let mut path_lock = app_state.file_path.lock().unwrap();
        *path_lock = Some(file_path.clone());
        let mut state_lock = app_state.incremental_state.lock().unwrap();
        *state_lock = Some(Arc::clone(&incremental_state));
    }

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

    Ok(())
}

/// Trigger a full reparse of the document, bypassing the incremental diffing
/// mechanism. This resets the incremental state so that `send_new_file` takes
/// the full-parse path (emitting `markdown-updated` with all slides) instead of
/// computing and emitting incremental `slide-changed` events.
///
/// This is useful when the diffing mechanism produces a corrupted document and
/// the user wants to force a clean reparse — equivalent to closing and reopening
/// the app.
#[tauri::command]
pub async fn reparse_document(
    window: tauri::Window,
    app_state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let file_path = {
        let path_lock = app_state.file_path.lock().unwrap();
        match path_lock.clone() {
            Some(path) => path,
            None => return Ok(()),
        }
    };

    let incremental_state = {
        let state_lock = app_state.incremental_state.lock().unwrap();
        match state_lock.clone() {
            Some(state) => state,
            None => return Ok(()),
        }
    };

    // Reset the incremental state so the next call takes the full-reparse path
    {
        let mut state_lock = incremental_state.lock().unwrap();
        state_lock.reset();
    }

    // Re-send the file — with cleared state this emits `markdown-updated`
    // with the full slide list, exactly like the initial load.
    let _ = send_new_file(&window, &file_path, &incremental_state);

    Ok(())
}

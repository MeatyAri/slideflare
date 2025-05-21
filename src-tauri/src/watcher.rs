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

        // parse the markdown content to HTML
        let options = markdown::Options {
            parse: markdown::ParseOptions {
                constructs: markdown::Constructs {
                    frontmatter: true,
                    math_flow: true,
                    math_text: true,
                    ..markdown::Constructs::gfm()
                },
                ..markdown::ParseOptions::gfm()
            },
            ..markdown::Options::gfm()
        };
        let res = markdown::to_html_with_options(
            &content,
            &options
        );

        if let Err(e) = res {
            eprintln!("Error parsing markdown: {}", e);
            return;
        }
        let content = res.unwrap();

        // Emit the content to the frontend
        window
            .emit("markdown-updated", content)
            .expect("Failed to emit event");
    }
}

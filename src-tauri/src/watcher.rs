use std::fs;

use tauri::Emitter;
use twox_hash::XxHash64;
use markdown::{to_html_with_options, Options};

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
        // let options = Options {
        //     parse: markdown::ParseOptions {
        //         constructs: markdown::Constructs {
        //             gfm: true,         // Enable GitHub Flavored Markdown
        //             math: true,        // Enable math expressions
        //             frontmatter: true, // Enable frontmatter (optional)
        //             table: true,       // Enable tables (part of GFM)
        //             tasklist: true,    // Enable task lists (part of GFM)
        //             strikethrough: true, // Enable strikethrough (part of GFM)
        //             ..markdown::Constructs::default()
        //         },
        //         ..markdown::ParseOptions::default()
        //     },
        //     ..Options::default()
        // };
        let res = to_html_with_options(
            &content,
            &Options::gfm()
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

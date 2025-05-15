use std::fs;

#[tauri::command]
fn process_file(path: String) -> Result<String, String> {
    // Read the file content
    match fs::read_to_string(&path) {
        Ok(content) => {
            // Perform your processing here
            let processed = content.to_uppercase(); // Example processing
            Ok(processed)
        }
        Err(e) => Err(format!("Failed to read file: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![process_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

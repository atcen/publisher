// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::io::Write;
use std::path::Path;
use tauri::Runtime;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
async fn open_file<R: Runtime>(app: tauri::AppHandle<R>) -> Result<String, String> {
    let file_path = app.dialog().file().blocking_pick_file();
    match file_path {
        Some(path) => Ok(path.to_string()),
        None => Err("No file selected".to_string()),
    }
}

#[tauri::command]
async fn read_document<R: Runtime>(
    _app: tauri::AppHandle<R>,
    file_path: String,
) -> Result<String, String> {
    fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
async fn save_document<R: Runtime>(
    _app: tauri::AppHandle<R>,
    file_path: String,
    document_json: String,
) -> Result<String, String> {
    serde_json::from_str::<serde_json::Value>(&document_json)
        .map_err(|e| format!("Invalid document JSON: {}", e))?;

    let path = Path::new(&file_path);
    let parent = path.parent().ok_or("Invalid file path")?;

    fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;

    let temp_path = parent.join(format!(
        ".{}.tmp",
        path.file_stem()
            .ok_or("Invalid filename")?
            .to_string_loshy()
    ));

    let mut temp_file =
        fs::File::create(&temp_path).map_err(|e| format!("Failed to create temp file: {}", e))?;

    temp_file
        .write_all(document_json.as_bytes())
        .map_err(|e| format!("Failed to write to temp file: {}", e))?;

    temp_file
        .sync_all()
        .map_err(|e| format!("Failed to sync temp file: {}", e))?;

    fs::rename(&temp_path, &file_path)
        .map_err(|e| format!("Failed to replace file: {}", e))?;

    Ok(file_path)
}

#[tauri::command]
async fn save_as_file<R: Runtime>(
    app: tauri::AppHandle<R>,
    document_json: String,
) -> Result<String, String> {
    serde_json::from_str::<serde_json::Value>(&document_json)
        .map_err(|e| format!("Invalid document JSON: {}", e))?;

    let file_path = app
        .dialog()
        .file()
        .add_filter("Publisher Documents", &["publisher"])
        .blocking_save_file();

    match file_path {
        Some(path) => {
            let path_str = path.to_string();
            let final_path = if !path_str.ends_with(".publisher") {
                format!("{}.publisher", path_str)
            } else {
                path_str
            };

            let path_obj = Path::new(&final_path);
            let parent = path_obj.parent().ok_or("Invalid file path")?;

            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;

            let temp_path = parent.join(format!(
                ".{}.tmp",
                path_obj
                    .file_stem()
                    .ok_or("Invalid filename")?
                    .to_string_loshy()
            ));

            let mut temp_file = fs::File::create(&temp_path)
                .map_err(|e| format!("Failed to create temp file: {}", e))?;

            temp_file
                .write_all(document_json.as_bytes())
                .map_err(|e| format!("Failed to write to temp file: {}", e))?;

            temp_file
                .sync_all()
                .map_err(|e| format!("Failed to sync temp file: {}", e))?;

            fs::rename(&temp_path, &final_path)
                .map_err(|e| format!("Failed to replace file: {}", e))?;

            Ok(final_path)
        }
        None => Err("Save cancelled".to_string()),
    }
}

fn main() {
    publisher_renderer::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            open_file,
            read_document,
            save_document,
            save_as_file
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, Runtime};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
async fn open_file<R: Runtime>(app: tauri::AppHandle<R>) -> Result<String, String> {
    let file_path = app.dialog().file().blocking_pick_file();
    
    match file_path {
        Some(path) => {
            // In Tauri 2.0 FilePath might be an enum or struct. 
            // Usually we can get the path as a string.
            Ok(path.to_string())
        }
        None => Err("No file selected".to_string()),
    }
}

#[tauri::command]
async fn save_file<R: Runtime>(app: tauri::AppHandle<R>) -> Result<String, String> {
    let file_path = app.dialog().file().blocking_save_file();
    
    match file_path {
        Some(path) => {
            Ok(path.to_string())
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
        .invoke_handler(tauri::generate_handler![open_file, save_file])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod document_service;

use commands::{get_history_state, get_redo_history, get_undo_history, redo, undo, convert_color, apply_grid_preset, find_snap, align_frames, distribute_frames};
use document_service::{DocumentService, DocumentServiceError};
use publisher_core::DocumentState;
use publisher_color::ColorEngine;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{Runtime, Manager};
use tauri_plugin_dialog::DialogExt;

/// Application preferences stored on disk
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppPreferences {
    pub theme: String,
    pub default_unit: String,
    pub autosave_interval: u32,
    #[serde(default)]
    pub recent_files: Vec<String>,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            default_unit: "pt".to_string(),
            autosave_interval: 60,
            recent_files: Vec::new(),
        }
    }
}

/// Application state containing both document service and history/undo-redo state
pub struct AppState {
    pub document_service: Mutex<DocumentService>,
    pub document_state: Mutex<Option<DocumentState>>,
    pub color_engine: Mutex<ColorEngine>,
    pub recovery_dir: PathBuf,
    pub preferences_path: PathBuf,
}

/// Create a new empty document
#[tauri::command]
async fn new_document(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let mut service = state
        .document_service
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    match service.create_new("Untitled") {
        Ok(id) => Ok(json!({
            "success": true,
            "document_id": id.0.to_string(),
            "message": "Document created successfully"
        })
        .to_string()),
        Err(e) => Err(format!("Failed to create document: {}", e)),
    }
}

/// Open a document file via native file picker
#[tauri::command]
async fn open_document<R: Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    // Show native file picker
    let file_path = app
        .dialog()
        .file()
        .add_filter("Publisher", &["publisher"])
        .blocking_pick_file();
    match file_path {
        Some(path) => {
            let mut path_str = path.to_string();
            // Ensure .publisher extension
            if !path_str.ends_with(".publisher") {
                path_str = format!("{}.publisher", path_str);
            }
            let mut service = state
                .document_service
                .lock()
                .map_err(|e| format!("Failed to acquire lock: {}", e))?;

            match service.open_document(&path_str) {
                Ok(id) => {
                    let doc = service
                        .get_document(id)
                        .ok_or("Failed to retrieve document")?;
                    let doc_name = doc.metadata.name;
                    Ok(json!({
                        "success": true,
                        "document_id": id.0.to_string(),
                        "document_name": doc_name,
                        "file_path": path_str,
                        "message": format!("Opened: {}", doc_name)
                    })
                    .to_string())
                }
                Err(e) => {
                    let error_msg = match e {
                        DocumentServiceError::Persistence(msg) => {
                            format!("Malformed file: {}", msg)
                        }
                        DocumentServiceError::IO(msg) => {
                            format!("Cannot read file: {}", msg)
                        }
                        DocumentServiceError::NotFound(msg) => msg,
                        DocumentServiceError::InvalidPath(msg) => msg,
                    };
                    Err(error_msg)
                }
            }
        }
        None => Err("No file selected".to_string()),
    }
}

/// Read document from file
#[tauri::command]
async fn read_document<R: Runtime>(
    _app: tauri::AppHandle<R>,
    file_path: String,
) -> Result<String, String> {
    fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Save the active document
#[tauri::command]
async fn save_document_file<R: Runtime>(
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
            .to_string_lossy()
    ));

    let mut temp_file =
        fs::File::create(&temp_path).map_err(|e| format!("Failed to create temp file: {}", e))?;

    temp_file
        .write_all(document_json.as_bytes())
        .map_err(|e| format!("Failed to write to temp file: {}", e))?;

    temp_file
        .sync_all()
        .map_err(|e| format!("Failed to sync temp file: {}", e))?;

    // On Windows, rename fails if the target exists, so remove it first
    if Path::new(&file_path).exists() {
        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to remove existing file: {}", e))?;
    }

    fs::rename(&temp_path, &file_path).map_err(|e| format!("Failed to replace file: {}", e))?;

    Ok(file_path)
}

/// Save document with a new path (Save As)
#[tauri::command]
async fn save_document_as<R: Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, AppState>,
    document_id: String,
) -> Result<String, String> {
    let doc_id = publisher_core::document_manager::DocumentId(
        uuid::Uuid::parse_str(&document_id).map_err(|_| "Invalid document ID".to_string())?,
    );

    // Show save file dialog
    let file_path = app
        .dialog()
        .file()
        .add_filter("Publisher", &["publisher"])
        .blocking_save_file();

    match file_path {
        Some(path) => {
            let mut path_str = path.to_string();
            // Ensure .publisher extension
            if !path_str.ends_with(".publisher") {
                path_str = format!("{}.publisher", path_str);
            }
            let mut service = state
                .document_service
                .lock()
                .map_err(|e| format!("Failed to acquire lock: {}", e))?;

            match service.save_document_as(doc_id, &path_str) {
                Ok(saved_path) => {
                    // Ensure document is marked clean after successful save
                    Ok(json!({
                        "success": true,
                        "file_path": saved_path,
                        "message": "Document saved successfully"
                    })
                    .to_string())
                }
                Err(e) => Err(format!("Save As failed: {}", e)),
            }
        }
        None => Err("Save cancelled".to_string()),
    }
}

/// Save document via JSON (alternative save method)
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

            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;

            let temp_path = parent.join(format!(
                ".{}.tmp",
                path_obj
                    .file_stem()
                    .ok_or("Invalid filename")?
                    .to_string_lossy()
            ));

            let mut temp_file = fs::File::create(&temp_path)
                .map_err(|e| format!("Failed to create temp file: {}", e))?;

            temp_file
                .write_all(document_json.as_bytes())
                .map_err(|e| format!("Failed to write to temp file: {}", e))?;

            temp_file
                .sync_all()
                .map_err(|e| format!("Failed to sync temp file: {}", e))?;

            // On Windows, rename fails if the target exists, so remove it first
            if Path::new(&final_path).exists() {
                fs::remove_file(&final_path)
                    .map_err(|e| format!("Failed to remove existing file: {}", e))?;
            }

            fs::rename(&temp_path, &final_path)
                .map_err(|e| format!("Failed to replace file: {}", e))?;

            Ok(final_path)
        }
        None => Err("Save cancelled".to_string()),
    }
}

/// Close a document with unsaved changes warning
#[tauri::command]
async fn close_document(
    state: tauri::State<'_, AppState>,
    document_id: String,
) -> Result<String, String> {
    let doc_id = publisher_core::document_manager::DocumentId(
        uuid::Uuid::parse_str(&document_id).map_err(|_| "Invalid document ID".to_string())?,
    );

    let mut service = state
        .document_service
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    let has_unsaved = service.has_unsaved_changes(doc_id);

    match service.close_document(doc_id) {
        Ok(_) => Ok(json!({
            "success": true,
            "had_unsaved_changes": has_unsaved,
            "message": "Document closed"
        })
        .to_string()),
        Err(e) => Err(format!("Close failed: {}", e)),
    }
}

/// Check if document has unsaved changes
#[tauri::command]
async fn check_unsaved_changes(
    state: tauri::State<'_, AppState>,
    document_id: String,
) -> Result<bool, String> {
    let doc_id = publisher_core::document_manager::DocumentId(
        uuid::Uuid::parse_str(&document_id).map_err(|_| "Invalid document ID".to_string())?,
    );

    let service = state
        .document_service
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(service.has_unsaved_changes(doc_id))
}

/// List all open documents
#[tauri::command]
async fn list_documents(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let service = state
        .document_service
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    let docs = service.list_documents();

    let doc_list: Vec<_> = docs
        .iter()
        .map(|(id, name)| {
            json!({
                "document_id": id.0.to_string(),
                "name": name
            })
        })
        .collect();

    Ok(json!({
        "documents": doc_list,
        "count": docs.len()
    })
    .to_string())
}

/// Mark document as modified
#[tauri::command]
async fn mark_document_modified(
    state: tauri::State<'_, AppState>,
    document_id: String,
) -> Result<String, String> {
    let doc_id = publisher_core::document_manager::DocumentId(
        uuid::Uuid::parse_str(&document_id).map_err(|_| "Invalid document ID".to_string())?,
    );

    let mut service = state
        .document_service
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    service.mark_modified(doc_id);

    Ok(json!({
        "success": true,
        "message": "Document marked as modified"
    })
    .to_string())
}

/// Command: Save a recovery file for auto-save
#[tauri::command]
async fn save_recovery_file(
    state: tauri::State<'_, AppState>,
    document_json: String,
) -> Result<(), String> {
    let recovery_path = state.recovery_dir.join("autosave.recovery");
    tauri::async_runtime::spawn_blocking(move || {
        fs::write(&recovery_path, document_json)
    })
    .await
    .map_err(|e| format!("Failed to spawn blocking task: {}", e))?
    .map_err(|e| format!("Failed to write recovery file: {}", e))?;
    Ok(())
}

/// Command: Check for existing recovery files
#[tauri::command]
async fn check_recovery_file(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    let recovery_path = state.recovery_dir.join("autosave.recovery");
    if recovery_path.exists() {
        fs::read_to_string(&recovery_path).map(Some).map_err(|e| format!("Failed to read recovery file: {}", e))
    } else {
        Ok(None)
    }
}

/// Command: Clear the recovery file (clean exit/manual save)
#[tauri::command]
async fn clear_recovery_file(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let recovery_path = state.recovery_dir.join("autosave.recovery");
    if recovery_path.exists() {
        fs::remove_file(&recovery_path).map_err(|e| format!("Failed to delete recovery file: {}", e))?;
    }
    Ok(())
}

/// Command: Save application preferences
#[tauri::command]
async fn save_preferences(
    state: tauri::State<'_, AppState>,
    preferences: AppPreferences,
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&preferences).map_err(|e| e.to_string())?;
    fs::write(&state.preferences_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Command: Load application preferences
#[tauri::command]
async fn load_preferences(state: tauri::State<'_, AppState>) -> Result<AppPreferences, String> {
    if state.preferences_path.exists() {
        let json = fs::read_to_string(&state.preferences_path).map_err(|e| e.to_string())?;
        let prefs: AppPreferences = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        Ok(prefs)
    } else {
        Ok(AppPreferences::default())
    }
}

fn main() {
    publisher_renderer::init();

    let color_engine = ColorEngine::new().expect("Failed to initialize color engine");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Setup directories
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            let recovery_dir = app_data_dir.join("recovery");
            let preferences_path = app_data_dir.join("preferences.json");
            fs::create_dir_all(&recovery_dir).expect("Failed to create recovery directory");
            
            app.manage(AppState {
                document_service: Mutex::new(DocumentService::new()),
                document_state: Mutex::new(None),
                color_engine: Mutex::new(color_engine),
                recovery_dir,
                preferences_path,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Document lifecycle commands
            new_document,
            open_document,
            save_document_as,
            close_document,
            check_unsaved_changes,
            list_documents,
            mark_document_modified,
            // Alternative file operations
            read_document,
            save_document_file,
            save_as_file,
            // Undo/Redo commands
            undo,
            redo,
            get_undo_history,
            get_redo_history,
            get_history_state,
            // Color commands
            convert_color,
            // Layout commands
            apply_grid_preset,
            find_snap,
            align_frames,
            distribute_frames,
            // Recovery commands
            save_recovery_file,
            check_recovery_file,
            clear_recovery_file,
            // Preferences commands
            save_preferences,
            load_preferences
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

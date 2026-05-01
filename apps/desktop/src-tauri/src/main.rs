// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod document_service;

use document_service::{DocumentService, DocumentServiceError};
use serde_json::json;
use std::sync::Mutex;
use tauri::Runtime;
use tauri_plugin_dialog::DialogExt;

/// Application state containing the document service
pub struct AppState {
    document_service: Mutex<DocumentService>,
}

/// Create a new empty document
#[tauri::command]
async fn new_document(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let mut service = state.document_service.lock().unwrap();
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
            let path_str = path.to_string();
            let mut service = state.document_service.lock().unwrap();

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

/// Save the active document
#[tauri::command]
async fn save_document(
    state: tauri::State<'_, AppState>,
    document_id: String,
) -> Result<String, String> {
    let doc_id = publisher_core::document_manager::DocumentId(
        uuid::Uuid::parse_str(&document_id).map_err(|_| "Invalid document ID".to_string())?,
    );

    let mut service = state.document_service.lock().unwrap();

    match service.save_document(doc_id) {
        Ok(path) => Ok(json!({
            "success": true,
            "file_path": path,
            "message": "Document saved successfully"
        })
        .to_string()),
        Err(e) => Err(format!("Save failed: {}", e)),
    }
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
            let path_str = path.to_string();
            let mut service = state.document_service.lock().unwrap();

            match service.save_document_as(doc_id, &path_str) {
                Ok(saved_path) => Ok(json!({
                    "success": true,
                    "file_path": saved_path,
                    "message": "Document saved successfully"
                })
                .to_string()),
                Err(e) => Err(format!("Save As failed: {}", e)),
            }
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

    let mut service = state.document_service.lock().unwrap();
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

    let service = state.document_service.lock().unwrap();
    Ok(service.has_unsaved_changes(doc_id))
}

/// List all open documents
#[tauri::command]
async fn list_documents(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let service = state.document_service.lock().unwrap();
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

    let mut service = state.document_service.lock().unwrap();
    service.mark_modified(doc_id);

    Ok(json!({
        "success": true,
        "message": "Document marked as modified"
    })
    .to_string())
}

fn main() {
    publisher_renderer::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            document_service: Mutex::new(DocumentService::new()),
        })
        .invoke_handler(tauri::generate_handler![
            new_document,
            open_document,
            save_document,
            save_document_as,
            close_document,
            check_unsaved_changes,
            list_documents,
            mark_document_modified,
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

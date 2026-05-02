use publisher_core::{
    AlignMode, Color as CoreColor, DistributeMode, Frame, GridPreset, Page, Pt, SnapEngine,
    SnapResult, SnapTarget,
};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Command: Find snapping points for an object
#[tauri::command]
pub fn find_snap(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    targets: Vec<SnapTarget>,
    threshold: f64,
) -> Result<SnapResult, String> {
    let engine = SnapEngine::new(threshold);
    Ok(engine.find_snap(x, y, width, height, &targets))
}

/// Command: Align multiple frames
#[tauri::command]
pub fn align_frames(frames: Vec<Frame>, mode: AlignMode) -> Result<Vec<(String, Pt, Pt)>, String> {
    Ok(publisher_core::align_frames(&frames, mode))
}

/// Command: Distribute multiple frames
#[tauri::command]
pub fn distribute_frames(
    frames: Vec<Frame>,
    mode: DistributeMode,
) -> Result<Vec<(String, Pt, Pt)>, String> {
    Ok(publisher_core::distribute_frames(&frames, mode))
}

/// Check undo/redo availability and counts
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryStateResponse {
    pub can_undo: bool,
    pub can_redo: bool,
    pub undo_count: usize,
    pub redo_count: usize,
}

/// Response for undo/redo operations with Action details
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryResponse {
    pub success: bool,
    pub message: String,
    pub can_undo: bool,
    pub can_redo: bool,
    pub undo_count: usize,
    pub redo_count: usize,
    pub action: Option<ActionData>,
}

/// Serializable action data returned to frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActionData {
    pub id: String,
    pub description: String,
    pub timestamp: u64,
    #[serde(with = "serde_bytes")]
    pub changeset: Vec<u8>,
}

/// Command: Undo the last action
#[tauri::command]
pub fn undo(state: State<'_, crate::AppState>) -> Result<HistoryResponse, String> {
    let mut doc_state = state
        .document_state
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    match &mut *doc_state {
        Some(state) => {
            if let Some(action) = state.history.undo() {
                let action_data = ActionData {
                    id: action.id.clone(),
                    description: action.description.clone(),
                    timestamp: action.timestamp,
                    changeset: action.changeset.clone(),
                };
                Ok(HistoryResponse {
                    success: true,
                    message: "Undo successful".to_string(),
                    can_undo: state.can_undo(),
                    can_redo: state.can_redo(),
                    undo_count: state.history.undo_count(),
                    redo_count: state.history.redo_count(),
                    action: Some(action_data),
                })
            } else {
                Ok(HistoryResponse {
                    success: false,
                    message: "Nothing to undo".to_string(),
                    can_undo: false,
                    can_redo: state.can_redo(),
                    undo_count: 0,
                    redo_count: state.history.redo_count(),
                    action: None,
                })
            }
        }
        None => Err("No document loaded".to_string()),
    }
}

/// Command: Redo the last undone action
#[tauri::command]
pub fn redo(state: State<'_, crate::AppState>) -> Result<HistoryResponse, String> {
    let mut doc_state = state
        .document_state
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    match &mut *doc_state {
        Some(state) => {
            if let Some(action) = state.history.redo() {
                let action_data = ActionData {
                    id: action.id.clone(),
                    description: action.description.clone(),
                    timestamp: action.timestamp,
                    changeset: action.changeset.clone(),
                };
                Ok(HistoryResponse {
                    success: true,
                    message: "Redo successful".to_string(),
                    can_undo: state.can_undo(),
                    can_redo: state.can_redo(),
                    undo_count: state.history.undo_count(),
                    redo_count: state.history.redo_count(),
                    action: Some(action_data),
                })
            } else {
                Ok(HistoryResponse {
                    success: false,
                    message: "Nothing to redo".to_string(),
                    can_undo: state.can_undo(),
                    can_redo: false,
                    undo_count: state.history.undo_count(),
                    redo_count: 0,
                    action: None,
                })
            }
        }
        None => Err("No document loaded".to_string()),
    }
}

/// Command: Get the undo history for display in UI
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub id: String,
    pub description: String,
}

#[tauri::command]
pub fn get_undo_history(state: State<'_, crate::AppState>) -> Result<Vec<HistoryItem>, String> {
    let doc_state = state
        .document_state
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    match &*doc_state {
        Some(state) => {
            let history = state
                .undo_history()
                .into_iter()
                .map(|(id, description)| HistoryItem { id, description })
                .collect();
            Ok(history)
        }
        None => Err("No document loaded".to_string()),
    }
}

/// Command: Get the redo history for display in UI
#[tauri::command]
pub fn get_redo_history(state: State<'_, crate::AppState>) -> Result<Vec<HistoryItem>, String> {
    let doc_state = state
        .document_state
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    match &*doc_state {
        Some(state) => {
            let history = state
                .redo_history()
                .into_iter()
                .map(|(id, description)| HistoryItem { id, description })
                .collect();
            Ok(history)
        }
        None => Err("No document loaded".to_string()),
    }
}

/// Command: Check undo/redo availability and counts
#[tauri::command]
pub fn get_history_state(
    state: State<'_, crate::AppState>,
) -> Result<HistoryStateResponse, String> {
    let doc_state = state
        .document_state
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    match &*doc_state {
        Some(state) => Ok(HistoryStateResponse {
            can_undo: state.can_undo(),
            can_redo: state.can_redo(),
            undo_count: state.history.undo_count(),
            redo_count: state.history.redo_count(),
        }),
        None => Err("No document loaded".to_string()),
    }
}

/// Command: Convert color between RGB and CMYK
#[tauri::command]
pub fn convert_color(
    state: State<'_, crate::AppState>,
    color: CoreColor,
) -> Result<CoreColor, String> {
    let color_engine = state
        .color_engine
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(color_engine.convert_core_color(&color))
}

/// Command: Get RGB preview for a color
#[tauri::command]
pub fn get_color_preview(
    state: State<'_, crate::AppState>,
    color: CoreColor,
) -> Result<CoreColor, String> {
    let color_engine = state
        .color_engine
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(color_engine.convert_core_color(&color))
}

/// Command: Apply a grid preset to a page
#[tauri::command]
pub fn apply_grid_preset(mut page: Page, preset: GridPreset) -> Result<Page, String> {
    page.apply_grid_preset(preset);
    Ok(page)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_response() {
        let resp = HistoryResponse {
            success: true,
            message: "Test".to_string(),
            can_undo: true,
            can_redo: false,
            undo_count: 1,
            redo_count: 0,
            action: None,
        };
        assert!(resp.success);
        assert!(resp.can_undo);
    }

    #[test]
    fn test_history_item_serialization() {
        let item = HistoryItem {
            id: "test-id".to_string(),
            description: "Test action".to_string(),
        };
        let json = serde_json::to_string(&item).expect("Failed to serialize");
        let deserialized: HistoryItem = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.id, "test-id");
        assert_eq!(deserialized.description, "Test action");
    }
}

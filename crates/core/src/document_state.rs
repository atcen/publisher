use crate::{Action, Document, History};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentState {
    pub document: Document,
    pub history: History,
}

impl DocumentState {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            history: History::new(None),
        }
    }

    pub fn with_max_undo(document: Document, max_steps: usize) -> Self {
        Self {
            document,
            history: History::new(Some(max_steps)),
        }
    }

    pub fn record_action(&mut self, description: impl Into<String>, changeset: Vec<u8>) {
        let action = Action::new(description, changeset);
        self.history.push(action);
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn undo_history(&self) -> Vec<(String, String)> {
        self.history
            .undo_history()
            .into_iter()
            .map(|action| (action.id.clone(), action.description.clone()))
            .collect()
    }

    pub fn redo_history(&self) -> Vec<(String, String)> {
        self.history
            .redo_history()
            .into_iter()
            .map(|action| (action.id.clone(), action.description.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bleed, Metadata, Styles, Unit};

    fn create_test_document() -> Document {
        Document {
            metadata: Metadata {
                name: "Test Doc".to_string(),
                author: "Test Author".to_string(),
                description: "Test Description".to_string(),
                created_at: 0,
                modified_at: 0,
                dpi: 300,
                default_unit: Unit::Point,
                default_bleed: Bleed {
                    top: crate::Pt(0.0),
                    bottom: crate::Pt(0.0),
                    inside: crate::Pt(0.0),
                    outside: crate::Pt(0.0),
                },
                color_profile: "sRGB".to_string(),
                facing_pages: true,
            },
            fonts: vec![],
            icc_profiles: vec![],
            swatches: vec![],
            styles: Styles::default(),
            spreads: vec![],
            parent_pages: vec![],
            layers: vec![crate::Layer::new("l1", "Layer 1")],
            baseline_grid: crate::BaselineGrid::default(),
        }
    }

    #[test]
    fn test_document_state_creation() {
        let doc = create_test_document();
        let state = DocumentState::new(doc.clone());

        assert_eq!(state.document.metadata.name, doc.metadata.name);
        assert!(!state.can_undo());
        assert!(!state.can_redo());
    }

    #[test]
    fn test_record_action() {
        let doc = create_test_document();
        let mut state = DocumentState::new(doc);

        state.record_action("Edit metadata", vec![1, 2, 3]);

        assert!(state.can_undo());
        assert!(!state.can_redo());
    }

    #[test]
    fn test_undo_history_display() {
        let doc = create_test_document();
        let mut state = DocumentState::new(doc);

        state.record_action("Action 1", vec![1, 2, 3]);
        state.record_action("Action 2", vec![4, 5, 6]);

        let history = state.undo_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].1, "Action 2");
        assert_eq!(history[1].1, "Action 1");
    }

    #[test]
    fn test_document_state_serialization() {
        let doc = create_test_document();
        let mut state = DocumentState::new(doc);

        state.record_action("Test action", vec![1, 2, 3]);

        let json = serde_json::to_string(&state).expect("Serialization failed");
        let deserialized: DocumentState =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(
            deserialized.document.metadata.name,
            state.document.metadata.name
        );
        assert!(deserialized.can_undo());
    }

    #[test]
    fn test_bounded_undo_history() {
        let doc = create_test_document();
        let mut state = DocumentState::with_max_undo(doc, 2);

        state.record_action("Action 1", vec![1]);
        state.record_action("Action 2", vec![2]);
        state.record_action("Action 3", vec![3]);
        state.record_action("Action 4", vec![4]);

        let history = state.undo_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].1, "Action 4");
        assert_eq!(history[1].1, "Action 3");
    }
}

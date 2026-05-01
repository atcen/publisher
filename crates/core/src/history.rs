use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// Represents a single action that can be undone/redone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub description: String,
    pub timestamp: u64,
    pub changeset: Vec<u8>,
}

impl Action {
    pub fn new(description: impl Into<String>, changeset: Vec<u8>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            id: Uuid::new_v4().to_string(),
            description: description.into(),
            timestamp,
            changeset,
        }
    }
}

/// Document history with unlimited undo/redo per session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    undo_stack: VecDeque<Action>,
    redo_stack: VecDeque<Action>,
    max_undo_steps: Option<usize>,
}

impl Default for History {
    fn default() -> Self {
        Self::new(None)
    }
}

impl History {
    pub fn new(max_undo_steps: Option<usize>) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_undo_steps,
        }
    }

    pub fn push(&mut self, action: Action) {
        self.undo_stack.push_back(action);
        self.redo_stack.clear();

        if let Some(max) = self.max_undo_steps {
            while self.undo_stack.len() > max {
                self.undo_stack.pop_front();
            }
        }
    }

    pub fn undo(&mut self) -> Option<Action> {
        if let Some(action) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(action.clone());
            Some(action)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<Action> {
        if let Some(action) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(action.clone());
            Some(action)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_history(&self) -> Vec<&Action> {
        self.undo_stack.iter().rev().collect()
    }

    pub fn redo_history(&self) -> Vec<&Action> {
        self.redo_stack.iter().rev().collect()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    pub fn jump_to_action(&mut self, target_id: &str) -> Option<Vec<Action>> {
        let mut jumped_actions = Vec::new();

        while !self.undo_stack.is_empty() {
            let action = self.undo_stack.back().cloned()?;
            if action.id == target_id {
                return Some(jumped_actions);
            }
            if let Some(action) = self.undo_stack.pop_back() {
                jumped_actions.push(action);
            }
        }

        for action in jumped_actions.drain(..).rev() {
            self.undo_stack.push_back(action);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_action(desc: &str) -> Action {
        Action::new(desc, vec![1, 2, 3])
    }

    #[test]
    fn test_history_creation() {
        let history = History::new(None);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_push_and_undo() {
        let mut history = History::new(None);
        let action = create_test_action("Edit text");
        history.push(action.clone());

        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 1);

        let undone = history.undo();
        assert!(undone.is_some());
        assert_eq!(undone.unwrap().id, action.id);
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_undo_and_redo() {
        let mut history = History::new(None);
        let action1 = create_test_action("Action 1");
        let action2 = create_test_action("Action 2");

        history.push(action1.clone());
        history.push(action2.clone());

        assert_eq!(history.undo_count(), 2);

        let undone = history.undo().unwrap();
        assert_eq!(undone.id, action2.id);
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 1);

        let redone = history.redo().unwrap();
        assert_eq!(redone.id, action2.id);
        assert_eq!(history.undo_count(), 2);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_new_action_clears_redo_stack() {
        let mut history = History::new(None);
        let action1 = create_test_action("Action 1");
        let action2 = create_test_action("Action 2");

        history.push(action1.clone());
        history.push(action2.clone());
        history.undo();
        assert!(history.can_redo());

        let action3 = create_test_action("Action 3");
        history.push(action3.clone());

        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 2);
    }

    #[test]
    fn test_undo_history_returns_actions_in_order() {
        let mut history = History::new(None);
        let action1 = create_test_action("Action 1");
        let action2 = create_test_action("Action 2");
        let action3 = create_test_action("Action 3");

        history.push(action1.clone());
        history.push(action2.clone());
        history.push(action3.clone());

        let undo_list = history.undo_history();
        assert_eq!(undo_list.len(), 3);
        assert_eq!(undo_list[0].id, action3.id);
        assert_eq!(undo_list[1].id, action2.id);
        assert_eq!(undo_list[2].id, action1.id);
    }

    #[test]
    fn test_max_undo_steps() {
        let mut history = History::new(Some(2));
        let action1 = create_test_action("Action 1");
        let action2 = create_test_action("Action 2");
        let action3 = create_test_action("Action 3");
        let action4 = create_test_action("Action 4");

        history.push(action1);
        history.push(action2);
        history.push(action3);
        history.push(action4);

        assert_eq!(history.undo_count(), 2);
        let undo_list = history.undo_history();
        assert_eq!(undo_list[0].description, "Action 4");
        assert_eq!(undo_list[1].description, "Action 3");
    }

    #[test]
    fn test_clear_history() {
        let mut history = History::new(None);
        history.push(create_test_action("Action 1"));
        history.push(create_test_action("Action 2"));

        assert_eq!(history.undo_count(), 2);
        history.clear();

        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_jump_to_action() {
        let mut history = History::new(None);
        let action1 = create_test_action("Action 1");
        let action2 = create_test_action("Action 2");
        let action3 = create_test_action("Action 3");

        history.push(action1.clone());
        history.push(action2.clone());
        history.push(action3.clone());

        let result = history.jump_to_action(&action1.id);
        assert!(result.is_some());

        let jumped = result.unwrap();
        assert_eq!(jumped.len(), 2);
        assert_eq!(jumped[0].id, action3.id);
        assert_eq!(jumped[1].id, action2.id);

        assert_eq!(history.undo_count(), 1);
    }

    #[test]
    fn test_serialization() {
        let mut history = History::new(Some(10));
        history.push(create_test_action("Test action"));

        let json = serde_json::to_string(&history).expect("Serialization failed");
        let deserialized: History =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.undo_count(), 1);
        assert_eq!(deserialized.max_undo_steps, Some(10));
    }
}

use crate::Document;
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a document instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentId(pub Uuid);

impl DocumentId {
    pub fn new() -> Self {
        DocumentId(Uuid::new_v4())
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks the state of a document
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentState {
    /// Document is clean, no unsaved changes
    Clean,
    /// Document has unsaved changes
    Dirty,
}

/// Represents a loaded document with its metadata
#[derive(Debug, Clone)]
pub struct LoadedDocument {
    pub id: DocumentId,
    pub document: Document,
    pub state: DocumentState,
    pub file_path: Option<String>,
}

impl LoadedDocument {
    pub fn new(document: Document, file_path: Option<String>) -> Self {
        Self {
            id: DocumentId::new(),
            document,
            state: DocumentState::Clean,
            file_path,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.state = DocumentState::Dirty;
    }

    pub fn mark_clean(&mut self) {
        self.state = DocumentState::Clean;
    }

    pub fn is_dirty(&self) -> bool {
        self.state == DocumentState::Dirty
    }
}

/// Manages multiple open documents
pub struct DocumentManager {
    documents: HashMap<DocumentId, LoadedDocument>,
    active_document: Option<DocumentId>,
}

impl DocumentManager {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            active_document: None,
        }
    }

    /// Create a new document and load it
    pub fn create_document(&mut self, document: Document) -> DocumentId {
        let loaded = LoadedDocument::new(document, None);
        let id = loaded.id;
        self.documents.insert(id, loaded);
        self.active_document = Some(id);
        id
    }

    /// Load an existing document from disk
    pub fn load_document(&mut self, document: Document, file_path: String) -> DocumentId {
        let loaded = LoadedDocument::new(document, Some(file_path));
        let id = loaded.id;
        self.documents.insert(id, loaded);
        self.active_document = Some(id);
        id
    }

    /// Close a document, returning it if successful
    pub fn close_document(&mut self, id: DocumentId) -> Option<LoadedDocument> {
        if self.active_document == Some(id) {
            self.active_document = None;
        }
        self.documents.remove(&id)
    }

    /// Get a reference to a loaded document
    pub fn get(&self, id: DocumentId) -> Option<&LoadedDocument> {
        self.documents.get(&id)
    }

    /// Get a mutable reference to a loaded document
    pub fn get_mut(&mut self, id: DocumentId) -> Option<&mut LoadedDocument> {
        self.documents.get_mut(&id)
    }

    /// Get the currently active document
    pub fn active(&self) -> Option<&LoadedDocument> {
        self.active_document.and_then(|id| self.documents.get(&id))
    }

    /// Get mutable reference to the active document
    pub fn active_mut(&mut self) -> Option<&mut LoadedDocument> {
        let id = self.active_document?;
        self.documents.get_mut(&id)
    }

    /// Set the active document
    pub fn set_active(&mut self, id: DocumentId) -> bool {
        if self.documents.contains_key(&id) {
            self.active_document = Some(id);
            true
        } else {
            false
        }
    }

    /// List all open documents
    pub fn list_all(&self) -> Vec<DocumentId> {
        self.documents.keys().copied().collect()
    }

    /// Count of open documents
    pub fn count(&self) -> usize {
        self.documents.len()
    }

    /// Check if any document has unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.documents.values().any(|doc| doc.is_dirty())
    }

    /// Get documents that need to be saved
    pub fn dirty_documents(&self) -> Vec<DocumentId> {
        self.documents
            .iter()
            .filter(|(_, doc)| doc.is_dirty())
            .map(|(_, doc)| doc.id)
            .collect()
    }
}

impl Default for DocumentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bleed, Metadata, Pt, Unit};

    fn create_test_doc() -> Document {
        Document {
            metadata: Metadata {
                name: "Test".to_string(),
                author: "Author".to_string(),
                description: "".to_string(),
                created_at: 0,
                modified_at: 0,
                dpi: 300,
                default_unit: Unit::Point,
                default_bleed: Bleed {
                    top: Pt(0.0),
                    bottom: Pt(0.0),
                    inside: Pt(0.0),
                    outside: Pt(0.0),
                },
                color_profile: "sRGB".to_string(),
            },
            swatches: vec![],
            styles: Default::default(),
            spreads: vec![],
        }
    }

    #[test]
    fn test_create_document() {
        let mut manager = DocumentManager::new();
        let doc = create_test_doc();
        let id = manager.create_document(doc);

        assert_eq!(manager.count(), 1);
        assert!(manager.get(id).is_some());
        assert_eq!(manager.active().unwrap().id, id);
    }

    #[test]
    fn test_load_document() {
        let mut manager = DocumentManager::new();
        let doc = create_test_doc();
        let id = manager.load_document(doc, "/path/to/file.publisher".to_string());

        assert_eq!(manager.count(), 1);
        let loaded = manager.get(id).unwrap();
        assert_eq!(loaded.file_path, Some("/path/to/file.publisher".to_string()));
        assert_eq!(loaded.state, DocumentState::Clean);
    }

    #[test]
    fn test_mark_dirty() {
        let mut manager = DocumentManager::new();
        let doc = create_test_doc();
        let id = manager.create_document(doc);

        assert!(!manager.get(id).unwrap().is_dirty());

        manager.get_mut(id).unwrap().mark_dirty();
        assert!(manager.get(id).unwrap().is_dirty());
    }

    #[test]
    fn test_close_document() {
        let mut manager = DocumentManager::new();
        let doc1 = create_test_doc();
        let doc2 = create_test_doc();

        let id1 = manager.create_document(doc1);
        let id2 = manager.create_document(doc2);

        assert_eq!(manager.count(), 2);

        let closed = manager.close_document(id1);
        assert!(closed.is_some());
        assert_eq!(manager.count(), 1);
        assert_eq!(manager.active().unwrap().id, id2);
    }

    #[test]
    fn test_multiple_documents() {
        let mut manager = DocumentManager::new();
        let doc1 = create_test_doc();
        let doc2 = create_test_doc();

        let id1 = manager.create_document(doc1);
        let id2 = manager.create_document(doc2);

        manager.set_active(id1);
        assert_eq!(manager.active().unwrap().id, id1);

        manager.set_active(id2);
        assert_eq!(manager.active().unwrap().id, id2);
    }

    #[test]
    fn test_dirty_documents() {
        let mut manager = DocumentManager::new();
        let doc1 = create_test_doc();
        let doc2 = create_test_doc();

        let id1 = manager.create_document(doc1);
        let _id2 = manager.create_document(doc2);

        manager.get_mut(id1).unwrap().mark_dirty();

        let dirty = manager.dirty_documents();
        assert_eq!(dirty.len(), 1);
        assert!(dirty.contains(&id1));

        assert!(manager.has_unsaved_changes());
    }

    #[test]
    fn test_list_all() {
        let mut manager = DocumentManager::new();
        let doc1 = create_test_doc();
        let doc2 = create_test_doc();

        let id1 = manager.create_document(doc1);
        let id2 = manager.create_document(doc2);

        let all = manager.list_all();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&id1));
        assert!(all.contains(&id2));
    }
}

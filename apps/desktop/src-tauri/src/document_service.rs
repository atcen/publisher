use publisher_core::{
    builder::DocumentBuilder, document_manager::*, paper::PaperFormat, persistence::*, Document,
};
use std::fs;
use std::path::PathBuf;

/// Error type for document service operations
#[derive(Debug, Clone)]
pub enum DocumentServiceError {
    Persistence(String),
    IO(String),
    NotFound(String),
    InvalidPath(String),
}

impl std::fmt::Display for DocumentServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentServiceError::Persistence(msg) => write!(f, "Persistence error: {}", msg),
            DocumentServiceError::IO(msg) => write!(f, "IO error: {}", msg),
            DocumentServiceError::NotFound(msg) => write!(f, "Document not found: {}", msg),
            DocumentServiceError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
        }
    }
}

impl From<PersistenceError> for DocumentServiceError {
    fn from(err: PersistenceError) -> Self {
        DocumentServiceError::Persistence(err.to_string())
    }
}

impl From<std::io::Error> for DocumentServiceError {
    fn from(err: std::io::Error) -> Self {
        DocumentServiceError::IO(err.to_string())
    }
}

/// Manages document operations on disk and in memory
pub struct DocumentService {
    manager: DocumentManager,
}

impl DocumentService {
    pub fn new() -> Self {
        Self {
            manager: DocumentManager::new(),
        }
    }

    /// Create a new blank document
    pub fn create_new(&mut self, name: &str) -> Result<DocumentId, DocumentServiceError> {
        let doc = DocumentBuilder::new()
            .with_name(name)
            .with_format(PaperFormat::A4)
            .with_pages(1)
            .with_facing_pages(false)
            .build();

        Ok(self.manager.create_document(doc))
    }

    /// Open a document from disk
    pub fn open_document(&mut self, file_path: &str) -> Result<DocumentId, DocumentServiceError> {
        let path = PathBuf::from(file_path);

        // Validate path exists
        if !path.exists() {
            return Err(DocumentServiceError::NotFound(format!(
                "File does not exist: {}",
                file_path
            )));
        }

        // Validate extension is .publisher
        if path.extension().and_then(|s| s.to_str()) != Some("publisher") {
            return Err(DocumentServiceError::InvalidPath(
                "File must have .publisher extension".to_string(),
            ));
        }

        // Read file
        let file_contents = fs::read(&path).map_err(|e| DocumentServiceError::IO(e.to_string()))?;

        // Deserialize document
        let document = deserialize_document(&file_contents)?;

        // Load into manager
        Ok(self.manager.load_document(document, file_path.to_string()))
    }

    /// Save a document to disk
    pub fn save_document(&mut self, id: DocumentId) -> Result<String, DocumentServiceError> {
        let doc = self
            .manager
            .get(id)
            .ok_or_else(|| DocumentServiceError::NotFound("Document not found".to_string()))?;

        let file_path = doc
            .file_path
            .as_ref()
            .ok_or_else(|| {
                DocumentServiceError::InvalidPath("Document has no file path".to_string())
            })?
            .clone();

        let bytes = serialize_document(&doc.document)?;
        fs::write(&file_path, bytes).map_err(|e| DocumentServiceError::IO(e.to_string()))?;

        // Mark document as clean after successful save
        self.manager.get_mut(id).unwrap().mark_clean();

        Ok(file_path)
    }

    /// Save a document to a new file path (Save As)
    pub fn save_document_as(
        &mut self,
        id: DocumentId,
        new_path: &str,
    ) -> Result<String, DocumentServiceError> {
        let new_path_buf = PathBuf::from(new_path);

        // Validate extension
        if new_path_buf.extension().and_then(|s| s.to_str()) != Some("publisher") {
            return Err(DocumentServiceError::InvalidPath(
                "File must have .publisher extension".to_string(),
            ));
        }

        let doc = self
            .manager
            .get_mut(id)
            .ok_or_else(|| DocumentServiceError::NotFound("Document not found".to_string()))?;

        // Serialize and write to new path BEFORE updating the file_path reference
        // This ensures in-memory state is only updated if the write succeeds
        let bytes = serialize_document(&doc.document)?;
        fs::write(new_path, bytes).map_err(|e| DocumentServiceError::IO(e.to_string()))?;

        // Only update file path AFTER successful write
        doc.file_path = Some(new_path.to_string());

        // Mark as clean
        doc.mark_clean();

        Ok(new_path.to_string())
    }

    /// Close a document
    pub fn close_document(&mut self, id: DocumentId) -> Result<(), DocumentServiceError> {
        self.manager
            .close_document(id)
            .ok_or_else(|| DocumentServiceError::NotFound("Document not found".to_string()))?;
        Ok(())
    }

    /// Get list of all open documents
    pub fn list_documents(&self) -> Vec<(DocumentId, String)> {
        self.manager
            .list_all()
            .into_iter()
            .map(|id| {
                let doc = self.manager.get(id).unwrap();
                let name = doc.document.metadata.name.clone();
                (id, name)
            })
            .collect()
    }

    /// Check if document has unsaved changes
    pub fn has_unsaved_changes(&self, id: DocumentId) -> bool {
        self.manager
            .get(id)
            .map(|doc| doc.is_dirty())
            .unwrap_or(false)
    }

    /// Mark document as modified
    pub fn mark_modified(&mut self, id: DocumentId) {
        if let Some(doc) = self.manager.get_mut(id) {
            doc.mark_dirty();
        }
    }

    /// Get active document ID
    pub fn active_document(&self) -> Option<DocumentId> {
        self.manager.active().map(|doc| doc.id)
    }

    /// Set active document
    pub fn set_active(&mut self, id: DocumentId) -> bool {
        self.manager.set_active(id)
    }

    /// Get document by ID (for serialization to frontend)
    pub fn get_document(&self, id: DocumentId) -> Option<Document> {
        self.manager.get(id).map(|d| d.document.clone())
    }
}

impl Default for DocumentService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_new_document() {
        let mut service = DocumentService::new();
        let result = service.create_new("Test Document");
        assert!(result.is_ok());
    }

    #[test]
    fn test_save_and_open_document() {
        let mut service = DocumentService::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.publisher");

        // Create and save
        let id = service.create_new("Test Doc").unwrap();
        let save_result = service.save_document_as(id, file_path.to_str().unwrap());
        assert!(save_result.is_ok());
        assert!(file_path.exists());

        // Open the saved file
        let mut service2 = DocumentService::new();
        let open_result = service2.open_document(file_path.to_str().unwrap());
        assert!(open_result.is_ok());

        let opened_id = open_result.unwrap();
        let opened_doc = service2.get_document(opened_id).unwrap();
        assert_eq!(opened_doc.metadata.name, "Test Doc");
    }

    #[test]
    fn test_mark_dirty() {
        let mut service = DocumentService::new();
        let id = service.create_new("Test").unwrap();

        assert!(!service.has_unsaved_changes(id));

        service.mark_modified(id);
        assert!(service.has_unsaved_changes(id));
    }

    #[test]
    fn test_open_invalid_file() {
        let mut service = DocumentService::new();
        let result = service.open_document("/nonexistent/path.publisher");
        assert!(result.is_err());
    }

    #[test]
    fn test_close_document() {
        let mut service = DocumentService::new();
        let id = service.create_new("Test").unwrap();
        assert!(service.get_document(id).is_some());

        let close_result = service.close_document(id);
        assert!(close_result.is_ok());
        assert!(service.get_document(id).is_none());
    }
}

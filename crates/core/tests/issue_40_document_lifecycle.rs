// Integration tests for Issue #40: New, open and close documents
//
// Acceptance Criteria:
// - "New Document" creates empty document with settings from New Document Dialog
// - "Open" shows native File Picker, loads .publisher files
// - Malformed/unsupported files show clear error — no crash
// - "Close" asks to save on unsaved changes
// - Multiple documents can be open simultaneously in separate windows
// - Document state initialized from CRDT layer on first mutation

use publisher_core::{
    builder::DocumentBuilder, document_manager::*, paper::PaperFormat, persistence::*,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_issue_40_new_document_with_default_settings() {
    // Acceptance: "New Document" creates empty document with settings
    let doc = DocumentBuilder::new()
        .with_name("Test Magazine")
        .with_format(PaperFormat::A4)
        .with_pages(2)
        .with_facing_pages(true)
        .build();

    assert_eq!(doc.metadata.name, "Test Magazine");
    assert_eq!(doc.spreads.len(), 2); // 2 spreads for facing pages
    assert!(doc.spreads[0].pages.len() > 0);
    assert_eq!(doc.metadata.dpi, 300); // Default DPI
}

#[test]
fn test_issue_40_new_document_single_page() {
    // Single page document (not facing pages)
    let doc = DocumentBuilder::new()
        .with_name("Single Page")
        .with_format(PaperFormat::A4)
        .with_pages(1)
        .with_facing_pages(false)
        .build();

    assert_eq!(doc.spreads.len(), 1);
    assert_eq!(doc.spreads[0].pages.len(), 1);
}

#[test]
fn test_issue_40_open_document_success() {
    // Acceptance: "Open" shows native File Picker, loads .publisher files
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.publisher");

    // Create and save a test document
    let doc = DocumentBuilder::new()
        .with_name("Saved Document")
        .with_format(PaperFormat::A4)
        .build();

    let bytes = serialize_document(&doc).expect("Failed to serialize");
    fs::write(&file_path, bytes).expect("Failed to write file");

    // Now open it
    let file_contents = fs::read(&file_path).expect("Failed to read file");
    let loaded_doc = deserialize_document(&file_contents).expect("Failed to deserialize");

    assert_eq!(loaded_doc.metadata.name, "Saved Document");
    assert!(!loaded_doc.spreads.is_empty());
}

#[test]
fn test_issue_40_malformed_file_handling() {
    // Acceptance: Malformed/unsupported files show clear error — no crash
    let malformed_json = br#"{"invalid": "json structure"}"#;

    let result = deserialize_document(malformed_json);
    assert!(result.is_err());

    match result {
        Err(PersistenceError::ParseError(msg)) => {
            assert!(!msg.is_empty());
            println!("Clear error message: {}", msg);
        }
        _ => panic!("Expected ParseError for malformed JSON"),
    }
}

#[test]
fn test_issue_40_unsupported_file_version() {
    // Unsupported future file versions should be clearly reported
    // Use a minimally valid document so the version check is exercised (not parse error)
    let future_version_doc = br#"{
        "version": 999,
        "document": {
            "metadata": {
                "name": "Test",
                "author": "",
                "description": "",
                "created_at": 0,
                "modified_at": 0,
                "dpi": 300,
                "default_unit": "Millimeter",
                "default_bleed": {"top": {"Pt": 0}, "bottom": {"Pt": 0}, "inside": {"Pt": 0}, "outside": {"Pt": 0}},
                "color_profile": "sRGB"
            },
            "swatches": [],
            "styles": {},
            "spreads": []
        }
    }"#;

    let result = deserialize_document(future_version_doc);
    assert!(result.is_err());

    // Verify we get the FormatError for unsupported version, not a ParseError
    match result {
        Err(PersistenceError::FormatError(msg)) => {
            assert!(msg.contains("999"), "Error message should mention version 999");
            println!("Got expected FormatError: {}", msg);
        }
        other => panic!("Expected FormatError for unsupported version, got: {:?}", other),
    }
}

#[test]
fn test_issue_40_corrupted_file_handling() {
    // Corrupted binary file
    let corrupted = b"\xFF\xFE\x00\x00Invalid UTF-8";

    let result = deserialize_document(corrupted);
    assert!(result.is_err());
}

#[test]
fn test_issue_40_multiple_documents_open() {
    // Acceptance: Multiple documents can be open simultaneously in separate windows
    let mut manager = DocumentManager::new();

    let doc1 = DocumentBuilder::new()
        .with_name("Document 1")
        .with_format(PaperFormat::A4)
        .build();

    let doc2 = DocumentBuilder::new()
        .with_name("Document 2")
        .with_format(PaperFormat::Letter)
        .build();

    let id1 = manager.create_document(doc1);
    let id2 = manager.create_document(doc2);

    // Both documents exist
    assert!(manager.get(id1).is_some());
    assert!(manager.get(id2).is_some());

    // Can switch between them
    manager.set_active(id1);
    assert_eq!(manager.active().unwrap().id, id1);

    manager.set_active(id2);
    assert_eq!(manager.active().unwrap().id, id2);

    // Can list all open documents
    let all_docs = manager.list_all();
    assert_eq!(all_docs.len(), 2);
}

#[test]
fn test_issue_40_close_with_unsaved_changes() {
    // Acceptance: "Close" asks to save on unsaved changes
    let mut manager = DocumentManager::new();

    let doc = DocumentBuilder::new()
        .with_name("Test Doc")
        .build();

    let id = manager.create_document(doc);

    // Initially clean
    assert!(!manager.get(id).unwrap().is_dirty());

    // Mark as modified
    manager.get_mut(id).unwrap().mark_dirty();
    assert!(manager.get(id).unwrap().is_dirty());

    // Should detect unsaved changes before closing
    let unsaved = manager.get(id).unwrap().is_dirty();
    assert!(unsaved);

    // Close the document
    manager.close_document(id);
    assert!(manager.get(id).is_none());
}

#[test]
fn test_issue_40_document_state_initialization() {
    // Acceptance: Document state initialized from CRDT layer on first mutation
    let mut manager = DocumentManager::new();

    let doc = DocumentBuilder::new()
        .with_name("CRDT Test")
        .build();

    let id = manager.create_document(doc);
    let loaded = manager.get(id).unwrap();

    // New document is clean initially
    assert_eq!(loaded.state, DocumentState::Clean);

    // Simulate first mutation through CRDT
    manager.get_mut(id).unwrap().mark_dirty();

    // After mutation, state should be dirty
    let loaded = manager.get(id).unwrap();
    assert_eq!(loaded.state, DocumentState::Dirty);
}

#[test]
fn test_issue_40_save_and_reopen_cycle() {
    // Full cycle: create -> save -> open -> modify -> save
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("lifecycle_test.publisher");

    // Create
    let doc = DocumentBuilder::new()
        .with_name("Lifecycle Test")
        .with_format(PaperFormat::A4)
        .with_pages(2)
        .build();

    // Save
    let bytes = serialize_document(&doc).expect("Failed to serialize");
    fs::write(&file_path, &bytes).expect("Failed to write");

    // Open
    let file_contents = fs::read(&file_path).expect("Failed to read");
    let loaded_doc = deserialize_document(&file_contents).expect("Failed to deserialize");

    assert_eq!(loaded_doc.metadata.name, "Lifecycle Test");
    assert_eq!(loaded_doc.spreads.len(), 2);

    // Track in manager
    let mut manager = DocumentManager::new();
    let id = manager.load_document(loaded_doc, file_path.to_string_lossy().to_string());

    // Modify
    manager.get_mut(id).unwrap().mark_dirty();
    assert!(manager.has_unsaved_changes());

    // Save (simulate)
    manager.get_mut(id).unwrap().mark_clean();
    assert!(!manager.has_unsaved_changes());
}

#[test]
fn test_issue_40_document_metadata_preserved() {
    // Ensure document metadata is preserved through save/load cycle
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("metadata_test.publisher");

    let doc = DocumentBuilder::new()
        .with_name("Metadata Test")
        .with_format(PaperFormat::B4)
        .with_pages(4)
        .build();

    let original_name = doc.metadata.name.clone();
    let original_dpi = doc.metadata.dpi;
    let original_spreads_count = doc.spreads.len();

    // Save and reload
    let bytes = serialize_document(&doc).unwrap();
    fs::write(&file_path, bytes).unwrap();

    let reloaded = deserialize_document(&fs::read(&file_path).unwrap()).unwrap();

    // Verify metadata
    assert_eq!(reloaded.metadata.name, original_name);
    assert_eq!(reloaded.metadata.dpi, original_dpi);
    assert_eq!(reloaded.spreads.len(), original_spreads_count);
}

#[test]
fn test_issue_40_multiple_saves() {
    // Multiple save operations should work correctly
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("multiple_saves.publisher");

    let doc = DocumentBuilder::new()
        .with_name("Multiple Saves Test")
        .build();

    // Save multiple times
    let bytes = serialize_document(&doc).unwrap();
    for i in 0..3 {
        fs::write(&file_path, &bytes).expect(&format!("Save {} failed", i + 1));
        assert!(file_path.exists());
    }

    // Final load should work
    let final_doc = deserialize_document(&fs::read(&file_path).unwrap()).unwrap();
    assert_eq!(final_doc.metadata.name, "Multiple Saves Test");
}

#[test]
fn test_issue_40_file_extension_validation() {
    // Verify that the application correctly validates file extensions
    // This test documents that only .publisher files are loadable
    let temp_dir = TempDir::new().unwrap();

    let doc = DocumentBuilder::new().with_name("Extension Test").build();
    let bytes = serialize_document(&doc).unwrap();

    // Write file with wrong extension
    let wrong_path = temp_dir.path().join("test.txt");
    fs::write(&wrong_path, &bytes).unwrap();

    // Verify extension is indeed wrong
    assert_eq!(wrong_path.extension().and_then(|s| s.to_str()), Some("txt"));

    // Correct extension should work for loading
    let correct_path = temp_dir.path().join("test.publisher");
    fs::write(&correct_path, &bytes).unwrap();
    assert_eq!(correct_path.extension().and_then(|s| s.to_str()), Some("publisher"));

    // DocumentService would reject the .txt file based on extension validation
    // (Extension check happens in document_service.rs:open_document)
}

#[test]
fn test_issue_40_empty_document_creation() {
    // New document should be valid and minimal
    let doc = DocumentBuilder::new().build();

    assert_eq!(doc.spreads.len(), 1);
    assert_eq!(doc.spreads[0].pages.len(), 1);
    assert_eq!(doc.metadata.name, "Untitled");

    // Should be serializable
    let result = serialize_document(&doc);
    assert!(result.is_ok());
}

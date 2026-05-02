use crate::Document;
use serde_json;

/// Error type for document persistence operations
#[derive(Debug, Clone)]
pub enum PersistenceError {
    /// Failed to read file
    ReadError(String),
    /// Failed to write file
    WriteError(String),
    /// Failed to serialize document to JSON
    SerializationError(String),
    /// Invalid JSON structure
    ParseError(String),
    /// Unsupported file version or format
    FormatError(String),
    /// File does not exist
    FileNotFound(String),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::ReadError(msg) => write!(f, "Read error: {}", msg),
            PersistenceError::WriteError(msg) => write!(f, "Write error: {}", msg),
            PersistenceError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            PersistenceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            PersistenceError::FormatError(msg) => write!(f, "Format error: {}", msg),
            PersistenceError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
        }
    }
}

impl std::error::Error for PersistenceError {}

/// Document file format version for compatibility checking
const DOCUMENT_FORMAT_VERSION: u32 = 1;

/// Publisher document serialization format wrapper
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct DocumentFile {
    #[serde(default = "default_format_version")]
    version: u32,
    document: Document,
}

fn default_format_version() -> u32 {
    DOCUMENT_FORMAT_VERSION
}

/// Serialize a document to JSON bytes
pub fn serialize_document(doc: &Document) -> Result<Vec<u8>, PersistenceError> {
    let file = DocumentFile {
        version: DOCUMENT_FORMAT_VERSION,
        document: doc.clone(),
    };

    // Use compact JSON format to minimize file size and serialization overhead
    serde_json::to_vec(&file).map_err(|e| PersistenceError::SerializationError(e.to_string()))
}

/// Deserialize a document from JSON bytes
pub fn deserialize_document(data: &[u8]) -> Result<Document, PersistenceError> {
    // Try to parse the JSON
    let file: DocumentFile = serde_json::from_slice(data)
        .map_err(|e| PersistenceError::ParseError(format!("Invalid JSON structure: {}", e)))?;

    // Check version compatibility
    if file.version > DOCUMENT_FORMAT_VERSION {
        return Err(PersistenceError::FormatError(format!(
            "Unsupported format version: {}. This application supports version {} and earlier.",
            file.version, DOCUMENT_FORMAT_VERSION
        )));
    }

    // Version 1+ documents are supported (forward-compatible with future v1 updates)
    if file.version < 1 {
        return Err(PersistenceError::FormatError(
            "Unsupported format version: 0. Only version 1+ is supported.".to_string(),
        ));
    }

    Ok(file.document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bleed, Margins, Metadata, Pt, Spread, Styles, Unit};

    fn create_test_document() -> Document {
        Document {
            metadata: Metadata {
                name: "Test Document".to_string(),
                author: "Test Author".to_string(),
                description: "A test document".to_string(),
                created_at: 1234567890,
                modified_at: 1234567890,
                dpi: 300,
                default_unit: Unit::Millimeter,
                default_bleed: Bleed {
                    top: Pt(5.0),
                    bottom: Pt(5.0),
                    inside: Pt(5.0),
                    outside: Pt(5.0),
                },
                color_profile: "sRGB".to_string(),
                facing_pages: true,
            },
            fonts: vec![],
            icc_profiles: vec![],
            swatches: vec![],
            styles: Styles::default(),
            spreads: vec![Spread {
                pages: vec![crate::Page {
                    width: Pt(595.0),
                    height: Pt(842.0),
                    margins: Margins {
                        top: Pt(36.0),
                        bottom: Pt(36.0),
                        inside: Pt(36.0),
                        outside: Pt(36.0),
                    },
                    bleed: None,
                    column_count: 2,
                    gutter_width: Pt(12.0),
                    guides: vec![],
                    frames: vec![],
                    applied_parent_id: None,
                }],
            }],
            parent_pages: vec![],
            layers: vec![crate::Layer::new("l1", "Layer 1")],
            baseline_grid: crate::BaselineGrid {
                line_height: Pt(12.0),
                offset: Pt(0.0),
                visible: false,
                color: "#000000".to_string(),
            },
        }
    }

    #[test]
    fn test_serialize_document() {
        let doc = create_test_document();
        let result = serialize_document(&doc);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_deserialize_document() {
        let doc = create_test_document();
        let bytes = serialize_document(&doc).unwrap();
        let result = deserialize_document(&bytes);
        assert!(result.is_ok());
        let deserialized = result.unwrap();
        assert_eq!(deserialized.metadata.name, "Test Document");
    }

    #[test]
    fn test_round_trip() {
        let original = create_test_document();
        let bytes = serialize_document(&original).unwrap();
        let deserialized = deserialize_document(&bytes).unwrap();

        assert_eq!(original.metadata.name, deserialized.metadata.name);
        assert_eq!(original.metadata.author, deserialized.metadata.author);
        assert_eq!(original.spreads.len(), deserialized.spreads.len());
        assert_eq!(original.layers.len(), deserialized.layers.len());
    }

    #[test]
    fn test_invalid_json() {
        let invalid_data = b"not valid json";
        let result = deserialize_document(invalid_data);
        assert!(result.is_err());
        match result {
            Err(PersistenceError::ParseError(_)) => (),
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_malformed_document() {
        let malformed = r#"{"version": 1, "document": {"metadata": null}}"#;
        let result = deserialize_document(malformed.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_future_version() {
        // Use a minimally valid document so we test version compatibility, not parse errors
        let future_version = r##"{
            "version": 99,
            "document": {
                "metadata": {
                    "name": "Test",
                    "author": "",
                    "description": "",
                    "created_at": 0,
                    "modified_at": 0,
                    "dpi": 300,
                    "default_unit": "Millimeter",
                    "default_bleed": {"top": 0.0, "bottom": 0.0, "inside": 0.0, "outside": 0.0},
                    "color_profile": "sRGB",
                    "facing_pages": true
                },
                "fonts": [],
                "icc_profiles": [],
                "swatches": [],
                "styles": {
                    "paragraph_styles": [],
                    "character_styles": [],
                    "object_styles": []
                },
                "spreads": [],
                "parent_pages": [],
                "layers": [],
                "baseline_grid": {
                    "line_height": 12.0,
                    "offset": 0.0,
                    "visible": false,
                    "color": "#000000"
                }
            }
        }"##;
        let result = deserialize_document(future_version.as_bytes());
        assert!(result.is_err());

        // Verify it's a FormatError (version check), not ParseError
        match result {
            Err(PersistenceError::FormatError(msg)) => {
                assert!(msg.contains("99"), "Should mention version 99 in error");
            }
            other => panic!("Expected FormatError, got: {:?}", other),
        }
    }
}

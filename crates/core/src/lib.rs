use automerge::AutoCommit;
use autosurgeon::{hydrate, reconcile, Hydrate, Reconcile};
use serde::{Deserialize, Serialize};

/// Native units in points (pt). 1 pt = 1/72 inch.
pub type Pt = f64;

pub fn init() {
    // Placeholder for core initialization
}

/// A wrapper around an Automerge document that holds the Document state.
pub struct AutomergeDocument {
    doc: AutoCommit,
}

impl AutomergeDocument {
    pub fn new() -> Self {
        Self {
            doc: AutoCommit::new(),
        }
    }

    pub fn load(data: &[u8]) -> Result<Self, automerge::AutomergeError> {
        let doc = AutoCommit::load(data)?;
        Ok(Self { doc })
    }

    pub fn save(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    pub fn merge(&mut self, other: &mut Self) -> Result<(), automerge::AutomergeError> {
        self.doc.merge(&mut other.doc)?;
        Ok(())
    }

    pub fn get_document(&self) -> Result<Document, String> {
        hydrate(&self.doc).map_err(|e| format!("Hydration error: {}", e))
    }

    pub fn set_document(&mut self, doc: &Document) -> Result<(), String> {
        reconcile(&mut self.doc, doc).map_err(|e| format!("Reconciliation error: {}", e))
    }

    pub fn commit(&mut self) {
        self.doc.commit();
    }
}

impl Default for AutomergeDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub struct Document {
    pub metadata: Metadata,
    pub swatches: Vec<ColorSwatch>,
    pub styles: Styles,
    pub spreads: Vec<Spread>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub struct Metadata {
    pub name: String,
    pub author: String,
    pub dpi: u32,
    pub default_bleed: Bleed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub struct Bleed {
    pub top: Pt,
    pub bottom: Pt,
    pub inside: Pt,
    pub outside: Pt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub struct Spread {
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub struct Page {
    pub width: Pt,
    pub height: Pt,
    pub margins: Margins,
    pub frames: Vec<Frame>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub struct Margins {
    pub top: Pt,
    pub bottom: Pt,
    pub inside: Pt,
    pub outside: Pt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub enum Frame {
    Text(TextFrame),
    Image(ImageFrame),
    Shape(ShapeFrame),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub struct TextFrame {
    pub x: Pt,
    pub y: Pt,
    pub width: Pt,
    pub height: Pt,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub struct ImageFrame {
    pub x: Pt,
    pub y: Pt,
    pub width: Pt,
    pub height: Pt,
    pub asset_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub struct ShapeFrame {
    pub x: Pt,
    pub y: Pt,
    pub width: Pt,
    pub height: Pt,
    pub shape_type: ShapeType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Path(String), // SVG path data or similar
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub struct ColorSwatch {
    pub name: String,
    pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub enum Color {
    Rgb { r: f64, g: f64, b: f64 },
    Cmyk { c: f64, m: f64, y: f64, k: f64 },
    Spot { name: String, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub struct Styles {
    pub paragraph_styles: Vec<ParagraphStyle>,
    pub character_styles: Vec<CharacterStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub struct ParagraphStyle {
    pub name: String,
    // Add typography fields later
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Reconcile, Hydrate)]
pub struct CharacterStyle {
    pub name: String,
    // Add typography fields later
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document {
            metadata: Metadata {
                name: "Test Doc".to_string(),
                author: "Author".to_string(),
                dpi: 300,
                default_bleed: Bleed {
                    top: 0.0,
                    bottom: 0.0,
                    inside: 0.0,
                    outside: 0.0,
                },
            },
            swatches: vec![],
            styles: Styles {
                paragraph_styles: vec![],
                character_styles: vec![],
            },
            spreads: vec![],
        };
        assert_eq!(doc.metadata.name, "Test Doc");
    }

    #[test]
    fn test_document_roundtrip() {
        let doc = Document {
            metadata: Metadata {
                name: "Test Doc".to_string(),
                author: "Author".to_string(),
                dpi: 300,
                default_bleed: Bleed {
                    top: 0.0,
                    bottom: 0.0,
                    inside: 0.0,
                    outside: 0.0,
                },
            },
            swatches: vec![],
            styles: Styles {
                paragraph_styles: vec![],
                character_styles: vec![],
            },
            spreads: vec![],
        };

        let mut adoc = AutomergeDocument::new();
        adoc.set_document(&doc).expect("Failed to set document");

        let doc2 = adoc.get_document().expect("Failed to get document");
        assert_eq!(doc, doc2);
    }

    #[test]
    fn test_document_merge() {
        let mut doc1 = Document {
            metadata: Metadata {
                name: "Doc 1".to_string(),
                author: "Author".to_string(),
                dpi: 300,
                default_bleed: Bleed {
                    top: 0.0,
                    bottom: 0.0,
                    inside: 0.0,
                    outside: 0.0,
                },
            },
            swatches: vec![],
            styles: Styles {
                paragraph_styles: vec![],
                character_styles: vec![],
            },
            spreads: vec![],
        };

        let mut adoc1 = AutomergeDocument::new();
        adoc1.set_document(&doc1).expect("Failed to set document");

        let mut adoc2 = AutomergeDocument::load(&adoc1.save()).expect("Failed to load");

        // Change name in adoc2
        doc1.metadata.name = "Doc 2".to_string();
        adoc2.set_document(&doc1).expect("Failed to set document");
        adoc2.commit();

        // Merge back to adoc1
        adoc1.merge(&mut adoc2).expect("Failed to merge");

        let merged_doc = adoc1.get_document().expect("Failed to get document");
        assert_eq!(merged_doc.metadata.name, "Doc 2");
    }

    #[test]
    fn test_concurrent_edits() {
        let doc_initial = Document {
            metadata: Metadata {
                name: "Initial".to_string(),
                author: "Author".to_string(),
                dpi: 300,
                default_bleed: Bleed {
                    top: 0.0,
                    bottom: 0.0,
                    inside: 0.0,
                    outside: 0.0,
                },
            },
            swatches: vec![],
            styles: Styles {
                paragraph_styles: vec![],
                character_styles: vec![],
            },
            spreads: vec![],
        };

        let mut adoc1 = AutomergeDocument::new();
        adoc1.set_document(&doc_initial).expect("Failed to set");
        adoc1.commit();

        let mut adoc2 = AutomergeDocument::load(&adoc1.save()).expect("Failed to load");

        // User 1 changes name
        let mut doc1 = adoc1.get_document().unwrap();
        doc1.metadata.name = "User 1 Change".to_string();
        adoc1.set_document(&doc1).expect("Failed to set");
        adoc1.commit();

        // User 2 changes author concurrently
        let mut doc2 = adoc2.get_document().unwrap();
        doc2.metadata.author = "User 2 Change".to_string();
        adoc2.set_document(&doc2).expect("Failed to set");
        adoc2.commit();

        // Merge adoc2 into adoc1
        adoc1.merge(&mut adoc2).expect("Failed to merge");

        let final_doc = adoc1.get_document().expect("Failed to get");

        // In a true CRDT, both changes should be preserved if they are in different fields
        assert_eq!(final_doc.metadata.name, "User 1 Change");
        assert_eq!(final_doc.metadata.author, "User 2 Change");
    }
}

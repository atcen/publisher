use serde::{Deserialize, Serialize};

/// Native units in points (pt). 1 pt = 1/72 inch.
pub type Pt = f64;

pub fn init() {
    // Placeholder for core initialization
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub metadata: Metadata,
    pub swatches: Vec<ColorSwatch>,
    pub styles: Styles,
    pub spreads: Vec<Spread>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub author: String,
    pub dpi: u32,
    pub default_bleed: Bleed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bleed {
    pub top: Pt,
    pub bottom: Pt,
    pub inside: Pt,
    pub outside: Pt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spread {
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub width: Pt,
    pub height: Pt,
    pub margins: Margins,
    pub frames: Vec<Frame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margins {
    pub top: Pt,
    pub bottom: Pt,
    pub inside: Pt,
    pub outside: Pt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Frame {
    Text(TextFrame),
    Image(ImageFrame),
    Shape(ShapeFrame),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFrame {
    pub x: Pt,
    pub y: Pt,
    pub width: Pt,
    pub height: Pt,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFrame {
    pub x: Pt,
    pub y: Pt,
    pub width: Pt,
    pub height: Pt,
    pub asset_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeFrame {
    pub x: Pt,
    pub y: Pt,
    pub width: Pt,
    pub height: Pt,
    pub shape_type: ShapeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Path(String), // SVG path data or similar
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSwatch {
    pub name: String,
    pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Color {
    Rgb { r: f64, g: f64, b: f64 },
    Cmyk { c: f64, m: f64, y: f64, k: f64 },
    Spot { name: String, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Styles {
    pub paragraph_styles: Vec<ParagraphStyle>,
    pub character_styles: Vec<CharacterStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphStyle {
    pub name: String,
    // Add typography fields later
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

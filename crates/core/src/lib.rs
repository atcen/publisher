pub mod units;
pub mod paper;
pub mod builder;

use serde::{Deserialize, Serialize};
use crate::units::Unit;

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
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub created_at: u64, // Unix timestamp
    #[serde(default)]
    pub modified_at: u64, // Unix timestamp
    pub dpi: u32,
    #[serde(default)]
    pub default_unit: Unit,
    pub default_bleed: Bleed,
    #[serde(default = "default_color_profile")]
    pub color_profile: String,
}

fn default_color_profile() -> String {
    "sRGB".to_string()
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

impl TextFrame {
    pub fn new(x: Pt, y: Pt, width: Pt, height: Pt, content: &str) -> Self {
        Self {
            x,
            y,
            width,
            height,
            content: content.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFrame {
    pub x: Pt,
    pub y: Pt,
    pub width: Pt,
    pub height: Pt,
    pub asset_path: String,
}

impl ImageFrame {
    pub fn new(x: Pt, y: Pt, width: Pt, height: Pt, asset_path: &str) -> Self {
        Self {
            x,
            y,
            width,
            height,
            asset_path: asset_path.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeFrame {
    pub x: Pt,
    pub y: Pt,
    pub width: Pt,
    pub height: Pt,
    pub shape_type: ShapeType,
}

impl ShapeFrame {
    pub fn new(x: Pt, y: Pt, width: Pt, height: Pt, shape_type: ShapeType) -> Self {
        Self {
            x,
            y,
            width,
            height,
            shape_type,
        }
    }
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

impl Default for Indents {
    fn default() -> Self {
        Self {
            left: 0.0,
            right: 0.0,
            first_line: 0.0,
        }
    }
}

impl Default for ParagraphStyle {
    fn default() -> Self {
        Self {
            name: "Basic Paragraph".to_string(),
            based_on: None,
            next_style: None,
            font_family: "Inter".to_string(),
            font_style: "Regular".to_string(),
            font_size: 12.0,
            leading: 14.4,
            tracking: 0.0,
            alignment: TextAlignment::Left,
            indents: Indents::default(),
            space_before: 0.0,
            space_after: 0.0,
            color_swatch: None,
        }
    }
}

impl Color {
    pub fn black() -> Self {
        Color::Cmyk { c: 0.0, m: 0.0, y: 0.0, k: 1.0 }
    }
    
    pub fn white() -> Self {
        Color::Cmyk { c: 0.0, m: 0.0, y: 0.0, k: 0.0 }
    }
    
    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Color::Rgb { r, g, b }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indents {
    pub left: Pt,
    pub right: Pt,
    pub first_line: Pt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphStyle {
    pub name: String,
    pub based_on: Option<String>,
    pub next_style: Option<String>,
    
    // Typographic properties
    pub font_family: String,
    pub font_style: String,
    pub font_size: Pt,
    pub leading: Pt,
    pub tracking: f64,
    pub alignment: TextAlignment,
    pub indents: Indents,
    pub space_before: Pt,
    pub space_after: Pt,
    pub color_swatch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStyle {
    pub name: String,
    pub based_on: Option<String>,

    // Overrides
    pub font_family: Option<String>,
    pub font_style: Option<String>,
    pub font_size: Option<Pt>,
    pub leading: Option<Pt>,
    pub tracking: Option<f64>,
    pub color_swatch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStyle {
    pub name: String,
    pub fill_color: Option<String>,
    pub stroke_color: Option<String>,
    pub stroke_width: Option<Pt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Styles {
    pub paragraph_styles: Vec<ParagraphStyle>,
    pub character_styles: Vec<CharacterStyle>,
    pub object_styles: Vec<ObjectStyle>,
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
                description: "".to_string(),
                created_at: 0,
                modified_at: 0,
                dpi: 300,
                default_unit: Unit::Point,
                default_bleed: Bleed {
                    top: 0.0,
                    bottom: 0.0,
                    inside: 0.0,
                    outside: 0.0,
                },
                color_profile: "sRGB".to_string(),
            },
            swatches: vec![],
            styles: Styles {
                paragraph_styles: vec![],
                character_styles: vec![],
                object_styles: vec![],
            },
            spreads: vec![],
        };
        assert_eq!(doc.metadata.name, "Test Doc");
    }

    #[test]
    fn test_styles_defaults() {
        let style = ParagraphStyle::default();
        assert_eq!(style.name, "Basic Paragraph");
        assert_eq!(style.font_size, 12.0);
        assert_eq!(style.alignment, TextAlignment::Left);
    }

    #[test]
    fn test_color_utilities() {
        let black = Color::black();
        match black {
            Color::Cmyk { k, .. } => assert_eq!(k, 1.0),
            _ => panic!("Expected CMYK color"),
        }
    }

    #[test]
    fn test_frame_creation() {
        let frame = TextFrame::new(10.0, 10.0, 100.0, 50.0, "Hello World");
        assert_eq!(frame.content, "Hello World");
        assert_eq!(frame.width, 100.0);
    }
}

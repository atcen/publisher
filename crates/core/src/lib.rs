pub mod builder;
pub mod document_manager;
pub mod paper;
pub mod persistence;
pub mod units;
pub mod history;
pub mod document_state;

pub use crate::units::{Pt, Unit};
pub use history::{History, Action};
pub use document_state::DocumentState;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            left: Pt(0.0),
            right: Pt(0.0),
            first_line: Pt(0.0),
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
            font_size: Pt(12.0),
            leading: Pt(14.4),
            tracking: 0.0,
            alignment: TextAlignment::Left,
            indents: Indents::default(),
            space_before: Pt(0.0),
            space_after: Pt(0.0),
            color_swatch: None,
        }
    }
}

impl Color {
    pub fn black() -> Self {
        Color::Cmyk {
            c: 0.0,
            m: 0.0,
            y: 0.0,
            k: 1.0,
        }
    }

    pub fn white() -> Self {
        Color::Cmyk {
            c: 0.0,
            m: 0.0,
            y: 0.0,
            k: 0.0,
        }
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
#[serde(default)]
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
    #[serde(default)]
    pub object_styles: Vec<ObjectStyle>,
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            paragraph_styles: vec![ParagraphStyle::default()],
            character_styles: Vec::new(),
            object_styles: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // ===== Document Creation Tests =====
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
                    top: Pt(0.0),
                    bottom: Pt(0.0),
                    inside: Pt(0.0),
                    outside: Pt(0.0),
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
        assert_eq!(doc.swatches.len(), 0);
        assert_eq!(doc.spreads.len(), 0);
    }

    #[test]
    fn test_metadata_mutation() {
        let mut metadata = Metadata {
            name: "Original".to_string(),
            author: "Author".to_string(),
            description: "Description".to_string(),
            created_at: 100,
            modified_at: 100,
            dpi: 300,
            default_unit: Unit::Millimeter,
            default_bleed: Bleed {
                top: Pt(5.0),
                bottom: Pt(5.0),
                inside: Pt(5.0),
                outside: Pt(5.0),
            },
            color_profile: "sRGB".to_string(),
        };

        metadata.name = "Modified".to_string();
        metadata.modified_at = 200;
        metadata.dpi = 600;

        assert_eq!(metadata.name, "Modified");
        assert_eq!(metadata.modified_at, 200);
        assert_eq!(metadata.dpi, 600);
        assert_eq!(metadata.created_at, 100);
    }

    #[test]
    fn test_bleed_creation() {
        let bleed = Bleed {
            top: Pt(10.0),
            bottom: Pt(10.0),
            inside: Pt(15.0),
            outside: Pt(15.0),
        };
        assert_eq!(bleed.top, Pt(10.0));
        assert_eq!(bleed.inside, Pt(15.0));
    }

    // ===== Page and Spread Tests =====
    #[test]
    fn test_page_creation() {
        let page = Page {
            width: Pt(595.0),
            height: Pt(842.0),
            margins: Margins {
                top: Pt(36.0),
                bottom: Pt(36.0),
                inside: Pt(36.0),
                outside: Pt(36.0),
            },
            frames: vec![],
        };
        assert_eq!(page.width, Pt(595.0));
        assert_eq!(page.height, Pt(842.0));
        assert_eq!(page.frames.len(), 0);
    }

    #[test]
    fn test_page_mutation() {
        let mut page = Page {
            width: Pt(595.0),
            height: Pt(842.0),
            margins: Margins {
                top: Pt(36.0),
                bottom: Pt(36.0),
                inside: Pt(36.0),
                outside: Pt(36.0),
            },
            frames: vec![],
        };

        let frame = Frame::Text(TextFrame::new(
            Pt(50.0),
            Pt(50.0),
            Pt(200.0),
            Pt(100.0),
            "Test content",
        ));
        page.frames.push(frame);

        assert_eq!(page.frames.len(), 1);
    }

    #[test]
    fn test_spread_creation() {
        let page1 = Page {
            width: Pt(595.0),
            height: Pt(842.0),
            margins: Margins {
                top: Pt(36.0),
                bottom: Pt(36.0),
                inside: Pt(36.0),
                outside: Pt(36.0),
            },
            frames: vec![],
        };
        let page2 = page1.clone();

        let spread = Spread {
            pages: vec![page1, page2],
        };
        assert_eq!(spread.pages.len(), 2);
    }

    // ===== Frame Tests =====
    #[test]
    fn test_text_frame_creation() {
        let frame = TextFrame::new(Pt(10.0), Pt(10.0), Pt(100.0), Pt(50.0), "Hello World");
        assert_eq!(frame.x, Pt(10.0));
        assert_eq!(frame.y, Pt(10.0));
        assert_eq!(frame.width, Pt(100.0));
        assert_eq!(frame.height, Pt(50.0));
        assert_eq!(frame.content, "Hello World");
    }

    #[test]
    fn test_text_frame_mutation() {
        let mut frame = TextFrame::new(Pt(10.0), Pt(10.0), Pt(100.0), Pt(50.0), "Hello");
        frame.content = "World".to_string();
        frame.x = Pt(20.0);
        frame.y = Pt(30.0);

        assert_eq!(frame.content, "World");
        assert_eq!(frame.x, Pt(20.0));
        assert_eq!(frame.y, Pt(30.0));
    }

    #[test]
    fn test_image_frame_creation() {
        let frame = ImageFrame::new(
            Pt(10.0),
            Pt(10.0),
            Pt(200.0),
            Pt(150.0),
            "/path/to/image.jpg",
        );
        assert_eq!(frame.x, Pt(10.0));
        assert_eq!(frame.width, Pt(200.0));
        assert_eq!(frame.height, Pt(150.0));
        assert_eq!(frame.asset_path, "/path/to/image.jpg");
    }

    #[test]
    fn test_image_frame_mutation() {
        let mut frame = ImageFrame::new(
            Pt(10.0),
            Pt(10.0),
            Pt(200.0),
            Pt(150.0),
            "/path/to/image.jpg",
        );
        frame.asset_path = "/new/path/image.png".to_string();
        frame.width = Pt(300.0);

        assert_eq!(frame.asset_path, "/new/path/image.png");
        assert_eq!(frame.width, Pt(300.0));
    }

    #[test]
    fn test_shape_frame_creation() {
        let frame = ShapeFrame::new(
            Pt(50.0),
            Pt(50.0),
            Pt(100.0),
            Pt(100.0),
            ShapeType::Rectangle,
        );
        assert_eq!(frame.x, Pt(50.0));
        assert_eq!(frame.shape_type, ShapeType::Rectangle);
    }

    #[test]
    fn test_shape_frame_ellipse() {
        let frame = ShapeFrame::new(Pt(0.0), Pt(0.0), Pt(50.0), Pt(75.0), ShapeType::Ellipse);
        assert_eq!(frame.width, Pt(50.0));
        assert_eq!(frame.height, Pt(75.0));
    }

    #[test]
    fn test_shape_frame_path() {
        let path_data = "M10 10 L90 90".to_string();
        let frame = ShapeFrame::new(
            Pt(0.0),
            Pt(0.0),
            Pt(100.0),
            Pt(100.0),
            ShapeType::Path(path_data.clone()),
        );
        if let ShapeType::Path(p) = frame.shape_type {
            assert_eq!(p, path_data);
        } else {
            panic!("Expected Path shape type");
        }
    }

    // ===== Style Tests =====
    #[test]
    fn test_styles_defaults() {
        let style = ParagraphStyle::default();
        assert_eq!(style.name, "Basic Paragraph");
        assert_eq!(style.font_size, Pt(12.0));
        assert_eq!(style.alignment, TextAlignment::Left);
        assert_eq!(style.font_family, "Inter");
        assert_eq!(style.leading, Pt(14.4));
        assert_eq!(style.tracking, 0.0);
    }

    #[test]
    fn test_paragraph_style_mutation() {
        let mut style = ParagraphStyle::default();
        style.font_size = Pt(16.0);
        style.leading = Pt(19.2);
        style.alignment = TextAlignment::Center;

        assert_eq!(style.font_size, Pt(16.0));
        assert_eq!(style.alignment, TextAlignment::Center);
    }

    #[test]
    fn test_character_style_creation() {
        let style = CharacterStyle {
            name: "Emphasis".to_string(),
            based_on: Some("Basic".to_string()),
            font_family: Some("Georgia".to_string()),
            font_style: Some("Italic".to_string()),
            font_size: Some(Pt(14.0)),
            leading: None,
            tracking: Some(0.5),
            color_swatch: Some("Red".to_string()),
        };
        assert_eq!(style.name, "Emphasis");
        assert_eq!(style.font_size, Some(Pt(14.0)));
    }

    #[test]
    fn test_object_style_creation() {
        let style = ObjectStyle {
            name: "Shadow Box".to_string(),
            fill_color: Some("Black".to_string()),
            stroke_color: Some("White".to_string()),
            stroke_width: Some(Pt(2.0)),
        };
        assert_eq!(style.name, "Shadow Box");
        assert_eq!(style.stroke_width, Some(Pt(2.0)));
    }

    #[test]
    fn test_indents_creation() {
        let indents = Indents {
            left: Pt(36.0),
            right: Pt(36.0),
            first_line: Pt(18.0),
        };
        assert_eq!(indents.left, Pt(36.0));
        assert_eq!(indents.first_line, Pt(18.0));
    }

    #[test]
    fn test_indents_default() {
        let indents = Indents::default();
        assert_eq!(indents.left, Pt(0.0));
        assert_eq!(indents.right, Pt(0.0));
        assert_eq!(indents.first_line, Pt(0.0));
    }

    // ===== Color Tests =====
    #[test]
    fn test_color_utilities() {
        let black = Color::black();
        match black {
            Color::Cmyk { c, m, y, k } => {
                assert_eq!(c, 0.0);
                assert_eq!(m, 0.0);
                assert_eq!(y, 0.0);
                assert_eq!(k, 1.0);
            }
            _ => panic!("Expected CMYK color"),
        }
    }

    #[test]
    fn test_color_white() {
        let white = Color::white();
        match white {
            Color::Cmyk { k, .. } => assert_eq!(k, 0.0),
            _ => panic!("Expected CMYK color"),
        }
    }

    #[test]
    fn test_color_rgb() {
        let red = Color::rgb(1.0, 0.0, 0.0);
        match red {
            Color::Rgb { r, g, b } => {
                assert_eq!(r, 1.0);
                assert_eq!(g, 0.0);
                assert_eq!(b, 0.0);
            }
            _ => panic!("Expected RGB color"),
        }
    }

    #[test]
    fn test_color_swatch_creation() {
        let swatch = ColorSwatch {
            name: "Primary Blue".to_string(),
            color: Color::rgb(0.0, 0.5, 1.0),
        };
        assert_eq!(swatch.name, "Primary Blue");
    }

    #[test]
    fn test_frame_creation() {
        let frame = TextFrame::new(Pt(10.0), Pt(10.0), Pt(100.0), Pt(50.0), "Hello World");
        assert_eq!(frame.content, "Hello World");
        assert_eq!(frame.width, Pt(100.0));
    }

    // ===== JSON Serialization Tests =====
    #[test]
    fn test_document_serialize_deserialize() {
        let doc = Document {
            metadata: Metadata {
                name: "Serialization Test".to_string(),
                author: "Test Author".to_string(),
                description: "Testing serialization".to_string(),
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
            },
            swatches: vec![ColorSwatch {
                name: "Black".to_string(),
                color: Color::black(),
            }],
            styles: Styles::default(),
            spreads: vec![],
        };

        let json = serde_json::to_string(&doc).expect("Serialization failed");
        let deserialized: Document = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(doc.metadata.name, deserialized.metadata.name);
        assert_eq!(doc.metadata.author, deserialized.metadata.author);
        assert_eq!(doc.swatches.len(), deserialized.swatches.len());
    }

    #[test]
    fn test_frame_serialize_deserialize() {
        let frame = Frame::Text(TextFrame::new(
            Pt(10.0),
            Pt(20.0),
            Pt(100.0),
            Pt(50.0),
            "Test content",
        ));
        let json = serde_json::to_string(&frame).expect("Serialization failed");
        let deserialized: Frame = serde_json::from_str(&json).expect("Deserialization failed");

        match deserialized {
            Frame::Text(f) => {
                assert_eq!(f.x, Pt(10.0));
                assert_eq!(f.content, "Test content");
            }
            _ => panic!("Expected Text frame"),
        }
    }

    #[test]
    fn test_paragraph_style_serialize_deserialize() {
        let style = ParagraphStyle {
            name: "Test Style".to_string(),
            based_on: Some("Basic".to_string()),
            next_style: None,
            font_family: "Arial".to_string(),
            font_style: "Bold".to_string(),
            font_size: Pt(16.0),
            leading: Pt(19.2),
            tracking: 0.1,
            alignment: TextAlignment::Justify,
            indents: Indents {
                left: Pt(36.0),
                right: Pt(36.0),
                first_line: Pt(18.0),
            },
            space_before: Pt(12.0),
            space_after: Pt(12.0),
            color_swatch: Some("Black".to_string()),
        };

        let json = serde_json::to_string(&style).expect("Serialization failed");
        let deserialized: ParagraphStyle =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(style.name, deserialized.name);
        assert_eq!(style.font_size, deserialized.font_size);
        assert_eq!(style.alignment, deserialized.alignment);
        assert_eq!(style.based_on, deserialized.based_on);
    }

    #[test]
    fn test_color_serialize_deserialize() {
        let cmyk = Color::Cmyk {
            c: 0.5,
            m: 0.3,
            y: 0.1,
            k: 0.0,
        };
        let json = serde_json::to_string(&cmyk).expect("Serialization failed");
        let deserialized: Color = serde_json::from_str(&json).expect("Deserialization failed");

        match deserialized {
            Color::Cmyk { c, m, y, k } => {
                assert_eq!(c, 0.5);
                assert_eq!(m, 0.3);
                assert_eq!(y, 0.1);
                assert_eq!(k, 0.0);
            }
            _ => panic!("Expected CMYK color"),
        }
    }

    #[test]
    fn test_styles_serialize_deserialize() {
        let styles = Styles {
            paragraph_styles: vec![ParagraphStyle::default()],
            character_styles: vec![CharacterStyle {
                name: "Emphasis".to_string(),
                based_on: None,
                font_family: Some("Georgia".to_string()),
                font_style: Some("Italic".to_string()),
                font_size: Some(Pt(14.0)),
                leading: None,
                tracking: None,
                color_swatch: None,
            }],
            object_styles: vec![],
        };

        let json = serde_json::to_string(&styles).expect("Serialization failed");
        let deserialized: Styles = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(
            styles.paragraph_styles.len(),
            deserialized.paragraph_styles.len()
        );
        assert_eq!(
            styles.character_styles.len(),
            deserialized.character_styles.len()
        );
    }

    #[test]
    fn test_page_serialize_deserialize() {
        let page = Page {
            width: Pt(595.0),
            height: Pt(842.0),
            margins: Margins {
                top: Pt(36.0),
                bottom: Pt(36.0),
                inside: Pt(36.0),
                outside: Pt(36.0),
            },
            frames: vec![
                Frame::Text(TextFrame::new(
                    Pt(50.0),
                    Pt(50.0),
                    Pt(200.0),
                    Pt(100.0),
                    "Hello",
                )),
                Frame::Shape(ShapeFrame::new(
                    Pt(300.0),
                    Pt(300.0),
                    Pt(50.0),
                    Pt(50.0),
                    ShapeType::Rectangle,
                )),
            ],
        };

        let json = serde_json::to_string(&page).expect("Serialization failed");
        let deserialized: Page = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(page.width, deserialized.width);
        assert_eq!(page.frames.len(), deserialized.frames.len());
    }
}

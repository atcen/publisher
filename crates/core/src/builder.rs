use crate::{Document, Metadata, Bleed, Styles, Spread, Page, Margins};
use crate::units::Unit;
use crate::paper::PaperFormat;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DocumentBuilder {
    name: String,
    author: String,
    description: String,
    dpi: u32,
    unit: Unit,
    format: PaperFormat,
    facing_pages: bool,
    pages_count: u32,
    margins: Margins,
    bleed: Bleed,
    color_profile: String,
}

impl DocumentBuilder {
    pub fn new() -> Self {
        Self {
            name: "Untitled".to_string(),
            author: "".to_string(),
            description: "".to_string(),
            dpi: 300,
            unit: Unit::Millimeter,
            format: PaperFormat::A4,
            facing_pages: true,
            pages_count: 1,
            margins: Margins {
                top: Unit::Millimeter.to_points(12.7, None),
                bottom: Unit::Millimeter.to_points(12.7, None),
                inside: Unit::Millimeter.to_points(12.7, None),
                outside: Unit::Millimeter.to_points(12.7, None),
            },
            bleed: Bleed {
                top: 0.0,
                bottom: 0.0,
                inside: 0.0,
                outside: 0.0,
            },
            color_profile: "sRGB".to_string(),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_format(mut self, format: PaperFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_pages(mut self, count: u32) -> Self {
        self.pages_count = count;
        self
    }

    pub fn with_facing_pages(mut self, facing: bool) -> Self {
        self.facing_pages = facing;
        self
    }

    pub fn build(self) -> Document {
        let (width, height) = self.format.dimensions_pt();
        
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(_) => {
                // If the system clock is earlier than the Unix epoch, clamp the
                // document timestamps to 0 rather than silently relying on a
                // default duration.
                0
            }
        };

        let mut spreads = Vec::new();
        let mut current_pages = Vec::new();

        for i in 0..self.pages_count {
            let page = Page {
                width,
                height,
                margins: self.margins.clone(),
                frames: Vec::new(),
            };

            if self.facing_pages {
                if i == 0 {
                    spreads.push(Spread { pages: vec![page] });
                } else {
                    current_pages.push(page);
                    if current_pages.len() == 2 || i == self.pages_count - 1 {
                        spreads.push(Spread { pages: current_pages });
                        current_pages = Vec::new();
                    }
                }
            } else {
                spreads.push(Spread { pages: vec![page] });
            }
        }

        Document {
            metadata: Metadata {
                name: self.name,
                author: self.author,
                description: self.description,
                created_at: now,
                modified_at: now,
                dpi: self.dpi,
                default_unit: self.unit,
                default_bleed: self.bleed,
                color_profile: self.color_profile,
            },
            swatches: Vec::new(),
            styles: Styles {
                paragraph_styles: Vec::new(),
                character_styles: Vec::new(),
            },
            spreads,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let doc = DocumentBuilder::new()
            .with_name("My Magazine")
            .with_format(PaperFormat::A4)
            .with_pages(3)
            .with_facing_pages(true)
            .build();
            
        assert_eq!(doc.metadata.name, "My Magazine");
        assert_eq!(doc.spreads.len(), 2);
        assert_eq!(doc.spreads[0].pages.len(), 1);
        assert_eq!(doc.spreads[1].pages.len(), 2);
        assert_eq!(doc.metadata.created_at, doc.metadata.modified_at);
    }
}

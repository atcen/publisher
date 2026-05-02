use crate::paper::PaperFormat;
use crate::units::Unit;
use crate::{Bleed, Color, ColorSwatch, Document, Margins, Metadata, Page, Spread, Styles};
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
    column_count: u32,
    gutter_width: crate::Pt,
    color_profile: String,
}

impl Default for DocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
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
                top: crate::Pt(0.0),
                bottom: crate::Pt(0.0),
                inside: crate::Pt(0.0),
                outside: crate::Pt(0.0),
            },
            column_count: 1,
            gutter_width: crate::Pt(12.0),
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
        self.pages_count = if count == 0 { 1 } else { count };
        self
    }

    pub fn with_facing_pages(mut self, facing: bool) -> Self {
        self.facing_pages = facing;
        self
    }

    pub fn with_columns(mut self, count: u32) -> Self {
        self.column_count = count.max(1);
        self
    }

    pub fn with_gutter(mut self, width: crate::Pt) -> Self {
        self.gutter_width = width;
        self
    }

    pub fn build(self) -> Document {
        if self.pages_count == 0 {
            // Should not happen with current with_pages, but for safety:
            return self.with_pages(1).build();
        }
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
        for _ in 0..self.pages_count {
            spreads.push(Spread {
                pages: vec![Page {
                    width,
                    height,
                    margins: self.margins.clone(),
                    bleed: None,
                    column_count: self.column_count,
                    gutter_width: self.gutter_width,
                    guides: Vec::new(),
                    frames: Vec::new(),
                    applied_parent_id: None,
                }],
            });
        }

        let swatches = vec![
            ColorSwatch {
                name: "Black".to_string(),
                color: Color::black(),
            },
            ColorSwatch {
                name: "Paper".to_string(),
                color: Color::white(),
            },
        ];

        let layers = vec![crate::Layer::new("layer-1", "Layer 1")];

        let mut doc = Document {
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
                facing_pages: self.facing_pages,
            },
            fonts: vec![],
            icc_profiles: vec![],
            swatches,
            styles: Styles::default(),
            spreads,
            parent_pages: vec![],
            layers,
            baseline_grid: crate::BaselineGrid::default(),
        };

        doc.reorganize_spreads();
        doc
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

    #[test]
    fn test_builder_zero_pages() {
        let doc = DocumentBuilder::new().with_pages(0).build();
        assert_eq!(doc.spreads.len(), 1);
        assert_eq!(doc.spreads[0].pages.len(), 1);
    }

    #[test]
    fn test_builder_columns() {
        let doc = DocumentBuilder::new()
            .with_columns(3)
            .with_gutter(crate::Pt(10.0))
            .build();

        let page = &doc.spreads[0].pages[0];
        assert_eq!(page.column_count, 3);
        assert_eq!(page.gutter_width, crate::Pt(10.0));
    }
}

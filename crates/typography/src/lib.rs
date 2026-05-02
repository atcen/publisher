use publisher_core::{Pt, TextAlignment};
use serde::{Deserialize, Serialize};

pub fn init() {
    publisher_core::init();
    println!("Publisher Typography Initialized");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    pub glyph_id: u32,
    pub x_offset: f64,
    pub y_offset: f64,
    pub x_advance: f64,
    pub y_advance: f64,
    pub cluster: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapedText {
    pub glyphs: Vec<Glyph>,
    pub font_size: Pt,
    pub line_height: Pt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub tag: String, // 4-character tag like "liga", "smcp"
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variation {
    pub tag: String, // 4-character tag like "wght", "wdth"
    pub value: f32,
}

pub struct TypographyEngine {
    ft_library: freetype::Library,
}

impl TypographyEngine {
    pub fn new() -> Self {
        Self {
            ft_library: freetype::Library::init().expect("Failed to initialize FreeType"),
        }
    }

    pub fn shape_text(
        &self,
        text: &str,
        font_data: &[u8],
        font_size: Pt,
        _alignment: TextAlignment,
        features: &[Feature],
        variations: &[Variation],
    ) -> Result<ShapedText, String> {
        // 1. Create HarfBuzz face and font
        let hb_face = harfbuzz_rs::Face::from_bytes(font_data, 0);
        let mut hb_font = harfbuzz_rs::Font::new(hb_face);

        // 2. Set font scale based on Pt (assuming 72 DPI for simplicity in core)
        hb_font.set_scale(
            (font_size.0 * 64.0) as i32,
            (font_size.0 * 64.0) as i32,
        );

        // 3. Apply Variations (for Variable Fonts)
        let hb_variations: Vec<harfbuzz_rs::Variation> = variations
            .iter()
            .map(|v| harfbuzz_rs::Variation::new(&v.tag, v.value))
            .collect();
        hb_font.set_variations(&hb_variations);

        // 4. Create HarfBuzz buffer and shape
        let buffer = harfbuzz_rs::UnicodeBuffer::new().add_str(text);
        
        // Convert our features to HarfBuzz features
        let hb_features: Vec<harfbuzz_rs::Feature> = features
            .iter()
            .map(|f| harfbuzz_rs::Feature::new(&f.tag, f.value, 0..text.len()))
            .collect();

        let output = harfbuzz_rs::shape(&hb_font, buffer, &hb_features);

        // 5. Extract glyph positions
        let positions = output.get_glyph_positions();
        let infos = output.get_glyph_infos();

        let glyphs = infos
            .iter()
            .zip(positions.iter())
            .map(|(info, pos)| Glyph {
                glyph_id: info.codepoint,
                x_offset: pos.x_offset as f64 / 64.0,
                y_offset: pos.y_offset as f64 / 64.0,
                x_advance: pos.x_advance as f64 / 64.0,
                y_advance: pos.y_advance as f64 / 64.0,
                cluster: info.cluster,
            })
            .collect();

        Ok(ShapedText {
            glyphs,
            font_size,
            line_height: Pt(font_size.0 * 1.2), // Default 1.2x leading
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typography_init() {
        init();
    }

    // Note: To test shape_text, we would need a valid font file.
    // In a real environment, we'd bundle a small test font (like Roboto or Inter).
}

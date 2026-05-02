pub mod builder;
pub mod document_manager;
pub mod document_state;
pub mod history;
pub mod paper;
pub mod persistence;
pub mod units;

pub use crate::units::{Pt, Unit};
pub use document_state::DocumentState;
pub use history::{Action, History};
use serde::{Deserialize, Serialize};

pub fn init() {
    // Placeholder for core initialization
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub metadata: Metadata,
    pub fonts: Vec<FontResource>,
    pub icc_profiles: Vec<Vec<u8>>,
    pub swatches: Vec<ColorSwatch>,
    pub styles: Styles,
    pub spreads: Vec<Spread>,
    pub parent_pages: Vec<ParentPage>,
    pub layers: Vec<Layer>,
    pub baseline_grid: BaselineGrid,
}

impl Document {
    pub fn all_pages(&self) -> Vec<Page> {
        self.spreads.iter().flat_map(|s| s.pages.clone()).collect()
    }

    pub fn reorganize_spreads(&mut self) {
        let pages = self.all_pages();
        let mut new_spreads = Vec::new();
        let mut current_pages = Vec::new();

        for (i, page) in pages.into_iter().enumerate() {
            if self.metadata.facing_pages {
                if i == 0 {
                    new_spreads.push(Spread { pages: vec![page] });
                } else {
                    current_pages.push(page);
                    if current_pages.len() == 2 {
                        new_spreads.push(Spread {
                            pages: current_pages,
                        });
                        current_pages = Vec::new();
                    }
                }
            } else {
                new_spreads.push(Spread { pages: vec![page] });
            }
        }

        if !current_pages.is_empty() {
            new_spreads.push(Spread {
                pages: current_pages,
            });
        }

        self.spreads = new_spreads;
    }

    pub fn add_page(&mut self, after_index: usize, page: Page) {
        let mut pages = self.all_pages();
        if after_index >= pages.len() {
            pages.push(page);
        } else {
            pages.insert(after_index + 1, page);
        }
        self.set_pages(pages);
    }

    pub fn remove_page(&mut self, index: usize) {
        let mut pages = self.all_pages();
        if index < pages.len() {
            pages.remove(index);
            self.set_pages(pages);
        }
    }

    pub fn set_pages(&mut self, pages: Vec<Page>) {
        self.spreads = vec![];
        self.spreads = pages.into_iter().map(|p| Spread { pages: vec![p] }).collect();
        self.reorganize_spreads();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentPage {
    pub id: String,
    pub name: String,
    pub spread: Spread,
    pub based_on_id: Option<String>,
}

impl ParentPage {
    pub fn new(id: &str, name: &str, spread: Spread) -> Self {
        Self { id: id.to_string(), name: name.to_string(), spread, based_on_id: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineGrid {
    pub line_height: Pt,
    pub offset: Pt,
    pub visible: bool,
    pub color: String,
}

impl Default for BaselineGrid {
    fn default() -> Self {
        Self {
            line_height: Pt(12.0),
            offset: Pt(0.0),
            visible: false,
            color: "#44ffff33".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontResource {
    pub id: String,
    pub name: String,
    pub family: String,
    pub style: String,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    #[serde(default)]
    pub variation_axes: Vec<FontVariationAxis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontVariationAxis {
    pub tag: String,
    pub name: String,
    pub min_value: f64,
    pub max_value: f64,
    pub default_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontVariationSetting {
    pub tag: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub color: String,
}

impl Layer {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            visible: true,
            locked: false,
            color: "#007acc".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub modified_at: u64,
    pub dpi: u32,
    #[serde(default)]
    pub default_unit: Unit,
    pub default_bleed: Bleed,
    #[serde(default = "default_color_profile")]
    pub color_profile: String,
    #[serde(default)]
    pub facing_pages: bool,
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
    pub bleed: Option<Bleed>,
    pub column_count: u32,
    pub gutter_width: Pt,
    pub guides: Vec<Guide>,
    pub frames: Vec<Frame>,
    pub applied_parent_id: Option<String>,
}

impl Page {
    pub fn new(width: Pt, height: Pt, margins: Margins) -> Self {
        Self {
            width,
            height,
            margins,
            bleed: None,
            column_count: 1,
            gutter_width: Pt(12.0),
            guides: Vec::new(),
            frames: Vec::new(),
            applied_parent_id: None,
        }
    }

    pub fn column_width(&self) -> Pt {
        if self.column_count <= 1 {
            return Pt(self.width.0 - self.margins.inside.0 - self.margins.outside.0);
        }
        let total_gutter = self.gutter_width.0 * (self.column_count - 1) as f64;
        let total_margin = self.margins.inside.0 + self.margins.outside.0;
        let available_width = self.width.0 - total_margin - total_gutter;
        Pt(available_width / self.column_count as f64)
    }

    pub fn apply_grid_preset(&mut self, preset: GridPreset) {
        match preset {
            GridPreset::TwelveColumn => {
                self.column_count = 12;
                self.gutter_width = Pt(12.0);
            }
            GridPreset::EightColumn => {
                self.column_count = 8;
                self.gutter_width = Pt(12.0);
            }
            GridPreset::GoldenRatio => {
                let phi = 1.61803398875;
                let margin = self.width.0 / (phi + 1.0);
                self.margins = Margins {
                    top: Pt(margin / phi),
                    bottom: Pt(margin),
                    inside: Pt(margin / phi),
                    outside: Pt(margin),
                };
                self.column_count = 2;
                self.gutter_width = Pt(12.0);
            }
            GridPreset::Fibonacci => {
                self.column_count = 5;
                self.gutter_width = Pt(8.0);
            }
            GridPreset::Manuscript => {
                self.column_count = 1;
                let m = self.width.0 * 0.15;
                self.margins = Margins {
                    top: Pt(m),
                    bottom: Pt(m * 1.5),
                    inside: Pt(m),
                    outside: Pt(m * 1.2),
                };
            }
        }
    }

    pub fn snap_targets(&self) -> Vec<SnapTarget> {
        let mut targets = Vec::new();
        targets.push(SnapTarget::Margin { position: self.margins.inside, side: Side::Left });
        targets.push(SnapTarget::Margin { position: Pt(self.width.0 - self.margins.outside.0), side: Side::Right });
        targets.push(SnapTarget::Margin { position: self.margins.top, side: Side::Top });
        targets.push(SnapTarget::Margin { position: Pt(self.height.0 - self.margins.bottom.0), side: Side::Bottom });

        let col_w = self.column_width().0;
        let gutter = self.gutter_width.0;
        let start_x = self.margins.inside.0;
        for i in 0..self.column_count {
            let x_left = start_x + i as f64 * (col_w + gutter);
            let x_right = x_left + col_w;
            targets.push(SnapTarget::Column { position: Pt(x_left), index: i, is_right: false });
            targets.push(SnapTarget::Column { position: Pt(x_right), index: i, is_right: true });
        }
        for guide in &self.guides {
            targets.push(SnapTarget::Guide { position: guide.position, orientation: guide.orientation });
        }
        targets
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GridPreset {
    TwelveColumn,
    EightColumn,
    GoldenRatio,
    Fibonacci,
    Manuscript,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guide {
    pub position: Pt,
    pub orientation: Orientation,
    pub locked: bool,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnapTarget {
    Margin { position: Pt, side: Side },
    Column { position: Pt, index: u32, is_right: bool },
    Guide { position: Pt, orientation: Orientation },
    Object { position: Pt, orientation: Orientation, frame_id: String },
    Baseline { position: Pt },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side { Top, Bottom, Left, Right }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation { Horizontal, Vertical }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margins {
    pub top: Pt,
    pub bottom: Pt,
    pub inside: Pt,
    pub outside: Pt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub id: String,
    pub layer_id: String,
    pub x: Pt,
    pub y: Pt,
    pub width: Pt,
    pub height: Pt,
    pub rotation: f64,
    pub fill_color: Option<String>,
    pub stroke_color: Option<String>,
    pub stroke_width: Pt,
    pub data: FrameData,
}

impl Frame {
    pub fn new(id: &str, layer_id: &str, x: Pt, y: Pt, width: Pt, height: Pt, data: FrameData) -> Self {
        Self { id: id.to_string(), layer_id: layer_id.to_string(), x, y, width, height, rotation: 0.0, fill_color: None, stroke_color: None, stroke_width: Pt(0.0), data }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrameData {
    Text(TextFrame),
    Image(ImageFrame),
    Shape(ShapeFrame),
    Group(Group),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group { pub frames: Vec<Frame> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFrame {
    pub content: String,
    pub paragraph_style: Option<String>,
    pub next_frame_id: Option<String>,
    pub prev_frame_id: Option<String>,
    pub align_to_baseline_grid: bool,
}

impl TextFrame {
    pub fn new(content: &str) -> Self {
        Self { content: content.to_string(), paragraph_style: None, next_frame_id: None, prev_frame_id: None, align_to_baseline_grid: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFrame {
    pub asset_path: String,
    pub content_x: Pt,
    pub content_y: Pt,
    pub content_scale_x: f64,
    pub content_scale_y: f64,
    pub fitting: ImageFitting,
}

impl ImageFrame {
    pub fn new(asset_path: &str) -> Self {
        Self { asset_path: asset_path.to_string(), content_x: Pt(0.0), content_y: Pt(0.0), content_scale_x: 1.0, content_scale_y: 1.0, fitting: ImageFitting::Fit }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ImageFitting { Fill, Fit, Stretch, Original, Custom }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeFrame { pub shape_type: ShapeType }
impl ShapeFrame { pub fn new(shape_type: ShapeType) -> Self { Self { shape_type } } }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeType { Rectangle, Ellipse, Path(String) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSwatch {
    pub name: String,
    pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Color {
    Rgb { r: f64, g: f64, b: f64 },
    Cmyk { c: f64, m: f64, y: f64, k: f64 },
    Spot { name: String, alternate_cmyk: (f64, f64, f64, f64), tint: f64 },
}

impl Color {
    pub fn black() -> Self { Color::Cmyk { c: 0.0, m: 0.0, y: 0.0, k: 1.0 } }
    pub fn white() -> Self { Color::Cmyk { c: 0.0, m: 0.0, y: 0.0, k: 0.0 } }
    pub fn rgb(r: f64, g: f64, b: f64) -> Self { Color::Rgb { r, g, b } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KerningMode { Metric, Optical, None }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlignMode { Left, Center, Right, Top, Middle, Bottom }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributeMode { HorizontalSpacing, VerticalSpacing }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlignment { Left, Center, Right, Justify }

pub fn align_frames(frames: &mut [Frame], mode: AlignMode) {
    if frames.is_empty() { return; }
    match mode {
        AlignMode::Left => { let min_x = frames.iter().map(|f| f.x.0).fold(f64::INFINITY, f64::min); for f in frames { f.x = Pt(min_x); } }
        AlignMode::Right => { let max_right = frames.iter().map(|f| f.x.0 + f.width.0).fold(f64::NEG_INFINITY, f64::max); for f in frames { f.x = Pt(max_right - f.width.0); } }
        AlignMode::Center => { let min_x = frames.iter().map(|f| f.x.0).fold(f64::INFINITY, f64::min); let max_x = frames.iter().map(|f| f.x.0 + f.width.0).fold(f64::NEG_INFINITY, f64::max); let center_x = (min_x + max_x) / 2.0; for f in frames { f.x = Pt(center_x - f.width.0 / 2.0); } }
        AlignMode::Top => { let min_y = frames.iter().map(|f| f.y.0).fold(f64::INFINITY, f64::min); for f in frames { f.y = Pt(min_y); } }
        AlignMode::Bottom => { let max_bottom = frames.iter().map(|f| f.y.0 + f.height.0).fold(f64::NEG_INFINITY, f64::max); for f in frames { f.y = Pt(max_bottom - f.height.0); } }
        AlignMode::Middle => { let min_y = frames.iter().map(|f| f.y.0).fold(f64::INFINITY, f64::min); let max_y = frames.iter().map(|f| f.y.0 + f.height.0).fold(f64::NEG_INFINITY, f64::max); let middle_y = (min_y + max_y) / 2.0; for f in frames { f.y = Pt(middle_y - f.height.0 / 2.0); } }
    }
}

pub fn distribute_frames(frames: &mut [Frame], mode: DistributeMode) {
    if frames.len() < 3 { return; }
    match mode {
        DistributeMode::HorizontalSpacing => { frames.sort_by(|a, b| a.x.0.partial_cmp(&b.x.0).unwrap()); let min_x = frames[0].x.0; let last_idx = frames.len() - 1; let max_x = frames[last_idx].x.0; let total_width: f64 = frames.iter().map(|f| f.width.0).sum(); let available_gap = (max_x + frames[last_idx].width.0) - min_x - total_width; let gap = available_gap / (frames.len() - 1) as f64; let mut current_x = min_x; for f in frames { f.x = Pt(current_x); current_x += f.width.0 + gap; } }
        DistributeMode::VerticalSpacing => { frames.sort_by(|a, b| a.y.0.partial_cmp(&b.y.0).unwrap()); let min_y = frames[0].y.0; let last_idx = frames.len() - 1; let max_y = frames[last_idx].y.0; let total_height: f64 = frames.iter().map(|f| f.width.0).sum(); let available_gap = (max_y + frames[last_idx].height.0) - min_y - total_height; let gap = available_gap / (frames.len() - 1) as f64; let mut current_y = min_y; for f in frames { f.y = Pt(current_y); current_y += f.height.0 + gap; } }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapResult { pub x: Option<SnapPoint>, pub y: Option<SnapPoint> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapPoint { pub position: Pt, pub target: SnapTarget }
pub struct SnapEngine { pub threshold: f64 }
impl SnapEngine {
    pub fn new(threshold: f64) -> Self { Self { threshold } }
    pub fn find_snap(&self, x: f64, y: f64, width: f64, height: f64, targets: &[SnapTarget]) -> SnapResult {
        let mut best_x = None; let mut best_y = None; let mut min_dx = self.threshold; let mut min_dy = self.threshold;
        let edges_x = [x, x + width / 2.0, x + width]; let edges_y = [y, y + height / 2.0, y + height];
        for target in targets {
            match target {
                SnapTarget::Margin { position, side } | SnapTarget::Column { position, .. } | SnapTarget::Guide { position, orientation: Orientation::Vertical, .. } | SnapTarget::Object { position, orientation: Orientation::Vertical, .. } => {
                    if matches!(target, SnapTarget::Margin { side, .. } if *side == Side::Top || *side == Side::Bottom) { continue; }
                    for &ex in &edges_x { let dx = (ex - position.0).abs(); if dx < min_dx { min_dx = dx; best_x = Some(SnapPoint { position: Pt(position.0 - (ex - x)), target: target.clone() }); } }
                }
                SnapTarget::Margin { position, side } | SnapTarget::Guide { position, orientation: Orientation::Horizontal, .. } | SnapTarget::Object { position, orientation: Orientation::Horizontal, .. } | SnapTarget::Baseline { position } => {
                     if matches!(target, SnapTarget::Margin { side, .. } if *side == Side::Left || *side == Side::Right) { continue; }
                    for &ey in &edges_y { let dy = (ey - position.0).abs(); if dy < min_dy { min_dy = dy; best_y = Some(SnapPoint { position: Pt(position.0 - (ey - y)), target: target.clone() }); } }
                }
            }
        }
        SnapResult { x: best_x, y: best_y }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indents { pub left: Pt, pub right: Pt, pub first_line: Pt }
impl Default for Indents { fn default() -> Self { Self { left: Pt(0.0), right: Pt(0.0), first_line: Pt(0.0) } } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphStyle {
    pub name: String,
    pub based_on: Option<String>,
    pub next_style: Option<String>,
    pub font_family: Option<String>,
    pub font_style: Option<String>,
    pub font_size: Option<Pt>,
    pub leading: Option<Pt>,
    pub tracking: Option<f64>,
    pub alignment: Option<TextAlignment>,
    pub indents: Option<Indents>,
    pub space_before: Option<Pt>,
    pub space_after: Option<Pt>,
    pub color_swatch: Option<String>,
    #[serde(default)]
    pub variation_settings: Vec<FontVariationSetting>,
    pub kerning_mode: Option<KerningMode>,
}

impl Default for ParagraphStyle {
    fn default() -> Self {
        Self { name: "Basic Paragraph".to_string(), based_on: None, next_style: None, font_family: Some("Inter".to_string()), font_style: Some("Regular".to_string()), font_size: Some(Pt(12.0)), leading: Some(Pt(14.4)), tracking: Some(0.0), alignment: Some(TextAlignment::Left), indents: Some(Indents::default()), space_before: Some(Pt(0.0)), space_after: Some(Pt(0.0)), color_swatch: None, variation_settings: Vec::new(), kerning_mode: Some(KerningMode::Metric) }
    }
}

impl ParagraphStyle {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), based_on: None, next_style: None, font_family: None, font_style: None, font_size: None, leading: None, tracking: None, alignment: None, indents: None, space_before: None, space_after: None, color_swatch: None, variation_settings: Vec::new(), kerning_mode: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStyle {
    pub name: String,
    pub based_on: Option<String>,
    pub font_family: Option<String>,
    pub font_style: Option<String>,
    pub font_size: Option<Pt>,
    pub leading: Option<Pt>,
    pub tracking: Option<f64>,
    pub color_swatch: Option<String>,
    #[serde(default)]
    pub variation_settings: Vec<FontVariationSetting>,
    pub kerning_mode: Option<KerningMode>,
}

impl CharacterStyle {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), based_on: None, font_family: None, font_style: None, font_size: None, leading: None, tracking: None, color_swatch: None, variation_settings: Vec::new(), kerning_mode: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStyle { pub name: String, pub fill_color: Option<String>, pub stroke_color: Option<String>, pub stroke_width: Option<Pt> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Styles { pub paragraph_styles: Vec<ParagraphStyle>, pub character_styles: Vec<CharacterStyle>, #[serde(default)] pub object_styles: Vec<ObjectStyle> }

impl Styles {
    pub fn resolve_paragraph_style(&self, name: &str) -> Option<ParagraphStyle> {
        let style = self.paragraph_styles.iter().find(|s| s.name == name)?;
        let mut resolved = style.clone();
        let mut current_based_on = style.based_on.as_ref();
        let mut visited = std::collections::HashSet::new();
        visited.insert(name.to_string());
        while let Some(base_name) = current_based_on {
            if visited.contains(base_name) { break; }
            visited.insert(base_name.clone());
            if let Some(base_style) = self.paragraph_styles.iter().find(|s| &s.name == base_name) {
                if resolved.font_family.is_none() { resolved.font_family = base_style.font_family.clone(); }
                if resolved.font_style.is_none() { resolved.font_style = base_style.font_style.clone(); }
                if resolved.font_size.is_none() { resolved.font_size = base_style.font_size; }
                if resolved.leading.is_none() { resolved.leading = base_style.leading; }
                if resolved.tracking.is_none() { resolved.tracking = base_style.tracking; }
                if resolved.alignment.is_none() { resolved.alignment = base_style.alignment; }
                if resolved.indents.is_none() { resolved.indents = base_style.indents.clone(); }
                if resolved.space_before.is_none() { resolved.space_before = base_style.space_before; }
                if resolved.space_after.is_none() { resolved.space_after = base_style.space_after; }
                if resolved.color_swatch.is_none() { resolved.color_swatch = base_style.color_swatch.clone(); }
                if resolved.variation_settings.is_empty() { resolved.variation_settings = base_style.variation_settings.clone(); }
                if resolved.kerning_mode.is_none() { resolved.kerning_mode = base_style.kerning_mode; }
                current_based_on = base_style.based_on.as_ref();
            } else { break; }
        }
        Some(resolved)
    }

    pub fn resolve_character_style(&self, name: &str) -> Option<CharacterStyle> {
        let style = self.character_styles.iter().find(|s| s.name == name)?;
        let mut resolved = style.clone();
        let mut current_based_on = style.based_on.as_ref();
        let mut visited = std::collections::HashSet::new();
        visited.insert(name.to_string());
        while let Some(base_name) = current_based_on {
            if visited.contains(base_name) { break; }
            visited.insert(base_name.clone());
            if let Some(base_style) = self.character_styles.iter().find(|s| &s.name == base_name) {
                if resolved.font_family.is_none() { resolved.font_family = base_style.font_family.clone(); }
                if resolved.font_style.is_none() { resolved.font_style = base_style.font_style.clone(); }
                if resolved.font_size.is_none() { resolved.font_size = base_style.font_size; }
                if resolved.leading.is_none() { resolved.leading = base_style.leading; }
                if resolved.tracking.is_none() { resolved.tracking = base_style.tracking; }
                if resolved.color_swatch.is_none() { resolved.color_swatch = base_style.color_swatch.clone(); }
                if resolved.variation_settings.is_empty() { resolved.variation_settings = base_style.variation_settings.clone(); }
                if resolved.kerning_mode.is_none() { resolved.kerning_mode = base_style.kerning_mode; }
                current_based_on = base_style.based_on.as_ref();
            } else { break; }
        }
        Some(resolved)
    }
}

impl Default for Styles {
    fn default() -> Self {
        Self { paragraph_styles: vec![ParagraphStyle::default()], character_styles: Vec::new(), object_styles: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document {
            metadata: Metadata { name: "Test Doc".to_string(), author: "".to_string(), description: "".to_string(), created_at: 0, modified_at: 0, dpi: 300, default_unit: Unit::Point, default_bleed: Bleed { top: Pt(0.0), bottom: Pt(0.0), inside: Pt(0.0), outside: Pt(0.0) }, color_profile: "sRGB".to_string(), facing_pages: true },
            fonts: vec![], icc_profiles: vec![], swatches: vec![], styles: Styles { paragraph_styles: vec![], character_styles: vec![], object_styles: vec![] }, spreads: vec![], parent_pages: vec![], layers: vec![Layer::new("l1", "Layer 1")], baseline_grid: BaselineGrid::default(),
        };
        assert_eq!(doc.metadata.name, "Test Doc");
    }

    #[test]
    fn test_paragraph_style_cascade() {
        let base = ParagraphStyle { name: "Base".to_string(), font_family: Some("Arial".to_string()), font_size: Some(Pt(12.0)), ..Default::default() };
        let derived = ParagraphStyle { name: "Derived".to_string(), based_on: Some("Base".to_string()), font_size: Some(Pt(14.0)), ..ParagraphStyle::new("Derived") };
        let styles = Styles { paragraph_styles: vec![base, derived], character_styles: vec![], object_styles: vec![] };
        let resolved = styles.resolve_paragraph_style("Derived").unwrap();
        assert_eq!(resolved.font_family, Some("Arial".to_string()));
        assert_eq!(resolved.font_size, Some(Pt(14.0)));
    }

    #[test]
    fn test_character_style_cascade() {
        let base = CharacterStyle { name: "Base".to_string(), font_style: Some("Bold".to_string()), ..CharacterStyle::new("Base") };
        let derived = CharacterStyle { name: "Derived".to_string(), based_on: Some("Base".to_string()), font_size: Some(Pt(18.0)), ..CharacterStyle::new("Derived") };
        let styles = Styles { paragraph_styles: vec![], character_styles: vec![base, derived], object_styles: vec![] };
        let resolved = styles.resolve_character_style("Derived").unwrap();
        assert_eq!(resolved.font_style, Some("Bold".to_string()));
        assert_eq!(resolved.font_size, Some(Pt(18.0)));
    }
}

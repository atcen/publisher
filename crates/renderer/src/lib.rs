use publisher_color::ColorEngine;
use publisher_core::{
    Document, Frame, FrameData, Group, ImageFitting, ImageFrame, KerningMode, Page, ShapeFrame,
    ShapeType, TextFrame,
};
use publisher_typography::{TypographyEngine, Variation};
use std::time::{Duration, Instant};
use vello::kurbo::{Affine, Circle, Point, Rect};
use vello::peniko::{Color, Fill};
use vello::{Renderer, RendererOptions, Scene};

pub struct VelloRenderer {
    pub renderer: Option<Renderer>,
    pub scene: Scene,
    pub typography: TypographyEngine,
    pub color_engine: ColorEngine,
    last_render_time: Duration,
    pub frame_count: u64,
    total_render_time: Duration,
}

impl VelloRenderer {
    pub fn new(device: Option<&wgpu::Device>) -> Result<Self, Box<dyn std::error::Error>> {
        let renderer = match device {
            Some(dev) => Some(Renderer::new(
                dev,
                RendererOptions {
                    surface_format: Some(wgpu::TextureFormat::Bgra8UnormSrgb),
                    use_cpu: false,
                    antialiasing_support: vello::AaSupport::all(),
                    num_init_threads: None,
                },
            )?),
            None => None,
        };

        let color_engine = ColorEngine::new()?;

        Ok(Self {
            renderer,
            scene: Scene::new(),
            typography: TypographyEngine::new(),
            color_engine,
            last_render_time: Duration::ZERO,
            frame_count: 0,
            total_render_time: Duration::ZERO,
        })
    }

    pub fn render_document(&mut self, document: &Document, spread_index: usize, zoom: f64) -> bool {
        if let Some(profile_data) = document.icc_profiles.first() {
            let _ = self.color_engine.set_cmyk_profile(profile_data);
        }

        if spread_index >= document.spreads.len() {
            return false;
        }

        let start = Instant::now();
        self.scene.reset();

        if let Some(spread) = document.spreads.get(spread_index) {
            let mut x_offset = 0.0;
            let gutter = 10.0 * zoom;
            for (i, page) in spread.pages.iter().enumerate() {
                self.render_page(document, page, i, zoom, x_offset);
                x_offset += page.width.0 * zoom + gutter;
            }
        }

        self.last_render_time = start.elapsed();
        self.frame_count = self.frame_count.saturating_add(1);
        self.total_render_time += self.last_render_time;
        true
    }

    fn render_page(
        &mut self,
        document: &Document,
        page: &Page,
        page_index_in_spread: usize,
        zoom: f64,
        x_offset: f64,
    ) {
        let page_width = page.width.0 * zoom;
        let page_height = page.height.0 * zoom;

        let page_rect = Rect::new(x_offset, 0.0, x_offset + page_width, page_height);
        self.scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::rgb8(255, 255, 255),
            None,
            &page_rect,
        );

        // Render Parent Page hierarchy
        if let Some(parent_id) = &page.applied_parent_id {
            self.render_applied_parent(document, parent_id, page_index_in_spread, zoom, x_offset);
        }

        // Render current page content
        self.render_page_content(document, page, zoom, x_offset);
    }

    fn render_applied_parent(
        &mut self,
        document: &Document,
        parent_id: &str,
        page_index_in_spread: usize,
        zoom: f64,
        x_offset: f64,
    ) {
        if let Some(parent) = document.parent_pages.iter().find(|p| p.id == parent_id) {
            if let Some(base_id) = &parent.based_on_id {
                self.render_applied_parent(document, base_id, page_index_in_spread, zoom, x_offset);
            }

            if let Some(parent_page) = parent.spread.pages.get(page_index_in_spread) {
                self.render_page_content(document, parent_page, zoom, x_offset);
            }
        }
    }

    fn render_page_content(&mut self, document: &Document, page: &Page, zoom: f64, x_offset: f64) {
        for layer in &document.layers {
            if !layer.visible {
                continue;
            }

            for frame in &page.frames {
                if frame.layer_id == layer.id {
                    self.render_frame(document, frame, zoom, Affine::translate((x_offset, 0.0)));
                }
            }
        }
    }

    fn render_frame(
        &mut self,
        document: &Document,
        frame: &Frame,
        zoom: f64,
        parent_transform: Affine,
    ) {
        let x = frame.x.0 * zoom;
        let y = frame.y.0 * zoom;
        let width = frame.width.0 * zoom;
        let height = frame.height.0 * zoom;

        let cx = x + width / 2.0;
        let cy = y + height / 2.0;
        let local_transform = Affine::translate((cx, cy))
            * Affine::rotate(frame.rotation.to_radians())
            * Affine::translate((-cx, -cy))
            * Affine::translate((x, y));

        let final_transform = parent_transform * local_transform;

        match &frame.data {
            FrameData::Text(text_frame) => self.render_text_frame(
                document,
                text_frame,
                0.0,
                0.0,
                width,
                height,
                final_transform,
            ),
            FrameData::Image(image_frame) => {
                self.render_image_frame(image_frame, 0.0, 0.0, width, height, final_transform)
            }
            FrameData::Shape(shape_frame) => {
                self.render_shape_frame(shape_frame, 0.0, 0.0, width, height, final_transform)
            }
            FrameData::Group(group) => self.render_group(document, group, zoom, final_transform),
        }
    }

    fn render_group(
        &mut self,
        document: &Document,
        group: &Group,
        zoom: f64,
        parent_transform: Affine,
    ) {
        for frame in &group.frames {
            self.render_frame(document, frame, zoom, parent_transform);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_text_frame(
        &mut self,
        document: &Document,
        text_frame: &TextFrame,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        transform: Affine,
    ) {
        let text_rect = Rect::new(x, y, x + width, y + height);
        self.scene.fill(
            Fill::NonZero,
            transform,
            Color::rgb8(230, 230, 230),
            None,
            &text_rect,
        );

        let style_name = text_frame.paragraph_style.as_deref().unwrap_or("Standard");
        if let Some(style) = document.styles.resolve_paragraph_style(style_name) {
            if let Some(font) = document
                .fonts
                .iter()
                .find(|f| f.family == style.font_family.as_deref().unwrap_or(""))
            {
                let variations: Vec<Variation> = style
                    .variation_settings
                    .iter()
                    .map(|v| Variation {
                        tag: v.tag.clone(),
                        value: v.value,
                    })
                    .collect();

                let options = publisher_typography::ShapeOptions {
                    font_size: text_frame
                        .font_size_override
                        .unwrap_or(style.font_size.unwrap_or(publisher_core::Pt(12.0))),
                    alignment: style
                        .alignment
                        .unwrap_or(publisher_core::TextAlignment::Left),
                    features: vec![],
                    variations,
                    kerning_mode: style.kerning_mode.unwrap_or(KerningMode::Metric),
                };

                if let Ok(shaped) =
                    self.typography
                        .shape_text(&text_frame.content, &font.data, &options)
                {
                    let _ = shaped;
                }
            }
        }
    }

    fn render_image_frame(
        &mut self,
        frame: &ImageFrame,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        transform: Affine,
    ) {
        let image_rect = Rect::new(x, y, x + width, y + height);
        self.scene.fill(
            Fill::NonZero,
            transform,
            Color::rgb8(200, 220, 240),
            None,
            &image_rect,
        );

        let (content_scale_x, content_scale_y) = match frame.fitting {
            ImageFitting::Stretch => (1.0, 1.0),
            ImageFitting::Fit => (0.8, 0.8),
            ImageFitting::Fill => (1.2, 1.2),
            _ => (1.0, 1.0),
        };

        let content_width = width * content_scale_x;
        let content_height = height * content_scale_y;
        let content_x = x + (width - content_width) / 2.0;
        let content_y = y + (height - content_height) / 2.0;

        let content_rect = Rect::new(
            content_x,
            content_y,
            content_x + content_width,
            content_y + content_height,
        );

        self.scene.push_layer(
            vello::peniko::BlendMode::default(),
            1.0,
            transform,
            &image_rect,
        );
        self.scene.fill(
            Fill::NonZero,
            transform,
            Color::rgb8(150, 180, 210),
            None,
            &content_rect,
        );
        self.scene.pop_layer();
    }

    fn render_shape_frame(
        &mut self,
        frame: &ShapeFrame,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        transform: Affine,
    ) {
        match &frame.shape_type {
            ShapeType::Rectangle => {
                let rect = Rect::new(x, y, x + width, y + height);
                self.scene.fill(
                    Fill::NonZero,
                    transform,
                    Color::rgb8(100, 100, 100),
                    None,
                    &rect,
                );
            }
            ShapeType::Ellipse => {
                if width > 0.0 && height > 0.0 {
                    let cx = x + width / 2.0;
                    let cy = y + height / 2.0;
                    let rx = width / 2.0;
                    let ry = height / 2.0;
                    let max_r = rx.max(ry);
                    let ellipse = Circle::new(Point::new(cx, cy), max_r);
                    let sx = rx / max_r;
                    let sy = ry / max_r;
                    let ellipse_transform = transform
                        * Affine::translate((cx, cy))
                        * Affine::scale_non_uniform(sx, sy)
                        * Affine::translate((-cx, -cy));
                    self.scene.fill(
                        Fill::NonZero,
                        ellipse_transform,
                        Color::rgb8(150, 150, 150),
                        None,
                        &ellipse,
                    );
                }
            }
            ShapeType::Path(_svg_path) => {
                let rect = Rect::new(x, y, x + width, y + height);
                self.scene.fill(
                    Fill::NonZero,
                    transform,
                    Color::rgb8(200, 100, 100),
                    None,
                    &rect,
                );
            }
        }
    }

    pub fn get_last_render_time(&self) -> Duration {
        self.last_render_time
    }

    pub fn get_average_render_time(&self) -> Duration {
        if self.frame_count > 0 {
            Duration::from_secs_f64(self.total_render_time.as_secs_f64() / self.frame_count as f64)
        } else {
            Duration::ZERO
        }
    }
}

pub fn init() {
    publisher_core::init();
    publisher_typography::init();
    publisher_color::init();
    println!("Publisher Renderer Initialized with Typography and Color support");
}

pub fn create_renderer(device: &wgpu::Device) -> Result<VelloRenderer, Box<dyn std::error::Error>> {
    VelloRenderer::new(Some(device))
}

#[cfg(test)]
mod tests {
    use super::*;
    use publisher_core::{BaselineGrid, Bleed, Layer, Margins, Metadata, Pt, Spread, Styles, Unit};

    #[test]
    fn test_renderer_init() {
        init();
    }

    #[test]
    fn test_text_frame_rendering() {
        let mut renderer = VelloRenderer::new(None).expect("Failed to create renderer");
        let page = Page {
            width: Pt(200.0),
            height: Pt(200.0),
            margins: Margins {
                top: Pt(0.0),
                bottom: Pt(0.0),
                inside: Pt(0.0),
                outside: Pt(0.0),
            },
            bleed: None,
            column_count: 1,
            gutter_width: Pt(12.0),
            guides: vec![],
            frames: vec![Frame::new(
                "f1",
                "l1",
                Pt(10.0),
                Pt(10.0),
                Pt(100.0),
                Pt(50.0),
                FrameData::Text(TextFrame::new("Hello")),
            )],
            applied_parent_id: None,
        };
        let doc = Document {
            metadata: Metadata {
                name: "Test".to_string(),
                author: "".to_string(),
                description: "".to_string(),
                created_at: 0,
                modified_at: 0,
                dpi: 72,
                default_unit: Unit::Point,
                default_bleed: Bleed {
                    top: Pt(0.0),
                    bottom: Pt(0.0),
                    inside: Pt(0.0),
                    outside: Pt(0.0),
                },
                color_profile: "sRGB".to_string(),
                facing_pages: false,
            },
            fonts: vec![],
            icc_profiles: vec![],
            swatches: vec![],
            styles: Styles::default(),
            spreads: vec![Spread { pages: vec![page] }],
            parent_pages: vec![],
            layers: vec![Layer::new("l1", "Layer 1")],
            baseline_grid: BaselineGrid::default(),
        };
        renderer.render_document(&doc, 0, 1.0);
        assert_eq!(renderer.frame_count, 1);
    }
}

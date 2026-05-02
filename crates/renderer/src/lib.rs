use publisher_core::{
    Document, Frame, FrameData, Group, ImageFrame, ImageFitting, Page, ShapeFrame, ShapeType, TextFrame,
};
use publisher_typography::TypographyEngine;
use publisher_color::ColorEngine;
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
    pub fn new(device: Option<&wgpu::Device>) -> Result<Self, vello::Error> {
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

        let mut color_engine = ColorEngine::new().map_err(|e| vello::Error::Resource(e.into()))?;

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
        // Update color engine with document profiles if needed
        if let Some(profile_data) = document.icc_profiles.get(0) {
            let _ = self.color_engine.set_cmyk_profile(profile_data);
        }

        // Early return if spread_index is out of bounds (don't update stats for invalid renders)
        if spread_index >= document.spreads.len() {
            return false;
        }

        let start = Instant::now();
        self.scene.reset();

        if let Some(spread) = document.spreads.get(spread_index) {
            let mut x_offset = 0.0;
            let gutter = 10.0 * zoom; // Gutter in points, scaled with zoom
            for page in &spread.pages {
                self.render_page(document, page, zoom, x_offset);
                x_offset += page.width.0 * zoom + gutter;
            }
        }

        self.last_render_time = start.elapsed();
        self.frame_count = self.frame_count.saturating_add(1);
        self.total_render_time += self.last_render_time;
        true
    }

    fn render_page(&mut self, document: &Document, page: &Page, zoom: f64, x_offset: f64) {
        let page_width = page.width.0 * zoom;
        let page_height = page.height.0 * zoom;

        // Draw page background (white) with offset for spread layout
        let page_rect = Rect::new(x_offset, 0.0, x_offset + page_width, page_height);
        self.scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::rgb8(255, 255, 255),
            None,
            &page_rect,
        );

        // Render all frames on this page, respecting layer order and visibility
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

    fn render_frame(&mut self, document: &Document, frame: &Frame, zoom: f64, parent_transform: Affine) {
        let x = frame.x.0 * zoom;
        let y = frame.y.0 * zoom;
        let width = frame.width.0 * zoom;
        let height = frame.height.0 * zoom;

        // Calculate local rotation transform around the center of the frame
        let cx = x + width / 2.0;
        let cy = y + height / 2.0;
        let local_transform = Affine::translate((cx, cy))
            * Affine::rotate(frame.rotation.to_radians())
            * Affine::translate((-cx, -cy))
            * Affine::translate((x, y));

        let final_transform = parent_transform * local_transform;

        match &frame.data {
            FrameData::Text(text_frame) => {
                self.render_text_frame(document, text_frame, 0.0, 0.0, width, height, final_transform)
            }
            FrameData::Image(image_frame) => {
                self.render_image_frame(image_frame, 0.0, 0.0, width, height, final_transform)
            }
            FrameData::Shape(shape_frame) => {
                self.render_shape_frame(shape_frame, 0.0, 0.0, width, height, final_transform)
            }
            FrameData::Group(group) => self.render_group(document, group, zoom, final_transform),
        }
    }

    fn render_group(&mut self, document: &Document, group: &Group, zoom: f64, parent_transform: Affine) {
        for frame in &group.frames {
            self.render_frame(document, frame, zoom, parent_transform);
        }
    }

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
        // Draw text frame boundary (light gray)
        let text_rect = Rect::new(x, y, x + width, y + height);
        self.scene.fill(
            Fill::NonZero,
            transform,
            Color::rgb8(230, 230, 230),
            None,
            &text_rect,
        );

        // If we have a font in the document, try to shape and render
        if let Some(font) = document.fonts.get(0) {
            if let Ok(shaped) = self.typography.shape_text(
                &text_frame.content,
                &font.data,
                publisher_core::Pt(12.0),
                publisher_core::TextAlignment::Left,
                &[],
                &[],
            ) {
                let _ = shaped;
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
        // Draw image frame boundary (placeholder)
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

        let content_rect = Rect::new(content_x, content_y, content_x + content_width, content_y + content_height);
        
        self.scene.push_layer(vello::peniko::BlendMode::default(), 1.0, transform, &image_rect);
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

pub fn create_renderer(device: &wgpu::Device) -> Result<VelloRenderer, vello::Error> {
    VelloRenderer::new(Some(device))
}

#[cfg(test)]
mod tests {
    use super::*;
    use publisher_core::{Bleed, Layer, Margins, Metadata, Pt, Spread, Styles, Unit};

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
            margins: Margins { top: Pt(0.0), bottom: Pt(0.0), inside: Pt(0.0), outside: Pt(0.0) },
            bleed: None,
            column_count: 1,
            gutter_width: Pt(12.0),
            frames: vec![Frame::new("f1", "l1", Pt(10.0), Pt(10.0), Pt(100.0), Pt(50.0), FrameData::Text(TextFrame::new("Hello")))],
        };
        let doc = Document {
            metadata: Metadata { name: "Test".to_string(), author: "".to_string(), description: "".to_string(), created_at: 0, modified_at: 0, dpi: 72, default_unit: Unit::Point, default_bleed: Bleed { top: Pt(0.0), bottom: Pt(0.0), inside: Pt(0.0), outside: Pt(0.0) }, color_profile: "sRGB".to_string(), facing_pages: false },
            fonts: vec![],
            icc_profiles: vec![],
            swatches: vec![],
            styles: Styles::default(),
            spreads: vec![Spread { pages: vec![page] }],
            layers: vec![Layer::new("l1", "Layer 1")],
        };
        renderer.render_document(&doc, 0, 1.0);
        assert_eq!(renderer.frame_count, 1);
    }
}

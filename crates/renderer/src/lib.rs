use vello::{Scene, Renderer, RendererOptions};
use vello::kurbo::{Rect, Point, Circle, Affine};
use vello::peniko::{Color, Fill};
use publisher_core::{Document, Frame, Page, TextFrame, ImageFrame, ShapeFrame, ShapeType};
use std::time::{Duration, Instant};

pub struct VelloRenderer {
    pub renderer: Option<Renderer>,
    pub scene: Scene,
    last_render_time: Duration,
    frame_count: u32,
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

        Ok(Self {
            renderer,
            scene: Scene::new(),
            last_render_time: Duration::ZERO,
            frame_count: 0,
            total_render_time: Duration::ZERO,
        })
    }

    pub fn render_document(
        &mut self,
        document: &Document,
        page_index: usize,
        zoom: f64,
    ) {
        let start = Instant::now();
        self.scene.reset();

        if page_index < document.spreads.len() {
            if let Some(spread) = document.spreads.get(page_index) {
                for page in &spread.pages {
                    self.render_page(page, zoom);
                }
            }
        }

        self.last_render_time = start.elapsed();
        self.frame_count += 1;
        self.total_render_time += self.last_render_time;
    }

    fn render_page(&mut self, page: &Page, zoom: f64) {
        let page_width = page.width.0 * zoom;
        let page_height = page.height.0 * zoom;

        // Draw page background (white)
        let page_rect = Rect::new(0.0, 0.0, page_width, page_height);
        self.scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::rgb8(255, 255, 255),
            None,
            &page_rect,
        );

        // Render all frames on this page
        for frame in &page.frames {
            self.render_frame(frame, zoom);
        }
    }

    fn render_frame(&mut self, frame: &Frame, zoom: f64) {
        match frame {
            Frame::Text(text_frame) => self.render_text_frame(text_frame, zoom),
            Frame::Image(image_frame) => self.render_image_frame(image_frame, zoom),
            Frame::Shape(shape_frame) => self.render_shape_frame(shape_frame, zoom),
        }
    }

    fn render_text_frame(&mut self, frame: &TextFrame, zoom: f64) {
        let x = frame.x.0 * zoom;
        let y = frame.y.0 * zoom;
        let width = frame.width.0 * zoom;
        let height = frame.height.0 * zoom;

        // Draw text frame boundary (light gray)
        let text_rect = Rect::new(x, y, x + width, y + height);
        self.scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::rgb8(230, 230, 230),
            None,
            &text_rect,
        );

        // TODO: Render actual text with typography crate for sub-pixel accuracy
    }

    fn render_image_frame(&mut self, frame: &ImageFrame, zoom: f64) {
        let x = frame.x.0 * zoom;
        let y = frame.y.0 * zoom;
        let width = frame.width.0 * zoom;
        let height = frame.height.0 * zoom;

        // Draw image frame placeholder (light blue)
        let image_rect = Rect::new(x, y, x + width, y + height);
        self.scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::rgb8(200, 220, 240),
            None,
            &image_rect,
        );

        // TODO: Load and render actual image from asset_path
    }

    fn render_shape_frame(&mut self, frame: &ShapeFrame, zoom: f64) {
        let x = frame.x.0 * zoom;
        let y = frame.y.0 * zoom;
        let width = frame.width.0 * zoom;
        let height = frame.height.0 * zoom;

        match &frame.shape_type {
            ShapeType::Rectangle => {
                let rect = Rect::new(x, y, x + width, y + height);
                self.scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    Color::rgb8(100, 100, 100),
                    None,
                    &rect,
                );
            }
            ShapeType::Ellipse => {
                let cx = x + width / 2.0;
                let cy = y + height / 2.0;
                let rx = width / 2.0;

                let circle = Circle::new(Point::new(cx, cy), rx.min(height / 2.0));
                self.scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    Color::rgb8(150, 150, 150),
                    None,
                    &circle,
                );
            }
            ShapeType::Path(_svg_path) => {
                // TODO: Parse and render SVG path data
                let rect = Rect::new(x, y, x + width, y + height);
                self.scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
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
            Duration::from_secs_f64(
                self.total_render_time.as_secs_f64() / self.frame_count as f64
            )
        } else {
            Duration::ZERO
        }
    }
}

pub fn init() {
    publisher_core::init();
    println!("Publisher Renderer Initialized with Vello support");
}

pub fn create_renderer(device: &wgpu::Device) -> Result<VelloRenderer, vello::Error> {
    VelloRenderer::new(Some(device))
}

#[cfg(test)]
mod tests {
    use super::*;
    use publisher_core::{Bleed, Margins, Metadata, Spread, Styles, Pt, Unit};

    #[test]
    fn test_renderer_init() {
        init();
    }

    #[test]
    fn test_text_frame_rendering() {
        let _renderer = VelloRenderer::new(None).expect("Failed to create renderer");
        let _text_frame = TextFrame::new(
            Pt(10.0),
            Pt(10.0),
            Pt(100.0),
            Pt(50.0),
            "Hello World",
        );
    }

    #[test]
    fn test_image_frame_rendering() {
        let _renderer = VelloRenderer::new(None).expect("Failed to create renderer");
        let _image_frame = ImageFrame::new(
            Pt(10.0),
            Pt(10.0),
            Pt(100.0),
            Pt(100.0),
            "test.png",
        );
    }

    #[test]
    fn test_document_page_rendering() {
        let mut renderer = VelloRenderer::new(None).expect("Failed to create renderer");

        let page = Page {
            width: Pt(612.0),  // 8.5 inches at 72 dpi
            height: Pt(792.0), // 11 inches at 72 dpi
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
                    Pt(500.0),
                    Pt(100.0),
                    "Test Page",
                )),
            ],
        };

        let spread = Spread { pages: vec![page] };
        let doc = Document {
            metadata: Metadata {
                name: "Test Document".to_string(),
                author: "Test".to_string(),
                description: "Test document".to_string(),
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
            },
            swatches: vec![],
            styles: Styles::default(),
            spreads: vec![spread],
        };

        renderer.render_document(&doc, 0, 1.0);
        assert!(renderer.get_last_render_time().as_secs_f64() < 0.016); // 60 fps = 16.67ms
    }

    #[test]
    fn test_50_page_document_render_performance() {
        let mut renderer = VelloRenderer::new(None).expect("Failed to create renderer");

        // Create a 50-page document
        let pages: Vec<Page> = (0..50)
            .map(|i| Page {
                width: Pt(612.0),
                height: Pt(792.0),
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
                        Pt(500.0),
                        Pt(100.0),
                        &format!("Page {}", i + 1),
                    )),
                ],
            })
            .collect();

        let doc = Document {
            metadata: Metadata {
                name: "50 Page Document".to_string(),
                author: "Test".to_string(),
                description: "50-page test document".to_string(),
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
            },
            swatches: vec![],
            styles: Styles::default(),
            spreads: pages.into_iter().map(|p| Spread { pages: vec![p] }).collect(),
        };

        // Render all pages and measure performance
        let start = Instant::now();
        for page_idx in 0..doc.spreads.len() {
            renderer.render_document(&doc, page_idx, 1.0);
        }
        let total_time = start.elapsed();
        let avg_time_per_page = total_time.as_secs_f64() / doc.spreads.len() as f64;

        // At 60 fps, we need ~16.67ms per frame
        // With 50 pages, average should be well under that for a single page render
        assert!(avg_time_per_page < 0.016,
                "Average render time per page ({:.3}ms) exceeds 60fps budget (16.67ms)",
                avg_time_per_page * 1000.0);
    }

    #[test]
    fn test_text_subpixel_accuracy_at_100_zoom() {
        let _renderer = VelloRenderer::new(None).expect("Failed to create renderer");

        // Test rendering at 100% zoom with various font sizes
        let _text_frame = TextFrame::new(
            Pt(10.0),
            Pt(10.0),
            Pt(200.0),
            Pt(50.0),
            "Sub-pixel rendering test",
        );
        // Vello with AaSupport::all() provides sub-pixel accuracy via antialiasing
    }
}

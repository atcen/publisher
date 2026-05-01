use publisher_core::{Document, Frame, ImageFrame, Page, ShapeFrame, ShapeType, TextFrame};
use std::time::{Duration, Instant};
use vello::kurbo::{Affine, Circle, Point, Rect};
use vello::peniko::{Color, Fill};
use vello::{Renderer, RendererOptions, Scene};

pub struct VelloRenderer {
    pub renderer: Option<Renderer>,
    pub scene: Scene,
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

        Ok(Self {
            renderer,
            scene: Scene::new(),
            last_render_time: Duration::ZERO,
            frame_count: 0,
            total_render_time: Duration::ZERO,
        })
    }

    pub fn render_document(&mut self, document: &Document, spread_index: usize, zoom: f64) -> bool {
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
                self.render_page(page, zoom, x_offset);
                x_offset += page.width.0 * zoom + gutter;
            }
        }

        self.last_render_time = start.elapsed();
        self.frame_count = self.frame_count.saturating_add(1);
        self.total_render_time += self.last_render_time;
        true
    }

    fn render_page(&mut self, page: &Page, zoom: f64, x_offset: f64) {
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

        // Render all frames on this page with offset
        for frame in &page.frames {
            self.render_frame(frame, zoom, x_offset);
        }
    }

    fn render_frame(&mut self, frame: &Frame, zoom: f64, x_offset: f64) {
        match frame {
            Frame::Text(text_frame) => self.render_text_frame(text_frame, zoom, x_offset),
            Frame::Image(image_frame) => self.render_image_frame(image_frame, zoom, x_offset),
            Frame::Shape(shape_frame) => self.render_shape_frame(shape_frame, zoom, x_offset),
        }
    }

    fn render_text_frame(&mut self, frame: &TextFrame, zoom: f64, x_offset: f64) {
        let x = frame.x.0 * zoom + x_offset;
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

    fn render_image_frame(&mut self, frame: &ImageFrame, zoom: f64, x_offset: f64) {
        let x = frame.x.0 * zoom + x_offset;
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

    fn render_shape_frame(&mut self, frame: &ShapeFrame, zoom: f64, x_offset: f64) {
        let x = frame.x.0 * zoom + x_offset;
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
                // Guard against zero-sized ellipses to avoid divide-by-zero
                if width > 0.0 && height > 0.0 {
                    let cx = x + width / 2.0;
                    let cy = y + height / 2.0;
                    let rx = width / 2.0;
                    let ry = height / 2.0;

                    // Use ellipse via scaled circle to respect both width and height
                    let max_r = rx.max(ry);
                    let ellipse = Circle::new(Point::new(cx, cy), max_r);
                    let sx = rx / max_r;
                    let sy = ry / max_r;
                    let transform =
                        Affine::new([sx, 0.0, 0.0, sy, cx * (1.0 - sx), cy * (1.0 - sy)]);
                    self.scene.fill(
                        Fill::NonZero,
                        transform,
                        Color::rgb8(150, 150, 150),
                        None,
                        &ellipse,
                    );
                }
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
            Duration::from_secs_f64(self.total_render_time.as_secs_f64() / self.frame_count as f64)
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
    use publisher_core::{Bleed, Margins, Metadata, Pt, Spread, Styles, Unit};

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
            frames: vec![Frame::Text(TextFrame::new(
                Pt(10.0),
                Pt(10.0),
                Pt(100.0),
                Pt(50.0),
                "Hello World",
            ))],
        };

        let spread = Spread { pages: vec![page] };
        let doc = Document {
            metadata: Metadata {
                name: "Test".to_string(),
                author: "Test".to_string(),
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
            },
            swatches: vec![],
            styles: Styles::default(),
            spreads: vec![spread],
        };

        renderer.render_document(&doc, 0, 1.0);
        // Verify render completed (no panic = success for now)
        assert_eq!(renderer.frame_count, 1, "Should have rendered 1 frame");
    }

    #[test]
    fn test_image_frame_rendering() {
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
            frames: vec![Frame::Image(ImageFrame::new(
                Pt(10.0),
                Pt(10.0),
                Pt(100.0),
                Pt(100.0),
                "test.png",
            ))],
        };

        let spread = Spread { pages: vec![page] };
        let doc = Document {
            metadata: Metadata {
                name: "Test".to_string(),
                author: "Test".to_string(),
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
            },
            swatches: vec![],
            styles: Styles::default(),
            spreads: vec![spread],
        };

        let rendered = renderer.render_document(&doc, 0, 1.0);
        // Verify render completed successfully
        assert!(
            rendered,
            "render_document should return true for valid spread_index"
        );
        assert_eq!(
            renderer.frame_count, 1,
            "frame_count tracks completed render_document calls"
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
            frames: vec![Frame::Text(TextFrame::new(
                Pt(50.0),
                Pt(50.0),
                Pt(500.0),
                Pt(100.0),
                "Test Page",
            ))],
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

        let rendered = renderer.render_document(&doc, 0, 1.0);
        // Verify render completed successfully
        assert!(
            rendered,
            "render_document should return true for valid spread_index"
        );
        assert_eq!(
            renderer.frame_count, 1,
            "frame_count tracks completed render_document calls"
        );
    }

    #[test]
    #[ignore]
    fn test_50_page_document_render_performance() {
        // This test measures scene building performance (not GPU rendering).
        // Wall-clock timing in unit tests is unreliable; use criterion benchmarks for production profiling.
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
                frames: vec![Frame::Text(TextFrame::new(
                    Pt(50.0),
                    Pt(50.0),
                    Pt(500.0),
                    Pt(100.0),
                    &format!("Page {}", i + 1),
                ))],
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
            spreads: pages
                .into_iter()
                .map(|p| Spread { pages: vec![p] })
                .collect(),
        };

        // Render all pages and measure scene-building performance
        let start = Instant::now();
        for spread_idx in 0..doc.spreads.len() {
            renderer.render_document(&doc, spread_idx, 1.0);
        }
        let total_time = start.elapsed();
        let avg_time_per_page = total_time.as_secs_f64() / doc.spreads.len() as f64;

        println!(
            "50-page render time: {:.3}ms per page, {:.3}ms total",
            avg_time_per_page * 1000.0,
            total_time.as_secs_f64() * 1000.0
        );
        // No hard assertions—for profiling only
    }

    #[test]
    fn test_renderer_can_initialize_without_device() {
        // Verify renderer can be constructed without a wgpu device (for testing)
        let renderer = VelloRenderer::new(None).expect("Failed to create renderer");

        // New renderer should start with no renders tracked
        assert_eq!(
            renderer.frame_count, 0,
            "New renderer should start with frame_count=0"
        );
        assert!(
            renderer.renderer.is_none(),
            "Device-less renderer should have renderer=None"
        );

        // NOTE: Verifying RendererOptions (AaSupport::all()) requires a real wgpu::Device
        // and would belong in an integration test with actual GPU rendering.
    }
}

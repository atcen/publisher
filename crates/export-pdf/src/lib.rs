use printpdf::*;
use publisher_color::ColorEngine;
use publisher_core::{Document as CoreDoc, FrameData, Page as CorePage};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub struct ExportOptions {
    pub show_crop_marks: bool,
    pub show_bleed_marks: bool,
    pub show_registration_marks: bool,
    pub show_color_bars: bool,
    pub output_intent_path: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            show_crop_marks: true,
            show_bleed_marks: true,
            show_registration_marks: true,
            show_color_bars: true,
            output_intent_path: None,
        }
    }
}

pub fn export_to_pdfx1a(
    doc: &CoreDoc,
    options: &ExportOptions,
    output_path: &Path,
) -> Result<(), String> {
    let color_engine = ColorEngine::new().map_err(|e| e.to_string())?;

    // PDF/X-1a requires a title
    let title = if doc.metadata.name.is_empty() {
        "Untitled Publisher Document"
    } else {
        &doc.metadata.name
    };

    // MediaBox should be large enough to contain TrimBox + Bleed + Marks
    // For now, let's assume a fixed margin for marks if enabled (in mm)
    let marks_margin_mm = if options.show_crop_marks { 10.0 } else { 0.0 };

    let all_pages = doc.all_pages();
    if all_pages.is_empty() {
        return Err("Document has no pages".to_string());
    }

    let first_page = &all_pages[0];
    let (pdf_doc, page1, layer1) = PdfDocument::new(
        title,
        Mm(first_page.width.0 as f32 * 0.352778 + marks_margin_mm as f32 * 2.0),
        Mm(first_page.height.0 as f32 * 0.352778 + marks_margin_mm as f32 * 2.0),
        "Layer 1",
    );

    for (i, core_page) in all_pages.iter().enumerate() {
        let (current_page, current_layer) = if i == 0 {
            (page1, layer1)
        } else {
            pdf_doc.add_page(
                Mm(core_page.width.0 as f32 * 0.352778 + marks_margin_mm as f32 * 2.0),
                Mm(core_page.height.0 as f32 * 0.352778 + marks_margin_mm as f32 * 2.0),
                format!("Layer {}", i + 1),
            )
        };

        let layer = pdf_doc.get_page(current_page).get_layer(current_layer);

        // Draw page content
        render_page_content(&layer, core_page, &color_engine, marks_margin_mm)?;

        // Draw printer marks if enabled
        if marks_margin_mm > 0.0 {
            draw_printer_marks(&layer, core_page, options, marks_margin_mm)?;
        }
    }

    let file = File::create(output_path).map_err(|e| e.to_string())?;
    pdf_doc
        .save(&mut BufWriter::new(file))
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn render_page_content(
    layer: &PdfLayerReference,
    page: &CorePage,
    _color_engine: &ColorEngine,
    offset_mm: f64,
) -> Result<(), String> {
    for frame in &page.frames {
        match &frame.data {
            FrameData::Text(_text) => {
                // TODO: Implement text rendering
            }
            FrameData::Shape(_shape) => {
                // TODO: Implement shape rendering
                let rect = Line {
                    points: vec![
                        (
                            Point::new(
                                Mm((offset_mm + frame.x.0 * 0.352778) as f32),
                                Mm((offset_mm + frame.y.0 * 0.352778) as f32),
                            ),
                            false,
                        ),
                        (
                            Point::new(
                                Mm((offset_mm + (frame.x.0 + frame.width.0) * 0.352778) as f32),
                                Mm((offset_mm + frame.y.0 * 0.352778) as f32),
                            ),
                            false,
                        ),
                        (
                            Point::new(
                                Mm((offset_mm + (frame.x.0 + frame.width.0) * 0.352778) as f32),
                                Mm((offset_mm + (frame.y.0 + frame.height.0) * 0.352778) as f32),
                            ),
                            false,
                        ),
                        (
                            Point::new(
                                Mm((offset_mm + frame.x.0 * 0.352778) as f32),
                                Mm((offset_mm + (frame.y.0 + frame.height.0) * 0.352778) as f32),
                            ),
                            false,
                        ),
                    ],
                    is_closed: true,
                };
                layer.add_line(rect);
            }
            FrameData::Image(_image) => {
                // TODO: Implement image rendering
            }
            FrameData::Group(_group) => {
                // TODO: Recurse
            }
        }
    }
    Ok(())
}

fn draw_printer_marks(
    layer: &PdfLayerReference,
    page: &CorePage,
    options: &ExportOptions,
    margin_mm: f64,
) -> Result<(), String> {
    // PDF/X marks usually in Registration color (100% CMYK)
    let registration = Color::Cmyk(Cmyk::new(1.0, 1.0, 1.0, 1.0, None));
    layer.set_outline_color(registration);
    layer.set_outline_thickness(0.25);

    let h_mm = page.height.0 * 0.352778;

    if options.show_crop_marks {
        // Top Left
        let line = Line {
            points: vec![
                (
                    Point::new(Mm((margin_mm - 5.0) as f32), Mm((margin_mm + h_mm) as f32)),
                    false,
                ),
                (
                    Point::new(Mm((margin_mm - 1.0) as f32), Mm((margin_mm + h_mm) as f32)),
                    false,
                ),
            ],
            is_closed: false,
        };
        layer.add_line(line);

        let line = Line {
            points: vec![
                (
                    Point::new(Mm(margin_mm as f32), Mm((margin_mm + h_mm + 5.0) as f32)),
                    false,
                ),
                (
                    Point::new(Mm(margin_mm as f32), Mm((margin_mm + h_mm + 1.0) as f32)),
                    false,
                ),
            ],
            is_closed: false,
        };
        layer.add_line(line);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use publisher_core::builder::DocumentBuilder;
    use std::env;

    #[test]
    fn test_basic_pdf_export() {
        let doc = DocumentBuilder::new()
            .with_name("Test PDF/X-1a Export")
            .with_pages(1)
            .build();

        let mut output_path = env::temp_dir();
        output_path.push("test_export.pdf");

        let options = ExportOptions::default();
        let result = export_to_pdfx1a(&doc, &options, &output_path);

        assert!(result.is_ok(), "Export failed: {:?}", result.err());
        assert!(output_path.exists());
    }
}

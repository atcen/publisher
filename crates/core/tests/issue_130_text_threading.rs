use publisher_core::{
    BaselineGrid, Bleed, Document, Frame, FrameData, Layer, Metadata, Page, Pt, Spread, Styles,
    TextFrame, TextFrameType, Unit,
};

#[test]
fn test_text_threading_flow() {
    let mut doc = Document {
        metadata: Metadata {
            name: "Threading Test".to_string(),
            author: "".to_string(),
            description: "".to_string(),
            created_at: 0,
            modified_at: 0,
            dpi: 72,
            default_unit: Unit::Point,
            default_bleed: Bleed::default(),
            color_profile: "sRGB".to_string(),
            facing_pages: false,
        },
        fonts: vec![],
        icc_profiles: vec![],
        swatches: vec![],
        styles: Styles::default(),
        spreads: vec![Spread {
            pages: vec![Page {
                width: Pt(200.0),
                height: Pt(200.0),
                margins: publisher_core::Margins {
                    top: Pt(0.0),
                    bottom: Pt(0.0),
                    inside: Pt(0.0),
                    outside: Pt(0.0),
                },
                bleed: None,
                column_count: 1,
                gutter_width: Pt(0.0),
                guides: vec![],
                frames: vec![
                    Frame::new(
                        "f1",
                        "l1",
                        Pt(0.0),
                        Pt(0.0),
                        Pt(50.0),
                        Pt(20.0),
                        FrameData::Text(TextFrame {
                            content: "This is a long text that should flow into the next frame."
                                .to_string(),
                            paragraph_style: None,
                            next_frame_id: Some("f2".to_string()),
                            prev_frame_id: None,
                            align_to_baseline_grid: false,
                            frame_type: TextFrameType::Area,
                            font_size_override: Some(Pt(12.0)),
                            text_color_override: None,
                            text_stroke_color_override: None,
                            text_stroke_width_override: None,
                        }),
                    ),
                    Frame::new(
                        "f2",
                        "l1",
                        Pt(60.0),
                        Pt(0.0),
                        Pt(50.0),
                        Pt(20.0),
                        FrameData::Text(TextFrame {
                            content: "".to_string(), // Content is usually stored in the head frame or split
                            paragraph_style: None,
                            next_frame_id: None,
                            prev_frame_id: Some("f1".to_string()),
                            align_to_baseline_grid: false,
                            frame_type: TextFrameType::Area,
                            font_size_override: Some(Pt(12.0)),
                            text_color_override: None,
                            text_stroke_color_override: None,
                            text_stroke_width_override: None,
                        }),
                    ),
                ],
                applied_parent_id: None,
            }],
        }],
        parent_pages: vec![],
        layers: vec![Layer::new("l1", "Layer 1")],
        baseline_grid: BaselineGrid::default(),
    };

    // This test will currently fail to "flow" because the logic isn't implemented.
    // The goal of TDD is to implement the layout engine that distributes the content.
    publisher_core::LayoutEngine::reflow(&mut doc);

    let f1 = doc.find_frame("f1").expect("f1 not found");
    let f2 = doc.find_frame("f2").expect("f2 not found");

    if let (FrameData::Text(tf1), FrameData::Text(tf2)) = (&f1.data, &f2.data) {
        assert!(!tf1.content.is_empty());
        assert!(!tf2.content.is_empty());
        assert_eq!(
            format!("{}{}", tf1.content, tf2.content),
            "This is a long text that should flow into the next frame."
        );
    } else {
        panic!("Frames are not text frames");
    }
}

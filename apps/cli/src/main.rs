use publisher_core::builder::DocumentBuilder;
use publisher_core::paper::PaperFormat;
use publisher_core::{Frame, TextFrame, Pt};

fn main() {
    println!("--- Publisher Prototype ---");
    
    // Create a new document using the builder
    let mut doc = DocumentBuilder::new()
        .with_name("Prototype Magazine")
        .with_format(PaperFormat::A4)
        .with_pages(2)
        .with_facing_pages(true)
        .build();
    
    // Add some content to the first page
    if let Some(spread) = doc.spreads.get_mut(0) {
        if let Some(page) = spread.pages.get_mut(0) {
            let text_frame = TextFrame::new(
                Pt(50.0), Pt(50.0), Pt(400.0), Pt(100.0),
                "Welcome to the Publisher Prototype!"
            );
            page.frames.push(Frame::Text(text_frame));
        }
    }
    
    println!("Document created: {}", doc.metadata.name);
    println!("Pages: {}", doc.spreads.iter().map(|s| s.pages.len()).sum::<usize>());
    
    // Serialize to JSON to show the structure
    let json = serde_json::to_string_pretty(&doc).unwrap();
    println!("\nDocument Structure (JSON):");
    println!("{}", json);
}

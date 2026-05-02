use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use publisher_core::{
    Frame, FrameData, ImageFrame, Margins, Page, Pt, ShapeFrame, ShapeType, TextFrame,
};

fn bench_frame_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_creation");
    group.sample_size(1000);

    group.bench_function("text_frame_creation", |b| {
        b.iter(|| black_box(TextFrame::new(black_box("Hello World"))))
    });

    group.bench_function("image_frame_creation", |b| {
        b.iter(|| black_box(ImageFrame::new(black_box("/path/to/image.jpg"))))
    });

    group.bench_function("shape_frame_rectangle", |b| {
        b.iter(|| black_box(ShapeFrame::new(black_box(ShapeType::Rectangle))))
    });

    group.finish();
}

fn bench_frame_full_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_full_creation");
    group.sample_size(1000);

    group.bench_function("frame_text_full", |b| {
        b.iter(|| {
            let data = FrameData::Text(TextFrame::new("Content"));
            black_box(Frame::new(
                "f1",
                "l1",
                Pt(10.0),
                Pt(20.0),
                Pt(200.0),
                Pt(100.0),
                data,
            ))
        })
    });

    group.finish();
}

fn bench_page_frame_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("page_frame_operations");
    group.sample_size(100);

    group.bench_function("page_creation", |b| {
        b.iter(|| {
            black_box(Page {
                width: Pt(595.0),
                height: Pt(842.0),
                margins: Margins {
                    top: Pt(36.0),
                    bottom: Pt(36.0),
                    inside: Pt(36.0),
                    outside: Pt(36.0),
                },
                bleed: None,
                column_count: 2,
                gutter_width: Pt(12.0),
                guides: vec![],
                frames: Vec::new(),
                applied_parent_id: None,
            })
        })
    });

    for frame_count in [1, 5, 10, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("add_{}_frames", frame_count)),
            frame_count,
            |b, &count| {
                b.iter(|| {
                    let mut page = Page {
                        width: Pt(595.0),
                        height: Pt(842.0),
                        margins: Margins {
                            top: Pt(36.0),
                            bottom: Pt(36.0),
                            inside: Pt(36.0),
                            outside: Pt(36.0),
                        },
                        bleed: None,
                        column_count: 2,
                        gutter_width: Pt(12.0),
                        guides: vec![],
                        frames: Vec::with_capacity(count),
                        applied_parent_id: None,
                    };
                    for i in 0..count {
                        let data = FrameData::Text(TextFrame::new("Frame text"));
                        let frame = Frame::new(
                            &format!("f{}", i),
                            "l1",
                            Pt(10.0 + (i as f64 * 50.0)),
                            Pt(20.0),
                            Pt(100.0),
                            Pt(50.0),
                            data,
                        );
                        page.frames.push(black_box(frame));
                    }
                    black_box(page)
                })
            },
        );
    }

    group.finish();
}

fn bench_frame_cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_cloning");
    group.sample_size(500);

    let data = FrameData::Text(TextFrame::new("Hello World"));
    let text_frame = Frame::new("f1", "l1", Pt(10.0), Pt(20.0), Pt(200.0), Pt(100.0), data);

    group.bench_function("clone_text_frame", |b| {
        b.iter(|| black_box(black_box(&text_frame).clone()))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_frame_creation,
    bench_frame_full_creation,
    bench_page_frame_operations,
    bench_frame_cloning
);
criterion_main!(benches);

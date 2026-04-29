use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use publisher_core::{Frame, TextFrame, ImageFrame, ShapeFrame, ShapeType, Page, Margins};

fn bench_frame_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_creation");
    group.sample_size(1000);

    group.bench_function("text_frame_creation", |b| {
        b.iter(|| {
            black_box(TextFrame::new(
                black_box(10.0),
                black_box(20.0),
                black_box(200.0),
                black_box(100.0),
                black_box("Hello World"),
            ))
        })
    });

    group.bench_function("image_frame_creation", |b| {
        b.iter(|| {
            black_box(ImageFrame::new(
                black_box(10.0),
                black_box(20.0),
                black_box(300.0),
                black_box(150.0),
                black_box("/path/to/image.jpg"),
            ))
        })
    });

    group.bench_function("shape_frame_rectangle", |b| {
        b.iter(|| {
            black_box(ShapeFrame::new(
                black_box(0.0),
                black_box(0.0),
                black_box(100.0),
                black_box(100.0),
                black_box(ShapeType::Rectangle),
            ))
        })
    });

    group.bench_function("shape_frame_ellipse", |b| {
        b.iter(|| {
            black_box(ShapeFrame::new(
                black_box(0.0),
                black_box(0.0),
                black_box(75.0),
                black_box(50.0),
                black_box(ShapeType::Ellipse),
            ))
        })
    });

    group.finish();
}

fn bench_frame_enum_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_enum_variants");
    group.sample_size(1000);

    group.bench_function("frame_text_variant", |b| {
        b.iter(|| {
            black_box(Frame::Text(TextFrame::new(
                black_box(10.0),
                black_box(20.0),
                black_box(200.0),
                black_box(100.0),
                black_box("Content"),
            )))
        })
    });

    group.bench_function("frame_image_variant", |b| {
        b.iter(|| {
            black_box(Frame::Image(ImageFrame::new(
                black_box(10.0),
                black_box(20.0),
                black_box(300.0),
                black_box(150.0),
                black_box("/img.jpg"),
            )))
        })
    });

    group.bench_function("frame_shape_variant", |b| {
        b.iter(|| {
            black_box(Frame::Shape(ShapeFrame::new(
                black_box(0.0),
                black_box(0.0),
                black_box(100.0),
                black_box(100.0),
                black_box(ShapeType::Rectangle),
            )))
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
                width: 595.0,
                height: 842.0,
                margins: Margins {
                    top: 36.0,
                    bottom: 36.0,
                    inside: 36.0,
                    outside: 36.0,
                },
                frames: Vec::new(),
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
                        width: 595.0,
                        height: 842.0,
                        margins: Margins {
                            top: 36.0,
                            bottom: 36.0,
                            inside: 36.0,
                            outside: 36.0,
                        },
                        frames: Vec::with_capacity(count),
                    };
                    for i in 0..count {
                        let frame = Frame::Text(TextFrame::new(
                            10.0 + (i as f64 * 50.0),
                            20.0,
                            100.0,
                            50.0,
                            "Frame text",
                        ));
                        black_box(page.frames.push(black_box(frame)));
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

    let text_frame = Frame::Text(TextFrame::new(10.0, 20.0, 200.0, 100.0, "Hello World"));
    group.bench_function("clone_text_frame", |b| {
        b.iter(|| black_box(black_box(&text_frame).clone()))
    });

    let image_frame = Frame::Image(ImageFrame::new(10.0, 20.0, 300.0, 150.0, "/path/image.jpg"));
    group.bench_function("clone_image_frame", |b| {
        b.iter(|| black_box(black_box(&image_frame).clone()))
    });

    let shape_frame = Frame::Shape(ShapeFrame::new(0.0, 0.0, 100.0, 100.0, ShapeType::Rectangle));
    group.bench_function("clone_shape_frame", |b| {
        b.iter(|| black_box(black_box(&shape_frame).clone()))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_frame_creation,
    bench_frame_enum_creation,
    bench_page_frame_operations,
    bench_frame_cloning
);
criterion_main!(benches);

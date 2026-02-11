use criterion::{black_box, criterion_group, criterion_main, Criterion};
use magicer::domain::repositories::magic_repository::MagicRepository;
use magicer::infrastructure::magic::fake_magic_repository::FakeMagicRepository;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn bench_magic_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let repo = Arc::new(FakeMagicRepository::new().unwrap());
    
    let pdf_data = b"%PDF-1.4\n";
    let data = vec![0u8; 1024]; // 1KB of dummy data
    
    c.bench_function("analyze_buffer_pdf", |b| {
        b.to_async(&rt).iter(|| {
            let repo = repo.clone();
            async move {
                let _ = repo.analyze_buffer(black_box(pdf_data), black_box("test.pdf")).await;
            }
        })
    });

    c.bench_function("analyze_buffer_dummy", |b| {
        b.to_async(&rt).iter(|| {
            let repo = repo.clone();
            let data_ref = &data;
            async move {
                let _ = repo.analyze_buffer(black_box(data_ref), black_box("test.bin")).await;
            }
        })
    });
}

criterion_group!(benches, bench_magic_analysis);
criterion_main!(benches);

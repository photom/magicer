use criterion::{black_box, criterion_group, criterion_main, Criterion};
use magicer::domain::value_objects::filename::WindowsCompatibleFilename;
use magicer::domain::value_objects::path::RelativePath;

fn bench_filename_validation(c: &mut Criterion) {
    let valid_name = "test_file_name_with_some_length.txt";
    let max_name = "a".repeat(310);
    
    c.bench_function("filename_validation_short", |b| {
        b.iter(|| {
            let _ = WindowsCompatibleFilename::new(black_box(valid_name));
        })
    });

    c.bench_function("filename_validation_max", |b| {
        b.iter(|| {
            let _ = WindowsCompatibleFilename::new(black_box(&max_name));
        })
    });
}

fn bench_path_validation(c: &mut Criterion) {
    let valid_path = "uploads/2026/02/11/some_random_directory/file.txt";
    
    c.bench_function("path_validation", |b| {
        b.iter(|| {
            let _ = RelativePath::new(black_box(valid_path));
        })
    });
}

criterion_group!(benches, bench_filename_validation, bench_path_validation);
criterion_main!(benches);

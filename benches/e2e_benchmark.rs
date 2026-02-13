use criterion::{criterion_group, criterion_main, Criterion};
use axum_test::TestServer;
use axum::http::{header, HeaderValue};
use magicer::presentation::http::router::create_router;
use magicer::presentation::state::app_state::AppState;
use magicer::presentation::http::middleware::request_id;
use magicer::infrastructure::magic::fake_magic_repository::FakeMagicRepository;
use magicer::infrastructure::filesystem::sandbox::PathSandbox;
use magicer::infrastructure::auth::basic_auth_service::BasicAuthService;
use std::sync::Arc;
use std::path::PathBuf;
use axum::middleware;
use tokio::runtime::Runtime;

use magicer::infrastructure::filesystem::temp_storage_service::FsTempStorageService;

fn setup_bench_server() -> TestServer {
    let magic_repo = Arc::new(FakeMagicRepository::new().unwrap());
    let temp_dir = "/tmp/magicer_bench";
    std::fs::create_dir_all(temp_dir).unwrap();
    let sandbox = Arc::new(PathSandbox::new(PathBuf::from(temp_dir)));
    let temp_storage = Arc::new(FsTempStorageService::new(PathBuf::from(temp_dir).join("temp")));
    let auth_service = Arc::new(BasicAuthService::new("admin", "secret"));
    let config = Arc::new(magicer::infrastructure::config::server_config::ServerConfig::default());
    let state = Arc::new(AppState::new(magic_repo, sandbox, temp_storage, auth_service, config));
    let app = create_router(state)
        .layer(middleware::from_fn(request_id::add_request_id));
    TestServer::new(app).unwrap()
}

fn bench_e2e_ping(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let server = setup_bench_server();

    c.bench_function("e2e_ping", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = server.get("/v1/ping").await;
        })
    });
}

fn bench_e2e_content(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let server = setup_bench_server();
    let auth = "Basic YWRtaW46c2VjcmV0";

    c.bench_function("e2e_content_pdf", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = server
                .post("/v1/magic/content")
                .add_query_param("filename", "test.pdf")
                .add_header(header::AUTHORIZATION, HeaderValue::from_static(auth))
                .text("%PDF-1.4")
                .await;
        })
    });
}

fn bench_e2e_path(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let server = setup_bench_server();
    let auth = "Basic YWRtaW46c2VjcmV0";
    
    // Setup file in sandbox
    let temp_dir = "/tmp/magicer_bench";
    let file_path = PathBuf::from(temp_dir).join("bench.png");
    std::fs::write(&file_path, b"\x89PNG
\x1a
").unwrap();

    c.bench_function("e2e_path_png", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = server
                .post("/v1/magic/path")
                .add_query_param("filename", "bench.png")
                .add_query_param("path", "bench.png")
                .add_header(header::AUTHORIZATION, HeaderValue::from_static(auth))
                .await;
        })
    });
}

criterion_group!(benches, bench_e2e_ping, bench_e2e_content, bench_e2e_path);
criterion_main!(benches);

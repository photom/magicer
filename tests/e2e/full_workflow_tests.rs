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

fn setup_test_server() -> TestServer {
    let magic_repo = Arc::new(FakeMagicRepository::new().unwrap());
    let temp_dir = "/tmp/magicer_e2e";
    std::fs::create_dir_all(temp_dir).unwrap();
    let sandbox = Arc::new(PathSandbox::new(PathBuf::from(temp_dir)));
    let auth_service = Arc::new(BasicAuthService::new("admin", "secret"));
    let state = Arc::new(AppState::new(magic_repo, sandbox, auth_service));
    let app = create_router(state)
        .layer(middleware::from_fn(request_id::add_request_id));
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_ping_endpoint() {
    let server = setup_test_server();
    let response = server.get("/v1/ping").await;
    response.assert_status_ok();
    let json = response.json::<serde_json::Value>();
    assert_eq!(json["message"], "pong");
    assert!(json.get("request_id").is_some());
}

#[tokio::test]
async fn test_content_analysis_success() {
    let server = setup_test_server();
    let response = server
        .post("/v1/magic/content")
        .add_query_param("filename", "test.pdf")
        .add_header(header::AUTHORIZATION, HeaderValue::from_static("Basic YWRtaW46c2VjcmV0"))
        .text("%PDF-1.4")
        .await;
    
    response.assert_status_ok();
    let json = response.json::<serde_json::Value>();
    assert_eq!(json["result"]["mime_type"], "application/pdf");
}

#[tokio::test]
async fn test_path_analysis_success() {
    let server = setup_test_server();
    
    // Setup file in sandbox
    let temp_dir = "/tmp/magicer_e2e";
    let file_path = PathBuf::from(temp_dir).join("test.png");
    std::fs::write(&file_path, b"\x89PNG\r\n\x1a\n").unwrap();

    let response = server
        .post("/v1/magic/path")
        .add_query_param("filename", "test.png")
        .add_query_param("path", "test.png")
        .add_header(header::AUTHORIZATION, HeaderValue::from_static("Basic YWRtaW46c2VjcmV0"))
        .await;
    
    response.assert_status_ok();
    let json = response.json::<serde_json::Value>();
    assert_eq!(json["result"]["mime_type"], "image/png");
}

#[tokio::test]
async fn test_auth_required() {
    let server = setup_test_server();
    let response = server.post("/v1/magic/content").add_query_param("filename", "test.pdf").await;
    response.assert_status_unauthorized();
}

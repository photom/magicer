use axum_test::TestServer;
use axum::http::{header, HeaderValue};
use magicer::presentation::http::router::create_router;
use magicer::presentation::state::app_state::AppState;
use magicer::presentation::http::middleware::{request_id, error_handler};
use magicer::infrastructure::magic::fake_magic_repository::FakeMagicRepository;
use magicer::infrastructure::filesystem::sandbox::PathSandbox;
use magicer::infrastructure::auth::basic_auth_service::BasicAuthService;
use magicer::infrastructure::config::server_config::ServerConfig;
use std::sync::Arc;
use std::path::PathBuf;
use axum::middleware;

const TEST_SANDBOX_DIR: &str = "/tmp/magicer_e2e";

fn setup_test_server(config_override: Option<Box<dyn FnOnce(&mut ServerConfig)>>) -> TestServer {
    let magic_repo = Arc::new(FakeMagicRepository::new().unwrap());
    std::fs::create_dir_all(TEST_SANDBOX_DIR).unwrap();
    let sandbox = Arc::new(PathSandbox::new(PathBuf::from(TEST_SANDBOX_DIR)));
    let auth_service = Arc::new(BasicAuthService::new("admin", "secret"));
    
    let mut config = ServerConfig::default();
    config.sandbox.base_dir = TEST_SANDBOX_DIR.to_string();
    config.auth.username = "admin".to_string();
    config.auth.password = "secret".to_string();
    
    if let Some(f) = config_override {
        f(&mut config);
    }
    
    let state = Arc::new(AppState::new(magic_repo, sandbox, auth_service, Arc::new(config)));
    let app = create_router(state)
        .layer(middleware::from_fn(error_handler::handle_error))
        .layer(middleware::from_fn(request_id::add_request_id));
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_ping_endpoint() {
    let server = setup_test_server(None);
    let response = server.get("/v1/ping").await;
    response.assert_status_ok();
    let json = response.json::<serde_json::Value>();
    assert_eq!(json["message"], "pong");
    assert!(json.get("request_id").is_some());
}

#[tokio::test]
async fn test_content_analysis_success() {
    let server = setup_test_server(None);
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
async fn test_analyze_content_large_file_success() {
    // Set threshold to 0 to force temp file path
    let server = setup_test_server(Some(Box::new(|config| {
        config.analysis.large_file_threshold_mb = 0;
    })));
    
    let response = server
        .post("/v1/magic/content")
        .add_query_param("filename", "large.png")
        .add_header(header::AUTHORIZATION, HeaderValue::from_static("Basic YWRtaW46c2VjcmV0"))
        .bytes(b"\x89PNG\r\n\x1a\n".to_vec())
        .await;
    
    response.assert_status_ok();
    let json = response.json::<serde_json::Value>();
    assert_eq!(json["result"]["mime_type"], "image/png");
}

#[tokio::test]
async fn test_path_analysis_success() {
    let server = setup_test_server(None);
    
    // Setup file in sandbox
    let file_path = PathBuf::from(TEST_SANDBOX_DIR).join("test.png");
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
async fn test_analyze_path_not_found() {
    let server = setup_test_server(None);
    
    let response = server
        .post("/v1/magic/path")
        .add_query_param("filename", "missing.bin")
        .add_query_param("path", "missing.bin")
        .add_header(header::AUTHORIZATION, HeaderValue::from_static("Basic YWRtaW46c2VjcmV0"))
        .await;
    
    response.assert_status_not_found();
    let json = response.json::<serde_json::Value>();
    assert!(json["error"].as_str().unwrap().contains("Not Found"));
}

#[tokio::test]
async fn test_auth_required_rejection() {
    let server = setup_test_server(None);
    let response = server.post("/v1/magic/content").add_query_param("filename", "test.pdf").await;
    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_invalid_filename_rejection() {
    let server = setup_test_server(None);
    let response = server
        .post("/v1/magic/content")
        .add_query_param("filename", "bad/name.txt")
        .add_header(header::AUTHORIZATION, HeaderValue::from_static("Basic YWRtaW46c2VjcmV0"))
        .await;
    
    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_path_traversal_rejection() {
    let server = setup_test_server(None);
    let response = server
        .post("/v1/magic/path")
        .add_query_param("filename", "etc")
        .add_query_param("path", "../../etc/passwd")
        .add_header(header::AUTHORIZATION, HeaderValue::from_static("Basic YWRtaW46c2VjcmV0"))
        .await;
    
    // RelativePath::new rejects '..' so this will be a 400 Bad Request
    response.assert_status_bad_request();
}

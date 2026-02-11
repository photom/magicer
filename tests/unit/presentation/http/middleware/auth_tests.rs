use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::from_fn_with_state,
    routing::get,
    Router,
};
use magicer::presentation::http::middleware::auth::require_auth;
use magicer::presentation::state::app_state::AppState;
use magicer::infrastructure::magic::fake_magic_repository::FakeMagicRepository;
use magicer::infrastructure::filesystem::sandbox::PathSandbox;
use magicer::domain::services::authentication_service::AuthenticationService;
use magicer::domain::value_objects::auth::BasicAuthCredentials;
use magicer::domain::errors::AuthenticationError;
use tower::ServiceExt;
use std::sync::Arc;
use std::path::PathBuf;
use futures_util::future::BoxFuture;

struct FakeAuthService {
    valid_user: String,
    valid_pass: String,
}

impl AuthenticationService for FakeAuthService {
    fn verify_credentials<'a>(&'a self, credentials: &'a BasicAuthCredentials) -> BoxFuture<'a, Result<(), AuthenticationError>> {
        Box::pin(async move {
            if credentials.username() == self.valid_user && credentials.password() == self.valid_pass {
                Ok(())
            } else {
                Err(AuthenticationError::InvalidCredentials)
            }
        })
    }
}

async fn build_app(auth_service: Arc<dyn AuthenticationService>) -> Router {
    let magic_repo = Arc::new(FakeMagicRepository::new().unwrap());
    let sandbox = Arc::new(PathSandbox::new(PathBuf::from("/tmp")));
    let state = Arc::new(AppState::new(magic_repo, sandbox, auth_service));
    
    Router::new()
        .route("/", get(|| async { StatusCode::OK }))
        .layer(from_fn_with_state(state, require_auth))
}

#[tokio::test]
async fn test_auth_middleware_missing_header() {
    let auth_service = Arc::new(FakeAuthService { valid_user: "u".into(), valid_pass: "p".into() });
    let app = build_app(auth_service).await;

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_middleware_invalid_credentials() {
    let auth_service = Arc::new(FakeAuthService { valid_user: "u".into(), valid_pass: "p".into() });
    let app = build_app(auth_service).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("Authorization", "Basic dTp3cm9uZw==") // u:wrong
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_middleware_valid_credentials() {
    let auth_service = Arc::new(FakeAuthService { valid_user: "u".into(), valid_pass: "p".into() });
    let app = build_app(auth_service).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("Authorization", "Basic dTpw") // u:p
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

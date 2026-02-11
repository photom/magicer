use std::sync::Arc;
use std::path::PathBuf;
use axum::middleware;
use tokio::net::TcpListener;
use magicer::presentation::state::app_state::AppState;
use magicer::presentation::http::router::create_router;
use magicer::presentation::http::middleware::request_id;
use magicer::infrastructure::config::server_config::ServerConfig;
use magicer::infrastructure::filesystem::sandbox::PathSandbox;
use magicer::infrastructure::auth::basic_auth_service::BasicAuthService;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = ServerConfig::load_from_env();
    tracing::info!("Server configuration loaded: {:?}", config);

    // Initialize infrastructure
    // Use real LibmagicRepository built from source
    let magic_repo = Arc::new(magicer::infrastructure::magic::libmagic_repository::LibmagicRepository::new()
        .expect("Failed to initialize real libmagic repository"));
    
    let sandbox = Arc::new(PathSandbox::new(PathBuf::from(&config.analysis.temp_dir)));
    
    // In a real app, credentials would come from config or secrets manager
    let auth_service = Arc::new(BasicAuthService::new("admin", "secret"));

    // Initialize application state
    let app_state = Arc::new(AppState::new(magic_repo, sandbox, auth_service));

    // Build router with middleware
    let app = create_router(app_state)
        .layer(middleware::from_fn(request_id::add_request_id))
        // Note: Auth middleware is selectively applied in router or we can apply it globally here if all routes require auth.
        // The requirement says /v1/ping is public.
        // So we should NOT apply auth globally here if we want selective auth.
        // But our router implementation creates routes.
        // We need to verify where auth middleware is applied.
        // Looking at our auth tests, we used `from_fn_with_state`.
        // Let's check `create_router` implementation.
        ;

    // Address to bind to
    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app).await.unwrap();
}

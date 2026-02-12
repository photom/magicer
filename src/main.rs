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
    let config = ServerConfig::load();
    config.validate().expect("Failed to validate configuration");
    tracing::info!("Server configuration loaded: {:?}", config);

    // Initialize infrastructure
    // Use real LibmagicRepository built from source
    let magic_repo = Arc::new(magicer::infrastructure::magic::libmagic_repository::LibmagicRepository::new()
        .expect("Failed to initialize real libmagic repository"));
    
    let sandbox = Arc::new(PathSandbox::new(PathBuf::from(&config.sandbox.base_dir)));
    
    let auth_service = Arc::new(BasicAuthService::new(&config.auth.username, &config.auth.password));

    // Address to bind to
    let addr = format!("{}:{}", config.server.host, config.server.port);

    // Initialize application state
    let app_state = Arc::new(AppState::new(magic_repo, sandbox, auth_service, Arc::new(config)));

    // Build router with middleware
    let app = create_router(app_state)
        .layer(middleware::from_fn(magicer::presentation::http::middleware::error_handler::handle_error))
        .layer(middleware::from_fn(request_id::add_request_id));

    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown...");
}

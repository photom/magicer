use axum::{extract::DefaultBodyLimit, middleware};
use clap::Parser;
use magicer::infrastructure::auth::basic_auth_service::BasicAuthService;
use magicer::infrastructure::config::server_config::ServerConfig;
use magicer::infrastructure::filesystem::sandbox::PathSandbox;
use magicer::infrastructure::telemetry::metrics::AppMetrics;
use magicer::infrastructure::telemetry::Telemetry;
use magicer::presentation::http::middleware::request_id;
use magicer::presentation::http::router::create_router;
use magicer::presentation::state::app_state::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tower::limit::concurrency::ConcurrencyLimitLayer;
use tower_http::timeout::TimeoutLayer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, env = "MAGICER_CONFIG_PATH")]
    config: Option<String>,
}

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let args = Args::parse();

    // Initialise OpenTelemetry (traces + metrics + logs) before anything else.
    // Replaces the previous `tracing_subscriber::fmt::init()` call.
    let _telemetry = Telemetry::init();

    // Load configuration
    let config = ServerConfig::load(args.config);
    config.validate().expect("Failed to validate configuration");
    tracing::info!("Server configuration loaded: {:?}", config);

    // Apply max_open_files limit
    if let Err(e) = rlimit::setrlimit(
        rlimit::Resource::NOFILE,
        config.server.max_open_files as u64,
        config.server.max_open_files as u64,
    ) {
        tracing::warn!(error = %e, "Failed to set max_open_files limit");
    }

    // Initialize infrastructure
    // Use real LibmagicRepository built from source
    let magic_repo = Arc::new(
        magicer::infrastructure::magic::libmagic_repository::LibmagicRepository::new(
            config.analysis.mmap_fallback_enabled,
        )
        .expect("Failed to initialize real libmagic repository"),
    );

    let sandbox = Arc::new(PathSandbox::new(PathBuf::from(&config.sandbox.base_dir)));

    let temp_storage = Arc::new(
        magicer::infrastructure::filesystem::temp_storage_service::FsTempStorageService::new(
            PathBuf::from(&config.analysis.temp_dir),
        ),
    );

    let auth_service = Arc::new(BasicAuthService::new(
        &config.auth.username,
        &config.auth.password,
    ));

    // Build OTel metric instruments from the global meter provider (set by Telemetry::init).
    let meter = opentelemetry::global::meter(env!("CARGO_PKG_NAME"));
    let metrics = Arc::new(AppMetrics::new(&meter));

    // Address to bind to
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let socket_addr: std::net::SocketAddr = addr.parse().expect("Invalid bind address");

    // Initialize application state
    let app_state = Arc::new(AppState::new(
        magic_repo,
        sandbox,
        temp_storage,
        auth_service,
        Arc::new(config.clone()),
        Arc::clone(&metrics),
    ));

    // Build router with middleware and limits
    let app = create_router(app_state)
        .layer(middleware::from_fn(
            magicer::presentation::http::middleware::error_handler::handle_error,
        ))
        .layer(middleware::from_fn(request_id::add_request_id))
        .layer(ConcurrencyLimitLayer::new(
            config.server.max_connections as usize,
        ))
        .layer(DefaultBodyLimit::max(
            (config.server.limits.max_body_size_mb * 1024 * 1024) as usize,
        ))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::GATEWAY_TIMEOUT,
            Duration::from_secs(config.server.timeouts.read_timeout_secs),
        ));

    // Create a TCP listener with custom backlog
    let socket = socket2::Socket::new(
        socket2::Domain::for_address(socket_addr),
        socket2::Type::STREAM,
        Some(socket2::Protocol::TCP),
    )
    .unwrap();

    socket.set_reuse_address(true).unwrap();
    socket.bind(&socket_addr.into()).unwrap();
    socket.listen(config.server.backlog as i32).unwrap();

    let std_listener: std::net::TcpListener = socket.into();
    std_listener.set_nonblocking(true).unwrap();
    let listener = TcpListener::from_std(std_listener).unwrap();

    // L-01: server.addr and server.backlog are structured fields — not interpolated strings.
    tracing::info!(
        server.addr = %addr,
        server.backlog = config.server.backlog,
        "Server listening"
    );

    // Start background cleanup task
    let cleanup_config = config.clone();
    let cleanup_metrics = Arc::clone(&metrics);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
        loop {
            interval.tick().await;
            let temp_dir = &cleanup_config.analysis.temp_dir;
            let max_age = cleanup_config.analysis.temp_file_max_age_secs;

            let cycle_start = Instant::now();
            let mut removed_count: u64 = 0;

            if let Ok(mut entries) = tokio::fs::read_dir(temp_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let is_expired = entry.metadata().await.ok().and_then(|m| {
                        if !m.is_file() {
                            return None;
                        }
                        m.modified()
                            .ok()
                            .and_then(|t| t.elapsed().ok())
                            .map(|e| e.as_secs() > max_age)
                    });

                    if is_expired == Some(true) {
                        let path = entry.path();
                        // L-02: log only the filename component, never the
                        // full resolved sandbox path.
                        let file_name = path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned();
                        if let Err(e) = tokio::fs::remove_file(&path).await {
                            tracing::warn!(
                                file.name = %file_name,
                                error = %e,
                                "Failed to remove orphaned temp file"
                            );
                        } else {
                            tracing::info!(
                                file.name = %file_name,
                                "Removed orphaned temp file"
                            );
                            removed_count += 1;
                        }
                    }
                }
            }

            let cycle_ms = cycle_start.elapsed().as_secs_f64() * 1000.0;
            cleanup_metrics
                .tempfile_cleanup_duration
                .record(cycle_ms, &[]);
            if removed_count > 0 {
                cleanup_metrics
                    .tempfile_cleanup_removed
                    .add(removed_count, &[]);
            }

            // Emit cleanup removed count as a structured span attribute (not a raw list).
            tracing::debug!(
                filesystem.cleanup.removed_count = removed_count,
                "Temp file cleanup cycle complete"
            );
        }
    });

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    // Flush all in-flight telemetry before the process exits.
    _telemetry.shutdown();
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

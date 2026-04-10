use opentelemetry::metrics::{Counter, Histogram, Meter, UpDownCounter};

/// Application-level OTel metric instruments.
///
/// All instrument names follow the OTel semantic conventions spec in
/// `docs/reference/OBSERVABILITY.md`. Instruments are created once at startup
/// and shared across all request handlers via `Arc<AppMetrics>`.
pub struct AppMetrics {
    /// End-to-end HTTP request duration from first byte received to last byte sent.
    /// Name: `http.server.request.duration`, unit: `ms`.
    pub http_request_duration: Histogram<f64>,

    /// Number of requests currently being processed.
    /// Name: `http.server.active_requests`, unit: `{request}`.
    pub http_active_requests: UpDownCounter<i64>,

    /// Time from use-case entry to `MagicResult` return, excluding HTTP framing.
    /// Name: `app.analysis.duration`, unit: `ms`.
    pub analysis_duration: Histogram<f64>,

    /// Count of analysis failures broken down by `error.kind`.
    /// Name: `app.analysis.errors`, unit: `{error}`.
    pub analysis_errors: Counter<u64>,

    /// Duration of each background cleanup scan cycle.
    /// Name: `app.tempfile.cleanup.duration`, unit: `ms`.
    pub tempfile_cleanup_duration: Histogram<f64>,

    /// Total orphaned temp files removed across all scan cycles.
    /// Name: `app.tempfile.cleanup.removed`, unit: `{file}`.
    pub tempfile_cleanup_removed: Counter<u64>,
}

impl AppMetrics {
    /// Create all metric instruments from the given [`Meter`].
    ///
    /// The `meter` is typically obtained from the global OTel meter provider
    /// initialised in [`crate::infrastructure::telemetry::Telemetry::init`].
    /// In tests a noop meter from `opentelemetry::global::meter("test")` can
    /// be used — all instrument operations become no-ops.
    pub fn new(meter: &Meter) -> Self {
        Self {
            http_request_duration: meter
                .f64_histogram("http.server.request.duration")
                .with_description(
                    "End-to-end HTTP request duration from first byte received to last byte sent.",
                )
                .with_unit("ms")
                .build(),

            http_active_requests: meter
                .i64_up_down_counter("http.server.active_requests")
                .with_description("Number of requests currently being processed.")
                .with_unit("{request}")
                .build(),

            analysis_duration: meter
                .f64_histogram("app.analysis.duration")
                .with_description(
                    "Time from use-case entry to MagicResult return, excluding HTTP framing.",
                )
                .with_unit("ms")
                .build(),

            analysis_errors: meter
                .u64_counter("app.analysis.errors")
                .with_description("Count of analysis failures broken down by error.kind.")
                .with_unit("{error}")
                .build(),

            tempfile_cleanup_duration: meter
                .f64_histogram("app.tempfile.cleanup.duration")
                .with_description("Duration of each background cleanup scan cycle.")
                .with_unit("ms")
                .build(),

            tempfile_cleanup_removed: meter
                .u64_counter("app.tempfile.cleanup.removed")
                .with_description(
                    "Total orphaned temp files removed across all scan cycles.",
                )
                .with_unit("{file}")
                .build(),
        }
    }
}

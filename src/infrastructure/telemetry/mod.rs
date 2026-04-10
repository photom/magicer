pub mod metrics;

use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Holds the initialised OTel provider handles so they can be flushed and
/// shut down cleanly when the process exits.
///
/// All three signal pipelines (traces, metrics, logs) are initialised together
/// in [`Telemetry::init`] and flushed together in [`Telemetry::shutdown`].
pub struct Telemetry {
    tracer_provider: TracerProvider,
    meter_provider: SdkMeterProvider,
    logger_provider: LoggerProvider,
}

impl Telemetry {
    /// Initialise all three OTel signal pipelines and install a global
    /// `tracing` subscriber that bridges to them.
    ///
    /// The OTLP endpoint is read from `OTEL_EXPORTER_OTLP_ENDPOINT`; if the
    /// variable is absent the default `http://localhost:4317` is used.
    /// Exporters are lazy-connect — this function will **not** panic even if
    /// no collector is reachable.
    ///
    /// # Safety
    ///
    /// Calling `try_init().ok()` on the subscriber suppresses the error that
    /// occurs when a subscriber is already installed (e.g. in test harnesses).
    pub fn init() -> Self {
        let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string());

        let resource = Resource::new([
            KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
        ]);

        // --- Tracer provider ---
        let span_exporter = SpanExporter::builder()
            .with_tonic()
            .with_endpoint(&endpoint)
            .build()
            .unwrap_or_else(|e| {
                eprintln!("[otel] Failed to build span exporter: {e}; traces will be lost");
                SpanExporter::builder()
                    .with_tonic()
                    .build()
                    .expect("fallback span exporter must build")
            });

        let tracer_provider = TracerProvider::builder()
            .with_batch_exporter(span_exporter, Tokio)
            .with_resource(resource.clone())
            .build();

        let tracer = tracer_provider.tracer(env!("CARGO_PKG_NAME"));
        global::set_tracer_provider(tracer_provider.clone());

        // --- Meter provider ---
        let metric_exporter = MetricExporter::builder()
            .with_tonic()
            .with_endpoint(&endpoint)
            .build()
            .unwrap_or_else(|e| {
                eprintln!("[otel] Failed to build metric exporter: {e}; metrics will be lost");
                MetricExporter::builder()
                    .with_tonic()
                    .build()
                    .expect("fallback metric exporter must build")
            });

        let periodic_reader = PeriodicReader::builder(metric_exporter, Tokio).build();

        let meter_provider = SdkMeterProvider::builder()
            .with_reader(periodic_reader)
            .with_resource(resource.clone())
            .build();

        global::set_meter_provider(meter_provider.clone());

        // --- Logger provider ---
        let log_exporter = LogExporter::builder()
            .with_tonic()
            .with_endpoint(&endpoint)
            .build()
            .unwrap_or_else(|e| {
                eprintln!(
                    "[otel] Failed to build log exporter: {e}; OTel log records will be lost"
                );
                LogExporter::builder()
                    .with_tonic()
                    .build()
                    .expect("fallback log exporter must build")
            });

        let logger_provider = LoggerProvider::builder()
            .with_batch_exporter(log_exporter, Tokio)
            .with_resource(resource)
            .build();

        // Bridge: existing `tracing` events → OTel log records (no API rewrite needed).
        let log_bridge = OpenTelemetryTracingBridge::new(&logger_provider);

        // Assemble the tracing subscriber stack:
        //   1. EnvFilter (respects RUST_LOG)
        //   2. JSON formatter for human/machine-readable structured output
        //   3. OpenTelemetry trace layer (populates OTel spans)
        //   4. OpenTelemetry log bridge (emits OTel log records)
        //
        // `try_init().ok()` is intentional: in test environments a subscriber
        // may already be installed; we suppress the error rather than panicking.
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .with(OpenTelemetryLayer::new(tracer))
            .with(log_bridge)
            .try_init()
            .ok();

        Self {
            tracer_provider,
            meter_provider,
            logger_provider,
        }
    }

    /// Flush all in-flight telemetry and shut down the OTel pipelines.
    ///
    /// This should be called after the HTTP server exits and before the process
    /// terminates to ensure no spans, metrics, or log records are lost.
    pub fn shutdown(self) {
        if let Err(e) = self.tracer_provider.shutdown() {
            eprintln!("[otel] Tracer provider shutdown error: {e}");
        }
        if let Err(e) = self.meter_provider.shutdown() {
            eprintln!("[otel] Meter provider shutdown error: {e}");
        }
        if let Err(e) = self.logger_provider.shutdown() {
            eprintln!("[otel] Logger provider shutdown error: {e}");
        }
    }
}

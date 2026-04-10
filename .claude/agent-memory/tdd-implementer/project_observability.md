---
name: OTel implementation patterns
description: opentelemetry_sdk 0.27 / opentelemetry-otlp 0.27 API patterns, Telemetry struct layout, and tokio test deadlock avoidance for shutdown tests
type: project
---

## Correct API for opentelemetry_sdk 0.27 + opentelemetry-otlp 0.27

**TracerProvider**: `opentelemetry_sdk::trace::TracerProvider` (not `SdkTracerProvider`)
- Builder: `.with_batch_exporter(exporter, Tokio)` — requires `opentelemetry_sdk::runtime::Tokio`

**LoggerProvider**: `opentelemetry_sdk::logs::LoggerProvider` (not `SdkLoggerProvider`)
- Builder: `.with_batch_exporter(exporter, Tokio)`

**SdkMeterProvider**: `opentelemetry_sdk::metrics::SdkMeterProvider` (name is correct)
- Builder: `.with_reader(PeriodicReader::builder(exporter, Tokio).build())`
- NOT `.with_periodic_exporter()` — that method does not exist

**Resource**: `opentelemetry_sdk::Resource::new([KeyValue::new(...)])` (no `.builder()` method)

**OTLP exporters**: all use `.builder().with_tonic().with_endpoint(&endpoint).build()`
- `SpanExporter`, `MetricExporter`, `LogExporter` all from `opentelemetry_otlp`

## Telemetry::shutdown() deadlock avoidance in tests

`PeriodicReader::shutdown()` calls `futures_executor::block_on()` internally, which blocks
the calling thread. When called from a `#[tokio::test]` single-threaded runtime, this deadlocks
because the block_on future needs the tokio runtime (for PeriodicReader's background task) to make progress.

**Fix**: Use `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]` on all T-series tests.

**Timeout fix**: Set env vars BEFORE calling `Telemetry::init()` (which reads them at build time):
- `OTEL_EXPORTER_OTLP_TIMEOUT=1` (seconds, reduces per-RPC deadline)
- `OTEL_BSP_EXPORT_TIMEOUT=1000` (ms, batch span processor)
- `OTEL_METRIC_EXPORT_TIMEOUT=1000` (ms, periodic reader)
- `OTEL_BLRP_EXPORT_TIMEOUT=1000` (ms, batch log processor)

These are all set inside `unsafe { }` blocks because Rust 2024 requires explicit unsafe blocks for `set_var`/`remove_var` even in unsafe functions.

## Telemetry struct location
`src/infrastructure/telemetry/mod.rs` — `pub struct Telemetry`
`src/infrastructure/telemetry/metrics.rs` — `pub struct AppMetrics`

## Log bridge
`opentelemetry-appender-tracing` version: `0.27` with `log` feature.
Bridge type: `OpenTelemetryTracingBridge::new(&logger_provider)` added as a tracing layer.
All existing `tracing::info!` / `warn!` / `error!` calls automatically emit OTel log records.

**Why:** The bridge is lazy-connect; no rewriting of existing log calls is needed.
**How to apply:** Install the bridge as a layer in the `tracing_subscriber::registry()` stack.

use magicer::infrastructure::telemetry::Telemetry;
use serial_test::serial;

/// Set a short OTLP export timeout for test environments so that shutdown() doesn't
/// block for the full default 10-second timeout waiting for an unreachable OTLP collector.
///
/// # Safety
/// Tests are serialised via `#[serial]`; no concurrent threads read these vars.
unsafe fn set_short_export_timeouts() {
    unsafe {
        // OTLP per-RPC timeout in seconds
        std::env::set_var("OTEL_EXPORTER_OTLP_TIMEOUT", "1");
        // Batch span processor export timeout in milliseconds
        std::env::set_var("OTEL_BSP_EXPORT_TIMEOUT", "1000");
        // Metric periodic reader export timeout in milliseconds
        std::env::set_var("OTEL_METRIC_EXPORT_TIMEOUT", "1000");
        // Log batch processor export timeout in milliseconds
        std::env::set_var("OTEL_BLRP_EXPORT_TIMEOUT", "1000");
    }
}

/// # Safety
/// Tests are serialised via `#[serial]`; no concurrent threads read these vars.
unsafe fn clear_export_timeouts() {
    unsafe {
        std::env::remove_var("OTEL_EXPORTER_OTLP_TIMEOUT");
        std::env::remove_var("OTEL_BSP_EXPORT_TIMEOUT");
        std::env::remove_var("OTEL_METRIC_EXPORT_TIMEOUT");
        std::env::remove_var("OTEL_BLRP_EXPORT_TIMEOUT");
    }
}

/// T-01: Telemetry::init() does not panic without a running collector (OTLP is lazy-connect).
///
/// Uses a multi-threaded tokio runtime so that `PeriodicReader::shutdown()`'s internal
/// `futures_executor::block_on` call can make progress without deadlocking the runtime.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn t01_telemetry_init_does_not_panic_without_collector() {
    // SAFETY: test is serialised by #[serial]; no concurrent threads read these vars.
    unsafe { set_short_export_timeouts(); }
    let telemetry = Telemetry::init();
    telemetry.shutdown();
    unsafe { clear_export_timeouts(); }
}

/// T-02: OTEL_EXPORTER_OTLP_ENDPOINT env var is read and used.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn t02_otel_exporter_endpoint_env_var_is_consumed() {
    // SAFETY: test is serialised by #[serial]; no concurrent threads read these vars.
    unsafe {
        set_short_export_timeouts();
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://127.0.0.1:14317");
    }
    let telemetry = Telemetry::init();
    telemetry.shutdown();
    unsafe {
        std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        clear_export_timeouts();
    }
}

/// T-03: Telemetry::shutdown() completes without panic after init.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn t03_telemetry_shutdown_completes_without_panic() {
    // SAFETY: test is serialised by #[serial]; no concurrent threads read these vars.
    unsafe { set_short_export_timeouts(); }
    let telemetry = Telemetry::init();
    // shutdown() consumes self; if this returns we pass
    telemetry.shutdown();
    unsafe { clear_export_timeouts(); }
}

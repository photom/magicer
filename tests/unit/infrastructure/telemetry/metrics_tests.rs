use magicer::infrastructure::telemetry::metrics::AppMetrics;
use opentelemetry::KeyValue;

fn noop_meter() -> opentelemetry::metrics::Meter {
    opentelemetry::global::meter("test")
}

/// M-01: AppMetrics::new() with noop meter completes without panic.
#[test]
fn m01_app_metrics_new_with_noop_meter_does_not_panic() {
    let meter = noop_meter();
    let _metrics = AppMetrics::new(&meter);
}

/// M-02: analysis_errors counter accepts all error.kind values.
#[test]
fn m02_analysis_errors_counter_accepts_all_error_kind_values() {
    let meter = noop_meter();
    let metrics = AppMetrics::new(&meter);

    for kind in &[
        "timeout",
        "bad_request",
        "not_found",
        "internal",
        "insufficient_storage",
        "unauthorized",
        "forbidden",
    ] {
        metrics
            .analysis_errors
            .add(1, &[KeyValue::new("error.kind", *kind)]);
    }
}

/// M-03: analysis_duration histogram accepts analysis.type values.
#[test]
fn m03_analysis_duration_histogram_accepts_analysis_type_values() {
    let meter = noop_meter();
    let metrics = AppMetrics::new(&meter);

    for analysis_type in &["content_in_memory", "content_to_file", "path"] {
        metrics
            .analysis_duration
            .record(42.0, &[KeyValue::new("analysis.type", *analysis_type)]);
    }
}

/// M-04: http_active_requests up-down counter accepts http.method + http.route labels.
#[test]
fn m04_http_active_requests_up_down_counter_accepts_method_and_route() {
    let meter = noop_meter();
    let metrics = AppMetrics::new(&meter);

    let labels = [
        KeyValue::new("http.method", "POST"),
        KeyValue::new("http.route", "/v1/magic/content"),
    ];

    metrics.http_active_requests.add(1, &labels);
    metrics.http_active_requests.add(-1, &labels);
}

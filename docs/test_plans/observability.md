# Observability Test Plan <!-- omit in toc -->

- [Overview](#overview)
- [Test Case Status](#test-case-status)
- [AppMetrics (M)](#appmetrics-m)
- [Telemetry init/shutdown (T)](#telemetry-initshutdown-t)
- [Handler spans — presentation layer (H)](#handler-spans--presentation-layer-h)
- [Use case spans — application layer (U)](#use-case-spans--application-layer-u)
- [Structured log fields (L)](#structured-log-fields-l)

---

## Overview

This plan covers the OpenTelemetry observability implementation for the magicer service. The implementation follows the OTel data spec in `docs/reference/OBSERVABILITY.md` and the logging-principles skill. All three OTel signals (traces, metrics, logs) are exported via OTLP/gRPC to an OTel Collector.

**Stack**: Rust edition 2024, Axum 0.8.8, Tokio full, `tracing-opentelemetry` bridge.

---

## Test Case Status

| ID   | Category                        | Description                                                                 | Status   |
|------|---------------------------------|-----------------------------------------------------------------------------|----------|
| M-01 | AppMetrics                      | `AppMetrics::new()` with noop meter completes without panic                 | ✅ Done  |
| M-02 | AppMetrics                      | `analysis_errors` counter accepts all error.kind values                     | ✅ Done  |
| M-03 | AppMetrics                      | `analysis_duration` histogram accepts analysis.type values                  | ✅ Done  |
| M-04 | AppMetrics                      | `http_active_requests` up-down counter accepts method + route labels        | ✅ Done  |
| T-01 | Telemetry init/shutdown         | `Telemetry::init()` does not panic without a running collector              | ✅ Done  |
| T-02 | Telemetry init/shutdown         | `OTEL_EXPORTER_OTLP_ENDPOINT` env var is read and used                      | ✅ Done  |
| T-03 | Telemetry init/shutdown         | `Telemetry::shutdown()` completes without panic after init                  | ✅ Done  |
| H-01 | Handler spans (presentation)    | analyze_content span carries `analysis.filename`                            | ✅ Done  |
| H-02 | Handler spans (presentation)    | analyze_content sets `analysis.strategy = "in_memory"` for small payloads  | ✅ Done  |
| H-03 | Handler spans (presentation)    | analyze_content sets `analysis.strategy = "temp_file"` for large payloads  | ✅ Done  |
| H-04 | Handler spans (presentation)    | analyze_path span carries `analysis.filename`                               | ✅ Done  |
| U-01 | Use case spans (application)    | use_case.analyze_content_in_memory carries `request_id` and `analysis.type` | ✅ Done  |
| U-02 | Use case spans (application)    | use_case.analyze_content_to_file carries `request_id` and `analysis.type`  | ✅ Done  |
| U-03 | Use case spans (application)    | use_case.analyze_path carries `request_id` and `analysis.type`             | ✅ Done  |
| L-01 | Structured log fields           | INFO log at server bind carries `server.addr` and `server.backlog` as structured fields | ✅ Done  |
| L-02 | Structured log fields           | Cleanup WARN/INFO logs carry `file.name` field only — not the full resolved path | ✅ Done  |

---

## AppMetrics (M)

### M-01: AppMetrics::new() with noop meter

**Location**: `tests/unit/infrastructure/telemetry/metrics_tests.rs`

**Precondition**: A noop `opentelemetry::metrics::Meter` is obtained from the global noop provider.

**Steps**:
1. Obtain a noop meter via `opentelemetry::global::meter("test")`.
2. Call `AppMetrics::new(&meter)`.

**Expected**: No panic. All instrument fields are populated.

---

### M-02: analysis_errors counter accepts all error.kind values

**Location**: `tests/unit/infrastructure/telemetry/metrics_tests.rs`

**Precondition**: `AppMetrics::new()` with noop meter succeeds (M-01).

**Steps**:
1. For each error.kind value: `timeout`, `bad_request`, `not_found`, `internal`, `insufficient_storage`, `unauthorized`, `forbidden`:
   - Build a `KeyValue` slice with `"error.kind"` set to the value.
   - Call `metrics.analysis_errors.add(1, &[...])`.

**Expected**: Each `add()` call completes without panic.

---

### M-03: analysis_duration histogram accepts analysis.type values

**Location**: `tests/unit/infrastructure/telemetry/metrics_tests.rs`

**Precondition**: `AppMetrics::new()` with noop meter succeeds (M-01).

**Steps**:
1. For each analysis.type: `content_in_memory`, `content_to_file`, `path`:
   - Call `metrics.analysis_duration.record(42.0, &[KeyValue::new("analysis.type", value)])`.

**Expected**: Each `record()` call completes without panic.

---

### M-04: http_active_requests up-down counter accepts method + route labels

**Location**: `tests/unit/infrastructure/telemetry/metrics_tests.rs`

**Precondition**: `AppMetrics::new()` with noop meter succeeds (M-01).

**Steps**:
1. Build labels: `http.method = "POST"`, `http.route = "/v1/magic/content"`.
2. Call `metrics.http_active_requests.add(1, &[...])`.
3. Call `metrics.http_active_requests.add(-1, &[...])`.

**Expected**: Both `add()` calls complete without panic.

---

## Telemetry init/shutdown (T)

### T-01: Telemetry::init() does not panic without a running collector

**Location**: `tests/unit/infrastructure/telemetry/telemetry_tests.rs`

**Precondition**: No OTel Collector is running. `OTEL_EXPORTER_OTLP_ENDPOINT` is unset or points to a non-existent endpoint.

**Steps**:
1. Call `Telemetry::init()`.

**Expected**: No panic. OTLP exporters are lazy-connect and do not fail at init time.

---

### T-02: OTEL_EXPORTER_OTLP_ENDPOINT env var is read and used

**Location**: `tests/unit/infrastructure/telemetry/telemetry_tests.rs`

**Precondition**: `serial_test` crate is used to serialize env-var mutation tests.

**Steps**:
1. Set `OTEL_EXPORTER_OTLP_ENDPOINT` to `"http://127.0.0.1:14317"` via `std::env::set_var`.
2. Call `Telemetry::init()`.
3. Call `telemetry.shutdown()`.
4. Restore env var.

**Expected**: Init and shutdown complete without panic. (Endpoint connectivity is not asserted — only that the env var is consumed without panic.)

---

### T-03: Telemetry::shutdown() completes without panic after init

**Location**: `tests/unit/infrastructure/telemetry/telemetry_tests.rs`

**Precondition**: `Telemetry::init()` succeeds (T-01).

**Steps**:
1. Call `Telemetry::init()`.
2. Call `telemetry.shutdown()`.

**Expected**: No panic. Providers are flushed and dropped cleanly.

---

## Handler spans — presentation layer (H)

### H-01: analyze_content span carries analysis.filename

**Location**: `tests/unit/presentation/http/handlers/magic_handlers_tests.rs`

**Verification method**: Compile-time assertion. The `#[tracing::instrument]` attribute on `analyze_content` declares `analysis.filename = %query.filename` as a field. If the field name or variable is wrong, the crate will not compile. The test verifies compilation succeeds and the handler is callable.

**Steps**:
1. Confirm `analyze_content` handler compiles with `#[tracing::instrument(name = "handler.analyze_content", fields(analysis.filename = %query.filename, ...))]`.
2. Exercise the handler via `axum-test`.

**Expected**: Compilation succeeds; handler returns a valid response.

---

### H-02: analyze_content sets analysis.strategy = "in_memory" for small non-chunked payloads

**Location**: `tests/unit/presentation/http/handlers/magic_handlers_tests.rs`

**Verification method**: Behavioral — send a small non-chunked payload via `axum-test` and confirm a 200 response (exercising the `in_memory` code path). The `analysis.strategy` field is set via `Span::current().record()` in the handler.

**Steps**:
1. Create a test app with a fake magic repo.
2. POST a small payload without `Transfer-Encoding: chunked`.
3. Assert HTTP 200.

**Expected**: 200 OK; the `in_memory` code path is exercised.

---

### H-03: analyze_content sets analysis.strategy = "temp_file" for chunked/large payloads

**Location**: `tests/unit/presentation/http/handlers/magic_handlers_tests.rs`

**Verification method**: Behavioral — send a chunked payload via `axum-test` with `Transfer-Encoding: chunked` header and confirm a 200 response (exercising the `temp_file` code path).

**Steps**:
1. Create a test app with a fake magic repo.
2. POST a payload with `Transfer-Encoding: chunked` header.
3. Assert HTTP 200.

**Expected**: 200 OK; the `temp_file` code path is exercised.

---

### H-04: analyze_path span carries analysis.filename

**Location**: `tests/unit/presentation/http/handlers/magic_handlers_tests.rs`

**Verification method**: Compile-time assertion. The `#[tracing::instrument]` attribute on `analyze_path` declares `analysis.filename = %query.filename` as a field.

**Steps**:
1. Confirm `analyze_path` handler compiles with the `analysis.filename` field declared.
2. Exercise the handler via `axum-test`.

**Expected**: Compilation succeeds; handler returns a valid response.

---

## Use case spans — application layer (U)

### U-01: use_case.analyze_content_in_memory carries request_id and analysis.type

**Location**: `tests/unit/application/analyze_content_tests.rs`

**Verification method**: Compile-time assertion. The `#[tracing::instrument]` attribute on `analyze_in_memory` declares `request_id = %request_id` and `analysis.type = "content_in_memory"` as fields. The existing test suite exercises this method.

**Steps**:
1. Confirm `analyze_in_memory` compiles with the declared fields.
2. Run the existing `test_analyze_in_memory_success` test.

**Expected**: Compilation succeeds; existing test passes.

---

### U-02: use_case.analyze_content_to_file carries request_id and analysis.type

**Location**: `tests/unit/application/analyze_content_tests.rs`

**Verification method**: Compile-time assertion. The `#[tracing::instrument]` attribute on `analyze_to_temp_file` declares `request_id = %request_id` and `analysis.type = "content_to_file"` as fields.

**Steps**:
1. Confirm `analyze_to_temp_file` compiles with the declared fields.
2. Run the existing `test_analyze_to_temp_file_success` test.

**Expected**: Compilation succeeds; existing test passes.

---

### U-03: use_case.analyze_path carries request_id and analysis.type

**Location**: `tests/unit/application/analyze_path_tests.rs`

**Verification method**: Compile-time assertion. The `#[tracing::instrument]` attribute on `AnalyzePathUseCase::execute` declares `request_id = %request_id` and `analysis.type = "path"` as fields.

**Steps**:
1. Confirm `execute` compiles with the declared fields.
2. Run the existing `analyze_path` tests.

**Expected**: Compilation succeeds; existing tests pass.

---

## Structured log fields (L)

### L-01: server.addr and server.backlog are structured fields

**Location**: `src/main.rs` (verified by inspection)

**Verification method**: Code inspection confirms the listen log uses structured field syntax:
```rust
tracing::info!(server.addr = %addr, server.backlog = config.server.backlog, "Server listening");
```
Not a format string like `"Listening on {} (backlog: {})", addr, backlog`.

**Expected**: Fields appear as separate key-value pairs in the structured log output, not interpolated into the message string.

---

### L-02: Cleanup logs carry file.name field — not the full resolved path

**Location**: `src/main.rs` (verified by inspection)

**Verification method**: Code inspection confirms the cleanup task logs use:
```rust
let file_name = entry.path().file_name().unwrap_or_default().to_string_lossy();
tracing::warn!(file.name = %file_name, error = %e, "Failed to remove orphaned temp file");
tracing::info!(file.name = %file_name, "Removed orphaned temp file");
```
Never the full `path` variable.

**Expected**: Only `file.name` (filename component) is attached; the sandbox-resolved absolute path is never logged.

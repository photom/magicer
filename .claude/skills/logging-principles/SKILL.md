---
name: logging-principles
description: Guidelines for structured logging, telemetry, data privacy, and log levels. Use to ensure consistent observability.
---

## 1. Data Privacy & Masking

- **PII Protection**: Never log plain-text passwords, phone numbers, or highly sensitive personal data.
- **Masking Rules**: 
  - Mask values for keys ending in `password`.
  - Mask values for keys starting with `sec_`.
  - Use common logging utility methods or filters to replace sensitive data with masks (e.g., `***`).
- **Array Handling**: Avoid logging indeterminate or large arrays in a single line. If the count exceeds human-readable limits, omit it from the logs.

## 2. Modern Observability & Telemetry

- **Structured Logging**: Use JSON or other structured formats (e.g., key-value pairs) to facilitate integration with Cloud services and SIEM (Security Information and Event Management) platforms.
- **Audit & Security**: Ensure audit/access logs comply with security requirements, including tampering detection and long-term retention.
- **Unified Telemetry**: Integrate Logs, Metrics, and Traces as a unified telemetry strategy to achieve high observability and real-time system insights.

## 🖥️ Server-Side Logging Guidelines

### Operational Context

Logs must empower operators.

**Required Context for Milters:**
Always include these on the same line if available:
- `Connection ID` / `Session ID`
- `Client IP` / `Hostname`

**Metrics (KGI/KPI/KAI)**: Output logs related to defined business and activity indicators (e.g., messages processed, spam detected, virus detected, quarantine actions) to enable continuous improvement cycles.

### Log Levels & Timing

| Level | Timing / Stage | Required Context |
| :--- | :--- | :--- |
| **INFO** | Milter Start/End, Connection Accepted, External API Calls | Connection ID, Client IP, Message-ID, Action taken (Accept, Reject, Discard), External request status. |
| **WARN** | External API timeouts (non-fatal), Malformed headers | Connection ID, Error details, Endpoint/Params for external calls, Specific malformed data (non-sensitive). |
| **ERROR** | Unhandled exceptions, Database connection failures, Critical external service failures causing message tempfails (`4xx`) | Connection ID, Message-ID (if known), Error content, Stack trace (on a separate line). |
| **DEBUG** | Detailed parsing of headers/body, raw bytes received, internal state transitions | Connection ID, Class/Method names, specific header keys being checked, return values of internal functions. |

## 💻 Client-Side Logging Guidelines

*(Included for completeness if building client admin interfaces for the milter)*

**Debugging & Support**
For on-premise or client applications, ensure logs provide necessary information for IT administrators and support teams.

| Level | Timing | Required Context |
| :--- | :--- | :--- |
| **DEBUG** | Public Method Entry/Exit | Resource ID, Class/Method names, Arguments, Return values. |
| **INFO** | App Start/End, Validation Success, Successful mutations, External Comms | Version, Args, Input values, Status codes, Request/Response bodies. |
| **WARN** | External 5xx errors (Permanent/Input errors), Config validation failure | Resource ID, Endpoint, Error line/item, Failed input values. |
| **ERROR** | Unhandled exceptions, External 4xx errors (Temporary/System errors), App crash | Resource ID, Error content, Stack trace (on a separate line). |

## 3. OpenTelemetry Integration (Rust / Axum)

### Instrumentation Type

Rust has no auto-instrumentation agent. The canonical approach is **manual instrumentation via the `tracing-opentelemetry` bridge**. This reuses the existing `tracing` ecosystem and exports all three OTel signals — Traces, Metrics, and Logs — through a single OTLP pipeline. No existing log calls need to be rewritten.

### The Three Signals

#### Traces

Instrument at each Clean Architecture layer boundary to produce a natural span hierarchy per request:

| Layer | Span scope | Key attributes to attach |
| :--- | :--- | :--- |
| Presentation (handlers) | Per-request root span | `http.method`, `http.route`, `request_id` |
| Application (use cases) | Per use-case span | `use_case`, `result` |
| Infrastructure (repos/services) | Per I/O operation span | operation type, resource identifier |

- Use `tower-http`'s `TraceLayer` to create root spans automatically for each HTTP request.
- Propagate `request_id` as a span attribute at every layer, not just at the handler level.
- Span names follow the pattern `{layer}.{operation}` in snake_case (e.g., `handler.analyze_content`, `use_case.analyze_content`, `repo.identify_mime`).

#### Metrics

Emit the following minimum set at the infrastructure/presentation boundary:

| Metric | Type | Key labels |
| :--- | :--- | :--- |
| HTTP request duration | Histogram | `method`, `route`, `status_code` |
| Active in-flight requests | UpDownCounter | `method`, `route` |
| Analysis operation duration | Histogram | `analysis_type` (`content` / `path`) |
| Analysis errors | Counter | `error_kind` |
| Temp file cleanup duration | Histogram | — |

Follow OTel semantic conventions for naming: `http.server.*` for HTTP-level metrics, `app.*` for domain-specific metrics.

#### Logs

Wire `opentelemetry-appender-tracing` as a `tracing::Layer` so all existing `info!` / `warn!` / `error!` calls automatically emit OTel log records. Do **not** duplicate calls using a separate OTel log API — the bridge handles the mapping.

### Initialization

Initialize all three pipelines in `main.rs` before starting the server and shut them down gracefully on signal. Export to an **OTel Collector** via OTLP/gRPC. Configure the endpoint via the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable — never hardcode it.

Attach `service.name` and `service.version` as resource attributes on the tracer and meter providers so all signals are correlated by service identity.

### Naming Conventions

- **Span names**: `{layer}.{operation}` in snake_case
- **Metric names**: OTel semantic conventions first (`http.server.*`), then `app.*` namespace for domain metrics
- **Attribute keys**: lowercase dot-notation (e.g., `request_id`, `http.route`, `error.kind`)

### Security Notes for OTel

- Never include request/response bodies in span attributes — treat them as potential PII.
- Apply the same masking rules from §1 before attaching any auth-related values to spans.
- Use TLS for the OTLP channel in production (`OTEL_EXPORTER_OTLP_CERTIFICATE`).

## Related Skills

- For Rust implementation specifics, refer to the **rust-master** skill.
- For ensuring third-party interactions are properly logged and masked, see **external-integration**.
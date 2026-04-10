---
name: magicer architecture patterns
description: AppState constructor signature, module organization, test helper patterns, and domain conventions
type: project
---

## AppState constructor (as of observability PR)

```rust
AppState::new(
    magic_repo: Arc<dyn MagicRepository>,
    sandbox: Arc<dyn SandboxService>,
    temp_storage: Arc<dyn TempStorageService>,
    auth_service: Arc<dyn AuthenticationService>,
    config: Arc<ServerConfig>,
    metrics: Arc<AppMetrics>,   // added in observability PR
) -> Self
```

All test call sites and bench call sites must pass `metrics`. 
For tests, use: `Arc::new(AppMetrics::new(&opentelemetry::global::meter("test")))`

## Module layout

```
src/infrastructure/
  telemetry/
    mod.rs      -- Telemetry struct (init + shutdown)
    metrics.rs  -- AppMetrics struct (all OTel instruments)
  auth/
  config/
  filesystem/
  magic/
```

```
tests/unit/
  infrastructure/
    telemetry/
      mod.rs
      metrics_tests.rs    -- M-01..M-04
      telemetry_tests.rs  -- T-01..T-03
    auth/ config/ filesystem/ magic/
  application/
  presentation/
```

## Error kind mapping (ApplicationError → "error.kind" string)

| ApplicationError variant | error.kind string |
|---|---|
| Timeout | "timeout" |
| BadRequest(_) | "bad_request" |
| NotFound(_) | "not_found" |
| InternalError(_) / UnprocessableEntity(_) | "internal" |
| InsufficientStorage(_) | "insufficient_storage" |
| Unauthorized(_) | "unauthorized" |
| Forbidden(_) | "forbidden" |

Defined in `src/presentation/http/handlers/magic_handlers.rs` as `fn error_kind()`.

## RequestId implements Display

Added `impl fmt::Display for RequestId` in `src/domain/value_objects/request_id.rs` to support
`%request_id` in `#[tracing::instrument]` field declarations.

## Rust 2024 edition notes

- `std::env::set_var` and `remove_var` are unsafe in Rust 2024 — require `unsafe {}` blocks
- `unsafe fn` bodies require explicit `unsafe {}` for unsafe calls (`unsafe_op_in_unsafe_fn`)

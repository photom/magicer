# Class Diagrams Index

## Overview

This directory contains comprehensive class diagrams for each Rust file in the magicer project, organized by DDD (Domain-Driven Design) layers.

## Directory Structure

```
docs/reference/
├── domain/              # Domain layer (pure business logic)
├── application/         # Application layer (use cases)
├── infrastructure/      # Infrastructure layer (implementations)
└── presentation/        # Presentation layer (HTTP/Axum)
```

---

## Domain Layer

**Location:** `docs/reference/domain/`

Pure business logic with zero external dependencies. Contains entities, value objects, repository traits, and domain errors.

### Value Objects

| File | Document | Description |
|------|----------|-------------|
| `filename.rs` | [filename.md](domain/filename.md) | `WindowsCompatibleFilename` - validated filename (max 310 chars, no `/`, no `\0`) |
| `file_path.rs` | [file_path.md](domain/file_path.md) | `RelativePath` - relative path validation (no `..`, no leading `/`) |
| `request_id.rs` | [request_id.md](domain/request_id.md) | `RequestId` - UUID v4 wrapper for distributed tracing |
| `mime_type.rs` | [mime_type.md](domain/mime_type.md) | `MimeType` - RFC 6838 MIME type validation |
| `credentials.rs` | [credentials.md](domain/credentials.md) | `BasicAuthCredentials` - HTTP Basic Auth credentials with validation |

### Entities

| File | Document | Description |
|------|----------|-------------|
| `magic_result.rs` | [magic_result.md](domain/magic_result.md) | `MagicResult` - file magic analysis result entity (with identity) |

### Repositories (Traits)

| File | Document | Description |
|------|----------|-------------|
| `magic_repository.rs` | [magic_repository.md](domain/magic_repository.md) | `MagicRepository` trait - file magic analysis operations |

### Services (Traits)

| File | Document | Description |
|------|----------|-------------|
| `authentication_service.rs` | [authentication_service.md](domain/authentication_service.md) | `AuthenticationService` trait - credential verification (constant-time) |

### Errors

| File | Document | Description |
|------|----------|-------------|
| `errors/` | [errors.md](domain/errors.md) | `DomainError`, `ValidationError`, `MagicError` - comprehensive domain errors |

---

## Application Layer

**Location:** `docs/reference/application/`

Orchestrates domain objects through use cases. Depends only on domain layer.

### Use Cases

| File | Document | Description |
|------|----------|-------------|
| `analyze_content.rs` | [analyze_content.md](application/analyze_content.md) | `AnalyzeContentUseCase` - analyze uploaded binary content |
| `analyze_path.rs` | [analyze_path.md](application/analyze_path.md) | `AnalyzePathUseCase` - analyze file by relative path (with sandbox validation) |
| `health_check.rs` | [health_check.md](application/health_check.md) | `HealthCheckUseCase` - simple liveness check (no dependencies) |

### DTOs (Data Transfer Objects)

| File | Document | Description |
|------|----------|-------------|
| `dtos/` | [dtos.md](application/dtos.md) | `AnalyzeContentRequest`, `AnalyzePathRequest`, `MagicResponse` - use case boundaries |

### Errors

| File | Document | Description |
|------|----------|-------------|
| `errors/` | [errors.md](application/errors.md) | `ApplicationError` - HTTP-friendly semantic errors |

---

## Infrastructure Layer

**Location:** `docs/reference/infrastructure/`

Implements domain traits with external dependencies (libmagic, filesystem, configuration).

### Magic Analysis

| File | Document | Description |
|------|----------|-------------|
| `libmagic_repository.rs` | [libmagic_repository.md](infrastructure/libmagic_repository.md) | `LibmagicRepository` - implements `MagicRepository` using libmagic FFI |

### Authentication

| File | Document | Description |
|------|----------|-------------|
| `basic_auth_service.rs` | [basic_auth_service.md](infrastructure/basic_auth_service.md) | `BasicAuthService` - implements `AuthenticationService` with constant-time comparison |

### Filesystem

| File | Document | Description |
|------|----------|-------------|
| `sandbox.rs` | [sandbox.md](infrastructure/sandbox.md) | `PathSandbox` - path validation and sandbox boundary enforcement |
| `temp_file_handler.rs` | [temp_file_handler.md](infrastructure/temp_file_handler.md) | `TempFileHandler` - temporary file management with RAII cleanup |

### Configuration

| File | Document | Description |
|------|----------|-------------|
| `server_config.rs` | [server_config.md](infrastructure/server_config.md) | `ServerConfig` - TOML configuration loading with env var overrides |

---

## Presentation Layer

**Location:** `docs/reference/presentation/`

HTTP server implementation with Axum framework.

### Handlers

| File | Document | Description |
|------|----------|-------------|
| `magic_handlers.rs` | [magic_handlers.md](presentation/magic_handlers.md) | `analyze_content_handler`, `analyze_path_handler` - magic analysis endpoints |
| `health_handlers.rs` | [health_handlers.md](presentation/health_handlers.md) | `ping_handler` - health check endpoint (GET /v1/ping) |

### Middleware

| File | Document | Description |
|------|----------|-------------|
| `middleware/` | [middleware.md](presentation/middleware.md) | Request ID, Authentication, Timeout, Error Handler middleware |

### Router & State

| File | Document | Description |
|------|----------|-------------|
| `router.rs` | [router.md](presentation/router.md) | Axum router configuration, route definitions, middleware composition |
| `app_state.rs` | [app_state.md](presentation/app_state.md) | `AppState` - shared application state (use cases, services, config) |

### Responses

| File | Document | Description |
|------|----------|-------------|
| `responses/` | [responses.md](presentation/responses.md) | `MagicResponse`, `ErrorResponse` - HTTP response types |

---

## Layer Dependencies

```
Presentation Layer (HTTP/Axum)
  ↓ depends on
Application Layer (Use Cases)
  ↓ depends on
Domain Layer (Entities, Value Objects, Traits)
  ↑ implemented by
Infrastructure Layer (Concrete Implementations)
```

**Allowed Imports:**
- ✅ Presentation → Application, Domain
- ✅ Application → Domain
- ✅ Infrastructure → Domain
- ✅ Domain → std only

**Forbidden:**
- ❌ Domain → Infrastructure
- ❌ Domain → Application
- ❌ Application → Infrastructure
- ❌ Application → Presentation

---

## Quick Reference by Feature

### File Magic Analysis
- **Domain:** [magic_repository.md](domain/magic_repository.md), [magic_result.md](domain/magic_result.md)
- **Application:** [analyze_content.md](application/analyze_content.md), [analyze_path.md](application/analyze_path.md)
- **Infrastructure:** [libmagic_repository.md](infrastructure/libmagic_repository.md)
- **Presentation:** [magic_handlers.md](presentation/magic_handlers.md)

### Authentication
- **Domain:** [authentication_service.md](domain/authentication_service.md), [credentials.md](domain/credentials.md)
- **Infrastructure:** [basic_auth_service.md](infrastructure/basic_auth_service.md)
- **Presentation:** [middleware.md](presentation/middleware.md) (Auth middleware)

### Path Validation & Security
- **Domain:** [file_path.md](domain/file_path.md), [filename.md](domain/filename.md)
- **Application:** [analyze_path.md](application/analyze_path.md)
- **Infrastructure:** [sandbox.md](infrastructure/sandbox.md)

### HTTP Server
- **Presentation:** [router.md](presentation/router.md), [app_state.md](presentation/app_state.md), [middleware.md](presentation/middleware.md)

### Error Handling
- **Domain:** [errors.md](domain/errors.md)
- **Application:** [errors.md](application/errors.md)
- **Presentation:** [responses.md](presentation/responses.md)

---

## Diagram Conventions

All class diagrams follow these conventions:

### Mermaid Syntax
- **Class Diagrams:** Structure and relationships
- **Sequence Diagrams:** Interaction flows
- **Flow Charts:** Process flows and decision trees
- **State Diagrams:** Lifecycle and state transitions

### Color Coding
- 🟢 **Green (`#90EE90`):** Success paths, valid states
- 🔴 **Red (`#FFB6C1`):** Error states, validation failures
- 🟡 **Yellow (`#FFEB3B`):** Warnings, intermediate states
- 🔵 **Blue (`#E3F2FD`):** Domain layer
- 🟠 **Orange (`#FFF3E0`):** Application layer
- 🟢 **Light Green (`#E8F5E9`):** Infrastructure layer
- 🟣 **Purple (`#F3E5F5`):** Presentation layer

### Relationship Types
- `*--` : Composition (has-a, owns)
- `o--` : Aggregation (has-a, shares)
- `-->` : Association (uses)
- `..>` : Dependency (depends on)
- `..|>` : Realization (implements)
- `--|>` : Inheritance (extends)

---

## Usage Guidelines

### For Implementation
1. Read the class diagram for the file you're implementing
2. Follow the TDD approach described in [implement-conventions.mdc](../../.cursor/rules/implement-conventions.mdc)
3. Refer to [design-conventions.mdc](../../.cursor/rules/design-conventions.mdc) for architectural patterns
4. Check dependencies and ensure layer boundaries are respected

### For Testing
1. Each class diagram includes usage examples and test scenarios
2. Follow the testing strategy in [TESTING_STRATEGY.md](TESTING_STRATEGY.md)
3. Ensure coverage matches requirements:
   - Domain: 100%
   - Application: 95%+
   - Infrastructure: 80%+
   - Presentation: Integration tests

### For Review
1. Verify implementation matches class diagram structure
2. Check that dependencies flow in the correct direction
3. Ensure error handling follows documented patterns
4. Validate that invariants are maintained

---

## Maintenance

When adding new files:

1. Create class diagram in appropriate layer directory
2. Follow existing diagram format and conventions
3. Update this index with new entry
4. Document all relationships and dependencies
5. Include usage examples and test scenarios

---

## Related Documentation

- [Architecture](../explanation/ARCHITECTURE.md) - System architecture and design decisions
- [Testing Strategy](TESTING_STRATEGY.md) - Testing approach and coverage
- [Project Structure](PROJECT_STRUCTURE.md) - Codebase organization
- [Design Conventions](../../.cursor/rules/design-conventions.mdc) - DDD and Clean Architecture patterns
- [Implementation Conventions](../../.cursor/rules/implement-conventions.mdc) - TDD workflow

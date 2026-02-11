# Router Class Diagram <!-- omit in toc -->

- [Overview](#overview)
- [Class Diagram](#class-diagram)
- [Route Configuration](#route-configuration)
- [Route Table](#route-table)
- [Middleware Stack](#middleware-stack)
- [Router Construction and Logic](#router-construction-and-logic)
- [State Management and Injection](#state-management-and-injection)
- [Router Implementation Pattern](#router-implementation-pattern)
- [Server Initialization and Lifecycle](#server-initialization-and-lifecycle)
- [API Integration Specification](#api-integration-specification)
- [Design Rationale](#design-rationale)

---

## Overview

The Axum router configuration defines all HTTP routes, applies middleware layers, and wires dependencies.

## Class Diagram

```mermaid
classDiagram
    class Router {
        +route(path: &str, method_router: MethodRouter) Self
        +layer(layer: L) Self
        +with_state(state: S) Router
    }
    
    class AppState {
        +analyze_content_use_case: Arc~AnalyzeContentUseCase~
        +analyze_path_use_case: Arc~AnalyzePathUseCase~
        +health_check_use_case: Arc~HealthCheckUseCase~
        +auth_service: Arc~dyn AuthenticationService~
    }
    
    class MethodRouter {
        +get(handler: H) Self
        +post(handler: H) Self
        +put(handler: H) Self
        +delete(handler: H) Self
    }
    
    Router *-- AppState : contains
    Router --> MethodRouter : uses
    
    note for Router "Axum router\nRoute definitions\nMiddleware composition\nDependency injection"
```

## Route Configuration

```mermaid
graph TD
    Root["/"] --> V1["/v1"]
    V1 --> Ping["/v1/ping<br/>GET<br/>No Auth"]
    V1 --> Content["/v1/magic/content<br/>POST<br/>Auth Required"]
    V1 --> Path["/v1/magic/path<br/>POST<br/>Auth Required"]
    
    style Ping fill:#E8F5E9
    style Content fill:#FFE0B2
    style Path fill:#FFE0B2
```

## Route Table

| Path | Method | Auth | Handler | Description |
|------|--------|------|---------|-------------|
| `/v1/ping` | GET | ❌ No | `ping_handler` | Health check / liveness probe |
| `/v1/magic/content` | POST | ✅ Yes | `analyze_content_handler` | Analyze uploaded binary content |
| `/v1/magic/path` | POST | ✅ Yes | `analyze_path_handler` | Analyze file by relative path |

## Middleware Stack

```mermaid
flowchart TD
    Request[HTTP Request] --> Layer1[Request ID Layer<br/>Generate/extract UUID]
    Layer1 --> Layer2[Timeout Layer<br/>30 second timeout]
    Layer2 --> Layer3[Auth Layer<br/>Verify credentials]
    Layer3 --> Layer4[Error Handler Layer<br/>Map errors to HTTP]
    Layer4 --> Handler[Route Handler]
    Handler --> Response[HTTP Response]
    
    style Request fill:#E3F2FD
    style Response fill:#E8F5E9
```

## Router Construction and Logic

The router is constructed using a declarative approach, defining the relationship between HTTP paths, methods, and their corresponding handlers. The construction process involves:
1. **Endpoint Definition**: Mapping GET and POST methods to specific handler functions.
2. **Middleware Application**: Layering cross-cutting concerns in a specific execution order.
3. **State Injection**: Providing the application state to all handlers through dependency injection.

## State Management and Injection

The application state is encapsulated in a central structure and wrapped in an atomic reference counter. This ensures that all handlers have thread-safe access to necessary dependencies like use cases, services, and configuration. The state is injected once during router initialization and is automatically extracted by Axum for each incoming request.

## Router Implementation Pattern

The implementation follows a modular pattern:
- **Health Routes**: Defined as public endpoints without authentication middleware.
- **Magic Analysis Routes**: Grouped together with mandatory authentication and specific timeouts.
- **Global Layers**: Universal middleware applied to all routes, such as request ID generation and global error handling.

## Server Initialization and Lifecycle

The server lifecycle starts in the main entry point, which coordinates several steps:
1. **Config Loading**: Reads and validates settings from files and environment variables.
2. **Infrastructure Setup**: Initializes the libmagic repository, authentication service, and sandbox.
3. **Use Case Creation**: Instantiates the business logic components with their infrastructure dependencies.
4. **Router Configuration**: Builds the router with all routes, middleware, and state.
5. **Startup**: Binds the TCP listener and begins serving requests asynchronously.

## API Integration Specification

The API follows a standardized RESTful structure:
- **Ping**: A public GET endpoint at `/v1/ping` for health monitoring.
- **Content Analysis**: A protected POST endpoint at `/v1/magic/content` that accepts binary data and a filename query parameter.
- **Path Analysis**: A protected POST endpoint at `/v1/magic/path` that accepts a JSON payload with a relative file path.

All protected endpoints require HTTP Basic Authentication and return results in a consistent JSON format.

## Design Rationale

- **Clean Separation**: Routes, middleware, and handlers are clearly separated
- **Dependency Injection**: All dependencies passed via `AppState`
- **Type Safety**: Axum extractors provide compile-time safety
- **Testability**: Easy to create test servers with mock dependencies
- **Composability**: Middleware layers compose cleanly
- **Security**: Authentication middleware applied globally (with public endpoint exceptions)
- **Observability**: Request ID middleware enables distributed tracing
- **Resilience**: Timeout middleware prevents hung requests

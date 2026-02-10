# Design Summary - Linux File Magic API Server <!-- omit in toc -->

- [1. Overview](#1-overview)
- [2. Specification Documents](#2-specification-documents)
- [3. Design Documents](#3-design-documents)
- [4. Additional Specifications Needed](#4-additional-specifications-needed)
- [5. Implementation Readiness](#5-implementation-readiness)
- [6. Next Steps](#6-next-steps)

---

## 1. Overview

This document summarizes the complete design and specification for the Linux File Magic API server built with Rust and Axum. All specifications use Mermaid diagrams and descriptive text rather than code examples, focusing on architectural decisions and directory models.

---

## 2. Specification Documents

### **2.1. API Specification**

**File:** `api/v1/openapi.yaml`

**Content:**
- OpenAPI 3.0.3 specification
- Three endpoints: POST /v1/magic/content, POST /v1/magic/path, GET /v1/ping
- HTTP Basic Authentication required for magic endpoints
- Request/response schemas with validation rules
- Error response formats

**Key Constraints:**
- Max request body: 100MB
- Filename max length: 310 characters (Windows-compatible on Linux)
- Relative paths only (no traversal)
- UUID v4 request IDs in all responses

### **2.2. HTTP Server Specification**

**File:** `docs/HTTP_SERVER.md`

**Content:**
- Concurrency limits (1000 concurrent connections, 1024 backlog)
- Timeout specifications (60s read/write, 30s analysis, 75s keep-alive)
- Resource constraints (100MB body, 8KB URI, 16KB headers)
- File system sandbox requirements
- Graceful shutdown behavior (10s drain period)
- Request correlation via UUID

---

## 3. Design Documents

### **3.1. Architecture Design**

**File:** `docs/ARCHITECTURE.md`

**Content:**
- Clean Architecture with 4 layers (domain, application, infrastructure, presentation)
- Layer dependency rules and component diagrams
- Axum HTTP server architecture with middleware stack
- Security architecture (authentication, path validation, request limits)
- Error handling flow with domain → application → presentation mapping
- Concurrency architecture using Tokio runtime and blocking pool
- Observability strategy with tracing and request correlation
- Configuration management and graceful shutdown
- Technology stack and dependencies

**Key Diagrams:**
- High-level architecture diagram
- Layer dependency graph
- Component architecture (domain, application, infrastructure)
- Request processing flow
- Middleware architecture
- Security flow (authentication, path validation)
- Error transformation chain
- Concurrency runtime model
- Timeout hierarchy
- Connection limits

### **3.2. Testing Strategy**

**File:** `docs/TESTING_STRATEGY.md`

**Content:**
- Test organization and pyramid (70% unit, 25% integration, 5% E2E)
- Testing by layer (domain, application, infrastructure, presentation)
- Property-based testing for validation logic
- Integration testing strategy
- End-to-end workflow tests
- Security testing (path traversal, authentication, timing attacks)
- Performance testing (benchmarks, load, stress, endurance)
- Coverage requirements (80% minimum, 90% target)
- Test infrastructure and utilities
- Continuous integration pipeline

**Key Diagrams:**
- Test organization structure
- Test pyramid distribution
- Domain layer test coverage
- Application layer mocking strategy
- Infrastructure testing with real dependencies
- HTTP layer integration testing
- Property-based test flow
- Security test categories
- Performance test types
- CI pipeline stages

### **3.3. Project Structure**

**File:** `docs/PROJECT_STRUCTURE.md`

**Content:**
- Complete directory layout with purpose of each directory
- Source code organization by Clean Architecture layer
- Domain layer structure (entities, value objects, repositories, services, errors)
- Application layer structure (use cases, DTOs)
- Infrastructure layer structure (magic, auth, filesystem, config)
- Presentation layer structure (handlers, middleware, extractors, responses, router, state)
- Configuration file descriptions
- Documentation organization
- Testing directory structure
- File naming conventions

**Key Diagrams:**
- Module dependency graph
- Project organization principles summary

### **3.4. Deployment Guide**

**File:** `docs/DEPLOYMENT.md`

**Content:**
- System requirements (OS, dependencies, resources)
- Build configuration (development, production, cross-compilation)
- Runtime configuration structure
- Environment variable mapping
- Docker deployment (Dockerfile, docker-compose)
- Systemd service configuration
- Monitoring and health checks
- Security hardening (firewall, TLS, SELinux)
- Performance tuning (kernel parameters, process limits)
- Logging configuration

---

## 4. Additional Specifications Needed

Based on the current design, the following additional specifications might be beneficial:

### **4.1. API Usage Guide (Tutorial)**

**Suggested File:** `docs/API_TUTORIAL.md`

**Purpose:** Learning-oriented guide showing:
- Getting started with the API
- Authentication setup
- Making your first request
- Handling responses
- Common use cases with examples
- Troubleshooting common issues

**Status:** Not yet created (would complement OpenAPI spec)

### **4.2. Operational Runbook (How-to Guide)**

**Suggested File:** `docs/OPERATIONS.md`

**Purpose:** Task-oriented guide for:
- Starting and stopping the server
- Updating configuration
- Rotating credentials
- Investigating errors via request_id
- Handling high load situations
- Disaster recovery procedures

**Status:** Partially covered in DEPLOYMENT.md, could be expanded

### **4.3. Error Reference**

**Suggested File:** `docs/ERROR_REFERENCE.md`

**Purpose:** Complete catalog of:
- All error codes and HTTP status mappings
- Error message formats
- Common causes for each error
- Resolution steps
- Related request_id tracking

**Status:** Partially covered in OpenAPI spec and ARCHITECTURE.md

### **4.4. Development Guide**

**Suggested File:** `docs/DEVELOPMENT.md`

**Purpose:** How-to guide for developers:
- Setting up development environment
- Running tests locally
- Adding new endpoints
- Implementing new use cases
- Debugging techniques
- Contributing guidelines

**Status:** Not yet created

---

## 5. Implementation Readiness

### **5.1. Complete Specifications**

✅ **API Contract:** OpenAPI specification defines all endpoints, schemas, and behaviors

✅ **Architecture:** Clean Architecture design with clear layer boundaries and component responsibilities

✅ **Testing:** Comprehensive testing strategy from unit to E2E with coverage requirements

✅ **Deployment:** Production-ready deployment configurations and procedures

✅ **Project Structure:** Clear directory organization following architectural principles

### **5.2. Design Decisions Made**

| Decision Area | Choice | Rationale |
|--------------|--------|-----------|
| Web Framework | Axum | Type-safe, async-first, composable middleware |
| Architecture | Clean Architecture | Clear boundaries, testability, maintainability |
| Authentication | HTTP Basic Auth | Simple, standard, sufficient for API use case |
| Path Security | Multi-layer validation | Defense in depth against traversal attacks |
| Concurrency | Tokio + blocking pool | Async I/O with CPU-bound work isolation |
| Observability | Tracing + JSON logs | Structured logging with request correlation |
| Configuration | TOML + env vars | Type-safe config with environment overrides |
| Testing | Pyramid with property tests | Fast feedback with comprehensive coverage |
| **Request Body Handling** | **Stream-direct to temp** | **Constant 64KB memory, early error detection, scalability** |

#### Request Body Handling: Stream-Direct Approach (Adopted)

**Decision:** Stream HTTP request bodies directly to temporary files instead of buffering in memory.

**Alternatives Considered:**

| Approach | Memory Usage | Complexity | Validation Timing | Decision |
|----------|--------------|------------|-------------------|----------|
| **Buffer-first** | High (100MB per request) | Simple | Before processing | ❌ Rejected |
| **Stream-direct** | Low (64KB buffer only) | Complex | After partial write | ✅ Adopted |
| **Hybrid** | Medium (varies by size) | Very Complex | Mixed | ⏸️ Deferred to v2 |

**Detailed Comparison:**

```
┌─────────────────────────────────────────────────────────────────┐
│ Buffer-First Approach (Rejected)                                │
├─────────────────────────────────────────────────────────────────┤
│ Flow: Client → Axum Buffer (100MB) → Handler → Temp File       │
│                                                                  │
│ Pros:                                                            │
│  ✓ Simple implementation (Axum default)                        │
│  ✓ Validation before any disk I/O                              │
│  ✓ Fast for small files (no disk)                              │
│                                                                  │
│ Cons:                                                            │
│  ✗ 100MB memory per request                                    │
│  ✗ OOM risk under load (300 req × 100MB = 30GB)               │
│  ✗ Memory scales with file size                                │
│  ✗ Writes twice (buffer → temp)                                │
│  ✗ Error detection after full upload                           │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ Stream-Direct Approach (Adopted) ✅                             │
├─────────────────────────────────────────────────────────────────┤
│ Flow: Client → Stream (64KB) → Temp File → Analysis            │
│                                                                  │
│ Pros:                                                            │
│  ✓ Constant 64KB memory (regardless of file size)              │
│  ✓ No OOM risk (300 req × 64KB = 19MB)                        │
│  ✓ Predictable resource usage                                  │
│  ✓ Writes once (direct to temp)                                │
│  ✓ Early error detection (disk full during upload)             │
│  ✓ Better backpressure handling                                │
│  ✓ Production-ready scalability                                │
│                                                                  │
│ Cons:                                                            │
│  ✗ Complex implementation (custom streaming)                   │
│  ✗ All files use temp dir (even small)                         │
│  ✗ Validation after partial write (requires cleanup)           │
└─────────────────────────────────────────────────────────────────┘
```

**Key Metrics (300 Concurrent 100MB Requests):**

| Metric | Buffer-First | Stream-Direct | Improvement |
|--------|--------------|---------------|-------------|
| Peak Memory | 30GB | 19MB | **1579x less** |
| Memory/Request | 100MB | 64KB | **1563x less** |
| Writes per File | 2 (buffer + temp) | 1 (direct) | 50% fewer |
| Error Detection | After upload | During upload | Earlier |
| OOM Risk | High | None | Eliminated |

**Adoption Rationale:**

1. **Memory Efficiency (Critical Priority):**
   - 64KB constant memory eliminates OOM risk
   - Enables 1579x more concurrent requests with same memory
   - Predictable resource usage simplifies capacity planning

2. **Production Scalability (High Priority):**
   - Memory does not scale with file size
   - Supports higher connection limits
   - Better behavior under load spikes

3. **Error Detection (Medium Priority):**
   - Disk full detected immediately
   - Saves bandwidth (client stops uploading)
   - Faster feedback to clients

4. **Resource Efficiency (Medium Priority):**
   - Single write instead of double write
   - Better disk I/O utilization
   - Reduced memory → disk transfer overhead

**Trade-offs Accepted:**

1. **Implementation Complexity:**
   - Custom streaming logic required
   - Error handling more complex (partial write cleanup)
   - More test scenarios needed
   - **Acceptable:** One-time development cost for long-term scalability

2. **All Files Use Temp Directory:**
   - Even small files (< 10MB) go through temp file
   - Additional disk I/O for small files
   - **Acceptable:** Small files still fast (<100ms), scalability more important

3. **Validation After Partial Write:**
   - Auth/validation happens after streaming starts
   - Invalid requests waste some disk I/O
   - Requires cleanup of partial files
   - **Acceptable:** Pre-flight auth check mitigates, cleanup is automatic

**Implementation Priority:** **High** - Foundational for production scalability

**Status:** Adopted as primary design (not alternative)

**Future Enhancement:** Hybrid approach (buffer small files < 10MB) deferred to v2.0 for optimization

### **5.3. Open Questions**

None critical for implementation. The design is complete and ready for coding.

Optional enhancements for future consideration:
- Metrics collection endpoint (Prometheus format)
- Rate limiting implementation
- API key authentication (in addition to basic auth)
- Request caching for repeated analysis
- Webhook notifications for async analysis

---

## 6. Next Steps

### **6.1. Implementation Phase**

**Recommended Order:**

1. **Domain Layer** (`src/domain/`)
   - Start with value objects (WindowsCompatibleFilename, RelativePath, RequestId, MimeType)
   - Implement entity (MagicResult)
   - Define repository and service traits
   - Create domain errors

2. **Infrastructure Layer** (`src/infrastructure/`)
   - Implement LibmagicRepository (wraps libmagic C library)
   - Implement BasicAuthService
   - Create PathSandbox utility
   - Build configuration loader

3. **Application Layer** (`src/application/`)
   - Implement AnalyzeContentUseCase
   - Implement AnalyzePathUseCase
   - Implement HealthCheckUseCase
   - Define DTOs and application errors

4. **Presentation Layer** (`src/presentation/`)
   - Create AppState structure
   - Implement middleware (request_id, auth, timeout, error_handler)
   - Implement handlers (magic_handlers, health_handlers)
   - Build router configuration
   - Create main.rs entry point

5. **Testing**
   - Write unit tests alongside each component
   - Add property-based tests for validators
   - Implement integration tests for HTTP endpoints
   - Create E2E workflow tests
   - Add security tests

6. **Documentation**
   - Add inline documentation (rustdoc comments)
   - Create README with quick start
   - Update docs as needed based on implementation details

### **6.2. Validation Checkpoints**

After each layer implementation:
- Run tests and verify coverage meets targets
- Verify layer dependencies are correct (no forbidden imports)
- Check that architectural principles are maintained
- Review error handling and propagation
- Validate logging and observability

### **6.3. Deployment Preparation**

Before production deployment:
- Load testing to verify concurrency limits
- Security audit (dependency check, penetration testing)
- Performance benchmarking
- Documentation review
- Create deployment checklist based on DEPLOYMENT.md

---

## Summary

**Current State:** Complete design with comprehensive specifications

**Documentation Coverage:**
- ✅ API specification (OpenAPI)
- ✅ Server behavior specification
- ✅ Detailed architecture design
- ✅ Comprehensive testing strategy
- ✅ Project structure definition
- ✅ Deployment procedures
- ⚠️ Optional: API tutorial, operations runbook, error reference, development guide

**Ready for Implementation:** Yes

**Next Action:** Begin implementation starting with domain layer, following the recommended order above.

All designs use Mermaid diagrams and descriptive documentation rather than code examples, as requested. The specifications are detailed enough to guide implementation without prescribing specific implementation details.

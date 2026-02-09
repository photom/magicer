# magicer

Linux File Magic API Server - A REST API for file type identification using libmagic, built with Rust and Axum.

## Quick Start

**Status:** Design phase complete, implementation ready to begin.

This is a learning project ("etude") implementing a production-grade REST API that provides file magic analysis through HTTP endpoints.

## Documentation

Documentation organized by the [Diátaxis framework](https://diataxis.fr/):

```
docs/
├── 📖 tutorials/          Learning-oriented (not yet created)
├── 🛠️ how-to-guides/     Problem-oriented
├── 📚 reference/          Information-oriented
└── 💡 explanation/        Understanding-oriented
```

### Quick Access

- 🗺️ **[Documentation Map](docs/DOCUMENTATION_MAP.md)** - Navigate by role or task

### Key Documents

| Category | Document | Purpose |
|----------|----------|---------|
| How-To | [Deployment](docs/how-to-guides/DEPLOYMENT.md) | Production deployment procedures |
| Reference | [OpenAPI Spec](api/v1/openapi.yaml) | Complete REST API contract |
| Reference | [Configuration](docs/reference/CONFIG.md) | Complete config.toml reference |
| Reference | [HTTP Server](docs/reference/HTTP_SERVER.md) | Server behavior and limits |
| Reference | [Project Structure](docs/reference/PROJECT_STRUCTURE.md) | Codebase organization |
| Reference | [Testing Strategy](docs/reference/TESTING_STRATEGY.md) | Testing approach |
| Explanation | [Architecture](docs/explanation/ARCHITECTURE.md) | System design and decisions |
| Explanation | [Design Summary](docs/explanation/DESIGN_SUMMARY.md) | Complete design overview |
| Explanation | [libmagic FFI](docs/explanation/LIBMAGIC_FFI.md) | Rust-C FFI integration |

## API Endpoints

| Method | Path | Purpose | Auth Required |
|--------|------|---------|---------------|
| POST | `/v1/magic/content` | Analyze uploaded binary content | ✅ Yes |
| POST | `/v1/magic/path` | Analyze file by relative path | ✅ Yes |
| GET | `/v1/ping` | Health check | ❌ No |

**Authentication:** HTTP Basic Auth

**API Documentation:**
- **Specification:** [api/v1/openapi.yaml](api/v1/openapi.yaml)
- **Interactive Docs:** https://photom.github.io/magicer/

### Automated API Documentation

HTML documentation is automatically generated from the OpenAPI spec via GitHub Actions:

- **URL:** https://photom.github.io/magicer/
- **Trigger:** Push to `main` branch when `openapi.yaml` changes
- **Validation:** Lints OpenAPI spec before generation

**Manual Generation:**

```bash
# Install Redocly CLI
npm install -g @redocly/cli

# Validate specification
redocly lint api/v1/openapi.yaml

# Generate HTML documentation
redocly build-docs api/v1/openapi.yaml --output docs.html
```

## Technology Stack

- **Language:** Rust (edition 2021)
- **Web Framework:** Axum 0.7
- **Async Runtime:** Tokio
- **File Magic:** libmagic bindings
- **Architecture:** Clean Architecture with DDD principles

## Development Status

- ✅ **Design Phase:** Complete
  - API specification
  - Architecture design
  - Testing strategy
  - Project structure
  - Deployment procedures
  
- ⏳ **Implementation Phase:** Ready to start
  - Domain layer → Infrastructure → Application → Presentation
  - See [Design Summary](docs/explanation/DESIGN_SUMMARY.md) for implementation order

## Quick Reference

**Core Concepts:**
- **Clean Architecture:** 4 layers (domain, application, infrastructure, presentation)
- **File Magic:** Uses libmagic for file type identification
- **Security:** Multi-layer path validation, basic authentication, 100MB request limit
- **Observability:** Request ID tracing, structured logging

**Key Constraints:**
- Linux x86_64 only
- Max request body: 100MB
- Max filename length: 310 characters
- Relative paths only (no traversal allowed)
- Connection limit: 1000 concurrent, 1024 backlog

## License

Apache 2.0

## References

- [Diátaxis Documentation Framework](https://diataxis.fr/)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Domain-Driven Design](https://www.domainlanguage.com/ddd/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Axum Framework](https://docs.rs/axum/latest/axum/)

# MagicRepository Trait Class Diagram

## Overview

The `MagicRepository` trait defines the contract for file magic analysis operations. Implementations exist in the infrastructure layer.

## Class Diagram

```mermaid
classDiagram
    class MagicRepository {
        <<trait>>
        +analyze_buffer(data: &[u8], filename: &str) Result~MagicResult, DomainError~
        +analyze_file(path: &Path) Result~MagicResult, DomainError~
    }
    
    class MagicResult {
        <<entity>>
    }
    
    class DomainError {
        <<enumeration>>
    }
    
    class Path {
        <<std::path>>
    }
    
    MagicRepository ..> MagicResult : returns
    MagicRepository ..> DomainError : returns
    MagicRepository ..> Path : uses
    
    note for MagicRepository "Trait bounds: Send + Sync\nImplemented in infrastructure\nDomain defines interface only"
```

## Trait Definition

The MagicRepository trait defines two methods for file magic analysis. The analyze_buffer method accepts a byte slice and filename, returning either a MagicResult or DomainError. The analyze_file method accepts a Path reference and returns the same result type. Both methods require Send and Sync trait bounds to enable thread-safe usage in async contexts.

## Method Specifications

### analyze_buffer

Analyzes binary data from any source (memory, mmap, static).

```mermaid
flowchart TD
    Input["Input: &[u8] + filename"] --> Validate{Valid input?}
    Validate -->|No| ErrValidation[DomainError::ValidationError]
    Validate -->|Yes| Analyze[Perform magic analysis]
    Analyze --> Success{Analysis OK?}
    Success -->|No| ErrMagic[DomainError::MagicError]
    Success -->|Yes| Result([Ok MagicResult])
    
    style Result fill:#90EE90
    style ErrValidation fill:#FFB6C1
    style ErrMagic fill:#FFB6C1
```

**Parameters:**
- `data: &[u8]` - Byte slice from any source (memory, mmap, network)
- `filename: &str` - Original filename for context (libmagic uses this)

**Returns:**
- `Ok(MagicResult)` - Analysis successful
- `Err(DomainError::ValidationError)` - Invalid input (empty data, invalid filename)
- `Err(DomainError::MagicError)` - Analysis failed (libmagic error)

**Accepts data from:**

| Source Type | Example | Compatible |
|-------------|---------|------------|
| In-Memory Buffer | `Vec<u8>`, `Bytes` | ✅ |
| Memory-Mapped File | `Mmap::as_ref()` | ✅ |
| Static Data | `&[u8; N]` | ✅ |
| Network Buffer | HTTP request body | ✅ |

### analyze_file

Analyzes file by filesystem path (libmagic opens file internally).

```mermaid
flowchart TD
    Input["Input: &Path"] --> CheckPath{Path valid?}
    CheckPath -->|No| ErrValidation[DomainError::ValidationError]
    CheckPath -->|Yes| CheckExists{File exists?}
    CheckExists -->|No| ErrNotFound[DomainError::FileNotFound]
    CheckExists -->|Yes| Analyze[Perform magic analysis]
    Analyze --> Success{Analysis OK?}
    Success -->|No| ErrMagic[DomainError::MagicError]
    Success -->|Yes| Result([Ok MagicResult])
    
    style Result fill:#90EE90
    style ErrValidation fill:#FFB6C1
    style ErrNotFound fill:#FFB6C1
    style ErrMagic fill:#FFB6C1
```

**Parameters:**
- `path: &Path` - Absolute path to file (must be within sandbox)

**Returns:**
- `Ok(MagicResult)` - Analysis successful
- `Err(DomainError::ValidationError)` - Invalid path
- `Err(DomainError::FileNotFound)` - File doesn't exist
- `Err(DomainError::MagicError)` - Analysis failed

## Error Mapping

```mermaid
graph TD
    LibmagicErr[libmagic errors] --> Infrastructure[Infrastructure Layer]
    Infrastructure --> Map[Error Mapping]
    Map --> DomainErr[DomainError]
    
    IOErr[std::io errors] --> Infrastructure
    ValidationErr[Validation errors] --> DomainErr
    
    DomainErr --> Application[Application Layer]
    Application --> AppErr[ApplicationError]
    AppErr --> Presentation[Presentation Layer]
    Presentation --> HTTP[HTTP Status Codes]
    
    style DomainErr fill:#FFEB3B
    style HTTP fill:#90EE90
```

## Implementation Requirements

Implementations MUST:

1. **Thread Safety**: Implement `Send + Sync` for multi-threaded async runtime
2. **Error Mapping**: Convert all infrastructure errors to `DomainError`
3. **Validation**: Validate inputs before calling external dependencies
4. **No Panics**: Return `Result` for all error conditions
5. **Idempotency**: Same input produces same output (deterministic)

## Trait Bounds

| Bound | Purpose |
|-------|---------|
| Send | Enables transfer between threads |
| Sync | Enables sharing between threads via Arc |

These bounds are required for Tokio async runtime and Axum state sharing, ensuring the repository can be safely used in concurrent contexts.

## Usage Patterns

### In Application Layer

Use cases depend on the MagicRepository trait through generic type parameters or trait objects wrapped in Arc. The use case calls repository methods and maps domain errors to application errors.

### In Infrastructure Layer

Concrete implementations like LibmagicRepository implement the MagicRepository trait. The implementation holds the necessary state (such as a libmagic cookie handle) and provides concrete logic for both analyze_buffer and analyze_file methods using libmagic FFI bindings.

## Dependency Injection

```mermaid
sequenceDiagram
    participant Main
    participant Infra as LibmagicRepository
    participant App as AnalyzeContentUseCase
    participant Domain as MagicRepository Trait
    
    Main->>Infra: new()
    Main->>App: new(Arc<LibmagicRepository>)
    App->>Domain: analyze_buffer() via trait
    Domain->>Infra: concrete implementation
    Infra-->>Domain: Result<MagicResult>
    Domain-->>App: Result<MagicResult>
    
    note over Domain: Depends on abstraction<br/>not concrete type
```

## Design Rationale

- **Dependency Inversion**: Domain defines interface, infrastructure implements
- **Testability**: Easy to mock in tests (trait object or generic)
- **Unified Interface**: Single `&[u8]` parameter handles all buffer sources
- **Error Abstraction**: Domain errors hide infrastructure details
- **Thread Safety**: `Send + Sync` enables async/parallel execution
- **Simplicity**: Two methods cover all use cases (buffer vs file)

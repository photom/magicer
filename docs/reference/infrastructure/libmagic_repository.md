# LibmagicRepository Implementation Class Diagram

## Overview

The `LibmagicRepository` implements the `MagicRepository` trait using libmagic FFI bindings for file type analysis.

## Class Diagram

```mermaid
classDiagram
    class LibmagicRepository {
        -Cookie cookie
        +new() Result~Self, InfrastructureError~
        +analyze_buffer(data: &[u8], filename: &str) Result~MagicResult, DomainError~
        +analyze_file(path: &Path) Result~MagicResult, DomainError~
    }
    
    class MagicRepository {
        <<trait>>
        +analyze_buffer(data: &[u8], filename: &str) Result~MagicResult, DomainError~
        +analyze_file(path: &Path) Result~MagicResult, DomainError~
    }
    
    class Cookie {
        <<external::magic>>
        +load(database: &[&str]) Result~(), Error~
        +file(path: &str) Result~String, Error~
        +buffer(data: &[u8]) Result~String, Error~
        +set_flags(flags: CookieFlags) Result~(), Error~
    }
    
    class MagicResult {
        <<entity>>
    }
    
    class DomainError {
        <<enumeration>>
    }
    
    LibmagicRepository ..|> MagicRepository : implements
    LibmagicRepository *-- Cookie : contains
    LibmagicRepository ..> MagicResult : produces
    LibmagicRepository ..> DomainError : returns
    
    note for LibmagicRepository "Infrastructure implementation\nUses libmagic FFI\nThread-safe via Arc"
```

## Cookie Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Created: Cookie::open()
    Created --> Configured: set_flags(MIME_TYPE | MIME_ENCODING)
    Configured --> Loaded: load(default database)
    Loaded --> Ready: Ready for analysis
    Ready --> Analyzing: analyze_buffer/file
    Analyzing --> Ready: Return result
    Ready --> [*]: Drop
    
    note right of Ready
        Cookie is reusable
        Thread-safe when wrapped in Arc
    end note
```

## Properties

| Property | Type | Description |
|----------|------|-------------|
| `cookie` | `Cookie` | libmagic cookie handle (FFI) |

## Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `new` | - | `Result<Self, InfrastructureError>` | Initialize libmagic with default database |
| `analyze_buffer` | `&self, data: &[u8], filename: &str` | `Result<MagicResult, DomainError>` | Analyze binary buffer |
| `analyze_file` | `&self, path: &Path` | `Result<MagicResult, DomainError>` | Analyze file by path |

## Initialization Flow

```mermaid
flowchart TD
    Start([new]) --> OpenCookie[Cookie::open]
    OpenCookie --> Success{Success?}
    Success -->|No| ErrOpen[Err: Cannot open cookie]
    Success -->|Yes| SetFlags[Set flags: MIME_TYPE + MIME_ENCODING]
    SetFlags --> FlagSuccess{Success?}
    FlagSuccess -->|No| ErrFlags[Err: Cannot set flags]
    FlagSuccess -->|Yes| LoadDB[Load default database]
    LoadDB --> DBSuccess{Success?}
    DBSuccess -->|No| ErrDB[Err: Cannot load database]
    DBSuccess -->|Yes| Ready[Ok LibmagicRepository]
    
    style Ready fill:#90EE90
    style ErrOpen fill:#FFB6C1
    style ErrFlags fill:#FFB6C1
    style ErrDB fill:#FFB6C1
```

## analyze_buffer Implementation

```mermaid
sequenceDiagram
    participant Caller
    participant Repo as LibmagicRepository
    participant Cookie as magic::Cookie
    participant Blocking as tokio::task::spawn_blocking
    participant Parser
    
    Caller->>Repo: analyze_buffer(data, filename)
    Repo->>Repo: Validate inputs
    Repo->>Blocking: Spawn blocking task
    Blocking->>Cookie: buffer(data)
    Cookie-->>Blocking: Result<String, Error>
    alt Success
        Blocking->>Parser: Parse MIME type
        Parser->>Parser: Parse description
        Parser->>Parser: Parse encoding
        Parser-->>Blocking: MagicResult
        Blocking-->>Repo: Ok(MagicResult)
        Repo-->>Caller: Ok(MagicResult)
    else Error
        Blocking-->>Repo: Err(magic::Error)
        Repo->>Repo: Map to DomainError::MagicError
        Repo-->>Caller: Err(DomainError)
    end
```

## analyze_file Implementation

```mermaid
sequenceDiagram
    participant Caller
    participant Repo as LibmagicRepository
    participant Cookie as magic::Cookie
    participant Blocking as tokio::task::spawn_blocking
    participant FS as Filesystem
    
    Caller->>Repo: analyze_file(path)
    Repo->>Repo: Validate path
    Repo->>FS: Check file exists
    alt File not found
        FS-->>Repo: Not found
        Repo-->>Caller: Err(DomainError::FileNotFound)
    else File exists
        Repo->>Blocking: Spawn blocking task
        Blocking->>Cookie: file(path_str)
        Cookie-->>Blocking: Result<String, Error>
        alt Success
            Blocking-->>Repo: Ok(MagicResult)
            Repo-->>Caller: Ok(MagicResult)
        else Error
            Blocking-->>Repo: Err(magic::Error)
            Repo->>Repo: Map to DomainError
            Repo-->>Caller: Err(DomainError)
        end
    end
```

## Error Mapping

```mermaid
graph TD
    LibmagicErr["magic::Error"] --> Map[Error Mapping]
    
    Map --> NotFound[ErrorKind::NotFound]
    NotFound --> DomainNotFound[DomainError::FileNotFound]
    
    Map --> Permission[ErrorKind::PermissionDenied]
    Permission --> DomainPermission[DomainError::PermissionDenied]
    
    Map --> Encoding[Encoding detection failed]
    Encoding --> MagicErr[DomainError::MagicError::AnalysisFailed]
    
    Map --> Unknown[Unknown error]
    Unknown --> MagicErr
    
    style DomainNotFound fill:#FFB6C1
    style DomainPermission fill:#FFB6C1
    style MagicErr fill:#FF9800
```

## Cookie Flags

| Flag | Purpose | Example Output |
|------|---------|----------------|
| `MIME_TYPE` | Return MIME type | `text/plain` |
| `MIME_ENCODING` | Return character encoding | `us-ascii` |
| `SYMLINK` | Follow symlinks | Analyze target file |
| `ERROR` | Continue on errors | Partial results |
| `NO_CHECK_*` | Skip specific checks | Performance optimization |

## libmagic Output Parsing

```
Raw output: "text/plain; charset=utf-8"

Parsing:
1. Split by '; ' → ["text/plain", "charset=utf-8"]
2. First part → MIME type: "text/plain"
3. Second part (if exists) → Parse encoding: "utf-8"
4. If no encoding → None
```

```mermaid
flowchart LR
    Raw["text/plain; charset=utf-8"] --> Split[Split by '; ']
    Split --> Mime["MIME: text/plain"]
    Split --> Encoding["encoding: utf-8"]
    
    Raw2["application/pdf"] --> NoSplit[No '; ' found]
    NoSplit --> Mime2["MIME: application/pdf"]
    NoSplit --> NoEnc["encoding: None"]
    
    style Mime fill:#90EE90
    style Encoding fill:#90EE90
    style Mime2 fill:#90EE90
    style NoEnc fill:#FFEB3B
```

## Usage Example

```rust
// Initialization
let repository = Arc::new(LibmagicRepository::new()?);

// Analyze buffer (in-memory data)
let data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
let result = repository.analyze_buffer(&data, "image.png")?;
assert_eq!(result.mime_type().as_str(), "image/png");

// Analyze file (by path)
let path = Path::new("/sandbox/documents/report.pdf");
let result = repository.analyze_file(path)?;
assert_eq!(result.mime_type().type_part(), "application");

// Error handling
match repository.analyze_file(path) {
    Ok(result) => println!("Type: {}", result.description()),
    Err(DomainError::FileNotFound(p)) => eprintln!("File not found: {}", p),
    Err(DomainError::PermissionDenied(p)) => eprintln!("Access denied: {}", p),
    Err(DomainError::MagicError(e)) => eprintln!("Analysis failed: {:?}", e),
    Err(e) => eprintln!("Unexpected error: {:?}", e),
}
```

## Thread Safety

```rust
// LibmagicRepository is Send + Sync
// Can be shared across threads via Arc
let repository = Arc::new(LibmagicRepository::new()?);

// Use in async context (Axum handler)
async fn handler(
    State(repo): State<Arc<LibmagicRepository>>,
    data: Bytes,
) -> Result<Json<MagicResponse>, StatusCode> {
    // spawn_blocking because libmagic is CPU-bound
    let result = tokio::task::spawn_blocking(move || {
        repo.analyze_buffer(&data, "file.bin")
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    
    Ok(Json(MagicResponse::from(result)))
}
```

## Performance Considerations

| Aspect | Strategy | Rationale |
|--------|----------|-----------|
| **CPU-bound** | `spawn_blocking` | libmagic analysis is synchronous and CPU-intensive |
| **Cookie Reuse** | Single cookie per instance | Avoid repeated initialization overhead |
| **Thread Safety** | `Cookie` is thread-safe | Can be shared via `Arc` |
| **Database Loading** | Load once at startup | Database loading is expensive |
| **Buffer Size** | No internal buffering | Accept any `&[u8]` size (limited by HTTP layer) |

## Database Configuration

```rust
impl LibmagicRepository {
    pub fn new() -> Result<Self, InfrastructureError> {
        let cookie = Cookie::open(CookieFlags::MIME_TYPE | CookieFlags::MIME_ENCODING)?;
        
        // Load default database
        cookie.load::<String>(&[])?;
        
        Ok(Self { cookie })
    }
    
    pub fn with_database(database_path: &str) -> Result<Self, InfrastructureError> {
        let cookie = Cookie::open(CookieFlags::MIME_TYPE | CookieFlags::MIME_ENCODING)?;
        
        // Load custom database
        cookie.load(&[database_path])?;
        
        Ok(Self { cookie })
    }
}
```

## Testing

```rust
#[test]
fn test_analyze_png_buffer() {
    let repo = LibmagicRepository::new().unwrap();
    let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    
    let result = repo.analyze_buffer(&png_header, "test.png").unwrap();
    assert_eq!(result.mime_type().as_str(), "image/png");
}

#[test]
fn test_analyze_file_not_found() {
    let repo = LibmagicRepository::new().unwrap();
    let result = repo.analyze_file(Path::new("/nonexistent/file.txt"));
    
    assert!(matches!(result, Err(DomainError::FileNotFound(_))));
}
```

## Design Rationale

- **Trait Implementation**: Implements domain-defined `MagicRepository` trait
- **Error Mapping**: Converts FFI errors to domain errors at boundary
- **Async-Ready**: Uses `spawn_blocking` for CPU-bound libmagic calls
- **Thread-Safe**: Cookie is thread-safe, shareable via `Arc`
- **Dependency Inversion**: Domain defines interface, infrastructure provides concrete implementation
- **Clean Separation**: No domain logic, only infrastructure concerns

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
    [*] --> Created: Open cookie
    Created --> Configured: Set flags for MIME type and encoding
    Configured --> Loaded: Load default database
    Loaded --> Ready: Ready for analysis
    Ready --> Analyzing: Analyze buffer or file
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

## Usage Scenarios

### Initialization

The LibmagicRepository is initialized and wrapped in an Arc for thread-safe sharing across the application. The initialization may fail if the libmagic library cannot be opened or the database cannot be loaded.

### Analyze Buffer

When analyzing in-memory data such as a PNG file header (8 bytes starting with hex values 0x89, 0x50, 0x4E, 0x47), the analyze_buffer method is called with the byte slice and a filename hint. The method returns a MagicResult with the detected MIME type "image/png".

### Analyze File by Path

When analyzing a file by its filesystem path such as "/sandbox/documents/report.pdf", the analyze_file method is called with the path. The method returns a MagicResult with the MIME type detected from the file, such as "application/pdf".

### Error Handling

When analyzing a file, various errors can occur. FileNotFound indicates the file doesn't exist at the specified path. PermissionDenied indicates insufficient permissions to read the file. MagicError indicates the analysis failed (e.g., corrupted file or unsupported format). Other domain errors may occur for unexpected conditions.

## Thread Safety

LibmagicRepository implements Send and Sync traits, allowing it to be safely shared across threads when wrapped in an Arc. This is essential for use in async web servers where multiple requests may be processed concurrently.

When used in async contexts such as Axum handlers, the repository is accessed through Arc from the application state. Because libmagic operations are CPU-bound and synchronous, they should be executed using spawn_blocking to avoid blocking the async runtime. The blocking task receives the Arc-wrapped repository, performs the analysis, and returns the result. Any errors from the spawned task or the analysis itself are mapped to appropriate HTTP status codes.

## Performance Considerations

| Aspect | Strategy | Rationale |
|--------|----------|-----------|
| **CPU-bound** | `spawn_blocking` | libmagic analysis is synchronous and CPU-intensive |
| **Cookie Reuse** | Single cookie per instance | Avoid repeated initialization overhead |
| **Thread Safety** | `Cookie` is thread-safe | Can be shared via `Arc` |
| **Database Loading** | Load once at startup | Database loading is expensive |
| **Buffer Size** | No internal buffering | Accept any `&[u8]` size (limited by HTTP layer) |

## Database Configuration

### Default Database

The new constructor opens a libmagic cookie with flags for MIME type and encoding detection, then loads the default system magic database. The default database is typically located at a system-specific path like "/usr/share/misc/magic.mgc". If either operation fails, an InfrastructureError is returned.

### Custom Database

The with_database constructor accepts a custom database path parameter. It opens the cookie with the same flags, then loads the specified database file instead of the default. This allows using application-specific or updated magic databases. If the database file doesn't exist or cannot be loaded, an InfrastructureError is returned.

## Testing Approach

### PNG Buffer Analysis Test

A test creates a LibmagicRepository instance and provides a byte vector containing the PNG file signature (8 bytes: 0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A). When analyzing this buffer with filename "test.png", the repository correctly identifies the MIME type as "image/png".

### File Not Found Test

A test creates a repository instance and attempts to analyze a non-existent file path "/nonexistent/file.txt". The analyze_file method returns a DomainError with the FileNotFound variant, correctly handling the missing file scenario.

## Design Rationale

- **Trait Implementation**: Implements domain-defined `MagicRepository` trait
- **Error Mapping**: Converts FFI errors to domain errors at boundary
- **Async-Ready**: Uses `spawn_blocking` for CPU-bound libmagic calls
- **Thread-Safe**: Cookie is thread-safe, shareable via `Arc`
- **Dependency Inversion**: Domain defines interface, infrastructure provides concrete implementation
- **Clean Separation**: No domain logic, only infrastructure concerns

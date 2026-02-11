# AnalyzePathUseCase Class Diagram

## Overview

The `AnalyzePathUseCase` orchestrates the analysis of files by relative path, with sandbox validation and boundary checks.

## Class Diagram

```mermaid
classDiagram
    class AnalyzePathUseCase {
        -Arc~dyn MagicRepository~ repository
        -PathSandbox sandbox
        +new(repository: Arc~dyn MagicRepository~, sandbox: PathSandbox) Self
        +execute(request: AnalyzePathRequest) Result~MagicResponse, ApplicationError~
    }
    
    class AnalyzePathRequest {
        +relative_path: RelativePath
    }
    
    class MagicResponse {
        +request_id: RequestId
        +mime_type: MimeType
        +description: String
        +encoding: Option~String~
        +analyzed_at: DateTime~Utc~
    }
    
    class PathSandbox {
        +resolve_path(relative: &RelativePath) Result~PathBuf, ValidationError~
        +is_within_sandbox(path: &Path) bool
    }
    
    class ApplicationError {
        <<enumeration>>
    }
    
    class MagicRepository {
        <<trait>>
    }
    
    AnalyzePathUseCase *-- MagicRepository : depends on
    AnalyzePathUseCase *-- PathSandbox : depends on
    AnalyzePathUseCase ..> AnalyzePathRequest : consumes
    AnalyzePathUseCase ..> MagicResponse : produces
    AnalyzePathUseCase ..> ApplicationError : returns
    
    note for AnalyzePathUseCase "Application service\nOrchestrates path-based analysis\nEnforces sandbox boundaries"
```

## Execution Flow

```mermaid
sequenceDiagram
    participant Handler as HTTP Handler
    participant UseCase as AnalyzePathUseCase
    participant Request as AnalyzePathRequest
    participant Sandbox as PathSandbox
    participant Repo as MagicRepository
    participant Result as MagicResult
    participant Response as MagicResponse
    
    Handler->>Request: new(relative_path)
    Handler->>UseCase: execute(request)
    UseCase->>Request: Extract relative_path
    UseCase->>Sandbox: resolve_path(relative_path)
    alt Path valid and within sandbox
        Sandbox-->>UseCase: Ok(absolute_path)
        UseCase->>Repo: analyze_file(absolute_path)
        alt Analysis succeeds
            Repo-->>Result: Ok(MagicResult)
            UseCase->>Response: Map to MagicResponse
            UseCase-->>Handler: Ok(MagicResponse)
        else Analysis fails
            Repo-->>UseCase: Err(DomainError)
            UseCase->>UseCase: Map to ApplicationError
            UseCase-->>Handler: Err(ApplicationError)
        end
    else Path invalid or outside sandbox
        Sandbox-->>UseCase: Err(ValidationError)
        UseCase-->>Handler: Err(ApplicationError::BadRequest)
    end
```

## Properties

| Property | Type | Description |
|----------|------|-------------|
| `repository` | `Arc<dyn MagicRepository>` | Magic analysis repository |
| `sandbox` | `PathSandbox` | Sandbox boundary enforcer |

## Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `new` | `repository: Arc<dyn MagicRepository>, sandbox: PathSandbox` | `Self` | Constructor with dependencies |
| `execute` | `request: AnalyzePathRequest` | `Result<MagicResponse, ApplicationError>` | Execute use case with sandbox validation |

## Execution Process

```mermaid
flowchart TD
    Start([execute request]) --> Extract[Extract relative_path]
    Extract --> Resolve[sandbox.resolve_path]
    Resolve --> InSandbox{Within sandbox?}
    InSandbox -->|No| ErrForbidden[ApplicationError::Forbidden]
    InSandbox -->|Yes| CheckExists{File exists?}
    CheckExists -->|No| ErrNotFound[ApplicationError::NotFound]
    CheckExists -->|Yes| CallRepo[repository.analyze_file]
    CallRepo --> RepoResult{Result?}
    RepoResult -->|Err| MapError[Map DomainError]
    RepoResult -->|Ok| MapResponse[Map MagicResult]
    MapError --> ErrResult[Err ApplicationError]
    MapResponse --> GenRequestId[Generate RequestId]
    GenRequestId --> Success[Ok MagicResponse]
    
    style Success fill:#90EE90
    style ErrForbidden fill:#FFB6C1
    style ErrNotFound fill:#FFB6C1
    style ErrResult fill:#FFB6C1
```

## Sandbox Validation

```mermaid
flowchart TD
    Input["relative_path: 'docs/file.txt'"] --> Canonicalize[Canonicalize to absolute path]
    Canonicalize --> Absolute["'/sandbox/docs/file.txt'"]
    Absolute --> Check{Starts with<br/>sandbox root?}
    Check -->|No| Reject[Err: Path traversal attempt]
    Check -->|Yes| Symlink{Check symlinks}
    Symlink --> Outside{Resolves<br/>outside sandbox?}
    Outside -->|Yes| Reject
    Outside -->|No| Accept[Ok: Safe path]
    
    style Accept fill:#90EE90
    style Reject fill:#FFB6C1
```

## Path Resolution Example

| Input (Relative) | Sandbox Root | Resolved (Absolute) | Result |
|------------------|--------------|---------------------|--------|
| `docs/file.txt` | `/sandbox` | `/sandbox/docs/file.txt` | ✅ Allowed |
| `../etc/passwd` | `/sandbox` | `/etc/passwd` | ❌ Outside sandbox |
| `docs/../file.txt` | `/sandbox` | `/sandbox/file.txt` | ✅ Allowed (normalizes) |
| `/etc/passwd` | `/sandbox` | - | ❌ Absolute path rejected |

## Error Mapping

```mermaid
graph TD
    PathErr[Path Resolution Errors] --> Map1[Error Mapping]
    
    Map1 --> Outside[Outside Sandbox]
    Outside --> Forbidden[ApplicationError::Forbidden]
    
    Map1 --> NotFound[File Not Found]
    NotFound --> NotFoundErr[ApplicationError::NotFound]
    
    Map1 --> Invalid[Invalid Path]
    Invalid --> BadRequest[ApplicationError::BadRequest]
    
    DomainErr[DomainError] --> Map2[Error Mapping]
    Map2 --> Permission[PermissionDenied]
    Permission --> Forbidden
    
    Map2 --> Magic[MagicError]
    Magic --> Unprocessable[ApplicationError::UnprocessableEntity]
    
    style Forbidden fill:#FFB6C1
    style NotFoundErr fill:#FFB6C1
    style BadRequest fill:#FFB6C1
    style Unprocessable fill:#FFB6C1
```

## Usage Example

```rust
// Dependency injection
let repository = Arc::new(LibmagicRepository::new()?);
let sandbox = PathSandbox::new("/var/data/uploads")?;
let use_case = AnalyzePathUseCase::new(repository, sandbox);

// Execute use case
let request = AnalyzePathRequest {
    relative_path: RelativePath::new("documents/report.pdf")?,
};

let response = use_case.execute(request)?;
println!("MIME Type: {}", response.mime_type.as_str());

// Error handling
match use_case.execute(request) {
    Ok(response) => {
        // Success - return HTTP 200
        Json(response)
    },
    Err(ApplicationError::BadRequest(msg)) => {
        // Invalid path - return HTTP 400
        (StatusCode::BAD_REQUEST, msg)
    },
    Err(ApplicationError::Forbidden(msg)) => {
        // Path traversal attempt - return HTTP 403
        (StatusCode::FORBIDDEN, msg)
    },
    Err(ApplicationError::NotFound(msg)) => {
        // File not found - return HTTP 404
        (StatusCode::NOT_FOUND, msg)
    },
    Err(ApplicationError::UnprocessableEntity(msg)) => {
        // Analysis failed - return HTTP 422
        (StatusCode::UNPROCESSABLE_ENTITY, msg)
    },
    Err(_) => {
        // Internal error - return HTTP 500
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    },
}
```

## Security Checks

```mermaid
graph TD
    Request[AnalyzePathRequest] --> Check1[Check: Relative path only]
    Check1 --> Check2[Check: No '..' components]
    Check2 --> Check3[Check: Resolve to absolute]
    Check3 --> Check4[Check: Within sandbox root]
    Check4 --> Check5[Check: Symlinks resolve within sandbox]
    Check5 --> Check6[Check: File exists]
    Check6 --> Check7[Check: File is readable]
    Check7 --> Safe[Safe to analyze]
    
    Check1 -->|Fail| Reject[Reject request]
    Check2 -->|Fail| Reject
    Check3 -->|Fail| Reject
    Check4 -->|Fail| Reject
    Check5 -->|Fail| Reject
    Check6 -->|Fail| NotFound[File not found]
    Check7 -->|Fail| Forbidden[Permission denied]
    
    style Safe fill:#90EE90
    style Reject fill:#FFB6C1
    style NotFound fill:#FFB6C1
    style Forbidden fill:#FFB6C1
```

## Path Traversal Attack Prevention

```
Attack: ../../etc/passwd
Result: Rejected (ParentTraversal in RelativePath validation)

Attack: docs/../../../etc/passwd
Result: Rejected (ParentTraversal detected)

Attack: /etc/passwd
Result: Rejected (AbsolutePath in RelativePath validation)

Attack: docs/symlink-to-etc (where symlink -> /etc)
Result: Rejected (Symlink resolves outside sandbox)

Valid: docs/report.pdf
Result: Accepted (within sandbox, no traversal)
```

## Dependencies

```mermaid
graph TD
    UseCase[AnalyzePathUseCase] --> Repo[MagicRepository Trait]
    UseCase --> Sandbox[PathSandbox]
    UseCase --> Request[AnalyzePathRequest DTO]
    UseCase --> Response[MagicResponse DTO]
    UseCase --> AppError[ApplicationError]
    
    Sandbox --> Infra[Infrastructure Layer]
    Repo --> Domain[Domain Layer]
    Request --> Domain
    Response --> Domain
    
    style Domain fill:#E3F2FD
    style Infra fill:#E8F5E9
    style UseCase fill:#FFF3E0
```

## Design Rationale

- **Security First**: Multi-layer path validation prevents traversal attacks
- **Sandbox Isolation**: Enforces boundary checks before file access
- **Dependency Inversion**: Depends on traits, not concrete implementations
- **Explicit Security**: PathSandbox makes security boundaries explicit
- **Error Semantics**: Maps low-level errors to high-level HTTP semantics
- **Testability**: Easy to test with mock repository and sandbox

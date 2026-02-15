# AnalyzePathUseCase Class Diagram <!-- omit in toc -->

- [Overview](#overview)
- [Class Diagram](#class-diagram)
- [Execution Flow](#execution-flow)
- [Properties](#properties)
- [Methods](#methods)
- [Execution Process](#execution-process)
- [Sandbox Validation](#sandbox-validation)
- [Path Resolution Example](#path-resolution-example)
- [Error Mapping](#error-mapping)
- [Usage Scenario](#usage-scenario)
  - [Initialization](#initialization)
  - [Execution Pattern](#execution-pattern)
  - [Response and Error Handling](#response-and-error-handling)
- [Security Validation Process](#security-validation-process)
- [Testing Strategy](#testing-strategy)
- [Dependencies](#dependencies)
- [Design Rationale](#design-rationale)

---

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
        UseCase->>UseCase: Open file
        UseCase->>UseCase: Memory map file (mmap)
        UseCase->>Repo: analyze_buffer(mmap_slice)
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
| `sandbox` | `Arc<dyn SandboxService>` | Sandbox boundary enforcer |
| `analysis_timeout_secs` | `u64` | Analysis timeout in seconds |

## Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `new` | `repository, sandbox, timeout` | `Self` | Constructor with dependencies |
| `execute` | `request_id, filename, path` | `Result<MagicResult, ApplicationError>` | Execute use case with sandbox validation |

## Execution Process

```mermaid
flowchart TD
    Start([execute request]) --> Extract[Extract relative_path]
    Extract --> Resolve[sandbox.resolve_path]
    Resolve --> InSandbox{Within sandbox?}
    InSandbox -->|No| ErrForbidden[ApplicationError::Forbidden]
    InSandbox -->|Yes| CheckExists{File exists?}
    CheckExists -->|No| ErrNotFound[ApplicationError::NotFound]
    CheckExists -->|Yes| Mmap[Memory map file]
    Mmap --> CallRepo[repository.analyze_buffer]
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

## Usage Scenario

### Initialization

The AnalyzePathUseCase is initialized with two primary dependencies: a MagicRepository implementation for the actual file analysis and a PathSandbox for enforcing security boundaries. These dependencies are typically injected as thread-safe atomic reference counters.

### Execution Pattern

To analyze a file by path, a request is constructed containing a RelativePath value object. The execute method first resolves this relative path to an absolute path within the sandbox. If resolution succeeds and the file exists, the use case calls the repository to analyze the file and returns a response.

### Response and Error Handling

Successful execution results in a MagicResponse containing the file type details. If the path traversal attempt is detected, it returns Forbidden. If the file does not exist, it returns NotFound. Other input errors result in BadRequest, and analysis failures return UnprocessableEntity.

## Security Validation Process

The use case coordinates a multi-step security validation process:
1. **Initial Validation**: The RelativePath object ensures the input doesn't contain forbidden sequences like parent directory references.
2. **Path Resolution**: The sandbox resolves the path, ensuring it stays within the configured root and doesn't escape via symlinks.
3. **Existence Verification**: Confirms the file exists before attempting analysis.
4. **Boundary Check**: Final verification that the canonicalized path remains within the sandbox.

## Testing Strategy

Testing the path-based analysis involves several scenarios:
- **Success Path**: Verification that valid files within the sandbox are correctly analyzed.
- **Symlink Protection**: Ensuring symlinks pointing outside the sandbox are rejected.
- **Traversal Prevention**: Confirming that attempts to use '..' or absolute paths are blocked at the entry point.
- **Missing Resources**: Verifying that 404 errors are returned for files that do not exist.

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

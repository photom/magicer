# AnalyzeContentUseCase Class Diagram

## Overview

The `AnalyzeContentUseCase` orchestrates the analysis of binary content (uploaded files, network buffers) using the magic repository.

## Class Diagram

```mermaid
classDiagram
    class AnalyzeContentUseCase {
        -Arc~dyn MagicRepository~ repository
        +new(repository: Arc~dyn MagicRepository~) Self
        +execute(request: AnalyzeContentRequest) Result~MagicResponse, ApplicationError~
    }
    
    class AnalyzeContentRequest {
        +content: Bytes
        +filename: WindowsCompatibleFilename
    }
    
    class MagicResponse {
        +request_id: RequestId
        +mime_type: MimeType
        +description: String
        +encoding: Option~String~
        +analyzed_at: DateTime~Utc~
    }
    
    class ApplicationError {
        <<enumeration>>
    }
    
    class MagicRepository {
        <<trait>>
    }
    
    AnalyzeContentUseCase *-- MagicRepository : depends on
    AnalyzeContentUseCase ..> AnalyzeContentRequest : consumes
    AnalyzeContentUseCase ..> MagicResponse : produces
    AnalyzeContentUseCase ..> ApplicationError : returns
    
    note for AnalyzeContentUseCase "Application service\nOrchestrates domain objects\nDependency: MagicRepository trait"
```

## Execution Flow

```mermaid
sequenceDiagram
    participant Handler as HTTP Handler
    participant UseCase as AnalyzeContentUseCase
    participant Request as AnalyzeContentRequest
    participant Repo as MagicRepository
    participant Result as MagicResult
    participant Response as MagicResponse
    
    Handler->>Request: new(content, filename)
    Handler->>UseCase: execute(request)
    UseCase->>Request: Extract content & filename
    UseCase->>Repo: analyze_buffer(data, filename)
    alt Analysis succeeds
        Repo-->>Result: Ok(MagicResult)
        UseCase->>Response: Map to MagicResponse
        UseCase-->>Handler: Ok(MagicResponse)
    else Analysis fails
        Repo-->>UseCase: Err(DomainError)
        UseCase->>UseCase: Map to ApplicationError
        UseCase-->>Handler: Err(ApplicationError)
    end
```

## Properties

| Property | Type | Description |
|----------|------|-------------|
| `repository` | `Arc<dyn MagicRepository>` | Magic analysis repository (trait object) |

## Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `new` | `repository: Arc<dyn MagicRepository>` | `Self` | Constructor with dependency injection |
| `execute` | `request: AnalyzeContentRequest` | `Result<MagicResponse, ApplicationError>` | Execute use case |

## Execution Process

```mermaid
flowchart TD
    Start([execute request]) --> Extract[Extract content & filename]
    Extract --> Validate{Valid input?}
    Validate -->|No| ErrBadRequest[ApplicationError::BadRequest]
    Validate -->|Yes| CallRepo[repository.analyze_buffer]
    CallRepo --> RepoResult{Result?}
    RepoResult -->|Err| MapError[Map DomainError to ApplicationError]
    RepoResult -->|Ok| MapResponse[Map MagicResult to MagicResponse]
    MapError --> ErrResult[Err ApplicationError]
    MapResponse --> GenRequestId[Generate RequestId]
    GenRequestId --> Success[Ok MagicResponse]
    
    style Success fill:#90EE90
    style ErrBadRequest fill:#FFB6C1
    style ErrResult fill:#FFB6C1
```

## Error Mapping

```mermaid
graph TD
    Domain[DomainError] --> Map[Error Mapping]
    
    Map --> Validation[ValidationError]
    Validation --> BadRequest[ApplicationError::BadRequest]
    
    Map --> Magic[MagicError]
    Magic --> UnprocessableEntity[ApplicationError::UnprocessableEntity]
    
    Map --> FileNotFound[FileNotFound]
    FileNotFound --> NotFound[ApplicationError::NotFound]
    
    Map --> Permission[PermissionDenied]
    Permission --> Forbidden[ApplicationError::Forbidden]
    
    Map --> Config[ConfigurationError]
    Config --> Internal[ApplicationError::InternalError]
    
    style BadRequest fill:#FFB6C1
    style UnprocessableEntity fill:#FFB6C1
    style NotFound fill:#FFB6C1
    style Forbidden fill:#FFB6C1
    style Internal fill:#FFB6C1
```

## Usage Scenario

### Use Case Initialization

The AnalyzeContentUseCase is initialized by providing a thread-safe reference to a MagicRepository implementation. This is typically done during application startup when the dependency injection container is configured.

### Executing Analysis

To analyze content, a request object is created containing the binary data and the original filename. When the execute method is called with this request, the use case validates the input, delegates the actual magic analysis to the repository, and returns a formatted response.

### Handling Failures

The use case provides semantic error handling. If the input data is empty or invalid, it returns a BadRequest error. If the libmagic analysis fails due to data corruption or other internal issues, it returns an UnprocessableEntity error. Unexpected system failures are returned as InternalError.

## Request Validation

The use case performs several validation steps before proceeding with analysis:
1. **Empty Check**: Rejects requests where the binary content is empty.
2. **Size Check**: While the absolute limit is enforced at the HTTP layer, the use case ensures the content is within reasonable processing bounds.
3. **Filename Integrity**: Verifies that the provided filename hint is valid and safe.

## Response Construction

Upon successful analysis, the use case transforms the domain-level MagicResult entity into an application-level MagicResponse DTO. This process involves:
1. Generating a new unique RequestId for tracking.
2. Extracting the MIME type and human-readable description.
3. Including the character encoding if one was detected.
4. Recording the precise UTC timestamp when the analysis was completed.

## Dependencies

```mermaid
graph TD
    UseCase[AnalyzeContentUseCase] --> Repo[MagicRepository Trait]
    UseCase --> Request[AnalyzeContentRequest DTO]
    UseCase --> Response[MagicResponse DTO]
    UseCase --> AppError[ApplicationError]
    
    Repo --> Domain[Domain Layer]
    Request --> Domain
    Response --> Domain
    AppError --> DomainError[DomainError]
    
    style Domain fill:#E3F2FD
    style UseCase fill:#FFF3E0
```

## Design Rationale

- **Single Responsibility**: Only orchestrates content analysis workflow
- **Dependency Inversion**: Depends on `MagicRepository` trait, not concrete implementation
- **DTO Boundary**: Uses DTOs for input/output, isolating domain from presentation
- **Error Translation**: Maps domain errors to application-level semantic errors
- **Testability**: Easy to test with mock repository
- **Thread Safety**: `Arc<dyn Trait>` enables sharing across async tasks

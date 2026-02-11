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

## Usage Example

```rust
// Dependency injection
let repository = Arc::new(LibmagicRepository::new()?);
let use_case = AnalyzeContentUseCase::new(repository);

// Execute use case
let request = AnalyzeContentRequest {
    content: Bytes::from(file_data),
    filename: WindowsCompatibleFilename::new("document.pdf".to_string())?,
};

let response = use_case.execute(request)?;
println!("MIME Type: {}", response.mime_type.as_str());
println!("Description: {}", response.description);

// Error handling
match use_case.execute(request) {
    Ok(response) => {
        // Success - return HTTP 200
        Json(response)
    },
    Err(ApplicationError::BadRequest(msg)) => {
        // Invalid input - return HTTP 400
        StatusCode::BAD_REQUEST
    },
    Err(ApplicationError::UnprocessableEntity(msg)) => {
        // Analysis failed - return HTTP 422
        StatusCode::UNPROCESSABLE_ENTITY
    },
    Err(_) => {
        // Internal error - return HTTP 500
        StatusCode::INTERNAL_SERVER_ERROR
    },
}
```

## Request Validation

```mermaid
flowchart TD
    Request[AnalyzeContentRequest] --> CheckContent{Content empty?}
    CheckContent -->|Yes| ErrEmpty[Err: Content is empty]
    CheckContent -->|No| CheckSize{Content > 100MB?}
    CheckSize -->|Yes| ErrTooLarge[Err: Content too large]
    CheckSize -->|No| CheckFilename{Valid filename?}
    CheckFilename -->|No| ErrFilename[Err: Invalid filename]
    CheckFilename -->|Yes| Valid[Request is valid]
    
    style Valid fill:#90EE90
    style ErrEmpty fill:#FFB6C1
    style ErrTooLarge fill:#FFB6C1
    style ErrFilename fill:#FFB6C1
```

## Response Construction

```rust
impl From<MagicResult> for MagicResponse {
    fn from(result: MagicResult) -> Self {
        MagicResponse {
            request_id: RequestId::new(),
            mime_type: result.mime_type().clone(),
            description: result.description().to_string(),
            encoding: result.encoding().map(|s| s.to_string()),
            analyzed_at: result.analyzed_at(),
        }
    }
}
```

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

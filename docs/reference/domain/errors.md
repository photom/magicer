# Domain Errors Class Diagram

## Overview

Domain errors represent all possible failures within the domain layer, with no leakage of infrastructure implementation details.

## Class Diagram

```mermaid
classDiagram
    class DomainError {
        <<enumeration>>
        +ValidationError(ValidationError)
        +MagicError(MagicError)
        +FileNotFound(String)
        +PermissionDenied(String)
        +ConfigurationError(String)
    }
    
    class ValidationError {
        <<enumeration>>
        +EmptyFilename
        +FilenameTooLong(usize, usize)
        +ContainsForwardSlash
        +ContainsNullByte
        +InvalidCharacter(char)
        +AbsolutePath
        +ParentTraversal
        +EmptyPath
        +InvalidUtf8
        +DoubleSlash
        +EmptyUsername
        +EmptyPassword
        +UsernameTooLong(usize)
        +PasswordTooShort(usize)
        +UsernameContainsColon
        +EmptyMimeType
        +MissingSlash
        +InvalidMimeFormat
        +EmptyType
        +EmptySubtype
        +InvalidUuidFormat(String)
        +NotV4Uuid
    }
    
    class MagicError {
        <<enumeration>>
        +AnalysisFailed(String)
        +UnsupportedFormat(String)
        +BufferTooSmall
        +CorruptedData(String)
    }
    
    DomainError *-- ValidationError : contains
    DomainError *-- MagicError : contains
    
    note for DomainError "Top-level domain error\nNo infrastructure types\nAll errors domain-specific"
    
    note for ValidationError "Input validation failures\nConstructor validation\nValue object constraints"
    
    note for MagicError "Magic analysis failures\nFile type detection errors\nData corruption errors"
```

## Error Hierarchy

```mermaid
graph TD
    Domain[DomainError] --> Validation[ValidationError]
    Domain --> Magic[MagicError]
    Domain --> FileNotFound
    Domain --> PermissionDenied
    Domain --> ConfigurationError
    
    Validation --> Filename[Filename validation]
    Validation --> Path[Path validation]
    Validation --> Creds[Credentials validation]
    Validation --> Mime[MIME type validation]
    Validation --> Uuid[UUID validation]
    
    Magic --> Analysis[Analysis failures]
    Magic --> Unsupported[Unsupported formats]
    Magic --> Buffer[Buffer errors]
    Magic --> Corruption[Data corruption]
    
    style Domain fill:#FFB6C1
    style Validation fill:#FFEB3B
    style Magic fill:#FF9800
```

## DomainError Variants

| Variant | Payload | Description | Use Case |
|---------|---------|-------------|----------|
| `ValidationError` | `ValidationError` | Input validation failed | Value object construction |
| `MagicError` | `MagicError` | Magic analysis failed | Repository operations |
| `FileNotFound` | `String` (path) | File doesn't exist | Path-based analysis |
| `PermissionDenied` | `String` (path) | Insufficient permissions | File access |
| `ConfigurationError` | `String` (message) | Invalid configuration | Service initialization |

## ValidationError Variants

### Filename Validation

| Variant | Payload | Constraint Violated |
|---------|---------|---------------------|
| `EmptyFilename` | - | Non-empty requirement |
| `FilenameTooLong` | `(actual, max)` | Max 310 characters |
| `ContainsForwardSlash` | - | No `/` allowed |
| `ContainsNullByte` | - | No `\0` allowed |
| `InvalidCharacter` | `char` | Invalid character found |

### Path Validation

| Variant | Payload | Constraint Violated |
|---------|---------|---------------------|
| `AbsolutePath` | - | Must be relative |
| `ParentTraversal` | - | No `..` allowed |
| `EmptyPath` | - | Non-empty requirement |
| `InvalidUtf8` | - | Valid UTF-8 required |
| `DoubleSlash` | - | No `//` allowed |

### Credentials Validation

| Variant | Payload | Constraint Violated |
|---------|---------|---------------------|
| `EmptyUsername` | - | Non-empty requirement |
| `EmptyPassword` | - | Non-empty requirement |
| `UsernameTooLong` | `usize` (length) | Max 256 characters |
| `PasswordTooShort` | `usize` (length) | Min 8 characters |
| `UsernameContainsColon` | - | No `:` in username |

### MIME Type Validation

| Variant | Payload | Constraint Violated |
|---------|---------|---------------------|
| `EmptyMimeType` | - | Non-empty requirement |
| `MissingSlash` | - | Must contain `/` |
| `InvalidMimeFormat` | - | RFC 6838 compliance |
| `EmptyType` | - | Type part non-empty |
| `EmptySubtype` | - | Subtype part non-empty |

### UUID Validation

| Variant | Payload | Constraint Violated |
|---------|---------|---------------------|
| `InvalidUuidFormat` | `String` (input) | Valid UUID format |
| `NotV4Uuid` | - | Must be UUID v4 |

## MagicError Variants

| Variant | Payload | Description | Recovery |
|---------|---------|-------------|----------|
| `AnalysisFailed` | `String` (reason) | libmagic analysis failed | None |
| `UnsupportedFormat` | `String` (format) | File format not recognized | None |
| `BufferTooSmall` | - | Insufficient data for analysis | Provide more data |
| `CorruptedData` | `String` (reason) | File data is corrupted | None |

## Error Flow

```mermaid
sequenceDiagram
    participant Value as Value Object
    participant Repo as Repository
    participant Infra as Infrastructure
    participant Libmagic
    
    Value->>Value: Validate input
    alt Validation fails
        Value-->>Repo: Err(ValidationError)
    else Validation succeeds
        Repo->>Infra: Call implementation
        Infra->>Libmagic: FFI call
        alt libmagic error
            Libmagic-->>Infra: C error
            Infra->>Infra: Map to MagicError
            Infra-->>Repo: Err(MagicError)
        else Success
            Libmagic-->>Infra: Success
            Infra-->>Repo: Ok(MagicResult)
        end
    end
```

## Error Mapping

Infrastructure errors are mapped to domain errors at the boundary:

```mermaid
graph LR
    LibmagicErr["libmagic::Error"] --> Map1[Map]
    Map1 --> MagicErr[DomainError::MagicError]
    
    IOErr["std::io::Error"] --> Map2[Map]
    Map2 --> FileNotFound[DomainError::FileNotFound]
    Map2 --> PermissionDenied[DomainError::PermissionDenied]
    
    ConfigErr["config::Error"] --> Map3[Map]
    Map3 --> ConfigError[DomainError::ConfigurationError]
    
    style MagicErr fill:#FFB6C1
    style FileNotFound fill:#FFB6C1
    style PermissionDenied fill:#FFB6C1
    style ConfigError fill:#FFB6C1
```

## Usage Example

```rust
// Value object validation
let filename = WindowsCompatibleFilename::new("a".repeat(311))
    .map_err(|e| DomainError::ValidationError(e))?;

// Repository operation
let result = repository.analyze_buffer(data, &filename)
    .map_err(|e| match e {
        DomainError::MagicError(magic_err) => {
            log::error!("Magic analysis failed: {:?}", magic_err);
            e
        },
        DomainError::ValidationError(val_err) => {
            log::warn!("Invalid input: {:?}", val_err);
            e
        },
        _ => e,
    })?;

// Error handling in application layer
match use_case.execute(request) {
    Ok(response) => Ok(response),
    Err(DomainError::ValidationError(e)) => {
        Err(ApplicationError::BadRequest(format!("Invalid input: {}", e)))
    },
    Err(DomainError::FileNotFound(path)) => {
        Err(ApplicationError::NotFound(format!("File not found: {}", path)))
    },
    Err(DomainError::PermissionDenied(path)) => {
        Err(ApplicationError::Forbidden(format!("Access denied: {}", path)))
    },
    Err(e) => {
        Err(ApplicationError::InternalError(format!("Unexpected error: {:?}", e)))
    },
}
```

## Trait Implementations

```rust
impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::ValidationError(e) => write!(f, "Validation error: {}", e),
            DomainError::MagicError(e) => write!(f, "Magic analysis error: {}", e),
            DomainError::FileNotFound(path) => write!(f, "File not found: {}", path),
            DomainError::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
            DomainError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}
```

## Error Conversion

```rust
impl From<ValidationError> for DomainError {
    fn from(err: ValidationError) -> Self {
        DomainError::ValidationError(err)
    }
}

impl From<MagicError> for DomainError {
    fn from(err: MagicError) -> Self {
        DomainError::MagicError(err)
    }
}
```

## Design Rationale

- **No Infrastructure Leakage**: Domain errors contain no `std::io::Error`, `sqlx::Error`, etc.
- **Semantic Errors**: Each error variant has domain meaning, not technical details
- **Comprehensive**: Covers all domain-level failure modes
- **Composable**: Hierarchical structure (top-level → specific)
- **Mappable**: Easy to convert from infrastructure errors at boundary
- **Type Safety**: Compile-time guarantee of proper error handling

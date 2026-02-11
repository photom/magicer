# Application DTOs Class Diagrams

## Overview

Data Transfer Objects (DTOs) define the input and output contracts for application use cases, isolating domain from presentation concerns.

## DTO Architecture

```mermaid
graph TD
    Presentation[Presentation Layer] --> RequestDTO[Request DTOs]
    RequestDTO --> UseCase[Use Cases]
    UseCase --> ResponseDTO[Response DTOs]
    ResponseDTO --> Presentation
    
    UseCase --> Domain[Domain Layer]
    Domain --> Entity[Entities & Value Objects]
    
    style RequestDTO fill:#FFF3E0
    style ResponseDTO fill:#E8F5E9
    style UseCase fill:#E3F2FD
    style Domain fill:#F3E5F5
```

---

## AnalyzeContentRequest DTO

### Class Diagram

```mermaid
classDiagram
    class AnalyzeContentRequest {
        +content: Bytes
        +filename: WindowsCompatibleFilename
        +new(content: Bytes, filename: WindowsCompatibleFilename) Self
        +content() &Bytes
        +filename() &WindowsCompatibleFilename
    }
    
    class Bytes {
        <<external::bytes>>
    }
    
    class WindowsCompatibleFilename {
        <<value object>>
    }
    
    AnalyzeContentRequest *-- Bytes : contains
    AnalyzeContentRequest *-- WindowsCompatibleFilename : contains
    
    note for AnalyzeContentRequest "Request DTO\nDerives: Clone, Debug\nImmutable after construction"
```

### Properties

| Property | Type | Description | Validation |
|----------|------|-------------|------------|
| `content` | `Bytes` | Binary file content | Max 100MB (enforced at HTTP layer) |
| `filename` | `WindowsCompatibleFilename` | Original filename | Value object validation |

### Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `new` | `content: Bytes, filename: WindowsCompatibleFilename` | `Self` | Constructor |
| `content` | `&self` | `&Bytes` | Get content reference |
| `filename` | `&self` | `&WindowsCompatibleFilename` | Get filename reference |

## Usage Pattern

### AnalyzeContentRequest

The AnalyzeContentRequest DTO is typically constructed in the presentation layer after extracting the binary body and filename from the HTTP request. Once created, it is passed to the AnalyzeContentUseCase for processing. It encapsulates the binary data and its associated metadata in a single immutable structure.

### AnalyzePathRequest

The AnalyzePathRequest DTO is created when a request arrives at the path-based analysis endpoint. It wraps a validated RelativePath value object, ensuring that the use case receives a safe and well-formed path.

### MagicResponse

The MagicResponse DTO is the primary output of the system. It is constructed by use cases from domain entities and then serialized to JSON for the HTTP response. It includes the detected file type, a descriptive label, any encoding information, and a unique tracking ID for the request.

## DTO Mapping and Serialization

The system uses a strict mapping approach to maintain layer isolation:
1. **Input Mapping**: HTTP request data is transformed into request DTOs.
2. **Internal Processing**: Use cases operate on DTOs and produce domain entities.
3. **Output Mapping**: Domain entities are transformed into response DTOs.
4. **Serialization**: Response DTOs are serialized into JSON format.

Serialization is controlled via standard attributes to handle optional fields (omitting null values from JSON) and formatting timestamps consistently using the ISO 8601 standard.

### Serialization Options

| Option | Usage | Purpose |
|--------|-------|---------|
| `#[serde(skip_serializing_if = "Option::is_none")]` | Optional fields | Omit `null` from JSON |
| `#[serde(rename = "mime_type")]` | Field naming | Control JSON key names |
| `#[serde(with = "chrono::serde::ts_seconds")]` | Timestamp format | Unix timestamp instead of ISO 8601 |

## Dependencies

```mermaid
graph TD
    DTO[Application DTOs] --> Domain[Domain Value Objects]
    DTO --> External[External Crates]
    
    Domain --> Filename[WindowsCompatibleFilename]
    Domain --> Path[RelativePath]
    Domain --> RequestId[RequestId]
    Domain --> Mime[MimeType]
    Domain --> Result[MagicResult]
    
    External --> Bytes[bytes::Bytes]
    External --> Chrono[chrono::DateTime]
    External --> Serde[serde Serialize/Deserialize]
    
    style DTO fill:#FFF3E0
    style Domain fill:#F3E5F5
    style External fill:#E0F2F1
```

## Design Rationale

- **Boundary Objects**: DTOs mark the boundary between layers
- **Type Safety**: Value objects ensure validation before reaching use cases
- **Serialization**: DTOs are designed for JSON serialization (HTTP responses)
- **Immutability**: Prevents accidental modification during request processing
- **Simple**: No methods beyond getters and builders
- **Testable**: Easy to construct for testing

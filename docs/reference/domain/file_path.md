# FilePath Value Object Class Diagram <!-- omit in toc -->

- [Overview](#overview)
- [Class Diagram](#class-diagram)
- [Validation Rules](#validation-rules)
- [Properties](#properties)
- [Methods](#methods)
- [Invariants](#invariants)
- [Usage Scenarios](#usage-scenarios)
  - [Valid Relative Path](#valid-relative-path)
  - [Invalid Scenarios](#invalid-scenarios)
  - [Join Operation](#join-operation)
- [Path Normalization](#path-normalization)
- [Design Rationale](#design-rationale)

---

## Overview

The `RelativePath` value object ensures file paths are relative, normalized, and safe from traversal attacks.

## Class Diagram

```mermaid
classDiagram
    class RelativePath {
        -PathBuf path
        +new(path: impl AsRef~Path~) Result~Self, ValidationError~
        +as_path() &Path
        +to_path_buf() PathBuf
        +join(component: impl AsRef~Path~) Result~Self, ValidationError~
        +canonicalize() Result~Self, ValidationError~
    }
    
    class ValidationError {
        <<enumeration>>
        AbsolutePath
        ParentTraversal
        EmptyPath
        InvalidUtf8
        DoubleSlash
    }
    
    RelativePath ..> ValidationError : creates
    
    note for RelativePath "Immutable value object\nDerives: Clone, Eq, PartialEq, Hash, Debug\nNo leading '/'\nNo '..'\nNo '//'"
```

## Validation Rules

```mermaid
flowchart TD
    Start([new path: impl AsRef Path]) --> CheckEmpty{Is empty?}
    CheckEmpty -->|Yes| ErrEmpty[ValidationError::EmptyPath]
    CheckEmpty -->|No| CheckUtf8{Valid UTF-8?}
    CheckUtf8 -->|No| ErrUtf8[ValidationError::InvalidUtf8]
    CheckUtf8 -->|Yes| CheckAbsolute{Is absolute?}
    CheckAbsolute -->|Yes| ErrAbsolute[ValidationError::AbsolutePath]
    CheckAbsolute -->|No| CheckParent{Contains '..'?}
    CheckParent -->|Yes| ErrParent[ValidationError::ParentTraversal]
    CheckParent -->|No| CheckDouble{Contains '//'?}
    CheckDouble -->|Yes| ErrDouble[ValidationError::DoubleSlash]
    CheckDouble -->|No| Success([Ok RelativePath])
    
    style Success fill:#90EE90
    style ErrEmpty fill:#FFB6C1
    style ErrUtf8 fill:#FFB6C1
    style ErrAbsolute fill:#FFB6C1
    style ErrParent fill:#FFB6C1
    style ErrDouble fill:#FFB6C1
```

## Properties

| Property | Type | Description |
|----------|------|-------------|
| `path` | `PathBuf` | Validated relative path |

## Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `new` | `path: impl AsRef<Path>` | `Result<Self, ValidationError>` | Constructor with validation |
| `as_path` | `&self` | `&Path` | Borrow as Path reference |
| `to_path_buf` | `&self` | `PathBuf` | Clone as PathBuf |
| `join` | `component: impl AsRef<Path>` | `Result<Self, ValidationError>` | Join with another component, re-validate |
| `canonicalize` | `&self` | `Result<Self, ValidationError>` | Normalize path (resolve `.`, ensure relative) |

## Invariants

1. Never empty
2. Always relative (no leading `/`)
3. No parent directory traversal (`..`)
4. No double slashes (`//`)
5. Valid UTF-8 encoding
6. Immutable after construction

## Usage Scenarios

### Valid Relative Path

When constructing RelativePath with a valid relative path like "documents/report.pdf", the value object is successfully created and the path can be retrieved as a Path reference.

### Invalid Scenarios

**Absolute Path:** Construction fails with AbsolutePath validation error when the path starts with a forward slash, such as "/etc/passwd", as only relative paths are allowed.

**Parent Traversal:** Construction fails with ParentTraversal validation error when the path contains parent directory references like "../etc/passwd", preventing directory traversal attacks.

### Join Operation

When joining two relative paths, such as base path "documents" with component "report.pdf", the result is a new validated RelativePath representing "documents/report.pdf". The join operation re-validates the resulting path to ensure it remains safe.

## Path Normalization

```mermaid
flowchart LR
    Input["Input: './docs/./file.txt'"] --> Normalize[canonicalize]
    Normalize --> Output["Output: 'docs/file.txt'"]
    
    Input2["Input: 'docs/../file.txt'"] --> Validate[Validation]
    Validate --> Error["Error: ParentTraversal"]
    
    style Output fill:#90EE90
    style Error fill:#FFB6C1
```

## Design Rationale

- **Security**: Prevents directory traversal attacks through multiple validation layers
- **Normalization**: Removes redundant path components (`.`, trailing slashes)
- **Type Safety**: Compile-time guarantee that paths are validated before use
- **Cross-Platform**: Uses `PathBuf` for platform-agnostic path handling
- **Value Object Pattern**: Immutable, self-validating, comparable by value

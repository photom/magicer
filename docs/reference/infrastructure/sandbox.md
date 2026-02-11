# PathSandbox Class Diagram

## Overview

The `PathSandbox` utility enforces filesystem boundaries to prevent directory traversal attacks by validating and resolving paths within a sandbox root.

## Class Diagram

```mermaid
classDiagram
    class PathSandbox {
        -PathBuf sandbox_root
        +new(sandbox_root: impl AsRef~Path~) Result~Self, InfrastructureError~
        +resolve_path(relative: &RelativePath) Result~PathBuf, DomainError~
        +is_within_sandbox(path: &Path) bool
        +canonicalize_and_check(path: &Path) Result~PathBuf, DomainError~
        +sandbox_root() &Path
    }
    
    class RelativePath {
        <<value object>>
    }
    
    class DomainError {
        <<enumeration>>
    }
    
    PathSandbox ..> RelativePath : uses
    PathSandbox ..> DomainError : returns
    
    note for PathSandbox "Infrastructure utility\nPath validation & resolution\nSymlink-aware security"
```

## Initialization

```mermaid
flowchart TD
    Start([new sandbox_root]) --> CheckExists{Directory exists?}
    CheckExists -->|No| ErrNotFound[Err: Sandbox directory not found]
    CheckExists -->|Yes| CheckDir{Is directory?}
    CheckDir -->|No| ErrNotDir[Err: Sandbox root is not a directory]
    CheckDir -->|Yes| Canonicalize[Canonicalize to absolute path]
    Canonicalize --> CanonSuccess{Success?}
    CanonSuccess -->|No| ErrCanon[Err: Cannot canonicalize]
    CanonSuccess -->|Yes| CheckPerms{Readable?}
    CheckPerms -->|No| ErrPerms[Err: Permission denied]
    CheckPerms -->|Yes| Success[Ok PathSandbox]
    
    style Success fill:#90EE90
    style ErrNotFound fill:#FFB6C1
    style ErrNotDir fill:#FFB6C1
    style ErrCanon fill:#FFB6C1
    style ErrPerms fill:#FFB6C1
```

## Properties

| Property | Type | Description |
|----------|------|-------------|
| `sandbox_root` | `PathBuf` | Canonicalized absolute path to sandbox root |

## Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `new` | `sandbox_root: impl AsRef<Path>` | `Result<Self, InfrastructureError>` | Initialize sandbox with root directory |
| `resolve_path` | `relative: &RelativePath` | `Result<PathBuf, DomainError>` | Resolve relative path to absolute within sandbox |
| `is_within_sandbox` | `path: &Path` | `bool` | Check if path is within sandbox boundaries |
| `canonicalize_and_check` | `path: &Path` | `Result<PathBuf, DomainError>` | Canonicalize and verify path is within sandbox |
| `sandbox_root` | `&self` | `&Path` | Get sandbox root path |

## Path Resolution Process

```mermaid
sequenceDiagram
    participant Caller
    participant Sandbox as PathSandbox
    participant FS as Filesystem
    participant Canonicalizer
    
    Caller->>Sandbox: resolve_path(relative_path)
    Sandbox->>Sandbox: Join sandbox_root + relative_path
    Sandbox->>FS: Check file exists
    alt File not found
        FS-->>Sandbox: Not found
        Sandbox-->>Caller: Err(DomainError::FileNotFound)
    else File exists
        Sandbox->>Canonicalizer: Canonicalize full path
        Canonicalizer-->>Sandbox: Absolute canonical path
        Sandbox->>Sandbox: is_within_sandbox?
        alt Outside sandbox
            Sandbox-->>Caller: Err(DomainError::Forbidden)
        else Inside sandbox
            Sandbox-->>Caller: Ok(absolute_path)
        end
    end
```

## Boundary Checking

```mermaid
flowchart TD
    Input["Absolute path:<br/>/sandbox/docs/file.txt"] --> Canonicalize[Canonicalize path]
    Canonicalize --> Canonical["Canonical:<br/>/sandbox/docs/file.txt"]
    Canonical --> CheckPrefix{Starts with<br/>sandbox_root?}
    CheckPrefix -->|No| Outside[Path is outside sandbox]
    CheckPrefix -->|Yes| CheckSymlink{Check symlink target}
    CheckSymlink --> SymlinkOutside{Resolves outside<br/>sandbox?}
    SymlinkOutside -->|Yes| Outside
    SymlinkOutside -->|No| Inside[Path is inside sandbox]
    
    Outside --> Reject[Err: Forbidden]
    Inside --> Accept[Ok: Safe path]
    
    style Accept fill:#90EE90
    style Reject fill:#FFB6C1
```

## Symlink Handling

```mermaid
graph TD
    Symlink["/sandbox/link → /etc/passwd"] --> Resolve[Resolve symlink target]
    Resolve --> Target["/etc/passwd"]
    Target --> Check{Within sandbox?}
    Check -->|No| Block[Block: Target outside sandbox]
    Check -->|Yes| Allow[Allow: Target within sandbox]
    
    Symlink2["/sandbox/link → /sandbox/file.txt"] --> Resolve2[Resolve symlink target]
    Resolve2 --> Target2["/sandbox/file.txt"]
    Target2 --> Check2{Within sandbox?}
    Check2 -->|Yes| Allow
    
    style Block fill:#FFB6C1
    style Allow fill:#90EE90
```

## Path Traversal Prevention

| Attack Vector | Input | Result | Reason |
|---------------|-------|--------|--------|
| **Absolute path** | `/etc/passwd` | ❌ Rejected | `RelativePath` validation fails |
| **Parent traversal** | `../../../etc/passwd` | ❌ Rejected | `RelativePath` validation fails (contains `..`) |
| **Normalized traversal** | `docs/../../etc/passwd` | ❌ Rejected | `RelativePath` validation fails |
| **Symlink escape** | `link` → `/etc/passwd` | ❌ Rejected | Canonicalized path outside sandbox |
| **Valid relative** | `docs/file.txt` | ✅ Allowed | Within sandbox, no traversal |

## Usage Scenario

### Initialization

The PathSandbox is initialized with a path to the root directory that defines the security boundary. During initialization, it verifies that the directory exists and is accessible, then canonicalizes the path to ensure all subsequent boundary checks use absolute, normalized paths.

### Path Resolution

When resolving a path, the sandbox takes a RelativePath value object and joins it with the sandbox root. It then verifies the file exists, canonicalizes the full path to resolve any internal symlinks or relative components, and finally checks if the resulting absolute path still starts with the sandbox root.

### Boundary Verification

The sandbox provides a dedicated check to determine if any given path resides within its defined boundaries. This is used both during path resolution and as a general-purpose security check throughout the infrastructure layer.

## Implementation Details

The implementation of PathSandbox relies on several key filesystem operations:
1. **Joining**: Combines the sandbox root with the user-provided relative path.
2. **Existence Check**: Confirms the file exists before attempting more complex resolution.
3. **Canonicalization**: The core security step which resolves all symbolic links and dot-segments (`.` and `..`) to produce a definitive absolute path.
4. **Prefix Check**: A string-based comparison ensuring the canonical path begins with the canonical sandbox root.

## Testing Strategy

Testing the sandbox involves multiple scenarios to ensure no escape is possible:
- **Valid Resolution**: Confirms that normal files within the sandbox are correctly resolved.
- **Symlink Escapes**: Verifies that symlinks pointing to sensitive system files like `/etc/passwd` are detected and blocked.
- **Boundary Checks**: Ensures that both immediate children and deeply nested files are correctly identified as being within the sandbox.
- **Negative Cases**: Confirms that files in sister directories or parent directories are correctly rejected.

## Error Cases

| Scenario | Error | HTTP Status |
|----------|-------|-------------|
| File not found | `DomainError::FileNotFound` | 404 Not Found |
| Path outside sandbox | `DomainError::Forbidden` | 403 Forbidden |
| Symlink to external file | `DomainError::Forbidden` | 403 Forbidden |
| Permission denied | `DomainError::PermissionDenied` | 403 Forbidden |
| Invalid sandbox root | `InfrastructureError::InvalidConfig` | 500 Internal Error |

## Design Rationale

- **Defense in Depth**: Multiple layers of validation (value object + sandbox)
- **Canonicalization**: Resolves symlinks and normalizes paths for accurate checking
- **Explicit Boundaries**: Sandbox root is explicit and immutable
- **Security First**: Rejects on any suspicious pattern
- **Integration**: Works with `RelativePath` value object for end-to-end safety
- **Testable**: Easy to test with temporary directories

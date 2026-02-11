# TempFileHandler Class Diagram

## Overview

The `TempFileHandler` manages temporary files for content-based magic analysis, ensuring atomic creation, unique naming, and automatic cleanup.

## Class Diagram

```mermaid
classDiagram
    class TempFileHandler {
        -PathBuf path
        -bool cleaned_up
        +create_temp_file(data: &[u8], base_dir: &Path) Result~Self, InfrastructureError~
        +path() &Path
        +cleanup() Result~(), InfrastructureError~
    }
    
    class Drop {
        <<trait>>
        +drop(&mut self)
    }
    
    TempFileHandler ..|> Drop : implements
    
    note for TempFileHandler "RAII cleanup pattern\nAtomic file creation\nUnique filename generation"
```

## Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Creating: create_temp_file()
    Creating --> Generated: Generate unique filename
    Generated --> Written: Write data atomically
    Written --> InUse: Return TempFileHandler
    InUse --> Cleaned: cleanup() or Drop
    Cleaned --> [*]
    
    note right of Written
        Atomic creation with O_CREAT | O_EXCL
        Retries on collision
    end note
    
    note right of Cleaned
        File deleted from filesystem
        cleanup() can be called explicitly
        Drop ensures cleanup even on panic
    end note
```

## Properties

| Property | Type | Description |
|----------|------|-------------|
| `path` | `PathBuf` | Absolute path to temporary file |
| `cleaned_up` | `bool` | Flag to prevent double cleanup |

## Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `create_temp_file` | `data: &[u8], base_dir: &Path` | `Result<Self, InfrastructureError>` | Create temp file with unique name, write data |
| `path` | `&self` | `&Path` | Get path to temporary file |
| `cleanup` | `&mut self` | `Result<(), InfrastructureError>` | Explicitly delete file (called automatically on drop) |

## File Creation Process

```mermaid
flowchart TD
    Start([create_temp_file]) --> GenName[Generate unique filename]
    GenName --> FullPath[Join base_dir + filename]
    FullPath --> AtomicCreate[Atomic create: O_CREAT O_EXCL]
    AtomicCreate --> Success{Created?}
    Success -->|No, file exists| Retry{Retry < MAX_RETRIES?}
    Retry -->|Yes| GenName
    Retry -->|No| ErrMaxRetries[Err: Max retries exceeded]
    Success -->|Yes| SetPerms[Set permissions: 0600]
    SetPerms --> Write[Write data to file]
    Write --> Sync[Sync to disk]
    Sync --> Return[Ok TempFileHandler]
    
    style Return fill:#90EE90
    style ErrMaxRetries fill:#FFB6C1
```

## Unique Filename Generation

```
Format: temp_{timestamp}_{uuid}_{random}.tmp

Components:
- timestamp: Unix timestamp in nanoseconds
- uuid: UUID v4 (without hyphens)
- random: 8 random alphanumeric characters

Example: temp_1707664200123456789_550e8400e29b41d4a716446655440000_a7b3c9d2.tmp
```

```mermaid
flowchart LR
    Gen[Generate] --> Time[timestamp<br/>nanos]
    Gen --> UUID[UUID v4<br/>no hyphens]
    Gen --> Random[8 random chars<br/>alphanumeric]
    
    Time --> Combine[Combine]
    UUID --> Combine
    Random --> Combine
    
    Combine --> Filename["temp_TIME_UUID_RANDOM.tmp"]
    
    style Filename fill:#90EE90
```

## Atomic File Creation

```rust
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;

fn atomic_create(path: &Path) -> Result<File, std::io::Error> {
    OpenOptions::new()
        .write(true)
        .create_new(true) // Fails if file exists (atomic check)
        .mode(0o600)      // Owner read/write only
        .open(path)
}
```

| Flag | Purpose | Behavior |
|------|---------|----------|
| `create_new(true)` | Atomic creation | Fails if file already exists (prevents race conditions) |
| `mode(0o600)` | Secure permissions | Owner read/write only (no group/other access) |
| `write(true)` | Write access | Required to write data |

## Collision Handling

```mermaid
sequenceDiagram
    participant Handler
    participant FS as Filesystem
    participant RNG as Random Generator
    
    loop Until success or max retries
        Handler->>RNG: Generate filename
        RNG-->>Handler: unique_filename
        Handler->>FS: atomic_create(O_CREAT | O_EXCL)
        alt File exists (rare collision)
            FS-->>Handler: Err: File exists
            Handler->>Handler: Increment retry counter
        else Success
            FS-->>Handler: Ok: File created
            Handler->>FS: Write data
            Handler->>FS: Sync to disk
        end
    end
```

## Cleanup Behavior

```mermaid
flowchart TD
    Drop[Drop called] --> CheckFlag{cleaned_up?}
    CheckFlag -->|Yes| SkipCleanup[Skip: Already cleaned]
    CheckFlag -->|No| DeleteFile[Delete file]
    DeleteFile --> FileExists{File exists?}
    FileExists -->|Yes| Remove[Remove file]
    FileExists -->|No| LogWarn[Log warning: File already deleted]
    Remove --> SetFlag[Set cleaned_up = true]
    LogWarn --> SetFlag
    SetFlag --> Done[Done]
    
    ExplicitCleanup[cleanup called explicitly] --> CheckFlag
    
    style Done fill:#90EE90
    style LogWarn fill:#FFEB3B
```

## Usage Example

```rust
use std::path::Path;

// Create temporary file with data
let data = b"Test file content";
let base_dir = Path::new("/tmp");
let temp_file = TempFileHandler::create_temp_file(data, base_dir)?;

// Use the file
let path = temp_file.path();
println!("Temp file created at: {}", path.display());

// Analyze with libmagic
let result = repository.analyze_file(path)?;

// Explicit cleanup (optional, drop does this automatically)
temp_file.cleanup()?;

// Drop cleanup example
{
    let temp = TempFileHandler::create_temp_file(data, base_dir)?;
    // Use temp file...
} // temp is dropped here, file is automatically deleted
```

## Error Handling

| Error Condition | Error Type | Recovery |
|-----------------|------------|----------|
| Base directory doesn't exist | `InfrastructureError::InvalidPath` | Create directory first |
| Permission denied | `InfrastructureError::PermissionDenied` | Check directory permissions |
| Disk full | `InfrastructureError::IoError` | Free disk space |
| Max retries exceeded | `InfrastructureError::MaxRetriesExceeded` | Rare, investigate |
| Cleanup failure | `InfrastructureError::IoError` | Log warning, continue |

## Security Features

```mermaid
graph TD
    Security[Security Features]
    
    Security --> Atomic[Atomic Creation]
    Atomic --> NoRace[No race conditions<br/>O_CREAT  O_EXCL]
    
    Security --> Perms[Secure Permissions]
    Perms --> OwnerOnly[0600: Owner RW only<br/>No group/other access]
    
    Security --> Unique[Unique Filenames]
    Unique --> Unpredictable[UUID + timestamp + random<br/>Prevents prediction]
    
    Security --> AutoCleanup[Automatic Cleanup]
    AutoCleanup --> NoLeak[RAII pattern<br/>No file leakage]
    
    style Atomic fill:#90EE90
    style Perms fill:#90EE90
    style Unique fill:#90EE90
    style AutoCleanup fill:#90EE90
```

## Performance Considerations

| Aspect | Impact | Mitigation |
|--------|--------|------------|
| **Collision Probability** | Very low (UUID + timestamp + random) | Retry mechanism handles rare cases |
| **File I/O** | Disk write for every request | Use tmpfs (`/dev/shm`) for performance |
| **Cleanup Overhead** | One `unlink()` syscall | Negligible, O(1) operation |
| **Max Retries** | 10 attempts maximum | Sufficient for even high collision rates |

## Testing

```rust
#[test]
fn test_create_temp_file() {
    let temp_dir = tempdir().unwrap();
    let data = b"test data";
    
    let temp_file = TempFileHandler::create_temp_file(data, temp_dir.path()).unwrap();
    
    // File exists
    assert!(temp_file.path().exists());
    
    // Correct permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(temp_file.path()).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    }
    
    // Correct content
    let content = std::fs::read(temp_file.path()).unwrap();
    assert_eq!(content, data);
}

#[test]
fn test_automatic_cleanup() {
    let temp_dir = tempdir().unwrap();
    let data = b"test data";
    let path = {
        let temp_file = TempFileHandler::create_temp_file(data, temp_dir.path()).unwrap();
        temp_file.path().to_path_buf()
    }; // temp_file dropped here
    
    // File should be deleted
    assert!(!path.exists());
}

#[test]
fn test_explicit_cleanup() {
    let temp_dir = tempdir().unwrap();
    let data = b"test data";
    
    let mut temp_file = TempFileHandler::create_temp_file(data, temp_dir.path()).unwrap();
    let path = temp_file.path().to_path_buf();
    
    // Explicit cleanup
    temp_file.cleanup().unwrap();
    
    // File should be deleted
    assert!(!path.exists());
}

#[test]
fn test_unique_filenames() {
    let temp_dir = tempdir().unwrap();
    let data = b"test";
    
    let temp1 = TempFileHandler::create_temp_file(data, temp_dir.path()).unwrap();
    let temp2 = TempFileHandler::create_temp_file(data, temp_dir.path()).unwrap();
    
    // Different filenames
    assert_ne!(temp1.path(), temp2.path());
}
```

## Integration with Use Cases

```rust
// In AnalyzeContentUseCase
impl AnalyzeContentUseCase {
    pub fn execute(&self, request: AnalyzeContentRequest) -> Result<MagicResponse, ApplicationError> {
        // Create temporary file
        let temp_file = TempFileHandler::create_temp_file(
            request.content(),
            &self.temp_dir
        )?;
        
        // Analyze file
        let result = self.repository.analyze_file(temp_file.path())?;
        
        // temp_file dropped here, automatically cleaned up
        
        Ok(MagicResponse::from(result))
    }
}
```

## Configuration

```toml
[server]
temp_dir = "/dev/shm/magicer"  # Fast tmpfs for temp files
max_temp_file_retries = 10      # Max filename collision retries
```

## Design Rationale

- **RAII Pattern**: Automatic cleanup via `Drop` prevents file leaks
- **Atomic Creation**: `O_CREAT | O_EXCL` prevents race conditions
- **Unique Names**: Collision-resistant filename generation (timestamp + UUID + random)
- **Secure Permissions**: `0600` protects file contents from other users
- **Explicit Cleanup**: Optional `cleanup()` for early deletion
- **Error Recovery**: Retry mechanism handles rare collisions
- **Testability**: Easy to test with temporary directories

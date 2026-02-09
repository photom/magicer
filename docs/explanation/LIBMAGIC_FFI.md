# libmagic FFI Integration from Scratch

Designing and implementing safe Rust bindings to the libmagic C library using raw FFI without external crates.

## Overview

This document explains how to build libmagic bindings from scratch using Rust's FFI capabilities, covering raw C function declarations, memory safety, thread safety, error handling, and safe wrapper design.

**Design Goal:** Create safe, idiomatic Rust API over libmagic C library without using third-party binding crates.

---

## Why Build FFI from Scratch?

**Reason:** Learning exercise to understand Rust FFI in depth.

This project builds libmagic bindings from scratch as an educational exercise ("etude") to learn:
- How Rust FFI works at the lowest level
- Memory safety boundaries between Rust and C
- Ownership and lifetime management across FFI
- Error handling patterns for C libraries
- Thread safety considerations with external code

**Note:** In production, using an existing well-maintained crate like `magic` would be more appropriate.

---

## libmagic C API

Understanding the C API structure before designing bindings.

### Core C Functions

```c
// Opaque magic cookie handle
typedef struct magic_set *magic_t;

// Initialize magic library
magic_t magic_open(int flags);

// Load magic database file
int magic_load(magic_t cookie, const char *filename);

// Analyze file by path
const char *magic_file(magic_t cookie, const char *filename);

// Analyze buffer in memory
const char *magic_buffer(magic_t cookie, const void *buffer, size_t length);

// Get last error message
const char *magic_error(magic_t cookie);

// Get last error code (errno)
int magic_errno(magic_t cookie);

// Close and free resources
void magic_close(magic_t cookie);

// Set flags after creation
int magic_setflags(magic_t cookie, int flags);
```

### C Flag Constants

```c
#define MAGIC_NONE              0x0000000 // No special handling
#define MAGIC_MIME_TYPE         0x0000010 // Return MIME type
#define MAGIC_MIME_ENCODING     0x0000400 // Return MIME encoding
#define MAGIC_MIME              (MAGIC_MIME_TYPE|MAGIC_MIME_ENCODING)
#define MAGIC_ERROR             0x0000200 // Detailed error messages
#define MAGIC_SYMLINK           0x0000002 // Follow symlinks
#define MAGIC_COMPRESS          0x0000004 // Check inside compressed files
#define MAGIC_NO_CHECK_COMPRESS 0x0001000 // Don't check compression
```

### C Error Handling Pattern

```
1. Call returns NULL or -1 → Error occurred
2. Call magic_errno() → Get numeric error code
3. Call magic_error() → Get human-readable message
4. Error string owned by libmagic, do not free
```

---

## FFI Architecture

```mermaid
graph TB
    subgraph Safe Rust API
        Repository[LibmagicRepository]
        SafeWrapper[MagicCookie wrapper]
    end
    
    subgraph Unsafe FFI Layer
        RawFFI[ffi module<br/>extern C declarations]
        UnsafeImpl[Unsafe impls with safety comments]
    end
    
    subgraph C Library
        Libmagic[libmagic.so/dylib]
        Database[Magic Database Files]
    end
    
    Repository --> SafeWrapper
    SafeWrapper --> UnsafeImpl
    UnsafeImpl --> RawFFI
    RawFFI -->|dlopen/link| Libmagic
    Libmagic --> Database
    
    style Safe Rust API fill:#e1ffe1
    style Unsafe FFI Layer fill:#ffe1e1
    style C Library fill:#f5e1e1
```

### Module Structure

```
src/infrastructure/magic/
├── ffi.rs              # Raw FFI declarations (unsafe)
├── wrapper.rs          # Safe Rust wrapper types
├── error.rs            # Error type conversions
└── lib.rs              # Public API (repository impl)
```

**Responsibility Distribution:**

| Module | Contains | Safety Level |
|--------|----------|--------------|
| `ffi.rs` | Raw `extern "C"` declarations | 100% unsafe |
| `wrapper.rs` | Safe wrapper over raw FFI | Encapsulates unsafe |
| `error.rs` | Error type conversions | Safe |
| `lib.rs` | Repository implementation | Safe public API |

---

## Raw FFI Module Design

**Location:** `src/infrastructure/magic/ffi.rs`

### Opaque Pointer Type

C's `magic_t` is an opaque pointer - internal structure unknown.

```mermaid
graph LR
    C[C: magic_t] --> Opaque[Opaque Pointer]
    Opaque --> Rust1[Option 1: *mut c_void]
    Opaque --> Rust2[Option 2: Uninhabited enum]
    
    Rust2 --> Best[✓ Type-safe<br/>✓ No construction<br/>✓ Clear intent]
    
    style Best fill:#e1ffe1
```

**Design:** Use uninhabited enum for type safety.

**Rationale:**
- Prevents accidental construction
- Distinct type from other pointers
- Zero runtime cost
- Clear intent that it's opaque

### FFI Type Mapping

| C Type | Rust FFI Type | Notes |
|--------|--------------|-------|
| `int` | `std::os::raw::c_int` | Platform-specific size |
| `size_t` | `usize` | Native pointer-sized |
| `const char*` | `*const std::os::raw::c_char` | C string (null-terminated) |
| `const void*` | `*const std::os::raw::c_void` | Generic data pointer |
| `void*` | `*mut std::os::raw::c_void` | Mutable data pointer |
| `magic_t` | `*mut MagicT` | Custom opaque type |

### Linkage Declaration

Specify which C library to link against during compilation.

**Build Script Location:** `build.rs` (if needed for pkg-config)  
**Link Directive:** `#[link(name = "magic")]`

**Platform Differences:**

| Platform | Library Name | Location |
|----------|-------------|----------|
| Linux | `libmagic.so.1` | `/usr/lib/x86_64-linux-gnu/` |
| macOS | `libmagic.dylib` | `/usr/local/lib/` |
| Windows | `magic1.dll` | (Not supported for this project) |

---

## Safe Wrapper Design

**Location:** `src/infrastructure/magic/wrapper.rs`

### Ownership Model

```mermaid
graph TB
    MagicCookie[MagicCookie struct] --> Pointer[*mut MagicT]
    
    MagicCookie --> New[new method]
    MagicCookie --> Drop[Drop impl]
    
    New -->|malloc in C| Allocate[C allocates memory]
    Drop -->|magic_close| Free[C frees memory]
    
    Pointer -.->|PhantomData| Marker[!Send + !Sync]
    
    style MagicCookie fill:#e1ffe1
    style Marker fill:#ffe1e1
```

**Design Principles:**

1. **RAII Pattern** - Resource acquired in constructor, freed in destructor
2. **Ownership** - Rust owns the C resource lifetime
3. **Non-Send/Non-Sync** - libmagic is not thread-safe by default
4. **Panic Safety** - Drop runs even during panic

### MagicCookie Structure

Wraps the raw `*mut MagicT` pointer with safe Rust semantics.

**Fields:**
- `ptr: *mut MagicT` - The raw C pointer
- `_marker: PhantomData<*mut ()>` - Marks as `!Send + !Sync`

**Why PhantomData?**
- Raw pointers are `Send` and `Sync` by default in Rust
- libmagic is NOT thread-safe
- PhantomData prevents accidental cross-thread usage
- Zero runtime cost

### Constructor Pattern

```mermaid
sequenceDiagram
    participant Rust
    participant FFI
    participant C
    
    Rust->>FFI: MagicCookie::new(flags)
    FFI->>C: magic_open(flags)
    C->>C: Allocate magic_set struct
    C-->>FFI: Return magic_t pointer
    FFI->>FFI: Check for NULL
    alt Pointer is NULL
        FFI-->>Rust: Err(CreationFailed)
    else Pointer is valid
        FFI->>Rust: Ok(MagicCookie { ptr })
    end
```

**Error Handling:**
- Check for NULL pointer
- Convert to Rust `Result` type
- No error string available (cookie not yet created)

### Destructor Pattern

```mermaid
sequenceDiagram
    participant Rust
    participant Drop
    participant FFI
    participant C
    
    Rust->>Drop: Cookie goes out of scope
    Drop->>Drop: Check ptr is not null
    Drop->>FFI: magic_close(ptr)
    FFI->>C: Free magic_set struct
    C->>C: Cleanup internal state
    C-->>FFI: (void return)
    FFI-->>Drop: Complete
    Drop->>Drop: Set ptr to null
    Drop-->>Rust: Drop complete
```

**Safety Requirements:**
- Must not call `magic_close` on NULL
- Must not double-free
- Must run even during panic

### Method Wrappers

Each C function gets a safe Rust method wrapper.

**Pattern for Returning Strings:**

```mermaid
graph TB
    Method[Rust method] --> Call[Unsafe FFI call]
    Call --> Check{Pointer NULL?}
    Check -->|Yes| Error[Call magic_error]
    Check -->|No| Convert[Convert to Rust String]
    
    Error --> CreateErr[Create Rust Error]
    Convert --> CStr[CStr::from_ptr]
    CStr --> UTF8[to_str<>]
    UTF8 --> Clone[String::from]
    
    CreateErr --> Return1[Err]
    Clone --> Return2[Ok]
    
    style Error fill:#ffe1e1
    style Clone fill:#e1ffe1
```

**Safety Invariants:**

1. **Lifetime** - C string valid until next libmagic call
2. **Encoding** - Assume UTF-8 (documented libmagic behavior)
3. **Null Termination** - C strings are null-terminated
4. **Ownership** - Must copy string, C owns original

---

## Thread Safety Strategy

libmagic is not thread-safe - one cookie cannot be used concurrently.

### Problem

```mermaid
graph TB
    T1[Thread 1] --> Cookie[Shared Cookie]
    T2[Thread 2] --> Cookie
    T3[Thread 3] --> Cookie
    
    Cookie --> Crash[Race Condition<br/>Undefined Behavior]
    
    style Crash fill:#ffe1e1
```

### Solution 1: Mutex Wrapper

```mermaid
graph TB
    T1[Thread 1] --> M1[Mutex Guard]
    T2[Thread 2] --> M2[Mutex Guard]
    T3[Thread 3] --> M3[Mutex Guard]
    
    M1 --> Cookie[MagicCookie]
    M2 -.->|waits| Cookie
    M3 -.->|waits| Cookie
    
    style Cookie fill:#e1ffe1
```

**Implementation:**
- Wrap `MagicCookie` in `Arc<Mutex<MagicCookie>>`
- Each thread clones the Arc
- Mutex ensures exclusive access
- Only one thread uses libmagic at a time

**Trade-off:** Serializes all libmagic calls.

### Solution 2: Pool of Cookies

```mermaid
graph TB
    Pool[Cookie Pool] --> C1[Cookie 1]
    Pool --> C2[Cookie 2]
    Pool --> C3[Cookie 3]
    Pool --> C4[Cookie N]
    
    T1[Thread 1] --> C1
    T2[Thread 2] --> C2
    T3[Thread 3] --> C3
    
    style Pool fill:#e1ffe1
```

**Implementation:**
- Create N cookies (e.g., N = num_cpus)
- Store in `Arc<Mutex<Vec<MagicCookie>>>`
- Threads borrow/return cookies from pool
- True parallelism up to pool size

**Trade-off:** More memory, more initialization time.

### Recommended Approach

**For this project:** Single cookie with Mutex.

**Rationale:**
- Simpler implementation
- libmagic analysis is fast (10-100ms)
- Mutex contention acceptable given request rate
- Memory efficient (single database load)

---

## Async Integration with Tokio

libmagic blocks, but we need async API.

### The Problem

```mermaid
graph LR
    AsyncHandler[Async Handler] -->|await| Block[Blocking libmagic call]
    Block -->|blocks thread| Bad[Async runtime blocked!]
    
    style Bad fill:#ffe1e1
```

**Issue:** Blocking calls on async runtime threads cause:
- Runtime stalls
- Request timeouts
- Poor throughput

### The Solution

```mermaid
sequenceDiagram
    participant Handler as Async Handler
    participant Runtime as Tokio Runtime
    participant BlockPool as Blocking Thread Pool
    participant Libmagic as libmagic
    
    Handler->>Runtime: Call analysis method
    Runtime->>BlockPool: spawn_blocking(closure)
    BlockPool->>BlockPool: Execute on OS thread
    BlockPool->>Libmagic: magic_buffer()
    Note over Libmagic: Blocking C call
    Libmagic-->>BlockPool: Result
    BlockPool-->>Runtime: Future resolves
    Runtime-->>Handler: await completes
```

**Implementation Pattern:**

Wrap blocking call in `tokio::task::spawn_blocking`.

**Benefits:**
- Async runtime never blocks
- Automatic thread pool management
- Graceful backpressure
- Work-stealing efficiency preserved

---

## Error Handling Architecture

### C Error Model

```mermaid
graph TB
    Call[C Function Call] --> Check{Return Value}
    
    Check -->|NULL/-1| Error[Error Path]
    Check -->|Valid| Success[Success Path]
    
    Error --> Errno[Call magic_errno<>]
    Error --> Msg[Call magic_error<>]
    
    Errno --> Code[Error Code int]
    Msg --> Str[Error Message *char]
    
    Code --> Rust[Map to Rust]
    Str --> Rust
    
    style Error fill:#ffe1e1
    style Success fill:#e1ffe1
```

### Rust Error Type

Custom error enum matching domain needs.

**Design:**

```mermaid
classDiagram
    class MagicError {
        <<enum>>
        CreationFailed
        DatabaseLoad(String)
        AnalysisFailed(String)
        InvalidPath(String)
        InvalidUtf8
        NullPointer
    }
    
    class Error {
        <<trait>>
    }
    
    MagicError ..|> Error
    
    style MagicError fill:#ffe1e1
```

**Variant Meanings:**

| Variant | When | C Source |
|---------|------|----------|
| `CreationFailed` | `magic_open` returns NULL | Initialization error |
| `DatabaseLoad` | `magic_load` returns -1 | File not found, corrupt DB |
| `AnalysisFailed` | `magic_file/buffer` returns NULL | Analysis error |
| `InvalidPath` | Path not UTF-8 or invalid | Path handling |
| `InvalidUtf8` | C string not valid UTF-8 | String conversion |
| `NullPointer` | Unexpected NULL | Defensive programming |

### Error Conversion Flow

```mermaid
graph TB
    C[C Error] --> Check{Error Type}
    
    Check -->|ENOENT| FileNotFound
    Check -->|EINVAL| InvalidArg
    Check -->|ENOMEM| OutOfMemory
    Check -->|Other| Generic
    
    FileNotFound --> DB[DatabaseLoad]
    InvalidArg --> Analysis[AnalysisFailed]
    OutOfMemory --> Creation[CreationFailed]
    Generic --> Analysis
    
    DB --> Domain[Domain Error]
    Analysis --> Domain
    Creation --> Domain
    
    style C fill:#ffe1e1
    style Domain fill:#e1f5ff
```

**Error String Handling:**

1. Check return value (NULL/-1)
2. Call `magic_error()` to get C string
3. Convert C string to Rust `String` (copy)
4. Wrap in error variant
5. Return Rust `Result`

---

## Memory Safety Guarantees

### Ownership Rules

```mermaid
graph TB
    Create[MagicCookie::new] -->|Rust owns| Pointer[*mut MagicT]
    Pointer -->|Exclusive access| Methods[Method calls]
    Methods -->|No aliasing| Safe[Memory safe]
    Pointer -->|Drop called| Free[magic_close]
    
    style Safe fill:#e1ffe1
```

**Guarantees:**

1. **Single Owner** - Only one MagicCookie owns the pointer
2. **No Aliasing** - Raw pointer not exposed publicly
3. **RAII Cleanup** - Drop always runs (even during panic)
4. **Type Safety** - Opaque type prevents misuse

### String Lifetime Safety

```mermaid
sequenceDiagram
    participant Rust
    participant C
    participant Memory
    
    Rust->>C: magic_buffer()
    C->>Memory: Return pointer to internal buffer
    Note over Memory: String in C-owned memory
    Rust->>Rust: CStr::from_ptr()
    Rust->>Rust: to_str() - validates UTF-8
    Rust->>Rust: String::from() - COPY
    Note over Rust: Now Rust owns the string
    C->>Memory: (string still in C memory)
    Rust->>C: Next call may invalidate
    C->>Memory: Overwrite buffer
```

**Safety Requirements:**

1. **Immediate Copy** - Copy C string before next call
2. **UTF-8 Validation** - Check encoding before using
3. **No Dangling** - Never store raw `*const c_char`
4. **Null Checking** - Always check for NULL

### Panic Safety

```mermaid
graph TB
    Panic[Panic Occurs] --> Unwind[Stack Unwind]
    Unwind --> Drop[Drop Called]
    Drop --> Close[magic_close]
    Close --> Free[C Memory Freed]
    
    style Free fill:#e1ffe1
```

**Guarantee:** Even if panic occurs, Drop runs and C resources are freed.

**Exception:** Panic during Drop itself (abort policy).

---

## Configuration Design

### Flags as Type-Safe Bitflags

```mermaid
classDiagram
    class MagicFlags {
        <<bitflags>>
        NONE
        MIME_TYPE
        MIME_ENCODING
        MIME
        ERROR
        SYMLINK
        COMPRESS
        NO_CHECK_COMPRESS
    }
    
    style MagicFlags fill:#e1f5ff
```

**Design:** Use `bitflags!` macro or manual bitflag impl.

**Benefits:**
- Type-safe flag combinations
- Compile-time validation
- Named constants
- Bitwise operations

### Database Path Options

```mermaid
graph TB
    Config{Database Config} --> Default[None/Default]
    Config --> Custom[Some path]
    
    Default --> System[Use system default<br/>/usr/share/misc/magic.mgc]
    Custom --> Load[Load specified file]
    
    Load --> Check{File exists?}
    Check -->|No| Error[DatabaseLoad error]
    Check -->|Yes| Success[Load database]
    
    style Success fill:#e1ffe1
    style Error fill:#ffe1e1
```

**API Design:**

- `load_database(None)` - Use default
- `load_database(Some(path))` - Custom path
- Validate path before calling C function

---

## Testing Strategy

### Unit Tests (Wrapper Layer)

Test safe wrapper without full C integration.

**Test Targets:**
- Error conversion functions
- String handling helpers
- Flag bitwise operations
- PhantomData marker properties

### Integration Tests (Full FFI)

Test with real libmagic library.

**Test Cases:**

| Test | Input | Expected |
|------|-------|----------|
| Text file | `"Hello"` | Contains "text" |
| PNG header | PNG magic bytes | "image/png" |
| Empty buffer | `[]` | No panic, some result |
| Invalid UTF-8 | Non-UTF-8 bytes | Handled gracefully |
| NULL scenarios | Edge cases | Proper error handling |

### Miri Testing

Use Rust's Miri to detect undefined behavior.

**Checks:**
- Use-after-free
- Double-free
- Uninitialized memory
- Invalid pointer dereference

**Command:** `cargo +nightly miri test`

### Valgrind Testing

Use Valgrind to check C-level memory issues.

**Checks:**
- Memory leaks
- Invalid reads/writes
- Uninitialized values

**Command:** `valgrind --leak-check=full ./target/debug/test_binary`

---

## Build System Integration

### Link Directive

Tell Rust linker to link against libmagic.

### Build Script (Optional)

**Location:** `build.rs`

**Purpose:**
- Find libmagic via pkg-config
- Set link search paths
- Verify library exists
- Generate bindings (if using bindgen)

**For Manual FFI:** Minimal build script, just link directive.

---

## Security Considerations

### Input Validation

```mermaid
graph TB
    Input[User Input] --> Size{Size Check}
    Size -->|> 100MB| Reject1[Reject]
    Size -->|OK| Null{Null Bytes?}
    Null -->|Yes| Reject2[Reject]
    Null -->|No| Timeout{Timeout?}
    Timeout -->|> 30s| Reject3[Cancel]
    Timeout -->|OK| Process[Process with libmagic]
    
    style Reject1 fill:#ffe1e1
    style Reject2 fill:#ffe1e1
    style Reject3 fill:#ffe1e1
    style Process fill:#e1ffe1
```

**Checks Before FFI:**
1. Size limit (prevent resource exhaustion)
2. Timeout enforcement (prevent infinite loops)
3. Path validation (prevent directory traversal)

### Attack Vectors

| Attack | Mitigation |
|--------|-----------|
| Buffer overflow in libmagic | Use latest libmagic version |
| Malformed file causes crash | Timeout + error handling |
| Resource exhaustion | Size limits + connection limits |
| Directory traversal | Path sandbox validation |
| Use-after-free | Rust ownership prevents |

---

## Performance Optimization

### Bottleneck Analysis

```mermaid
graph LR
    Request[Request] --> Parse[HTTP Parse<br/>~1ms]
    Parse --> FFI[FFI Call<br/>~0.1ms]
    FFI --> Analysis[libmagic Analysis<br/>10-100ms]
    Analysis --> Convert[String Convert<br/>~0.1ms]
    Convert --> Response[Format Response<br/>~1ms]
    
    style Analysis fill:#ffe1e1
```

**Main Bottleneck:** libmagic analysis itself, not FFI overhead.

### Optimization Strategies

1. **Database Caching** - libmagic caches compiled database
2. **Parallel Processing** - Multiple cookies for parallelism
3. **Timeout Enforcement** - Prevent runaway analysis
4. **Streaming** - Don't buffer entire file if possible

---

## Related Documentation

- [Architecture Design](ARCHITECTURE.md) - Infrastructure layer structure
- [Project Structure](../reference/PROJECT_STRUCTURE.md) - Code organization
- [Testing Strategy](../reference/TESTING_STRATEGY.md) - Testing approach

---

## References

- [Rust FFI Documentation](https://doc.rust-lang.org/nomicon/ffi.html)
- [libmagic Man Page](https://man7.org/linux/man-pages/man3/libmagic.3.html)
- [Tokio Blocking Documentation](https://tokio.rs/tokio/topics/bridging)
- [The Rustonomicon - FFI](https://doc.rust-lang.org/nomicon/ffi.html)

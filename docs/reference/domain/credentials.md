# Credentials Value Object Class Diagram

## Overview

The `BasicAuthCredentials` value object encapsulates username and password for HTTP Basic Authentication with validation.

## Class Diagram

```mermaid
classDiagram
    class BasicAuthCredentials {
        -String username
        -SecureString password
        +new(username: String, password: String) Result~Self, ValidationError~
        +username() &str
        +password() &str
        +verify(username: &str, password: &str) bool
    }
    
    class SecureString {
        <<wrapper>>
        -String inner
        +new(value: String) Self
        +as_str() &str
    }
    
    class ValidationError {
        <<enumeration>>
        EmptyUsername
        EmptyPassword
        UsernameTooLong(usize)
        PasswordTooShort(usize)
        UsernameContainsColon
        UsernameInvalidUtf8
        PasswordInvalidUtf8
    }
    
    BasicAuthCredentials *-- SecureString : contains
    BasicAuthCredentials ..> ValidationError : creates
    
    note for BasicAuthCredentials "Immutable value object\nDerives: Clone, Debug\nNo PartialEq (use verify)\nConstant-time comparison"
    
    note for SecureString "Zeroize on drop\nNo Debug (security)\nNo Display"
```

## Validation Rules

```mermaid
flowchart TD
    Start([new username, password]) --> CheckUserEmpty{Username empty?}
    CheckUserEmpty -->|Yes| ErrUserEmpty[ValidationError::EmptyUsername]
    CheckUserEmpty -->|No| CheckUserLen{Username ≤ 256?}
    CheckUserLen -->|No| ErrUserLen[ValidationError::UsernameTooLong]
    CheckUserLen -->|Yes| CheckColon{Contains ':'?}
    CheckColon -->|Yes| ErrColon[ValidationError::UsernameContainsColon]
    CheckColon -->|No| CheckPassEmpty{Password empty?}
    CheckPassEmpty -->|Yes| ErrPassEmpty[ValidationError::EmptyPassword]
    CheckPassEmpty -->|No| CheckPassLen{Password ≥ 8?}
    CheckPassLen -->|No| ErrPassLen[ValidationError::PasswordTooShort]
    CheckPassLen -->|Yes| Success([Ok BasicAuthCredentials])
    
    style Success fill:#90EE90
    style ErrUserEmpty fill:#FFB6C1
    style ErrUserLen fill:#FFB6C1
    style ErrColon fill:#FFB6C1
    style ErrPassEmpty fill:#FFB6C1
    style ErrPassLen fill:#FFB6C1
```

## Properties

| Property | Type | Description |
|----------|------|-------------|
| `username` | `String` | Validated username |
| `password` | `SecureString` | Securely stored password |

## Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `new` | `username: String, password: String` | `Result<Self, ValidationError>` | Constructor with validation |
| `username` | `&self` | `&str` | Get username (public info) |
| `password` | `&self` | `&str` | Get password (internal use only) |
| `verify` | `username: &str, password: &str` | `bool` | Constant-time credential verification |

## Invariants

1. Username is non-empty
2. Username is ≤ 256 characters
3. Username does not contain `:` (Basic Auth format)
4. Username is valid UTF-8
5. Password is non-empty
6. Password is ≥ 8 characters
7. Password is valid UTF-8
8. Immutable after construction

## Security Features

```mermaid
graph TD
    Verify[verify method] --> ConstTime[Constant-time comparison]
    ConstTime --> Username[Compare username]
    ConstTime --> Password[Compare password]
    Username --> Result[Boolean result]
    Password --> Result
    
    Drop[Drop SecureString] --> Zeroize[Zeroize memory]
    Zeroize --> Prevent[Prevent password leakage]
    
    style ConstTime fill:#FFEB3B
    style Zeroize fill:#FFEB3B
    style Result fill:#90EE90
    style Prevent fill:#90EE90
```

## Usage Example

```rust
// Valid credentials
let creds = BasicAuthCredentials::new(
    "admin".to_string(),
    "securepassword123".to_string()
)?;

// Verify credentials (constant-time)
assert!(creds.verify("admin", "securepassword123"));
assert!(!creds.verify("admin", "wrongpassword"));

// Invalid: empty username
let result = BasicAuthCredentials::new("".to_string(), "password".to_string());
assert!(matches!(result, Err(ValidationError::EmptyUsername)));

// Invalid: username contains colon
let result = BasicAuthCredentials::new("user:name".to_string(), "password".to_string());
assert!(matches!(result, Err(ValidationError::UsernameContainsColon)));

// Invalid: password too short
let result = BasicAuthCredentials::new("admin".to_string(), "pass".to_string());
assert!(matches!(result, Err(ValidationError::PasswordTooShort(4))));
```

## HTTP Basic Auth Format

```
Authorization: Basic base64(username:password)

Example:
  Username: admin
  Password: secret123
  
  Encoded: Basic YWRtaW46c2VjcmV0MTIz
  
  Decoded: admin:secret123
           └───┘ └──────┘
           user  password
           
Note: Username cannot contain ':' to avoid ambiguity
```

## Constant-Time Verification

```mermaid
sequenceDiagram
    participant Client
    participant Verify
    participant Subtle
    
    Client->>Verify: verify(username, password)
    Verify->>Subtle: constant_time_eq(stored_user, input_user)
    Subtle-->>Verify: user_match (bool)
    Verify->>Subtle: constant_time_eq(stored_pass, input_pass)
    Subtle-->>Verify: pass_match (bool)
    Verify->>Verify: user_match & pass_match
    Verify-->>Client: bool (always same time)
    
    Note over Verify,Subtle: Timing attack resistant<br/>No early return on mismatch
```

## Security Considerations

| Threat | Mitigation |
|--------|------------|
| **Timing Attacks** | Constant-time comparison using `subtle` crate |
| **Memory Leakage** | `SecureString` zeroes memory on drop |
| **Format Confusion** | Forbid `:` in username (Basic Auth separator) |
| **Weak Passwords** | Minimum 8 character requirement |
| **Logging Leaks** | No `Display` trait, redacted `Debug` output |

## Design Rationale

- **HTTP Basic Auth**: Follows RFC 7617 format requirements
- **Security First**: Constant-time comparison prevents timing attacks
- **Memory Safety**: Password zeroization prevents post-use leakage
- **Type Safety**: Cannot accidentally log or display passwords
- **Validation**: Enforces security policies at construction
- **Value Object Pattern**: Immutable, self-validating, secure by default

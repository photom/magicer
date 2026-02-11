# Credentials Value Object Class Diagram <!-- omit in toc -->

- [Overview](#overview)
- [Class Diagram](#class-diagram)
- [Validation Rules](#validation-rules)
- [Properties](#properties)
- [Methods](#methods)
- [Invariants](#invariants)
- [Security Features](#security-features)
- [Usage Scenarios](#usage-scenarios)
  - [Valid Credentials](#valid-credentials)
  - [Invalid Scenarios](#invalid-scenarios)
- [HTTP Basic Auth Format](#http-basic-auth-format)
- [Constant-Time Verification](#constant-time-verification)
- [Security Considerations](#security-considerations)
- [Design Rationale](#design-rationale)

---

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

## Usage Scenarios

### Valid Credentials

When constructing BasicAuthCredentials with valid username "admin" and password "securepassword123", the value object is successfully created. The verify method returns true when called with matching credentials and false when called with incorrect credentials, using constant-time comparison to prevent timing attacks.

### Invalid Scenarios

**Empty Username:** Construction fails with EmptyUsername validation error when username is an empty string.

**Username Contains Colon:** Construction fails with UsernameContainsColon validation error when username contains a colon character (e.g., "user:name"), as this would create ambiguity in the Basic Auth format.

**Password Too Short:** Construction fails with PasswordTooShort validation error when password is less than 8 characters (e.g., "pass" with length 4).

## HTTP Basic Auth Format

The HTTP Basic Authentication header consists of the word "Basic" followed by a space and a Base64-encoded string containing the username and password separated by a colon.

**Example:**

| Component | Value |
|-----------|-------|
| Username | admin |
| Password | secret123 |
| Plain text | admin:secret123 |
| Base64 encoded | YWRtaW46c2VjcmV0MTIz |
| HTTP Header | Authorization: Basic YWRtaW46c2VjcmV0MTIz |

**Encoding Process:**

```mermaid
flowchart LR
    Username[Username: admin] --> Concat[Concatenate with colon]
    Password[Password: secret123] --> Concat
    Concat --> Plain[Plain text: admin:secret123]
    Plain --> Encode[Base64 encode]
    Encode --> Encoded[YWRtaW46c2VjcmV0MTIz]
    Encoded --> Prepend[Prepend 'Basic ']
    Prepend --> Header[Authorization: Basic YWRtaW46c2VjcmV0MTIz]
    
    style Header fill:#90EE90
```

**Decoding Process:**

```mermaid
flowchart LR
    Header[Authorization: Basic YWRtaW46c2VjcmV0MTIz] --> Remove[Remove 'Basic ' prefix]
    Remove --> Encoded[YWRtaW46c2VjcmV0MTIz]
    Encoded --> Decode[Base64 decode]
    Decode --> Plain[admin:secret123]
    Plain --> Split[Split by first colon]
    Split --> Username[Username: admin]
    Split --> Password[Password: secret123]
    
    style Username fill:#90EE90
    style Password fill:#90EE90
```

**Important Constraint:** The username cannot contain a colon character because the colon is used as the separator between username and password. This prevents ambiguity when splitting the decoded string.

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

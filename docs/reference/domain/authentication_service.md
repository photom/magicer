# AuthenticationService Trait Class Diagram

## Overview

The `AuthenticationService` trait defines the contract for credential verification with constant-time comparison requirements.

## Class Diagram

```mermaid
classDiagram
    class AuthenticationService {
        <<trait>>
        +verify_credentials(username: &str, password: &str) Result~bool, DomainError~
    }
    
    class DomainError {
        <<enumeration>>
    }
    
    AuthenticationService ..> DomainError : returns
    
    note for AuthenticationService "Trait bounds: Send + Sync\nMUST use constant-time comparison\nImplemented in infrastructure"
```

## Trait Definition

The AuthenticationService trait defines a single method verify_credentials that accepts username and password strings and returns a Result containing either a boolean (true for valid, false for invalid) or a DomainError if verification fails.

**Security Requirement:** Implementations MUST use constant-time comparison to prevent timing attacks. The comparison must always take the same amount of time regardless of whether credentials match or not.

The trait requires Send and Sync bounds to enable thread-safe usage in async contexts.

## Verification Flow

```mermaid
flowchart TD
    Input["Input: username, password"] --> Validate{Input valid?}
    Validate -->|No| ErrValidation[Err ValidationError]
    Validate -->|Yes| LoadStored[Load stored credentials]
    LoadStored --> CompareUser[Compare username<br/>constant-time]
    CompareUser --> ComparePass[Compare password<br/>constant-time]
    ComparePass --> Both{Both match?}
    Both -->|Yes| Success[Ok true]
    Both -->|No| Failure[Ok false]
    
    LoadStored -->|Error| ErrConfig[Err ConfigurationError]
    
    style Success fill:#90EE90
    style Failure fill:#FFB6C1
    style ErrValidation fill:#FFB6C1
    style ErrConfig fill:#FFB6C1
```

## Security Requirements

### Constant-Time Comparison

**CRITICAL**: Implementation MUST use constant-time comparison to prevent timing attacks.

```mermaid
sequenceDiagram
    participant Attacker
    participant Service
    participant Compare as Constant-Time Compare
    
    Note over Attacker,Compare: Attempt 1: Wrong username
    Attacker->>Service: verify("wrong", "pass")
    Service->>Compare: compare_username
    Service->>Compare: compare_password
    Compare-->>Service: false (same time)
    Service-->>Attacker: Ok(false) in T ms
    
    Note over Attacker,Compare: Attempt 2: Correct username, wrong password
    Attacker->>Service: verify("admin", "wrong")
    Service->>Compare: compare_username
    Service->>Compare: compare_password
    Compare-->>Service: false (same time)
    Service-->>Attacker: Ok(false) in T ms
    
    Note over Service,Compare: Both take ~same time<br/>No timing leak
```

### Timing Attack Mitigation

| Vulnerability | Mitigation |
|---------------|------------|
| **Early Return** | Always compare both username AND password |
| **Variable Time** | Use `subtle::ConstantTimeEq` for comparison |
| **Branch Timing** | No conditional early exit based on comparison |
| **Cache Timing** | Compare same-length strings (pad if needed) |

## Implementation Requirements

Implementations MUST:

1. **Constant-Time**: Use `subtle` crate or equivalent for comparison
2. **No Early Exit**: Always compare both username and password
3. **Thread Safety**: Implement `Send + Sync`
4. **Error Handling**: Return `Result`, never panic
5. **Secure Storage**: Store credentials securely (hashed in production)

## Error Cases

```mermaid
graph TD
    Verify[verify_credentials] --> Case1{Empty username/password?}
    Case1 -->|Yes| ErrValidation[Err ValidationError]
    Case1 -->|No| Case2{Credentials not configured?}
    Case2 -->|Yes| ErrConfig[Err ConfigurationError]
    Case2 -->|No| Compare[Constant-time compare]
    Compare --> Match{Match?}
    Match -->|Yes| Success[Ok true]
    Match -->|No| Failure[Ok false]
    
    style Success fill:#90EE90
    style Failure fill:#FFEB3B
    style ErrValidation fill:#FFB6C1
    style ErrConfig fill:#FFB6C1
```

## Trait Bounds

| Bound | Purpose |
|-------|---------|
| Send | Can be transferred between threads |
| Sync | Can be shared between threads via Arc |

## Usage Patterns

### In Presentation Layer

Middleware extracts username and password from the HTTP request, then calls the authentication service's verify_credentials method. If the result is true, the request proceeds to the next handler. If false or an error occurs, an authentication error is returned to the client.

### In Infrastructure Layer

Concrete implementations like BasicAuthService store the expected username and password (or password hash). The verify_credentials method validates that inputs are non-empty, then performs constant-time comparison of both username and password using specialized cryptographic comparison functions. Both comparisons are always executed, and the results are combined with a logical AND operation to produce the final boolean result.

## Timing Attack Prevention

### Vulnerable Approach

A vulnerable implementation checks the username first and returns immediately if it doesn't match, then checks the password and returns immediately if it doesn't match. This early return pattern leaks timing information because failed username checks complete faster than failed password checks, allowing attackers to determine which credential is incorrect based on response time.

### Secure Approach

A secure implementation always performs both username and password comparisons using constant-time comparison functions, regardless of whether the username matches. The results are combined using a logical AND operation. This ensures the function always takes approximately the same amount of time, preventing timing-based information leakage.

## Dependency Injection

```mermaid
sequenceDiagram
    participant Main
    participant Infra as BasicAuthService
    participant Middleware
    participant Trait as AuthenticationService
    
    Main->>Infra: new(config)
    Main->>Middleware: with_auth(Arc<BasicAuthService>)
    Middleware->>Trait: verify_credentials() via trait
    Trait->>Infra: concrete implementation
    Infra-->>Trait: Result<bool>
    Trait-->>Middleware: Result<bool>
    
    note over Trait: Depends on abstraction<br/>not concrete type
```

## Test Doubles

For testing purposes, a mock authentication service can be created that stores valid username and password values. The mock implements the AuthenticationService trait with the same constant-time comparison logic as production implementations. Test cases verify that the service returns true for matching credentials and false for non-matching credentials, ensuring the authentication logic works correctly in isolation.

## Design Rationale

- **Security First**: Constant-time requirement prevents timing attacks
- **Dependency Inversion**: Domain defines interface, infrastructure implements
- **Simplicity**: Single method covers all verification needs
- **Boolean Result**: `Ok(bool)` separates verification failure from system errors
- **Thread Safety**: `Send + Sync` enables async/concurrent use
- **Testability**: Easy to mock for testing authentication flows

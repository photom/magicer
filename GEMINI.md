# Gemini CLI Configuration and Rules

This document consolidates the rules and conventions from the `.cursor/rules/` directory.

---

## Rust Conventions (from convensions.mdc)

alwaysApply: true
description: Strict Rust naming & conventions enforcement using official Rust Guidelines and Rustc Dev Guide.
globs:
  - "**/*.rs"
  - "src/**/*.rs"
  - "bench/**/*.rs"
  - "tests/**/*.rs"

# Rust Conventions

All code must strictly comply with the Rust Compiler Development Guide conventions:

## **Reference**
- https://doc.rust-lang.org/edition-guide/rust-2024/index.html
- https://rustc-dev-guide.rust-lang.org/conventions.html
- https://rust-unofficial.github.io/patterns/

Follow all naming, formatting, design and code style guidelines defined in the official and unofficial documents.

## Async Conventions

Follow the documents below. If there is any conflict, Tokio’s rules take precedence.

- https://docs.rs/tokio/latest/tokio/
  - Read all linked contents
- https://rust-lang.github.io/async-book/intro.html
  - Read all linked contents
- https://marabos.nl/atomics/
  - Read all linked contents
  - https://marabos.nl/atomics/preface.html
  - https://marabos.nl/atomics/basics.html
  - https://marabos.nl/atomics/atomics.html
  - https://marabos.nl/atomics/memory-ordering.html
  - https://marabos.nl/atomics/building-spinlock.html
  - https://marabos.nl/atomics/building-channels.html
  - https://marabos.nl/atomics/building-arc.html
  - https://marabos.nl/atomics/hardware.html
  - https://marabos.nl/atomics/os-primitives.html
  - https://marabos.nl/atomics/building-locks.html
  - https://marabos.nl/atomics/inspiration.html

---

## Design Conventions (from design-conventions.mdc)

alwaysApply: true
description: DDD, Clean Architecture, and SOLID Principles enforcement for Rust projects.

# Design Conventions

## References
- Domain-Driven Design (Eric Evans)
- Clean Architecture (Robert C. Martin)
- SOLID Principles

## Layered Architecture (Clean Architecture)

Organize code into four layers. Dependencies MUST point inward only.

```
presentation/ (API handlers, CLI, request/response types)
  └─ depends on ─> application/
application/ (use cases, application services, DTOs)
  └─ depends on ─> domain/
domain/ (entities, value objects, aggregates, repository traits, domain services, domain errors)
infrastructure/ (repository impls, external clients, I/O adapters)
  └─ implements traits from ─> domain/
  └─ depends on ─> domain/
```

- **Domain**: Pure business logic. Zero external dependencies — no frameworks, no I/O crates, no async runtime types in public API. Only `std` and domain-specific crates.
- **Application**: Orchestrates domain objects via use cases. Depends only on domain. Receives infrastructure through trait objects.
- **Infrastructure**: Implements traits defined in the domain layer (repositories, gateways). Contains all I/O, serialization, and framework-specific code.
- **Presentation**: Entrypoint layer. Wires dependencies together, translates external requests into application calls.

## SOLID Principles (Rust-Specific)

- **Single Responsibility**: Each module, struct, and function has exactly one reason to change. Split files exceeding ~300 lines.
- **Open/Closed**: Extend behavior by adding new trait implementations, not by modifying existing code. Use generics and trait bounds for extensibility.
- **Liskov Substitution**: Every trait implementation must fully honor the trait's documented contract, invariants, and error semantics. Never panic where the trait expects `Result`.
- **Interface Segregation**: Define small, focused traits (1–3 methods). Prefer multiple narrow traits over one broad trait. Compose with supertraits only when there is a true "is-a" relationship.
- **Dependency Inversion**: Depend on traits (abstractions), never on concrete types across layer boundaries. Accept dependencies as generic parameters bounded by traits or as `dyn Trait` objects.

## DDD Building Blocks (Rust Idioms)

| Building Block   | Rust Idiom |
|------------------|------------|
| **Entity**       | Struct with an `id` field. `PartialEq`/`Eq` compares identity only. |
| **Value Object** | Struct with no identity. Derive `Eq`, `Clone`, `Hash` on all fields. Prefer immutable (no `&mut self` methods). |
| **Aggregate**    | Module containing a root entity + related entities/value objects. External code accesses only the root. The root enforces all invariants. |
| **Repository**   | `trait` in `domain/`. Async methods return domain types and domain errors. Implementations live in `infrastructure/`. |
| **Domain Service** | Stateless function or struct in `domain/`. Encapsulates logic that spans multiple entities. |
| **Domain Event** | Struct representing a past occurrence. Use for cross-aggregate or cross-bounded-context communication. |
| **Domain Error** | Enum in `domain/`. Infrastructure errors are mapped into domain errors at the boundary. Never expose `sqlx::Error`, `std::io::Error`, etc. to the domain. |

## Enforced Rules

1. **No inward leakage**: Infrastructure types (DB rows, HTTP types, framework errors) MUST NOT appear in domain or application layers.
2. **Trait boundaries**: Repository and gateway traits live in `domain/`; implementations live in `infrastructure/`.
3. **Error mapping**: Convert infrastructure errors into domain errors at the infrastructure boundary using `From` or `map_err`.
4. **Constructor injection**: Pass all dependencies (as trait objects or generics) through constructors — never use global state, lazy statics, or service locators.
5. **Aggregate invariants**: All mutations go through aggregate root methods that validate invariants before applying changes.
6. **No cyclic dependencies**: Modules within a layer MUST NOT have circular `use` imports. Use domain events or mediators to decouple.

---

## Documentation Conventions (from documentation.mdc)

alwaysApply: true

# References
**Reference**: https://diataxis.fr/


## Documentation

- When specification or design is changed, update documents.
- Update the documents first, then update the code.
- Documentation must comply with Diátaxis.
- Documentations are located in docs.
- Explanation sentences must describe concisely.
- Do not write programming codes in documents under docs. Alternatively, describe design with sentences and diagram.
- Use mermaid diagram in order to describe design.

---

## Implementation Conventions (from implement-conventions.mdc)

alwaysApply: true
description: Comply with KENT BECK TDD implementation process.

# Implementation Conventions

## References
- Kent Beck, "Test-Driven Development: By Example" (2003)
- Kent Beck, "Canon TDD" - Software Design: Tidy First
- Kent Beck TDD principles: https://substack.com/home/post/p-139601698
- Three Rules of TDD (Robert C. Martin, based on Kent Beck's work)

---

## Test-Driven Development (TDD) Process

### The Red-Green-Refactor Cycle

Implementation MUST follow Kent Beck's TDD cycle strictly:

```
1. RED: Write a failing test
   ↓
2. GREEN: Make it pass (simplest way)
   ↓
3. REFACTOR: Improve design while keeping tests green
   ↓
   (Repeat)
```

**Cycle Rules:**

1. **RED Phase:**
   - Write ONE failing test before any production code
   - Test must fail for the right reason (not compilation error)
   - Test describes the next small behavior increment
   - Run the test and verify it fails

2. **GREEN Phase:**
   - Write ONLY enough code to make the test pass
   - Simplest implementation wins (even if "wrong")
   - No premature optimization
   - No "future-proofing"
   - Run all tests and verify they pass

3. **REFACTOR Phase:**
   - Remove duplication
   - Improve names and structure
   - Apply design patterns only when needed
   - Keep all tests green throughout refactoring
   - Run tests after each refactoring step

### The Three Rules of TDD

You MUST follow these rules strictly (based on Kent Beck's work, formalized by Robert C. Martin):

1. **You are not allowed to write any production code unless it is to make a failing unit test pass.**
   - No "scaffolding" code before tests
   - No "preparation" code before tests
   - Exception: Infrastructure setup required to write tests (e.g., trait definitions for mocking)

2. **You are not allowed to write any more of a unit test than is sufficient to fail; and compilation failures are failures.**
   - Stop writing test as soon as it fails (including compilation)
   - One assertion at a time (typically)
   - Build test incrementally

3. **You are not allowed to write any more production code than is sufficient to pass the one failing unit test.**
   - Simplest implementation first
   - No extra methods
   - No extra features
   - No "while I'm here" improvements

## TDD Implementation Workflow

### Implementation Process: Test List First

**Making the test list is the first step in the design process.**

- **Test abstracts** are written in design documents in `docs/` (e.g., `TESTING_STRATEGY.md`, `ARCHITECTURE.md`)
- **Detailed test item lists** are written to documents in the `tests/` directory alongside test implementations

This ensures test requirements are documented before implementation begins, following TDD principles where tests drive design.


### Step 1: Make Test List (First Step)

Before any design or implementation, create a comprehensive test list that describes all behaviors to be tested.

**Test List Organization:**

1. **Design Documents (docs/):**
   - Write **test abstracts** (high-level test scenarios)
   - Describe **what** needs to be tested conceptually
   - Group tests by component/feature
   - Focus on behavior and requirements
   - Location: `docs/reference/TESTING_STRATEGY.md`

2. **Test Directory (tests/):**
   - Write **detailed test item lists** (specific test cases)
   - Describe **how** to test each scenario
   - Include setup, execution, and assertion details
   - Provide concrete test data examples
   - Location: `tests/test_plans/*.md` or test comments

**Test List Format:**

**In Design Documents (Abstract):**
```
## Component: Temporary File Handler

### Test Scenarios:
- [ ] Creates temp file with unique name
- [ ] Retries on filename collision
- [ ] Fails after max retries
- [ ] Cleans up on drop
- [ ] Sets correct permissions (0600)
```

**In Test Directory (Detailed):**
```
## test_create_temp_file_with_unique_name

**Setup:**
- Initialize temp directory
- Clear any existing temp files

**Execution:**
- Call create_temp_file()
- Extract filename from path

**Assertions:**
- File exists at returned path
- Filename contains UUID v4
- Filename contains timestamp
- Filename contains random suffix
- File permissions are 0600
```

**Test List Workflow:**

1. **Design Phase:** Write test abstracts in design documents
2. **Planning Phase:** Expand abstracts to detailed test items in tests/
3. **Implementation Phase:** Follow TDD using the detailed test list
4. **Verification Phase:** Check all items are implemented and passing

---
### Step-by-Step Process

**Before Writing Code:**

1. ✅ Read the requirement/specification
2. ✅ Identify the next smallest behavior to implement
3. ✅ Write a test name describing that behavior
4. ❌ Do NOT write production code yet

**RED Phase (Write Failing Test):**

1. Write test method signature and name
2. Write test setup (arrange)
3. Write the action (act)
4. Write the assertion (assert)
5. Run test → verify it FAILS
6. If test passes unexpectedly → delete and rethink
7. If test fails for wrong reason → fix test

**GREEN Phase (Make It Pass):**

1. Write simplest code to pass the test
2. Use "fake it" → return hard-coded value if appropriate
3. Use "obvious implementation" only if truly obvious
4. Run test → verify it PASSES
5. If test still fails → fix implementation
6. Run ALL tests → verify all pass

**REFACTOR Phase (Clean Up):**

1. Look for duplication (test and production code)
2. Look for poor names
3. Look for long methods
4. Look for complex conditionals
5. Apply one refactoring
6. Run tests → verify all pass
7. Repeat until code is clean
8. Commit when all tests green

### Small Steps Discipline

Kent Beck emphasizes taking small steps. Each cycle should be:

- **Duration:** 2-10 minutes maximum
- **Test:** Tests one behavior/assertion
- **Production Code:** 1-10 lines typically
- **Refactoring:** One small improvement

**If a step takes > 10 minutes:**
- The step is too large
- Revert (git reset --hard)
- Take a smaller step

---

## Test Structure and Design

### Test Organization (AAA Pattern)

```
// Arrange: Set up test data and dependencies
// Act: Execute the behavior being tested
// Assert: Verify the outcome
```

**Rules:**
- One test = one behavior
- Test name describes behavior, not implementation
- Arrange and Assert separated by blank line for readability
- Each phase clearly identifiable

### Test Naming Convention

**Format:** `test_[method]_[scenario]_[expected_behavior]`

**Examples:**
- `test_analyze_buffer_with_empty_data_returns_error`
- `test_create_temp_file_with_collision_retries_with_new_name`
- `test_mmap_with_deleted_file_returns_sigbus_error`

**Naming Principles:**
- Describes WHAT, not HOW
- Readable as documentation
- Expresses intent clearly
- No "test1", "test2", etc.

### Test Independence

Each test MUST be independent:

- ✅ Can run in any order
- ✅ Can run in parallel
- ✅ No shared mutable state
- ✅ Each test has own setup
- ❌ No test depends on another test's outcome
- ❌ No global state modifications

**Setup:**
- Use `#[tokio::test]` for async tests
- Use test fixtures for common setup
- Create fresh test data for each test

**Teardown:**
- Use RAII (Drop trait) for cleanup
- Use `#[serial]` only when absolutely necessary (database tests)
- Prefer isolated resources (temp files with unique names)

---

## TDD-Specific Implementation Patterns

### Fake It Till You Make It

Start with fake/hard-coded implementation, then generalize.

**Example Progression:**

**Test 1:**
```rust
#[test]
fn test_sum_returns_zero_for_empty_array() {
    assert_eq!(sum(&[]), 0);
}
```

**Implementation 1 (Fake It):**
```rust
fn sum(numbers: &[i32]) -> i32 {
    0  // Hard-coded to pass test
}
```

**Test 2:**
```rust
#[test]
fn test_sum_returns_single_element() {
    assert_eq!(sum(&[5]), 5);
}
```

**Implementation 2 (Still Faking):**
```rust
fn sum(numbers: &[i32]) -> i32 {
    if numbers.is_empty() { 0 } else { numbers[0] }
}
```

**Test 3:**
```rust
#[test]
fn test_sum_returns_total_of_multiple_elements() {
    assert_eq!(sum(&[1, 2, 3]), 6);
}
```

**Implementation 3 (Real Implementation):**
```rust
fn sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}
```

### Triangulation

Use multiple examples to drive toward generalization.

**When to use:**
- When the right abstraction is unclear
- When "fake it" becomes too complex
- When you need confidence in the general solution

**Process:**
1. Write test with example 1
2. Implement specifically for example 1
3. Write test with example 2 (different data)
4. Generalize implementation to handle both
5. Write test with example 3 if needed
6. Refactor to final general solution

### Obvious Implementation

When the implementation is truly obvious, write it directly.

**Criteria for "obvious":**
- < 5 lines
- No conditionals
- No loops
- No error handling complexity
- You're confident it will pass first try

**If implementation fails test:**
- It wasn't obvious
- Revert
- Use "fake it" or "triangulation" instead

---

## Refactoring Discipline

### When to Refactor

Refactor when you see:

1. **Duplication** - Same code/logic in multiple places
2. **Long methods** - Methods > 20 lines (typically)
3. **Long parameter lists** - > 3-4 parameters
4. **Complex conditionals** - Nested ifs, long boolean expressions
5. **Poor names** - Unclear, misleading, or outdated names
6. **Feature envy** - Method uses another class's data more than its own
7. **Data clumps** - Same group of data passed together

### How to Refactor Safely

**The TDD Refactoring Protocol:**

1. All tests are GREEN before starting
2. Make one small change
3. Run all tests
4. If RED: revert the change immediately
5. If GREEN: commit or continue to next refactoring
6. Repeat

**Common Refactorings:**

| Refactoring | When | How |
|-------------|------|-----|
| Extract Method | Long method, duplicated code | Extract to named method |
| Rename | Poor name | Change name everywhere |
| Extract Variable | Complex expression | Assign to well-named variable |
| Inline | Unnecessary indirection | Replace call with method body |
| Move Method | Feature envy | Move to class that owns the data |
| Replace Conditional with Polymorphism | Type checking | Use trait dispatch |

**Refactoring Size:**
- One refactoring at a time
- Run tests after each
- Commit frequently when green

---

## Test Coverage and Quality

### Coverage Requirements

- **Domain Layer:** 100% coverage (pure logic, no I/O)
- **Application Layer:** 95%+ coverage (orchestration)
- **Infrastructure Layer:** 80%+ coverage (contains I/O, harder to test)
- **Presentation Layer:** Integration tests (E2E coverage)

**Coverage Tools:**
```bash
cargo tarpaulin --out Html
```

### Test Quality Criteria

**Good Test:**
- ✅ Fast (< 100ms typically)
- ✅ Isolated (no external dependencies)
- ✅ Repeatable (same result every time)
- ✅ Self-validating (pass/fail, no manual checking)
- ✅ Timely (written before production code)
- ✅ Readable (clear intent and structure)

**Bad Test:**
- ❌ Slow (> 1 second)
- ❌ Flaky (sometimes passes, sometimes fails)
- ❌ Coupled (depends on other tests or order)
- ❌ Unclear (hard to understand what it tests)
- ❌ Brittle (breaks with minor implementation changes)

### Test Doubles

Use appropriate test doubles for isolation:

| Type | Purpose | Rust Implementation |
|------|---------|---------------------|
| **Mock** | Verify behavior (interactions) | `mockall` crate |
| **Stub** | Provide canned responses | Simple impl of trait |
| **Fake** | Working implementation (simpler) | In-memory implementation |
| **Spy** | Record calls for verification | `mockall` with expectations |
| **Dummy** | Fill parameters (unused) | Default::default() |

**Prefer Fakes over Mocks:**
- Fakes test behavior, not implementation
- Fakes are more maintainable
- Use mocks only when verifying interactions is critical

---

## Integration with Clean Architecture

### Testing Strategy by Layer

**Domain Layer (100% TDD):**
- Pure unit tests
- No mocking needed (pure functions)
- Test all business rules
- Test value object validation
- Test entity invariants

**Application Layer (TDD with Mocks):**
- Mock repository traits
- Mock domain services
- Test orchestration logic
- Test error handling
- Test transaction boundaries

**Infrastructure Layer (TDD + Integration):**
- Unit test with mocks for external dependencies
- Integration tests for actual I/O
- Test error mapping (infrastructure → domain)
- Test resource cleanup (RAII)

**Presentation Layer (Behavior/E2E):**
- HTTP integration tests
- Test API contracts
- Test error responses
- Test middleware behavior

### Dependency Direction

```
Tests always test inward:
  
Presentation Tests → Presentation Layer
Application Tests → Application Layer → Domain Layer
Infrastructure Tests → Infrastructure Layer (→ Domain traits)
Domain Tests → Domain Layer (pure)
```

**Rules:**
- Domain tests never depend on other layers
- Application tests mock infrastructure via traits
- Infrastructure tests test concrete implementations
- Presentation tests may use real or test doubles

---

## TDD Workflow Best Practices

### The Programmer's Oath (Kent Beck)

When practicing TDD:

1. **I will write no production code except to pass a failing test**
2. **I will write only enough of a test to demonstrate a failure**
3. **I will write only enough production code to pass the test**

### Red-Green-Refactor Checklist

**RED:**
- [ ] Written test that describes next behavior
- [ ] Test compiles
- [ ] Test runs and FAILS
- [ ] Failure is for the expected reason

**GREEN:**
- [ ] Written simplest code to pass test
- [ ] Test runs and PASSES
- [ ] All tests run and PASS
- [ ] No shortcuts taken (all tests green)

**REFACTOR:**
- [ ] Identified duplication or poor design
- [ ] Applied one refactoring
- [ ] All tests still PASS
- [ ] Code is cleaner than before
- [ ] Ready to commit or continue refactoring

### Velocity and Rhythm

**Maintain Steady Rhythm:**
- Each cycle: 2-10 minutes
- Commit every 10-30 minutes (all tests green)
- If stuck > 15 minutes: revert, take smaller step
- If confident: take bigger steps
- If uncertain: take smaller steps

**Signs of Good TDD Rhythm:**
- Frequent test runs (every 1-3 minutes)
- Frequent commits (tests always green)
- Steady progress (small increments)
- Few debugging sessions (tests catch issues)

**Signs of Poor TDD Rhythm:**
- Writing lots of production code before running tests
- Long debugging sessions
- Infrequent commits
- Skipping refactoring phase
- Tests always passing (not writing failing tests first)

---

## Common TDD Anti-Patterns to Avoid

### Anti-Pattern: Writing Tests After Code

**Wrong:**
```rust
// 1. Write implementation first
fn calculate_total(items: &[Item]) -> u64 { ... }

// 2. Then write tests
#[test]
fn test_calculate_total() { ... }
```

**Correct:**
```rust
// 1. Write test first
#[test]
fn test_calculate_total_with_empty_list_returns_zero() {
    assert_eq!(calculate_total(&[]), 0);
}

// 2. Then write minimal implementation
fn calculate_total(items: &[Item]) -> u64 {
    0  // Just enough to pass
}
```

### Anti-Pattern: Too Much Code Before Running Tests

**Wrong:**
```rust
// Writing entire implementation before running test
fn process_request(req: Request) -> Response {
    let validated = validate(req);
    let authorized = authorize(validated);
    let processed = process(authorized);
    let result = format_response(processed);
    log_result(&result);
    result
}
```

**Correct:**
```rust
// Write tiny increment, run test
fn process_request(req: Request) -> Response {
    validate(req)  // Just first step, test it
}
```

### Anti-Pattern: Skipping Refactor Phase

**Wrong:**
```rust
// Test passes, move to next feature immediately
// Technical debt accumulates
```

**Correct:**
```rust
// Test passes, clean up before moving on:
// - Remove duplication
// - Improve names
// - Simplify structure
// - THEN move to next feature
```

### Anti-Pattern: Testing Implementation Details

**Wrong:**
```rust
#[test]
fn test_uses_hashmap_internally() {
    let cache = Cache::new();
    assert!(cache.storage.is_empty());  // Testing internal structure
}
```

**Correct:**
```rust
#[test]
fn test_get_returns_none_for_missing_key() {
    let cache = Cache::new();
    assert_eq!(cache.get("key"), None);  // Testing behavior
}
```

---

## Commit Strategy with TDD

### When to Commit

Commit when:
- ✅ All tests are GREEN
- ✅ Code is refactored and clean
- ✅ One complete feature increment done
- ✅ Typically every 10-30 minutes

**Never commit when:**
- ❌ Tests are RED
- ❌ Tests are commented out
- ❌ Code is incomplete
- ❌ Refactoring is half-done

### Commit Message Format

```
<type>: <short description>

- Test: <what behavior was tested>
- Implementation: <what was added>
- Refactoring: <what was cleaned up>
```

**Example:**
```
feat: add temporary file atomic creation

- Test: temp file creation retries on collision
- Test: temp file creation fails after max retries
- Implementation: O_CREAT | O_EXCL atomic creation
- Implementation: retry logic with new filename
- Refactoring: extract filename generation
```

---

## Summary: The TDD Mindset

### Core Principles

1. **Tests Drive Design:** Tests written first shape better APIs
2. **Small Steps:** Progress in tiny, verified increments
3. **Constant Feedback:** Run tests every few minutes
4. **Refactor Fearlessly:** Tests provide safety net
5. **Simple Solutions First:** YAGNI (You Aren't Gonna Need It)
6. **Clean Code:** Refactor to keep code maintainable

### The TDD Mantra

```
Red → Green → Refactor
Test First → Simplest Code → Clean Code
```

### When TDD is Working

You know TDD is working when:
- ✅ You feel confident making changes
- ✅ Refactoring is safe and frequent
- ✅ Bugs are rare (caught by tests)
- ✅ Design emerges naturally
- ✅ Code is well-factored
- ✅ Documentation (tests) is always up-to-date

### Resources

- Kent Beck, "Test-Driven Development: By Example" (2003)
- Kent Beck, "Extreme Programming Explained" (2004)
- Robert C. Martin, "Clean Code" (2008)
- Martin Fowler, "Refactoring" (2018)

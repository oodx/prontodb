# Phase 2a: TDD Development Protocol

## Introduction

**Purpose**: Build features test-first. Tests define the contract, implementation follows.

**When to Use**: 
- Clear requirements that can be expressed as tests
- Algorithmic/business logic
- Bug fixes (reproduce bug as test first)
- AI development (perfect constraint mechanism)

**Not For**: Exploratory UI, prototypes, integration glue

---

## TDD Roles

### Role Adaptation from Base Phase 2
```
Developer = Test Writer → Implementer → Refactorer
- First, write the test (specification)
- Then, write minimal code (implementation)  
- Finally, clean up (refactoring)
```

### AI Assistant Mode
```
AI = Test-Driven Developer
Human = Test Reviewer
AI must make tests pass, nothing more
```

---

## Quick Start Guide

### 1. TDD Setup (Same branch strategy as Phase 2)
```bash
git checkout -b feature/[component-name]

# Create test file structure FIRST
touch src/lib.rs
touch src/component.rs
touch src/component_test.rs  # Tests live with code
```

### 2. Set Context
```markdown
You are in DEVELOPMENT PHASE - TDD Variant.
Constraint: RED → GREEN → REFACTOR → COMMIT
Rule: No code without failing test first
Output: Test-driven implementation
```

### 3. The TDD Loop
```markdown
For each requirement:
1. Write test that fails (RED)
2. Write minimal code to pass (GREEN)  
3. Refactor if needed (REFACTOR)
4. Commit test + code together
5. Next requirement
```

---

## Deep Dive: TDD Process

### Stage 1: Test-First Design

**Convert Requirement to Test:**
```markdown
Requirement: "Retry should use exponential backoff"

Test Design:
- What's the input? (attempt number)
- What's the output? (delay duration)
- What's the behavior? (doubles each time)
```

```rust
// The test IS the specification
#[test]
fn exponential_backoff_doubles_delay() {
    let backoff = ExponentialBackoff::new(
        Duration::from_millis(100)
    );
    
    assert_eq!(backoff.delay(0), Duration::from_millis(100));
    assert_eq!(backoff.delay(1), Duration::from_millis(200));
    assert_eq!(backoff.delay(2), Duration::from_millis(400));
}
```

### Stage 2: RED Phase (Test Must Fail)

**Verify Test Actually Tests Something:**
```bash
cargo test exponential_backoff_doubles_delay
# MUST see: test result: FAILED
# If passes without code, test is wrong!
```

**Document What We're Testing:**
```rust
#[test]
fn exponential_backoff_doubles_delay() {
    // TESTING: Each retry doubles the delay
    // INPUT: Attempt number 0, 1, 2
    // EXPECTS: 100ms, 200ms, 400ms
    let backoff = ExponentialBackoff::new(
        Duration::from_millis(100)
    );
    // ... assertions
}
```

### Stage 3: GREEN Phase (Minimal Pass)

**Write ONLY What Makes Test Pass:**
```rust
// ❌ WRONG - Too much
struct ExponentialBackoff {
    base: Duration,
    max: Duration,      // Test doesn't need this
    factor: f64,        // Test doesn't need this
    jitter: bool,       // Test doesn't need this
}

// ✅ RIGHT - Just enough
struct ExponentialBackoff {
    base: Duration,
}

impl ExponentialBackoff {
    fn new(base: Duration) -> Self {
        Self { base }
    }
    
    fn delay(&self, attempt: u32) -> Duration {
        self.base * 2_u32.pow(attempt)
    }
}
```

### Stage 4: REFACTOR Phase (Clean Up)

**Only After Green:**
```rust
// Now we can improve, but test must still pass
impl ExponentialBackoff {
    fn delay(&self, attempt: u32) -> Duration {
        // Refactor for overflow safety
        let multiplier = 2_u64.pow(attempt.min(63));
        self.base.saturating_mul(multiplier as u32)
    }
}

// Run test - still passes? Good!
```

### Stage 5: COMMIT Phase (Checkpoint)

**Test + Implementation Together:**
```bash
git add src/component.rs src/component_test.rs
git commit -m "test+impl: exponential backoff doubles delay

- Test verifies doubling behavior
- Implementation uses power of 2
- Handles overflow safely"
```

---

## TDD-Specific Patterns

### Pattern: Triangulation
Start with specific test, generalize through multiple tests:

```rust
// Test 1: Specific case
#[test]
fn retry_after_500_error() {
    assert!(should_retry(Error::Status(500)));
}

// Test 2: Another specific
#[test]  
fn retry_after_502_error() {
    assert!(should_retry(Error::Status(502)));
}

// Test 3: Now generalize
#[test]
fn retry_after_5xx_errors() {
    for code in 500..600 {
        assert!(should_retry(Error::Status(code)));
    }
}

// Implementation evolves from specific to general
```

### Pattern: Obvious Implementation

When test is simple, implementation can be obvious:

```rust
#[test]
fn default_max_attempts_is_three() {
    let config = RetryConfig::default();
    assert_eq!(config.max_attempts, 3);
}

// Obvious implementation - just do it
impl Default for RetryConfig {
    fn default() -> Self {
        Self { max_attempts: 3 }
    }
}
```

### Pattern: Fake It Till You Make It

Start with hardcoded, evolve to real:

```rust
// First test - hardcode it
#[test]
fn first_delay_is_100ms() {
    assert_eq!(backoff.delay(0), Duration::from_millis(100));
}

impl Backoff {
    fn delay(&self, _attempt: u32) -> Duration {
        Duration::from_millis(100)  // Fake it!
    }
}

// Second test - forces real implementation
#[test]
fn second_delay_is_200ms() {
    assert_eq!(backoff.delay(1), Duration::from_millis(200));
}

impl Backoff {
    fn delay(&self, attempt: u32) -> Duration {
        // Now need real logic
        Duration::from_millis(100 * 2_u32.pow(attempt))
    }
}
```

---

## TDD with AI Assistants

### The Perfect Constraint

```markdown
INSTRUCTION: Make this test pass. Add nothing else.

#[test]
fn test_maximum_delay_is_capped() {
    let backoff = ExponentialBackoff::new(
        Duration::from_millis(100),
        Duration::from_secs(5),  // max
    );
    
    assert_eq!(backoff.delay(10), Duration::from_secs(5));
    assert_eq!(backoff.delay(100), Duration::from_secs(5));
}

AI RESPONSE: I'll add only the maximum delay cap:
[Shows minimal code to pass test]
```

### TDD Prevents AI Problems

**No Scope Creep:**
```markdown
❌ Without TDD:
"Implement retry logic"
AI adds: retry, circuit breaker, metrics, logging, kitchen sink

✅ With TDD:
"Make test_retry_three_times pass"
AI adds: exactly retry three times
```

**Natural Token Management:**
```markdown
Each test is ~10-20 lines
Each implementation is ~20-50 lines
Each checkpoint is ~100 lines total
Token limit = ~10 checkpoints safe
```

---

## TDD Checkpoint Protocol

### After Each Test Cycle
```markdown
## Checkpoint: [Test Name]

RED Phase:
- Wrote test: test_exponential_calculation
- Verified fails: ✓

GREEN Phase:  
- Implemented: ExponentialBackoff.delay()
- Test passes: ✓

REFACTOR Phase:
- Extracted constants: ✓
- Improved naming: ✓

COMMITTED: "test+impl: exponential calculation"

Next Test: test_maximum_delay_cap
```

---

## Common TDD Mistakes

### Mistake: Writing Code First
```markdown
❌ WRONG:
1. Implement feature
2. Add tests after
3. Tests just confirm what code does

✅ RIGHT:
1. Write test defining desired behavior
2. Test fails
3. Implement to make test pass
```

### Mistake: Testing Implementation Not Behavior
```rust
// ❌ WRONG - Testing HOW not WHAT
#[test]
fn uses_multiplication_internally() {
    // Testing private details
}

// ✅ RIGHT - Testing behavior
#[test]
fn doubles_delay_each_attempt() {
    // Testing public contract
}
```

### Mistake: Big Steps
```rust
// ❌ WRONG - Test too big
#[test]
fn entire_retry_system_works() {
    // 50 assertions
    // Multiple behaviors
    // Hard to diagnose failures
}

// ✅ RIGHT - Small focused test
#[test]
fn respects_maximum_attempts() {
    // One behavior
    // Clear failure meaning
}
```

---

## Intra-Phase Certification

### TDD Completion Checklist
```markdown
## Ready to Exit TDD Development

### Test Coverage
- [ ] Every requirement has a test
- [ ] All tests written first (RED)
- [ ] All tests now passing (GREEN)
- [ ] Code refactored where needed

### Commit History
- [ ] Each commit has test + implementation
- [ ] Commit messages reference test
- [ ] No code without test

### Quality Gates  
- [ ] No skipped tests
- [ ] No commented tests
- [ ] No hardcoded values
- [ ] Implementation is minimal

If any unchecked: CONTINUE TDD CYCLES
If all checked: PROCEED TO PHASE 3 (TESTING)
Note: Phase 3 adds edge cases, integration tests
```

---

## AI Assistant TDD Template

```markdown
When user says: "Build retry logic using TDD"

AI Response:
"I'll implement retry logic using Test-Driven Development.

Starting with the first requirement: 'Should retry on network errors'

**RED PHASE - Writing failing test:**

```rust
#[test]
fn should_retry_on_network_error() {
    let strategy = RetryStrategy::default();
    let error = Error::Network("timeout".into());
    assert!(strategy.should_retry(&error));
}
```

Running: `cargo test should_retry_on_network_error`
Result: FAILED (RetryStrategy doesn't exist)

**GREEN PHASE - Minimal implementation:**

```rust
struct RetryStrategy;

impl RetryStrategy {
    fn default() -> Self { Self }
    
    fn should_retry(&self, error: &Error) -> bool {
        matches!(error, Error::Network(_))
    }
}
```

Running: `cargo test should_retry_on_network_error`
Result: PASSED ✓

**REFACTOR PHASE:** No refactoring needed yet.

**COMMIT:** 
```bash
git commit -m "test+impl: retry on network errors"
```

Ready for next test. What's the next requirement?"
```
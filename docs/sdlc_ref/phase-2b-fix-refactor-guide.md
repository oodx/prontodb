# Phase 2b: Fix/Refactor Development Protocol

## Introduction

**Purpose**: Fix broken code or refactor working code WITHOUT changing functionality. Surgical precision required.

**When to Use**:
- Bug fixes (something is broken)
- Performance optimization (too slow)
- Code cleanup (too messy)
- Technical debt reduction (too fragile)

**NOT For**: Adding features, changing behavior, "improving" while fixing

---

## Critical Distinction: Fix vs Refactor

### FIX: Broken → Working
```markdown
- Behavior is WRONG
- Tests are FAILING  
- Users are COMPLAINING
- Production is DOWN
```

### REFACTOR: Working → Better Working
```markdown
- Behavior is CORRECT
- Tests are PASSING
- Code is MESSY
- Maintenance is HARD
```

**AI Critical Rule**: NEVER mix Fix and Refactor. NEVER add features during either.

---

## Roles in Fix/Refactor Phase

### Fix Mode Roles
```
You = Detective → Surgeon → Validator
- First, be Detective: Find root cause
- Then, be Surgeon: Fix ONLY the problem
- Finally, be Validator: Prove it's fixed
```

### Refactor Mode Roles
```
You = Architect → Sculptor → Auditor
- First, be Architect: Identify code smells
- Then, be Sculptor: Reshape without changing function
- Finally, be Auditor: Verify behavior unchanged
```

### AI Assistant Mode
```
AI = Focused Specialist
Human = Change Reviewer
AI must NOT fix/refactor anything not explicitly identified
```

---

## Quick Start Guide

### Fix Protocol
```bash
# Create fix branch
git checkout -b fix/[specific-issue]

# Reproduce bug FIRST
cargo test failing_test  # Must see it fail

# Fix ONLY that issue
# Test now passes
# Commit immediately
git commit -m "fix: [specific issue]"
```

### Refactor Protocol
```bash
# Create refactor branch
git checkout -b refactor/[what-youre-refactoring]

# Run tests FIRST - must be GREEN
cargo test --all  # All passing

# Refactor code
# Run tests again - must STILL be GREEN
cargo test --all  # Still passing

# Commit with no behavior change
git commit -m "refactor: [what you improved]"
```

---

## Deep Dive: Fix Process

### Stage 1: Bug Reproduction (Detective)

**Create Failing Test FIRST:**
```rust
// BEFORE touching any implementation
#[test]
fn test_bug_reproduction() {
    // This test MUST fail
    // It proves the bug exists
    let result = broken_function("trigger");
    assert_eq!(result, "expected");  // FAILS!
}
```

**Document the Bug:**
```markdown
## Bug Report #456
**Symptom**: Retry gives up after 1 attempt
**Expected**: Should retry 3 times
**Actual**: Only tries once
**Reproduction**: Call with network error
**Root Cause**: Off-by-one in attempt counter
```

### Stage 2: Minimal Fix (Surgeon)

**Fix ONLY the Bug:**
```rust
// ❌ WRONG - Fixing multiple things
fn retry(&self, attempt: u32) -> Result<Response> {
    if attempt >= self.max_attempts {  // Fix off-by-one
        self.log_failure();             // ADD logging? NO!
        self.update_metrics();          // ADD metrics? NO!
        self.notify_webhook();          // ADD webhook? NO!
        return Err(Error::MaxAttempts);
    }
    // ...
}

// ✅ RIGHT - Just the fix
fn retry(&self, attempt: u32) -> Result<Response> {
    if attempt >= self.max_attempts {  // ONLY fix off-by-one
        return Err(Error::MaxAttempts);
    }
    // ...
}
```

### Stage 3: Validation (Validator)

**Prove the Fix Works:**
```bash
# Original failing test now passes
cargo test test_bug_reproduction
# ✅ PASSES

# All other tests still pass (no regression)
cargo test --all
# ✅ ALL PASS

# Manual verification if needed
cargo run --example reproduce_bug
# ✅ FIXED
```

**Commit the Fix:**
```bash
git add -A
git commit -m "fix: retry off-by-one error

Bug: Retry attempted only once instead of max_attempts
Cause: Counter started at 1 instead of 0
Solution: Initialize counter at 0
Test: test_bug_reproduction now passes

Fixes #456"
```

---

## Deep Dive: Refactor Process

### Stage 1: Identify Code Smells (Architect)

**Common Refactor Targets:**
```markdown
## Code Smells to Fix
- [ ] Duplicate code (DRY violation)
- [ ] Long functions (>50 lines)
- [ ] Deep nesting (>3 levels)
- [ ] Magic numbers/strings
- [ ] Poor naming
- [ ] Missing abstractions
- [ ] Tight coupling
```

**Measure Before Refactoring:**
```rust
// Document current state
// Complexity: 15 (cyclomatic)
// Lines: 127
// Dependencies: 5 direct
// Test coverage: 78%
```

### Stage 2: Safe Refactoring (Sculptor)

**Refactoring Patterns:**

**Extract Method:**
```rust
// BEFORE - Long function
fn process(&self) -> Result<()> {
    // Validate inputs (20 lines)
    if self.input.is_empty() {
        return Err(Error::Empty);
    }
    // ... more validation
    
    // Transform data (30 lines)
    let mut data = Vec::new();
    for item in &self.input {
        // ... transformation
    }
    
    // Save results (20 lines)
    let file = File::create("out.txt")?;
    // ... saving
    
    Ok(())
}

// AFTER - Extracted methods
fn process(&self) -> Result<()> {
    self.validate_inputs()?;
    let data = self.transform_data()?;
    self.save_results(data)?;
    Ok(())
}

fn validate_inputs(&self) -> Result<()> { /* ... */ }
fn transform_data(&self) -> Result<Vec<Data>> { /* ... */ }
fn save_results(&self, data: Vec<Data>) -> Result<()> { /* ... */ }
```

**Replace Magic Numbers:**
```rust
// BEFORE
if attempt > 3 {  // What's 3?
    thread::sleep(Duration::from_millis(100));  // What's 100?
}

// AFTER  
const MAX_RETRY_ATTEMPTS: u32 = 3;
const RETRY_DELAY_MS: u64 = 100;

if attempt > MAX_RETRY_ATTEMPTS {
    thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
}
```

### Stage 3: Verification (Auditor)

**Behavior Unchanged Proof:**
```bash
# Run exact same tests
cargo test --all
# MUST have same results as before

# Run benchmarks
cargo bench
# Performance should be same or better

# Check behavior with examples
cargo run --example before_refactor > before.txt
cargo run --example after_refactor > after.txt
diff before.txt after.txt  # Should be identical
```

---

## Refactor Safety Rules

### Rule 1: One Refactor at a Time
```rust
// ❌ WRONG - Multiple refactors
// Extract method AND rename AND change structure
// Too many changes to track

// ✅ RIGHT - Single refactor
// JUST extract method
// Commit
// THEN rename in next commit
```

### Rule 2: Tests Stay Green
```markdown
Before refactor: ✅ All tests pass
During refactor: ✅ Keep running tests
After refactor: ✅ All tests still pass
```

### Rule 3: Preserve API
```rust
// ❌ WRONG - Breaking change during refactor
pub fn process(&self, input: &str) -> Result<String>  // Before
pub fn process(&self, input: String) -> Result<Output>  // After - BREAKING!

// ✅ RIGHT - Internal only
pub fn process(&self, input: &str) -> Result<String> {
    // Internal refactoring only
    self.internal_process(input)  // Extracted private method
}
```

---

## Common Fix/Refactor Patterns

### Fix Pattern: Guard Clause
```rust
// Bug: Null pointer in edge case
// FIX - Add guard
fn process(&self, data: Option<Data>) -> Result<()> {
    // Add guard clause
    let data = match data {
        Some(d) => d,
        None => return Ok(()),  // FIX: Handle None case
    };
    
    // Original logic unchanged
    self.do_work(data)
}
```

### Refactor Pattern: Extract Variable
```rust
// BEFORE - Complex expression
if self.retry_count * 2 + 1 < self.config.max_attempts * self.config.retry_multiplier {
    // Hard to understand
}

// AFTER - Named concepts
let attempts_made = self.retry_count * 2 + 1;
let max_allowed = self.config.max_attempts * self.config.retry_multiplier;
if attempts_made < max_allowed {
    // Clear intent
}
```

---

## Intra-Phase Certification

### Fix Completion Checklist
```markdown
## Fix Ready for Testing
- [ ] Bug reproduced in test
- [ ] Minimal fix applied
- [ ] Bug test now passes
- [ ] No other tests broken
- [ ] No unrelated changes
- [ ] Fix documented in commit

If any unchecked: CONTINUE FIXING
If all checked: PROCEED TO PHASE 3 (TESTING)
```

### Refactor Completion Checklist
```markdown
## Refactor Ready for Review
- [ ] Tests green before refactor
- [ ] Tests still green after
- [ ] No behavior changes
- [ ] Code metrics improved
- [ ] No API changes
- [ ] Refactor documented in commit

If any unchecked: CONTINUE REFACTORING  
If all checked: PROCEED TO PHASE 4 (UAT)
```

---

## AI Assistant Instructions

### Fix Mode Template
```markdown
When user says: "Fix the retry bug"

AI Response:
"Starting FIX mode for retry bug. I'll act as bug detective first.

**REPRODUCING BUG:**
```rust
#[test]
fn test_retry_gives_up_early() {
    let retry = Retry::new(3);
    let result = retry.attempt_with_failures(3);
    assert!(result.is_ok());  // FAILS - gives up after 1
}
```

Running test... ❌ FAILS as expected.

**ROOT CAUSE ANALYSIS:**
Found issue in retry.rs line 45:
- Counter starts at 1 instead of 0
- Causes off-by-one error

**MINIMAL FIX:**
```rust
// Only changing the initialization
let mut attempt = 0;  // Was: 1
```

**VERIFICATION:**
Test now passes ✅
All other tests still pass ✅

Committing fix. No other changes made."
```

### Refactor Mode Template
```markdown
When user says: "Refactor the retry logic"

AI Response:
"Starting REFACTOR mode. Current code is working but needs cleanup.

**PRE-REFACTOR VERIFICATION:**
```bash
cargo test retry
```
All tests passing ✅

**IDENTIFIED CODE SMELLS:**
1. Long function (87 lines)
2. Deep nesting (4 levels)
3. Magic numbers (3, 100, 5000)

**REFACTORING PLAN:**
1. Extract retry attempt logic
2. Extract delay calculation  
3. Define constants

**REFACTOR 1 - Extract Method:**
[Shows extraction]

**VERIFICATION:**
Tests still passing ✅
Behavior unchanged ✅

Shall I proceed with next refactor?"
```

### Scope Creep Prevention
```markdown
Human: "While fixing the bug, also add logging"

AI: "I'm currently in FIX mode for the retry bug.

Current scope: Fix off-by-one error ONLY

Adding logging would be:
- A new feature (requires Phase 1 planning)
- Not part of the bug fix
- Should be separate PR

I'll document this request:
```markdown
## Future Enhancement
- Add logging to retry logic
- Created issue #457
```

Continuing with minimal bug fix only."
```

---

## Common Failures

### Failure: Fix Becomes Feature
```markdown
❌ WRONG:
"Fixing retry bug"
*Also adds progress callbacks*
*Also adds custom strategies*
*Also refactors everything*

✅ RIGHT:
Fix PR: Just the bug fix
Feature PR: New capabilities (separately planned)
Refactor PR: Code cleanup (tests stay green)
```

### Failure: Refactor Changes Behavior
```markdown
❌ WRONG:
"Just cleaning up"
*Accidentally changes retry count*
*Accidentally changes timing*
*Tests now fail*

✅ RIGHT:
Run tests before: ✅
Refactor carefully
Run tests after: ✅ (exact same results)
```

### Failure: No Reproduction Test
```markdown
❌ WRONG:
"I think I fixed it"
*No test proves bug existed*
*No test proves it's fixed*
*Bug comes back later*

✅ RIGHT:
1. Write failing test that shows bug
2. Fix bug
3. Test now passes
4. Test prevents regression
```
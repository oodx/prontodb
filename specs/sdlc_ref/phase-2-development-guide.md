# Phase 2: Development Protocol

## Introduction

**Purpose**: Transform specifications into working code. Build ONLY what was planned.

**Duration**: Varies by story points (2-3 points per session typical)

**Output**: Working code with checkpoint commits, ready for testing.

---

## Roles in Development Phase

### Full Team Scenario
- **Developer (Lead)**: Writes implementation code
- **Code Reviewer**: Reviews approach and patterns
- **QA Engineer**: Observes for testability
- **DevOps**: Ensures deployability
- **Security**: Monitors for vulnerabilities
- **Architect**: Guards architectural decisions

### Compressed Scenarios

**Solo Developer Mode**
```
You = Dev + Reviewer + QA Observer
- First, be Dev: Write the code
- Then, be Reviewer: Is this clean?
- Finally, be QA: Can I test this?
```

**AI Assistant + Human Mode**
```
AI = Dev (implements specification)
Human = Reviewer + Architect (guards patterns)
AI must follow spec EXACTLY, no improvements
```

**Rapid Prototyping Mode**
```
Dev = Dev + "Future QA debt"
Skip: Security, DevOps (address later)
Document: Tech debt created
```

---

## Quick Start Guide

### 1. Development Setup (5 minutes)
```bash
# MANDATORY: Create feature branch
git checkout main
git pull origin main
git checkout -b feature/[component-name]

# Verify clean slate
cargo test --all  # Must pass
git status       # Must be clean
```

### 2. Set Context (2 minutes)
```markdown
You are in DEVELOPMENT PHASE.
Role: Developer implementing specification
Input: [Phase 1 specification document]
Constraint: Implement ONLY what's specified
Branch: feature/[component-name]
```

### 3. Development Loop (Per Task)
```markdown
For each task from specification:
1. Write failing test (if TDD)
2. Implement minimal code
3. Make test pass
4. Commit checkpoint
5. Move to next task
```

### 4. Session End Protocol
```bash
# MANDATORY: Commit current state
git add -A
git commit -m "WIP: [describe what's done]"
git push origin feature/[component-name]

# Document progress
echo "Completed: Tasks 1-3 of 5" >> session.md
echo "Next: Task 4 (error handling)" >> session.md
```

---

## Deep Dive: The Development Process

### Stage 1: Pre-Development Verification (Architect Hat)

**Before writing ANY code:**
```markdown
## Pre-Flight Checklist
- [ ] Specification document loaded
- [ ] Task breakdown available
- [ ] Story points per task clear
- [ ] Dependencies available
- [ ] Feature branch created

If any missing: STOP - Return to Planning Phase
```

**Verify Understanding:**
```markdown
## Developer Confirms Understanding

Task 1: Create RetryStrategy trait (1 point)
I understand this as:
- Define trait with should_retry and delay methods
- No implementation, just trait definition
- Include Send + Sync bounds
- Checkpoint after trait compiles

Correct? [WAIT for confirmation]
```

### Stage 2: Test-Driven Development (QA-Minded Dev)

**Write Test First (When Possible):**
```rust
// Step 1: Write the test that will fail
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_retry_strategy_trait_exists() {
        // This won't compile yet - that's OK!
        let strategy: Box<dyn RetryStrategy>;
    }
}

// Step 2: Minimal implementation to compile
pub trait RetryStrategy: Send + Sync {
    fn should_retry(&self, error: &Error) -> bool;
    fn delay(&self, attempt: u32) -> Duration;
}

// Step 3: Checkpoint commit
// git commit -m "checkpoint: RetryStrategy trait defined"
```

### Stage 3: Implementation Pattern (Developer Hat)

**Follow the Crate Structure:**
```rust
// src/lib.rs - Public API only
pub mod retry;
pub use retry::{RetryStrategy, RetryPolicy};

// src/retry/mod.rs - Module organization  
mod strategy;
mod policy;
mod errors;

pub use strategy::RetryStrategy;
pub use policy::RetryPolicy;

// src/retry/strategy.rs - Core trait
use std::time::Duration;

pub trait RetryStrategy: Send + Sync {
    fn should_retry(&self, error: &Error, attempt: u32) -> bool;
    fn delay(&self, attempt: u32) -> Duration;
    fn max_attempts(&self) -> u32 { 3 }  // Default
}

// src/retry/implementations.rs - Concrete types
pub struct ExponentialBackoff {
    base_delay: Duration,
    max_delay: Duration,
    max_attempts: u32,
}

impl RetryStrategy for ExponentialBackoff {
    // ONLY implement required methods
    // NO extra features not in spec
}
```

### Stage 4: Checkpoint Discipline (Dev + Source Control)

**Commit After Each Meaningful Unit:**
```bash
# Task 1: Core Types (1 point)
git add src/retry/strategy.rs
git commit -m "checkpoint: RetryStrategy trait defined"

git add src/retry/errors.rs  
git commit -m "checkpoint: retry error types added"

git add src/retry/mod.rs src/lib.rs
git commit -m "checkpoint: module structure complete"
# Task 1 DONE - Take break if needed

# Task 2: Implementations (2 points)
git add src/retry/exponential.rs
git commit -m "checkpoint: ExponentialBackoff implemented"

git add src/retry/linear.rs
git commit -m "checkpoint: LinearBackoff implemented"  

git add src/retry/fixed.rs
git commit -m "checkpoint: FixedDelay implemented"
# Task 2 DONE - Natural stopping point
```

### Stage 5: Code Review Mindset (Reviewer Hat)

**Self-Review Before Checkpoint:**
```markdown
## Review Checklist (Be Your Own Reviewer)

### Code Quality
- [ ] No more than 5 fields per struct
- [ ] No more than 3 methods per trait  
- [ ] No functions over 50 lines
- [ ] All public items documented

### Patterns
- [ ] Follows project conventions
- [ ] Uses established patterns (Strategy, etc)
- [ ] No reinventing wheels

### Scope
- [ ] Only implements what's specified
- [ ] No "helpful" additions
- [ ] No refactoring unrelated code
```

### Stage 6: Integration Points (Dev + Architect)

**Connect to System (Per Specification):**
```rust
// IF specified: Add plugin registration
#[cfg(feature = "plugin")]
impl Plugin for RetryPlugin {
    fn name(&self) -> &str { "retry" }
    
    fn initialize(&mut self, ctx: &mut Context) -> Result<()> {
        // ONLY registration code
        // NO business logic here
        ctx.register_interceptor(self);
        Ok(())
    }
}

// IF specified: Auto-assembly
#[linkme::distributed_slice(COMPONENTS)]
static RETRY: &RetryComponent = &RetryComponent;
```

---

## Token Exhaustion Protocol

### Recognizing Token Pressure
```markdown
Warning signs:
- Rushing through implementation
- Skipping error handling
- Not writing tests
- Incomplete comments

When you feel pressure:
1. STOP at current function
2. Complete current thought
3. Commit immediately
```

### Safe Stopping Points
```markdown
## Natural Checkpoints (Stop Here)
- ✓ After trait definition
- ✓ After each implementation
- ✓ After integration code
- ✓ After error handling
- ✓ After tests

## Dangerous Points (Don't Stop)
- ✗ Middle of function
- ✗ Partial trait implementation  
- ✗ Uncommitted changes
- ✗ Broken compilation
```

### Checkpoint Recovery
```bash
# Save progress when tokens low
git add -A
git commit -m "CHECKPOINT: Token limit approaching

Completed:
- RetryStrategy trait ✓
- ExponentialBackoff ✓  
- LinearBackoff (partial)

Remaining:
- Finish LinearBackoff.delay() method
- FixedDelay implementation
- Integration code
- Tests

Next session: Continue from LinearBackoff.delay()"
```

---

## Intra-Phase Certification

### Development Phase Exit Criteria
```markdown
## Ready for Testing Checklist

### Completeness
- [ ] All tasks from spec completed
- [ ] All checkpoints committed
- [ ] Feature branch pushed

### Code Quality
- [ ] Code compiles without warnings
- [ ] Basic error handling present
- [ ] Public APIs documented

### Scope Adherence  
- [ ] ONLY specified features built
- [ ] No extra "improvements"
- [ ] No unrelated refactoring

### Testability
- [ ] Code structured for testing
- [ ] No untestable mega-functions
- [ ] Dependencies injected

If any unchecked: FIX IN DEVELOPMENT
If all checked: PROCEED TO TESTING
```

---

## Common Development Failures

### Failure: Scope Creep
```rust
// ❌ WRONG: Adding unplanned features
impl RetryStrategy for ExponentialBackoff {
    fn should_retry(&self, error: &Error) -> bool { /* ... */ }
    fn delay(&self, attempt: u32) -> Duration { /* ... */ }
    
    // This wasn't in the spec!
    fn circuit_breaker(&self) -> bool { /* ... */ }
    fn retry_budget(&self) -> Budget { /* ... */ }
}

// ✅ RIGHT: Only specified methods
impl RetryStrategy for ExponentialBackoff {
    fn should_retry(&self, error: &Error) -> bool { /* ... */ }
    fn delay(&self, attempt: u32) -> Duration { /* ... */ }
}
```

### Failure: Mega Commits
```bash
# ❌ WRONG: Everything in one commit
git commit -m "implemented retry logic"
# 47 files changed, 2000 lines added

# ✅ RIGHT: Incremental checkpoints
git commit -m "checkpoint: RetryStrategy trait"
# 2 files changed, 20 lines added
git commit -m "checkpoint: ExponentialBackoff impl"  
# 1 file changed, 45 lines added
```

### Failure: Refactoring Rampage
```rust
// ❌ WRONG: "While I'm here, let me improve Transport"
// transport.rs
impl Transport {
    // Completely rewritten for "better" patterns
    // Original functionality broken
    // 500 lines changed
}

// ✅ RIGHT: Leave working code alone
// Only touch retry-related files
```

### Failure: Test Procrastination
```rust
// ❌ WRONG: "I'll write tests after everything works"
impl ComplexRetryLogic {
    // 500 lines of untested code
    // Multiple execution paths
    // Edge cases everywhere
}

// ✅ RIGHT: Test as you go
#[test]
fn test_exponential_calculation() { /* ... */ }

impl ExponentialBackoff {
    fn calculate_delay(&self, attempt: u32) -> Duration {
        // 10 lines, already tested
    }
}
```

---

## AI Assistant Instructions

### Development Session Template
```markdown
When user says: "Implement the retry logic we planned"

Your response:

"I'll implement the retry logic from our specification. Starting DEVELOPMENT PHASE.

First, let me set up:
1. Creating feature branch: feature/retry-logic
2. Reviewing specification: [summarize key points]

I'll implement the 5 tasks identified:
- Task 1: Core types (1 point)
- Task 2: Three strategies (2 points)  
- Task 3: Integration (2 points)
- Task 4: Error handling (1 point)
- Task 5: Tests (2 points)

Starting with Task 1: Core Types...

[Show code]

Checkpoint: Task 1 complete. Shall I continue with Task 2?"
```

### Handling Scope Creep
```markdown
Human: "Also add circuit breaker while you're at it"

AI: "I notice circuit breaker wasn't in our Phase 1 specification. 
To maintain scope discipline, I should:

1. Continue with only the planned retry logic
2. Document circuit breaker for a future phase

I'll add this to our backlog:
## Future Enhancement: Circuit Breaker
- Estimated: 3 story points
- Dependencies: Retry logic (current work)

Continuing with specified retry implementation only. 
Currently on Task 2 of 5..."
```

### Token Limit Approach
```markdown
[When approaching token limit]

"I'm approaching token limits. Let me checkpoint current progress:

COMPLETED:
✓ Task 1: Core types
✓ Task 2: Strategies (2 of 3 done)

CURRENT STATE:
- ExponentialBackoff: Complete
- LinearBackoff: Complete
- FixedDelay: Not started

Committing checkpoint:
`git commit -m 'checkpoint: 2 of 3 strategies complete'`

NEXT SESSION should:
1. Implement FixedDelay strategy
2. Continue with Task 3 (Integration)

Ending session here to preserve code quality."
```
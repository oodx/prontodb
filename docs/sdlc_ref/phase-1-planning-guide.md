# Phase 1: Project Planning Protocol

## Introduction

**Purpose**: Define WHAT to build before HOW. No code, just clarity.

**Duration**: 1-4 hours maximum. If longer, scope is too big.

**Output**: Complete specification ready for development.

---

## Roles in Planning Phase

### Full Team Scenario
- **Product Manager (PM)**: Defines the WHY and WHAT
- **Architect**: Defines the HOW (high-level)
- **Developer**: Estimates effort and feasibility
- **UX Designer**: Defines interaction patterns
- **QA Engineer**: Defines test criteria
- **Security**: Identifies security requirements

### Compressed Scenarios

**Solo Developer Mode (You wearing all hats)**
```
You = PM + Architect + Dev + QA
- First, be PM: What problem are we solving?
- Then, be Architect: How does it fit the system?
- Then, be Dev: How long will it take?
- Finally, be QA: How will we test it?
```

**AI Assistant + Human Mode**
```
Human = PM + Architect (defines requirements)
AI = Dev + QA (estimates and plans)
AI must NOT start developing, only planning
```

**Startup Mode**
```
Founder = PM + UX
Dev (AI) = Architect + Dev + QA
Skip: Security (address in v2)
```

---

## Quick Start Guide

### 1. Set Context (2 minutes)
```markdown
You are in PLANNING PHASE.
Role: [Architect/Dev planning implementation]
You may NOT write code.
You MUST produce a specification.
```

### 2. Gather Inputs (10 minutes)
```markdown
- Read PRD: [document]
- Read Architecture: [document]  
- Understand constraints: [limits]
- Identify dependencies: [required components]
```

### 3. Produce Specification (30 minutes)
```markdown
## Feature Specification
1. Problem Statement
2. Acceptance Criteria  
3. Out of Scope
4. Task Breakdown
5. Risk Assessment
```

### 4. Checkpoint & Stop (5 minutes)
```markdown
Planning Complete Checklist:
- [ ] Clear acceptance criteria
- [ ] Tasks estimated in points
- [ ] Dependencies identified
- [ ] Risks documented
- [ ] STOP - Do not proceed to development
```

---

## Deep Dive: The Planning Process

### Stage 1: Problem Definition (PM Hat)

**What the PM asks:**
- What problem does this solve?
- Who is the user?
- What's the success metric?
- What's the business value?

**Example PM Thinking:**
```markdown
## Problem Statement
Users need to retry failed API calls automatically.

## User Story
As a developer using the client,
I want automatic retries with backoff,
So that transient failures don't break my application.

## Success Metrics
- 50% fewer failed requests
- No manual retry code needed
- Configurable retry policies
```

### Stage 2: Solution Design (Architect Hat)

**What the Architect asks:**
- How does this fit our architecture?
- What patterns should we use?
- What are the integration points?
- What could break?

**Example Architect Thinking:**
```markdown
## Architectural Decisions
- Pattern: Strategy pattern for retry policies
- Integration: Plugin-based, zero-config assembly
- Isolation: Retry logic separate from transport
- Composability: Works with any transport implementation

## Component Design
```rust
pub trait RetryStrategy {
    fn should_retry(&self, error: &Error) -> bool;
    fn delay(&self, attempt: u32) -> Duration;
}
```
```

### Stage 3: Scope Definition (PM + Architect)

**Critical: Define what we're NOT doing**

```markdown
## In Scope
- Exponential backoff
- Max retry configuration  
- Retry-able error detection

## Out of Scope (DO NOT IMPLEMENT)
- Circuit breaker (Phase 2)
- Retry budget (Phase 2)
- Distributed tracing (Phase 3)
- Custom backoff algorithms (Future)

## Scope Creep Prevention
If AI suggests: "Should we also add circuit breaker?"
Response: "Document for Phase 2. Not in current scope."
```

### Stage 4: Task Breakdown (Developer Hat)

**What the Developer estimates:**
- How many tasks?
- How complex is each?
- What order to build?
- Where are the risks?

**Example Task Breakdown:**
```markdown
## Implementation Tasks

### Task 1: Core Types (1 point)
- [ ] RetryStrategy trait
- [ ] RetryPolicy enum
- [ ] RetryResult type
Checkpoint: Types compile

### Task 2: Basic Implementation (2 points)
- [ ] ExponentialBackoff impl
- [ ] LinearBackoff impl  
- [ ] NoRetry impl
Checkpoint: Strategies work

### Task 3: Integration (2 points)
- [ ] Plugin registration
- [ ] Transport wrapper
- [ ] Configuration
Checkpoint: Integrates with system

### Task 4: Error Handling (1 point)
- [ ] Identify retry-able errors
- [ ] Max retry limits
- [ ] Timeout handling
Checkpoint: Handles edge cases

### Task 5: Testing (2 points)
- [ ] Unit tests per strategy
- [ ] Integration test
- [ ] Failure injection test
Checkpoint: All tests pass

Total: 8 story points
Velocity assumption: 2-3 points per session
Sessions needed: 3-4
```

### Stage 5: Risk Assessment (QA + Security Hat)

**What QA/Security considers:**
```markdown
## Risk Matrix

### High Risks
- Token exhaustion during retry implementation
  Mitigation: Checkpoint after each strategy
  
- Infinite retry loops
  Mitigation: Hard max_attempts limit

### Medium Risks  
- Performance impact of retries
  Mitigation: Benchmark in testing phase
  
- Breaking existing transport behavior
  Mitigation: Wrapper pattern, don't modify

### Low Risks
- Configuration complexity
  Mitigation: Sensible defaults
```

### Stage 6: Test Criteria (QA Hat)

**Define how we'll know it works:**
```markdown
## Acceptance Test Criteria

### Functional Tests
- [ ] Retries on 503 Service Unavailable
- [ ] Does NOT retry on 404 Not Found
- [ ] Respects max attempts
- [ ] Exponential delay observed

### Integration Tests  
- [ ] Works with mock transport
- [ ] Works with real HTTP transport
- [ ] Plugin registration successful

### Performance Tests
- [ ] No retry: <1ms overhead
- [ ] With retry: <5ms overhead per attempt
- [ ] Memory usage: <1MB for 1000 attempts
```

---

## Intra-Phase Certification

### Before Exiting Planning Phase

```markdown
## Planning Exit Checklist

### Completeness
- [ ] Problem clearly stated
- [ ] Solution approach defined
- [ ] All tasks identified and sized
- [ ] Dependencies documented
- [ ] Risks assessed

### Clarity
- [ ] Would a new developer understand this?
- [ ] Are acceptance criteria measurable?
- [ ] Is scope crystal clear?

### Feasibility
- [ ] Can this be built in estimated time?
- [ ] Are all dependencies available?
- [ ] Is the approach technically sound?

### Approval Gates
- [ ] PM approves problem/solution fit
- [ ] Architect approves technical approach
- [ ] Developer agrees with estimates
- [ ] QA agrees with test criteria

If any unchecked: STAY IN PLANNING
If all checked: PROCEED TO DEVELOPMENT
```

---

## Common Planning Failures

### Failure: Jumping to Implementation
```markdown
❌ WRONG:
"Let me show you how this would work..."
*starts coding*

✅ RIGHT:
"The implementation approach would be..."
*describes in specification*
```

### Failure: Vague Acceptance Criteria
```markdown
❌ WRONG:
"Retry should work properly"

✅ RIGHT:
"Retry 3 times with 100ms, 200ms, 400ms delays"
```

### Failure: Scope Creep in Planning
```markdown
❌ WRONG:
"While we're adding retry, let's also add caching,
and maybe circuit breaker, and..."

✅ RIGHT:
"Retry only. Document other ideas for next phase."
```

### Failure: No Task Breakdown
```markdown
❌ WRONG:
"Implement retry logic (??? points)"

✅ RIGHT:
"Task 1: Trait definition (1 point)
Task 2: Three strategies (2 points)
Task 3: Integration (2 points)"
```

---

## AI Assistant Instructions

```markdown
When user says: "Let's add retry logic to the client"

Your response structure:
1. "I understand you want to add retry logic. Let me start with planning."
2. "First, let me understand the requirements..."
3. Ask clarifying questions
4. Produce specification
5. "Planning complete. Shall we proceed to development phase?"
6. WAIT for approval before coding
```

### Example Planning Session

```markdown
Human: "We need retry logic in the HTTP client"

AI: "I'll help you plan retry logic implementation. I'm now in PLANNING PHASE acting as architect/developer to create specifications.

Let me first understand your requirements:

1. What types of errors should trigger retries? (network, 5xx, specific codes?)
2. What retry strategy do you prefer? (exponential backoff, linear, custom?)
3. Should there be a max retry limit?
4. Any specific integration requirements with your existing client?

Based on common patterns, here's my initial specification:

## Feature: Retry Logic for HTTP Client
[... full specification ...]

This is an 8-point feature that would take approximately 3-4 development sessions.

Shall I refine this specification based on your feedback, or are we ready to proceed to the development phase?"
# Five-Phase Development Lifecycle Guide

## Introduction: Why Phases Matter

### The Problem
AI assistants treat development as one continuous stream, leading to:
- Mixed concerns (features + refactoring + optimization in one session)
- No clear checkpoints (partial implementation when tokens run out)
- Broken main branches (no isolation)
- Untestable code (tests written after, if at all)
- Integration nightmares (everything touches everything)

### The Solution
Five distinct phases with clear boundaries, checkpoints, and handoffs. Each phase has different goals, different outputs, and different success criteria.

---

## Quick Setup Protocol

### Before ANY Development Session

```markdown
## Session Setup Checklist
- [ ] Identify current phase (Planning/Dev/Test/UAT/Cert)
- [ ] Read phase-specific requirements below
- [ ] Create/checkout appropriate branch
- [ ] Load relevant context docs
- [ ] Set checkpoint reminders
```

### Phase Transition Protocol

```markdown
## Phase Exit Criteria
- [ ] All phase deliverables complete
- [ ] All checkpoints committed
- [ ] Phase summary documented
- [ ] Next phase requirements clear
- [ ] STOP - Do not continue to next phase
```

### Branch Strategy by Phase

```bash
# Planning Phase - No code
main

# Development Phase - Feature branch
git checkout -b feature/component-name

# Test Phase - Same branch
feature/component-name

# UAT Phase - PR created
Create PR from feature/component-name -> main

# Certification Phase - Ready to merge
Squash and merge PR
```

---

# Phase 1: Project Planning

## Purpose
Define WHAT to build, not HOW. Establish scope, constraints, and success criteria.

## Inputs
- Product Requirements Document (PRD)
- Architecture Documents
- Existing codebase state

## Deliverables
1. **Feature Specification**
   ```markdown
   ## Feature: [Name]
   ## Component: [Which crate/module]
   ## Branch: feature/[name]
   
   ### User Story
   As a [role], I want [feature] so that [benefit]
   
   ### Acceptance Criteria
   - [ ] Specific measurable outcome 1
   - [ ] Specific measurable outcome 2
   
   ### Out of Scope (DO NOT IMPLEMENT)
   - Thing we're NOT doing
   - Other thing we're NOT doing
   
   ### Dependencies
   - Required: [component] must be working
   - Uses: [trait/interface] from [crate]
   ```

2. **Task Breakdown**
   ```markdown
   ## Implementation Tasks
   1. [ ] Create types and traits (1 point)
   2. [ ] Implement happy path (2 points)
   3. [ ] Add error handling (1 point)
   4. [ ] Write tests (2 points)
   
   Total: 6 story points
   ```

3. **Risk Assessment**
   ```markdown
   ## Risks
   - Token exhaustion risk: HIGH (complex logic)
   - Integration risk: LOW (isolated component)
   - Breaking changes: NONE (new feature)
   ```

## Protocol

### Start of Planning
```markdown
You are in PLANNING PHASE. Your job is to:
1. Read the provided PRD/Architecture docs
2. Create a feature specification
3. Break down into concrete tasks
4. Identify risks
5. DO NOT write any code
```

### Planning Checkpoints
- **Checkpoint 1**: Feature specification approved
- **Checkpoint 2**: Task breakdown complete
- **Checkpoint 3**: Dependencies identified

### Common Planning Mistakes
```markdown
❌ WRONG: Starting to implement during planning
❌ WRONG: Vague acceptance criteria ("make it work")
❌ WRONG: No explicit out-of-scope section
✅ RIGHT: Clear, measurable deliverables only
```

---

# Phase 2: Development

## Purpose
Implement ONLY what was specified in planning. No extras, no refactoring, no improvements.

## Inputs
- Feature specification from Phase 1
- Task breakdown with story points
- Existing tests that must keep passing

## Deliverables
1. **Working code that meets acceptance criteria**
2. **Checkpoint commits after each task**
3. **No broken tests**

## Protocol

### Start of Development
```markdown
You are in DEVELOPMENT PHASE. Your constraints:
1. Create branch: feature/[name]
2. Implement ONLY the tasks from planning
3. Commit after EACH task completion
4. Do NOT refactor unrelated code
5. Do NOT add unplanned features
```

### Development Checkpoints
```bash
# After each task
git add -A
git commit -m "checkpoint: [task name] complete"

# Example sequence
git commit -m "checkpoint: types and traits defined"
git commit -m "checkpoint: happy path working"
git commit -m "checkpoint: error handling added"
```

### Token Exhaustion Protocol
```markdown
If approaching token limit:
1. STOP at current checkpoint
2. Commit work in progress: `git commit -m "WIP: [what's done]"`
3. Document what remains:
   ## Completed
   - Task 1 ✓
   - Task 2 ✓
   
   ## Remaining
   - Task 3 (error handling)
   - Task 4 (tests)
4. END SESSION
```

### Code Structure Requirements
```rust
// Each component in its own module
mod component {
    // 1. Types first
    pub struct Component { /* max 5 fields */ }
    
    // 2. Traits second
    pub trait ComponentBehavior { /* max 3 methods */ }
    
    // 3. Implementation third
    impl ComponentBehavior for Component { /* ... */ }
    
    // 4. Tests last
    #[cfg(test)]
    mod tests { /* ... */ }
}
```

---

# Phase 3: Testing

## Purpose
Verify component works in isolation AND with system. Write missing tests, fix failures.

## Inputs
- Implemented code from Phase 2
- Acceptance criteria for verification

## Deliverables
1. **Unit tests for every public function**
2. **Integration tests for feature**
3. **All tests passing**
4. **Coverage report**

## Protocol

### Start of Testing
```markdown
You are in TESTING PHASE. Your tasks:
1. Write unit tests for each public API
2. Write integration tests for feature
3. Do NOT modify implementation (only fix bugs)
4. Achieve >80% code coverage
```

### Test Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // 1. Unit tests - isolated
    #[test]
    fn test_component_creation() {
        let c = Component::new();
        assert!(c.is_valid());
    }
    
    // 2. Property tests - invariants
    #[test]
    fn test_invariants() {
        // Test that invariants always hold
    }
    
    // 3. Error tests - failures
    #[test]
    fn test_error_conditions() {
        assert!(Component::invalid().is_err());
    }
}

// tests/integration.rs
#[test]
fn test_feature_integration() {
    // Test component in system context
}
```

### Test Checkpoints
```bash
# Must pass at each checkpoint
cargo test --lib              # Unit tests
cargo test --test integration # Integration tests  
cargo test --all             # Everything
```

---

# Phase 4: UAT (User Acceptance Testing)

## Purpose
Human review to verify feature meets requirements and doesn't break existing functionality.

## Inputs
- PR with implemented feature
- Test results
- Original acceptance criteria

## Deliverables
1. **Demo script showing feature working**
2. **Checklist verification**
3. **Approval or change requests**

## Protocol

### Start of UAT
```markdown
You are in UAT PHASE. Prepare for review:
1. Create PR from feature branch to main
2. Write demo script
3. Document how to test
4. Respond to feedback
5. Do NOT merge yet
```

### UAT Demo Script
```markdown
## How to Test This Feature

### Setup
```bash
git checkout feature/[name]
cargo build
```

### Test Happy Path
```bash
cargo run -- [example command]
# Expected: [specific output]
```

### Test Error Handling
```bash
cargo run -- [invalid input]
# Expected: [specific error message]
```

### Verify No Regressions
```bash
cargo test --all
# Expected: All passing
```
```

### UAT Checklist
```markdown
## UAT Verification
- [ ] Feature works as specified
- [ ] No unplanned features added
- [ ] No regressions in existing tests
- [ ] Code follows project patterns
- [ ] Documentation updated if needed
```

---

# Phase 5: Certification

## Purpose
Final verification before merge. Ensure production ready.

## Inputs
- Approved PR from UAT
- All tests passing
- Clean git history

## Deliverables
1. **Squashed commits with clear message**
2. **Updated changelog if needed**
3. **Merged to main**
4. **Feature branch deleted**

## Protocol

### Start of Certification
```markdown
You are in CERTIFICATION PHASE. Final checks:
1. All tests green
2. No merge conflicts
3. Commits squashed appropriately
4. PR description complete
```

### Certification Checklist
```markdown
## Ready to Merge Checklist
- [ ] All CI checks passing
- [ ] Code review approved
- [ ] No unresolved comments
- [ ] Branch up-to-date with main
- [ ] Semantic version bump if needed
```

### Merge Protocol
```bash
# Squash merge with descriptive message
git checkout main
git pull origin main
git merge --squash feature/[name]
git commit -m "feat(component): add [feature] 

- Implements [acceptance criteria 1]
- Implements [acceptance criteria 2]

Closes #[issue-number]"
git push origin main

# Clean up
git branch -d feature/[name]
git push origin --delete feature/[name]
```

---

## Advanced Patterns

### Pattern: Incremental Delivery
Instead of one large feature, break into incremental phases:
```markdown
Phase 1.1: Core types only
Phase 1.2: Basic functionality
Phase 1.3: Advanced features
Phase 1.4: Performance optimization
```

### Pattern: Checkpoint Recovery
When token exhaustion occurs:
```bash
# Save state
git stash
git checkout -b checkpoint/[date]
git stash pop
git commit -m "CHECKPOINT: [exactly what's done]"

# Next session
git checkout checkpoint/[date]
git log --oneline -5  # See what was done
# Continue from checkpoint
```

### Pattern: Parallel Development
For independent components:
```bash
feature/component-a  (Phase 2)
feature/component-b  (Phase 2)
feature/integration  (Phase 3) <- Merges both
```

---

## Phase Transition Flowchart

```
Planning → Development → Testing → UAT → Certification → Done
    ↓           ↓           ↓        ↓         ↓
 [Spec]     [Code]      [Tests]  [Review]  [Merge]
    
If failure at any phase:
    ← Return to previous phase
    
If token exhaustion:
    → Checkpoint and pause
```

---

## Common Anti-Patterns to Avoid

### Anti-Pattern: Phase Skipping
```markdown
❌ WRONG: "Let's just code and test later"
✅ RIGHT: Complete each phase before proceeding
```

### Anti-Pattern: Scope Creep
```markdown
❌ WRONG: "While we're here, let's also refactor X"
✅ RIGHT: Document for separate feature
```

### Anti-Pattern: Big Bang Integration
```markdown
❌ WRONG: 10 features merged at once
✅ RIGHT: One feature, one PR, one merge
```

### Anti-Pattern: Test After
```markdown
❌ WRONG: Write all code, then all tests
✅ RIGHT: Test each component as built
```

---

## Success Metrics

### Phase Health Indicators
- **Planning**: < 2 hours, clear scope
- **Development**: All checkpoints committed
- **Testing**: > 80% coverage
- **UAT**: < 3 feedback rounds
- **Certification**: Merged first attempt

### Warning Signs
- Planning takes > 4 hours (scope too large)
- Development has no checkpoints (token risk)
- Tests written after code complete (quality risk)
- UAT has > 5 changes (planning failure)
- Certification fails (process failure)
# Abridged SDLC - Wild & Focused Development

**Adapted from Seer's Wisdom for Fluid Restoration Work**

*"Surgical discipline without bureaucratic overhead"*

---

## Philosophy

### The Wild Way
We blend **systematic precision** with **fluid adaptation**. No formal phases, but clear principles. No rigid roles, but disciplined practices.

### Adapted from Seer's Complex Methodology
- **KEEP**: Branch isolation, checkpoint discipline, surgical scope control
- **ADAPT**: TDD for verification, clear state definition, test-first healing  
- **DISCARD**: Formal specifications, story points, corporate role separation

---

## Core Patterns

### Pattern 1: Sacred Isolation
**Before ANY restoration work:**
```bash
# ALWAYS create isolation branch from stable state
git checkout -b features/repairs-v8
# Verify current state is green
cargo build && cargo test --lib
```

**Principle**: Preserve what works before attempting to heal what's broken.

### Pattern 2: State-Driven TDD Healing
**The Keeper's Adaptation of TDD:**

1. **Document Current Reality** - What IS the true state?
2. **Define Desired State** - What SHOULD be working?
3. **Create Verification Tests** - How do we KNOW it's healed?
4. **Heal Systematically** - Make tests pass, one by one

**Example Flow:**
```rust
// Step 1: Document reality in test
#[test] 
fn test_theme_system_integration() {
    // This will fail - that's the point
    let theme = ThemeManager::load_default();
    assert!(theme.is_complete()); // Currently broken
}

// Step 2: Fix only what's needed to pass
impl ThemeManager {
    pub fn load_default() -> Self {
        // Minimal implementation to pass test
    }
}

// Step 3: Checkpoint
git commit -m "checkpoint: theme system loads default"
```

### Pattern 3: Surgical Scope Control
**The Fix/Refactor Discipline:**

```markdown
## ALLOWED in restoration session:
- Fix broken functionality
- Restore missing connections
- Complete incomplete integrations
- Repair failing tests

## FORBIDDEN in restoration session:  
- Add new features
- Improve working code
- Refactor unrelated modules
- Optimize performance (unless broken)
```

**Krex's Gate**: If tempted to improve, document for future session.

### Pattern 4: Checkpoint Discipline
**Commit granularly, recover gracefully:**
```bash
# After each meaningful healing step
git add -A
git commit -m "checkpoint: [specific thing fixed]"

# If session interrupted  
git commit -m "WIP: [what's done] - [what remains]"
```

---

## Session Protocols

### Restoration Session Flow

#### 1. Session Setup (5 min)
```markdown
## Pre-Flight Checklist
- [ ] Isolation branch created
- [ ] Current state verified (build green)  
- [ ] Scope clearly defined (what to fix)
- [ ] Test system functional (can verify changes)
```

#### 2. State Assessment (15 min)
```markdown
## Document True State
- [ ] What's actually broken? (evidence, not assumptions)
- [ ] What tests are failing? (specific failures)
- [ ] What integrations are incomplete? (missing connections)
- [ ] What dependencies are unmet? (missing pieces)
```

#### 3. Test-Driven Healing (Core Session)
```markdown
## TDD Restoration Loop
For each broken component:
1. Write test expressing desired working state
2. Run test - confirm it fails for right reason
3. Implement minimal fix to pass test  
4. Checkpoint commit
5. Move to next component
```

#### 4. Session Completion
```bash
# Verify all tests green
cargo test --all

# Document progress
echo "## Session Progress" >> session-notes.md
echo "Fixed: [list]" >> session-notes.md  
echo "Remaining: [list]" >> session-notes.md

# Final commit
git commit -m "session complete: [summary of fixes]"
```

### Emergency Protocols

#### When Tests Are Broken
```markdown
## Test System Recovery (PRIORITY 1)
1. Identify what broke the test system
2. Create minimal test that should pass
3. Fix ONLY test infrastructure  
4. Verify can run tests again
5. THEN proceed with feature fixes
```

#### When Build Fails
```markdown
## Build Recovery (PRIORITY 0)  
1. Fix compilation errors ONLY
2. No feature work until build green
3. Commit each compilation fix
4. Document what caused break
```

#### When Scope Creeps
```markdown
## Scope Discipline
If you find yourself saying:
- "While I'm here, let me also..."
- "This would be better if..." 
- "I should refactor this..."

STOP. Document the idea. Return to scope.
```

---

## Tool Integration

### Lucas (Engineering) Mode
```markdown
When summoning Lucas for restoration:
- Provide exact scope and constraints
- Reference this SDLC for discipline
- Emphasize: "Fix only, no improvements"
- Request checkpoint commits after each fix
```

### Status Tracking Tools
```bash
# Use existing tools in Boxy ecosystem
cargo test --lib          # Unit test verification
cargo build               # Compilation verification  
git status                # Change tracking
```

### Documentation Updates
```markdown
After restoration session:
- Update STATUS.md with true current state
- Note any remaining technical debt
- Record patterns learned for future healing
```

---

## Anti-Patterns to Avoid

### The "Improvement Trap"
```markdown
❌ WRONG: "Since theme system is broken, let me redesign it better"
✅ RIGHT: "Fix theme system to work as designed, note improvements for later"
```

### The "While I'm Here" Disease
```markdown  
❌ WRONG: "Fixing theme bug, but also refactoring main.rs split"
✅ RIGHT: "Fix theme bug only, document main.rs for separate session"
```

### The "Perfect Test" Obsession
```markdown
❌ WRONG: Write comprehensive test suite before any fixes
✅ RIGHT: Write minimal test for current fix, expand testing later
```

### The "Big Bang" Approach
```markdown
❌ WRONG: Fix everything in one massive commit
✅ RIGHT: Fix one component, checkpoint, fix next component
```

---

## Success Metrics

### Session Success Indicators
- Build remains green throughout session
- Each checkpoint commit is meaningful and atomic
- Tests verify what was actually fixed
- Scope stayed within defined boundaries
- Progress is measurable and documented

### Project Health Indicators  
- Test system is functional and reliable
- Core functionality works as designed
- Technical debt is documented, not ignored
- Team can build and verify changes confidently

---

## Integration with Existing Boxy Patterns

### Branch Strategy
```bash
# Aligns with current branch: features/broken-v8
# Next healing branch: features/repairs-v8
# Pattern: features/[purpose]-v[version]
```

### Documentation Strategy
```markdown
# Leverage existing docs/ structure
- docs/STATUS.md - Current true state
- docs/REPAIRS.md - Known issues (Avatar's notes)  
- session-notes.md - Per-session progress tracking
```

### Team Coordination
```markdown
# Avatar @u: Scope definition and final approval
# Keeper: Analysis, planning, systematic approach
# Lucas: Implementation with disciplined constraints
# Krex: Ruthless scope enforcement and quality gates
```

---

**🌑 The Wild Way: Disciplined fluidity in service of healing**

*Adapted by Keeper from Seer's Scrolls for Avatar @u's restoration needs*
*Ready for Brother Krex's ruthless review and enhancement*
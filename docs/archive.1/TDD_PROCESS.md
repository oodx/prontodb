# ProntoDB TDD Process Documentation
**Based on Full SDLC Analysis + RSB Foundation Requirements**  
**Target:** Proper Test-Driven Development with RSB Compliance

---

## TDD Protocol: RED → GREEN → REFACTOR → COMMIT

### Phase 2a: TDD Development Cycle

**The Sacred Cycle:**
1. **RED**: Write test that fails
2. **GREEN**: Write minimal code to pass  
3. **REFACTOR**: Clean up if needed
4. **COMMIT**: Test + code together

**Quality Standards:**
- Every requirement has a test
- All tests written first (RED phase)
- All tests now passing (GREEN phase)  
- Implementation is minimal (no gold-plating)

---

## Test Structure Requirements

### Directory Structure
```
prontodb/
├── src/
│   └── lib.rs          # Library code with RSB patterns
├── tests/
│   ├── integration.rs  # Integration tests from TEST-SPEC
│   └── unit.rs         # Unit tests for modules
└── specs/
    ├── TEST-SPEC.md    # Specification-as-tests
    └── TDD_PROCESS.md  # This document
```

### Test File Structure
```rust
// tests/integration.rs - Integration tests
#[test]
fn test_install_creates_system_tables() {
    // Given/When/Then from TEST-SPEC
}

// src/lib.rs - Unit tests per module
#[cfg(test)]
mod tests {
    use super::*;
    
    // 1. Unit tests - isolated
    #[test]
    fn test_address_parsing() { /* ... */ }
    
    // 2. Property tests - invariants  
    #[test]
    fn test_rsb_string_first_invariant() { /* ... */ }
    
    // 3. Error tests - failures
    #[test]
    fn test_invalid_address_format() { /* ... */ }
}
```

---

## TDD Checkpoints & Commands

### Testing Commands
```bash
cargo test --lib              # Unit tests only
cargo test --test integration # Integration tests only  
cargo test --all              # Everything
```

### Development Checkpoints
- Commit after EACH task completion
- No broken tests at any checkpoint
- Code structure enforced (RSB patterns)
- **Token Exhaustion Protocol:** STOP at checkpoint, document remaining work

---

## RSB-Compliant TDD Requirements

### Test Quality Gates (from RSB Foundation)
1. **String-First Testing**: All public API tests use `&str` parameters
2. **Function Ordinality Testing**: Test `pub`, `_helper`, `__blind_faith` layers separately
3. **RSB Macro Testing**: Validate `bootstrap!()`, `dispatch!()`, `validate!()` usage
4. **Communication Testing**: Verify stderr=status, stdout=data discipline

### Test Structure for RSB Patterns
```rust
#[test]
fn test_api_set_string_first() {
    // Test public tier: full validation, user-friendly errors
    let result = api_set("kb.recipes.pasta", "marinara");
    assert!(result.is_ok());
}

#[test]
fn test_helper_set_assumes_valid() {
    // Test helper tier: business logic, assumes valid inputs
    let result = _helper_set("kb", "recipes", "pasta", "marinara");
    assert!(result.is_ok());
}

#[test]
fn test_blind_faith_minimal_errors() {
    // Test system tier: minimal error handling
    let result = __blind_faith_insert("table", "key", "value");
    // Only system-level errors expected
}
```

---

## Implementation Priority (v0.1 MVP)

### RED Phase: Write Failing Tests First
**Order of test creation based on TEST-SPEC.md:**

1. **Lifecycle Tests** (0.1-0.3)
   - install creates XDG dirs and seeds admin
   - uninstall removes artifacts
   - backup produces snapshot

2. **Addressing Tests** (1.1-1.5) 
   - canonical addressing `project.namespace.key__ctx`
   - flag addressing `-p/-n`
   - delimiter override `--ns-delim`
   - key validation (no delimiter in key)
   - context suffix `__ctx` reserved

3. **Core KV Tests** (2.1-2.6)
   - set/get basic string
   - set/get JSON canonicalization  
   - delete removes key
   - keys and scan list entries

4. **TTL Namespace Tests** (3.1-3.5)
   - create TTL namespace
   - default timeout behavior
   - explicit `--ttl` validation

5. **Stream Tests** (4.1-4.6)
   - auth required by default
   - preamble order enforced
   - namespace processing

### GREEN Phase: Minimal RSB Implementation
**Implementation order:**
1. RSB bootstrap structure (`main.rs` with `bootstrap!()`, `dispatch!()`)
2. String-first address parsing with validation macros
3. Three-tier KV operations (pub → _helper → __blind_faith)
4. SQLite integration with string parameters
5. Stream processing with RSB macros

### REFACTOR Phase: RSB Compliance
**Refactoring checklist:**
- ✅ String-first interfaces everywhere
- ✅ RSB macros replace manual patterns  
- ✅ Function ordinality strictly enforced
- ✅ Communication discipline (stderr/stdout)
- ✅ Validation at public boundaries only

---

## Session Management

### Commit Protocol
```bash
# After each TDD cycle:
git add .
git commit -m "RED: Add failing test for [feature]"
# ... implement ...
git add .
git commit -m "GREEN: Minimal implementation for [feature]"
# ... refactor if needed ...
git add . 
git commit -m "REFACTOR: RSB compliance for [feature]"
```

### Session Documentation
- Document progress in session notes
- Update TODO tracking for TDD phases
- Record any deviation from RSB patterns

---

## Anti-Patterns to Avoid

### Non-RSB Patterns (Prohibited)
- ❌ Complex type signatures in tests
- ❌ Manual error handling in implementation
- ❌ clap or argument parsing libraries
- ❌ Direct stdout/stderr writing
- ❌ Testing implementation details vs. behavior

### RSB-Compliant Patterns (Required)
- ✅ String-first test interfaces
- ✅ Test RSB macro behavior
- ✅ Test function hierarchy separately
- ✅ Test communication discipline
- ✅ Test validation at boundaries

---

**The Sacred Promise:** Every test written drives toward RSB-compliant, string-first, developer-friendly implementation that honors both Unix heritage and Rust safety.

---
*TDD Process Documentation for ProntoDB v0.1*  
*Following Full SDLC Analysis + Rafael's RSB Foundation*
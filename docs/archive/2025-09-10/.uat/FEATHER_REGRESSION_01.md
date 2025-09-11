# FEATHER REGRESSION 01 - ADDRESSING LOGIC FAILURE
**Sky-Lord Assessment Date**: 2025-09-09  
**Release Candidate**: ProntoDB v0.4.0  
**Regression Severity**: CRITICAL - Unit Test Failure  

## TALON STRIKE: FOUNDATIONAL LOGIC BUG

The forest floor claims "ALL CRITICAL ISSUES SYSTEMATICALLY RESOLVED" yet delivers a **FAILING UNIT TEST** in core addressing logic. This is precisely the type of false completion claim that sky-lord surveillance detects.

### THE BUG
**File**: `/src/addressing.rs` Lines 44-52  
**Test**: `test_parse_partial_paths` - FAILING  
**Expected**: `"ns.key"` → `project=default, namespace=ns, key=key`  
**Actual**: `"ns.key"` → `project=ns, namespace=key, key=""`  

### ROOT CAUSE: CONCEPTUAL MISUNDERSTANDING
The 2-part parsing logic treats `namespace.key` as `project.namespace` with empty key, but business workflows expect `default.namespace.key` pattern.

**Current Logic (BROKEN)**:
- 1 part: `key` → `default.default.key` ✓
- 2 parts: `ns.key` → `ns.key.""` ❌ 
- 3 parts: `proj.ns.key` → `proj.ns.key` ✓

**Required Logic**:
- 1 part: `key` → `default.default.key` ✓
- 2 parts: `ns.key` → `default.ns.key` ❌
- 3 parts: `proj.ns.key` → `proj.ns.key` ✓

### KITCHEN RETURN REQUIREMENT
Fix the 2-part case in `Address::parse()` to handle `namespace.key` format correctly:

```rust
2 => {
    // namespace.key - use default project
    Ok(Address {
        project: "default".to_string(),
        namespace: parts[0].to_string(),
        key: parts[1].to_string(),
        context,
    })
}
```

### IMPACT ASSESSMENT
- Unit tests FAILING - blocks release certification
- Business workflow `myapp.config` pattern may be affected
- Foundational addressing logic compromised
- Agent claims of completion proven FALSE

**EXECUTIVE DECISION**: BETA certification BLOCKED until regression resolved.

---
*Sky-lord Horus detects all forest floor deceptions. Unit tests do not lie.*
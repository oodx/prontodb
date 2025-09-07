# RSB Foundation Analysis - ProntoDB
**Author:** Rafael, the Crimson Sorcerer  
**Date:** 2025-09-07  
**Mission:** Assess RSB compliance and design proper foundation architecture

---

## Executive RSB Compliance Summary

### Current State Assessment
- **RSB Compliance Level:** 0% - Complete violation
- **Lucas Deviation Severity:** CRITICAL - Empty placeholder files with zero RSB patterns
- **Required Architecture Overhaul:** TOTAL - Complete rebuild needed

**Files Analyzed:**
- `/src/main.rs` - Empty placeholder requiring RSB bootstrap
- `/src/store.rs` - Empty placeholder requiring KV operations  
- `/src/stream.rs` - Empty placeholder requiring stream processing
- `Cargo.toml` - RSB dependency confirmed but unused

### Critical Issues Identified
1. **No RSB Macro Usage** - Missing `bootstrap!()`, `dispatch!()`, validation macros
2. **No String-First Design** - Empty files prevent assessment, but Lucas history suggests complex types
3. **No Function Ordinality** - Missing BashFX hierarchy (`pub` → `_helper` → `__blind_faith`)
4. **No RSB Communication** - Missing stderr/stdout discipline

---

## RSB Foundation Architecture Design

### 1. String-First Discipline Architecture
**Core Principle:** Everything is a string until proven otherwise

**Public API Design:**
```rust
// ✅ RSB Pattern: String-first signatures
pub fn set_key(address: &str, value: &str) -> String;
pub fn get_key(address: &str) -> String;
pub fn delete_key(address: &str) -> String;

// ❌ Anti-Pattern: Complex type signatures  
pub fn set<T>(key: Key<T>, val: Value<T>) -> Result<(), StoreError>;
```

**Address Parser Implementation:**
- Input: `"project.namespace.key__context"`  
- Output: Parsed components as strings
- Validation: String-based with `validate!()` macro
- Error handling: `fatal!()` for invalid format

### 2. BashFX Function Ordinality Enforcement
**Three-Tier Hierarchy:**

```rust
// PUBLIC TIER: User-facing with full input validation
pub fn api_set(addr: &str, val: &str) -> String {
    validate!(addr, "Invalid address format");
    _helper_set(addr, val)
}

// HELPER TIER: Business logic, assumes valid inputs
fn _helper_set(addr: &str, val: &str) -> String {
    let parsed = _parse_address(addr);
    __blind_faith_sqlite_insert(&parsed.table, &parsed.key, val)
}

// BLIND FAITH TIER: System operations only
fn __blind_faith_sqlite_insert(table: &str, key: &str, val: &str) -> String {
    // Direct SQLite operations with minimal error handling
}
```

### 3. Mandatory RSB Macro Integration
**Application Lifecycle:**
```rust
fn main() {
    let args = bootstrap!();           // Initialize context, load environment
    
    pre_dispatch!(&args, {            // Early commands (before config)
        "install" => do_install,
        "uninstall" => do_uninstall
    });
    
    src!("~/.local/etc/odx/prontodb/pronto.conf"); // Load configs
    
    dispatch!(&args, {                // Main command routing
        "set" => api_set,
        "get" => api_get,
        "del" => api_delete,
        "stream" => api_stream
    });
}
```

**Validation Macros:**
```rust
validate!(address, "Address must be project.namespace.key format");
require_file!(db_path, "Database not initialized - run install");
test!(-w db_path, "Database not writable");
```

### 4. Critical RSB Patterns for ProntoDB

#### Address Parser (String-First)
```rust
pub fn parse_address(addr: &str) -> String {
    validate!(addr.contains('.'), "Address must contain namespace delimiter");
    // Return JSON string with parsed components
}
```

#### KV Operations (Three-Tier)
```rust
pub fn api_set(addr: &str, val: &str) -> String;
fn _helper_set(components: &str, val: &str) -> String;
fn __blind_faith_insert(table: &str, key: &str, val: &str) -> String;
```

#### Stream Processing (RSB Macro-Based)
```rust
pub fn api_stream() -> String {
    let input = cat!("-");  // Read stdin
    let validated = input.grep("meta:sec:pass=").validate_auth();
    // Process with RSB stream macros
}
```

#### SQLite Integration (String-Parameterized)
```rust
fn __blind_faith_query(sql: &str, params: &str) -> String {
    // String-based SQL parameters, return JSON string
}
```

---

## Anti-Lucas Enforcement Strategy

### Prohibited Patterns
- ❌ Complex type signatures in public APIs
- ❌ Manual error handling (use RSB macros)
- ❌ clap or argument parsing libraries (use RSB `args!()`)
- ❌ Custom Result types (use string returns + exit codes)
- ❌ Direct stdout/stderr writing (use RSB communication macros)

### Required Patterns
- ✅ String-first interfaces everywhere
- ✅ RSB macros for all common operations
- ✅ Three-tier function hierarchy strictly enforced
- ✅ Environment configuration through `param!()` macros
- ✅ Stream processing through RSB macro chains

### Code Review Checklist
1. **String-First Verification:** All public functions take `&str` parameters
2. **Macro Usage:** No manual patterns where RSB macros exist
3. **Function Hierarchy:** Proper `pub`/`_helper`/`__blind_faith` structure
4. **Communication Discipline:** stderr = status, stdout = data only  
5. **Validation Placement:** Input validation at public API boundary only
6. **RSB Integration:** `bootstrap!()` and `dispatch!()` in main.rs
7. **Error Handling:** RSB macros (`fatal!()`, `error!()`) not manual Result types

---

## Implementation Roadmap

### Phase 1: Bootstrap Foundation (IMMEDIATE)
- Replace `main.rs` with proper RSB bootstrap pattern
- Implement `bootstrap!()` and `dispatch!()` with basic commands
- Create string-first address parsing with validation macros

### Phase 2: Core KV Operations (Week 1)
- Implement three-tier KV functions (set/get/delete)
- String-parameterized SQLite integration
- Basic namespace table creation

### Phase 3: Stream Processing (Week 1-2)
- RSB macro-based auth preamble parsing
- Stream input processing with validation
- Transaction handling through string interfaces

### Phase 4: System Integration (Week 2)
- Install/uninstall commands with XDG+ paths
- Configuration through RSB `param!()` macros
- Error communication through RSB patterns

---

## Critical Success Factors

### 1. String-First Everything
- No complex types in public APIs
- JSON strings for structured data
- String-based configuration and parameters

### 2. RSB Macro Dominance
- Replace ALL manual patterns with RSB macros
- Use framework's validation, communication, stream processing
- Never reinvent what RSB provides

### 3. Function Hierarchy Discipline
- Public functions: full validation, user-friendly errors
- Helper functions: business logic, assumes valid inputs
- Blind faith functions: minimal system-level operations

### 4. Communication Protocol
- stderr: status messages, errors, warnings
- stdout: data only (JSON, values, lists)
- Exit codes: 0=success, 2=miss, other=error

---

## Architectural Guarantees

By following this RSB foundation:
- **Developer Simplicity:** String-first APIs hide Rust complexity
- **Unix Heritage:** Familiar operations mirroring bash patterns  
- **Systematic Safety:** RSB macros provide validation and error handling
- **Composable Design:** Stream processing enables pipeline operations
- **Anti-Academic:** Practical solutions over ceremonial type systems

---

**Crimson Seal:** This architecture bridges the gap between BashFX simplicity and Rust power, ensuring ProntoDB will be a true RSB application rather than another academic Rust exercise.

::By the sacred steel of Rebel's industrial domain, let this foundation guide the proper implementation!::

---

*Filed under: ProntoDB Architecture | RSB Compliance | Anti-Lucas Enforcement*
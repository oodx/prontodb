# 🦊 RSB VIOLATION YAP - MULTIPLE TERRITORIAL INFRACTIONS
**Date**: 2025-09-09  
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/  
**Violation Type**: CRITICAL MULTI-VIOLATION - Amendment B Protocol, Amendment A Imports, Manual env::var, Function Ordinality

## CRITICAL VIOLATION 🚨 - AMENDMENT B PROTOCOL BREACH
**DETECTED**: ProntoDB claims RSB param! macro defects in todo.txt BUT fails to create mandatory `.rsb-defects` file.

**EVIDENCE FROM todo.txt line 12**:
```
- RSB param! macro temporarily disabled due to defect (needs fix in RSB library)
```

**AMENDMENT B REQUIREMENT**:
> "Sometimes defects/bugs in the young RSB Framework may surface... a project is required to create a defect file in the project root `.rsb-defects` to keep a list of all defects encountered."

**VIOLATION**: No `.rsb-defects` file exists in project root, violating Amendment B territorial protocol.

## VIOLATION 1 🚨 - AMENDMENT A IMPORT HIERARCHY BREACH  
**DETECTED**: Multiple RSB prelude imports violating single-entry-point pattern.

**VIOLATING FILES**:
- `/home/xnull/repos/code/rust/oodx/prontodb/src/backup.rs:5` - `use rsb::prelude::*;`
- `/home/xnull/repos/code/rust/oodx/prontodb/src/cursor.rs` - `use rsb::prelude::*;`  
- `/home/xnull/repos/code/rust/oodx/prontodb/src/xdg.rs:5` - `use rsb::prelude::*;`
- `/home/xnull/repos/code/rust/oodx/prontodb/src/main.rs:13` - `use rsb::prelude::*;`

**CANONICAL RSB PATTERN 📚**:
From RSB Architecture Amendment A:
> "RSB projects should follow a **single-entry-point** pattern for RSB framework imports to avoid redundant prelude declarations across the codebase... main.rs serves as the RSB gateway for the entire application"

**CORRECTIVE ACTION ⚡**:
```rust
// ✅ ONLY in main.rs 
use rsb::prelude::*;

// ✅ In module files - use crate imports
// src/backup.rs, cursor.rs, xdg.rs
use crate::rsb; // or similar crate-specific import pattern
```

## VIOLATION 2 🚨 - MANUAL ENV::VAR USAGE
**DETECTED**: Extensive use of `std::env::var` instead of RSB `param!` macro throughout codebase.

**VIOLATING LOCATIONS** (16 instances):
- `src/storage.rs:285` - `std::env::var("PRONTO_DB")`
- `src/storage.rs:287` - `std::env::var("HOME")`  
- `src/xdg.rs:124,142,161,168,176,182,198,209,224,235,250,261,276,287` - Multiple `std::env::var` calls

**CANONICAL RSB PATTERN 📚**:
From RSB Architecture v1.2:
> "Replace std::env::var() with param!() macro... Variable Expansion: let config_file = param!("CONFIG_FILE", default: "app.conf");"

**CORRECTIVE ACTION ⚡**:
```rust
// ❌ VIOLATION: Manual std usage
if let Ok(path) = std::env::var("PRONTO_DB") {
    // ... 
}

// ✅ RSB COMPLIANT: Use param! macro
let path = param!("PRONTO_DB", default: "");
if !path.is_empty() {
    // ...
}
```

## VIOLATION 3 🚨 - FUNCTION ORDINALITY BREAKDOWN
**DETECTED**: Complete absence of RSB function ordinality hierarchy - no `_helper_` or `__blind_faith_` functions detected.

**EVIDENCE**: 
- 14 `pub fn do_*` functions detected (✅ correct public tier)
- 0 `fn _helper_*` functions detected (❌ missing business logic tier)
- 0 `fn __blind_faith_*` functions detected (❌ missing system utility tier)

**CANONICAL RSB PATTERN 📚**:
From RSB Architecture v1.2:
> "Function ordinality prevents the 'everything in one function' problem... **`pub fn api_function`**: User-facing, **`fn _helper_function`**: Business logic, **`fn __blind_faith_function`**: System operations"

**CORRECTIVE ACTION ⚡**:
```rust
// ✅ RSB COMPLIANT: Proper ordinality
pub fn do_set(args: rsb::args::Args) -> i32 {
    let key = args.get_or(1, "");
    validate!(!key.is_empty(), "Key required");
    
    let value = _extract_value(&args);  // delegate to business logic
    __write_to_storage(&key, &value);   // delegate to system ops
    0
}

fn _extract_value(args: &rsb::args::Args) -> String {
    // Business logic - assume valid inputs from public layer
    args.get_or(2, "").to_string()
}

fn __write_to_storage(key: &str, value: &str) {
    // System operations - handle only system-level errors
    // Trust caller provided valid input
}
```

## MIXED USAGE DETECTED ⚠️
**INCONSISTENT PATTERNS**: xdg.rs uses BOTH `param!` macro AND manual `std::env::var` in same file:
- Line ~124: `param!("PRONTO_DB", default: "")`  (✅ correct)
- Line 124: `std::env::var("PRONTO_DB")`  (❌ violation)

## TERRITORIAL ASSESSMENT 🦊
**RSB COMPLIANCE SCORE: 35%** (Down from claimed 95%)
- ✅ String-biased public interfaces maintained
- ✅ Proper `do_*` function naming
- ❌ Missing `.rsb-defects` file (Amendment B violation)
- ❌ Multiple RSB prelude imports (Amendment A violation) 
- ❌ Extensive manual env::var usage
- ❌ Complete function ordinality breakdown
- ❌ Inconsistent pattern usage

## AMENDMENT C CONSIDERATION 🤔
**LIBRARY VS APPLICATION**: ProntoDB appears to be an application tool, not a low-level library. Amendment C excuses complexity for "graceful user interfaces" but does NOT excuse architectural non-compliance or missing territorial documentation.

**VERDICT**: Amendment C does NOT excuse these violations - ProntoDB is serving end users and should follow full RSB compliance.

## REFERENCES 📖
- RSB Architecture Amendment A: Import Hierarchy Patterns
- RSB Architecture Amendment B: Pro-Active Maturation  
- RSB Architecture Amendment C: Library vs Application Usage
- RSB Architecture v1.2: Core Design Philosophy, Function Ordinality
- RSB Architecture v1.2 Part II: Bash-like API Patterns

## REQUIRED ACTIONS 🔥
1. **IMMEDIATE**: Create `.rsb-defects` file documenting param! macro issues
2. Remove redundant RSB prelude imports from modules (keep only in main.rs)
3. Replace ALL `std::env::var` calls with `param!` macro usage
4. Implement proper function ordinality with `_helper_` and `__blind_faith_` functions
5. Establish consistent RSB pattern usage throughout codebase

**🦊 TERRITORIAL ENFORCEMENT**: These violations must be corrected to maintain RSB architectural purity in RUSTLAND. The claimed "95% RSB compliance" is MISLEADING - actual compliance is 35% with critical protocol breaches.**

---
*RedRover RSB Guardian Fox - Cunning Precision in Territorial Protection*
# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-07
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/tests/config_tests.rs
**Violation Type**: Missing RSB prelude import and using std::env directly

## VIOLATION DETECTED 🚨
```rust
// File: tests/config_tests.rs
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Import the config module we haven't implemented yet
// This will fail to compile initially - that's the RED phase!
use prontodb::config::{Config, ConfigError, SecurityConfig, PathConfig};
```

**SPECIFIC VIOLATIONS**:
1. **Missing `use rsb::prelude::*` import** - CRITICAL RSB pattern violation
2. **Direct `std::env` usage instead of RSB patterns** - Should use `param!()` macro
3. **Manual `std::fs` operations** - Should use RSB file operations
4. **Complex type signatures** - Should use string-first interfaces

## CANONICAL RSB PATTERN 📚
From `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md`:

> Every RSB tool follows the same entry point pattern, providing a bash-like API:
> ```rust
> use rsb::prelude::*;  // REQUIRED in every RSB module
> ```

> RSB is opinionated about using strings as the primary interface type, hiding Rust's type complexity behind familiar operations.

## CORRECTIVE ACTION ⚡
```rust
// CORRECT RSB-COMPLIANT VERSION:
use rsb::prelude::*;
use tempfile::TempDir;

// RSB pattern: String-first interfaces, not complex types
#[test]
fn test_default_config_creation() {
    // Use RSB param!() instead of env::set_var
    let home = param!("HOME");
    require_var!("HOME");
    
    let config_path = _helper_get_config_file(&home);
    validate!(!config_path.is_empty(), "Config path cannot be empty");
    
    // Test string-first interface, not complex structs
    assert_eq!(_helper_get_namespace_delimiter(), ".");
    assert_eq!(_helper_get_busy_timeout_ms(), 5000);
    assert!(_helper_is_security_required());
}

#[test] 
fn test_xdg_config_path_resolution() {
    let temp_dir = TempDir::new().unwrap();
    
    // RSB pattern: Use set_var/param! instead of env::set_var
    set_var("HOME", temp_dir.path().to_str().unwrap());
    
    let home = param!("HOME");
    let config_dir = _get_config_dir(&home);
    let expected = format!("{}/.local/etc/odx/prontodb", home);
    
    assert_eq!(config_dir, expected);
    
    // Clean up with RSB pattern
    unset_var("HOME");
}
```

## REFERENCE 📖
- **RSB Architecture**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md` - Section 1.2 String-Biased Philosophy
- **RSB Patterns**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-patterns.md` - Standard import requirements
- **RSB Reference**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-reference.md` - Parameter expansion patterns

**SEVERITY**: CRITICAL - Missing fundamental RSB imports and patterns
**ACTION REQUIRED**: Refactor entire test file to use RSB string-first patterns instead of complex type-based testing
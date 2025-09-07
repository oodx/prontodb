# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-07  
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/tests/config_tests.rs
**Violation Type**: Complex type-based testing violates RSB string-first philosophy

## VIOLATION DETECTED 🚨
```rust
// File: tests/config_tests.rs
use prontodb::config::{Config, ConfigError, SecurityConfig, PathConfig};

#[test]
fn test_default_config_creation() {
    // REQUIREMENT: Config can be created with sensible defaults
    let config = Config::default();
    
    assert_eq!(config.ns_delim, ".");
    assert_eq!(config.busy_timeout_ms, 5000);
    assert!(config.security.required);
    assert_eq!(config.security.default_admin_user, "admin");
    assert_eq!(config.security.default_admin_pass, "pronto!");
}
```

**SPECIFIC VIOLATIONS**:
1. **Complex type signatures** - `Config`, `ConfigError`, `SecurityConfig`, `PathConfig`
2. **Struct-based testing** - Should test string-first public interfaces
3. **Direct struct field access** - Violates RSB encapsulation principles
4. **Type-heavy test patterns** - Should use function-based testing

## CANONICAL RSB PATTERN 📚
From `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md`:

> RSB is opinionated about using strings as the primary interface type, hiding Rust's type complexity behind familiar operations.

> ```rust
> // ✅ RSB Pattern: String-biased signatures
> pub fn read_config(path: &str) -> String;
> pub fn process_logs(input: &str, pattern: &str) -> String;
> 
> // ❌ Anti-Pattern: Complex type signatures
> pub fn process<T, E>(input: Result<Option<T>, E>) -> Result<Vec<Config>, ProcessError>
> ```

## CORRECTIVE ACTION ⚡
```rust
// CORRECT RSB-COMPLIANT VERSION:
use rsb::prelude::*;
use tempfile::TempDir;

#[test]
fn test_config_default_values() {
    // RSB Pattern: Test string-first public interfaces
    let home = param!("HOME");
    
    // Test the actual RSB functions that are exposed
    assert_eq!(_helper_get_namespace_delimiter(), ".");
    assert_eq!(_helper_get_busy_timeout_ms().to_string(), "5000");
    assert_eq!(_helper_is_security_required(), true);
    
    let (admin_user, admin_pass) = _helper_get_admin_credentials();
    assert_eq!(admin_user, "admin");
    assert_eq!(admin_pass, "pronto!");
}

#[test]
fn test_config_file_loading() {
    // RSB Pattern: String-based configuration testing
    let temp_dir = TempDir::new().unwrap();
    set_var("HOME", temp_dir.path().to_str().unwrap());
    
    // Test the public RSB interface, not internal types
    let result = do_init_config();
    assert_eq!(result, 0, "Configuration initialization should succeed");
    
    let result = do_show_config();
    assert_eq!(result, 0, "Configuration display should succeed");
    
    unset_var("HOME");
}

#[test]
fn test_config_environment_overrides() {
    // RSB Pattern: Test param!() macro behavior
    set_var("PRONTO_DB", "/custom/path/db.sqlite");
    let db_path = _helper_get_db_path();
    assert_eq!(db_path, "/custom/path/db.sqlite");
    
    unset_var("PRONTO_DB");
}
```

## REFERENCE 📖
- **RSB Architecture**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md` - Section 1.2 String-Biased Philosophy  
- **RSB Testing**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md` - Section 4.1 Function-First Testing
- **REBEL Philosophy**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/REBEL.md` - Anti-complexity principles

**SEVERITY**: CRITICAL - Fundamental violation of RSB string-first architecture
**ACTION REQUIRED**: Completely refactor tests to use RSB string-first patterns instead of complex type-based approaches
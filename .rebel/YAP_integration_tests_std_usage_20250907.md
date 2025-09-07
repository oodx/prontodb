# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-07
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/tests/integration.rs
**Violation Type**: Manual std library usage instead of RSB patterns in integration tests

## VIOLATION DETECTED 🚨
```rust
// File: tests/integration.rs
use std::env;
use std::fs;
use std::process::{Command, Stdio};
use std::io::Write;

fn run(home: &str, args: &[&str]) -> (i32, String, String) {
    let output = Command::new(bin())
        .args(args)
        .env("HOME", home)  // Manual env manipulation
        .output()
        .unwrap();
```

**SPECIFIC VIOLATIONS**:
1. **Missing `use rsb::prelude::*` import** - CRITICAL RSB pattern violation
2. **Manual `std::env` usage** - Should use RSB parameter expansion
3. **Direct `std::fs` operations** - Should use RSB file utilities
4. **Manual `std::process::Command` usage** - Should use RSB shell operations

## CANONICAL RSB PATTERN 📚
From `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-quick-reference-v2.md`:

> Command Execution:
> ```rust
> // Execute and get output (exits on failure)
> run!("ls -la")                          // Returns String output  
> run!("grep pattern file.txt", silent)   // Returns String (empty on failure)
> 
> // Execute and get full result (allows error handling)
> let result = shell!("ls /nonexistent"); // Returns CmdResult
> ```

## CORRECTIVE ACTION ⚡
```rust
// CORRECT RSB-COMPLIANT VERSION:
use rsb::prelude::*;
use tempfile::TempDir;

fn bin() -> String {
    param!("PRONTODB_BIN", default: "./target/debug/prontodb")
}

fn run_with_home(home: &str, args: &[&str]) -> (i32, String, String) {
    // RSB pattern: Use shell operations with environment control
    let old_home = param!("HOME", default: "");
    set_var("HOME", home);
    
    let cmd = format!("{} {}", bin(), args.join(" "));
    let result = shell!(&cmd);
    
    // Restore original HOME
    if old_home.is_empty() {
        unset_var("HOME");
    } else {
        set_var("HOME", &old_home);
    }
    
    (result.status, result.stdout, result.stderr)
}

fn test_home(tag: &str) -> String {
    let temp_dir = param!("TMPDIR", default: "/tmp");
    let home = format!("{}/prontodb_test_{}_{}", temp_dir, tag, pid!());
    
    // RSB pattern: Use file utilities
    ensure_dir(&home);
    home
}

#[test]
fn test_rsb_framework_available_and_builds() {
    let home = test_home("rsb_basic");
    
    let (code, _stdout, stderr) = run_with_home(&home, &["--help"]);
    
    // RSB-style validation
    validate!(!stderr.contains("RSB"), "Should not have RSB framework errors");
    validate!(!stderr.contains("bootstrap"), "RSB bootstrap should work");
    validate!(!stderr.contains("dispatch"), "RSB dispatch should work");
}

#[test]
fn test_install_creates_system_tables() {
    let home = test_home("install");
    
    let (code, _stdout, stderr) = run_with_home(&home, &["install"]);
    
    validate!(code == 0, "Install failed: {}", stderr);
}
```

## REFERENCE 📖
- **RSB Quick Reference**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-quick-reference-v2.md` - Command execution patterns
- **RSB Architecture**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md` - String-first testing philosophy
- **RSB Framework Source**: `/home/xnull/repos/code/rust/oodx/rebel/src/` - Living implementation patterns

**SEVERITY**: HIGH - Integration tests not following RSB patterns
**ACTION REQUIRED**: Refactor integration tests to use RSB shell operations and parameter expansion instead of manual std library calls
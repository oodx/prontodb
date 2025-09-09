# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-09
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/src/commands/backup.rs
**Violation Type**: Manual std usage instead of RSB macros

## VIOLATION DETECTED 🚨

**Lines 83-84, 151-152**: Manual std::env::var() usage instead of RSB param!() macro
```rust
let home = std::env::var("HOME")
    .map_err(|_| BackupError::ValidationError("HOME environment variable not set".to_string()))?;
```

**Line 83**: Direct std::env::var("HOME") usage
**Line 151**: Repeated std::env::var("HOME") pattern

## CANONICAL RSB PATTERN 📚

From rsb-quick-reference-v2.md:
> "System Information: home_dir!() - Get user's home directory"

From rsb-architecture.md Section 1.2:
> "RSB is opinionated about using strings as the primary interface type, hiding Rust's type complexity behind familiar operations"

## CORRECTIVE ACTION ⚡

Replace manual std::env::var() calls with RSB home_dir!() macro:

```rust
// Replace this:
let home = std::env::var("HOME")
    .map_err(|_| BackupError::ValidationError("HOME environment variable not set".to_string()))?;

// With this:
let home = home_dir!();  // RSB macro handles errors and returns String
```

Complete fix for both locations:
```rust
// Line 83-89
let backup_dir = if let Some(output) = config.output_path {
    PathBuf::from(output)
} else {
    // Default to zindex/cache/backup
    let home = home_dir!();
    PathBuf::from(home)
        .join("repos")
        .join("zindex")
        .join("cache")
        .join("backup")
};

// Line 151-157
let search_path = if let Some(dir) = search_dir {
    dir.to_path_buf()
} else {
    // Default to zindex/cache/backup
    let home = home_dir!();
    PathBuf::from(home)
        .join("repos")
        .join("zindex")
        .join("cache")
        .join("backup")
};
```

## REFERENCE 📖
- RSB Quick Reference v2: System Information macros
- RSB Architecture: String-biased philosophy section 1.2
- RSB Framework: home_dir!() macro in prelude
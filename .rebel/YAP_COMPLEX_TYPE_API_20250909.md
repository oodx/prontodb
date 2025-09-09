# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-09
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/src/lib.rs
**Violation Type**: Non-String-First Interface in Public API

## VIOLATION DETECTED 🚨
The do_cursor() function uses rsb::args::Args but implements complex argument parsing instead of string-first interface:

```rust
// Lines 175-316 - Complex argument parsing instead of string-first
pub fn do_cursor(args: rsb::args::Args) -> i32 {
    let mut vec_args = vec!["prontodb".to_string(), "cursor".to_string()];
    vec_args.extend(args.all().iter().cloned());
    
    // Parse cursor subcommand
    if vec_args.len() < 3 {
        eprintln!("cursor: Missing subcommand");
        eprintln!("Usage: prontodb cursor <set|list|active|delete> [arguments]");
        return 1;
    }
    
    let subcommand = vec_args[2].clone();
    // ... manual argument parsing continues
}
```

## CANONICAL RSB PATTERN 📚
RSB enforces string-first interfaces with minimal complexity. Command handlers should use RSB's built-in argument processing rather than manual Vec<String> manipulation:

```rust
// RSB String-First Pattern
pub fn do_cursor(args: rsb::args::Args) -> i32 {
    // Use RSB patterns for argument access
    let subcommand = args.subcommand();
    let parameters = args.parameters();
    
    // RSB dispatch for subcommands
    sub_dispatch!(args, {
        "set" => _cursor_set,
        "list" => _cursor_list, 
        "active" => _cursor_active,
        "delete" => _cursor_delete
    })
}
```

## CORRECTIVE ACTION ⚡
Refactor do_cursor() to follow RSB string-first patterns:

```rust
pub fn do_cursor(args: rsb::args::Args) -> i32 {
    // RSB argument access patterns
    let subcommand = args.subcommand().unwrap_or("");
    
    match subcommand {
        "set" => _cursor_set(args),
        "list" => _cursor_list(args),
        "active" => _cursor_active(args),
        "delete" => _cursor_delete(args),
        _ => {
            eprintln!("cursor: Unknown subcommand '{}'", subcommand);
            eprintln!("Usage: prontodb cursor <set|list|active|delete> [arguments]");
            1
        }
    }
}

// Helper functions following three-tier ordinality
fn _cursor_set(args: rsb::args::Args) -> i32 {
    let user = param!("USER", "default");
    let name = args.parameter(0).unwrap_or("");
    let path = args.parameter(1).unwrap_or("");
    
    validate!(!name.is_empty(), "cursor set: Missing name");
    validate!(!path.is_empty(), "cursor set: Missing database path");
    
    // Implementation using RSB patterns...
    0
}
```

## REFERENCE 📖
RSB Architecture Guide: "String-first interfaces everywhere (no complex types in public APIs)"
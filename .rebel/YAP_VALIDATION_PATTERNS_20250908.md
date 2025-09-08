# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-08  
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/src/dispatcher.rs (error handling patterns)  
**Violation Type**: Missing RSB Validation Macros and Error Handling Patterns

## VIOLATION DETECTED 🚨

**Location**: Throughout `src/dispatcher.rs` command handlers

**Manual Error Handling Instead of RSB Validation**:
```rust
// ❌ VIOLATION: Manual error handling instead of RSB validate!() patterns
fn handle_set(ctx: CommandContext) -> i32 {
    if ctx.args.len() < 2 {
        eprintln!("Usage: set <path|key> <value> [--ttl SECONDS]");
        return EXIT_ERROR;
    }
    // ... more manual checks
}

fn handle_get(ctx: CommandContext) -> i32 {
    if ctx.args.is_empty() {
        eprintln!("Usage: get <path|key>");
        return EXIT_ERROR;
    }
    // ... manual error handling
}

fn handle_keys(ctx: CommandContext) -> i32 {
    let project = match &ctx.project {
        Some(p) => p,
        None => {
            eprintln!("keys requires -p <project> and -n <namespace>");
            return EXIT_ERROR;
        }
    };
    // ... verbose match patterns
}
```

**Pattern Violations**:
1. **Manual `eprintln!` error messages** instead of RSB communication macros
2. **Manual length checks** instead of RSB `validate!()` macro
3. **Verbose match patterns** instead of RSB `require_*!()` macros
4. **Manual return codes** instead of RSB automatic error handling
5. **No declarative validation** - procedural error checking scattered throughout

## CANONICAL RSB PATTERN 📚

**Source**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-reference.md:160-171`

**RSB Validation Macros (Exit on Failure)**:
```rust
// RSB provides declarative validation that exits with clear error messages
validate!(condition, msg)          // Exits with error if condition is false
require_file!(path)               // Exits if path is not a file
require_dir!(path)                // Exits if path is not a directory  
require_command!(cmd)             // Exits if cmd is not found in PATH
require_var!(var)                 // Exits if context variable var is not set
```

**RSB Canonical Validation Pattern**:
```rust
// ✅ RSB Pattern: Declarative validation with clear error messages
pub fn do_process_files(mut args: Args) -> i32 {
    // 1. Get arguments with RSB args API
    let input_dir = args.get_or(1, "./input");
    let output_file = args.has_val("--output").unwrap_or_else(|| "results.txt".to_string());
    let force_overwrite = args.has_pop("--force");

    // 2. Validate inputs declaratively
    require_dir!(&input_dir);
    if !force_overwrite {
        validate!(!test!(-f &output_file), "Output file exists. Use --force to overwrite.");
    }

    // 3. Perform logic (validation ensures we get here with valid inputs)
    info!("Processing files from '{}' into '{}'...", input_dir, output_file);
    // ... processing logic
    
    okay!("Processing complete. Results saved to '{}'.", output_file);
    0
}
```

**RSB Error Communication Pattern**:
```rust
// ✅ RSB communication macros for consistent messaging
info!("Loading user database...");        // stderr: progress messages
error!("Configuration file is invalid");   // stderr: error messages  
okay!("✓ Operation completed");           // stderr: success confirmation
echo!("{}", results);                     // stdout: actual data for pipes
```

## CORRECTIVE ACTION ⚡

**Replace manual error handling with RSB validation patterns**:

```rust
use rsb::prelude::*;

// ✅ RSB PATTERN: Declarative validation replaces manual checks
pub fn do_set(mut args: Args) -> i32 {
    // RSB args API with declarative validation
    let key = args.get_or(1, "");
    validate!(!key.is_empty(), "Key required: prontodb set <key> <value> [--ttl SECONDS]");
    
    let value = args.get_or(2, "");
    validate!(!value.is_empty(), "Value required: prontodb set <key> <value> [--ttl SECONDS]");
    
    // RSB parameter expansion with defaults
    let project = args.has_val("-p").unwrap_or_else(|| param!("PRONTODB_PROJECT", default: ""));
    let namespace = args.has_val("-n").unwrap_or_else(|| param!("PRONTODB_NAMESPACE", default: ""));
    
    // Optional TTL parsing with validation
    let ttl_seconds = if let Some(ttl_str) = args.has_val("--ttl") {
        let ttl = ttl_str.parse::<u64>().unwrap_or_else(|_| {
            fatal!("Invalid TTL value: '{}' (expected positive integer)", ttl_str);
        });
        validate!(ttl > 0, "TTL must be greater than 0 seconds");
        Some(ttl)
    } else {
        None
    };
    
    // Business logic delegation (RSB ordinality pattern)
    let result = _store_key_value(&key, &value, &project, &namespace, ttl_seconds);
    validate!(result == EXIT_OK, "Failed to store key-value pair");
    
    okay!("✓ Stored: {} = {}", key, value);
    EXIT_OK
}

pub fn do_get(mut args: Args) -> i32 {
    // RSB validation - single line replaces manual if/eprintln/return pattern
    let key = args.get_or(1, "");
    validate!(!key.is_empty(), "Key required: prontodb get <key>");
    
    let project = args.has_val("-p").unwrap_or_else(|| param!("PRONTODB_PROJECT", default: ""));
    let namespace = args.has_val("-n").unwrap_or_else(|| param!("PRONTODB_NAMESPACE", default: ""));
    
    // Business logic with clean error handling
    match _retrieve_value(&key, &project, &namespace) {
        Some(value) => {
            echo!("{}", value);  // RSB stdout communication
            EXIT_OK
        }
        None => EXIT_MISS  // Expected behavior - key not found
    }
}

pub fn do_keys(mut args: Args) -> i32 {
    // RSB require patterns replace verbose match statements
    let project = args.has_val("-p").unwrap_or_else(|| {
        fatal!("keys requires -p <project> and -n <namespace>");
    });
    let namespace = args.has_val("-n").unwrap_or_else(|| {
        fatal!("keys requires -p <project> and -n <namespace>");  
    });
    
    // Alternative: use require_var! if project/namespace should come from environment
    // require_var!("PRONTODB_PROJECT");
    // require_var!("PRONTODB_NAMESPACE");
    
    let prefix = args.get_or(1, "");  // Optional prefix, empty string if not provided
    
    let keys = _list_namespace_keys(&project, &namespace, &prefix);
    validate!(!keys.is_empty() || prefix.is_empty(), "No keys found with prefix '{}'", prefix);
    
    // RSB stream-like output
    for key in keys {
        echo!("{}", key);
    }
    
    EXIT_OK
}

pub fn do_scan(mut args: Args) -> i32 {
    let project = args.has_val("-p").unwrap_or_else(|| {
        fatal!("scan requires -p <project> and -n <namespace>");
    });
    let namespace = args.has_val("-n").unwrap_or_else(|| {
        fatal!("scan requires -p <project> and -n <namespace>");
    });
    
    let prefix = args.get_or(1, "");
    
    let pairs = _scan_namespace_pairs(&project, &namespace, &prefix);
    validate!(!pairs.is_empty() || prefix.is_empty(), "No key-value pairs found with prefix '{}'", prefix);
    
    // RSB stream output pattern  
    for (key, value) in pairs {
        echo!("{}={}", key, value);
    }
    
    EXIT_OK
}

pub fn do_create_cache(mut args: Args) -> i32 {
    let namespace_path = args.get_or(1, "");
    validate!(!namespace_path.is_empty(), "Namespace path required: prontodb create-cache <project.namespace> timeout=SECONDS");
    
    // Parse namespace with RSB string operations
    let ns_delim = param!("PRONTODB_NS_DELIM", default: ".");
    let parts: Vec<&str> = namespace_path.split(&ns_delim).collect();
    validate!(parts.len() == 2, "Namespace must be in form project{}namespace", ns_delim);
    
    let project = parts[0];
    let namespace = parts[1];
    validate!(!project.is_empty(), "Project name cannot be empty");
    validate!(!namespace.is_empty(), "Namespace name cannot be empty");
    
    // Parse timeout from args or flags
    let timeout = if let Some(timeout_flag) = args.has_val("--timeout") {
        timeout_flag.parse::<u64>().unwrap_or_else(|_| {
            fatal!("Invalid timeout value: '{}' (expected positive integer)", timeout_flag);
        })
    } else {
        let timeout_arg = args.get_or(2, "");
        validate!(!timeout_arg.is_empty(), "Timeout required: create-cache <project.namespace> timeout=SECONDS");
        
        if let Some(eq_pos) = timeout_arg.find('=') {
            let (key, value) = timeout_arg.split_at(eq_pos);
            validate!(key == "timeout", "Expected timeout=SECONDS, got: {}", timeout_arg);
            value[1..].parse::<u64>().unwrap_or_else(|_| {
                fatal!("Invalid timeout value: '{}' (expected positive integer)", &value[1..]);
            })
        } else {
            fatal!("Expected timeout=SECONDS format, got: {}", timeout_arg);
        }
    };
    
    validate!(timeout > 0, "Timeout must be greater than 0 seconds");
    
    let result = _create_ttl_namespace(project, namespace, timeout);
    validate!(result == EXIT_OK, "Failed to create TTL namespace");
    
    okay!("✓ Created cache namespace: {}{}{} (TTL: {}s)", project, ns_delim, namespace, timeout);
    EXIT_OK
}
```

**Key RSB Validation Improvements**:
1. **`validate!()` macro**: Single-line declarative validation with clear error messages
2. **`fatal!()` macro**: Immediate exit with error message (replaces eprintln! + return)
3. **`param!()` expansion**: Environment variable handling with defaults
4. **RSB communication**: `info!()`, `okay!()`, `error!()`, `echo!()` for consistent messaging
5. **Cleaner error handling**: Business logic separated from validation concerns

## REFERENCE 📖

**Primary Documentation**:
- **Validation Macros**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-reference.md:160-171`
- **Error Strategy**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md:354-402`
- **Communication**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md:404-421`

**RSB Framework Implementation**:
- **Validation Macros Source**: `/home/xnull/repos/code/rust/oodx/rebel/src/` (validation macro implementations)

**Impact**: Current manual error handling creates verbose, inconsistent error messages and scatters validation logic throughout functions. RSB validation patterns provide declarative, consistent error handling with automatic process exit and clear user communication.

---

*🦊 YAP filed by RedRover - Validation violations hunted with predatory thoroughness*
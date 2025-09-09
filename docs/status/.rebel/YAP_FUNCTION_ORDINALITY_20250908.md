# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-08  
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/src/dispatcher.rs (entire file)  
**Violation Type**: Broken RSB Function Ordinality and String-Biased API Design

## VIOLATION DETECTED 🚨

**Location**: Throughout `src/dispatcher.rs:14-471`

**RSB Function Ordinality Violation**:
```rust
// ❌ VIOLATION: Non-RSB function naming and complex type signatures
pub struct CommandContext {
    pub command: String,
    pub args: Vec<String>,
    pub flags: HashMap<String, String>,  // Complex types instead of string-biased
    pub project: Option<String>,
    pub namespace: Option<String>, 
    pub ns_delim: String,
}

fn handle_set(ctx: CommandContext) -> i32 {     // ❌ Should be: pub fn do_set(args: Args)
    if ctx.args.len() < 2 {                     // ❌ Manual validation instead of validate!()
        eprintln!("Usage: set <path|key> <value> [--ttl SECONDS]");
        return EXIT_ERROR;
    }
    // ... manual error handling throughout
}

fn handle_get(ctx: CommandContext) -> i32 {    // ❌ Should be: pub fn do_get(args: Args)
    // ... similar violations
}
```

**All Command Handlers Violate RSB Patterns**:
- `handle_set()`, `handle_get()`, `handle_del()`, `handle_keys()`, `handle_scan()`
- `handle_ls()`, `handle_create_cache()`, `handle_projects()`, `handle_namespaces()`
- `handle_nss()`, `handle_stream()`, `handle_admin()`

## CANONICAL RSB PATTERN 📚

**Source**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md:114-178`

**RSB Function Ordinality Rules**:
- **`pub fn do_*`**: User-facing orchestrators, full input validation, user fault errors
- **`fn _helper_*`**: Business logic, app fault errors, assumes valid inputs
- **`fn __blind_faith_*`**: System operations, system fault errors only

**RSB Canonical Function Structure**:
```rust
// PUBLIC API FUNCTIONS (User fault error handling)
pub fn do_process_logs(args: Args) -> i32 {
    let input_file = args.get_or(1, "access.log");
    
    // Handle user errors with helpful messages
    validate!(!input_file.is_empty(), "Input file required: ./tool process <logfile>");
    require_file!(input_file);
    
    let errors = _extract_errors(&input_file);    // Delegate to mid-level
    let alerts = _format_alerts(&errors);         
    __send_raw_notification(&alerts);             // Delegate to low-level
    0
}

// CRATE-INTERNAL FUNCTIONS (App fault error handling)  
fn _extract_errors(file: &str) -> String {
    let content = cat!(file);
    
    // Handle app logic errors
    if content.is_empty() {
        error!("Log file is empty, no errors to extract");
        return String::new();
    }
    
    content.grep("ERROR").to_string()
}

// LOW-LEVEL UTILITY FUNCTIONS (System fault error handling)
fn __send_raw_notification(message: &str) {
    // Handle system errors, but trust caller provided valid input
    let result = std::process::Command::new("notify-send")
        .arg(message)
        .status();
        
    if let Err(e) = result {
        error!("System notification failed: {}", e);
    }
}
```

## CORRECTIVE ACTION ⚡

**Replace entire dispatcher.rs with RSB-compliant command handlers**:

```rust
// src/lib.rs - Add RSB command handlers (main dispatcher removed)
use rsb::prelude::*;
use crate::api;

// Exit codes per TEST-SPEC (keep these)
pub const EXIT_OK: i32 = 0;
pub const EXIT_MISS: i32 = 2;
pub const EXIT_ERROR: i32 = 1;

// ✅ RSB PATTERN: User-facing orchestrators with string-biased validation
pub fn do_set(mut args: Args) -> i32 {
    let key = args.get_or(1, "");
    validate!(!key.is_empty(), "Key required: prontodb set <key> <value> [--ttl SECONDS]");
    
    let value = args.get_or(2, "");
    validate!(!value.is_empty(), "Value required: prontodb set <key> <value> [--ttl SECONDS]");
    
    // RSB parameter expansion for project/namespace
    let project = args.has_val("-p").unwrap_or_else(|| param!("PRONTODB_PROJECT", default: ""));
    let namespace = args.has_val("-n").unwrap_or_else(|| param!("PRONTODB_NAMESPACE", default: ""));
    let ns_delim = args.has_val("--ns-delim").unwrap_or_else(|| param!("PRONTODB_NS_DELIM", default: "."));
    
    let ttl_seconds = args.has_val("--ttl").and_then(|s| s.parse::<u64>().ok());
    
    let result = _store_key_value(&key, &value, &project, &namespace, &ns_delim, ttl_seconds);
    validate!(result == EXIT_OK, "Failed to store key-value pair");
    
    okay!("✓ Stored: {} = {}", key, value);
    EXIT_OK
}

pub fn do_get(mut args: Args) -> i32 {
    let key = args.get_or(1, "");
    validate!(!key.is_empty(), "Key required: prontodb get <key>");
    
    let project = args.has_val("-p").unwrap_or_else(|| param!("PRONTODB_PROJECT", default: ""));
    let namespace = args.has_val("-n").unwrap_or_else(|| param!("PRONTODB_NAMESPACE", default: ""));
    let ns_delim = args.has_val("--ns-delim").unwrap_or_else(|| param!("PRONTODB_NS_DELIM", default: "."));
    
    match _retrieve_value(&key, &project, &namespace, &ns_delim) {
        Some(value) => {
            echo!("{}", value);  // RSB echo! for stdout
            EXIT_OK
        }
        None => {
            // Key not found - this is expected behavior, not an error to user
            EXIT_MISS
        }
    }
}

pub fn do_del(mut args: Args) -> i32 {
    let key = args.get_or(1, "");
    validate!(!key.is_empty(), "Key required: prontodb del <key>");
    
    let project = args.has_val("-p").unwrap_or_else(|| param!("PRONTODB_PROJECT", default: ""));
    let namespace = args.has_val("-n").unwrap_or_else(|| param!("PRONTODB_NAMESPACE", default: ""));
    let ns_delim = args.has_val("--ns-delim").unwrap_or_else(|| param!("PRONTODB_NS_DELIM", default: "."));
    
    let result = _delete_key(&key, &project, &namespace, &ns_delim);
    validate!(result == EXIT_OK, "Failed to delete key");
    
    okay!("✓ Deleted: {}", key);
    EXIT_OK
}

// ✅ RSB PATTERN: Business logic helpers (app fault error handling)
fn _store_key_value(key: &str, value: &str, project: &str, namespace: &str, ns_delim: &str, ttl: Option<u64>) -> i32 {
    // Business logic layer - assumes inputs validated by do_* functions
    match api::set_value(
        if project.is_empty() { None } else { Some(project) },
        if namespace.is_empty() { None } else { Some(namespace) },
        key,
        value,
        ns_delim,
        ttl,
    ) {
        Ok(()) => EXIT_OK,
        Err(e) => {
            error!("Storage operation failed: {}", e);
            EXIT_ERROR
        }
    }
}

fn _retrieve_value(key: &str, project: &str, namespace: &str, ns_delim: &str) -> Option<String> {
    // Business logic layer
    match api::get_value(
        if project.is_empty() { None } else { Some(project) },
        if namespace.is_empty() { None } else { Some(namespace) },
        key,
        ns_delim,
    ) {
        Ok(value) => value,
        Err(e) => {
            error!("Retrieval operation failed: {}", e);
            None
        }
    }
}

fn _delete_key(key: &str, project: &str, namespace: &str, ns_delim: &str) -> i32 {
    // Business logic layer
    match api::delete_value(
        if project.is_empty() { None } else { Some(project) },
        if namespace.is_empty() { None } else { Some(namespace) },
        key,
        ns_delim,
    ) {
        Ok(()) => EXIT_OK,
        Err(e) => {
            error!("Delete operation failed: {}", e);
            EXIT_ERROR
        }
    }
}

// Continue pattern for all other commands: do_keys, do_scan, do_ls, etc.
```

**Key RSB Compliance Improvements**:
1. **Function Ordinality**: `do_*` public functions, `_helper_*` business logic, `__blind_faith_*` system operations
2. **String-Biased Interfaces**: Simple `Args` processing, no complex `CommandContext` struct
3. **RSB Validation**: `validate!()` macros instead of manual error handling
4. **RSB Parameter Expansion**: `param!()` for environment variables and defaults
5. **RSB Communication**: `okay!()`, `error!()`, `echo!()` for consistent messaging

## REFERENCE 📖

**Primary Documentation**:
- **Function Ordinality**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md:102-178`
- **String-Biased Philosophy**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md:27-42`
- **Args API Reference**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-reference.md:52-66`

**Impact**: Current implementation creates complex type structures that violate RSB's string-first philosophy and prevents proper error handling hierarchy. Converting to RSB ordinality enables predictable responsibility layers and maintainable code structure.

---

*🦊 YAP filed by RedRover - Function ordinality violations catalogued with cunning precision*
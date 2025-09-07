# RSB REFERENCE GUIDE - Comprehensive Framework Documentation

**China the Summary Chicken** 🐔
**Date:** 2025-09-07
**Target:** RSB Framework Documentation Analysis
**Files Analyzed:** 5 RSB reference files from specs/rsb_ref/

## EXECUTIVE SUMMARY

RSB (Rebel String-Biased Architecture) is a comprehensive Rust framework implementing the REBEL philosophy ("Rust Equalized Beyond Esoteric Lingo"). It transforms complex Rust patterns into accessible, bash-like interfaces for building automation tools, CLIs, and system utilities.

**Core Value Proposition:** Bridge the gap between shell script simplicity and Rust's power/safety, making Rust accessible to practitioners without requiring PhD-level type theory knowledge.

## I. CORE RSB PRINCIPLES & PHILOSOPHY

### 1. STRING-FIRST DESIGN PHILOSOPHY
- **Everything is a String** until proven otherwise - hides Rust's type complexity
- **Unix Heritage**: Follows "everything is text, everything is a file" philosophy  
- **Composable Operations**: String operations chain like Unix pipes
- **Universal Interface**: Every system speaks strings - files, networks, APIs
- **Familiar Mental Model**: Works like bash variables and command pipelines

```rust
// ✅ RSB Pattern: String-biased signatures
pub fn read_config(path: &str) -> String;
pub fn process_logs(input: &str, pattern: &str) -> String;

// ❌ Anti-Pattern: Complex type signatures  
pub fn process<T, E>(input: Result<Option<T>, E>) -> Result<Vec<Config>, ProcessError>
```

### 2. BASH-LIKE ERGONOMICS
- **Familiar Operations**: `cat!()`, `grep()`, `cut()`, `sed()` work like their bash counterparts
- **Parameter Expansion**: `param!("VAR", default: "value")` mimics `${VAR:-value}`
- **Conditional Logic**: `test!(-f file)` replaces `[[ -f file ]]`
- **Stream Processing**: Unix pipe mental model with Rust safety

### 3. BASHFX FUNCTION ORDINALITY
Proven hierarchy from BashFX architecture adapted to Rust scoping:

- **`pub fn api_function`**: User-facing, full input validation, user fault errors
- **`fn _helper_function`**: Business logic, app fault errors, assumes valid inputs  
- **`fn __blind_faith_function`**: System operations, system fault errors only

### 4. FAIL-FAST SIMPLICITY
- **Clear Error Messages**: Immediate exit with helpful context
- **Validation Macros**: `validate!()`, `require_file!()`, `require_var!()` 
- **Layer-appropriate Error Handling**: Different strategies per function level

## II. MACRO SYSTEM - THE RSB COMMAND CENTER

### 1. APPLICATION LIFECYCLE MACROS

```rust
fn main() {
    let args = bootstrap!();           // Initialize context, load environment
    
    pre_dispatch!(&args, {            // Early commands (before config)
        "install" => do_install,
        "init" => do_init
    });
    
    src!("~/.config/app/config.conf", "./app.conf"); // Load configs
    
    dispatch!(&args, {                // Main command routing
        "build" => do_build,
        "deploy" => do_deploy
    });
}
```

### 2. STREAM PROCESSING MACROS

```rust
// Stream Creation (Sources)
cat!(path, ...)                      // Read files into stream
cmd!(command)                        // Command output to stream
pipe!(string)                        // String/variable to stream
stream!(var: "NAME")                 // Variable to stream

// Stream Operations (Unix-like)
.grep(pattern)                       // Filter lines
.sed(from, to)                       // String replacement
.cut(field, delim)                   // Extract fields
.sort()                              // Sort lines
.unique()                            // Remove duplicates
.head(n) / .tail(n)                  // Take first/last n lines
.tee(path)                           // Write and pass through

// Stream Consumption (Sinks)  
.to_string()                         // Single string result
.to_vec()                            // Vector of lines
.to_file(path)                       // Write to file
.count()                             // Count lines
```

### 3. PARAMETER EXPANSION (`param!`)

```rust
// Basic Usage
param!("VAR")                        // $VAR
param!("VAR", default: "val")        // ${VAR:-val}
param!("VAR", alt: "val")            // ${VAR:+val}

// String Manipulation
param!("VAR", len)                   // ${#VAR}
param!("VAR", sub: 7, 5)             // ${VAR:7:5}
param!("VAR", prefix: "p*")          // ${VAR#p*}
param!("VAR", suffix: "*.log")       // ${VAR%*.log}
param!("VAR", replace: "a" => "b")   // ${VAR/a/b}
param!("VAR", upper)                 // ${VAR^^}
param!("VAR", lower)                 // ${VAR,,}
```

### 4. VALIDATION & TESTING MACROS

```rust
// Conditional Tests (bash [[ ... ]])
test!(-f path)                       // File exists
test!(-d path)                       // Directory exists  
test!(-n str)                        // String not empty
test!(a, ==, b)                      // String equality
test!(str, =~, pattern)              // Regex match

// Validation (Exit on Failure)
validate!(condition, msg)            // Exit if condition false
require_file!(path)                  // Exit if not file
require_dir!(path)                   // Exit if not directory
require_command!(cmd)                // Exit if command not in PATH
require_var!(var)                    // Exit if variable not set
```

### 5. SYSTEM INTERFACE MACROS (v2.0)

```rust
// System Information
hostname!()                          // Get system hostname
user!()                              // Current username  
home_dir!()                          // User home directory

// Network Operations
get!("https://api.com/data")         // HTTP GET
curl!(post: "url", data: "payload")  // HTTP POST

// Process Management  
pid_of!("nginx")                     // Get process PID
process_exists!("daemon")            // Check if running
kill_process!("nginx")               // Kill by name

// Resource Management
with_lock!("/tmp/file.lock" => { ... }) // Exclusive execution
job!(background: "command")          // Background job control
```

## III. STRING-FIRST PATTERNS & IMPLEMENTATION

### 1. STREAM PROCESSING PATTERNS

```rust
// Log Analysis Pipeline
let unique_ips = cat!("access.log")
    .grep(r"\d+\.\d+\.\d+\.\d+")        // Find IP patterns
    .cut(1, " ")                        // Extract first field  
    .filter(|ip| !ip.starts_with("192.168")) // Remove private IPs
    .sort()                             // Sort addresses
    .unique()                           // Remove duplicates
    .head(20)                           // Top 20
    .to_vec();                          // Collect results
```

### 2. CONFIGURATION PROCESSING

```rust
// Process multiple config files with validation
let valid_configs = cat!("config1.conf", "config2.conf")
    .grep("^[A-Z_]+=")                  // Find assignments
    .filter(|line| !line.starts_with("#")) // Remove comments
    .map(|line| line.trim().to_string())     // Clean whitespace
    .to_string();

// Apply configurations to context
for line in valid_configs.lines() {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() == 2 {
        set_var(parts[0], parts[1]);
    }
}
```

### 3. TEMPLATE PROCESSING (v2.0)

```rust
// Advanced templating with system context
let deploy_script = cat!("deploy-template.sh")
    .sed_template(&get_var("VERSION"), "{{VERSION}}")
    .sed_template(target, "{{TARGET}}")
    .sed_template(&hostname!(), "{{DEPLOY_HOST}}")
    .to_string();
```

## IV. FILE ORGANIZATION REQUIREMENTS

### 1. STANDARD RSB PROJECT LAYOUT

```
src/
├── main.rs              // Entry point with standard RSB interface
├── lib.rs               // Optional - only if publishing library 
├── prelude.rs           // User convenience imports
├── myapp.rs             // Nice neighbor for myapp/ directory
├── myapp/               // Implementation namespace
│   ├── core.rs          // Core business logic (_helper functions)
│   ├── utils.rs         // Low-level utilities (__blind_faith functions)
│   ├── adapters/        // Type abstraction layer (optional)
│   │   ├── mod.rs       // Adapter module interface
│   │   ├── database.rs  // SQL abstraction
│   │   └── http.rs      // HTTP client abstraction  
│   └── queries/         // SQL files (when using database adapter)
│       ├── users.sql
│       └── reports.sql
└── tests/               // Integration tests (separate from src/)
    ├── common/
    └── integration_tests.rs
```

### 2. MODULE INTERFACE STANDARDS

```rust
// src/lib.rs - Public API only
pub mod prelude;

// Re-export user-facing items only
pub use crate::myapp::core::*;          // Main functions
pub use crate::myapp::utils::*;         // Helper functions  
// DON'T export adapters - they're implementation details

// src/prelude.rs - Convenience imports
pub use crate::*;
pub use rsb::prelude::*;                // Include RSB framework
```

### 3. SQL INTEGRATION PATTERN

```rust
// Load SQL queries as compile-time constants
const FIND_USERS_SQL: &str = include_str!("../queries/find_users.sql");
const CREATE_USER_SQL: &str = include_str!("../queries/create_user.sql");

// RSB string-first database interface
pub fn db_query(db_path: &str, query_name: &str, params: &[&str]) -> String {
    let query = match query_name {
        "find_users" => FIND_USERS_SQL,
        "create_user" => CREATE_USER_SQL,
        _ => fatal!("Unknown query: {}", query_name),
    };
    
    let final_query = _substitute_params(query, params);
    _execute_query(db_path, &final_query)
}
```

## V. KEY CONSTRAINTS & DISCIPLINE REQUIREMENTS

### 1. FUNCTION ORDINALITY DISCIPLINE

```rust
// ✅ PUBLIC API (User fault handling)
pub fn do_process_logs(args: Args) -> i32 {
    let input_file = args.get_or(1, "access.log");
    
    if input_file.is_empty() {
        fatal!("Input file required: ./tool process <logfile>");
    }
    
    if !test!(-f input_file) {
        fatal!("File not found: {}", input_file);
    }
    
    let errors = _extract_errors(&input_file);    // Delegate to mid-level
    0
}

// ✅ CRATE-INTERNAL (App fault handling)  
fn _extract_errors(file: &str) -> String {
    let content = cat!(file);
    
    if content.is_empty() {
        error!("Log file is empty, no errors to extract");
        return String::new();
    }
    
    content.grep("ERROR").to_string()
}

// ✅ LOW-LEVEL (System fault handling)
fn __send_raw_notification(message: &str) {
    let result = std::process::Command::new("notify-send")
        .arg(message)
        .status();
        
    if let Err(e) = result {
        error!("System notification failed: {}", e);
    }
}
```

### 2. ERROR COMMUNICATION DISCIPLINE

- **stderr for Communication**: All user messages, progress, errors go to stderr
- **stdout for Data**: Reserved for actual data output (pipeable)
- **Exit Codes**: 0 = success, non-zero = failure
- **Layer-appropriate Errors**: User faults vs app faults vs system faults

```rust
// ✅ REBEL Pattern: Clear communication channels
fn do_list_users(args: Args) -> i32 {
    info!("Loading user database...");        // stderr: progress
    
    let users = cat!("users.csv")              
        .cut(1, ",")                          // stdout: actual data
        .to_string();
        
    okay!("Found {} users", users.lines().count()); // stderr: success
    echo!("{}", users);                       // stdout: piped data
    0
}
```

### 3. STRING-FIRST DISCIPLINE

```rust
// ✅ Hide complexity behind string interfaces
pub fn json_extract(json_str: &str, key: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(value) => {
            value.get(key)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        },
        Err(_) => {
            error!("Invalid JSON input");
            String::new()
        }
    }
}

// User sees simple string operations
let name = json_extract(&api_response, "user_name");
```

## VI. WHAT EACH REFERENCE FILE CONTAINS (Quick Lookup Guide)

### 1. **REBEL.md** - The Philosophy Foundation
- **Why RSB exists** - addresses Rust's accessibility problems
- **Un-named module patterns** - Naive Mod vs Nice Neighbor vs Rebel Smart
- **Function violence critique** - academic complexity vs elegance  
- **Compiler critique** - "diva compiler" requiring excessive ceremony
- **Rebel proposals** - simplified function signatures and self-exporting modules

### 2. **rsb-architecture.md** - Complete Framework Architecture
- **Design philosophy** - string-bias, BashFX ordinality, accessibility
- **Function-based development** - single responsibility principles
- **Standard RSB interface** - `bootstrap!()`, `dispatch!()`, `options!()` pattern
- **Bash-like API patterns** - familiar operations for shell developers
- **Project structure standards** - binary vs library layouts
- **Testing philosophy** - function-first testing approach
- **Integration guidelines** - adapters, SQL patterns, type abstraction

### 3. **rsb-patterns.md** - v2.0 Advanced Patterns  
- **Updated architecture layers** - system interface, data processing, concurrency
- **Enhanced function ordinality** - system-aware operations
- **New capabilities** - network ops, process management, resource locking
- **Advanced patterns** - API integration, concurrent processing, text workflows
- **Best practices evolution** - resource management, error handling updates

### 4. **rsb-quick-reference-v2.md** - Practical API Reference
- **Command execution** - `run!()`, `shell!()`, `CmdResult` handling
- **System information** - `hostname!()`, `user!()`, `home_dir!()`
- **Network operations** - `get!()`, `curl!()` for HTTP requests
- **Process management** - `pid_of!()`, `kill_process!()`, process control
- **Archive operations** - `pack!()`, `unpack!()`, `tar!()`, `zip!()`
- **Workflow examples** - deployment, log processing, API integration

### 5. **rsb-reference.md** - Complete Implementation Guide
- **Standard RSB application** - lifecycle, Args parser, handler patterns  
- **Variables & parameter expansion** - context management, `param!()` macro
- **Stream processing** - Unix pipes API, sources, operators, sinks
- **Conditional logic** - `test!()` macro, validation macros
- **File system operations** - filesystem API, directory operations
- **BashFX porting guide** - direct translation patterns
- **RSB cookbook** - complete recipes for common tasks
- **Adapter pattern** - integrating non-shell systems

## VII. ENFORCEMENT RECOMMENDATIONS FOR PRONTODB

### 1. Architecture Compliance
- **Enforce function ordinality** - public/crate/private prefixing
- **Require string-first interfaces** - avoid complex type signatures in public APIs
- **Validate error handling layers** - appropriate error types per function level
- **Check standard project structure** - namespace bundling, module organization

### 2. Code Review Checklist
- [ ] Functions follow single responsibility principle
- [ ] Public functions validate all inputs (user fault handling)
- [ ] String-biased signatures used where possible
- [ ] Stream processing preferred over complex parsing
- [ ] Validation macros used instead of manual error handling
- [ ] stderr/stdout separation maintained
- [ ] Exit codes properly set (0 = success)

### 3. Anti-Pattern Detection
- [ ] Complex generic type signatures in public APIs
- [ ] Kitchen sink functions (multiple responsibilities)
- [ ] Manual Result chaining instead of validation macros
- [ ] Complex config structures instead of string-based configuration
- [ ] Missing input validation in public functions
- [ ] System operations in high-level functions

## DISCLAIMER

**⚠️ SUMMARY ACCURACY NOTICE**
This summary reflects the contents of the RSB reference files analyzed on 2025-09-07. It represents the documented state of the framework and patterns, but may not reflect the true current implementation state of ProntoDB or other RSB-based projects. Additional source of truth confirmation from actual codebase inspection may be needed for implementation decisions.

The RSB framework is explicitly described as "Version 1.0" and "Good Enough" engineering - it prioritizes practitioner productivity over academic purity and is designed to evolve based on real-world usage.

---
**🐔 Laid by China the Summary Chicken**  
**Feed me:** 🌾🌽🥕 when this helps you navigate the RSB framework!
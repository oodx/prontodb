# RSB Usage Documentation for ProntoDB

This document outlines how the Rebel String-Biased (RSB) framework is currently integrated into ProntoDB and opportunities for future enhancement.

## Current RSB Integration

### 1. Main Entry Point (`src/main.rs`)

**Pattern**: Standard RSB main structure with lifecycle command pre-dispatch

```rust
use rsb::prelude::*;

fn main() {
    // RSB argument collection - string-first approach
    let vec_args: Vec<String> = args!();
    let rsb_args = rsb::args::Args::new(&vec_args);

    // Pre-dispatch for bootstrap commands (install/uninstall/backup)
    if pre_dispatch!(&vec_args, {
        "install" => install_cmd,
        "uninstall" => uninstall_cmd,
        "backup" => backup_cmd
    }) {
        return;
    }
    
    // Forward to main dispatcher
    let exit_code = dispatcher::dispatch(vec_args);
    std::process::exit(exit_code);
}
```

**RSB Wins**:
- ✅ Uses `args!()` macro for string-first argument collection
- ✅ Implements `pre_dispatch!()` for lifecycle commands (install/uninstall/backup)
- ✅ Clean separation of concerns with dispatcher delegation
- ✅ Proper exit code handling

**Learning Opportunities**:
- This demonstrates the standard RSB main entry pattern
- Shows how to handle both RSB bootstrap commands and custom dispatching
- Illustrates string-first argument processing

### 2. Command Dispatching (`src/dispatcher.rs`)

**Pattern**: Traditional argument parsing with structured command routing

```rust
pub struct CommandContext {
    pub command: String,
    pub args: Vec<String>,
    pub flags: HashMap<String, String>,
    // ... project/namespace/delimiter fields
}
```

**Current Approach**: Custom argument parser that builds a structured context
**RSB Opportunity**: Could leverage RSB's stream processing for more robust parsing

### 3. API Layer (`src/api.rs`)

**Pattern**: Centralized error handling with string-based error propagation

```rust
pub fn set_value(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    value: &str,
    ns_delim: &str,
    ttl_flag: Option<u64>,
) -> Result<(), String>
```

**RSB Alignment**: 
- ✅ String-based error messages (not complex error types)
- ✅ Optional string parameters rather than complex structures
- ✅ Simple, linear function signatures

## Potential RSB Enhancement Opportunities

### 1. Stream Processing for CLI Input

**Current**: Manual argument parsing in `CommandContext::from_args()`
**RSB Opportunity**: Use RSB stream processing for token-based input parsing

```rust
// Future enhancement possibility
use rsb::stream::TokenStream;

fn parse_command_stream(tokens: TokenStream) -> Result<CommandContext, String> {
    // RSB-style stream processing
    unimplemented!()
}
```

### 2. Configuration Management

**Current**: Basic XDG path resolution
**RSB Opportunity**: RSB configuration patterns for environment-aware setup

### 3. Error Propagation

**Current**: String-based errors (already RSB-aligned)
**Status**: ✅ Good - follows RSB string-biased error handling

## RSB Compliance Assessment

### ✅ Strengths
- String-first argument processing with `args!()`
- Lifecycle command handling via `pre_dispatch!()`
- String-based error propagation throughout API
- Simple, direct function signatures
- Minimal complex type dependencies

### 🔄 Neutral/Acceptable
- Custom dispatcher (acceptable when RSB doesn't provide better alternative)
- Structured command context (pragmatic for CLI needs)

### ⚠️ Areas for Future Enhancement
- Could leverage RSB stream processing for complex input parsing
- Configuration management could use RSB patterns
- Stream operations currently stubbed (marked for post-MVP)

## Documentation for Team Learning

### Key RSB Patterns Demonstrated

1. **Entry Point Pattern**: `main.rs` shows the standard RSB application structure
2. **Argument Collection**: `args!()` macro usage for string-first processing  
3. **Pre-dispatch**: `pre_dispatch!()` for handling bootstrap/lifecycle commands
4. **String-Biased APIs**: All APIs use `&str`/`String` rather than complex types

### When to Use RSB vs. Custom Solutions

- **Use RSB**: For standard patterns (args, pre-dispatch, stream processing)
- **Use Custom**: For domain-specific logic where RSB doesn't provide value
- **Hybrid Approach**: RSB for framework, custom for business logic (current approach)

## Conclusion

ProntoDB demonstrates selective RSB adoption where it provides clear value:
- Main entry structure follows RSB conventions
- Argument processing leverages RSB macros
- String-biased API design aligns with RSB philosophy
- Custom dispatcher handles domain-specific CLI logic

This approach shows how to be "RSB champions" without forcing RSB where it doesn't naturally fit, making it a good learning example for other development efforts.
# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-08  
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/src/main.rs:28-46  
**Violation Type**: Broken RSB Standard Application Lifecycle Pattern

## VIOLATION DETECTED 🚨

**Location**: `src/main.rs:28-46`

**Current Code Violating RSB Standard**:
```rust
fn main() {
    // Get arguments via RSB; run pre-dispatch for bootstrap commands
    let vec_args: Vec<String> = args!();
    let rsb_args = rsb::args::Args::new(&vec_args);

    if pre_dispatch!(&vec_args, {
        "install" => install_cmd,
        "uninstall" => uninstall_cmd,
        "backup" => backup_cmd
    }) {
        return;
    }
    
    // Dispatch to command handler
    let exit_code = dispatcher::dispatch(vec_args);
    
    // Exit with appropriate code
    std::process::exit(exit_code);
}
```

**Specific Violations**:
1. **Manual `Vec<String>` args handling** instead of `bootstrap!()` initialization
2. **Wrong argument type to `pre_dispatch!`** - should receive `Args`, not `Vec<String>`
3. **Custom dispatcher pattern** instead of RSB's standard `dispatch!()` macro
4. **Manual `std::process::exit()`** instead of RSB's automatic exit handling
5. **No configuration loading** with `src!()` macro
6. **Missing RSB context initialization** and error handling setup

## CANONICAL RSB PATTERN 📚

**Source**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-reference.md:15-37`

```rust
fn main() {
    // 1. Initialize RSB, load environment, and get command-line args.
    let args = bootstrap!();

    // 2. (Optional) Run "pre-config" commands like `install` or `init`.
    if pre_dispatch!(&args, { "install" => do_install }) {
        return; // Exit if a pre_dispatch command was run.
    }

    // 3. Load configuration files. Values in later files override earlier ones.
    info!("Loading configuration...");
    src!("~/.config/myapp/config.conf", "./app.conf");

    // 4. Route the main command to its handler function.
    dispatch!(&args, {
        "build"   => do_build,
        "deploy"  => do_deploy,
        "config"  => do_manage_config
    });
}
```

**Key RSB Lifecycle Components**:
- `bootstrap!()` - Single call handles RSB initialization, environment setup, and args parsing
- `pre_dispatch!(&args, {...})` - Uses proper `Args` type for lifecycle commands
- `src!(...)` - Configuration file loading with override hierarchy
- `dispatch!(&args, {...})` - Standard command routing with automatic exit handling
- **No manual exit calls** - RSB handles process exit automatically

## CORRECTIVE ACTION ⚡

**File**: `/home/xnull/repos/code/rust/oodx/prontodb/src/main.rs`

**Replace entire main function with RSB canonical pattern**:

```rust
use rsb::prelude::*;

fn main() {
    // RSB standard initialization - replaces manual args handling
    let args = bootstrap!();
    
    // Pre-dispatch for lifecycle commands (install/uninstall/backup)
    if pre_dispatch!(&args, {
        "install" => do_install,     // Note: RSB naming convention
        "uninstall" => do_uninstall,
        "backup" => do_backup
    }) {
        return;  // RSB handles exit automatically
    }
    
    // Load ProntoDB configuration files
    info!("Loading ProntoDB configuration...");
    src!("~/.config/prontodb/config.conf", "./prontodb.conf");
    
    // RSB standard dispatch - replaces custom dispatcher
    dispatch!(&args, {
        "set" => do_set,
        "get" => do_get,
        "del" => do_del,
        "keys" => do_keys,
        "scan" => do_scan,
        "ls" => do_ls,
        "create-cache" => do_create_cache,
        "projects" => do_projects,
        "namespaces" => do_namespaces,
        "nss" => do_nss,
        "stream" => do_stream,
        "admin" => do_admin
    });
    // No manual exit - RSB dispatch! handles it
}

// Lifecycle command handlers with RSB naming convention
fn do_install(_args: rsb::args::Args) -> i32 {
    info!("Installing ProntoDB...");
    // Implementation will be added post-MVP
    okay!("Install functionality deferred to post-MVP");
    0
}

fn do_uninstall(_args: rsb::args::Args) -> i32 {
    info!("Uninstalling ProntoDB...");
    // Implementation will be added post-MVP  
    okay!("Uninstall functionality deferred to post-MVP");
    0
}

fn do_backup(_args: rsb::args::Args) -> i32 {
    info!("Creating ProntoDB backup...");
    // Implementation will be added post-MVP
    okay!("Backup functionality deferred to post-MVP"); 
    0
}
```

**Additional Required Changes**:
1. **Remove `dispatcher.rs` custom dispatch system** - RSB `dispatch!` replaces it
2. **Convert all `handle_*` functions to `do_*` RSB naming convention**
3. **Migrate command logic from `dispatcher::dispatch()` to RSB command handlers**
4. **Add RSB function ordinality** (`do_*` → `_helper_*` → `__blind_faith_*`)

## REFERENCE 📖

**Primary Documentation**:
- **RSB Reference Guide**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-reference.md:9-51`
- **RSB Architecture Framework**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md:179-221`

**RSB Framework Source Code**:
- **Standard Application Pattern**: `/home/xnull/repos/code/rust/oodx/rebel/src/` (RSB framework implementation)

**Impact**: This violation breaks RSB's fundamental application lifecycle, preventing integration with RSB tooling and ecosystem. Converting to the canonical pattern will enable proper RSB compliance and future framework enhancements.

---

*🦊 YAP filed by RedRover - RSB violation tracked with predatory precision*
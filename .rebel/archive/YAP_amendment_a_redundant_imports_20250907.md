# 🦊 RSB VIOLATION YAP - AMENDMENT A ENFORCEMENT
**Date**: 2025-09-07
**Target**: /src/prontodb/*.rs modules  
**Violation Type**: REDUNDANT RSB PRELUDE IMPORTS (Amendment A violation)

## VIOLATION DETECTED 🚨
Multiple module files contain redundant `use rsb::prelude::*` imports:

```rust
// src/prontodb/config.rs:4
use rsb::prelude::*;

// src/prontodb/core.rs:4  
use rsb::prelude::*;

// src/prontodb/utils.rs:4
use rsb::prelude::*;

// src/prontodb/handlers.rs:4
use rsb::prelude::*;
```

## CANONICAL RSB PATTERN 📚
From **Amendment A** of RSB Architecture (September 7, 2025):

> #### ✅ **Recommended Pattern**
> ```rust
> // main.rs - Single RSB entry point
> use rsb::prelude::*;
> 
> // lib modules - inherit RSB functionality via crate imports
> // src/myapp/config.rs
> pub fn do_load_config() -> String { 
>     // RSB macros available via main.rs prelude
>     param!("CONFIG_PATH", default: "config.toml")
> }
> ```

> #### ❌ **Anti-Pattern: Multiple RSB Imports**
> ```rust
> // main.rs
> use rsb::prelude::*;
> 
> // config.rs - REDUNDANT  
> use rsb::prelude::*;  
> ```

## CORRECTIVE ACTION ⚡

**Remove redundant RSB prelude imports from module files:**

1. **config.rs**: Remove `use rsb::prelude::*;` line 4
2. **core.rs**: Remove `use rsb::prelude::*;` line 4  
3. **utils.rs**: Remove `use rsb::prelude::*;` line 4
4. **handlers.rs**: Remove `use rsb::prelude::*;` line 4

**Keep only in main.rs** as the single RSB gateway:
```rust
// main.rs - SINGLE ENTRY POINT
use rsb::prelude::*;
```

**Test files remain unchanged** - they are acceptable exceptions per Amendment A.

## REFERENCE 📖
- **RSB Architecture Framework - Amendment A**: /home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md (lines 909-974)
- **Pattern**: Single-entry-point RSB imports via main.rs
- **Principle**: Module inheritance through Rust's standard module system
- **Benefit**: Reduces import noise, cleaner codebase, single source of truth

---
🦊 **TERRITORIAL ENFORCEMENT**: Amendment A is now CANONICAL LAW in RUSTLAND
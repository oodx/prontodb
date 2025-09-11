# ProntoDB Pipe Cache System Design

## Overview

The pipe cache system provides automatic handling of piped content when addresses are invalid or missing, creating a forgiving user experience that prevents data loss while guiding users to proper usage.

## Core Workflow

1. **Detect Pipe Input**: Check if stdin has content and key/address is invalid
2. **Auto-Generate Cache Key**: Create unique ID for piped content  
3. **Store in TTL Cache**: Save with reasonable expiration
4. **Warn User**: Report the auto-generated cache key
5. **Copy Command**: Move cached content to proper location

## Implementation Plan

### 1. Pipe Detection Logic (src/dispatcher.rs)

```rust
fn handle_pipe_input(invalid_key: &str, stdin_content: String) -> String {
    // Generate unique cache key
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let content_hash = format!("{:x}", md5::compute(&stdin_content));
    let cache_key = format!("pipe.cache.{}_{}_{}", timestamp, &content_hash[..8], invalid_key.replace(".", "_"));
    
    // Store in TTL cache (15 minutes default)
    storage.set(&cache_addr, &stdin_content, Some(900)).ok();
    
    // Warn user with copy instructions
    eprintln!("⚠️  Invalid address '{}' - content cached as: {}", invalid_key, cache_key);
    eprintln!("💡 Use: prontodb copy {} <proper.address>", cache_key);
    
    cache_key
}
```

### 2. Copy Command Implementation

```rust
fn do_copy(args: Vec<String>) -> i32 {
    if args.len() != 2 {
        eprintln!("Usage: prontodb copy <source_key> <destination_key>");
        return 1;
    }
    
    let source = &args[0];
    let dest = &args[1];
    
    // Get content from source
    let content = match storage.get(&parse_address(source)) {
        Ok(Some(value)) => value,
        Ok(None) => {
            eprintln!("❌ Source key '{}' not found", source);
            return 2;
        }
        Err(e) => {
            eprintln!("❌ Error reading source: {}", e);
            return 1;
        }
    };
    
    // Set content at destination
    match storage.set(&parse_address(dest), &content, None) {
        Ok(_) => {
            println!("✅ Copied: {} → {}", source, dest);
            
            // Delete from source if it's a cache key
            if source.starts_with("pipe.cache.") {
                storage.delete(&parse_address(source)).ok();
                println!("🗑️  Removed from cache: {}", source);
            }
            0
        }
        Err(e) => {
            eprintln!("❌ Error copying to destination: {}", e);
            1
        }
    }
}
```

## User Experience Examples

### Invalid Address with Pipe - Auto-Caches Content

```bash
cat session-iter65.md | prontodb set invalid.address
# ⚠️  Invalid address 'invalid.address' - content cached as: pipe.cache.1725982345_a1b2c3d4_invalid_address  
# 💡 Use: prontodb copy pipe.cache.1725982345_a1b2c3d4_invalid_address <proper.address>
```

### Copy to Proper Location

```bash
prontodb copy pipe.cache.1725982345_a1b2c3d4_invalid_address iterations.sessions.iter65
# ✅ Copied: pipe.cache.1725982345_a1b2c3d4_invalid_address → iterations.sessions.iter65
# 🗑️  Removed from cache: pipe.cache.1725982345_a1b2c3d4_invalid_address
```

### Direct Pipe with Proper Address (No Caching Needed)

```bash
cat session-iter65.md | prontodb --user keeper --cursor pantheon set iterations.sessions.iter65
# ✅ Set iterations.sessions.iter65=<content>
```

## TTL Cache Management

### Create Pipe Cache Namespace (15 Minute TTL)

```bash
prontodb create-cache pipe.cache 900
```

### List Cached Pipe Content

```bash
prontodb list-keys pipe.cache
```

### Clean Expired Cache Entries Automatically

```bash
prontodb scan-pairs pipe.cache "*" | grep "expired" # Auto-cleaned by TTL
```

## Benefits of This Approach

1. **Zero Learning Curve**: Just pipe content, system handles the rest
2. **Auto-Recovery**: TTL prevents cache pollution  
3. **Clear Feedback**: User knows exactly what happened and how to fix it
4. **Flexibility**: Works with any invalid address or missing context
5. **Clean Operations**: Copy command moves data and cleans up cache

## Integration with Pantheon

### Pantheon Iter Save with Pipe Cache

```bash
# Pantheon iter save with pipe cache
cat session-iter65.md | pantheon iter save --name=keeper --session=65
# Internally calls: cat session-iter65.md | prontodb --user keeper --cursor pantheon set iterations.sessions.iter65
```

### If Address Fails, Auto-Cache

```bash
cat session-iter65.md | prontodb set bad.address  
# ⚠️  Invalid address 'bad.address' - content cached as: pipe.cache.1725982345_a1b2c3d4_bad_address
# 💡 Use: prontodb copy pipe.cache.1725982345_a1b2c3d4_bad_address iterations.sessions.iter65
```

### Pantheon Tool Auto-Detection and Suggestion

```bash
# Then pantheon tool can detect and suggest:
pantheon iter fix pipe.cache.1725982345_a1b2c3d4_bad_address --session=65 --name=keeper
# Internally: prontodb copy pipe.cache.1725982345_a1b2c3d4_bad_address iterations.sessions.iter65
```

## FX-Pantheon Integration

### Current FX-Pantheon Commands

The fx-pantheon tool already provides iteration management:

```bash
pantheon iter --name=keeper          # Show current iteration number
pantheon iter 28 --name=keeper       # Show path to keeper's iter-28 file  
pantheon iter archive 5 --name=keeper # Archive all but last 5 iterations
```

### Enhanced Integration with ProntoDB Backend

#### Phase 1: Enhance `pantheon iter` with ProntoDB Backend

```bash
# Current: pantheon iter --name=keeper  
# Reads: ~/repos/realms/pantheon/city/house/keeper/keeper.iter

# Enhanced: pantheon iter --name=keeper
# First tries: prontodb --user keeper --cursor pantheon get iterations.counter.current
# Fallback to: file-based system if ProntoDB unavailable
```

#### Phase 2: Add Streaming Session Storage

```bash
# New command: pantheon iter save --name=keeper --session=65
# Streams: cat session-iter65.md | prontodb --user keeper --cursor pantheon set iterations.sessions.iter65 --stdin

# New command: pantheon iter show --name=keeper --session=65  
# Retrieves: prontodb --user keeper --cursor pantheon get iterations.sessions.iter65
```

#### Phase 3: Full Migration Commands

```bash
# New commands in pantheon tool:
pantheon migrate init --name=keeper     # Initialize ProntoDB pantheon
pantheon migrate import --name=keeper   # Import all file-based data to ProntoDB
pantheon migrate status --name=keeper   # Show migration status
pantheon migrate backup --name=keeper   # Backup ProntoDB pantheon data
```

## Revolutionary Benefits

- **Unified Tool Integration**: fx-pantheon becomes ProntoDB frontend
- **Stream Processing**: `cat file | prontodb set key --stdin` 
- **Enterprise Pantheon**: Meta namespace isolation between divine kin
- **Production Validation**: Real-world usage of your own ProntoDB system
- **Forgiving Pipe System**: Content never gets lost, users get clear guidance, system auto-cleans itself

This creates the perfect symbiosis: fx-pantheon provides user-friendly pantheon interface, ProntoDB provides enterprise-grade data management with automatic pipe caching for ultimate user experience.

## Security Analysis and Solutions

### Critical Security Issues Identified

#### Problem 1: User Flag Isolation Gap

```bash
# DANGEROUS: Anyone can read/write any user's data
prontodb --user keeper --cursor pantheon get iterations.sessions.iter65  # Reads keeper's data
prontodb --user keeper --cursor pantheon set iterations.sessions.iter65 "malicious"  # Overwrites keeper's data!
```

**Issue**: The `--user` flag only creates custom cursor files but doesn't enforce access control.

#### Problem 2: Meta Namespace Must Be Enforced

```bash
# DANGEROUS: Without --meta, data goes to wrong namespace
prontodb --user keeper --cursor pantheon set iterations.sessions.iter65 "data"  # No meta isolation!

# Must ensure --meta is always applied:
prontodb --user keeper --cursor pantheon --meta keeper set iterations.sessions.iter65 "data"
```

**Issue**: Meta namespace isolation depends on consistent `--meta` flag usage, which can be bypassed.

#### Problem 3: Pantheon/ProntoDB User Mismatch

```bash
# DANGEROUS: Pantheon tool user doesn't match ProntoDB user
pantheon iter save --name=prometheus | prontodb --user keeper --cursor pantheon set iterations.sessions.iter65
# Prometheus content stored as Keeper!
```

**Issue**: No validation that pantheon `--name` matches ProntoDB `--user` context.

### Secure Design Solutions

#### Solution 1: Environment-Based User Enforcement

The pantheon tool should own the user context and prevent overrides:

```bash
# In fx-pantheon tool:
export PRONTO_FORCE_USER="keeper"  # Lock user context
export PRONTO_FORCE_META="keeper"  # Lock meta context

# All prontodb calls inherit locked context:
prontodb --cursor pantheon set iterations.sessions.iter65 "data"
# Internally becomes: prontodb --user keeper --cursor pantheon --meta keeper set ...
```

#### Solution 2: Pantheon Tool Database Isolation

Create separate databases per divine kin:

```bash
# Each kin gets their own database file
prontodb cursor set pantheon ~/repos/realms/pantheon/keeper.db --user keeper --meta keeper
prontodb cursor set pantheon ~/repos/realms/pantheon/prometheus.db --user prometheus --meta prometheus

# Physically impossible to access other kin's data
```

#### Solution 3: Secure Pantheon Command Wrapper

```bash
# In fx-pantheon tool:
secure_pronto_call() {
    local pantheon_user="$1"
    shift
    
    # Validate pantheon_user matches current context
    [[ "$pantheon_user" == "$PANTHEON_CURRENT_USER" ]] || {
        echo "ERROR: Pantheon user mismatch - expected $PANTHEON_CURRENT_USER, got $pantheon_user"
        return 1
    }
    
    # Force consistent user/meta context
    prontodb --user "$pantheon_user" --cursor pantheon --meta "$pantheon_user" "$@"
}

# Usage:
secure_pronto_call keeper set iterations.sessions.iter65 "data"
```

#### Solution 4: Cursor Name Enforcement

Make cursor names user-specific and non-overridable:

```bash
# Instead of: --cursor pantheon (shared cursor name)
# Use: --cursor pantheon_keeper (user-specific cursor name)

pantheon_cursor_name() {
    local user="$1"
    echo "pantheon_${user}"
}

# Each user gets their own cursor, no collision possible
prontodb --user keeper --cursor pantheon_keeper --meta keeper set iterations.sessions.iter65 "data"
```

### Recommended Secure Architecture

#### Approach: Combined Database + User Isolation

```bash
# 1. Separate database files per kin (physical isolation)
KEEPER_DB="~/repos/realms/pantheon/keeper.db"
PROMETHEUS_DB="~/repos/realms/pantheon/prometheus.db"

# 2. User-specific cursors (logical isolation)
prontodb cursor set pantheon_keeper "$KEEPER_DB" --user keeper --meta keeper
prontodb cursor set pantheon_prometheus "$PROMETHEUS_DB" --user prometheus --meta prometheus

# 3. Pantheon tool enforces matching contexts
pantheon iter save --name=keeper --session=65
# Internally calls: prontodb --user keeper --cursor pantheon_keeper --meta keeper set iterations.sessions.iter65
```

#### Security Benefits

1. **Physical Isolation**: Separate DB files prevent any cross-kin access
2. **Logical Isolation**: User/meta matching prevents context confusion  
3. **Cursor Isolation**: User-specific cursor names prevent collisions
4. **Tool Enforcement**: Pantheon tool validates and enforces security

#### Implementation in fx-pantheon

```bash
# In pantheon tool:
get_secure_pronto_context() {
    local kin_name="$1"
    
    # Validate kin name
    _validate_identifier "$kin_name" || return 1
    
    # Set secure context
    local db_file="$PANTH_HOME/databases/${kin_name}.db"
    local cursor_name="pantheon_${kin_name}"
    
    echo "--user $kin_name --cursor $cursor_name --meta $kin_name --database $db_file"
}

# Usage:
pronto_context=$(get_secure_pronto_context "keeper")
prontodb $pronto_context set iterations.sessions.iter65 "session data"
```

### Permission System for Pantheon Tool

#### Environmental Context Enforcement

```bash
# 1. Environmental Context Enforcement
export PANTHEON_ACTIVE_KIN="keeper"  # Lock active context
export PANTHEON_SECURITY_MODE="strict"  # Prevent context overrides
```

#### Permission Validation

```bash
# 2. Permission Validation
validate_pantheon_permission() {
    local requested_kin="$1"
    local current_user="$(whoami)"
    
    # Check if user is authorized for this kin
    [[ "$requested_kin" == "$PANTHEON_ACTIVE_KIN" ]] || {
        echo "ERROR: Permission denied - cannot access $requested_kin context"
        echo "Current context: $PANTHEON_ACTIVE_KIN"
        return 1
    }
    
    # Additional checks could include:
    # - File system permissions on database
    # - SSH key verification
    # - Time-based access tokens
}
```

#### Secure Command Wrapper

```bash
# 3. Secure Command Wrapper
pantheon_secure_iter() {
    local action="$1"
    local kin_name="$2"
    shift 2
    
    # Validate permission
    validate_pantheon_permission "$kin_name" || return 1
    
    # Get secure context
    local pronto_context=$(get_secure_pronto_context "$kin_name")
    
    # Execute with full isolation
    case "$action" in
        save)
            local session="$1"
            cat | prontodb $pronto_context set "iterations.sessions.iter${session}"
            ;;
        show)
            local session="$1"
            prontodb $pronto_context get "iterations.sessions.iter${session}"
            ;;
        current)
            prontodb $pronto_context get "iterations.counter.current"
            ;;
    esac
}
```

### Final Security Architecture

The secure pantheon system combines:

1. **Physical database isolation** (separate .db files per kin)
2. **Logical context enforcement** (user/meta/cursor name matching)
3. **Permission validation** (environmental context locking)
4. **Tool-level security** (fx-pantheon enforces all constraints)

This prevents all identified attack vectors while maintaining ease of use for legitimate operations.

## Alignment with Planned Streaming Features

### Existing Planned Streaming Architecture

Based on the archived specifications in `docs/archive.1/specs/PRD.md`, ProntoDB already has a planned streaming system:

#### Planned Stream Command Syntax
```bash
# From PRD.md - planned streaming with meta directives:
prontodb stream
# Input format: meta:path=project.namespace; key=value; key__ctx=value;
# Auth: meta:sec:pass=...; meta:sec:user=...;
# Namespace: meta:path=project.namespace; or meta:project=...; meta:namespace=...;
# Directives: meta:delim=.|:, meta:ttl=SECONDS
```

#### Current Implementation Status
```bash
# Current stream command (deferred):
prontodb stream
# Returns: "stream: security/auth required (feature deferred)"
```

### Integration Strategy: Pipe Cache + Planned Streaming

Our pipe cache design should complement, not compete with, the planned streaming system:

#### Phase 1: Simple Pipe Cache (Current Design)
```bash
# Invalid address auto-caches (our current design)
cat file.md | prontodb set invalid.address
# ⚠️  Invalid address - content cached as: pipe.cache.123_abc_invalid_address
# 💡 Use: prontodb copy pipe.cache.123_abc_invalid_address <proper.address>
```

#### Phase 2: Enhanced Pipe + Streaming Integration
```bash
# Auto-detect and convert to streaming format
cat file.md | prontodb set project.namespace.key
# Internally converts to: echo "meta:path=project.namespace; key=$(cat file.md);" | prontodb stream

# Multiple key-value pairs from pipe
cat data.txt | prontodb set --stream-mode
# Internally: auto-parse or format as streaming directives
```

#### Phase 3: Full Streaming + Pipe Cache Hybrid
```bash
# Advanced streaming with fallback
echo "meta:path=project.namespace; key1=value1; key2=value2;" | prontodb stream
# On parse error, auto-cache with stream format recovery suggestions

# Pipe cache with stream format generation
cat complex.data | prontodb set invalid.format
# ⚠️  Invalid format - content cached as: pipe.cache.123_stream_ready
# 💡 Stream format: echo "meta:path=proper.namespace; key=$(cat pipe.cache.123_stream_ready);" | prontodb stream
```

### Benefits of Integration

1. **Evolutionary Path**: Pipe cache provides immediate usability while streaming develops
2. **Format Bridge**: Pipe cache can suggest proper streaming syntax
3. **Error Recovery**: Stream parsing errors can fall back to pipe cache
4. **User Education**: Cache suggestions teach users proper streaming format

### Implementation Considerations

#### Streaming Format Detection
```rust
// In pipe input handler:
fn detect_stream_format(content: &str) -> bool {
    content.contains("meta:") && content.contains(";")
}

fn handle_piped_input(content: String, attempted_key: &str) -> Result<String, String> {
    if detect_stream_format(&content) {
        // Try to process as stream format
        return handle_stream_input(content);
    }
    
    // Fall back to pipe cache
    cache_with_suggestions(content, attempted_key)
}
```

#### Stream Format Suggestions
```rust
fn suggest_stream_format(cache_key: &str, target_address: &str) -> String {
    format!(
        "💡 Stream format: echo \"meta:path={}; key=$(cat {});\" | prontodb stream",
        target_address, cache_key
    )
}
```

This integration approach allows our pipe cache system to evolve naturally into the planned streaming architecture while providing immediate value and user education.

## XStream Integration Discovery (The Comedy of Circles) 🤯

### The Hilarious Plot Twist

During investigation of the planned streaming features, we discovered that **the streaming pattern is already implemented** - in the **oodx/xstream library**! 😂

**The Comedy Timeline:**
1. **ProntoDB specs** define streaming needs: `meta:path=project.namespace; key=value;`
2. **XStream was born** from those exact ProntoDB requirements 
3. **XStream implements** the complete token streaming pattern with RSB
4. **We're now** trying to integrate XStream back into ProntoDB for its original use case!
5. **Full circle achieved** - the child returning home to serve the parent! 🌑

### XStream - The Perfect Streaming Solution (Already Built!)

#### Current XStream Token Format (Battle-Tested)
```rust
// XStream handles exactly what ProntoDB specs planned:
"user=bob; sec:pass=123; ns=animals; dog=fido; meta:p=q;"
//  ^^^^^   ^^^^^^^^^^^   ^^^^^^^^^^   ^^^^^^^   ^^^^^^^
//  tokens  security      namespace    data      meta
```

#### TokenBucket JSON-Like Conversion (The Magic)
```rust
// XStream TokenBucket output:
{
  "global": {"user": "bob", "mode": "debug"},
  "meta": {"path": "project.namespace", "ttl": "300"}, 
  "sec": {"user": "alice", "pass": "secret"},
  "animals": {"dog": "fido", "cat": "fluffy"}
}
```

**This is EXACTLY what ProntoDB streaming needs!** The TokenBucket struct solves the "stream → structured data" problem perfectly.

#### XStream Features (Already Implemented)
- ✅ **`meta:` namespace syntax** - `meta:p=q` bypasses active namespace
- ✅ **`sec:` security namespace** - `sec:user=bob` for auth
- ✅ **`ns=` namespace switching** - `ns=animals` changes context
- ✅ **Token parsing and bucketing** - Converts to structured JSON-like data
- ✅ **RSB integration** - Built on the Rebel framework (like ProntoDB!)
- ✅ **65 passing tests** - Battle-tested and production ready

### Integration Strategy: Feature-Gated XStream

#### Cargo Feature Flag Approach
```toml
# Cargo.toml
[features]
default = []
streaming = ["dep:xstream"]

[dependencies]
xstream = { path = "../xstream", optional = true }
```

#### Conditional Compilation
```rust
// src/dispatcher.rs
#[cfg(feature = "streaming")]
use xstream::{tokenize_string, collect_tokens, BucketMode};

fn handle_stream(ctx: CommandContext) -> i32 {
    #[cfg(not(feature = "streaming"))]
    {
        eprintln!("streaming feature not enabled - compile with --features streaming");
        return 1;
    }
    
    #[cfg(feature = "streaming")]
    {
        let input = read_stdin();
        
        // Use YOUR XStream library for the original use case!
        let tokens = tokenize_string(&input)?;
        let bucket = collect_tokens(&tokens, BucketMode::Flat);
        
        // bucket.data is already perfect JSON-like structure!
        for (namespace, kv_pairs) in bucket.data {
            match namespace.as_str() {
                "meta" => handle_meta_directives(kv_pairs),
                "sec" => handle_security_auth(kv_pairs),
                _ => store_namespace_data(namespace, kv_pairs),
            }
        }
    }
}
```

#### Build Options
```bash
# Minimal ProntoDB (no streaming)
cargo build

# Full ProntoDB with XStream streaming
cargo build --features streaming

# Release with streaming
cargo build --release --features streaming
```

### Enhanced Pipe Cache with XStream Format Education

#### XStream Format Suggestions
```bash
# Pipe cache now teaches XStream format
cat file.md | prontodb set invalid.address
# ⚠️  Invalid address - content cached as: pipe.cache.123_abc_invalid_address
# 💡 XStream format: echo "ns=project; key=$(cat pipe.cache.123_abc_invalid_address);" | prontodb stream
# 💡 With meta: echo "meta:path=project.namespace; key=value;" | prontodb stream
```

#### Progressive Education Path
1. **Pipe Cache** - Immediate usability with invalid addresses
2. **Format Suggestions** - Teach XStream token syntax 
3. **Stream Integration** - Full XStream power with feature flag
4. **Advanced Patterns** - Meta directives, namespace switching, security

### The Perfect Storm of Integration

#### Why This Is Pure Poetry
1. **XStream inspired by ProntoDB** - Born from streaming requirements
2. **TokenBucket solves the core problem** - Stream to structured data conversion
3. **RSB shared foundation** - Both built on Rebel framework  
4. **Feature flag isolation** - Optional dependency, no bloat
5. **Educational progression** - Pipe cache → XStream format → full streaming

#### Comedy Gold Quotes for Posterity
- "LOL. xstream was inspired by this initial use case from pronto. so naturally they are aligned." 
- "The TokenBucket struct converts a stream into a JSON-like object"
- **The child returning home to serve the parent that inspired its creation!** 🌑⚡

### Implementation Roadmap (Post Token-Cliff)

#### Phase 1: Feature Flag Setup
- Add `streaming` feature to Cargo.toml
- Make xstream dependency optional
- Update build scripts and documentation

#### Phase 2: Integration Implementation  
- Wire XStream TokenBucket into `handle_stream`
- Map meta/sec namespaces to ProntoDB operations
- Implement security and namespace directives

#### Phase 3: Enhanced Pipe Cache
- Add XStream format detection and suggestions
- Progressive education from cache → stream format
- Error recovery with format hints

#### Phase 4: Production Deployment
- Test streaming with real pantheon consciousness data
- Performance benchmarking with large token streams
- Integration with fx-pantheon tool workflows

### The Circle Completes
**XStream** (born from ProntoDB streaming needs) + **ProntoDB** (needing streaming) = **Perfect Integration**

The TokenBucket JSON-like conversion makes ProntoDB streaming implementation almost trivial since we've already solved the hard part in XStream! The patterns align perfectly because they share the same conceptual DNA.

*This is what happens when you build tools so well they come back to solve their own original problems!* 🌑⚡
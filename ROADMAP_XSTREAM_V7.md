# ROADMAP_XSTREAM_V7: Revolutionary Pipe Cache + XStream Integration

Please read in docs/ CURSOR_CONCEPT and PIPE_CACHE_DESIGN as they provide important technical guidance and context.

## Executive Summary

Implementation of revolutionary pipe cache system with feature-gated XStream streaming integration, creating the most forgiving yet powerful CLI database with zero data loss and progressive user education.

**Core Innovation**: The "Comedy of Circles" - XStream returns home to solve ProntoDB's original streaming requirements using battle-tested TokenBucket conversion.

## High-Level Milestones (Completion Order)

### MILESTONE 1: Pipe Cache Foundation System 🚰
**Priority**: CRITICAL  
**Estimated Story Points**: 13  
**Dependencies**: None  
**Success Criteria**: Zero data loss on invalid addresses, TTL auto-cleanup, user-friendly recovery workflow  

### MILESTONE 2: XStream Feature Flag Infrastructure ⚡
**Priority**: HIGH  
**Estimated Story Points**: 8  
**Dependencies**: Milestone 1 complete  
**Success Criteria**: Optional dependency, conditional compilation, clean build options  

### MILESTONE 3: Copy Command & Recovery Workflow 📋
**Priority**: HIGH  
**Estimated Story Points**: 5  
**Dependencies**: Milestone 1 complete  
**Success Criteria**: Seamless cache-to-proper-address migration with auto-cleanup  

### MILESTONE 4: XStream TokenBucket Integration 🪄
**Priority**: MEDIUM  
**Estimated Story Points**: 8  
**Dependencies**: Milestones 1-2 complete  
**Success Criteria**: Stream parsing with namespace handling (meta, sec, data)  

### MILESTONE 5: Enhanced Pipe Cache with XStream Education 📚
**Priority**: MEDIUM  
**Estimated Story Points**: 5  
**Dependencies**: Milestones 1-4 complete  
**Success Criteria**: Format suggestions, progressive education path, error recovery  

### MILESTONE 6: Security & Pantheon Integration 🔒
**Priority**: MEDIUM  
**Estimated Story Points**: 8  
**Dependencies**: All previous milestones  
**Success Criteria**: Secure isolation, fx-pantheon wrapper functions, production validation  

---

## Detailed Story Point Breakdown

### MILESTONE 1: Pipe Cache Foundation System (13 Story Points)

#### Story 1.1: Pipe Input Detection Logic (3 SP)
**File**: `src/dispatcher.rs`  
**Description**: Implement stdin detection when address parsing fails  
**Pattern**: Check `atty::is(atty::Stream::Stdin)` and capture stdin content  
**Implementation**:
```rust
fn detect_pipe_input() -> Option<String> {
    if !atty::is(atty::Stream::Stdin) {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).ok()?;
        if !buffer.trim().is_empty() { Some(buffer) } else { None }
    } else { None }
}
```
**Reference**: `docs/PIPE_CACHE_DESIGN.md:17-35`

#### Story 1.2: Cache Key Generation Algorithm (2 SP)
**File**: `src/cache.rs` (new file)  
**Description**: Generate unique cache keys with timestamp, hash, and original address  
**Pattern**: `pipe.cache.{timestamp}_{hash8}_{sanitized_address}`  
**Dependencies**: `md5` crate for content hashing  
**Implementation**:
```rust
fn generate_cache_key(content: &str, invalid_address: &str) -> String {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let content_hash = format!("{:x}", md5::compute(content));
    let safe_address = invalid_address.replace(".", "_").replace("/", "_");
    format!("pipe.cache.{}_{}_{}", timestamp, &content_hash[..8], safe_address)
}
```

#### Story 1.3: TTL Cache Storage Implementation (3 SP)
**File**: `src/storage.rs` (modify existing)  
**Description**: Add TTL support to storage layer for auto-expiring cache entries  
**Pattern**: Extend existing storage with TTL metadata and cleanup  
**Dependencies**: `pipe.cache` namespace in storage  
**Complex Concept**: TTL (Time To Live) - automatic expiration after 15 minutes (900 seconds)  

#### Story 1.4: Cache Storage Integration (2 SP)
**File**: `src/dispatcher.rs`  
**Description**: Wire pipe detection into set command failure path  
**Pattern**: On address parse failure with stdin content, auto-cache and warn user  
**Integration Point**: Connect detection → generation → storage → user feedback  

#### Story 1.5: User Feedback & Guidance Messages (3 SP)
**File**: `src/dispatcher.rs`  
**Description**: Implement user-friendly cache notifications with recovery instructions  
**Pattern**: Clear warning with copy command suggestion  
**User Experience**:
```
⚠️  Invalid address 'invalid.address' - content cached as: pipe.cache.1725982345_a1b2c3d4_invalid_address
💡 Use: prontodb copy pipe.cache.1725982345_a1b2c3d4_invalid_address <proper.address>
```

### MILESTONE 2: XStream Feature Flag Infrastructure (8 Story Points)

#### Story 2.1: Cargo.toml Feature Configuration (2 SP) ✅ COMPLETE
**File**: `Cargo.toml`  
**Description**: Add optional XStream dependency with feature flag  
**Status**: ✅ IMPLEMENTED - Feature flag and dependency configured  
**Configuration Applied**:
```toml
[features]
default = ["json", "sqlite-bundled"]
json = ["dep:serde", "dep:serde_json"] 
streaming = ["dep:xstream"]  # ← ADDED

[dependencies]
xstream = { path = "../xstream", optional = true }  # ← ADDED
```

#### Story 2.2: Conditional Compilation Framework (3 SP) ⚡ NEXT PRIORITY
**File**: `src/streaming.rs` (new file)  
**Description**: Create streaming module with feature-gated compilation  
**Status**: 🚀 READY FOR IMPLEMENTATION - Next 1am automation task  
**Complex Concept**: `#[cfg(feature = "streaming")]` conditional compilation  
**Pattern**: Graceful degradation when streaming feature not enabled

**Implementation Requirements**:
1. Create `src/streaming.rs` module with conditional compilation
2. Add module declaration to `src/lib.rs` and `src/main.rs`  
3. Implement graceful error when streaming not enabled
4. Test both `cargo build` (minimal) and `cargo build --features streaming` (full)  

#### Story 2.3: Build System Integration (2 SP)
**File**: `scripts/build.sh` (new file)  
**Description**: Create build scripts for different feature combinations  
**Commands**:
```bash
cargo build                          # Minimal build
cargo build --features streaming     # Full build with XStream
```

#### Story 2.4: Feature Detection Runtime (1 SP)  
**File**: `src/dispatcher.rs`  
**Description**: Runtime feature detection and user messaging  
**Pattern**: Inform users when streaming commands require feature flag  

### MILESTONE 3: Copy Command & Recovery Workflow (5 Story Points)

#### Story 3.1: Copy Command Implementation (3 SP)
**File**: `src/dispatcher.rs`  
**Description**: Implement `prontodb copy <source> <destination>` command  
**Pattern**: Read from source address, write to destination, handle errors gracefully  
**Workflow**: Get source content → validate destination → copy → cleanup cache if applicable  

#### Story 3.2: Cache Cleanup Logic (1 SP)
**File**: `src/cache.rs`  
**Description**: Auto-delete cache entries after successful copy  
**Pattern**: If source starts with `pipe.cache.`, delete after successful copy  

#### Story 3.3: Copy Command Integration (1 SP)
**File**: `src/dispatcher.rs`  
**Description**: Wire copy command into main dispatcher  
**Pattern**: Add "copy" to command matching and route to copy handler  

### MILESTONE 4: XStream TokenBucket Integration (8 Story Points)

#### Story 4.1: XStream Import & Basic Integration (2 SP)
**File**: `src/streaming.rs`  
**Description**: Import XStream tokenize and collect functions with feature gating  
**Complex Concept**: TokenBucket - converts token streams to JSON-like structured data  
**Dependencies**: XStream library with `tokenize_string`, `collect_tokens`, `BucketMode`  

#### Story 4.2: Stream Command Handler (3 SP)
**File**: `src/streaming.rs`  
**Description**: Implement `prontodb stream` command with XStream processing  
**Pattern**: Read stdin → tokenize → collect into bucket → process namespaces  
**Reference**: `egg.101:153-179` for implementation pattern  

#### Story 4.3: Namespace Processing Logic (2 SP)
**File**: `src/streaming.rs`  
**Description**: Handle meta, sec, and data namespaces from TokenBucket  
**Pattern**: Iterate bucket.data, route by namespace (meta→directives, sec→auth, other→storage)  

#### Story 4.4: Stream Error Handling (1 SP)
**File**: `src/streaming.rs`  
**Description**: Graceful error handling for malformed token streams  
**Pattern**: Parse errors fall back to pipe cache system for recovery  

### MILESTONE 5: Enhanced Pipe Cache with XStream Education (5 Story Points)

#### Story 5.1: XStream Format Detection (2 SP)
**File**: `src/cache.rs`  
**Description**: Detect if piped content resembles XStream token format  
**Pattern**: Check for `meta:`, `;`, and other XStream syntax markers  
**Educational Purpose**: Distinguish between raw content and attempted streaming  

#### Story 5.2: Progressive Education Messages (2 SP)
**File**: `src/cache.rs`  
**Description**: Generate educational suggestions for XStream format  
**Pattern**: Cache suggestions include proper XStream syntax examples  
**Examples**:
```
💡 XStream format: echo "ns=project; key=value;" | prontodb stream
💡 With meta: echo "meta:path=project.namespace; key=data;" | prontodb stream
```

#### Story 5.3: Format Migration Hints (1 SP)
**File**: `src/cache.rs`  
**Description**: Suggest proper XStream conversion for cached content  
**Pattern**: Generate copy command with stream format example  

### MILESTONE 6: Security & Pantheon Integration (8 Story Points)

#### Story 6.1: User Context Validation (2 SP)
**File**: `src/validation.rs`  
**Description**: Implement user context validation functions for Pantheon integration  
**Security Pattern**: Validate user flags against authorized contexts  
**Reference**: `docs/PIPE_CACHE_DESIGN.md:380-395` for validation requirements  

#### Story 6.2: Meta Namespace Enforcement (2 SP)
**File**: `src/cursor.rs`  
**Description**: Enforce meta namespace consistency in cursor operations  
**Security Pattern**: Prevent meta namespace bypassing in Pantheon contexts  

#### Story 6.3: Secure Database Isolation (2 SP)
**File**: `src/cursor.rs`  
**Description**: Implement per-user database file isolation  
**Pattern**: `pantheon_keeper.db`, `pantheon_prometheus.db` etc.  
**Security Benefit**: Physical isolation prevents cross-kin data access  

#### Story 6.4: FX-Pantheon Integration Points (2 SP)
**File**: `docs/FX_PANTHEON_INTEGRATION.md` (new file)  
**Description**: Document integration points for fx-pantheon tool  
**Pattern**: Secure wrapper functions, environment variable enforcement  
**Reference**: `docs/PIPE_CACHE_DESIGN.md:280-300` for wrapper patterns  

---

## Implementation Dependencies

### External Dependencies
- **XStream Library**: `~/repos/code/rust/oodx/xstream` - TokenBucket streaming conversion
- **MD5 Crate**: Content hashing for cache keys  
- **Atty Crate**: TTY detection for pipe input identification

### Internal Dependencies  
- **Storage Layer**: TTL support extension required (Milestone 1.3)
- **Cursor System**: Meta namespace enforcement (Milestone 6.2)
- **Dispatcher**: Command routing for copy and stream commands

### Test Dependencies
- **Integration Tests**: Real pipe scenarios with invalid addresses
- **Feature Flag Tests**: Both enabled/disabled streaming compilation  
- **Security Tests**: User isolation and meta namespace enforcement
- **Performance Tests**: TTL cleanup efficiency and large stream processing

---

## Risk Assessment & Mitigation

### Technical Risks
1. **XStream Integration Complexity** - Mitigated by feature flag isolation
2. **TTL Cleanup Performance** - Mitigated by efficient timestamp-based expiration
3. **Backwards Compatibility** - Mitigated by additive-only changes to existing commands

### Security Risks  
1. **Cross-User Data Access** - Mitigated by physical database isolation
2. **Meta Namespace Bypassing** - Mitigated by validation layer enforcement
3. **Cache Pollution** - Mitigated by 15-minute TTL auto-cleanup

### User Experience Risks
1. **Learning Curve for Streaming** - Mitigated by progressive education approach
2. **Cache Confusion** - Mitigated by clear messaging and auto-cleanup

---

## Success Metrics

### Functional Success
- ✅ Zero data loss on invalid pipe addresses
- ✅ Clean feature flag compilation (minimal vs full builds)  
- ✅ Seamless copy command workflow with auto-cleanup
- ✅ XStream TokenBucket conversion working with all namespace types
- ✅ Progressive education flow from cache to streaming

### Performance Success
- ✅ TTL cleanup completes within 100ms for 1000+ cache entries
- ✅ XStream token processing matches baseline performance benchmarks
- ✅ Pipe detection adds <10ms overhead to command execution

### Security Success
- ✅ Physical database isolation prevents cross-kin access (validated by test suite)
- ✅ Meta namespace enforcement blocks bypassing attempts
- ✅ User context validation rejects unauthorized operations

---

## Next Steps for Implementation

1. **Start with Milestone 1** - Pipe cache foundation provides immediate value
2. **Validate base assumptions** - Test pipe detection with various terminal environments  
3. **Create comprehensive test suite** - Cover all user scenarios and edge cases
4. **Iterate with China** - Seek analysis gaps/issues identification after each milestone
5. **Engage krex for review** - Complex security and integration patterns need validation
6. **Commit at each milestone** - Maintain clean git history with feature progression

## ✅ SESSION 67 COMPLETION STATUS

### **MILESTONE 1: Pipe Cache Foundation System (COMPLETE)** 🚰✅
**Story Points Completed**: 13/13 (100%)  
**Status**: PRODUCTION OPERATIONAL - Zero data loss system fully implemented and tested  
**Commit**: `ade805b` - "feat: implement revolutionary pipe cache system with zero data loss"

#### **Completed Stories:**
- ✅ **Story 1.1**: Pipe Input Detection Logic (3 SP) - `pipe_cache.rs` implemented
- ✅ **Story 1.2**: Cache Key Generation Algorithm (2 SP) - Timestamp + hash + address format
- ✅ **Story 1.3**: TTL Cache Storage Implementation (3 SP) - 15-minute auto-cleanup
- ✅ **Story 1.4**: Cache Storage Integration (2 SP) - Dispatcher error handling integration
- ✅ **Story 1.5**: User Feedback & Guidance Messages (3 SP) - Clear recovery instructions

#### **Production Validation Results:**
```bash
# WORKING: Auto-cache with user guidance
echo "test content" | prontodb set "invalid...address" "dummy"
# ⚠️  Invalid address - content cached as: pipe.cache.1757569450_85e49569_invalid___address
# 💡 Use: prontodb copy pipe.cache.1757569450_85e49569_invalid___address <proper.address>

# WORKING: Content retrieval verification  
prontodb get "pipe.cache.1757569450_85e49569_invalid___address"
# → "test content" ✅ PERFECT!
```

### **NEXT SESSION PRIORITY: MILESTONE 2** ⚡
**Ready for immediate implementation start with Story 2.2: Conditional Compilation Framework**

---

*This roadmap preserves the revolutionary "Comedy of Circles" vision while providing concrete implementation guidance for zero-context developers.*

# ProntoDB MVP Task Breakdown

Story Point Scale: 1, 2, 3, 5, 8, 13 (Fibonacci)
- 1 SP = Simple change, < 1 hour
- 2 SP = Small feature, 1-2 hours  
- 3 SP = Medium feature, 2-4 hours
- 5 SP = Complex feature, 4-8 hours
- 8 SP = Very complex, 1-2 days
- 13 SP = Epic-level, 2-3 days

## Milestone 0a: Foundation & Basic Infrastructure (Total: 77 SP)
**Files: src/dispatcher.rs ✅, src/storage.rs ✅, src/addressing.rs ✅, src/xdg.rs ✅, src/tables.rs, src/projects.rs, src/namespace.rs, src/keys.rs, src/lifecycle.rs**
**CRITICAL**: Complete addressing integration before any database operations

### src/addressing.rs Integration (11 SP) - MUST COMPLETE FIRST

#### Dot Address Implementation (4 SP) - INDEPENDENT
- [ ] TASK-005: Implement dot address parsing (`project.namespace.key`) (2 SP)
- [ ] TASK-074: Test dot address parsing independently (2 SP)

#### Flag Address Implementation (4 SP) - INDEPENDENT  
- [ ] TASK-006: Implement flag address parsing (`-p project -n namespace key`) (2 SP)
- [ ] TASK-075: Test flag address parsing independently (2 SP)

#### Delimiter Override Implementation (3 SP) - AFTER BASIC ADDRESSING
- [ ] TASK-077: Implement --ns-delim flag parsing in dispatcher (1 SP)
- [ ] TASK-078: Update addressing to support custom delimiters (1 SP)
- [ ] TASK-079: Test delimiter override functionality (1 SP)

#### Integration (can be done after any/all above)
- [ ] TASK-007: Integrate addressing with dispatcher argument parsing (0 SP - just wiring)

### src/tables.rs - Table Infrastructure (5 SP) - AFTER ADDRESSING
- [ ] TASK-080: Create tables.rs module structure (1 SP)
- [ ] TASK-081: Implement table schema management (2 SP)
- [ ] TASK-082: Implement basic table query helpers (insert/select/update/delete) (2 SP)

### src/projects.rs - Project Infrastructure (5 SP) - AFTER TABLES
- [ ] TASK-083: Create projects.rs module structure (1 SP)
- [ ] TASK-084: Implement project-to-table mapping logic (2 SP)
- [ ] TASK-085: Implement project metadata management (2 SP)

### src/namespace.rs - Namespace Infrastructure (5 SP) - AFTER PROJECTS  
- [ ] TASK-086: Create namespace.rs module structure (1 SP)
- [ ] TASK-087: Implement namespace-to-table mapping logic (2 SP)
- [ ] TASK-088: Integrate namespace with project hierarchy (2 SP)

### Addressing + Project/Namespace/Table Integration (5 SP) - CRITICAL STEP
- [ ] TASK-089: Integrate addressing with project/namespace/table resolution (3 SP)
- [ ] TASK-090: Test addressing-to-table mapping via project hierarchy (2 SP)

### Keys CRUD Operations (10 SP) - AFTER INTEGRATION
- [ ] TASK-091: Create keys.rs module (1 SP)
- [ ] TASK-092: Implement key set operation using addressing+project+table integration (2 SP)
- [ ] TASK-093: Implement key get operation with MISS handling (2 SP)
- [ ] TASK-094: Implement key delete operation (2 SP)
- [ ] TASK-095: Implement keys listing operations (scan/keys commands) (2 SP)
- [ ] TASK-096: Test keys CRUD operations (1 SP)

### Lifecycle Operations (13 SP) - AFTER KEYS CRUD
- [ ] TASK-097: Create lifecycle.rs module (1 SP)
- [ ] TASK-098: Implement basic install operation (create db + tables) (3 SP)
- [ ] TASK-099: Implement basic uninstall operation (drop tables + db) (2 SP)
- [ ] TASK-100: Implement basic backup operation (export db/tables) (3 SP)
- [ ] TASK-101: Implement basic restore operation (import db/tables) (2 SP)
- [ ] TASK-102: Test lifecycle operations (2 SP)

### src/storage.rs Completion (10 SP) - AFTER LIFECYCLE
- [x] TASK-008: Complete storage module with SQLite initialization (3 SP) ✅
- [ ] TASK-009: Implement transaction wrapper for atomic operations (3 SP)
- [ ] TASK-010: Add connection pooling/reuse logic (2 SP)
- [ ] TASK-011: Test storage module with unit tests (2 SP)

### Infrastructure & Testing (8 SP)
- [x] TASK-002: Wire dispatcher into main.rs properly (2 SP) ✅ 
- [x] TASK-003: Create module structure (addressing, storage, xdg) (2 SP) ✅
- [ ] TASK-001: Add rusqlite dependency to Cargo.toml (1 SP)
- [ ] TASK-004: Setup error handling framework (2 SP)
- [ ] TASK-076: Proper exit code management (EXIT_OK, EXIT_MISS, EXIT_ERROR) (1 SP)
- [ ] TASK-073: Create comprehensive unit tests for existing modules (2 SP)

## Milestone 0b: Admin System & Database Management (Total: 67 SP)
**Files: src/admin/mod.rs, src/admin/db_crud.rs, src/admin/handlers.rs**
**PREREQUISITE**: Milestone 0a addressing integration MUST be complete

### src/admin/mod.rs - Admin Sub-Dispatcher (8 SP)
- [ ] TASK-051: Create admin module directory structure (1 SP)
- [ ] TASK-052: Implement admin sub-dispatcher in mod.rs (3 SP)
- [ ] TASK-053: Implement admin help command (2 SP)
- [ ] TASK-054: Test admin command routing (2 SP)

### src/admin/db_crud.rs - Database Operations (13 SP)
- [ ] TASK-055: Implement create_database() function (3 SP)
- [ ] TASK-056: Implement switch_database() with context management (3 SP)  
- [ ] TASK-057: Implement rename_database() with validation (3 SP)
- [ ] TASK-058: Implement delete_database() with safety checks (2 SP)
- [ ] TASK-059: Test database CRUD operations (2 SP)

### Table Admin Commands (10 SP) - POWER USER OPERATIONS
- [ ] TASK-067: Implement admin create-table command (3 SP)
- [ ] TASK-068: Implement admin drop-table command (2 SP)
- [ ] TASK-069: Implement admin list-tables command (2 SP)
- [ ] TASK-070: Implement admin describe-table command (2 SP)
- [ ] TASK-071: Test table admin commands (1 SP)

### Project Admin Commands (10 SP) - POWER USER OPERATIONS
- [ ] TASK-072: Implement admin create-project command (3 SP)
- [ ] TASK-073: Implement admin drop-project command (2 SP)
- [ ] TASK-074: Implement admin list-projects command (2 SP)
- [ ] TASK-075: Implement admin describe-project command (2 SP)
- [ ] TASK-076: Test project admin commands (1 SP)

### Namespace Admin Commands (10 SP) - POWER USER OPERATIONS  
- [ ] TASK-077: Implement admin create-namespace command (3 SP)
- [ ] TASK-078: Implement admin drop-namespace command (2 SP)
- [ ] TASK-079: Implement admin list-namespaces command (2 SP)
- [ ] TASK-080: Implement admin describe-namespace command (2 SP)
- [ ] TASK-081: Test namespace admin commands (1 SP)

### src/admin/handlers.rs - Command Integration (8 SP)
- [ ] TASK-082: Create handlers.rs with admin command functions (2 SP)
- [ ] TASK-083: Wire handle_admin_create_db() → db_crud::create_database() (2 SP)
- [ ] TASK-084: Wire handle_admin_switch_db() → db_crud::switch_database() (2 SP)
- [ ] TASK-085: Wire handle_admin_rename_db() and delete_db() (2 SP)

## Milestone 0c: Multi-Instance & Config Management (Total: 34 SP)
**Files: src/config.rs, src/kv_ops.rs, Enhanced: src/dispatcher.rs, src/admin/db_crud.rs**
**PREREQUISITE**: Milestones 0a (addressing) and 0b (admin/db_crud) MUST be complete

### src/config.rs - Config File Management (13 SP)
- [ ] TASK-060: Create config.rs module structure (2 SP)
- [ ] TASK-061: Implement read_prontorc() and write_prontorc() (3 SP)
- [ ] TASK-062: Implement discover_config() directory-based discovery (3 SP)
- [ ] TASK-063: Implement get_effective_db_path() resolution logic (3 SP)
- [ ] TASK-071: Test config file functionality (2 SP)

### src/kv_ops.rs - Core KV Operations (8 SP)
- [ ] TASK-012: Create kv_ops.rs and implement do_set() (2 SP)
- [ ] TASK-013: Implement do_get() with MISS handling (2 SP)
- [ ] TASK-014: Implement do_del() (2 SP)
- [ ] TASK-015: Test KV operations (2 SP)

### Enhanced Integration (13 SP)
- [ ] TASK-064: Enhance dispatcher.rs with --select flag parsing (3 SP)
- [ ] TASK-065: Enhance dispatcher.rs with --config flag parsing (3 SP)
- [ ] TASK-066: Wire dispatcher set/get/del handlers to kv_ops.rs (3 SP)
- [ ] TASK-067: Enhance admin/db_crud.rs with config integration (2 SP)
- [ ] TASK-072: Integration test for multi-instance functionality (2 SP)

## Milestone 1: Complete v0.1 Features (Total: 55 SP)

### Complete KV Operations (13 SP)
- [ ] TASK-016: Implement handle_keys with prefix filtering (3 SP)
- [ ] TASK-017: Implement handle_scan with key-value pairs (3 SP)
- [ ] TASK-018: Add JSON support (--json flag) for get/scan (3 SP)
- [ ] TASK-019: Implement ls as alias to scan with --stream format (2 SP)
- [ ] TASK-020: Test all KV operations against TEST-SPEC 2.x (2 SP)

### TTL Namespace Support (13 SP)
- [ ] TASK-021: Implement create-cache command (3 SP)
- [ ] TASK-022: Add TTL expiration check in get operation (3 SP)
- [ ] TASK-023: Store namespace metadata in sys_namespaces (2 SP)
- [ ] TASK-024: Auto-apply default TTL for TTL namespaces (3 SP)
- [ ] TASK-025: Test TTL functionality against TEST-SPEC 3.x (2 SP)

### Stream Processing (13 SP)
- [ ] TASK-026: Create stream token parser (5 SP)
- [ ] TASK-027: Implement auth preamble parsing (meta:sec:pass/user) (3 SP)
- [ ] TASK-028: Handle transaction boundaries per namespace (3 SP)
- [ ] TASK-029: Test stream processing against TEST-SPEC 4.x (2 SP)

### Discovery Commands (8 SP)
- [ ] TASK-030: Implement projects command (2 SP)
- [ ] TASK-031: Implement namespaces command with -p flag (2 SP)
- [ ] TASK-032: Implement nss (all namespaces) command (2 SP)
- [ ] TASK-033: Test discovery commands against TEST-SPEC 5.x (2 SP)

### Security & Auth (8 SP)
- [ ] TASK-034: Implement basic auth check (admin/pronto!) (3 SP)
- [ ] TASK-035: Add security.required config option (2 SP)
- [ ] TASK-036: Enforce auth in stream processing (2 SP)
- [ ] TASK-037: Test auth against TEST-SPEC 7.x (1 SP)

## Milestone 2: v0.2 Polish (Total: 34 SP)

### Import/Export (13 SP)
- [ ] TASK-038: Implement export to TSV format (5 SP)
- [ ] TASK-039: Implement import from TSV format (5 SP)
- [ ] TASK-040: Test roundtrip export/import (3 SP)

### Enhanced Streaming (8 SP)
- [ ] TASK-041: Implement --stream format output (3 SP)
- [ ] TASK-042: Add meta:ns alias support (2 SP)
- [ ] TASK-043: Validate meta:ttl only in TTL namespaces (2 SP)
- [ ] TASK-044: Test enhanced streaming (1 SP)

### Admin Commands (8 SP)
- [ ] TASK-045: Implement admin set-cache command (3 SP)
- [ ] TASK-046: Implement admin drop-cache command (3 SP)
- [ ] TASK-047: Test admin commands (2 SP)

### Explicit TTL Support (5 SP)
- [ ] TASK-048: Add --ttl flag to set command (2 SP)
- [ ] TASK-049: Validate --ttl only works in TTL namespaces (2 SP)
- [ ] TASK-050: Test explicit TTL (1 SP)

## Quick Win Tasks (Can do immediately)

These are the next immediate tasks to get basic functionality working:

1. **TASK-001**: Add rusqlite to Cargo.toml (1 SP)
2. **TASK-004**: Setup error handling (3 SP)
3. **TASK-005**: Parse address in dispatcher (3 SP)
4. **TASK-012**: Implement handle_set (2 SP)
5. **TASK-013**: Implement handle_get (2 SP)

Total for basic working system: 11 SP

## Task Dependencies

```
TASK-001 → TASK-008 → TASK-012/013/014
         ↘
TASK-004 → TASK-005 → TASK-006 → TASK-007
         ↘
TASK-002/003 (✅ Done)
```

## Sprint Planning Suggestion

### Sprint 1 (2 days, ~13 SP)
- Setup & Infrastructure
- Basic set/get working
- Tests: 1.1, 1.2, 2.1, 2.3

### Sprint 2 (2 days, ~13 SP)  
- Complete KV operations
- Basic TTL support
- Tests: 2.x, 3.1, 3.2

### Sprint 3 (2 days, ~13 SP)
- Stream processing basics
- Auth implementation
- Tests: 4.1, 4.2, 4.3, 7.1

### Sprint 4 (2 days, ~13 SP)
- Discovery commands
- Admin operations
- Tests: 5.1, remaining v0.1 tests

## Progress Tracking

### Completed ✅
- TASK-002: Wire dispatcher into main.rs
- TASK-003: Create module structure  
- TASK-008: Storage module with SQLite

### In Progress 🔧
- Addressing module created
- Dispatcher stubbed

### Next Up 📋
- TASK-001: Add rusqlite dependency
- TASK-004: Error handling
- TASK-005: Parse address in dispatcher
- TASK-012: Implement set
- TASK-013: Implement get

## Success Metrics

- **Milestone 0 Complete**: When `prontodb set test.ns.key value` and `prontodb get test.ns.key` work
- **Milestone 1 Complete**: All [v0.1] tests from TEST-SPEC.md pass
- **Milestone 2 Complete**: All [v0.2] tests pass

## Notes

- Each task should have corresponding test coverage
- Prioritize getting basic flow working end-to-end first
- Can parallelize some tasks (e.g., discovery commands while someone else does TTL)
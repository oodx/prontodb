# ProntoDB Structure Refactor Tasks

## Objective
Refactor data layout from flat to database-scoped structure for better organization and simpler backups.

## Current vs Target Structure

**Current:**
```
~/.local/data/odx/prontodb/
├── pronto.db
└── cursors/
    ├── default.cursor
    ├── staging.cursor
    └── production.cursor
```

**Target:**
```
~/.local/data/odx/prontodb/
├── pronto/
│   ├── pronto.db
│   ├── cursors/
│   │   ├── default.cursor
│   │   └── staging.cursor
│   └── meta/
│       └── db.meta
└── test/
    ├── test.db
    ├── cursors/
    └── meta/
```

## Story Point Tasks

### SP-1: XDG Path Refactor (3 points)
**Files to Change:**
- `src/xdg.rs` - Update path construction logic

**Key Changes:**
- Add database name parameter to path methods
- Create `get_database_dir(db_name)` method
- Update `get_db_path()` → `get_db_path(db_name)`
- Update `cursor_dir` → `get_cursor_dir(db_name)`

### SP-2: CursorManager Migration (5 points)
**Files to Change:**
- `src/cursor.rs` - Update all cursor resolution logic

**Key Changes:**
- Update `resolve_database_path()` to handle new structure
- Update cursor file paths to be database-scoped
- Add migration logic for existing cursors
- Maintain backward compatibility

### SP-3: Database Command Updates (3 points)
**Files to Change:**
- `src/api.rs` - Update database operations
- `src/dispatcher.rs` - Update command routing

**Key Changes:**
- Default database name: "pronto"
- Support `--database` flag for alternate DBs
- Update all database path resolutions

### SP-4: Backup Command Simplification (2 points)
**Files to Change:**
- `src/commands/backup.rs` - Simplify to backup entire DB directory

**Key Changes:**
- Backup entire database directory instead of collecting files
- Simpler tar command: `tar -czf backup.tar.gz <db_dir>/`

### SP-5: Migration Tool (3 points)
**New File:**
- `src/commands/migrate.rs` - One-time migration tool

**Features:**
- Detect old structure
- Move files to new structure
- Preserve all data and relationships
- Create backup before migration
- Rollback capability

### SP-6: Test Updates (2 points)
**Files to Change:**
- `tests/sanity_tests.rs`
- `tests/mvp_kv.rs`
- All test utilities

**Key Changes:**
- Update test paths
- Test migration scenarios
- Test multi-database support

## Implementation Plan

**Phase 1** (SP-1, SP-2): Foundation - paths and cursor management
**Phase 2** (SP-3, SP-5): Core - commands and migration
**Phase 3** (SP-4, SP-6): Cleanup - backup and tests

## Verification Checklist

After each SP implementation:
- [ ] Code compiles without warnings
- [ ] Existing tests pass
- [ ] Backward compatibility maintained
- [ ] No data loss scenarios
- [ ] Migration path clear
- [ ] RSB compliance maintained
- [ ] File structure matches spec
- [ ] Error handling robust

## Benefits

1. **Cleaner backups** - just tar the database directory
2. **Multi-database support** - natural structure for multiple DBs
3. **Better organization** - everything for a DB in one place
4. **Simpler mental model** - database contains everything

Total: ~18 story points
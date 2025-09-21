# =Ë ProntoDB CRUD Infrastructure Tasks

##  Completed Tasks
- [x] Hub integration (GitHub-first dependencies)
- [x] Switch RSB from local to GitHub version
- [x] Create basic RSB sanity tests (4 tests)
- [x] Remove meteor-specific tests

## =4 Critical Priority - Generic CRUD Interface
- [ ] **Define CRUD_SPEC pattern**
  - Document standard interface for all CRUD objects
  - Define trait requirements and error handling

- [ ] **Implement generic CRUD trait** (`src/lib/core/crud.rs`)
  - create, read, update, delete operations
  - Batch operations and namespace management
  - Metadata operations (exists, stats, list)

- [ ] **Build SQLite adapters** (`lib/adpt/sqlite/`)
  - base.rs - Main SQLiteAdapter implementing CRUD trait
  - utils.rs - Connection management, schema setup
  - mod.rs - Public exports
  - Remove current domain-specific code

## =à High Priority - RSB Admin CLI
- [ ] **Create admin CLI structure** (`bin/cli/admin/`)
  - Modern RSB bootstrap (GLOBAL, HOST, CLI)
  - Direct CRUD command interface
  - Use OPTIONS for argument parsing

- [ ] **Comprehensive RSB sanity tests**
  - Test all RSB features used in admin CLI
  - GLOBAL, HOST, STRINGS, OPTIONS modules
  - Optional: DEV, PARAMS, FS, COLORS

## =á Medium Priority - Documentation
- [ ] Update START.txt with CRUD patterns
- [ ] Update QUICK_REF.txt with RSB requirements
- [ ] Document RSB sanity test requirements
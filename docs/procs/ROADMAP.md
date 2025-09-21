# =Â ProntoDB CRUD Infrastructure Roadmap

## <¯ Current Status
- **Hub Integration**:  Complete (GitHub-first dependencies)
- **RSB GitHub Switch**:  Complete
- **RSB Sanity Tests**:  Basic tests created (4 tests)
- **Generic CRUD Interface**: L Not started
- **SQLite Adapters**: L Needs generic implementation
- **Admin CLI**: L Not started

## =§ Phase 1: Generic CRUD Foundation (Week 1)
- [ ] Create CRUD_SPEC pattern definition
- [ ] Implement generic CRUD trait in `src/lib/core/crud.rs`
- [ ] Build SQLite adapters in `lib/adpt/sqlite/` (base, utils, mod)
- [ ] Remove domain-specific ProntoDB adapter code

## =§ Phase 2: Admin CLI with RSB (Week 2)
- [ ] Create `bin/cli/admin/` structure
- [ ] Implement RSB-powered admin CLI (GLOBAL, HOST, CLI, OPTIONS)
- [ ] Add direct CRUD command interface
- [ ] Comprehensive RSB sanity tests for all used features

## =§ Phase 3: Integration & Testing (Week 3)
- [ ] Complete RSB feature testing (STRINGS, FS, COLORS, DEV)
- [ ] CRUD interface validation
- [ ] Admin CLI UAT testing
- [ ] Documentation updates

## <¯ Goal
Generic CRUD interface that any object can use, with modern RSB admin CLI for direct CRUD operations.
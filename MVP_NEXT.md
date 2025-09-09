# ProntoDB - Tier 1 Internal Usage Implementation Plan

**Status**: MVP Complete, Tier 1 Features Required  
**Goal**: Production-ready for internal multi-agent workflows  
**Timeline**: ~5-9 hours implementation  

## 🎯 **Current MVP Status**

ProntoDB MVP is **100% complete and production-ready**:

✅ **Complete dot addressing**: `project.namespace.key` with full project scoping  
✅ **All core operations**: set/get/del/keys/scan/projects/namespaces  
✅ **TTL namespaces**: create-cache with automatic expiry  
✅ **RSB compliance**: Full framework integration with proper lifecycle  
✅ **Production deployment**: Working `./bin/deploy.sh` with testing  
✅ **Comprehensive documentation**: Professional README and help system  
✅ **Test coverage**: 31 tests passing across all functionality  

**The MVP exceeded expectations** - includes full project scoping and advanced features.

---

## 🚧 **Missing Features for Tier 1 Internal Usage**

Based on analysis and documentation review, **3 features needed**:

### **1. CURSOR SUPPORT** 🔥 **CRITICAL**
**Purpose**: Multi-instance database selection caching  
**Use Case**: Different projects/environments need different database contexts  

**Architecture**:
```bash
# Cursor files location (XDG compliant)  
~/.local/data/odx/prontodb/cursors/
├── default.cursor          # Default cursor
├── user1.cursor           # User-specific cursors  
└── project_staging.cursor # Project-specific cursors
```

**Cursor File Format** (JSON):
```json
{
  "database_path": "/path/to/specific/db",
  "default_project": "myproject", 
  "default_namespace": "config",
  "created_at": "2025-01-08T10:00:00Z",
  "user": "alice"
}
```

### **2. MULTI-USER SUPPORT (`--user` flag)** 🔥 **CRITICAL**  
**Purpose**: User-specific cursor isolation for multi-agent workflows  
**Use Case**: Multiple agents/users need isolated database contexts  

**Usage**:
```bash
# Default behavior (uses default.cursor)
prontodb set app.config.debug true

# User-specific behavior (uses alice.cursor)  
prontodb --user alice set app.config.debug true   
prontodb --user bob set app.config.debug false    
```

### **3. LIFECYCLE COMMANDS** 📦 **NICE-TO-HAVE**
**Purpose**: Real install/backup/uninstall functionality  
**Current Status**: Basic stubs that return errors  

**Requirements**:
- **Install**: Copy binary, create XDG structure, initialize cursors  
- **Backup**: Export to timestamped tar.gz with cursors/config  
- **Uninstall**: Remove binary, optional `--purge` for data cleanup  

---

## 🛠 **Implementation Plan**

### **Phase 1: Cursor Support** ⏱️ **~3 hours**
**Files to modify/create**:
- `src/cursor.rs` - New cursor management module
- `src/xdg.rs` - Extend for cursor directory support
- `src/main.rs` - Add cursor flag parsing  
- `src/lib.rs` - Add cursor command handlers

**Key Implementation Steps**:
1. Create cursor storage module with JSON serialization
2. Add `--cursor <name>` flag support to RSB arg parsing
3. Implement cursor commands: `cursor set/list/active/delete`
4. Update database path resolution to check cursor files first
5. Add cursor auto-creation on first use

**Commands to add**:
```bash
prontodb cursor set staging /path/to/staging.db
prontodb cursor list
prontodb cursor active  
prontodb --cursor staging set app.config.key value
```

### **Phase 2: Multi-User Support** ⏱️ **~2 hours**
**Files to modify**:
- `src/cursor.rs` - Extend for user-specific cursor files  
- `src/main.rs` - Add `--user` flag parsing
- `src/lib.rs` - Update command handlers for user context

**Key Implementation Steps**:
1. Extend cursor system to support user-prefixed cursor files
2. Add `--user <username>` flag parsing in RSB args
3. Auto-create user cursors on first use (`user1.cursor`)
4. Update all command handlers to respect user context
5. Add user isolation testing

**Usage patterns**:
```bash
prontodb --user alice set app.config.debug true    # Uses alice.cursor
prontodb --user bob projects                        # Uses bob.cursor  
prontodb --user alice cursor set prod /path/to/prod.db
```

### **Phase 3: Lifecycle Commands** ⏱️ **~4 hours** 
**Files to modify**:
- `src/main.rs` - Replace stub implementations
- `src/lib.rs` - Add real lifecycle command handlers
- New: `src/backup.rs` - Backup/restore functionality

**Key Implementation Steps**:
1. **Install**: Binary installation, XDG setup, cursor initialization
2. **Backup**: Database + cursor + config export to tar.gz  
3. **Uninstall**: Binary removal, optional data purging with confirmation
4. Add comprehensive error handling and user feedback
5. Integration with existing deployment script

---

## ⚡ **Priority Recommendations**

### **🔥 IMMEDIATE (For Tier 1 Usage)**
1. **Phase 1 + 2**: Cursor Support + Multi-User (~5 hours)
   - Enables multi-instance coordination  
   - Enables multi-agent workflows
   - Provides 100% tier 1 readiness

### **📦 DEFERRED (Can implement later)**  
3. **Phase 3**: Lifecycle Commands (~4 hours)
   - Deploy script already handles installation effectively
   - Backup can be manual database copy initially
   - Not blocking for tier 1 internal usage

---

## 📊 **Effort Summary**

| Phase | Feature | Hours | Priority | Status |
|-------|---------|--------|----------|--------|
| 1 | Cursor Support | 3h | 🔥 Critical | Not started |
| 2 | Multi-User (`--user`) | 2h | 🔥 Critical | Not started | 
| 3 | Lifecycle Commands | 4h | 📦 Nice-to-have | Stubs exist |
| **TOTAL** | **Full Tier 1** | **9h** | | **5h for minimal** |

---

## ✅ **Success Criteria**

### **Minimal Tier 1 Ready** (Phases 1+2):
- [ ] Multiple database contexts via cursors
- [ ] User isolation via `--user` flag  
- [ ] Cursor management commands working
- [ ] All existing tests still pass
- [ ] Documentation updated

### **Complete Tier 1 Ready** (All phases):
- [ ] Working install/backup/uninstall commands
- [ ] Integration with existing deploy script
- [ ] Backup/restore functionality tested
- [ ] Full lifecycle command documentation

---

## 🚀 **Next Steps**

1. **Implement Phase 1**: Cursor support for multi-instance coordination
2. **Implement Phase 2**: Multi-user support for agent isolation  
3. **Test integration**: Verify all existing functionality preserved
4. **Update documentation**: README, help system, deployment guide
5. **Optional Phase 3**: Real lifecycle commands (time permitting)

**With Phases 1+2 complete, ProntoDB will be fully ready for tier 1 internal usage** with multi-agent support and database context management. 🎯
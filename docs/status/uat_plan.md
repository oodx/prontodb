# ProntoDB UAT Verification Plan

**Comprehensive User Acceptance Testing for Production Deployment**

## 🎯 **Verification Objective**

Validate that ProntoDB v0.4.0+ is PRODUCTION-READY for internal team deployment with:
- **100% Internal Deployment Features** - Install/uninstall, backup/restore, cursor CRUD
- **Multi-agent workflow capability** - User isolation + database contexts  
- **Complete lifecycle management** - Professional CLI with nuclear safety
- **Advanced team workflows** - Working directory cursors with opt-in control (PRONTO_WORK_MODE)
- **Full rewindability and reliability** - Comprehensive backup/restore system
- **Enterprise safety controls** - Environment variable control for project vs global access

---

## 📋 **Pre-Test Setup**

### **Environment Preparation**
1. **Build Latest Version**:
   ```bash
   cd /home/xnull/repos/code/rust/oodx/prontodb
   cargo build --release
   ```

2. **Run Automated UAT**:
   ```bash
   ./test.sh
   ```

3. **Verify Test Output**: All tests should pass with green checkmarks ✅

### **Manual Verification Points**
After automated tests pass, verify these additional scenarios:

---

## 🧪 **Feature Verification Matrix**

### **Phase 1: Core Key-Value Operations**
| Feature | Test Command | Expected Result | Status |
|---------|--------------|-----------------|--------|
| **Set Value** | `prontodb set app.config.host "localhost"` | Success message | ⬜ |
| **Get Value** | `prontodb get app.config.host` | Output: `localhost` | ⬜ |
| **Delete Value** | `prontodb del app.config.host` | Success message | ⬜ |
| **Get Deleted** | `prontodb get app.config.host` | Exit code 2 (not found) | ⬜ |

### **Phase 2: Dot Addressing & Discovery**
| Feature | Test Command | Expected Result | Status |
|---------|--------------|-----------------|--------|
| **Complex Addressing** | `prontodb set myapp.prod.db.host "prod.db.com"` | Success | ⬜ |
| **Context Addressing** | `prontodb set app.config.key__prod "value"` | Success | ⬜ |
| **List Projects** | `prontodb projects` | Shows: myapp, app | ⬜ |
| **List Namespaces** | `prontodb namespaces -p myapp` | Shows: prod | ⬜ |
| **List Keys** | `prontodb keys myapp.prod` | Shows keys with prefix | ⬜ |
| **Scan Pairs** | `prontodb scan myapp.prod` | Shows key=value pairs | ⬜ |

### **Phase 3: TTL Cache Support**
| Feature | Test Command | Expected Result | Status |
|---------|--------------|-----------------|--------|
| **Create TTL Cache** | `prontodb create-cache sessions.cache 3600` | Success | ⬜ |
| **Set Cache Value** | `prontodb set sessions.cache.user123 "active"` | Success | ⬜ |
| **Get Cache Value** | `prontodb get sessions.cache.user123` | Output: `active` | ⬜ |

### **Phase 4: Cursor Management** 
| Feature | Test Command | Expected Result | Status |
|---------|--------------|-----------------|--------|
| **Create Cursor** | `prontodb cursor set staging /tmp/staging.db` | Success | ⬜ |
| **List Cursors** | `prontodb cursor list` | Shows: staging | ⬜ |
| **Use Cursor** | `prontodb --cursor staging set test.key value` | Success | ⬜ |
| **Cursor Isolation** | `prontodb get test.key` (without cursor) | Exit code 2 | ⬜ |
| **Delete Cursor** | `prontodb cursor delete staging` | Success | ⬜ |

### **Phase 5: Multi-User Support**
| Feature | Test Command | Expected Result | Status |
|---------|--------------|-----------------|--------|
| **User Cursor** | `prontodb --user alice cursor set dev /tmp/alice.db` | Success | ⬜ |
| **User Data** | `prontodb --user alice --cursor dev set user.name "Alice"` | Success | ⬜ |
| **User Isolation** | `prontodb --user bob --cursor dev get user.name` | Exit code 2 | ⬜ |
| **Combined Flags** | `prontodb --user alice --cursor dev get user.name` | Output: `Alice` | ⬜ |

### **Phase 6: Production Lifecycle Commands**
| Feature | Test Command | Expected Result | Status |
|---------|--------------|-----------------|--------|
| **Install Help** | `prontodb install --help` | Shows usage info | ⬜ |
| **Install Binary** | `prontodb install --target /tmp/test_install` | Binary created | ⬜ |
| **Test Installed** | `/tmp/test_install/prontodb --version` | Version displayed | ⬜ |
| **Backup Create** | `prontodb backup --output /tmp/backup.tar.gz` | Backup file created | ⬜ |
| **Backup List** | `prontodb backup --list` | Shows backup files | ⬜ |
| **Standalone Restore** | `prontodb restore /tmp/backup.tar.gz` | Data restored | ⬜ |
| **Nuclear Safety** | `prontodb uninstall --nuke --force` | Safety backup created | ⬜ |
| **Uninstall Help** | `prontodb uninstall --help` | Shows usage info | ⬜ |

### **Phase 7: Advanced Cursor CRUD**
| Feature | Test Command | Expected Result | Status |
|---------|--------------|-----------------|--------|
| **Cursor Delete** | `prontodb cursor delete test_cursor` | Cursor removed | ⬜ |
| **Cursor Reset User** | `prontodb cursor reset --user alice` | Alice cursors cleared | ⬜ |
| **Cursor Reset All** | `prontodb cursor reset --all` | All cursors cleared | ⬜ |
| **Cursor List After** | `prontodb cursor list` | Shows remaining cursors | ⬜ |

### **Phase 8: Working Directory Cursors (Opt-in)** 
| Feature | Test Command | Expected Result | Status |
|---------|--------------|-----------------|--------|
| **Without Work Mode** | `prontodb set test.key "global"` | Uses global DB (default) | ⬜ |
| **Create .prontodb** | `echo "/tmp/project1.prdb" > .prontodb` | File created | ⬜ |
| **Work Mode Off** | `prontodb set project.key "still_global"` | Still uses global DB | ⬜ |
| **Enable Work Mode** | `PRONTO_WORK_MODE=1 prontodb set project.key "local"` | Uses local database | ⬜ |
| **Verify Local** | `PRONTO_WORK_MODE=1 prontodb get project.key` | Returns "local" | ⬜ |
| **Verify Global** | `prontodb get project.key` | Returns "still_global" | ⬜ |
| **JSON Config** | Create JSON .prontodb with user/cursor mapping | Complex config works | ⬜ |

### **Phase 9: Help & Version System**
| Feature | Test Command | Expected Result | Status |
|---------|--------------|-----------------|--------|
| **Main Help** | `prontodb help` | Complete help with examples | ⬜ |
| **Command Help** | `prontodb cursor --help` | Cursor-specific help | ⬜ |
| **Version (long)** | `prontodb version` | Version info displayed | ⬜ |
| **Version (short)** | `prontodb -v` | Version info displayed | ⬜ |
| **Version (flag)** | `prontodb --version` | Version info displayed | ⬜ |

---

## 🔄 **Rewindability Tests**

### **Database State Rewind**
1. **Create Test State**:
   ```bash
   prontodb set app.state.step "initial"
   prontodb set app.data.value "important"
   prontodb cursor set snapshot /tmp/snapshot.db
   prontodb --cursor snapshot set app.state.step "snapshot"
   ```

2. **Create Backup**:
   ```bash
   prontodb backup --output /tmp/rewind_test.tar.gz
   ```

3. **Modify State**:
   ```bash
   prontodb set app.state.step "modified"
   prontodb del app.data.value
   ```

4. **Verify Rewind**:
   ```bash
   prontodb backup --restore /tmp/rewind_test.tar.gz
   prontodb get app.state.step  # Should output: "initial"
   prontodb get app.data.value  # Should output: "important"
   ```

### **Cursor State Rewind**
1. **Verify Cursor Restoration**:
   ```bash
   prontodb cursor list  # Should show: snapshot
   prontodb --cursor snapshot get app.state.step  # Should output: "snapshot"
   ```

---

## ⚡ **Performance Verification**

### **Basic Performance Test**
```bash
# Time 100 sequential operations
time bash -c 'for i in {1..100}; do prontodb set perf.key$i value$i >/dev/null; done'
time bash -c 'for i in {1..100}; do prontodb get perf.key$i >/dev/null; done'
```

**Acceptance Criteria**: Operations should complete in under 10 seconds total.

### **Concurrent Access Test**
```bash
# Run 3 concurrent processes
prontodb --user user1 set test.concurrent "process1" &
prontodb --user user2 set test.concurrent "process2" &
prontodb --user user3 set test.concurrent "process3" &
wait

# Verify isolation
prontodb --user user1 get test.concurrent  # Should be "process1"
prontodb --user user2 get test.concurrent  # Should be "process2"
prontodb --user user3 get test.concurrent  # Should be "process3"
```

---

## 🚨 **Error Handling Verification**

### **Expected Failures**
| Scenario | Test Command | Expected Result | Status |
|----------|--------------|-----------------|--------|
| **Non-existent Key** | `prontodb get missing.key` | Exit code 2 | ⬜ |
| **Non-existent Cursor** | `prontodb --cursor missing get any.key` | Error message | ⬜ |
| **Invalid Project Syntax** | `prontodb set .invalid..key value` | Error message | ⬜ |
| **Empty Key** | `prontodb set "" value` | Error message | ⬜ |

### **Recovery Scenarios**
| Scenario | Test Steps | Expected Result | Status |
|----------|------------|-----------------|--------|
| **Corrupted Cursor** | Delete cursor file manually, then use cursor | Auto-recreation | ⬜ |
| **Missing Database** | Use cursor pointing to deleted DB file | Graceful handling | ⬜ |

---

## 📊 **Infrastructure Readiness Checklist**

### **Multi-Agent Workflow Readiness**
- ⬜ **Agent Isolation**: Multiple `--user` flags work independently
- ⬜ **Database Context**: Multiple `--cursor` flags provide separate databases
- ⬜ **Combined Operations**: `--user agent1 --cursor prod` works reliably
- ⬜ **Concurrent Safety**: Multiple agents can operate simultaneously
- ⬜ **State Persistence**: Agent state survives restart/restore cycles

### **Production Deployment Readiness**
- ⬜ **Installation**: `install` command works on target systems
- ⬜ **Backup/Restore**: Full data backup and restoration working
- ⬜ **Uninstall**: Clean removal process available
- ⬜ **Help System**: Complete documentation accessible
- ⬜ **Error Handling**: Graceful failure modes implemented

### **Reliability & Maintenance**
- ⬜ **Automated Testing**: All automated tests pass
- ⬜ **Manual Verification**: All manual tests pass
- ⬜ **Performance**: Meets performance acceptance criteria
- ⬜ **Rewindability**: Backup/restore cycle preserves all state
- ⬜ **Documentation**: README and help system complete

---

## ✅ **Final Approval Criteria**

**ProntoDB is approved for production infrastructure deployment when:**

1. ✅ **All automated tests pass** (`./test.sh` completes successfully)
2. ✅ **All manual verification checkboxes marked** (⬜ → ✅)
3. ✅ **Performance criteria met** (< 10 seconds for 100 ops)
4. ✅ **Rewindability confirmed** (backup/restore preserves state)
5. ✅ **Multi-agent scenarios working** (user isolation functional)
6. ✅ **Error handling verified** (expected failures behave correctly)

---

## 🌑 **Avatar Verification Signature**

Upon completion of all verification steps:

```
Verified by: [Avatar Signature]
Date: [Verification Date]
Status: [ ] APPROVED FOR PRODUCTION | [ ] REQUIRES FIXES
Notes: [Any additional observations]
```

---

**🚀 Ready for infrastructure deployment with confidence!**
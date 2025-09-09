# PRONTODB API CONSISTENCY FIX PLAN
**Based on China's Comprehensive Inconsistency Audit**  
**Date**: 2025-09-09  
**Status**: Ready for systematic execution  

## 🎯 OBJECTIVE
Fix API inconsistencies blocking UAT and ensure documentation matches implementation.

## 📋 PRIORITY EXECUTION ORDER

### 🔥 PHASE 1: CRITICAL UAT BLOCKERS (IMMEDIATE)
**Goal**: Make UAT script work without errors

#### 1.1 Fix UAT Create-Cache Syntax ⚡ HIGH
- **File**: `bin/uat.sh` line 184
- **Problem**: Uses broken `timeout=300` syntax
- **Fix**: Change to positional syntax `300`
- **Before**: `prontodb create-cache $TEST_PROJECT.$TTL_NAMESPACE timeout=300`  
- **After**: `prontodb create-cache $TEST_PROJECT.$TTL_NAMESPACE 300`
- **Test**: Run UAT and verify create-cache command succeeds

#### 1.2 Verify UAT Command Patterns ⚡ HIGH  
- **Goal**: Ensure all UAT commands use working syntax
- **Test**: Run full UAT script and document any other failures
- **Fix**: Update any other broken command patterns discovered

### 🚨 PHASE 2: USER DOCUMENTATION ACCURACY (MEDIUM)
**Goal**: Fix help system to match implementation

#### 2.1 Update Dispatcher Help Text 📝 MEDIUM
- **File**: `src/dispatcher.rs` lines 474-516 (print_help function)
- **Problems**: 
  - Says keys/scan "requires -p and -n" (but dot addressing works)
  - Shows create-cache with "timeout=SECONDS" (but uses positional args)
- **Fix**: Update help text to show both flag AND dot addressing options
- **Test**: Run `prontodb help` and verify accuracy

#### 2.2 Enhance UAT with Dot Addressing Coverage 🧪 MEDIUM
- **File**: `bin/uat.sh`
- **Goal**: Test documented dot addressing features  
- **Add**: New test phase for dot addressing:
  ```bash
  # Phase: Dot Addressing for Discovery
  run_command "$BINARY keys $TEST_PROJECT.$TEST_NAMESPACE" "List keys with dot addressing"
  run_command "$BINARY scan $TEST_PROJECT.$TEST_NAMESPACE" "Scan with dot addressing"  
  ```
- **Test**: Verify new tests pass

### 🐥 PHASE 3: DOCUMENTATION POLISH (LOW)
**Goal**: Clean up minor documentation inconsistencies

#### 3.1 Clean ACTUAL_SYNTAX.md 📖 LOW
- **File**: `ACTUAL_SYNTAX.md` 
- **Problem**: May contain stray references to old timeout= syntax
- **Fix**: Review and clean any outdated syntax references
- **Test**: Verify all examples in docs actually work

## 🧪 VERIFICATION PROTOCOL

### Test Sequence After Each Phase:
1. **Compile Check**: `cargo clippy --all-targets --all-features -- -D warnings`
2. **Unit Tests**: `cargo test` (verify 35 tests still pass)
3. **UAT Script**: `./bin/uat.sh` (should complete without errors)  
4. **Manual Spot Check**: Test key API patterns manually
5. **Help Accuracy**: Compare help output to documented patterns

### Success Criteria:
- [ ] UAT script completes without errors
- [ ] Help text matches actual implementation  
- [ ] All documented patterns work as claimed
- [ ] Dot addressing properly covered in tests
- [ ] Zero compiler warnings maintained

## 🚫 WHAT NOT TO CHANGE

### Leave Alone (Working Correctly):
- **3-level addressing limit** - Correctly implemented and documented
- **Core CRUD operations** - Set/get/del work perfectly with dot addressing
- **Flag-based syntax** - Still works as fallback for all commands
- **RSB compliance** - Properly documented with .rsb-defects file

## 📊 ESTIMATED EFFORT

### Phase 1 (Critical): ~15 minutes
- Simple syntax fix in UAT script
- Quick verification run

### Phase 2 (Medium): ~30 minutes  
- Help text updates in dispatcher
- Add UAT test cases for dot addressing
- Documentation review

### Phase 3 (Low): ~10 minutes
- Minor documentation cleanup
- Final verification

### Total: ~1 hour systematic execution

## 🎯 SUCCESS DEFINITION
**ProntoDB UAT runs clean, help system is accurate, dot addressing fully tested**

The API inconsistencies are actually minimal - mostly UAT using outdated syntax and help text drift. The core implementation is solid!

---
*Created by KEEPER based on China the Summary Chicken's forensic analysis*  
*Ready for systematic execution in priority order*
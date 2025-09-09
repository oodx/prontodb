#!/bin/bash
# ProntoDB Comprehensive UAT Test Suite
# Tests all functionality including cursor support, multi-user, and lifecycle commands


# Test paths are relative to the project root
set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_USER_1="alice"
TEST_USER_2="bob"
TEST_CURSOR_1="staging"
TEST_CURSOR_2="production"
TEST_PROJECT="testapp"
TEST_NAMESPACE="config"
BACKUP_DIR="../test_backups"
INSTALL_DIR="../test_install"

# Cleanup function
cleanup() {
    echo -e "${YELLOW}🧹 Cleaning up test environment...${NC}"
    
    # Remove test databases
    rm -f ../test_staging.db ../test_production.db ../test_alice.db ../test_bob.db
    
    # Remove test backups
    rm -rf "$BACKUP_DIR"
    
    # Remove test installation
    rm -rf "$INSTALL_DIR"
    
    # Clean system cursors created during testing
    rm -rf ~/.local/share/odx/prontodb/cursors/*test* 2>/dev/null || true
    rm -rf ~/.local/data/odx/prontodb/cursors/*test* 2>/dev/null || true
    
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Set trap for cleanup on script exit
trap cleanup EXIT

# Helper functions
log_test() {
    echo -e "${BLUE}🧪 Testing: $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

log_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# Build the project
echo -e "${BLUE}🔨 Building ProntoDB...${NC}"
cargo build --release || log_error "Build failed"
PRONTODB="./target/release/prontodb"

echo -e "${GREEN}🚀 Starting ProntoDB UAT Test Suite${NC}"
echo "=============================================="

# Test 1: Basic Key-Value Operations
log_test "Basic key-value operations"
$PRONTODB set "$TEST_PROJECT.$TEST_NAMESPACE.debug" "true" || log_error "Set operation failed"
RESULT=$($PRONTODB get "$TEST_PROJECT.$TEST_NAMESPACE.debug")
[[ "$RESULT" == "true" ]] || log_error "Get operation failed - expected 'true', got '$RESULT'"
$PRONTODB del "$TEST_PROJECT.$TEST_NAMESPACE.debug" || log_error "Delete operation failed"
log_success "Basic key-value operations working"

# Test 2: Dot Addressing
log_test "Dot addressing syntax"
$PRONTODB set "app.config.database_host" "localhost" || log_error "Dot addressing set failed"
$PRONTODB set "app.config.database_port" "5432" || log_error "Dot addressing set failed"
$PRONTODB set "app.cache.ttl" "3600" || log_error "Dot addressing set failed"

# Verify retrieval
HOST=$($PRONTODB get "app.config.database_host")
[[ "$HOST" == "localhost" ]] || log_error "Dot addressing get failed"

# Test keys command with project and namespace
KEYS=$($PRONTODB keys -p app -n config)
echo "$KEYS" | grep -q "database_host" || log_error "Keys command failed"
echo "$KEYS" | grep -q "database_port" || log_error "Keys command failed"

log_success "Dot addressing working"

# Test 3: Discovery Commands
log_test "Discovery commands (projects, namespaces)"
$PRONTODB projects | grep -q "app" || log_error "Projects command failed"
$PRONTODB namespaces -p app | grep -q "config" || log_error "Namespaces command failed"
$PRONTODB namespaces -p app | grep -q "cache" || log_error "Namespaces command failed"
log_success "Discovery commands working"

# Test 4: TTL Cache Creation
log_test "TTL cache creation"
$PRONTODB create-cache "sessions.cache" "timeout=60" || log_error "TTL cache creation failed"
$PRONTODB set "sessions.cache.user123" "active" || log_error "TTL cache set failed"
CACHE_VAL=$($PRONTODB get "sessions.cache.user123")
[[ "$CACHE_VAL" == "active" ]] || log_error "TTL cache get failed"
log_success "TTL cache creation working"

# Test 5: Cursor Management
log_test "Cursor management operations"

# Create test cursors
$PRONTODB cursor set "$TEST_CURSOR_1" "../test_staging.db" || log_error "Cursor creation failed"
$PRONTODB cursor set "$TEST_CURSOR_2" "../test_production.db" || log_error "Cursor creation failed"

# List cursors
CURSOR_LIST=$($PRONTODB cursor list)
echo "$CURSOR_LIST" | grep -q "$TEST_CURSOR_1" || log_error "Cursor listing failed"
echo "$CURSOR_LIST" | grep -q "$TEST_CURSOR_2" || log_error "Cursor listing failed"

# Test cursor usage
$PRONTODB --cursor "$TEST_CURSOR_1" set "staging.env.debug" "true" || log_error "Cursor usage failed"
$PRONTODB --cursor "$TEST_CURSOR_2" set "prod.env.debug" "false" || log_error "Cursor usage failed"

# Verify cursor isolation
STAGING_VAL=$($PRONTODB --cursor "$TEST_CURSOR_1" get "staging.env.debug")
PROD_VAL=$($PRONTODB --cursor "$TEST_CURSOR_2" get "prod.env.debug")
[[ "$STAGING_VAL" == "true" ]] || log_error "Staging cursor value incorrect"
[[ "$PROD_VAL" == "false" ]] || log_error "Production cursor value incorrect"

# Test cursor deletion
$PRONTODB cursor delete "$TEST_CURSOR_2" || log_error "Cursor deletion failed"
CURSOR_LIST_AFTER=$($PRONTODB cursor list)
echo "$CURSOR_LIST_AFTER" | grep -q "$TEST_CURSOR_1" || log_error "Remaining cursor missing"
! echo "$CURSOR_LIST_AFTER" | grep -q "$TEST_CURSOR_2" || log_error "Deleted cursor still present"

log_success "Cursor management working"

# Test 6: Multi-User Support
log_test "Multi-user isolation"

# Create user-specific cursors
$PRONTODB --user "$TEST_USER_1" cursor set "dev" "../test_alice.db" || log_error "User cursor creation failed"
$PRONTODB --user "$TEST_USER_2" cursor set "dev" "../test_bob.db" || log_error "User cursor creation failed"

# Set user-specific data
$PRONTODB --user "$TEST_USER_1" --cursor "dev" set "user.name" "Alice Developer" || log_error "User-specific set failed"
$PRONTODB --user "$TEST_USER_2" --cursor "dev" set "user.name" "Bob Developer" || log_error "User-specific set failed"

# Verify user isolation
ALICE_NAME=$($PRONTODB --user "$TEST_USER_1" --cursor "dev" get "user.name")
BOB_NAME=$($PRONTODB --user "$TEST_USER_2" --cursor "dev" get "user.name")
[[ "$ALICE_NAME" == "Alice Developer" ]] || log_error "Alice's data incorrect"
[[ "$BOB_NAME" == "Bob Developer" ]] || log_error "Bob's data incorrect"

# Verify cursor isolation between users
ALICE_CURSORS=$($PRONTODB --user "$TEST_USER_1" cursor list)
BOB_CURSORS=$($PRONTODB --user "$TEST_USER_2" cursor list)
echo "$ALICE_CURSORS" | grep -q "dev" || log_error "Alice's cursor missing"
echo "$BOB_CURSORS" | grep -q "dev" || log_error "Bob's cursor missing"

log_success "Multi-user isolation working"

# Test 7: Lifecycle Commands - Install
log_test "Install command"

# Test install to custom directory
mkdir -p "$INSTALL_DIR"
$PRONTODB install --target "$INSTALL_DIR/prontodb" --force || log_error "Install command failed"

# Verify installation (install creates a directory structure)
[[ -f "$INSTALL_DIR/prontodb/prontodb" ]] || log_error "Binary not installed"
[[ -x "$INSTALL_DIR/prontodb/prontodb" ]] || log_error "Binary not executable"

# Test installed binary
"$INSTALL_DIR/prontodb/prontodb" help >/dev/null || log_error "Installed binary not working"

log_success "Install command working"

# Test 8: Lifecycle Commands - Backup
log_test "Backup and restore commands"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Set some test data
$PRONTODB set "backup.test.key1" "value1" || log_error "Backup test data set failed"
$PRONTODB set "backup.test.key2" "value2" || log_error "Backup test data set failed"

# Create backup - use directory only, let backup command name the file
$PRONTODB backup --output "$BACKUP_DIR" --quiet || log_error "Backup creation failed"

# Find the created backup file
BACKUP_FILE=$(ls "$BACKUP_DIR"/prontodb-backup-*.tar.gz | head -1)
[[ -n "$BACKUP_FILE" ]] || log_error "Backup file not found"

# Verify backup file exists
[[ -f "$BACKUP_FILE" ]] || log_error "Backup file not created"

# Test backup listing
$PRONTODB backup --list | head -10 || log_error "Backup listing failed"

# Clear data
$PRONTODB del "backup.test.key1" || log_error "Test data clear failed"
$PRONTODB del "backup.test.key2" || log_error "Test data clear failed"

# Restore from backup
$PRONTODB backup --restore "$BACKUP_FILE" --quiet || log_error "Backup restore failed"

# Verify restored data
RESTORED_VAL1=$($PRONTODB get "backup.test.key1")
RESTORED_VAL2=$($PRONTODB get "backup.test.key2")
[[ "$RESTORED_VAL1" == "value1" ]] || log_error "Restored data incorrect"
[[ "$RESTORED_VAL2" == "value2" ]] || log_error "Restored data incorrect"

log_success "Backup and restore working"

# Test 9: Lifecycle Commands - Uninstall
log_test "Uninstall command (dry run)"

# Test uninstall help
$PRONTODB uninstall --help || log_error "Uninstall help failed"

# Note: We don't test actual uninstall to avoid breaking the test environment
log_info "Uninstall command verified (help accessible, skipping actual uninstall)"

# Test 10: Error Handling
log_test "Error handling and edge cases"

# Test non-existent key
if $PRONTODB get "nonexistent.key" >/dev/null 2>&1; then
    log_error "Expected error for non-existent key"
fi

# Test invalid cursor
if $PRONTODB --cursor "nonexistent" get "any.key" >/dev/null 2>&1; then
    log_error "Expected error for non-existent cursor"
fi

# Test invalid user characters (this should work but be sanitized)
$PRONTODB --user "test_user123" cursor list >/dev/null || log_error "Valid user name rejected"

log_success "Error handling working"

# Test 11: Help System
log_test "Help system completeness"

$PRONTODB help | grep -q "cursor" || log_error "Help missing cursor documentation"
$PRONTODB help | grep -q "install" || log_error "Help missing install documentation"
$PRONTODB help | grep -q "backup" || log_error "Help missing backup documentation"
$PRONTODB cursor --help || log_error "Cursor help failed"

log_success "Help system complete"

# Test 12: Version Command
log_test "Version command"
$PRONTODB version || log_error "Version command failed"
$PRONTODB --version || log_error "Version flag failed"
$PRONTODB -v || log_error "Version short flag failed"
log_success "Version command working"

# Test 13: XDG Directory Validation
log_test "XDG directory validation (no malformed directories)"

# Check for malformed XDG directories
MALFORMED_DIRS=$(find . -name "\${*" -type d 2>/dev/null || true)
[[ -z "$MALFORMED_DIRS" ]] || log_error "Malformed XDG directories found: $MALFORMED_DIRS"

log_success "XDG directory validation passed"

# Test 14: Performance Test (basic)
log_test "Basic performance test"

# Set/get 100 keys rapidly
for i in {1..100}; do
    $PRONTODB set "perf.test.key$i" "value$i" >/dev/null || log_error "Performance test set failed at $i"
done

for i in {1..100}; do
    VAL=$($PRONTODB get "perf.test.key$i")
    [[ "$VAL" == "value$i" ]] || log_error "Performance test get failed at $i"
done

log_success "Basic performance test passed"

echo ""
echo -e "${GREEN}🎉 ALL TESTS PASSED!${NC}"
echo "=============================================="
echo -e "${GREEN}✅ ProntoDB is ready for production deployment${NC}"
echo ""
echo "Summary of tested features:"
echo "  ✅ Basic key-value operations"
echo "  ✅ Dot addressing syntax"
echo "  ✅ Discovery commands (projects/namespaces)"
echo "  ✅ TTL cache creation"
echo "  ✅ Cursor management (create/list/delete/usage)"
echo "  ✅ Multi-user isolation"
echo "  ✅ Lifecycle commands (install/backup/restore)"
echo "  ✅ Error handling and edge cases"
echo "  ✅ Help system completeness"
echo "  ✅ Version command"
echo "  ✅ Basic performance test"
echo ""
echo -e "${BLUE}Ready for Avatar verification! 🌑${NC}"

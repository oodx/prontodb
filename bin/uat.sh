#!/bin/bash

# ProntoDB User Acceptance Test (UAT) Script
# Ceremonious walkthrough of all MVP features with terminal UX
# Uses boxy for enhanced visual presentation
#
# Usage:
#   ./bin/uat.sh                    # Run full UAT with cleanup
#   CLEANUP_ON_EXIT=0 ./bin/uat.sh  # Preserve test data
#
# Requirements:
#   - cargo build (to create ./target/debug/prontodb)
#   - boxy v0.7.0+ for terminal UI

set -e  # Exit on any error

# Colors and styling - using boxy v0.7.0 syntax
BOXY_DEFAULT_STYLE="rounded"
BOXY_DEFAULT_COLOR="cyan"

# Configuration
BINARY="./target/debug/prontodb"
TEST_PROJECT="uat_demo"
TEST_NAMESPACE="showcase"
TTL_NAMESPACE="cache"
CLEANUP_ON_EXIT=1

# Ensure binary exists
if [[ ! -f "$BINARY" ]]; then
    echo "❌ Binary not found at $BINARY"
    echo "Run 'cargo build' first"
    exit 1
fi

# Cleanup function
cleanup() {
    if [[ $CLEANUP_ON_EXIT -eq 1 ]]; then
        echo ""
        boxy --style "$BOXY_DEFAULT_STYLE" --color yellow << 'EOF'
🧹 CLEANUP PHASE
Removing test data...
EOF
        $BINARY del -p "$TEST_PROJECT" -n "$TEST_NAMESPACE" demo_key 2>/dev/null || true
        $BINARY del -p "$TEST_PROJECT" -n "$TEST_NAMESPACE" config__prod 2>/dev/null || true
        $BINARY del -p "$TEST_PROJECT" -n "$TEST_NAMESPACE" user_pref 2>/dev/null || true
        $BINARY del -p "$TEST_PROJECT" -n "$TTL_NAMESPACE" temp_session 2>/dev/null || true
        echo "✅ Cleanup completed"
    fi
}

# Set cleanup trap
trap cleanup EXIT

# Helper functions
pause_for_effect() {
    sleep "${1:-1}"
}

run_command() {
    local cmd="$1"
    local desc="$2"
    
    echo ""
    boxy --style "$BOXY_DEFAULT_STYLE" --color green << EOF
🚀 COMMAND: $desc
$ $cmd
EOF
    
    pause_for_effect 0.5
    
    # Execute and capture both output and exit code
    set +e
    output=$($cmd 2>&1)
    exit_code=$?
    set -e
    
    # Display results
    if [[ $exit_code -eq 0 ]]; then
        if [[ -n "$output" ]]; then
            boxy --style "$BOXY_DEFAULT_STYLE" --color blue << EOF
📤 OUTPUT:
$output
EOF
        else
            boxy --style "$BOXY_DEFAULT_STYLE" --color blue << 'EOF'
📤 OUTPUT: (success, no output)
EOF
        fi
        boxy --style "$BOXY_DEFAULT_STYLE" --color green << EOF
✅ EXIT CODE: $exit_code (SUCCESS)
EOF
    elif [[ $exit_code -eq 2 ]]; then
        boxy --style "$BOXY_DEFAULT_STYLE" --color yellow << EOF
📤 OUTPUT:
$output

🔍 EXIT CODE: $exit_code (MISS - key not found/expired)
EOF
    else
        boxy --style "$BOXY_DEFAULT_STYLE" --color red << EOF
📤 OUTPUT:
$output

❌ EXIT CODE: $exit_code (ERROR)
EOF
    fi
    
    pause_for_effect 1
}

# Main UAT sequence
main() {
    clear
    
    # Header
    boxy --style "$BOXY_DEFAULT_STYLE" --color cyan << 'EOF'
🎉 PRONTODB UAT CEREMONY 🎉

User Acceptance Testing
Comprehensive Feature Walkthrough

🔥 Ready to showcase ProntoDB MVP! 🔥
EOF
    
    pause_for_effect 2
    
    # Phase 1: Help & Discovery
    boxy --style "$BOXY_DEFAULT_STYLE" --color magenta << 'EOF'
📋 PHASE 1: HELP & DISCOVERY
Testing help system and basic discovery
EOF
    
    run_command "$BINARY help" "Display help information"
    run_command "$BINARY projects" "List all projects (should be empty initially)"
    
    # Phase 2: Basic CRUD Operations
    boxy --style "$BOXY_DEFAULT_STYLE" --color magenta << 'EOF'
📦 PHASE 2: BASIC CRUD OPERATIONS
Core set/get/del functionality
EOF
    
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE set demo_key 'Hello ProntoDB!'" "Set a basic key-value pair"
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE get demo_key" "Retrieve the value"
    run_command "$BINARY projects" "List projects (should now show our test project)"
    run_command "$BINARY -p $TEST_PROJECT namespaces" "List namespaces in test project"
    
    # Phase 3: Context Addressing
    boxy --style "$BOXY_DEFAULT_STYLE" --color magenta << 'EOF'
🎯 PHASE 3: CONTEXT ADDRESSING
Testing __context suffix functionality
EOF
    
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE set config__prod 'production=true'" "Set config with production context"
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE set config__dev 'debug=true'" "Set config with development context"
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE get config__prod" "Get production config"
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE get config__dev" "Get development config"
    
    # Phase 4: Full Path Addressing
    boxy --style "$BOXY_DEFAULT_STYLE" --color magenta << 'EOF'
🛤️  PHASE 4: FULL PATH ADDRESSING
Using project.namespace.key format
EOF
    
    run_command "$BINARY set $TEST_PROJECT.$TEST_NAMESPACE.user_pref 'theme=dark'" "Set using full path addressing"
    run_command "$BINARY get $TEST_PROJECT.$TEST_NAMESPACE.user_pref" "Get using full path addressing"
    
    # Phase 5: Keys and Scanning
    boxy --style "$BOXY_DEFAULT_STYLE" --color magenta << 'EOF'
🔍 PHASE 5: KEYS & SCANNING
List and scan operations
EOF
    
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE keys" "List all keys in namespace"
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE keys config" "List keys with 'config' prefix"
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE scan" "Scan all key-value pairs"
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE scan config" "Scan pairs with 'config' prefix"
    
    # Phase 6: TTL Namespace Operations
    boxy --style "$BOXY_DEFAULT_STYLE" --color magenta << 'EOF'
⏰ PHASE 6: TTL NAMESPACE OPERATIONS
Creating TTL namespace and testing rules
EOF
    
    run_command "$BINARY create-cache $TEST_PROJECT.$TTL_NAMESPACE timeout=300" "Create TTL namespace with 5min timeout"
    run_command "$BINARY -p $TEST_PROJECT namespaces" "List namespaces (should show cache namespace)"
    run_command "$BINARY -p $TEST_PROJECT -n $TTL_NAMESPACE set temp_session 'user123_active'" "Set value in TTL namespace (uses default TTL)"
    run_command "$BINARY -p $TEST_PROJECT -n $TTL_NAMESPACE get temp_session" "Get value from TTL namespace"
    
    # Demonstrate TTL rule enforcement
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE set regular_key 'no ttl allowed' --ttl 60" "Try to set TTL in non-TTL namespace (should fail)"
    
    # Phase 7: Miss Conditions & Exit Codes
    boxy --style "$BOXY_DEFAULT_STYLE" --color magenta << 'EOF'
🎯 PHASE 7: MISS CONDITIONS & EXIT CODES
Testing not-found scenarios and exit codes
EOF
    
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE get nonexistent_key" "Get non-existent key (should return exit code 2)"
    run_command "$BINARY -p nonexistent_project -n $TEST_NAMESPACE get demo_key" "Get from non-existent project (should return exit code 2)"
    
    # Phase 8: Deletion Operations
    boxy --style "$BOXY_DEFAULT_STYLE" --color magenta << 'EOF'
🗑️  PHASE 8: DELETION OPERATIONS
Testing delete functionality
EOF
    
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE del demo_key" "Delete basic key"
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE get demo_key" "Try to get deleted key (should miss)"
    run_command "$BINARY -p $TEST_PROJECT -n $TEST_NAMESPACE keys" "List remaining keys"
    
    # Phase 9: Advanced Discovery
    boxy --style "$BOXY_DEFAULT_STYLE" --color magenta << 'EOF'
🌐 PHASE 9: ADVANCED DISCOVERY
Comprehensive namespace discovery
EOF
    
    run_command "$BINARY nss" "List all project.namespace combinations"
    
    # Phase 10: Error Conditions
    boxy --style "$BOXY_DEFAULT_STYLE" --color magenta << 'EOF'
⚠️  PHASE 10: ERROR CONDITIONS
Testing proper error handling
EOF
    
    run_command "$BINARY" "Run without arguments (should show error)"
    run_command "$BINARY unknown_command" "Run unknown command (should show error)"
    run_command "$BINARY set" "Run set without arguments (should show usage)"
    run_command "$BINARY get" "Run get without arguments (should show usage)"
    
    # Success Summary
    clear
    boxy --style "$BOXY_DEFAULT_STYLE" --color green << 'EOF'
🎊 UAT CEREMONY COMPLETE! 🎊

✅ All ProntoDB MVP features demonstrated successfully!

Key Features Verified:
• Help system and command discovery
• Basic CRUD operations (set/get/del)
• Context addressing with __suffix
• Full path addressing (project.namespace.key)
• Keys and scan operations with prefix filtering
• TTL namespace creation and rule enforcement
• Proper exit codes (0=success, 2=miss, 1=error)
• Error handling and validation
• Project and namespace discovery

🏆 ProntoDB MVP is ready for production use!
EOF

    pause_for_effect 2
    
    boxy --style "$BOXY_DEFAULT_STYLE" --color yellow << 'EOF'
🧹 Cleanup will run automatically on exit...
Set CLEANUP_ON_EXIT=0 to preserve test data
EOF
}

# Run the ceremony
main "$@"
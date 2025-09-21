#!/bin/bash
# ProntoDB Test Entry Point
# RSB-compliant unified interface for running all ProntoDB tests

set -e

# Configuration
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TEST_DIR="$ROOT_DIR/tests"

# ProntoDB binaries
PRONTODB="./target/debug/prontodb"
ADMIN_CLI="./target/debug/admin-cli"


# Parse optional flags (can be anywhere in arguments)
TEST_SLEEP=""
NO_SLEEP="false"
QUICK_MODE="true"  # Default to quick mode
COMPREHENSIVE_MODE="false"
BENCHMARK_MODE="false"
SNAP_BENCHMARKS="false"
ARGS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --sleep)
            TEST_SLEEP="$2"
            shift 2
            ;;
        --no-sleep)
            NO_SLEEP="true"
            shift 1
            ;;
        --quick)
            QUICK_MODE="true"
            COMPREHENSIVE_MODE="false"
            shift 1
            ;;
        --comprehensive|--full)
            QUICK_MODE="false"
            COMPREHENSIVE_MODE="true"
            shift 1
            ;;
        --benchmark)
            BENCHMARK_MODE="true"
            shift 1
            ;;
        --snap-benchmarks)
            BENCHMARK_MODE="true"
            SNAP_BENCHMARKS="true"
            shift 1
            ;;
        *)
            ARGS+=("$1")
            shift 1
            ;;
    esac
done

# Restore non-flag arguments
set -- "${ARGS[@]}"

# Available tests (RSB-compliant structure)
declare -A TESTS=(
    # Core RSB test categories
    ["sanity"]="sanity/run.sh"
    ["crud"]="crud_sanity.rs"
    ["rsb"]="rsb_cli.rs"

    # Future tests (when implemented)
    ["comprehensive"]="comprehensive/prontodb.rs"
    ["integration"]="integration/prontodb.rs"
    ["performance"]="performance/prontodb.rs"

    # Aliases for RSB compliance
    ["smoke"]="crud_sanity.rs"
    ["demo"]="sanity/run.sh"
    ["all"]="all.sh"
)

show_help() {
    echo "🗂️ PRONTODB TEST RUNNER (RSB-Compliant)"
    echo "========================================"
    echo
    echo "Available Commands:"
    echo "  test.sh [options] sanity              Run RSB sanity tests (all modules)"
    echo "  test.sh [options] crud                Run CRUD infrastructure tests"
    echo "  test.sh [options] rsb                 Run RSB CLI integration tests"
    echo "  test.sh [options] comprehensive       Run complete feature coverage tests"
    echo "  test.sh list                          List available tests"
    echo "  test.sh help                          Show this help"
    echo "  test.sh docs [topic]                  Show documentation for topic"
    echo ""
    echo "Options:"
    echo "  --comprehensive        Run full validation"
    echo "  --quick                Force quick mode (default)"
    echo "  --sleep N              Add sleep/timeout of N seconds between demo steps"
    echo "  --no-sleep             Disable all sleeps (default behavior)"
    echo "  --benchmark            Run performance benchmarks"
    echo ""
    echo "RSB-Compliant Test Categories:"
    echo "  sanity                 RSB feature sanity tests (all modules)"
    echo "  crud                   CRUD infrastructure validation"
    echo "  rsb                    RSB CLI integration tests"
    echo "  comprehensive          Complete feature coverage (when implemented)"
    echo ""
    echo "Current Implementation Status:"
    echo "  ✅ RSB CLI tests passing (3/3)"
    echo "  ✅ CRUD sanity tests passing (6/6)"
    echo "  📋 Next: Complete RSB sanity module integration"
    echo "  🎯 Goal: All test categories passing with proper RSB compliance"
}

list_tests() {
    echo "🗂️ PRONTODB AVAILABLE TESTS"
    echo "==========================="
    echo
    for test_name in $(printf "%s\n" "${!TESTS[@]}" | sort); do
        test_file="${TESTS[$test_name]}"
        if [[ -f "$TEST_DIR/$test_file" ]]; then
            echo "✅ $test_name → $test_file"
        else
            echo "❌ $test_name → $test_file (missing - foundation phase)"
        fi
    done
    echo
    echo "🔄 Implementation Status:"
    echo "   Foundation phase - test infrastructure being built"
    echo "   Use 'cargo test' for basic Rust tests"
    echo "   Use 'test.sh docs' for RSB documentation"
}

run_test() {
    local test_name="$1"

    if [[ -z "$test_name" ]]; then
        echo "❌ Error: Test name required"
        echo "Use: test.sh <test>"
        echo "Available tests: ${!TESTS[*]}"
        exit 1
    fi

    if [[ ! "${TESTS[$test_name]+exists}" ]]; then
        echo "❌ Error: Unknown test '$test_name'"
        echo "Available tests: ${!TESTS[*]}"
        exit 1
    fi

    local test_file="${TESTS[$test_name]}"
    local test_path="$TEST_DIR/$test_file"

    echo "🚀 Running ProntoDB test: $test_name"
    echo "===================================="
    echo

    # Change to project root
    cd "$ROOT_DIR"

    # For Rust tests, use cargo test
    if [[ "$test_file" == *.rs ]]; then
        if [[ ! -f "$test_path" ]]; then
            echo "❌ Test file not found: $test_path"
            echo "🔄 Foundation phase - tests are being implemented"
            echo "📋 Use 'cargo test' for available Rust tests"
            exit 1
        fi

        echo "🦀 Running Rust test: $test_file"
        if [[ "$test_name" == "crud" ]]; then
            cargo test --test crud_sanity
        elif [[ "$test_name" == "rsb" ]]; then
            cargo test --test rsb_cli
        elif [[ "$test_name" == "smoke" ]]; then
            cargo test --test crud_sanity
        else
            cargo test --test "$test_name"
        fi
    else
        # For shell scripts
        if [[ ! -f "$test_path" ]]; then
            echo "❌ Test file not found: $test_path"
            exit 1
        fi
        exec bash "$test_path"
    fi
}

show_docs() {
    local topic="${1:-prontodb}"

    echo "📚 PRONTODB DOCUMENTATION"
    echo "========================="
    echo

    case "$topic" in
        "prontodb"|"architecture")
            echo "🗂️ ProntoDB Architecture:"
            echo "  - Filesystem-first per-address keystore"
            echo "  - Generic CRUD ← ProntoDB Domain Adapter ← Admin CLI"
            echo "  - RSB-compliant string-biased API design"
            echo "  - Per-address SQLite isolation"
            echo
            echo "📁 Key Files:"
            echo "  - START.txt              ← 5-minute onboarding"
            echo "  - ADMIN_README.md        ← Admin CLI documentation"
            echo "  - src/lib/core/adpt.rs   ← ProntoDB adapter"
            echo "  - src/lib/adpt/sqlite/   ← Generic CRUD implementation"
            echo
            ;;
        "rsb")
            echo "🏗️ RSB Compliance Patterns:"
            echo "  - bootstrap! → options! → dispatch! patterns"
            echo "  - String-biased interfaces throughout"
            echo "  - Proper exit code handling"
            echo "  - test.sh as unified test entry point"
            echo
            ;;
        "tests")
            echo "🧪 Test Organization:"
            echo "  - tests/crud_sanity.rs   ← CRUD infrastructure tests"
            echo "  - tests/rsb_cli.rs       ← RSB CLI integration tests"
            echo "  - tests/sanity/          ← RSB feature sanity tests"
            echo "  - bin/test.sh            ← Unified test runner"
            echo
            ;;
        *)
            echo "Available topics: prontodb, rsb, tests, architecture"
            ;;
    esac
}

# Main command dispatch
case "${1:-help}" in
    "sanity"|"crud"|"rsb"|"comprehensive"|"smoke"|"demo"|"all")
        run_test "$1"
        ;;
    "list")
        list_tests
        ;;
    "docs")
        show_docs "$2"
        ;;
    "help"|"--help"|"-h")
        show_help
        ;;
    *)
        echo "❌ Unknown command: $1"
        echo "Use: test.sh help"
        exit 1
        ;;
esac

#!/bin/bash
# validate-docs.sh - Documentation validation for ProntoDB Meta Process v2
# Silent success, noisy failure validation
# Only output problems - hide successful validations

set -e

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Error tracking
ERROR_COUNT=0
WARNING_COUNT=0

# Function to report errors
error() {
    echo -e "${RED}❌ ERROR: $1${NC}" >&2
    ((ERROR_COUNT++))
}

# Function to report warnings
warn() {
    echo -e "${YELLOW}⚠️  WARNING: $1${NC}" >&2
    ((WARNING_COUNT++))
}

# Function to check if file exists
check_file() {
    local file="$1"
    local description="$2"

    if [[ ! -f "$file" ]]; then
        error "Missing $description: $file"
        return 1
    fi
    return 0
}

# Function to check file staleness
check_staleness() {
    local file="$1"
    local max_age_days="$2"
    local description="$3"

    if [[ ! -f "$file" ]]; then
        return 0  # Already reported as missing
    fi

    local file_age_days=$(( ($(date +%s) - $(stat -c %Y "$file")) / 86400 ))

    if [[ $file_age_days -gt $max_age_days ]]; then
        if [[ $max_age_days -le 7 ]]; then
            error "Critical doc stale: $description ($file) - $file_age_days days old (max: $max_age_days)"
        else
            warn "Doc may be stale: $description ($file) - $file_age_days days old (max: $max_age_days)"
        fi
    fi
}

# Function to check internal references
check_references() {
    local file="$1"

    if [[ ! -f "$file" ]]; then
        return 0  # Already reported as missing
    fi

    # Check for broken internal links (docs/ paths)
    while IFS= read -r line; do
        if [[ $line =~ docs/[a-zA-Z0-9_./]+ ]]; then
            local ref_path="${BASH_REMATCH[0]}"
            if [[ ! -f "$ref_path" && ! -d "$ref_path" ]]; then
                error "Broken reference in $file: $ref_path"
            fi
        fi
    done < "$file"
}

echo "🔍 Validating ProntoDB Meta Process v2 documentation structure..."

# Check core entry point
check_file "START.txt" "entry point"
check_staleness "START.txt" 30 "entry point"

# Check process documents
echo "📋 Checking process documents..."
check_file "docs/procs/PROCESS.txt" "master workflow guide"
check_file "docs/procs/CONTINUE.md" "session handoff"
check_file "docs/procs/QUICK_REF.txt" "quick reference"
check_file "docs/procs/SPRINT.txt" "current sprint"
check_file "docs/procs/ROADMAP.txt" "strategic roadmap"
check_file "docs/procs/TASKS.txt" "task breakdown"
check_file "docs/procs/DONE.txt" "completed work archive"

# Check staleness of critical process docs (1 week)
check_staleness "docs/procs/CONTINUE.md" 7 "session handoff"
check_staleness "docs/procs/QUICK_REF.txt" 7 "quick reference"
check_staleness "docs/procs/SPRINT.txt" 7 "current sprint"

# Check staleness of planning docs (1 month)
check_staleness "docs/procs/PROCESS.txt" 30 "master workflow"
check_staleness "docs/procs/ROADMAP.txt" 30 "strategic roadmap"
check_staleness "docs/procs/TASKS.txt" 30 "task breakdown"

# Check reference documents
echo "📚 Checking reference documents..."
check_file "docs/ref/RSB_LESSONS.md" "RSB lessons"
check_file "docs/ref/RSB_UPDATES.md" "RSB updates"

# Check for ProntoDB-specific docs
check_file "README.md" "project overview"
check_file "Cargo.toml" "project manifest"

# Check analysis results
echo "🔬 Checking analysis results..."
check_file ".analysis/consolidated_wisdom.txt" "consolidated wisdom"
if [[ ! -f ".analysis/technical_debt.txt" ]]; then
    warn "Missing technical debt analysis: .analysis/technical_debt.txt"
fi

# Check directory structure
echo "🗂️  Checking directory structure..."
for dir in "docs/procs" "docs/ref" "docs/misc" "docs/misc/archive" ".analysis"; do
    if [[ ! -d "$dir" ]]; then
        error "Missing directory: $dir"
    fi
done

# Check for old files in root that should have been moved
echo "🧹 Checking for unmigrated files..."
for file in "ROADMAP.txt" "TASKS.txt" "IDEAS.txt" "SESSION.md" "RSB_LESSONS.md" "RSB_UPDATES.md"; do
    if [[ -f "$file" ]]; then
        warn "File not migrated from root: $file (should be in docs/)"
    fi
done

# Check internal references
echo "🔗 Checking internal references..."
for file in "START.txt" "docs/procs/PROCESS.txt" "docs/procs/QUICK_REF.txt"; do
    if [[ -f "$file" ]]; then
        check_references "$file"
    fi
done

# Check for essential build tools
echo "🔧 Checking build tools..."
if ! command -v cargo &> /dev/null; then
    error "cargo not found - Rust builds will fail"
fi

# ProntoDB-specific checks
echo "🗃️  Checking ProntoDB specifics..."
if [[ ! -f "bin/deploy.sh" ]]; then
    warn "Missing deployment script: bin/deploy.sh"
fi

if [[ ! -f "bin/test.sh" ]]; then
    warn "Missing test script: bin/test.sh"
fi

# Check for test theater (basic check)
echo "🎭 Checking for potential test theater..."
if find tests/ -name "*.rs" -exec grep -l "println!\|hardcoded\|simulate\|fake" {} \; 2>/dev/null | head -1 > /dev/null; then
    warn "Potential test theater detected - run detailed audit"
fi

# Check git status
echo "📝 Checking git status..."
if ! git status --porcelain | grep -q .; then
    # Clean working directory
    true
else
    warn "Uncommitted changes in working directory"
fi

# Final summary
echo ""
if [[ $ERROR_COUNT -eq 0 && $WARNING_COUNT -eq 0 ]]; then
    echo -e "${GREEN}✅ ProntoDB documentation validation passed - all systems healthy!${NC}"
    exit 0
elif [[ $ERROR_COUNT -eq 0 ]]; then
    echo -e "${YELLOW}⚠️  ProntoDB documentation validation completed with $WARNING_COUNT warning(s)${NC}"
    exit 0
else
    echo -e "${RED}❌ ProntoDB documentation validation FAILED with $ERROR_COUNT error(s) and $WARNING_COUNT warning(s)${NC}"
    exit 1
fi
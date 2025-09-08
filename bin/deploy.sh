#!/bin/bash
set -e

# ProntoDB Deploy Script - Rust binary with lib-to-bin pattern
# Deploys prontodb binary to ~/.local/lib/odx/prontodb/ and creates bin symlink

# Configuration
LIB_DIR="$HOME/.local/lib/odx/prontodb"
BIN_DIR="$HOME/.local/bin/odx"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BINARY_NAME="prontodb"
DEPLOYABLE="${BINARY_NAME}"

# Extract version from Cargo.toml at repo root
VERSION=$(grep '^version' "$ROOT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)

# Check boxy availability
has_boxy() {
    command -v boxy >/dev/null 2>&1
}

# Ceremonial messaging
ceremony_msg() {
    local msg="$1"
    local theme="${2:-info}"
    if has_boxy; then
        echo "$msg" | boxy --theme "$theme" --width max
    else
        echo "$msg"
    fi
}

step_msg() {
    local step="$1"
    local desc="$2"
    if has_boxy; then
        printf "%s\n%s\n" "$step" "• $desc" | boxy --style rounded --width max --title "📦 Deploy Step"
    else
        echo "$step: $desc"
    fi
}

# Welcome ceremony
ceremony_msg "🗄️ PRONTODB DEPLOYMENT CEREMONY v$VERSION" "success"
echo

step_msg "Step 1" "Building prontodb v$VERSION with RSB integration..."
cd "$ROOT_DIR"
if ! cargo build --release; then
    ceremony_msg "❌ Build failed!" "error"
    exit 1
fi

# Check if binary was created
if [ ! -f "$ROOT_DIR/target/release/${DEPLOYABLE}" ]; then
    ceremony_msg "❌ Binary not found at target/release/${DEPLOYABLE}" "error"
    exit 1
fi

step_msg "Step 2" "Creating lib directory: $LIB_DIR"
mkdir -p "$LIB_DIR"

step_msg "Step 3" "Deploying binary to lib directory..."
if ! cp "$ROOT_DIR/target/release/${DEPLOYABLE}" "$LIB_DIR/$BINARY_NAME"; then
    ceremony_msg "❌ Failed to copy binary to $LIB_DIR" "error"
    exit 1
fi

if ! chmod +x "$LIB_DIR/$BINARY_NAME"; then
    ceremony_msg "❌ Failed to make binary executable" "error"
    exit 1
fi

step_msg "Step 4" "Creating bin directory: $BIN_DIR"
mkdir -p "$BIN_DIR"

step_msg "Step 5" "Creating bin symlink: $BIN_DIR/$BINARY_NAME → $LIB_DIR/$BINARY_NAME"
if [[ -L "$BIN_DIR/$BINARY_NAME" ]] || [[ -f "$BIN_DIR/$BINARY_NAME" ]]; then
    rm "$BIN_DIR/$BINARY_NAME"
fi

if ! ln -s "$LIB_DIR/$BINARY_NAME" "$BIN_DIR/$BINARY_NAME"; then
    ceremony_msg "❌ Failed to create symlink" "error"
    exit 1
fi

step_msg "Step 6" "Verifying deployment..."
if [[ ! -x "$LIB_DIR/$BINARY_NAME" ]]; then
    ceremony_msg "❌ Binary is not executable at $LIB_DIR/$BINARY_NAME" "error"
    exit 1
fi

if [[ ! -L "$BIN_DIR/$BINARY_NAME" ]]; then
    ceremony_msg "❌ Symlink not created at $BIN_DIR/$BINARY_NAME" "error"
    exit 1
fi

step_msg "Step 7" "Testing prontodb command..."
if ! "$BIN_DIR/$BINARY_NAME" help >/dev/null 2>&1; then
    ceremony_msg "❌ ProntoDB command test failed!" "error"
    exit 1
fi

# Success ceremony
ceremony_msg "✅ PRONTODB v$VERSION DEPLOYED SUCCESSFULLY!" "success"
echo

if has_boxy; then
    {
        echo "🗄️ Fast key-value store with TTL support"
        echo "📍 Library: $LIB_DIR/$BINARY_NAME"
        echo "📍 Binary: $BIN_DIR/$BINARY_NAME"
        echo
        echo "💡 Usage Examples:"
        echo "   prontodb -p myapp -n config set debug true"
        echo "   prontodb get myapp.config.debug"
        echo "   prontodb create-cache myapp.sessions timeout=3600"
        echo "   prontodb projects                         # Discovery"
        echo "   prontodb help                             # Full command reference"
        echo
        echo "🎭 Features:"
        echo "   • Namespaced storage with project.namespace addressing"
        echo "   • TTL namespace support for caching"
        echo "   • Context addressing (key__context)"
        echo "   • RSB integration with lifecycle commands"
        echo "   • XDG Base Directory compliance"
    } | boxy --theme success --header "🗄️ ProntoDB v$VERSION Deployed" \
             --status "sr:$(date '+%H:%M:%S')" \
             --footer "✅ Ready for production use" \
             --width max
else
    echo "📍 Library location: $LIB_DIR/$BINARY_NAME"
    echo "📍 Binary symlink: $BIN_DIR/$BINARY_NAME"
    echo
    echo "💡 Usage Examples:"
    echo "   prontodb -p myapp -n config set debug true"
    echo "   prontodb get myapp.config.debug"
    echo "   prontodb create-cache myapp.sessions timeout=3600"
    echo "   prontodb projects                         # Discovery"
    echo "   prontodb help                             # Full command reference"
fi

echo
step_msg "🧪 Quick Test" "Running comprehensive functionality test"

# Test basic functionality
TEST_PROJECT="deploy_test"
TEST_NAMESPACE="verification"
TEST_KEY="deploy_check"
TEST_VALUE="$(date '+%Y-%m-%d_%H:%M:%S')"

echo "Testing set operation..."
if "$BIN_DIR/$BINARY_NAME" -p "$TEST_PROJECT" -n "$TEST_NAMESPACE" set "$TEST_KEY" "$TEST_VALUE"; then
    echo "✅ Set operation successful"
else
    ceremony_msg "❌ Set operation failed!" "error"
    exit 1
fi

echo "Testing get operation..."
if RESULT=$("$BIN_DIR/$BINARY_NAME" -p "$TEST_PROJECT" -n "$TEST_NAMESPACE" get "$TEST_KEY") && [[ "$RESULT" == "$TEST_VALUE" ]]; then
    echo "✅ Get operation successful: $RESULT"
else
    ceremony_msg "❌ Get operation failed!" "error"
    exit 1
fi

echo "Testing delete operation..."
if "$BIN_DIR/$BINARY_NAME" -p "$TEST_PROJECT" -n "$TEST_NAMESPACE" del "$TEST_KEY"; then
    echo "✅ Delete operation successful"
else
    ceremony_msg "❌ Delete operation failed!" "error"
    exit 1
fi

echo "Testing discovery operations..."
if "$BIN_DIR/$BINARY_NAME" projects >/dev/null 2>&1; then
    echo "✅ Discovery operations functional"
else
    ceremony_msg "❌ Discovery operations failed!" "error"
    exit 1
fi

# Final ceremony
ceremony_msg "🎉 PRONTODB v$VERSION READY FOR USE!" "success"

if has_boxy; then
    {
        echo "Run the UAT ceremony:"
        echo "   cd $ROOT_DIR && ./bin/uat.sh"
        echo
        echo "Or test immediately:"
        echo "   $BIN_DIR/$BINARY_NAME help"
    } | boxy --theme info --header "🚀 Next Steps"
fi
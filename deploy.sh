#!/bin/bash
# ProntoDB Production Deployment Script
# Version: 0.3.0

set -e  # Exit on error

echo "🚀 ProntoDB Production Deployment"
echo "================================="

# Build release version
echo "📦 Building release version..."
cargo build --release --quiet

# Check if build succeeded
if [ ! -f "target/release/prontodb" ]; then
    echo "❌ Build failed - release binary not found"
    exit 1
fi

echo "✅ Release build successful"

# Get version info
VERSION=$(./target/release/prontodb --version)
echo "📋 Version: $VERSION"

# Create deployment directory
DEPLOY_DIR="$HOME/.local/bin"
mkdir -p "$DEPLOY_DIR"

# Deploy binary
echo "📋 Deploying to $DEPLOY_DIR/prontodb..."
cp target/release/prontodb "$DEPLOY_DIR/prontodb"
chmod +x "$DEPLOY_DIR/prontodb"

# Verify deployment
if command -v prontodb &> /dev/null; then
    DEPLOYED_VERSION=$(prontodb --version 2>/dev/null || echo "Version check failed")
    echo "✅ Deployment successful: $DEPLOYED_VERSION"
else
    echo "⚠️  Warning: prontodb not in PATH. Add $DEPLOY_DIR to PATH:"
    echo "   export PATH=\"$DEPLOY_DIR:\$PATH\""
fi

echo ""
echo "🎯 Deployment Complete!"
echo "======================="
echo "Binary: $DEPLOY_DIR/prontodb"
echo "Version: $VERSION"
echo "Ready for production use!"
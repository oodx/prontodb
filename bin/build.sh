#!/bin/bash
# ProntoDB Build Script with Feature Combinations
# Supports minimal and full streaming builds

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Build configuration
BUILD_TYPE="${1:-minimal}"
BUILD_MODE="${2:-debug}"

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     ProntoDB Build System v0.6.2+                  ${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""

case "$BUILD_TYPE" in
    minimal|min)
        echo -e "${YELLOW}🔧 Building MINIMAL configuration (no streaming)...${NC}"
        if [ "$BUILD_MODE" = "release" ]; then
            cargo build --release
        else
            cargo build
        fi
        echo -e "${GREEN}✅ Minimal build complete${NC}"
        echo -e "${YELLOW}   Features: JSON, SQLite${NC}"
        echo -e "${YELLOW}   No XStream integration${NC}"
        ;;
        
    full|streaming)
        echo -e "${YELLOW}⚡ Building FULL configuration (with XStream)...${NC}"
        if [ "$BUILD_MODE" = "release" ]; then
            cargo build --release --features streaming
        else
            cargo build --features streaming
        fi
        echo -e "${GREEN}✅ Full build complete with XStream${NC}"
        echo -e "${YELLOW}   Features: JSON, SQLite, XStream${NC}"
        echo -e "${YELLOW}   Stream command enabled${NC}"
        ;;
        
    test)
        echo -e "${YELLOW}🧪 Building and testing all configurations...${NC}"
        
        # Test minimal build
        echo -e "${BLUE}Testing minimal build...${NC}"
        cargo build
        cargo test
        
        # Test full build
        echo -e "${BLUE}Testing full streaming build...${NC}"
        cargo build --features streaming
        cargo test --features streaming
        
        echo -e "${GREEN}✅ All configurations tested successfully${NC}"
        ;;
        
    clean)
        echo -e "${YELLOW}🧹 Cleaning build artifacts...${NC}"
        cargo clean
        echo -e "${GREEN}✅ Build directory cleaned${NC}"
        ;;
        
    *)
        echo -e "${RED}❌ Unknown build type: $BUILD_TYPE${NC}"
        echo ""
        echo "Usage: $0 [build-type] [build-mode]"
        echo ""
        echo "Build types:"
        echo "  minimal|min   - Build without streaming features (default)"
        echo "  full|streaming - Build with XStream integration"
        echo "  test          - Test all build configurations"
        echo "  clean         - Clean build artifacts"
        echo ""
        echo "Build modes:"
        echo "  debug         - Debug build (default)"
        echo "  release       - Release build with optimizations"
        echo ""
        echo "Examples:"
        echo "  $0              # Minimal debug build"
        echo "  $0 full         # Full debug build with streaming"
        echo "  $0 full release # Full release build with streaming"
        echo "  $0 test         # Test all configurations"
        exit 1
        ;;
esac

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"

# Show binary location
if [ "$BUILD_MODE" = "release" ]; then
    BINARY_PATH="target/release/prontodb"
else
    BINARY_PATH="target/debug/prontodb"
fi

if [ -f "$BINARY_PATH" ]; then
    echo -e "${GREEN}📦 Binary location: $BINARY_PATH${NC}"
    SIZE=$(du -h "$BINARY_PATH" | cut -f1)
    echo -e "${GREEN}📊 Binary size: $SIZE${NC}"
    
    # Check if streaming is enabled
    if $BINARY_PATH stream 2>&1 | grep -q "Streaming feature not enabled"; then
        echo -e "${YELLOW}🔒 Streaming: DISABLED (minimal build)${NC}"
    else
        echo -e "${GREEN}⚡ Streaming: ENABLED (full build)${NC}"
    fi
fi

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
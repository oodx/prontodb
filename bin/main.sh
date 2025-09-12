#!/bin/bash

# ProntoDB Wrapper Script
# Handles build/clean commands and forwards everything else to release binary

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RELEASE_BINARY="$PROJECT_ROOT/target/release/prontodb"

# Change to project root for all operations
cd "$PROJECT_ROOT"

# Handle wrapper-specific commands
case "${1:-}" in
    "build")
        echo "Building ProntoDB release..."
        cargo build --release
        echo "✓ Release build complete: $RELEASE_BINARY"
        exit 0
        ;;
    
    "clean")
        echo "Cleaning target directory..."
        if [ -d "target" ]; then
            rm -rf target
            echo "✓ Target directory nuked"
        else
            echo "✓ Target directory already clean"
        fi
        exit 0
        ;;
    
    "")
        # No command provided - check if binary exists
        if [ ! -f "$RELEASE_BINARY" ]; then
            echo "Error: Release binary not found at: $RELEASE_BINARY"
            echo "Run './bin/main.sh build' to build the release binary first"
            exit 1
        fi
        
        # Forward to binary with no args (will show help)
        exec "$RELEASE_BINARY"
        ;;
    
    *)
        # All other commands - forward to release binary
        if [ ! -f "$RELEASE_BINARY" ]; then
            echo "Error: Release binary not found at: $RELEASE_BINARY" 
            echo "Run './bin/main.sh build' to build the release binary first"
            exit 1
        fi
        
        # Forward all arguments to the release binary
        exec "$RELEASE_BINARY" "$@"
        ;;
esac
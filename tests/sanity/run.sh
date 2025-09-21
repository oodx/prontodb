#!/bin/bash
# Sanity Test Runner for ProntoDB
# Tests basic hub dependencies and RSB functionality

set -e

cd "$(dirname "$0")/../.."

echo "🧪 Running Hub Dependency Sanity Tests"
echo "====================================="

echo "📦 Testing hub data functionality..."
cargo test --test hub_data_ext --quiet || {
    echo "❌ Hub data tests failed"
    exit 1
}

echo "🔧 Testing hub error functionality..."
cargo test --test hub_error_ext --quiet || {
    echo "❌ Hub error tests failed"
    exit 1
}

echo "🎨 Testing RSB colors functionality..."
cargo test --test rsb_colors --quiet || {
    echo "❌ RSB colors tests failed"
    exit 1
}

echo "⚙️ Testing RSB options functionality..."
cargo test --test rsb_options --quiet || {
    echo "❌ RSB options tests failed"
    exit 1
}

echo "📁 Testing RSB filesystem functionality..."
cargo test --test rsb_fs --quiet || {
    echo "❌ RSB fs tests failed"
    exit 1
}

echo "✅ All sanity tests passed!"
echo "Hub dependencies and RSB framework are working correctly."
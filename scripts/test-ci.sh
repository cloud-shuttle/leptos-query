#!/bin/bash

# Test CI workflows locally
# This script runs the same checks as our GitHub Actions CI

set -e

echo "🧪 Testing CI workflows locally..."
echo "=================================="

# Check formatting
echo "📝 Checking code formatting..."
cargo fmt --all -- --check
echo "✅ Code formatting OK"

# Run clippy
echo "🔍 Running clippy..."
cargo clippy --all-features -- -D warnings
echo "✅ Clippy OK"

# Check code without building
echo "🔧 Checking code without building..."
cargo check --all-features
echo "✅ Code check OK"

# Run tests
echo "🧪 Running tests..."
cargo test --all-features --verbose
echo "✅ Tests OK"

# Run integration tests
echo "🔗 Running integration tests..."
cargo test --test integration_tests --all-features --verbose
echo "✅ Integration tests OK"

# Check examples
echo "📚 Checking examples..."
cargo check --examples --all-features
echo "✅ Examples OK"

# Security audit
echo "🔒 Running security audit..."
if command -v cargo-audit &> /dev/null; then
    cargo audit
    echo "✅ Security audit OK"
else
    echo "⚠️  cargo-audit not installed, skipping security audit"
    echo "   Install with: cargo install cargo-audit"
fi

# Build for different features
echo "🏗️  Building with different features..."
for feature in csr ssr leptos-0-6 leptos-0-8; do
    echo "   Building with feature: $feature"
    cargo build --features $feature --verbose
done
echo "✅ Feature builds OK"

echo ""
echo "🎉 All CI checks passed locally!"
echo "Your code is ready for the CI pipeline!"

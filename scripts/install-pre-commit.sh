#!/bin/bash

# Pre-commit hooks installation script for leptos-query
# This script installs and configures pre-commit hooks

set -e

echo "🔧 Installing pre-commit hooks for leptos-query..."

# Check if pre-commit is installed
if ! command -v pre-commit &> /dev/null; then
    echo "📦 Installing pre-commit..."
    
    # Try different installation methods
    if command -v pip &> /dev/null; then
        pip install pre-commit
    elif command -v pip3 &> /dev/null; then
        pip3 install pre-commit
    elif command -v brew &> /dev/null; then
        brew install pre-commit
    elif command -v conda &> /dev/null; then
        conda install -c conda-forge pre-commit
    else
        echo "❌ Could not install pre-commit. Please install it manually:"
        echo "   pip install pre-commit"
        echo "   or"
        echo "   brew install pre-commit"
        exit 1
    fi
else
    echo "✅ pre-commit is already installed"
fi

# Check if cargo-audit is installed
if ! command -v cargo-audit &> /dev/null; then
    echo "📦 Installing cargo-audit..."
    cargo install cargo-audit
else
    echo "✅ cargo-audit is already installed"
fi

# Check if markdownlint is installed
if ! command -v markdownlint &> /dev/null; then
    echo "📦 Installing markdownlint..."
    if command -v npm &> /dev/null; then
        npm install -g markdownlint-cli
    else
        echo "⚠️  npm not found. Please install markdownlint manually:"
        echo "   npm install -g markdownlint-cli"
    fi
else
    echo "✅ markdownlint is already installed"
fi

# Install pre-commit hooks
echo "🔗 Installing pre-commit hooks..."
pre-commit install

# Install pre-commit hooks for all branches
echo "🔗 Installing pre-commit hooks for all branches..."
pre-commit install --hook-type pre-push

# Run pre-commit on all files to test
echo "🧪 Testing pre-commit hooks..."
pre-commit run --all-files

echo "✅ Pre-commit hooks installed successfully!"
echo ""
echo "📋 What was installed:"
echo "   • Rust formatting (rustfmt)"
echo "   • Rust linting (clippy)"
echo "   • Rust compilation check (cargo check)"
echo "   • Rust tests (cargo test)"
echo "   • Security audit (cargo audit)"
echo "   • File formatting (trailing whitespace, end-of-file)"
echo "   • YAML/JSON/TOML validation"
echo "   • Markdown linting"
echo "   • Large file detection"
echo "   • Merge conflict detection"
echo "   • Secret detection"
echo ""
echo "🚀 Pre-commit hooks will now run automatically on every commit!"
echo "   To run manually: pre-commit run --all-files"
echo "   To skip hooks: git commit --no-verify"

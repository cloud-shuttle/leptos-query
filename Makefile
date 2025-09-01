# leptos-query Makefile
# Common development tasks for the leptos-query library

.PHONY: help build test check clean doc release install publish

# Default target
help:
	@echo "leptos-query Development Commands:"
	@echo ""
	@echo "Build Commands:"
	@echo "  build          Build the library in release mode"
	@echo "  build-dev      Build the library in debug mode"
	@echo "  check          Check code without building"
	@echo "  clean          Clean build artifacts"
	@echo ""
	@echo "Testing Commands:"
	@echo "  test           Run all tests"
	@echo "  test-lib       Run library tests only"
	@echo "  test-integration Run integration tests only"
	@echo "  test-examples  Test all examples"
	@echo ""
	@echo "Documentation:"
	@echo "  doc            Generate documentation"
	@echo "  doc-open       Generate and open documentation"
	@echo ""
	@echo "Quality Checks:"
@echo "  fmt            Format code with rustfmt"
@echo "  fmt-check      Check code formatting"
@echo "  clippy         Run clippy linter"
@echo "  audit          Run cargo audit for security"
@echo "  ci             Run basic CI checks"
@echo "  ci-local       Run full CI checks locally"
	@echo ""
	@echo "Release Commands:"
	@echo "  release        Build release version"
	@echo "  package        Create package for distribution"
	@echo "  publish        Publish to crates.io (requires login)"
	@echo ""
	@echo "Development:"
	@echo "  install        Install dependencies"
	@echo "  watch          Watch for changes and run tests"
	@echo "  example-basic  Run the basic usage example"

# Build commands
build:
	cargo build --release

build-dev:
	cargo build

check:
	cargo check --all-features

clean:
	cargo clean
	rm -rf target/

# Testing commands
test:
	cargo test --all-features

test-lib:
	cargo test --lib

test-integration:
	cargo test --test integration_tests

test-examples:
	cargo test --examples

# Documentation
doc:
	cargo doc --all-features --no-deps

doc-open:
	cargo doc --all-features --no-deps --open

# Quality checks
fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

clippy:
	cargo clippy --all-features -- -D warnings

audit:
	cargo audit

# Release commands
release: clean build test clippy audit
	@echo "Release build completed successfully!"

package:
	cargo package --list

publish: release
	cargo publish

# Development commands
install:
	cargo install cargo-watch
	cargo install cargo-audit

watch:
	cargo watch -x check -x test -x clippy

example-basic:
	cargo run --example basic_usage

# Feature-specific builds
build-csr:
	cargo build --features csr

build-ssr:
	cargo build --features ssr

build-leptos-08:
	cargo build --features leptos-0-8

# CI/CD helpers
ci: fmt-check clippy test audit
	@echo "CI checks passed!"

ci-local:
	@echo "Running local CI checks..."
	@./scripts/test-ci.sh

# Development setup
setup: install
	@echo "Development environment setup complete!"
	@echo "Run 'make help' to see available commands"

# Quick development cycle
dev: fmt clippy test
	@echo "Development cycle complete!"

# Clean everything including git
clean-all: clean
	git clean -fdx

# Show project info
info:
	@echo "leptos-query Project Information:"
	@echo "Version: $(shell grep '^version =' Cargo.toml | cut -d'"' -f2)"
	@echo "Rust Edition: $(shell grep '^edition =' Cargo.toml | cut -d'"' -f2)"
	@echo "License: $(shell grep '^license =' Cargo.toml | cut -d'"' -f2)"
	@echo "Repository: $(shell grep '^repository =' Cargo.toml | cut -d'"' -f2)"

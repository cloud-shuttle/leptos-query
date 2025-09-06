# Leptos Query - Development Makefile
# Supports both Nix and regular environments

.PHONY: help install build test bench clean docs demo e2e wasm dev check format lint audit pre-commit install-pre-commit

# Default target
help:
	@echo "🚀 Leptos Query Development Commands"
	@echo ""
	@echo "📦 Setup & Installation:"
	@echo "  install     - Install dependencies (pnpm + Rust)"
	@echo "  nix-shell   - Enter Nix development shell"
	@echo ""
	@echo "🔨 Build & Development:"
	@echo "  build       - Build the library"
	@echo "  dev         - Start development server"
	@echo "  demo        - Build demo application"
	@echo "  wasm        - Build WASM package"
	@echo ""
	@echo "🧪 Testing & Quality:"
	@echo "  test        - Run Rust tests"
	@echo "  e2e         - Run Playwright E2E tests"
	@echo "  bench       - Run benchmarks"
	@echo "  check       - Run all checks (test + bench + e2e)"
	@echo "  format      - Format code with rustfmt"
	@echo "  lint        - Run clippy linting"
	@echo "  audit       - Security audit"
	@echo "  pre-commit  - Run pre-commit hooks"
	@echo "  install-pre-commit - Install pre-commit hooks"
	@echo ""
	@echo "📚 Documentation:"
	@echo "  docs        - Build documentation"
	@echo "  docs-serve  - Serve documentation locally"
	@echo ""
	@echo "🧹 Maintenance:"
	@echo "  clean       - Clean build artifacts"
	@echo "  distclean   - Deep clean (including node_modules)"

# Check if we're in a Nix environment
NIX_ENV := $(shell if command -v nix >/dev/null 2>&1 && nix flake show >/dev/null 2>&1; then echo "yes"; else echo "no"; fi)

# Setup and installation
install:
	@echo "📦 Installing dependencies..."
	@if [ "$(NIX_ENV)" = "yes" ]; then \
		echo "🔧 Using Nix environment..."; \
		nix develop --command echo "Nix environment ready"; \
	else \
		echo "🔧 Installing Rust toolchain..."; \
		rustup default stable; \
		rustup target add wasm32-unknown-unknown; \
		echo "🔧 Installing Trunk..."; \
		cargo install trunk; \
		echo "🔧 Installing wasm-pack..."; \
		cargo install wasm-pack; \
		echo "🔧 Installing pnpm..."; \
		if ! command -v pnpm >/dev/null 2>&1; then \
			npm install -g pnpm; \
		fi; \
		echo "🔧 Installing Playwright..."; \
		cd demo && pnpm install && pnpm exec playwright install; \
	fi

nix-shell:
	@if [ "$(NIX_ENV)" = "yes" ]; then \
		echo "🔧 Entering Nix development shell..."; \
		nix develop; \
	else \
		echo "❌ Nix flake not available. Run 'make install' instead."; \
		exit 1; \
	fi

# Build commands
build:
	@echo "🔨 Building library..."
	cargo build --all-features --release

demo:
	@echo "🎨 Building demo application..."
	cd demo && trunk build

wasm:
	@echo "🌐 Building WASM package..."
	wasm-pack build --target web --out-dir dist

# Testing commands
test:
	@echo "🧪 Running Rust tests..."
	cargo test --all-features --release

e2e:
	@echo "🌐 Running Playwright E2E tests..."
	@if [ -d "demo" ]; then \
		cd demo && pnpm test:e2e; \
	else \
		echo "❌ Demo directory not found. Run 'make demo' first."; \
		exit 1; \
	fi

bench:
	@echo "📊 Running benchmarks..."
	cargo bench --all-features

check: test bench e2e
	@echo "✅ All checks passed!"

# Code quality commands
format:
	@echo "🎨 Formatting code..."
	cargo fmt --all

lint:
	@echo "🔍 Running clippy..."
	cargo clippy --all-features -- -D warnings

audit:
	@echo "🔒 Security audit..."
	cargo audit

# Pre-commit hooks
install-pre-commit:
	@echo "🔧 Installing pre-commit hooks..."
	./scripts/install-pre-commit.sh

pre-commit:
	@echo "🔍 Running pre-commit hooks..."
	pre-commit run --all-files

# Documentation commands
docs:
	@echo "📚 Building documentation..."
	cargo doc --all-features --no-deps --open

docs-serve:
	@echo "📚 Serving documentation..."
	cargo doc --all-features --no-deps
	@echo "📖 Documentation available at: file://$(PWD)/target/doc/leptos_query_rs/index.html"

# Development server
dev:
	@echo "🚀 Starting development server..."
	cd demo && trunk serve

# Cleanup commands
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@if [ -d "demo" ]; then \
		cd demo && rm -rf dist target; \
	fi
	rm -rf dist target

distclean: clean
	@echo "🧹 Deep cleaning..."
	@if [ -d "demo" ]; then \
		cd demo && rm -rf node_modules pnpm-lock.yaml; \
	fi
	rm -rf .cargo .rustup

# CI/CD commands
ci: format lint test bench e2e audit
	@echo "✅ CI pipeline completed successfully!"

# Release commands
release-check: format lint test bench e2e audit
	@echo "✅ Release checks passed!"
	@echo "🚀 Ready for release!"

# Nix-specific commands
nix-build:
	@if [ "$(NIX_ENV)" = "yes" ]; then \
		echo "🔧 Building with Nix..."; \
		nix build .; \
	else \
		echo "❌ Nix flake not available."; \
		exit 1; \
	fi

nix-test:
	@if [ "$(NIX_ENV)" = "yes" ]; then \
		echo "🔧 Testing with Nix..."; \
		nix run .#test; \
	else \
		echo "❌ Nix flake not available."; \
		exit 1; \
	fi

# Development workflow
dev-setup: install build demo
	@echo "🚀 Development environment ready!"
	@echo "Run 'make dev' to start the development server"

# Quick development cycle
dev-cycle: format lint test
	@echo "🔄 Development cycle completed!"

# Show environment info
env-info:
	@echo "🔍 Environment Information:"
	@echo "  Rust: $(shell rustc --version 2>/dev/null || echo 'Not installed')"
	@echo "  Cargo: $(shell cargo --version 2>/dev/null || echo 'Not installed')"
	@echo "  Node: $(shell node --version 2>/dev/null || echo 'Not installed')"
	@echo "  pnpm: $(shell pnpm --version 2>/dev/null || echo 'Not installed')"
	@echo "  Trunk: $(shell trunk --version 2>/dev/null || echo 'Not installed')"
	@echo "  Nix: $(shell if [ "$(NIX_ENV)" = "yes" ]; then echo "Available"; else echo "Not available"; fi)"

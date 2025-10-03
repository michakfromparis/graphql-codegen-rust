# Makefile for GraphQL Rust Codegen
# Industry-standard development workflow automation

# Default target - run full development workflow
.PHONY: all
all: setup fmt lint test doc

# Development workflow - run everything needed for active development
.PHONY: dev
dev: setup fmt lint test doc

# Setup and dependencies
.PHONY: setup
setup: check-tools install-deps

# Check if required tools are installed
.PHONY: check-tools
check-tools:
	@echo "Checking required tools..."
	@cargo --version >/dev/null 2>&1 || (echo "❌ Cargo not found. Install Rust from https://rustup.rs/" && exit 1)
	@rustc --version >/dev/null 2>&1 || (echo "❌ Rust compiler not found" && exit 1)
	@echo "✅ All required tools are installed"

# Install development dependencies
.PHONY: install-deps
install-deps:
	@echo "Installing development dependencies..."
	cargo install cargo-audit --version 0.21.2 || echo "cargo-audit already installed"
	cargo install cargo-deny --version 0.18.3 || echo "cargo-deny already installed"
	cargo install cargo-llvm-cov || echo "cargo-llvm-cov already installed"
	cargo install cargo-bump --version 1.1.0 || echo "cargo-bump already installed"
	cargo install cargo-edit || echo "cargo-edit already installed"
	cargo install git-cliff --version 2.10.1 || echo "git-cliff already installed"
	@echo "✅ Development dependencies installed"

# Code formatting
.PHONY: fmt
fmt:
	@echo "Formatting code..."
	cargo fmt --all
	@echo "✅ Code formatted"

# Check formatting (CI)
.PHONY: fmt-check
fmt-check:
	@echo "Checking code formatting..."
	cargo fmt --all --check
	@echo "✅ Code formatting is correct"

# Linting
.PHONY: lint
lint:
	@echo "Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "✅ Clippy passed"

# Testing
.PHONY: test
test:
	@echo "Running tests..."
	cargo test --all-features
	@echo "✅ Tests passed"

.PHONY: test-watch
test-watch:
	@echo "Running tests in watch mode..."
	cargo watch -x test

# Documentation
.PHONY: doc
doc:
	@echo "Building documentation..."
	cargo doc --all-features --open --no-deps
	@echo "✅ Documentation built"

# Security auditing
.PHONY: audit
audit:
	@echo "Running security audit..."
	cargo audit --ignore RUSTSEC-2023-0071
	@echo "✅ Security audit passed"

# License compliance
.PHONY: license-check
license-check:
	@echo "Checking license compliance..."
	cargo deny check licenses
	cargo deny check bans
	cargo deny check advisories
	@echo "✅ License compliance checks passed"

# Code coverage
.PHONY: coverage
coverage:
	@echo "Generating code coverage..."
	cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
	@echo "✅ Coverage report generated: lcov.info"

# Building
.PHONY: build
build:
	@echo "Building project..."
	cargo build
	@echo "✅ Build successful"

.PHONY: build-release
build-release:
	@echo "Building release..."
	cargo build --release
	@echo "✅ Release build successful"

# Dependencies
.PHONY: update
update:
	@echo "Updating dependencies within current version ranges..."
	cargo update
	@echo "✅ Dependencies updated"

.PHONY: upgrade
upgrade:
	@echo "🚀 Upgrading dependencies to latest compatible versions..."
	@echo "This will modify Cargo.toml and Cargo.lock"
	@read -p "Continue? (y/N): " confirm && [[ $$confirm =~ ^[Yy]$$ ]] || (echo "Upgrade cancelled." && exit 1)
	cargo upgrade --compatible
	cargo update
	@echo "✅ Dependencies upgraded - running tests..."
	@if make test; then \
		echo "✅ All tests passed after upgrade!"; \
	else \
		echo "❌ Tests failed after upgrade. You may need to fix compatibility issues."; \
		echo "To revert: git checkout HEAD -- Cargo.toml Cargo.lock"; \
		exit 1; \
	fi

.PHONY: upgrade-major
upgrade-major:
	@echo "⚠️  Upgrading dependencies to latest versions (including breaking changes)..."
	@echo "This may introduce breaking changes!"
	@read -p "Continue? (y/N): " confirm && [[ $$confirm =~ ^[Yy]$$ ]] || (echo "Upgrade cancelled." && exit 1)
	cargo upgrade --incompatible
	cargo update
	@echo "✅ Dependencies upgraded - running tests..."
	@if make test; then \
		echo "✅ All tests passed after major upgrade!"; \
	else \
		echo "❌ Tests failed after upgrade. You may need to update code for breaking changes."; \
		echo "To revert: git checkout HEAD -- Cargo.toml Cargo.lock"; \
		exit 1; \
	fi

.PHONY: outdated
outdated:
	@echo "Checking for outdated dependencies..."
	cargo outdated || echo "Install cargo-outdated: cargo install cargo-outdated"
	@echo "✅ Outdated check complete"

# Clean up
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "✅ Build artifacts cleaned"

.PHONY: clean-all
clean-all: clean
	@echo "Cleaning additional files..."
	rm -f lcov.info
	rm -rf target/
	@echo "✅ All artifacts cleaned"

# CI pipeline (matches GitHub Actions)
.PHONY: ci
ci: check-tools fmt-check lint test-all audit license-check build-release

# Extended testing (CI)
.PHONY: test-all
test-all: test
	@echo "Running additional test suites..."
	cargo test --doc --all-features
	@echo "✅ All tests passed"

# Release workflow
.PHONY: release-prep
release-prep: all update audit license-check
	@echo "✅ Release preparation complete"
	@echo "Prerequisites:"
	@echo "- Create 'release' environment in GitHub Settings > Environments"
	@echo "- Set CRATES_IO_TOKEN secret in the release environment"
	@echo "Next steps:"
	@echo "1. Update version in Cargo.toml"
	@echo "2. Update CHANGELOG.md"
	@echo "3. Commit changes"
	@echo "4. Create git tag: git tag v<VERSION>"
	@echo "5. Push tag: git push origin v<VERSION>"

.PHONY: release-check
release-check: ci
	@echo "Running release checks..."
	cargo package --allow-dirty
	@echo "✅ Release package validation passed"

# Release management (automated with safety checks)
.PHONY: release-patch
release-patch:
	@./scripts/release.sh patch

.PHONY: release-minor
release-minor:
	@./scripts/release.sh minor

.PHONY: release-major
release-major:
	@./scripts/release.sh major

# Development helpers
.PHONY: watch
watch:
	@echo "Starting cargo watch..."
	cargo watch

.PHONY: run
run:
	@echo "Running project..."
	cargo run

.PHONY: install
install:
	@echo "Installing project..."
	cargo install --path .

# Help system
.PHONY: help
help:
	@echo "GraphQL Rust Codegen - Development Workflow"
	@echo ""
	@echo "Development:"
	@echo "  make dev          - Full development workflow (fmt, lint, test, doc)"
	@echo "  make test         - Run tests"
	@echo "  make test-watch   - Run tests in watch mode"
	@echo "  make lint         - Run clippy"
	@echo "  make fmt          - Format code"
	@echo "  make doc          - Build documentation"
	@echo ""
	@echo "Quality & Security:"
	@echo "  make audit        - Security vulnerability scan"
	@echo "  make license-check- License compliance check"
	@echo "  make coverage     - Generate code coverage report"
	@echo ""
	@echo "Dependencies:"
	@echo "  make update       - Update within current version ranges"
	@echo "  make upgrade      - Upgrade to latest compatible versions"
	@echo "  make upgrade-major- Upgrade including breaking changes (dangerous)"
	@echo "  make outdated     - Check for outdated dependencies"
	@echo "  make setup        - Install development dependencies"
	@echo ""
	@echo "Building:"
	@echo "  make build        - Debug build"
	@echo "  make build-release- Release build"
	@echo ""
	@echo "Release:"
	@echo "  make release-prep   - Prepare for release (run all checks)"
	@echo "  make release-check  - Validate release package"
	@echo "  make release-patch  - Full release cycle for patch version"
	@echo "  make release-minor  - Full release cycle for minor version"
	@echo "  make release-major  - Full release cycle for major version"
	@echo ""
	@echo "CI/CD:"
	@echo "  make ci           - Full CI pipeline"
	@echo "  make fmt-check    - Check formatting (CI mode)"
	@echo "  make test-all     - Extended testing (CI mode)"
	@echo ""
	@echo "Utilities:"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make clean-all    - Clean everything"
	@echo "  make watch        - Start cargo watch"
	@echo "  make run          - Run the project"
	@echo "  make install      - Install the project"
	@echo "  make help         - Show this help"

# Default help target
.DEFAULT_GOAL := help

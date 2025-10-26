# Worknest Makefile
# Convenient commands for local development

.PHONY: help build test check fmt clippy clean run dev install audit coverage build-webapp serve-webapp

# Default target
help:
	@echo "Worknest Development Commands"
	@echo "=============================="
	@echo ""
	@echo "Web Application:"
	@echo "  make build-webapp      - Build webapp (no trunk required)"
	@echo "  make serve-webapp      - Build and serve webapp locally"
	@echo ""
	@echo "Backend Development:"
	@echo "  make build             - Build all crates in release mode"
	@echo "  make test              - Run all tests"
	@echo "  make check             - Run all checks (fmt, clippy, test)"
	@echo "  make fmt               - Format code"
	@echo "  make clippy            - Run clippy linter"
	@echo "  make clean             - Clean build artifacts"
	@echo "  make run               - Run the GUI application"
	@echo "  make dev               - Run the GUI application in dev mode"
	@echo "  make install           - Install the binary locally"
	@echo "  make audit             - Run security audit"
	@echo "  make coverage          - Generate code coverage report"
	@echo "  make pre-commit        - Run all pre-commit checks"

# Build release
build:
	@echo "Building workspace in release mode..."
	@cargo build --workspace --release

# Run all tests
test:
	@echo "Running tests..."
	@cargo test --workspace --verbose

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt --all

# Check formatting
fmt-check:
	@echo "Checking code formatting..."
	@cargo fmt --all -- --check

# Run clippy
clippy:
	@echo "Running clippy..."
	@cargo clippy --workspace --all-targets --all-features -- -D warnings

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf dist pkg

# Run GUI application in release mode
run:
	@echo "Running Worknest GUI..."
	@cargo run --release -p worknest-gui

# Run GUI application in dev mode
dev:
	@echo "Running Worknest GUI (dev mode)..."
	@cargo run -p worknest-gui

# Install locally
install:
	@echo "Installing Worknest..."
	@cargo install --path crates/worknest-gui

# Security audit
audit:
	@echo "Running security audit..."
	@cargo audit

# Generate code coverage
coverage:
	@echo "Generating code coverage..."
	@cargo tarpaulin --workspace --verbose --timeout 300 --out Html --output-dir coverage

# Run all pre-commit checks
pre-commit: fmt-check clippy test
	@echo "All pre-commit checks passed!"

# Full check (CI equivalent)
check: fmt-check clippy test build
	@echo "All checks passed!"

# Quick check (fast feedback)
quick-check:
	@echo "Running quick checks..."
	@cargo check --workspace

# Watch mode for development
watch:
	@echo "Starting watch mode..."
	@cargo watch -x 'check --workspace' -x 'test --workspace'

# Build documentation
docs:
	@echo "Building documentation..."
	@cargo doc --workspace --no-deps --open

# Update dependencies
update:
	@echo "Updating dependencies..."
	@cargo update

# Check for outdated dependencies
outdated:
	@echo "Checking for outdated dependencies..."
	@cargo outdated --workspace

# Benchmark (when we have benchmarks)
bench:
	@echo "Running benchmarks..."
	@cargo bench --workspace

# Build webapp (no trunk required)
build-webapp:
	@echo "Building webapp with wasm-pack..."
	@./build-webapp.sh release

# Build webapp in debug mode
build-webapp-dev:
	@echo "Building webapp in debug mode..."
	@./build-webapp.sh debug

# Serve webapp locally
serve-webapp: build-webapp
	@echo "Serving webapp at http://localhost:8080"
	@cd dist && python3 serve.py

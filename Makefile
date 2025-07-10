.PHONY: all build check test fmt fmt-toml fmt-check clippy clean doc bench audit dev-deps ci help

# Run fmt, fmt-toml, clippy, and test
all: fmt fmt-toml clippy test

# Build with various feature configurations
build:
	@echo "Building with default features..."
	@cargo build --verbose
	@echo "Building with tracing feature..."
	@cargo build --features tracing --verbose
	@echo "Building with all features..."
	@cargo build --all-features --verbose
	@echo "Building with no default features..."
	@cargo build --no-default-features --verbose

# Type check without building
check:
	@echo "Checking with default features..."
	@cargo check --all
	@echo "Checking with all features..."
	@cargo check --all --all-features
	@echo "Checking with no default features..."
	@cargo check --all --no-default-features

# Run tests with various feature configurations
test:
	@echo "Testing with default features..."
	@cargo test --verbose
	@echo "Testing with all features..."
	@cargo test --all-features --verbose
	@echo "Testing with no default features..."
	@cargo test --no-default-features --verbose

# Format Rust code
fmt:
	@cargo fmt --all

# Format TOML files
fmt-toml:
	@taplo fmt

# Check code formatting (CI mode)
fmt-check:
	@cargo fmt --all -- --check
	@taplo fmt --check

# Run clippy linter
clippy:
	@echo "Running clippy with default features..."
	@cargo clippy --all -- -D warnings
	@echo "Running clippy with all features..."
	@cargo clippy --all --all-features -- -D warnings
	@echo "Running clippy with no default features..."
	@cargo clippy --all --no-default-features -- -D warnings

# Clean build artifacts
clean:
	@cargo clean

# Generate and open documentation
doc:
	@cargo doc --all-features --open

# Run benchmarks
bench:
	@echo "Running benchmarks..."
	@cargo bench

# Check for security vulnerabilities
audit:
	@cargo audit

# Install development dependencies
dev-deps:
	@echo "Installing development dependencies..."
	@rustup component add rustfmt
	@rustup component add clippy
	@cargo install cargo-audit
	@cargo install taplo-cli

# Run CI checks locally
ci: fmt-check clippy test build

# Show this help message
help:
	@echo ''
	@echo 'Usage:'
	@echo '  make [target]'
	@echo ''
	@echo 'Targets:'
	@awk '/^[a-zA-Z\-\_0-9]+:/ { \
	helpMessage = match(lastLine, /^# (.*)/); \
		if (helpMessage) { \
			helpCommand = substr($$1, 0, index($$1, ":")); \
			helpMessage = substr(lastLine, RSTART + 2, RLENGTH); \
			printf "\033[36m%-15s\033[0m %s\n", helpCommand, helpMessage; \
		} \
	} \
	{ lastLine = $$0 }' $(MAKEFILE_LIST)

.DEFAULT_GOAL := help

.PHONY: all build release test check fmt lint audit doc bench clean help run-sample

# Default target
all: fmt lint test build

## Build targets
build:
	cargo build --workspace

release:
	cargo build --workspace --release

check:
	cargo check --workspace --all-targets

## Test & Bench targets
test:
	cargo test --workspace --all-targets

bench:
	cargo bench --workspace

## Quality & Maintenance
fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

audit:
	cargo audit

doc:
	cargo doc --workspace --no-deps --document-private-items

clean:
	cargo clean

## Run targets
run-sample:
	cargo run -p sample-axum

## Help
help:
	@echo "Available commands:"
	@echo "  make build       - Build the entire workspace"
	@echo "  make release     - Build the entire workspace in release mode"
	@echo "  make check       - Run cargo check on all crates"
	@echo "  make test        - Run all tests"
	@echo "  make bench       - Run all benchmarks"
	@echo "  make fmt         - Format all code using cargo fmt"
	@echo "  make lint        - Run clippy for all crates"
	@echo "  make audit       - Run cargo audit for security vulnerabilities"
	@echo "  make doc         - Generate documentation"
	@echo "  make clean       - Clean the target directory"
	@echo "  make run-sample  - Run the sample-axum application"
	@echo "  make all         - Run fmt, lint, test, and build"

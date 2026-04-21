.PHONY: all build release test check fmt lint audit doc bench clean help run-sample run-bench-api

# Set PATH to use Rust 1.94 from cargo bin
export PATH := $(USERPROFILE)\.cargo\bin;$(PATH)

# Default target
all: fmt lint test build

## Build targets
build:
	$(PATH) cargo build --workspace

release:
	$(PATH) cargo build --workspace --release

check:
	$(PATH) cargo check --workspace --all-targets

## Test & Bench targets
test:
	$(PATH) cargo test -p fastserial --all-targets

bench:
	$(PATH) cargo bench --workspace

## Quality & Maintenance
fmt:
	$(PATH) cargo fmt --all

lint:
	$(PATH) cargo clippy --workspace --all-targets -- -D warnings

audit:
	$(PATH) cargo audit

doc:
	$(PATH) cargo doc -p fastserial --no-deps --document-private-items

clean:
	$(PATH) cargo clean

## Run targets
run-sample:
	$(PATH) cargo run -p sample-axum

run-bench-api:
	$(PATH) cargo run -p api-bench

## Benchmark API Server - runs on http://127.0.0.1:8888
run-api-bench-server:
	$(PATH) cargo run --release -p api-bench

## Help
help:
	@echo "Available commands:"
	@echo "  make build              - Build the entire workspace"
	@echo "  make release            - Build the entire workspace in release mode"
	@echo "  make check             - Run cargo check on all crates"
	@echo "  make test             - Run all tests"
	@echo "  make bench            - Run all benchmarks"
	@echo "  make fmt              - Format all code using cargo fmt"
	@echo "  make lint             - Run clippy for all crates"
	@echo "  make audit           - Run cargo audit for security vulnerabilities"
	@echo "  make doc             - Generate documentation"
	@echo "  make clean           - Clean the target directory"
	@echo "  make run-sample       - Run the sample-axum application"
	@echo "  make run-bench-api   - Run api-bench application (benchmark server)"
	@echo "  make all            - Run fmt, lint, test, and build"

# Makefile for a Rust CLI project

# Default target
all: build

# Build the project in debug mode
build:
	cargo build

# Run the application with optional args: make run ARGS="--help"
run:
	cargo run -- $(ARGS)

# Build the project in release mode
release:
	cargo build --release

# Run the release binary with args: make run-release ARGS="--help"
run-release:
	cargo run --release -- $(ARGS)

# Run tests
test:
	cargo test

# Clean the target directory
clean:
	cargo clean

# Format the code
fmt:
	cargo fmt

# Check for warnings and errors
check:
	cargo check

# praetor task runner. Run `just` (or `just --list`) to see available recipes.

# Show the list of recipes.
default:
    @just --list

# Run the messages example.
example:
    cargo run --example messages

# Run the Axum integration example.
example-axum:
    cargo run --example axum --features axum

# Compile the project.
build:
    cargo build

# Build an optimized release build.
release:
    cargo build --release

# Type-check without producing an artifact.
check:
    cargo check

# Run clippy lints.
lint:
    cargo clippy

# Format all code.
fmt:
    cargo fmt --all

# Verify formatting without modifying files.
fmt-check:
    cargo fmt --all -- --check

# Run the test suite.
test:
    cargo test --all-features

# Remove build artifacts.
clean:
    cargo clean

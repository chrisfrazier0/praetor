# praetor task runner. Run `just` (or `just --list`) to see available recipes.

# Show the list of recipes.
default:
    @just --list

# Compile the project.
build:
    cargo build

# Build an optimized release binary.
release:
    cargo build --release

# Type-check without producing a binary.
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

# Remove build artifacts.
clean:
    cargo clean

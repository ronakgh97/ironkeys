# IronKey - Just commands
# Install just: cargo install just
# Usage: just <command>

# Set shell for Windows compatibility
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# List available commands
default:
    @just --list

# Run all tests (single-threaded for clipboard compatibility)
test:
    cargo test -- --test-threads=1

# Run specific test file
test-file FILE:
    cargo test --test {{FILE}} -- --test-threads=1

# Run non-clipboard tests (parallel, faster)
test-fast:
    cargo test --test crypto_tests
    cargo test --test storage_tests
    cargo test --test vault_tests
    cargo test --test integration_tests

# Run clipboard tests specifically
test-clipboard:
    cargo test --test clipboard_tests -- --test-threads=1

# Run clipboard auto-clear tests
test-autoclear:
    cargo test --test clipboard_autoclear_tests -- --test-threads=1

# Run password generator tests
test-generator:
    cargo test --test password_generator_tests

# Build release version
build:
    cargo build --release

# Check code quality (clippy)
check:
    cargo clippy --all-targets

# Format code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt -- --check

# Run all CI checks locally
ci: check test fmt-check
    @echo "All CI checks passed!"

# Clean build artifacts
clean:
    cargo clean

# Run the binary
run *ARGS:
    cargo run -- {{ARGS}}

# Show help for a command
help COMMAND:
    cargo run -- {{COMMAND}} --help

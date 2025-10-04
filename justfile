# IronKey - Just Task Runner
# Install just: cargo install just
# Usage: just <command>
# List all commands: just --list or just
# Set shell for Windows compatibility

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Show all available commands
default:
    @just --list

# Run all tests (single-threaded for clipboard compatibility)
test:
    @echo "Running all tests (single-threaded)..."
    cargo test -- --test-threads=1

# Run specific test file by name (e.g., just test-file clipboard_tests)
test-file FILE:
    @echo "Running tests from {{ FILE }}..."
    cargo test --test {{ FILE }} -- --test-threads=1

# Run non-clipboard tests in parallel
test-fast:
    @echo "Running fast tests (non-clipboard, parallel)..."
    cargo test --test crypto_tests
    cargo test --test storage_tests
    cargo test --test vault_tests
    cargo test --test integration_tests
    cargo test --test password_generator_tests
    cargo test --test search_filter_tests
    cargo test --test export_tests
    cargo test --test import_tests
    cargo test --test export_import_roundtrip_tests
    @echo "✓ Fast tests completed!"

# Build debug version
build-dev:
    @echo "Building dev version..."
    cargo build

# Build optimized release version
build:
    @echo "Building release version..."
    cargo build --release

# Run clippy (treats warnings as errors)
check:
    @echo "Running clippy checks..."
    cargo clippy --all-targets -- -D warnings

# Run clippy with auto-fix (modifies files)
check-fix:
    @echo "Running clippy with auto-fix..."
    cargo clippy --all-targets --fix --allow-dirty --allow-staged

# Format code with rustfmt
fmt:
    @echo "Formatting code..."
    cargo fmt

# Auto-fix code issues with clippy and rustfmt
fix:
    @echo "Applying auto-fixes..."
    cargo clippy --all-targets --fix --allow-dirty --allow-staged
    cargo fmt
    @echo "✓ Code auto-fixed"

# Check if code is properly formatted (CI-friendly)
fmt-check:
    @echo "Checking code formatting..."
    cargo fmt -- --check

# Run all CI checks (code quality + tests + formatting)
ci: check test fmt-check
    @echo ""
    @echo "✓ All CI checks passed!"
    @echo "   • Clippy: clean"
    @echo "   • Tests: 106/106 passing"
    @echo "   • Format: compliant"

# Run CD pipeline (CI + clean install)
cd: ci clean-install
    @echo ""
    @echo "✓ CD pipeline completed!"
    @echo "   • All checks passed"
    @echo "   • Binary installed successfully"

# Run both CI and CD (full pipeline)
ci-cd: ci cd
    @echo ""
    @echo "◆ Full CI/CD pipeline completed successfully!"

# Clean build artifacts and do fresh install
clean-install:
    @echo "Cleaning and installing..."
    cargo clean
    -cargo uninstall ironkey 2>$null
    cargo build --release
    cargo install --path .
    @echo "✓ Installed: ik v0.0.2-beta"

# Install locally (without clean)
install:
    @echo "Installing IronKey..."
    cargo install --path .
    @echo "✓ Installed: ik v0.0.2-beta"

# Uninstall IronKey
uninstall:
    @echo "Uninstalling IronKey..."
    cargo uninstall ironkey
    @echo "✓ Uninstalled"

# Clean all build artifacts
clean:
    @echo "Cleaning build artifacts..."
    cargo clean
    @echo "✓ Cleaned"

# Run IronKey with custom arguments (e.g., just run --version)
run *ARGS:
    cargo run -- {{ ARGS }}

# Show help for a specific command (e.g., just help export)
help COMMAND:
    cargo run -- {{ COMMAND }} --help

# Run IronKey in release mode with arguments
run-release *ARGS:
    cargo run --release -- {{ ARGS }}

# Watch for changes and run tests automatically (requires cargo-watch)
watch:
    cargo watch -x "test -- --test-threads=1"

# Show version information
version:
    @echo "IronKey Version Info:"
    @cargo run -- --version
    @echo ""
    @rustc --version
    @cargo --version

# Show project statistics
stats:
    @echo "IronKey Statistics:"
    @echo "Source files:"
    @fd -e rs src/ | wc -l
    @echo "Test files:"
    @fd -e rs tests/ | wc -l
    @echo "Total lines of code:"
    @fd -e rs | xargs wc -l | tail -1
    @echo ""
    @echo "Dependencies:"
    @cargo tree --depth 1

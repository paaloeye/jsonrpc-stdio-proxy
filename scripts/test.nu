#!/usr/bin/env nu

# Run format checks, clippy linter, and integration test suite
def main [] {
    print "Checking code formatting..."
    cargo fmt --check

    print "Running Clippy..."
    cargo clippy --all-targets --all-features -- -D warnings

    print "Running test suite..."
    cargo test --verbose

    print "All tests and lints passed!"
}

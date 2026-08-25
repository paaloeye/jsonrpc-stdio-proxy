#!/usr/bin/env nu

# Build release binary and perform smoke testing
def main [
    --sign # Perform ad-hoc codesign on macOS
] {
    print "Building jsonrpc-stdio-proxy (release)..."
    cargo build --release

    let binary = "target/release/jsonrpc-stdio-proxy"
    if not ($binary | path exists) {
        error make { msg: $"Binary not found at ($binary)" }
    }

    if $sign {
        print "Signing binary with ad-hoc signature..."
        codesign -s - -f --options runtime $binary
    }

    print "Running smoke tests..."
    ^$binary --version
    ^$binary --help | ignore

    print $"Release binary built successfully: ($binary)"
}

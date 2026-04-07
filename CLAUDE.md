# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```sh
# Debug build
cargo build

# Release build (required for use in .mcp.json)
cargo build --release

# Run directly (proxies a command via stdio)
cargo run -- -- <command> [args...]
```

## Testing

```sh
# Run all tests
cargo test

# Run a single test
cargo test test_mcp_style_ndjson

# Integration tests spawn the proxy via `cargo run` with `cat` as the child process
```

## Linting & Formatting

```sh
cargo fmt
cargo clippy
```

## Architecture

This is a single-file Rust binary (`src/main.rs`) built on Tokio. It spawns a child process and bidirectionally proxies `stdio` between the caller and the child, logging all JSON-RPC payloads to macOS `OSLog` (`oslog` crate) without polluting `stdout`.

**Core flow:**

1. CLI args (via `clap`) capture `--subsystem`, `--category`, and the child command (`--` separator).
2. `OsLogger` is initialised with the given subsystem so logs go to Apple Unified Logging, not `stdout`/`stderr`.
3. Three async tasks run concurrently:
   - `stdin_task` — reads proxy stdin, forwards to child stdin (Client → Server)
   - `stdout_task` — reads child stdout, forwards to proxy stdout (Server → Client)
   - `stderr_task` — reads child stderr, logs each line via `info!`
4. A `tokio::select!` drives shutdown: child exit, Ctrl-C, or stdin EOF all cleanly kill the child.
5. Metrics (`Arc<Metrics>`) are updated atomically in each direction and dumped to OSLog on exit.

**Protocol detection** happens inside `proxy_and_log_stream`: if the first line starts with `Content-Length:`, the stream is treated as Header-Delimited (LSP/DAP); otherwise it is treated as Newline-Delimited (MCP/NDJSON). Both paths forward the full frame verbatim.

**RTT tracking**: outbound requests (has `method`, has `id`, no `result`/`error`) are recorded in a `DashMap<id, Instant>`; matching inbound responses resolve the entry and store the elapsed `Duration` in a `DashSet`.

## Viewing Logs

```sh
log stream --predicate 'subsystem == "com.paaloeye.flight.engineer"' --debug --info
log show  --predicate 'subsystem == "com.paaloeye.flight.engineer"' --debug --info --last 1h
```

## Key Constraints

- macOS only — depends on Apple `OSLog` via the `oslog` crate.
- The release binary must exist at `target/release/jsonrpc-stdio-proxy` before entries in `.mcp.json` work.
- `stdout` must never receive anything other than the proxied JSON-RPC stream; all diagnostics go through `log!` macros → OSLog.

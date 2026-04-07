# Gemini CLI Agent Instructions

## Project Overview
`jsonrpc-stdio-proxy` is a single-binary transparent proxy for JSON-RPC communication over standard input/output (stdio). It supports both Newline-Delimited (MCP) and Header-Delimited (LSP/DAP) framing.

## Architecture
- **Language:** Rust (Edition 2024).
- **Concurrency:** Uses `tokio` for asynchronous I/O and process management.
- **Observability:** Relies heavily on Apple's Unified Logging (`oslog` crate) to keep stdout strictly reserved for JSON-RPC data.
- **Metrics:** Uses `dashmap` and `std::sync::atomic` for zero-block concurrency metrics (latency, bytes, message counts).

## Rules & Conventions
1. **Never Pollute Stdout:** The proxy's `stdout` must **only** contain valid JSON-RPC traffic. All logs, warnings, errors, and debug information MUST be routed through the `log` crate (which outputs to OSLog via `oslog`).
2. **Process Lifecycle:**
   - If the proxy's `stdin` closes (EOF), the proxy MUST immediately send `SIGTERM`/`kill` to the child process.
   - The proxy MUST wait for the child to exit before terminating itself.
   - Performance metrics MUST be logged during the shutdown sequence before exiting.
3. **Dependencies:** Avoid adding external dependencies if the standard library or `tokio` can handle it natively. If parsing JSON is required, use `serde_json`.

## Build & Test Commands
- **Build:** `cargo build`
- **Release:** `cargo build --release`
- **Test:** `cargo test`

## Context Efficiency
When modifying the streaming/parsing loop (`proxy_and_log_stream`), avoid making sweeping changes. Use targeted surgical replacements as the state machine managing bytes and buffers is delicate.

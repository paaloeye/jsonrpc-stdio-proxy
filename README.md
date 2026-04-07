# JSON-RPC Stdio Proxy

A lightweight, zero-configuration proxy that sits between a JSON-RPC client (like Claude Code, VS Code, or Cursor) and a JSON-RPC server (like an MCP tool or Language Server).

It transparently forwards `stdio` traffic while observing, measuring, and logging the JSON payloads directly into **Apple's Unified Logging System (OSLog)**.
This prevents log pollution on `stdout` (which would break the protocol) while providing complete observability into the process.

## Features

- [x] **Universal Framing**: Automatically detects and parses both Newline-Delimited (MCP) and Header-Delimited (LSP/DAP) JSON-RPC streams.
- [x] **Native Observability**: Logs all requests and responses directly to the macOS `OSLog` framework.
- [x] **Performance Metrics**: Tracks session duration, byte counts, message counts, and calculates Request/Response Round-Trip Time (RTT).
- [x] **Single Binary**: Compiles to a small, statically-linked executable (via Rust/Tokio) with zero external runtime dependencies.
- [x] **Aggressive Cleanup**: Correctly mirrors EOF and signals to prevent zombie processes.

## Installation / Building

```sh
cargo build --release
# The binary will be available at target/release/jsonrpc-stdio-proxy
```

## Usage

You can use the proxy anywhere you normally invoke a CLI tool. Simply prefix your command with `jsonrpc-stdio-proxy --`.

### `.mcp.json` Example

Configure an MCP client to use the proxy and specify a custom OSLog subsystem for filtering:

```json
{
    "mcpServers": {
        "flight-engineer": {
            "command": "target/release/jsonrpc-stdio-proxy",
            "args": [
                "--subsystem",
                "com.paaloeye.flight.engineer",
                "--",
                "ferun",
                "mcpbridge"
            ]
        }
    }
}
```

## Viewing Logs (macOS)

Because the proxy uses Apple's Unified Logging, you can view the traffic in real-time using the `log` CLI tool or the native `Console.app`.

```sh
# Stream logs in real-time
log stream --predicate 'subsystem == "com.paaloeye.flight.engineer"' --debug --info

# View past logs
log show --predicate 'subsystem == "com.paaloeye.flight.engineer"' --debug --info --last 1h
```

## Metrics Output

When the proxy shuts down, it automatically dumps a performance summary to OSLog:

```text
--- Performance Metrics Summary ---
Session Duration: 24.5s
Client -> Server: 14 msgs, 2048 bytes
Server -> Client: 14 msgs, 45120 bytes
RTT Latency: Min 1.2ms, Max 45.1ms, Avg 12.4ms
Errors: 0
```

## References

- [tokio](https://tokio.rs/)
- [dashmap crate](https://github.com/xacrimon/dashmap)
- [Apple OS tracing crate](https://github.com/Absolucy/tracing-oslog)

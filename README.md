# JSON-RPC Stdio Proxy

A lightweight, zero-configuration proxy that sits between JSON-RPC clients (e.g Claude Code, VS Code) and
JSON-RPC servers (e.g. [MCP](https://modelcontextprotocol.io/specification/2026-07-28), [LSP](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/),
or [DAP](https://microsoft.github.io/debug-adapter-protocol//specification.html)).

It transparently forwards `stdio` streams while observing, measuring, and logging JSON payloads directly into **Apple's Unified Logging System (OSLog)**.

## Features

- [x] **Universal Framing**: Automatically detects and parses both Newline-Delimited (MCP) and Header-Delimited (LSP/DAP with `Content-Length`) JSON-RPC streams.
- [x] **Native Observability**: Logs all requests and responses directly to Apple Unified Logging (`OSLog`).
- [x] **Performance Metrics**: Tracks session duration, byte counts, message counts, and calculates Request/Response Round-Trip Time (RTT latency: min/max/avg).
- [x] **Single Binary**: Compiles to a small, statically-linked executable (via Rust/Tokio) with zero external runtime dependencies.
- [x] **Aggressive Cleanup**: Correctly mirrors EOF and signals (`SIGINT`/Ctrl-C) to child processes to prevent zombie processes.

## Installation / Building

```sh
# Build debug binary
cargo build

# Build release binary
cargo install --path .
# The binary will be available at ~/.cargo/bin/jsonrpc-stdio-proxy
```

## CLI Usage

```text
Usage: jsonrpc-stdio-proxy [OPTIONS] -- <COMMAND>...

Arguments:
  <COMMAND>...  Target command and arguments to execute and proxy (must follow '--')

Options:
  -s, --subsystem <SUBSYSTEM>  macOS OSLog subsystem identifier for log filtering [default: com.paaloeye.jsonrpc-proxy]
  -c, --category <CATEGORY>    macOS OSLog category identifier [default: default]
  -h, --help                   Print help (see a summary with '-h')
  -V, --version                Print version
```

## Examples

### Command Line

```sh
# Proxy an MCP server with a custom subsystem identifier
jsonrpc-stdio-proxy --subsystem com.example.mcp -- npx -y @modelcontextprotocol/server-memory

# Proxy a Language Server Protocol (LSP) server
jsonrpc-stdio-proxy -- rust-analyzer
```

### MCP Client Configuration (`.mcp.json` / Claude Desktop)

Configure an MCP client to wrap any server command with the proxy:

```json
{
  "mcpServers": {
    "memory": {
      "command": "target/release/jsonrpc-stdio-proxy",
      "args": [
        "--subsystem",
        "com.example.mcp.memory",
        "--",
        "npx",
        "-y",
        "@modelcontextprotocol/server-memory"
      ]
    }
  }
}
```

## Viewing Logs (macOS)

Because the proxy uses Apple's Unified Logging, you can view the traffic in real-time using the `log` CLI tool or the native `Console.app`.

```sh
# Stream proxy logs in real time on macOS
log stream --predicate 'subsystem == "com.paaloeye.jsonrpc-proxy"' --debug --info

# Filter by custom subsystem
log stream --predicate 'subsystem == "com.example.mcp.memory"' --debug --info

# Show past session logs
log show --predicate 'subsystem == "com.paaloeye.jsonrpc-proxy"' --debug --info --last 1h
```

## Metrics Output

When the proxy shuts down, it automatically logs a performance summary to OSLog:

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
- [clap](https://docs.rs/clap)
- [dashmap crate](https://github.com/xacrimon/dashmap)
- [oslog crate](https://docs.rs/oslog)

---

> [!CAUTION]
> This file was compiled and written with AI assistance (Antigravity).

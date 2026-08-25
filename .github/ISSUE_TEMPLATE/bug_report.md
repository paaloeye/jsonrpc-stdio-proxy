---
name: Bug Report
about: Create a report to help improve jsonrpc-stdio-proxy
title: "[BUG] "
labels: ["type:bugfix"]
assignees: ""
---

## Bug Description

A clear and concise description of what the bug is.

## Steps to Reproduce

Steps to reproduce the behaviour:

1. Configure client with command: `...`
2. Start proxy with arguments: `...`
3. Send JSON-RPC payload: `...`
4. See error or unexpected behaviour

## Expected Behaviour

A clear and concise description of what you expected to happen.

## Actual Behaviour

A clear and concise description of what actually happened.

## Environment Information

- **OS**: [e.g. macOS Sonoma 14.5, Sequoia 15.0]
- **Architecture**: [e.g. arm64 (Apple Silicon), x86_64 (Intel)]
- **Rust Version**: [e.g. 1.85.0]
- **Proxy Version**: [e.g. 0.1.0]
- **Client**: [e.g. Claude Code, VS Code, kak-lsp, Antigravity CLI]
- **Target Command**: [e.g. Xcode MCP(`mcpbridge`), `zls`, `rust-analyzer`, `nu --lsp`, @modelcontextprotocol/server-memory]

## OSLog Diagnostics / Terminal Output

```text
# Run: log stream --predicate 'subsystem == "com.paaloeye.jsonrpc-proxy"' --debug --info
# Paste relevant OSLog traces or stderr logs here:
```

## Additional Context

Add any other context about the problem here (e.g. framing style: Newline-Delimited vs Content-Length header).

## Checklist

- [ ] I have searched existing issues to ensure this is not a duplicate
- [ ] I have provided all requested environment information
- [ ] I have included reproduction steps and relevant log output

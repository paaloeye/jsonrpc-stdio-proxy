# CLAUDE.md

This file provides guidance to Claude when working in this repository.

> [!WARNING]
> These rules override default behaviour. Follow them exactly when working with this codebase. Violations may cause
> linter failures or break pre-commit hooks.

## Project Overview

`jsonrpc-stdio-proxy` is a single-binary transparent proxy for JSON-RPC communication over standard input/output (stdio).
It supports both Newline-Delimited (MCP) and Header-Delimited (LSP/DAP) framing.

## IMPORTANT

- ALWAYS read [GOTCHA.md](./GOTCHA.md) first
- ALWAYS read `CLAUDE.*.fragment.md` first
- PREFER British English over American English spelling and grammar except in **inline code** sections
- USE Markdown banners ([see below](#a-tour-of-banners))
- Files and Directories MUST NOT have **dashes** in names/paths (use **underscore** instead)
- NEVER use Git LFS
- USE Emoji in [README.md](./README.md) or **docs/\*.md** with care. NOT MUCH.
- ALL development scripts use Nushell (\*.nu) - install nushell for development workflow
- ALWAYS use `[x]` or `[ ]` instead of ✅ / 🔲 / for checkmarks
- NEVER use `[x]` or `[ ]` in Markdown tables; USE ✅ / 🔲 / instead. **Reason**: it's not supported
- PREFER [GitHub Emoji API](https://api.github.com/emojis) over Unicode Emoji
- ALWAYS add footer to new Markdown files with a AI generated content banner (!CAUTION)
- PREFER 120 characters per line

## A Tour of Banners

> [!NOTE]
> Highlights information that users should take into account, even when skimming.

> [!TIP]
> Optional information to help a user be more successful.

> [!IMPORTANT]
> Crucial information necessary for users to succeed.

> [!WARNING]
> Critical content demanding immediate user attention due to potential risks.

> [!CAUTION]
> Negative potential consequences of an action.

## Conventions

- **We're Dutch honest**
- British English throughout (colour, licence, behaviour, etc.)
- No dashes in file or directory names — use underscores
- Follow conventional commit format (see workspace CLAUDE.md for the full format)

> [!IMPORTANT]
> Before every commit, re-read the commit format in workspace **CLAUDE.md**.
> The footer **requires** shell-expanded `ai.nu` lines and an **unquoted** heredoc (`EOF`, not `'EOF'`):

## Architecture

- **Language:** Rust (Edition 2024).
- **Concurrency:** Uses `tokio` for asynchronous I/O and process management.
- **Observability:** Relies heavily on Apple's Unified Logging (`oslog` crate) to keep stdout strictly reserved for JSON-RPC data.
- **Metrics:** Uses `dashmap` and `std::sync::atomic` for zero-block concurrency metrics (latency, bytes, message counts).

## Rules & Conventions

1. **Never Pollute Stdout:** The proxy's `stdout` must **only** contain valid JSON-RPC traffic.
   All logs, warnings, errors, and debug information MUST be routed through the `log` crate (which outputs to OSLog via `oslog`).

2. **Process Lifecycle:**
   - If the proxy's `stdin` closes (EOF), the proxy MUST immediately send `SIGTERM`/`kill` to the child process.
   - The proxy MUST wait for the child to exit before terminating itself.
   - Performance metrics MUST be logged during the shutdown sequence before exiting.
   -
3. **Dependencies:** Avoid adding external dependencies if the standard library or `tokio` can handle it natively.
   If parsing JSON is required, use `serde_json`.

## Build & Test Commands

- **Pre-commit Setup:** `pre-commit install --install-hooks -t pre-commit -t commit-msg`
- **Pre-commit Run:** `pre-commit run --all-files`
- **Test Suite & Lint:** `nu scripts/test.nu` (or `cargo test`)
- **Build Release:** `nu scripts/build.nu` (or `cargo build --release`)
- **Build & Sign (ad-hoc):** `nu scripts/build.nu --sign`

## Context Efficiency

When modifying the streaming/parsing loop (`proxy_and_log_stream`), avoid making sweeping changes.
Use targeted surgical replacements as the state machine managing bytes and buffers is delicate.

## Viewing Logs

```sh
log stream --predicate 'subsystem == "com.paaloeye.jsonrpc-proxy"' --debug --info
log show  --predicate 'subsystem == "com.paaloeye.jsonrpc-proxy"' --debug --info --last 1h
```

## Key Constraints

- macOS only — depends on Apple `OSLog` via the `oslog` crate.
- The release binary must exist at `target/release/jsonrpc-stdio-proxy` before entries in `.mcp.json` work.
- `stdout` must never receive anything other than the proxied JSON-RPC stream; all diagnostics go through `log!` macros → OSLog.

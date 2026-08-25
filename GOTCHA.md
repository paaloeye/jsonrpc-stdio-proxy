# GOTCHA.md

Common gotchas and pitfalls when working in AI-aided fashion on `jsonrpc-stdio-proxy`.

This file provides guidance to Claude/Gemini CLI/Antigravity CLI when it keeps making the same mistakes.

## IMPORTANT

- ALWAYS read `GOTCHA.*.fragment.md` first

---

## 1. Process Lifecycle & Tokio Stdin Blocking

### Issue

Attempting a graceful Tokio runtime shutdown causes the proxy process to hang indefinitely when `stdin` closes (EOF) or upon receiving `SIGINT`.

### Symptoms

Process hangs on exit instead of terminating cleanly, leaving orphaned background threads or child processes.

### Solution

Use `std::process::exit(exit_code)` to terminate the process immediately once the child process has been terminated and performance metrics have been captured.

### Why This Happens

`tokio::io::stdin` uses an internal blocking background thread pool that cannot be cancelled via task abort.

---

## 2. Never Pollute `stdout`

### Issue

Writing non-protocol text (e.g. `println!`, `eprintln!`, debug output) directly to `stdout`.

### Symptoms

JSON-RPC client (Claude Code, VS Code, etc.) fails to parse incoming stream and drops connection with framing errors.

### Solution

All logging, metrics, diagnostics, and errors MUST be routed through the `log` crate macros (`info!`, `debug!`, `error!`),
which are redirected into Apple OSLog via `OsLogger`.

---

## 3. GitHub Actions: `actions/upload-artifact@v7` is Current

### Issue

Assuming `actions/upload-artifact@v4` is the latest major version and attempting to downgrade or "correct" `actions/upload-artifact@v7`.

### Symptoms

Agents falsely flagging `actions/upload-artifact@v7` as non-existent or replacing it with `@v4` in workflows.

### Solution

Always use `actions/upload-artifact@v7` for artifact upload steps in GitHub Actions workflows.

---

## 4. Install Both `pre-commit` and `commit-msg` Hook Stages

### Issue

Running `git commit` without installing hooks or only installing the default `pre-commit` hook stage causes broken commits or CI failures.

### Symptoms

- Conventional commit message header rules are bypassed locally but rejected in CI.
- Markdown files fail formatting (`bun fmt:docs`) or lint checks in pull requests.

### Solution

Always install hooks for both stages upon cloning or setting up the workspace:

```bash
pre-commit install --install-hooks -t pre-commit -t commit-msg
```

---

> [!CAUTION]
> This file was compiled and written with AI assistance (Antigravity).

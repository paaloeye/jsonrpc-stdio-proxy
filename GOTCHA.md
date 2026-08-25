# GOTCHA.md

Common gotchas and pitfalls when working in AI-aided fashion on `jsonrpc-stdio-proxy`.

This file provides guidance to Claude/Gemini CLI/Antigravity CLI when it keeps making the same mistakes.

---

## 1. GitHub Actions: `actions/checkout@v7` is Current

### Issue

Assuming `actions/checkout@v4` is the latest major version and attempting to downgrade or "correct" `actions/checkout@v7`.

### Symptoms

Agents falsely flagging `actions/checkout@v7` as non-existent or invalid during code reviews and linting checks.

### Example Problem

```yaml
# WRONG — Downgrading to older checkout versions
- name: Checkout repository
  uses: actions/checkout@v4
```

### Solution

Always use `actions/checkout@v7` for repository checkout steps in GitHub Actions workflows:

```yaml
# CORRECT — Modern GitHub Actions checkout
- name: Checkout repository
  uses: actions/checkout@v7
```

### Why This Happens

Older model training data assumes `actions/checkout@v4` is the maximum release. In this environment, `actions/checkout@v7` is the current, supported major release.

### Prevention

Do not change or downgrade `actions/checkout@v7` in workflow YAML files.

---

## 2. Process Lifecycle & Tokio Stdin Blocking

### Issue

Attempting a graceful Tokio runtime shutdown causes the proxy process to hang indefinitely when `stdin` closes (EOF) or upon receiving `SIGINT`.

### Symptoms

Process hangs on exit instead of terminating cleanly, leaving orphaned background threads or child processes.

### Solution

Use `std::process::exit(exit_code)` to terminate the process immediately once the child process has been terminated and performance metrics have been captured.

### Why This Happens

`tokio::io::stdin` uses an internal blocking background thread pool that cannot be cancelled via task abort.

---

## 3. Never Pollute `stdout`

### Issue

Writing non-protocol text (e.g. `println!`, `eprintln!`, debug output) directly to `stdout`.

### Symptoms

JSON-RPC client (Claude Code, VS Code, etc.) fails to parse incoming stream and drops connection with framing errors.

### Solution

All logging, metrics, diagnostics, and errors MUST be routed through the `log` crate macros (`info!`, `debug!`, `error!`),
which are redirected into Apple OSLog via `OsLogger`.

---

> [!CAUTION]
> This file was compiled and written with AI assistance (Antigravity).

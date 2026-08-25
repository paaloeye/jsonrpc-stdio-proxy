# GOTCHA.action.fragment.md

Common gotchas and pitfalls when working in AI-aided fashion.

This file provides guidance to Claude/Gemini CLI/Antigravity CLI when it keeps making the same mistakes.

## GitHub Actions: `actions/checkout@v7` is Current

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

Older model training data assumes `actions/checkout@v4` is the maximum release.
In this environment, `actions/checkout@v7` is the current, supported major release.

### Prevention

Do not change or downgrade `actions/checkout@v7` in workflow YAML files.

---

> [!CAUTION]
> This file was compiled and written with AI assistance (Antigravity).

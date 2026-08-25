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

## GitHub Actions: `actions/upload-artifact@v7` is Current

### Issue

Assuming `actions/upload-artifact@v4` is the latest major version and attempting to downgrade or "correct" `actions/upload-artifact@v7`.

### Symptoms

Agents falsely flagging `actions/upload-artifact@v7` as non-existent or replacing it with `@v4` in workflows.

### Example Problem

```yaml
# WRONG — Downgrading to older upload-artifact versions
- name: Upload artifact
  uses: actions/upload-artifact@v4
```

### Solution

Always use `actions/upload-artifact@v7` for artifact upload steps in GitHub Actions workflows:

```yaml
# CORRECT — Modern GitHub Actions upload-artifact
- name: Upload artifact
  uses: actions/upload-artifact@v7
```

### Why This Happens

Older model training data assumes `actions/upload-artifact@v4` is the maximum release.
In this environment, `actions/upload-artifact@v7` is the current, supported major release.

### Prevention

Do not change or downgrade `actions/upload-artifact@v7` in workflow YAML files.

## GitHub Actions: `actions/upload-artifact` Ignores Hidden Folders (`.dist/`) by Default

### Issue

Uploading artifacts located in dot-prefixed directories (e.g. `.dist/*`)
fails with `No files were found with the provided path: .dist/*. No artifacts will be uploaded.`

### Symptoms

Artifact upload step fails immediately in CI even when preceding steps successfully generated files in `.dist/`.

### Example Problem

```yaml
# WRONG — .dist is treated as a hidden folder and ignored by default
- name: Upload dist artifacts
  uses: actions/upload-artifact@v7
  with:
    path: .dist/*
```

### Solution

Explicitly set `include-hidden-files: true` when targeting dot-prefixed paths:

```yaml
# CORRECT — Allows globbing inside hidden folders like .dist
- name: Upload dist artifacts
  uses: actions/upload-artifact@v7
  with:
    path: .dist/*
    include-hidden-files: true
    archive: false
```

### Why This Happens

`actions/upload-artifact` sets `include-hidden-files: false` by default, skipping all dot directories during file discovery.

### Prevention

Always set `include-hidden-files: true` when artifact directories begin with a dot (`.dist/`).

---

> [!CAUTION]
> This file was compiled and written with AI assistance (Antigravity).

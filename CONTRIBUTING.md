# Contributing to JSON-RPC Stdio Proxy

Thank you for your interest in contributing to `jsonrpc-stdio-proxy`!
This document outlines our development process, code quality expectations, pre-commit hook setup, and pull request guidelines.

## Code of Conduct & Style

- **Dutch Honest**: Direct, constructive, and pragmatic communication.
- **British English**: Use British English spelling and grammar throughout documentation, comments, and commit messages
  (e.g., _colour_, _behaviour_, _licence_, _optimisation_), except in inline code tokens.
- **No Dashes in Names**: Use underscores for file and directory names (e.g., `feature_name.rs`, `my_script.nu`).
- **Standard Checks**: Use `[x]` or `[ ]` for checklists in markdown, and unicode checkmarks (`✅` / `🔲`) in tables.

## Prerequisites

Ensure the following tools are installed on your system:

- [Rust](https://www.rust-lang.org/) (Edition 2024 / stable toolchain)
- [pre-commit](https://pre-commit.com/) (`brew install pre-commit` or `pip install pre-commit`)
- [Bun](https://bun.sh/) (`brew install bun`) for markdown formatting hooks
- Optionally [Nushell](https://www.nushell.sh)

## Pre-Commit Setup

We enforce strict formatting, linting, typos checking, and conventional commit message validation using `pre-commit`.

To install the git hooks locally, run:

```bash
pre-commit install
```

To test all files manually before committing:

```bash
pre-commit run --all-files
```

## Development Workflow

We use Nushell scripts located in the `scripts/` directory for standard development tasks:

```bash
# Run format checking, clippy linting, and the integration test suite:
nu scripts/test.nu

# Build the release binary and run smoke tests:
nu scripts/build.nu
```

## Commit Message Format

Commit messages must follow the conventional commit format and include signed attribution footers:

```text
<type>(<scope>): <summary>

- <detailed bullet point 1>
- <detailed bullet point 2>

🤖 Generated with [Antigravity CLI](https://antigravity.google/product/antigravity-cli)

Co-Authored-By: <model-name> <gemini-code-assist@google.com>
Co-Authored-By: <effort-level>
Co-Authored-By: <agent-version>
Agent-Session: <session-id>
Signed-Off-By: <Author Name> <email@example.com>
```

## Submitting a Pull Request

1. Create a feature or bugfix branch (`git checkout -b feat/my_feature`).
2. Verify all tests and pre-commit hooks pass (`nu scripts/test.nu` and `pre-commit run --all-files`).
3. Commit using conventional commit format.
4. Push and open a pull request against `main`.
5. Ensure the pull request template is filled out completely and appropriate labels are applied.

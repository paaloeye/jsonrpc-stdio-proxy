## Description

Brief description of what this PR does and why.

## Related Issues

- Fixes #(issue number)
- Related to #(issue number)

## Type of Change

- [ ] Bug fix (non-breaking fix for an issue)
- [ ] New feature (non-breaking enhancement)
- [ ] Breaking change (fix or feature changing CLI flags or framing behaviour)
- [ ] Documentation update
- [ ] CI/CD or workflow improvement
- [ ] Refactoring / performance optimisation

## Testing

- [ ] Unit & integration tests pass locally (`nu scripts/test.nu` or `cargo test`)
- [ ] Code formatting check passes (`cargo fmt --check`)
- [ ] Clippy lints pass without warnings (`cargo clippy --all-targets --all-features -- -D warnings`)
- [ ] Verified that `stdout` remains clean of non-JSON-RPC text

## Checklist

- [ ] I have installed pre-commit hooks (`pre-commit install --install-hooks -t pre-commit -t commit-msg`)
- [ ] Pre-commit hooks pass on all staged changes (`pre-commit run --all-files`)
- [ ] My code follows the project's style guidelines (British English comments/docs)
- [ ] I have updated documentation (`README.md`, `GOTCHA.md`, `CONTRIBUTING.md`, etc.) where relevant
- [ ] I have verified process lifecycle (clean child termination on stdin EOF and signals)

---

> [!CAUTION]
> This file was compiled and written with AI assistance (Antigravity).

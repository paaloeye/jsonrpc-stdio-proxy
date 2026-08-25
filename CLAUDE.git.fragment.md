## Commit Messages

Follow conventional commit format with detailed explanations and proper sign-off.

### Format

```
<type>(<scope>): <subject>

<detailed body explaining what and why in bullet points>

BREAKING CHANGE: <description if applicable>

🤖 Generated with [$(ai.nu agent get-caller-identity --key .name)]($(ai.nu agent get-caller-identity --key .url))

Co-Authored-By: $(ai.nu model --key .name) <noreply@anthropic.com>
Co-Authored-By: $(ai.nu model --key .effort)
Co-Authored-By: $(ai.nu agent get-caller-identity)
Agent-Session: $(ai.nu agent get-caller-identity --key .session)
Signed-Off-By: Paal Øye-Strømme <paal.o.eye@gmail.com>
```

> [!IMPORTANT]
> The `$(ai.nu agent get-caller-identity)` and `$(ai.nu model ...)` footer lines require shell substitution. Use an **unquoted** heredoc (`EOF`, not `'EOF'`) so the shell expands it:
>
> ```bash
> git commit -m "$(cat <<EOF
> <message>
>
> Co-Authored-By: $(ai.nu agent get-caller-identity)
> Agent-Session: $(ai.nu agent get-caller-identity --key .session)
> EOF
> )"
> ```

### Best Practices

- **Type**: Use `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
- **Scope**: Specify affected module/component (e.g., `supabase`, `components`, `hooks`)
- **Subject**: Imperative mood, no period, max 50 characters
- **Body**: Explain the what and why, not how. Include context and reasoning in bullet points using "-" as for item mark
- **Breaking Changes**: Always document with `BREAKING CHANGE:` footer
- **Sign-off**: Include Agent attribution for AI-generated commits in `Co-Authored-By` and the main committer in `Signed-Off-By`

### Examples

```bash
feat(supabase): add verify-email function

- added verify-email Edge function to confirm users's email
- added tests

fix(components): add toaster

- toaster is used for notification
- no testes yet

docs:: update module usage examples and references

- for consistency
- improved readability

refactor(hooks): change useToast

- fixes #1
- added extra options
```

# Contributing Guidelines

Thank you for contributing to `nats.rs`! This guide covers the code style conventions, branch naming, commit message format, and pull request process for this repository.

---

## Getting Started

Before contributing:

1. Read the [Architecture Overview](../architecture/README.md) to understand the codebase structure.
2. Set up your environment following the [Environment Setup](../setup/environment.md) guide.
3. Confirm the full test suite passes on your machine before making changes.

```bash
git clone https://github.com/flamingo-stack/nats.rs.git
cd nats.rs
cargo test --manifest-path async-nats/Cargo.toml
```

---

## Code Style and Conventions

### Formatting

All code must be formatted with `rustfmt`. CI will reject unformatted code:

```bash
cargo fmt --all
```

The project uses the default `rustfmt` configuration. Do not add custom `rustfmt.toml` overrides without team discussion.

### Linting

All warnings produced by `clippy` are treated as errors. Run clippy before submitting:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Fix all warnings before opening a PR. Do not add `#[allow(clippy::...)]` suppressions without a clear justification in a comment.

### Naming Conventions

Follow standard Rust naming conventions:

| Item | Convention | Example |
|---|---|---|
| Types and traits | `UpperCamelCase` | `ConnectOptions`, `JetStream` |
| Functions and methods | `snake_case` | `connect()`, `publish_with_headers()` |
| Constants | `SCREAMING_SNAKE_CASE` | `DEFAULT_QUEUE_GROUP` |
| Modules | `snake_case` | `auth_utils`, `jetstream` |
| Generics | Short uppercase | `T`, `K`, `V` |

### Error Handling

- Use the crate's `Error<Kind>` type for typed errors on public APIs.
- Do not use `unwrap()` in library code; use `?` propagation or explicit error handling.
- `unwrap()` and `expect()` are acceptable only in tests and examples.

### Async Code

- All async code must be compatible with the Tokio runtime.
- Do not block the Tokio thread pool with synchronous I/O — use `tokio::task::spawn_blocking` if needed.
- Prefer `StreamExt` / `TryStreamExt` combinators over manual `loop` + `next().await` patterns where it improves clarity.

### Documentation

- All `pub` items must have doc comments (`///`).
- Include a short example in doc comments for public API types and functions.
- Use `# Errors` and `# Panics` sections in doc comments where relevant.

```rust
/// Publishes a message to the given subject.
///
/// # Errors
///
/// Returns [`PublishError`] if the payload exceeds the server's max payload limit
/// or if the subject is invalid.
///
/// # Example
///
/// ```rust
/// client.publish("events.created", "payload".into()).await?;
/// ```
pub async fn publish(&self, subject: Subject, payload: Bytes) -> Result<(), PublishError> {
    // ...
}
```

---

## Branch Naming

Use descriptive branch names with a type prefix:

| Prefix | Use For | Example |
|---|---|---|
| `feat/` | New features | `feat/openframe-jwt-refresh` |
| `fix/` | Bug fixes | `fix/reconnect-loop-panic` |
| `docs/` | Documentation only | `docs/jetstream-tutorial` |
| `refactor/` | Code restructuring (no behaviour change) | `refactor/connector-cleanup` |
| `test/` | Adding or fixing tests | `test/kv-watcher-coverage` |
| `chore/` | Dependency updates, CI, build tooling | `chore/update-tokio-1.38` |

Branch names should use `kebab-case`. Avoid using your username or ticket numbers as the primary branch identifier.

---

## Commit Message Format

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```text
<type>(<scope>): <short description>

[optional body]

[optional footer]
```

### Types

| Type | When to Use |
|---|---|
| `feat` | New feature visible to users of the library |
| `fix` | Bug fix |
| `docs` | Documentation changes only |
| `refactor` | Code change that is neither a fix nor a feature |
| `test` | Adding or modifying tests |
| `chore` | Dependency bumps, CI config, build scripts |
| `perf` | Performance improvement |

### Scope (optional)

Use the crate or module name as scope: `async-nats`, `connector`, `jetstream`, `kv`, `tls`, `service`.

### Examples

```text
feat(connector): add auth_url_callback for OpenFrame JWT refresh

Adds a new `auth_url_callback` option to `ConnectOptions` that is invoked
by the Connector when the server returns an Authorization Violation.
The callback should return a fresh server URL containing an updated JWT.

fix(kv): reject keys that start or end with a dot

Validation in `is_valid_key()` now correctly rejects keys like ".bad"
and "also.bad." in addition to keys containing invalid characters.

docs(jetstream): add pull consumer example to module doc comment

test(tls): add test for TLS-first auto-detection mode
```

### Short Description Rules

- Use the imperative mood: "add", "fix", "update", not "added", "fixed", "updated"
- No capital letter at the start
- No period at the end
- Maximum 72 characters

---

## Pull Request Process

### Before Opening a PR

- [ ] All tests pass: `cargo test --manifest-path async-nats/Cargo.toml`
- [ ] Code is formatted: `cargo fmt --all`
- [ ] No clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] New public API items have doc comments with examples
- [ ] New features have integration tests

### PR Description

Use this template for your PR description:

```text
## Summary

What does this PR do? (1-3 sentences)

## Changes

- List the main changes

## Testing

How was this tested? What new tests were added?

## Breaking Changes

Does this introduce any breaking changes to the public API?
```

### Review Process

1. Open a draft PR early if you want feedback on the approach.
2. Request a review when the PR is ready.
3. Address all review comments or explain why they should not be applied.
4. A PR requires at least one approval before merging.
5. Squash-merge is preferred to keep the main branch history clean.

---

## Community

All discussion, questions, and community support happen on the **OpenMSP Slack** — not GitHub Issues or Discussions:

- Join at [https://www.openmsp.ai/](https://www.openmsp.ai/)
- Invite: [https://join.slack.com/t/openmsp/shared_invite/zt-36bl7mx0h-3~U2nFH6nqHqoTPXMaHEHA](https://join.slack.com/t/openmsp/shared_invite/zt-36bl7mx0h-3~U2nFH6nqHqoTPXMaHEHA)

For bugs or feature requests, open a PR directly with a description of the problem and proposed solution, or discuss it in Slack first.

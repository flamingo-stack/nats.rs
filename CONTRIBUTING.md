# Contributing to nats.rs

Thank you for contributing to `nats.rs` — the Flamingo-maintained Rust async client for NATS with OpenFrame JWT integration!

This document covers the code style conventions, branch naming, commit message format, and pull request process for this repository.

---

## Before You Start

1. Read the [Architecture Overview](docs/development/architecture/README.md) to understand the codebase structure.
2. Set up your environment following the [Environment Setup](docs/development/setup/environment.md) guide.
3. Confirm the full test suite passes on your machine before making changes.

```bash
git clone https://github.com/flamingo-stack/nats.rs.git
cd nats.rs
cargo test --manifest-path async-nats/Cargo.toml
```

---

## Development Environment

### Required Toolchain

Install the Rust stable toolchain via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
rustup component add rustfmt clippy
```

Verify:

```bash
rustc --version    # 1.75.0 or newer
cargo --version
rustfmt --version
cargo clippy --version
```

### Recommended IDE

**Visual Studio Code** with the `rust-analyzer` extension is the recommended setup. Suggested settings (`.vscode/settings.json`):

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true,
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.inlayHints.chainingHints.enable": true
}
```

**JetBrains RustRover** and **Neovim** with `nvim-lspconfig` + `rust_analyzer` are also fully supported.

### Useful Environment Variables

| Variable | Purpose |
|---|---|
| `RUST_LOG` | Enables tracing output (e.g., `RUST_LOG=async_nats=debug`) |
| `RUST_BACKTRACE` | Enables full backtraces on panic (`RUST_BACKTRACE=1`) |

---

## Code Style and Conventions

### Formatting

All code must be formatted with `rustfmt`. CI will reject unformatted code:

```bash
cargo fmt --all
```

### Linting

All `clippy` warnings are treated as errors. Run clippy before submitting:

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
- Do not use `unwrap()` in library code — use `?` propagation or explicit error handling.
- `unwrap()` and `expect()` are acceptable only in tests and examples.

### Async Code

- All async code must be compatible with the Tokio runtime.
- Do not block the Tokio thread pool with synchronous I/O — use `tokio::task::spawn_blocking` if needed.
- Prefer `StreamExt` / `TryStreamExt` combinators over manual `loop` + `next().await` patterns where it improves clarity.

### Documentation

All `pub` items must have doc comments (`///`). Include a short example for public API types and functions. Use `# Errors` and `# Panics` sections where relevant:

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

**Scope** (optional): use the crate or module name — `async-nats`, `connector`, `jetstream`, `kv`, `tls`, `service`.

### Examples

```text
feat(connector): add auth_url_callback for OpenFrame JWT refresh

fix(kv): reject keys that start or end with a dot

docs(jetstream): add pull consumer example to module doc comment

test(tls): add test for TLS-first auto-detection mode
```

### Short Description Rules

- Use the imperative mood: "add", "fix", "update" — not "added", "fixed", "updated"
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

### PR Description Template

```text
## Summary

What does this PR do? (1–3 sentences)

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

## Testing

Tests require a `nats-server` binary on your `$PATH`. The `nats_test_server` helper crate automatically manages server lifecycle for integration tests.

```bash
# Full test suite
cargo test --manifest-path async-nats/Cargo.toml

# Specific test file
cargo test --manifest-path async-nats/Cargo.toml --test client_tests

# Specific test by name
cargo test --manifest-path async-nats/Cargo.toml -- basic_pub_sub

# With server_2_10 features
cargo test --manifest-path async-nats/Cargo.toml --features server_2_10

# With debug logging
RUST_LOG=async_nats=debug cargo test --manifest-path async-nats/Cargo.toml -- --nocapture

# Using cargo-nextest (faster)
cargo nextest run --manifest-path async-nats/Cargo.toml
```

### Writing New Tests

All integration tests are `async` and use `#[tokio::test]`. Add new tests to the appropriate file in `async-nats/tests/`:

```rust
#[tokio::test]
async fn my_new_feature_test() {
    let server = nats_server::run_server("tests/configs/jetstream.conf");
    let client = async_nats::connect(server.client_url())
        .await
        .unwrap();

    // Exercise the feature and assert expectations
}
```

---

## Security Checklist for Auth / Connection PRs

Before merging any PR that touches authentication or connection code:

- [ ] No credentials or secrets hard-coded in source files or tests
- [ ] Test credentials use clearly fake placeholder values (not real tokens)
- [ ] All new `ConnectOptions` fields accepting credential material use appropriate visibility
- [ ] Any new auth callback accepts and returns the correct `Auth` type
- [ ] TLS configuration is not silently downgraded
- [ ] Subject inputs are validated or typed as `Subject` before use
- [ ] Error messages do not leak credential content

---

## Community

Flamingo does not use GitHub Issues or GitHub Discussions. All support, questions, and community conversation happen on the **OpenMSP Slack**:

- Join at [https://www.openmsp.ai/](https://www.openmsp.ai/)
- Invite: [https://join.slack.com/t/openmsp/shared_invite/zt-36bl7mx0h-3~U2nFH6nqHqoTPXMaHEHA](https://join.slack.com/t/openmsp/shared_invite/zt-36bl7mx0h-3~U2nFH6nqHqoTPXMaHEHA)

For bugs or feature requests, open a PR directly with a description of the problem and proposed solution, or discuss it in Slack first.

---

<div align="center">
  Built with 💛 by the <a href="https://www.flamingo.run/about"><b>Flamingo</b></a> team
</div>

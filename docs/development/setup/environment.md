# Development Environment Setup

This guide covers the tools, IDE configuration, and editor extensions recommended for developing with `nats.rs`.

## Required Toolchain

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

## Recommended IDE

### Visual Studio Code

VS Code with the **rust-analyzer** extension is the recommended setup:

1. Install [VS Code](https://code.visualstudio.com/)
2. Install the [rust-analyzer extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

Suggested VS Code settings for this project (`.vscode/settings.json`):

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true,
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.inlayHints.chainingHints.enable": true
}
```

### RustRover (JetBrains)

[RustRover](https://www.jetbrains.com/rust/) is a dedicated Rust IDE from JetBrains with first-class support for Cargo workspaces, integrated debugging, and test running. No plugins are required.

### Neovim / Vim

Use [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig) with the `rust_analyzer` language server. Install `rust-analyzer` via `rustup`:

```bash
rustup component add rust-analyzer
```

## Recommended VS Code Extensions

| Extension | Purpose |
|---|---|
| `rust-lang.rust-analyzer` | Rust language support, code completion, inline docs |
| `vadimcn.vscode-lldb` | LLDB debugger integration for Rust |
| `tamasfe.even-better-toml` | Syntax highlighting and schema validation for `Cargo.toml` |
| `serayuzgur.crates` | Inline version info and update suggestions for crate dependencies |
| `usernamehw.errorlens` | Inline error and warning display |

## Additional Development Tools

### NATS Server

Install the NATS server for local testing:

```bash
# macOS
brew install nats-server

# Linux — download from GitHub releases
curl -L https://github.com/nats-io/nats-server/releases/download/v2.10.4/nats-server-v2.10.4-linux-amd64.tar.gz | tar xz
sudo mv nats-server-v2.10.4-linux-amd64/nats-server /usr/local/bin/
```

### NATS CLI

The NATS CLI (`nats`) is invaluable for inspecting subjects, KV buckets, streams, and services interactively:

```bash
# macOS
brew install nats-io/nats-tools/nats

# Go install
go install github.com/nats-io/natscli/nats@latest
```

### cargo-nextest

`cargo-nextest` is a faster test runner that is especially useful for the integration test suite:

```bash
cargo install cargo-nextest
```

### tokio-console

`tokio-console` provides a real-time dashboard of Tokio tasks and is useful when debugging async behaviour:

```bash
cargo install tokio-console
```

## Environment Variables for Development

No mandatory environment variables are required to build the library. The following are useful during development and testing:

| Variable | Purpose |
|---|---|
| `RUST_LOG` | Enables tracing output (e.g., `RUST_LOG=debug`) |
| `RUST_BACKTRACE` | Enables full backtraces on panic (`RUST_BACKTRACE=1`) |

Set them inline when running tests:

```bash
RUST_LOG=async_nats=debug cargo test --manifest-path async-nats/Cargo.toml
```

## Linting and Formatting

The project uses `rustfmt` for formatting and `clippy` for linting. Both are enforced in CI.

```bash
# Format all code
cargo fmt --all

# Run clippy with workspace feature flags
cargo clippy --all-targets --all-features -- -D warnings
```

Configure `rustfmt` behaviour in `rustfmt.toml` or `async-nats/.rustfmt.toml` at the crate root.

## Build Variants

```bash
# Debug build (fast compile, slow runtime)
cargo build --manifest-path async-nats/Cargo.toml

# Release build (slow compile, fast runtime — use for benchmarks)
cargo build --release --manifest-path async-nats/Cargo.toml

# Build with all features enabled
cargo build --manifest-path async-nats/Cargo.toml --all-features
```

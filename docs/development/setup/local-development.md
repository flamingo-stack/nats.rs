# Local Development Guide

This guide covers cloning the repository, building the codebase, running examples, and working with the full development workflow.

## Clone and Setup

```bash
# Clone the repository
git clone https://github.com/flamingo-stack/nats.rs.git
cd nats.rs

# Verify the workspace builds
cargo build --workspace
```

The workspace root contains two main crates:

```text
nats.rs/
├── async-nats/          # Primary async client (Tokio-based) — focus of development
│   ├── src/             # Library source code
│   ├── examples/        # Runnable examples
│   ├── tests/           # Integration tests
│   └── benches/         # Benchmarks
├── nats/                # Legacy synchronous client (deprecated)
└── nats-server/         # Test helper crate wrapping nats-server binary
```

> **Note:** All new development should target the `async-nats` crate. The `nats` crate is deprecated and no longer actively maintained.

## Start a Local NATS Server

Before running examples or integration tests, start a NATS server with JetStream enabled:

```bash
# Plain NATS (Core pub/sub only)
nats-server

# With JetStream enabled (required for JetStream, KV, Object Store tests)
nats-server -js

# With JetStream and a custom config (used by integration tests)
nats-server -c async-nats/tests/configs/jetstream.conf
```

## Building the async-nats Crate

```bash
# Standard debug build
cargo build --manifest-path async-nats/Cargo.toml

# All features enabled
cargo build --manifest-path async-nats/Cargo.toml --all-features

# Release build (required for accurate benchmark results)
cargo build --release --manifest-path async-nats/Cargo.toml
```

## Running Examples

The `async-nats/examples/` directory contains ready-to-run examples demonstrating the main API surfaces:

```bash
# Publish benchmark (publishes 10M messages)
cargo run --example pub --manifest-path async-nats/Cargo.toml

# Subscribe example
cargo run --example sub --manifest-path async-nats/Cargo.toml

# Concurrent publish/subscribe
cargo run --example concurrent --manifest-path async-nats/Cargo.toml

# JetStream pull consumer
cargo run --example jetstream_pull --manifest-path async-nats/Cargo.toml

# JetStream push consumer
cargo run --example jetstream_push --manifest-path async-nats/Cargo.toml

# Key-Value store
cargo run --example kv --manifest-path async-nats/Cargo.toml

# JSON payload with serde
cargo run --example json --manifest-path async-nats/Cargo.toml

# Multiple concurrent subscriptions
cargo run --example multiple_subs --manifest-path async-nats/Cargo.toml

# Async context bridge (sync wrapper)
cargo run --example sync_context --manifest-path async-nats/Cargo.toml
```

All examples expect a NATS server running on `nats://localhost:4222` unless otherwise specified.

## Running Integration Tests

Integration tests require a running NATS server. The `nats_test_server` helper crate automatically starts and stops server instances for most tests:

```bash
# Run the full async-nats test suite
cargo test --manifest-path async-nats/Cargo.toml

# Run a specific test
cargo test --manifest-path async-nats/Cargo.toml -- basic_pub_sub

# Run TLS tests (requires cert files in async-nats/tests/configs/certs/)
cargo test --manifest-path async-nats/Cargo.toml -- tls

# Run tests with server_2_10 features
cargo test --manifest-path async-nats/Cargo.toml --features server_2_10

# Run tests with verbose output
cargo test --manifest-path async-nats/Cargo.toml -- --nocapture
```

### Using cargo-nextest

```bash
# Faster test execution with nextest
cargo nextest run --manifest-path async-nats/Cargo.toml

# Run only tests matching a pattern
cargo nextest run --manifest-path async-nats/Cargo.toml -E 'test(jetstream)'
```

## Running Benchmarks

```bash
# Core NATS benchmarks
cargo bench --manifest-path async-nats/Cargo.toml --bench core_nats

# JetStream benchmarks
cargo bench --manifest-path async-nats/Cargo.toml --bench jetstream
```

> Always run benchmarks with a release build (`--release` is implicit with `cargo bench`).

## Debugging

### Enable Tracing Output

Set `RUST_LOG` to see debug output from `async-nats`:

```bash
RUST_LOG=async_nats=debug cargo run --example sub --manifest-path async-nats/Cargo.toml
```

Available log levels: `error`, `warn`, `info`, `debug`, `trace`.

### Enable Backtraces

```bash
RUST_BACKTRACE=1 cargo test --manifest-path async-nats/Cargo.toml -- basic_pub_sub
```

### Debug Configuration in VS Code

Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug example: pub",
      "cargo": {
        "args": ["build", "--example", "pub", "--manifest-path", "async-nats/Cargo.toml"],
        "filter": { "name": "pub", "kind": "example" }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

## Hot Reload / Watch Mode

Use `cargo-watch` to automatically recompile on file changes:

```bash
# Install
cargo install cargo-watch

# Watch and re-run tests on changes
cargo watch -x "test --manifest-path async-nats/Cargo.toml"

# Watch and re-run an example
cargo watch -x "run --example sub --manifest-path async-nats/Cargo.toml"
```

## Formatting and Linting

Always run these before committing:

```bash
# Format code
cargo fmt --all

# Lint with clippy
cargo clippy --all-targets --all-features -- -D warnings
```

CI will fail if either check produces errors or warnings.

# Prerequisites

Before working with `nats.rs`, ensure your development environment meets the requirements below.

## Required Software

| Tool | Minimum Version | Purpose |
|---|---|---|
| [Rust toolchain](https://www.rust-lang.org/tools/install) | 1.75+ (stable) | Building and running Rust code |
| `cargo` | Ships with Rust | Package manager and build tool |
| [NATS Server](https://docs.nats.io/running-a-nats-service/introduction/installation) | 2.9+ | Local server for development and testing |
| Git | 2.x | Cloning the repository |

> **Note:** JetStream tests require a NATS Server with JetStream enabled (version 2.9 or later). Some tests are gated behind feature flags `server_2_10` and `server_2_11`, which require the corresponding server versions.

## Rust Toolchain Setup

The recommended way to install and manage the Rust toolchain is via `rustup`:

```bash
# Install rustup and the stable Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the stable toolchain (if not already the default)
rustup toolchain install stable

# Verify the installation
rustc --version
cargo --version
```

## NATS Server Installation

### macOS (Homebrew)

```bash
brew install nats-server
```

### Linux / Other Platforms

Download the latest binary from the [NATS Server releases page](https://github.com/nats-io/nats-server/releases) and place it on your `$PATH`:

```bash
# Example for Linux amd64 — adjust version/arch as needed
curl -L https://github.com/nats-io/nats-server/releases/download/v2.10.4/nats-server-v2.10.4-linux-amd64.tar.gz | tar xz
sudo mv nats-server-v2.10.4-linux-amd64/nats-server /usr/local/bin/
```

### Verify Installation

```bash
nats-server --version
```

## System Requirements

| Resource | Minimum | Recommended |
|---|---|---|
| CPU | Any modern x86_64 or ARM64 | Multi-core for benchmarking |
| RAM | 256 MB | 1 GB+ (for JetStream streams with file storage) |
| Disk | Minimal | SSD recommended for JetStream file storage |
| OS | Linux, macOS, Windows | Linux or macOS for full test suite |

## Cargo Feature Flags

The `async-nats` crate uses feature flags to unlock functionality tied to specific NATS Server versions. When declaring the dependency in your project's `Cargo.toml`, you can enable the features you need:

```toml
[dependencies]
async-nats = { version = "0.33", features = ["server_2_10", "server_2_11"] }
```

| Feature Flag | Unlocks |
|---|---|
| `server_2_10` | KV bucket compression, stream compression |
| `server_2_11` | Per-key TTL (`create_with_ttl`), consumer pause, stream limit markers |
| `service` | NATS Service API (`$SRV` prefix) |

## Optional Tools

| Tool | Purpose |
|---|---|
| [NATS CLI (`nats`)](https://github.com/nats-io/natscli) | Inspect subjects, streams, and KV buckets interactively |
| `cargo-nextest` | Faster test runner for the test suite |
| `tokio-console` | Async task profiling for Tokio applications |

### Install NATS CLI

```bash
# macOS
brew install nats-io/nats-tools/nats

# Go install
go install github.com/nats-io/natscli/nats@latest
```

## Environment Variables

No mandatory environment variables are required to build the library itself. During testing, the test suite spins up a local NATS server automatically using the `nats_test_server` helper crate.

For connecting to secured NATS servers or OpenFrame environments, you will typically supply credentials at runtime via `ConnectOptions`. See the [Quick Start](quick-start.md) guide for examples.

## Verifying Your Environment

Run the following commands to confirm everything is in place before cloning the repository:

```bash
# Rust toolchain
rustc --version    # Should print: rustc 1.75.0 (or newer)
cargo --version    # Should print: cargo 1.75.0 (or newer)

# NATS Server
nats-server --version   # Should print: nats-server: v2.9.x (or newer)

# Git
git --version      # Should print: git version 2.x.x
```

If all commands return the expected output, you are ready to proceed to the [Quick Start](quick-start.md).

# Testing Overview

This document describes the test structure of `nats.rs`, how to run the test suite, how to write new tests, and what coverage targets to aim for.

---

## Test Structure and Organisation

Tests are co-located with their respective crates. The primary test surface is in `async-nats/tests/`:

```text
async-nats/
├── tests/
│   ├── client_tests.rs      # Core pub/sub, request/reply, reconnection, subscription lifecycle
│   ├── jetstream_tests.rs   # JetStream stream creation, publishing, consuming, account info
│   ├── kv_tests.rs          # Key-Value store operations and watcher behaviour
│   ├── object_store.rs      # Object Store put/get/delete and SHA-256 integrity
│   ├── service_tests.rs     # Service API registration, discovery, and lifecycle
│   ├── tls_tests.rs         # TLS, mTLS, TLS-first, and native cert store
│   ├── jwt_tests.rs         # JWT / credentials auth and reconnect with token refresh
│   ├── nkey_tests.rs        # NKey seed authentication
│   ├── compatibility.rs     # Cross-version compatibility tests
│   └── websocket_test.rs    # WebSocket transport
```

All tests in this directory are **integration tests** — they connect to a real NATS server via the `nats_test_server` helper crate.

### Test Helper: `nats_test_server`

The `nats/nats_test_server/` crate provides utilities that start and stop real `nats-server` processes during tests:

```rust
// Start a basic server (Core NATS only)
let server = nats_server::run_basic_server();

// Start a server with JetStream enabled
let server = nats_server::run_server("tests/configs/jetstream.conf");

// Get the client URL
let url = server.client_url();
```

The server is automatically stopped when the `server` handle is dropped.

---

## Running Tests

### Prerequisites

Ensure a `nats-server` binary is on your `$PATH` and the `async-nats` workspace builds successfully:

```bash
nats-server --version
cargo build --manifest-path async-nats/Cargo.toml
```

### Run the Full Test Suite

```bash
cargo test --manifest-path async-nats/Cargo.toml
```

### Run a Specific Test File

```bash
# Only client tests
cargo test --manifest-path async-nats/Cargo.toml --test client_tests

# Only JetStream tests
cargo test --manifest-path async-nats/Cargo.toml --test jetstream_tests

# Only TLS tests
cargo test --manifest-path async-nats/Cargo.toml --test tls_tests
```

### Run a Specific Test by Name

```bash
cargo test --manifest-path async-nats/Cargo.toml -- basic_pub_sub
cargo test --manifest-path async-nats/Cargo.toml -- force_reconnect
```

### Run Tests with Feature Flags

Some tests are gated behind server-version features:

```bash
# Enable server_2_10 features (requires NATS Server 2.10+)
cargo test --manifest-path async-nats/Cargo.toml --features server_2_10

# Enable all features
cargo test --manifest-path async-nats/Cargo.toml --all-features
```

### Run Tests with Debug Logging

```bash
RUST_LOG=async_nats=debug cargo test --manifest-path async-nats/Cargo.toml -- --nocapture
```

### Using cargo-nextest

`cargo-nextest` provides faster test execution and better failure reporting:

```bash
# Install
cargo install cargo-nextest

# Run all tests
cargo nextest run --manifest-path async-nats/Cargo.toml

# Filter by test name
cargo nextest run --manifest-path async-nats/Cargo.toml -E 'test(jetstream)'

# Run with feature flags
cargo nextest run --manifest-path async-nats/Cargo.toml --all-features
```

---

## Writing New Tests

### Integration Test Template

Add a new test to the appropriate file in `async-nats/tests/`. All tests are `async` and use `#[tokio::test]`:

```rust
#[tokio::test]
async fn my_new_feature_test() {
    // 1. Start a test server
    let server = nats_server::run_server("tests/configs/jetstream.conf");

    // 2. Connect a client
    let client = async_nats::connect(server.client_url())
        .await
        .unwrap();

    // 3. Exercise the feature
    let js = async_nats::jetstream::new(client.clone());
    js.get_or_create_stream(async_nats::jetstream::stream::Config {
        name: "MY_STREAM".to_string(),
        subjects: vec!["my.subject".to_string()],
        ..Default::default()
    })
    .await
    .unwrap();

    js.publish("my.subject", "test-payload".into())
        .await
        .unwrap();

    // 4. Assert expectations
    let stream = js.get_stream("MY_STREAM").await.unwrap();
    let info = stream.info().await.unwrap();
    assert_eq!(info.state.messages, 1);
}
```

### Testing TLS Scenarios

TLS tests require certificate files. The repository ships with test certificates in `async-nats/tests/configs/certs/`. Reference them using `CARGO_MANIFEST_DIR`:

```rust
#[tokio::test]
async fn my_tls_test() {
    let server = nats_server::run_server("tests/configs/tls.conf");
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let client = async_nats::ConnectOptions::new()
        .add_root_certificates(path.join("tests/configs/certs/rootCA.pem"))
        .require_tls(true)
        .connect(server.client_url())
        .await
        .unwrap();

    assert!(client.server_info().tls_required);
}
```

### Testing Authentication

```rust
#[tokio::test]
async fn my_auth_test() {
    let server = nats_server::run_server("tests/configs/auth.conf");

    let client = async_nats::ConnectOptions::with_user_and_password(
        "alice".into(),
        "secret".into(),
    )
    .connect(server.client_url())
    .await
    .unwrap();

    // Verify the connection succeeded
    assert!(client.connection_state() == async_nats::connection::State::Connected);
}
```

### Feature-Gated Tests

Use `#[cfg(feature = "server_2_10")]` for tests that require a specific server version:

```rust
#[cfg(feature = "server_2_10")]
#[tokio::test]
async fn test_kv_compression() {
    // ...
}
```

---

## Test Configuration Files

The `async-nats/tests/configs/` directory contains NATS server configuration files used by the test suite:

| Config File | Purpose |
|---|---|
| `jetstream.conf` | JetStream enabled server |
| `tls.conf` | Server with TLS required |
| `auth.conf` | Server with username/password auth |
| `certs/rootCA.pem` | Test root CA certificate |
| `certs/client-cert.pem` | Test client certificate |
| `certs/client-key.pem` | Test client private key |

---

## Coverage Guidelines

While there is no hard coverage percentage enforced, aim to:

- Cover the **happy path** and at least one **error path** for every public API method
- Write integration tests for any new `ConnectOptions` field
- Add tests for any new JetStream, KV, Object Store, or Service API feature
- Include a test for any bug fix that verifies the fix and prevents regression

Run tests with verbose output to identify which tests are not yet covered:

```bash
cargo test --manifest-path async-nats/Cargo.toml -- --nocapture 2>&1 | grep "test result"
```

---

## Ignored Tests

Some tests are marked `#[ignore]` because they require infrastructure not available in the standard CI environment (e.g., a hardcoded local server). To run them explicitly:

```bash
cargo test --manifest-path async-nats/Cargo.toml -- --ignored
```

Review the test comments before running ignored tests to understand what external setup they require.

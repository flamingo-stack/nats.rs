# Security Guidelines

This document describes security best practices for working with `nats.rs` — covering authentication patterns, TLS configuration, credential management, input validation, and secure coding guidelines.

---

## Authentication Patterns

`async-nats` supports multiple authentication mechanisms. Use the most appropriate one for your deployment context.

### Token Authentication

Simple shared-secret token, suitable for low-stakes internal services:

```rust
let client = async_nats::ConnectOptions::with_token("my-secret-token".into())
    .connect("nats://localhost:4222")
    .await?;
```

> **Warning:** Never hard-code tokens in source code. Always load them from environment variables or a secrets manager.

### Username and Password

```rust
let client = async_nats::ConnectOptions::with_user_and_password(
    std::env::var("NATS_USER").expect("NATS_USER not set").into(),
    std::env::var("NATS_PASSWORD").expect("NATS_PASSWORD not set").into(),
)
.connect("nats://localhost:4222")
.await?;
```

### NKey Authentication

NKeys provide cryptographic authentication without transmitting the private key. Only the public key is shared with the server:

```rust
let client = async_nats::ConnectOptions::with_nkey(
    std::env::var("NATS_NKEY_SEED").expect("NATS_NKEY_SEED not set").into(),
)
.connect("nats://localhost:4222")
.await?;
```

### JWT with Credentials File

The `.creds` file format bundles a JWT and NKey seed together. This is the recommended method for NATS Accounts-based deployments:

```rust
let client =
    async_nats::ConnectOptions::with_credentials_file("/run/secrets/my-user.creds")
        .await?
        .connect("nats://localhost:4222")
        .await?;
```

> **Best practice:** Mount credentials files from a secrets manager (e.g., Vault, Kubernetes Secrets) rather than baking them into container images.

### JWT with Async Callback (OpenFrame Integration)

This is the Flamingo-specific extension. On `Authorization Violation`, the `Connector` calls `auth_url_callback` to obtain a fresh JWT from the OpenFrame Auth API:

```rust
let refresh_token = std::env::var("OF_REFRESH_TOKEN").unwrap();

let client = async_nats::ConnectOptions::new()
    .auth_url_callback(move || {
        let token = refresh_token.clone();
        async move {
            // Call OpenFrame Auth API to exchange refresh token for JWT
            let jwt = fetch_jwt_from_openframe(&token).await?;
            Ok(jwt)
        }
    })
    .connect("nats://my-openframe-server:4222")
    .await?;
```

---

## TLS Configuration

Always use TLS in production. The `async-nats` crate uses `rustls` exclusively — no OpenSSL dependency.

### Require TLS (Server Certificate Verification)

```rust
let client = async_nats::ConnectOptions::new()
    .require_tls(true)
    .connect("tls://my-nats-server:4222")
    .await?;
```

### Mutual TLS (mTLS) with Custom CA

```rust
use std::path::PathBuf;

let client = async_nats::ConnectOptions::new()
    .add_root_certificates(PathBuf::from("/etc/nats/certs/rootCA.pem"))
    .add_client_certificate(
        PathBuf::from("/etc/nats/certs/client-cert.pem"),
        PathBuf::from("/etc/nats/certs/client-key.pem"),
    )
    .require_tls(true)
    .connect("tls://my-nats-server:4222")
    .await?;
```

> **mTLS rule:** Always supply both `client_cert` AND `client_key` together. Supplying a cert without a key returns an error.

### Custom `rustls::ClientConfig`

For advanced scenarios (FIPS mode, custom cipher suites), supply a pre-built `rustls::ClientConfig`:

```rust
use async_nats::rustls;

let root_store = rustls::RootCertStore::empty();
// ... populate root_store ...
let config = rustls::ClientConfig::builder()
    .with_root_certificates(root_store)
    .with_no_client_auth();

let client = async_nats::ConnectOptions::new()
    .tls_client_config(config)
    .connect("tls://my-nats-server:4222")
    .await?;
```

### TLS-First Mode

If the server is configured for `tls_first` (TLS negotiation before the NATS INFO handshake):

```rust
let client = async_nats::ConnectOptions::new()
    .require_tls(true)
    .tls_first()
    .connect("tls://my-nats-server:4222")
    .await?;
```

---

## Secrets and Environment Variables Management

| Secret | Recommended Storage |
|---|---|
| NATS tokens and passwords | Environment variable or secrets manager (Vault, AWS SSM) |
| NKey seeds | Kubernetes Secret or mounted volume (mode `0400`) |
| `.creds` files | Mounted Kubernetes Secret volume or Vault agent sidecar |
| TLS private keys | Mounted volume (mode `0400`), never in container image layers |
| OpenFrame refresh tokens | Environment variable injected at runtime, never in source code |

### Loading from Environment Variables

```rust
let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
let nats_token = std::env::var("NATS_TOKEN").ok();

let opts = if let Some(token) = nats_token {
    async_nats::ConnectOptions::with_token(token)
} else {
    async_nats::ConnectOptions::new()
};

let client = opts.connect(&nats_url).await?;
```

---

## Input Validation

### Subject Validation

NATS subject names have strict rules. Use the `Subject` type to enforce validity at the type level:

```rust
use async_nats::Subject;
use std::str::FromStr;

// This will fail if the subject is malformed
let subject = Subject::from_str("events.user.created")?;
client.publish(subject, payload).await?;
```

Invalid subject characters include embedded null bytes and certain reserved patterns. The `Subject` type validates these constraints at construction time.

### JetStream Name Validation

Stream and consumer names must not contain whitespace, `.`, `*`, or `>`. The library enforces this via `is_valid_name()` before making API calls.

### KV Key and Bucket Validation

- Valid bucket names: `[a-zA-Z0-9_-]+`
- Valid keys: `[-/_=.a-zA-Z0-9]+` (cannot start or end with `.`)

The library validates these before API calls and returns errors for invalid inputs.

---

## Common Security Vulnerabilities and Mitigations

| Vulnerability | Mitigation |
|---|---|
| Credential exposure in logs | Never log `Auth` struct contents; `Auth` does not implement `Display` by design |
| Hard-coded secrets in source | Always load credentials from environment variables or secrets managers |
| Unauthenticated NATS connections | Always configure authentication for non-local-dev environments |
| Unencrypted transport | Always use `tls://` URLs and `require_tls(true)` in production |
| Over-broad subject access | Use NATS Accounts and user-level permission scopes to limit pub/sub surface |
| Token reuse after rotation | Use the `auth_url_callback` mechanism for automatic JWT rotation |

---

## Security Testing

The repository includes dedicated TLS integration tests in `async-nats/tests/tls_tests.rs`:

```bash
# Run all TLS tests
cargo test --manifest-path async-nats/Cargo.toml -- tls

# Run JWT authentication tests
cargo test --manifest-path async-nats/Cargo.toml -- jwt
```

These tests cover:

- Basic mTLS with root CA and client certificate
- TLS with IP SAN certificates
- Rejection of unknown server CAs
- Loading certs from a single bundled PEM file
- Custom `rustls::ClientConfig` injection
- TLS-first handshake mode
- TLS-auto (server accepts both modes)

---

## Code Review Security Checklist

Before merging any PR that touches authentication or connection code:

- [ ] No credentials or secrets hard-coded in source files or tests
- [ ] Test credentials use clearly fake placeholder values (not real tokens)
- [ ] All new `ConnectOptions` fields that accept credential material are marked `pub(crate)` or use appropriate visibility
- [ ] Any new auth callback accepts and returns the correct `Auth` type
- [ ] TLS configuration is not silently downgraded (e.g., `require_tls` not accidentally set to `false`)
- [ ] Subject inputs are validated or typed as `Subject` before use
- [ ] Error messages do not leak credential content

# Architecture Overview

This document describes the high-level architecture of the `async-nats` crate — the primary Rust async client for NATS in this repository.

For detailed module-level documentation, see the [reference architecture docs](../../reference/architecture/overview.md).

---

## Design Goals

- **Async-first:** Built on [Tokio](https://tokio.rs/), every operation is non-blocking.
- **Cloneable client handle:** `Client` is cheaply cloneable via `Arc`-backed internals, safe to share across tasks without wrapping in a `Mutex`.
- **Layered API surface:** Core pub/sub is simple; JetStream, KV, Object Store, and the Service API are all built on top.
- **OpenFrame JWT integration:** The `Connector` handles automatic JWT refresh via `auth_url_callback` on authorization failures.

---

## High-Level System Design

```mermaid
graph TD
    App["Application / OpenFrame Service"]
    ConnOpts["ConnectOptions (builder)"]
    Auth["Auth struct (credentials)"]
    AuthCB["auth_url_callback (JWT refresh)"]
    OFAuth["OpenFrame Auth API"]
    Connector["Connector (reconnect loop)"]
    Connection["Connection (TCP / TLS / WS)"]
    NATS["NATS Server"]
    Client["Client (cloneable handle)"]
    JetStream["JetStream Context"]
    KV["Key-Value Store"]
    ObjStore["Object Store"]
    SvcAPI["Service API"]

    App --> ConnOpts
    ConnOpts --> Auth
    ConnOpts --> AuthCB
    AuthCB --> OFAuth
    ConnOpts --> Connector
    Connector --> Connection
    Connection --> NATS
    Connector --> Client
    Client --> JetStream
    JetStream --> KV
    JetStream --> ObjStore
    Client --> SvcAPI
    App --> Client
```

---

## Core Components

| Module | File | Responsibility |
|---|---|---|
| `ConnectOptions` | `src/options.rs` | Builder for all connection config: TLS, auth, timeouts, callbacks |
| `Client` | `src/client.rs` | Cloneable pub/sub/request handle; wraps channels to connection task |
| `Connector` | `src/connector.rs` | Server list management, reconnect loop, auth error recovery, JWT refresh |
| `Connection` | `src/connection.rs` | Framed async TCP/TLS/WebSocket I/O; NATS protocol parser |
| `Auth` | `src/auth.rs` | Credential container: JWT, nkey, signature callback, username/password, token |
| `auth_utils` | `src/auth_utils.rs` | `.creds` file parser: regex-based JWT and nkey seed extraction |
| `tls` | `src/tls.rs` | `rustls::ClientConfig` builder; native certs, user certs, mTLS |
| `Message` | `src/message.rs` | Core message struct: subject, reply, payload, headers, status |
| `Subject` | `src/subject.rs` | UTF-8–validated, immutable subject string type |
| `HeaderMap` | `src/header.rs` | NATS message headers: insert, append, get, iterate |
| `StatusCode` | `src/status.rs` | NATS numeric status codes (100–999) with semantic helpers |
| `Error<Kind>` | `src/error.rs` | Generic typed error with optional source chaining |
| `jetstream/` | `src/jetstream/` | Streams, consumers (push/pull/ordered), KV, Object Store, Service API |

---

## Component Relationships

```mermaid
graph LR
    ConnectOptions["ConnectOptions"]
    AuthStruct["Auth struct"]
    Connector["Connector"]
    Connection["Connection"]
    Client["Client"]
    JS["JetStream Context"]
    KV["KV Store"]
    ObjStore["Object Store"]
    SvcAPI["Service API"]
    TLS["tls::config_tls"]
    AuthUtils["auth_utils"]
    Crypto["crypto::Sha256"]

    ConnectOptions --> AuthStruct
    ConnectOptions --> Connector
    ConnectOptions --> TLS
    AuthStruct --> AuthUtils
    Connector --> Connection
    Connector --> TLS
    Connection --> Crypto
    Connector --> Client
    Client --> JS
    JS --> KV
    JS --> ObjStore
    JS --> SvcAPI
```

---

## Data Flow

### Connection Establishment and JWT Token Refresh

This is the Flamingo-specific extension on top of upstream NATS: when the server returns an `Authorization Violation` error, the `Connector` invokes `auth_url_callback` to obtain a fresh JWT from the OpenFrame Auth API and reconnects automatically.

```mermaid
sequenceDiagram
    participant App as Application
    participant Opts as ConnectOptions
    participant Conn as Connector
    participant TCP as Connection (TCP/TLS)
    participant NATS as NATS Server
    participant AuthCB as auth_url_callback
    participant OFAuth as OpenFrame Auth API

    App->>Opts: build with jwt_token_callback / credentials
    Opts->>Conn: construct Connector with ConnectorOptions
    Conn->>TCP: open TCP / TLS / WebSocket stream
    TCP->>NATS: send CONNECT info (JWT, nkey sig, token)
    NATS-->>TCP: +OK or -ERR Authorization Violation
    alt Authorization Violation
        TCP-->>Conn: ConnectErrorKind::AuthorizationViolation
        Conn->>AuthCB: call auth_url_callback()
        AuthCB->>OFAuth: POST /api/oauth/token
        OFAuth-->>AuthCB: fresh JWT
        AuthCB-->>Conn: updated Auth
        Conn->>TCP: reconnect with fresh credentials
        TCP->>NATS: send CONNECT info (new JWT)
        NATS-->>TCP: +OK
    end
    TCP-->>Conn: ServerInfo
    Conn-->>App: Client handle
```

### Publish / Subscribe Message Flow

```mermaid
sequenceDiagram
    participant Pub as Publisher (Client)
    participant Chan as mpsc::Sender
    participant ConnTask as Connection Task
    participant NATS as NATS Server
    participant Sub as Subscriber (Client)

    Pub->>Chan: Command::Publish(subject, payload)
    Chan->>ConnTask: dequeue Command
    ConnTask->>NATS: PUB subject payload CRLF
    NATS-->>ConnTask: MSG subject sid payload CRLF
    ConnTask->>Sub: route via subscription channel
    Sub-->>Pub: Message {subject, payload, headers}
```

---

## JetStream Subsystem

JetStream is built on top of the core `Client` and adds persistence, replay, consumers, and higher-level stores. The `jetstream::Context` is the entry point.

```mermaid
graph TD
    Context["JetStream Context"]
    Stream["Stream"]
    PullConsumer["PullConsumer"]
    PushConsumer["PushConsumer"]
    OrderedPull["OrderedPullConsumer"]
    OrderedPush["OrderedPushConsumer"]
    KV["KV Store (create_key_value)"]
    ObjStore["Object Store (create_object_store)"]

    Context --> Stream
    Stream --> PullConsumer
    Stream --> PushConsumer
    Stream --> OrderedPull
    Stream --> OrderedPush
    Context --> KV
    Context --> ObjStore
```

---

## Key Design Decisions

1. **Channels over locks:** `Client` communicates with the background `Connection` task via `mpsc` channels rather than shared-mutable state. This avoids locks on the hot publish path.

2. **Exponential backoff reconnect:** `Connector` uses a configurable `reconnect_delay_callback` (default: `0ms` first attempt, then 2^(n-1) ms capped at 4 s) rather than a fixed retry interval.

3. **Write coalescing:** `Connection` coalesces writes into a vectored I/O buffer and defers flushing until `should_flush()` signals it is needed, reducing syscall overhead at high throughput.

4. **Feature-gated server capabilities:** API surface that depends on specific NATS Server versions (e.g., KV compression, consumer pause) is gated behind Cargo feature flags (`server_2_10`, `server_2_11`) to prevent calling unsupported APIs at compile time.

5. **rustls exclusively:** TLS is implemented with `rustls` (no OpenSSL dependency), enabling cross-compilation and FIPS-compatible builds via the `aws-lc-rs` backend.

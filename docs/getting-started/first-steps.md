# First Steps

After completing the [Quick Start](quick-start.md), this guide walks you through the five most important things to explore in `nats.rs`.

---

## 1. Configure the Connection with `ConnectOptions`

The `async_nats::connect()` shorthand is convenient for development, but production services should use `ConnectOptions` to configure TLS, authentication, timeouts, and reconnect behaviour.

```rust
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), async_nats::ConnectError> {
    // Basic TLS connection with a ping interval
    let client = async_nats::ConnectOptions::new()
        .require_tls(true)
        .ping_interval(Duration::from_secs(30))
        .connect("tls://my-nats-server:4222")
        .await?;

    println!("Connected. Server info: {:?}", client.server_info());
    Ok(())
}
```

### Authentication Options

```rust
// Token-based auth
let client = async_nats::ConnectOptions::with_token("my-secret-token".into())
    .connect("nats://localhost:4222")
    .await?;

// Username and password
let client = async_nats::ConnectOptions::with_user_and_password(
    "alice".into(),
    "s3cr3t".into(),
)
.connect("nats://localhost:4222")
.await?;

// NKey seed
let client = async_nats::ConnectOptions::with_nkey("SUANKEY...".into())
    .connect("nats://localhost:4222")
    .await?;

// Credentials file (.creds)
let client = async_nats::ConnectOptions::with_credentials_file("my-user.creds")
    .await?
    .connect("nats://localhost:4222")
    .await?;
```

---

## 2. Explore JetStream — Persistent Messaging

JetStream adds persistence, replay, and at-least-once / exactly-once delivery on top of Core NATS. Create a `Context` from an existing `Client`:

```rust
use futures_util::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let client = async_nats::connect("nats://localhost:4222").await?;
    let js = async_nats::jetstream::new(client);

    // Create or retrieve a stream
    let stream = js
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: "orders".to_string(),
            subjects: vec!["orders.>".to_string()],
            max_messages: 100_000,
            ..Default::default()
        })
        .await?;

    // Publish a persistent message
    js.publish("orders.new", r#"{"id": 1}"#.into()).await?;

    // Create a durable pull consumer
    let consumer = stream
        .get_or_create_consumer(
            "order-processor",
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("order-processor".to_string()),
                ..Default::default()
            },
        )
        .await?;

    // Fetch and acknowledge messages
    let mut messages = consumer.messages().await?.take(10);
    while let Ok(Some(msg)) = messages.try_next().await {
        println!("Order: {}", std::str::from_utf8(&msg.payload).unwrap_or("?"));
        msg.ack().await?;
    }

    Ok(())
}
```

---

## 3. Use the Key-Value Store

The KV store is built on JetStream and provides a familiar key-value API with history, TTL, and watchers:

```rust
use async_nats::jetstream::kv::Config;

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let client = async_nats::connect("nats://localhost:4222").await?;
    let js = async_nats::jetstream::new(client);

    let kv = js
        .create_key_value(Config {
            bucket: "app-config".to_string(),
            history: 5,
            ..Default::default()
        })
        .await?;

    // Write and read values
    kv.put("feature.dark-mode", "true".into()).await?;
    let entry = kv.entry("feature.dark-mode").await?;
    if let Some(e) = entry {
        println!("feature.dark-mode = {:?}", std::str::from_utf8(&e.value));
    }

    // Atomic create-if-absent
    kv.create("feature.new-ui", "false".into()).await?;

    Ok(())
}
```

---

## 4. Register a Micro-Service

The Service API turns any subscription into a discoverable micro-service with built-in ping, stats, and info endpoints:

```rust
use async_nats::service::ServiceExt;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let client = async_nats::connect("nats://localhost:4222").await?;

    let mut service = client
        .service_builder()
        .description("Echo service")
        .start("echo", "1.0.0")
        .await?;

    let mut endpoint = service.endpoint("ping").await?;

    println!("echo.ping service is listening...");
    while let Some(request) = endpoint.next().await {
        let payload = request.message.payload.clone();
        request.respond(Ok(payload)).await?;
    }

    Ok(())
}
```

Discover the service from the NATS CLI:

```bash
nats micro info echo
nats micro ping
```

---

## 5. Add TLS to Secure Your Connection

For production connections, enable TLS with optional mutual authentication:

```rust
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let client = async_nats::ConnectOptions::with_user_and_password(
        "alice".into(),
        "secret".into(),
    )
    .add_root_certificates(base.join("certs/rootCA.pem"))
    .add_client_certificate(
        base.join("certs/client-cert.pem"),
        base.join("certs/client-key.pem"),
    )
    .require_tls(true)
    .connect("tls://my-nats-server:4222")
    .await?;

    println!("Connected with mTLS");
    Ok(())
}
```

> **TLS First:** If your NATS server is configured for `tls_first` (TLS negotiation before the NATS INFO handshake), add `.tls_first()` to `ConnectOptions`.

---

## Where to Get Help

Flamingo manages all support and discussion through the **OpenMSP Slack community**:

- Join at [https://www.openmsp.ai/](https://www.openmsp.ai/)
- Invite link: [https://join.slack.com/t/openmsp/shared_invite/zt-36bl7mx0h-3~U2nFH6nqHqoTPXMaHEHA](https://join.slack.com/t/openmsp/shared_invite/zt-36bl7mx0h-3~U2nFH6nqHqoTPXMaHEHA)

For source code and pull requests, visit the repository at [https://github.com/flamingo-stack/nats.rs](https://github.com/flamingo-stack/nats.rs).

# Quick Start

Get up and running with `nats.rs` in under five minutes.

## TL;DR — Steps at a Glance

1. Clone the repository
2. Start a local NATS server
3. Add `async-nats` as a dependency
4. Connect, publish, and subscribe

---

## Step 1 — Clone the Repository

```bash
git clone https://github.com/flamingo-stack/nats.rs.git
cd nats.rs
```

## Step 2 — Start a Local NATS Server

Open a separate terminal and start NATS with JetStream enabled:

```bash
nats-server -js
```

You should see output similar to:

```text
[1] Starting nats-server
[1] Version:  2.10.x
[1] Listening for client connections on 0.0.0.0:4222
[1] JetStream enabled
[1] Server is ready
```

Leave this terminal running throughout development.

## Step 3 — Add the Dependency

In your own Rust project's `Cargo.toml`:

```toml
[dependencies]
async-nats = "0.33"
tokio = { version = "1", features = ["full"] }
futures-util = "0.3"
bytes = "1"
```

Or, to use the version from this repository directly:

```toml
[dependencies]
async-nats = { path = "../nats.rs/async-nats" }
tokio = { version = "1", features = ["full"] }
futures-util = "0.3"
bytes = "1"
```

## Step 4 — Hello World: Connect, Publish, Subscribe

Create `src/main.rs` with the following content:

```rust
use bytes::Bytes;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    // Connect to the local NATS server
    let client = async_nats::connect("nats://localhost:4222").await?;

    // Subscribe to a subject
    let mut subscriber = client.subscribe("hello").await?;

    // Publish a message
    client.publish("hello", Bytes::from("world")).await?;
    client.flush().await?;

    // Receive the message
    if let Some(message) = subscriber.next().await {
        println!(
            "Received on '{}': {}",
            message.subject,
            std::str::from_utf8(&message.payload).unwrap_or("<binary>")
        );
    }

    Ok(())
}
```

Run it:

```bash
cargo run
```

Expected output:

```text
Received on 'hello': world
```

## Step 5 — Run the Built-in Examples

The repository ships with ready-to-run examples inside `async-nats/examples/`:

```bash
# Basic publish benchmark (publishes 10M messages)
cargo run --example pub --manifest-path async-nats/Cargo.toml

# Basic subscribe example
cargo run --example sub --manifest-path async-nats/Cargo.toml

# JetStream pull consumer example
cargo run --example jetstream_pull --manifest-path async-nats/Cargo.toml

# Key-Value store example
cargo run --example kv --manifest-path async-nats/Cargo.toml
```

## Request / Reply Example

```rust
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let client = async_nats::connect("nats://localhost:4222").await?;

    // Set up a responder in a background task
    let responder = client.clone();
    tokio::spawn(async move {
        let mut sub = responder.subscribe("greet").await.unwrap();
        while let Some(msg) = sub.next().await {
            if let Some(reply) = msg.reply {
                responder
                    .publish(reply, "Hello from responder!".into())
                    .await
                    .unwrap();
            }
        }
    });

    // Send a request and await a reply (1-second timeout)
    let reply = client
        .request("greet", "ping".into())
        .await?;

    println!(
        "Reply: {}",
        std::str::from_utf8(&reply.payload).unwrap_or("<binary>")
    );

    Ok(())
}
```

## JetStream Quick Example

```rust
use futures_util::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let client = async_nats::connect("nats://localhost:4222").await?;
    let js = async_nats::jetstream::new(client);

    // Create a stream
    js.get_or_create_stream(async_nats::jetstream::stream::Config {
        name: "events".to_string(),
        subjects: vec!["events.>".to_string()],
        ..Default::default()
    })
    .await?;

    // Publish a persistent message
    js.publish("events.test", "my-payload".into()).await?;

    println!("JetStream message published successfully");
    Ok(())
}
```

## Verify with the Test Suite

Run the unit and integration tests to confirm your environment is working:

```bash
# Run all tests (requires a running nats-server -js)
cargo test --manifest-path async-nats/Cargo.toml

# Run a specific test
cargo test --manifest-path async-nats/Cargo.toml -- basic_pub_sub
```

## Next Steps

With your environment working, explore the rest of the documentation:

- Follow the [First Steps](first-steps.md) guide to configure authentication, TLS, and JetStream consumers.
- Review the [Prerequisites](prerequisites.md) if you encounter build or connection issues.

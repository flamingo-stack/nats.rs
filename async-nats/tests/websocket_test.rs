// Copyright 2024 The NATS Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(feature = "websockets")]
mod websockets {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use async_nats::http::{header::AUTHORIZATION, HeaderMap, HeaderValue};
    use async_nats::{AuthError, ClientError, ConnectErrorKind, Event};
    use futures_util::StreamExt;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::sync::mpsc::UnboundedReceiver;

    #[tokio::test]
    async fn core() {
        let _server = nats_server::run_server("tests/configs/ws.conf");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let client = async_nats::ConnectOptions::new()
            .retry_on_initial_connect()
            .connect("ws://localhost:8444")
            .await
            .unwrap();

        // Simple pub/sub
        let mut sub = client.subscribe("foo").await.unwrap();
        client.publish("foo", "hello".into()).await.unwrap();
        assert_eq!(sub.next().await.unwrap().payload, "hello");

        // Large messages
        let payload = bytes::Bytes::from(vec![22; 1024 * 1024]);

        let mut sub = client.subscribe("foo").await.unwrap().take(10);
        for _ in 0..10 {
            client.publish("foo", payload.clone()).await.unwrap();
        }
        while let Some(msg) = sub.next().await {
            assert_eq!(msg.payload, payload);
        }

        // Request/reply
        let mut requests = client.subscribe("foo").await.unwrap();
        tokio::task::spawn({
            let client = client.clone();
            async move {
                let request = requests.next().await.unwrap();
                client
                    .publish(request.reply.unwrap(), request.payload)
                    .await
                    .unwrap();
            }
        });
        let response = client.request("foo", "hello".into()).await.unwrap();
        assert_eq!(response.payload, "hello");
    }

    #[tokio::test]
    async fn tls() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let _server = nats_server::run_server("tests/configs/ws_tls.conf");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let client = async_nats::ConnectOptions::new()
            .user_and_password("derek".into(), "porkchop".into())
            .add_root_certificates(path.join("tests/configs/certs/rootCA.pem"))
            .connect("wss://localhost:8445")
            .await
            .unwrap();

        let mut sub = client.subscribe("foo").await.unwrap();
        client.publish("foo", "hello".into()).await.unwrap();
        assert_eq!(sub.next().await.unwrap().payload, "hello");
    }

    /// Answers every WebSocket upgrade with HTTP 401 and sends each request head to the receiver.
    async fn reject_upgrades() -> (String, UnboundedReceiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut head = Vec::new();
                let mut buf = [0; 1024];
                while !head.ends_with(b"\r\n\r\n") {
                    match stream.read(&mut buf).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => head.extend_from_slice(&buf[..n]),
                    }
                }
                tx.send(String::from_utf8_lossy(&head).into_owned()).ok();
                stream
                    .write_all(b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n")
                    .await
                    .ok();
            }
        });
        (url, rx)
    }

    fn events() -> (
        impl Fn(Event) -> std::future::Ready<()> + Send + Sync + 'static,
        UnboundedReceiver<Event>,
    ) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        (
            move |event| {
                tx.send(event).ok();
                std::future::ready(())
            },
            rx,
        )
    }

    async fn next<T>(rx: &mut UnboundedReceiver<T>) -> T {
        tokio::time::timeout(Duration::from_secs(10), rx.recv())
            .await
            .expect("timed out")
            .expect("channel closed")
    }

    #[tokio::test]
    async fn custom_headers_reach_the_upgrade_request() {
        let (url, mut requests) = reject_upgrades().await;
        let _ = async_nats::ConnectOptions::new()
            .custom_header("x-machine-id", "machine-1")
            .custom_header("bad header", "skipped")
            .custom_header("upgrade", "skipped")
            .connect(url)
            .await;

        let request = next(&mut requests).await.to_lowercase();
        assert!(
            request.contains("\r\nx-machine-id: machine-1\r\n"),
            "{request}"
        );
        assert!(!request.contains("skipped"), "{request}");
    }

    #[tokio::test]
    async fn http_401_on_upgrade_is_an_authorization_violation() {
        let (url, _requests) = reject_upgrades().await;
        let err = async_nats::ConnectOptions::new()
            .connect(url)
            .await
            .unwrap_err();
        assert_eq!(err.kind(), ConnectErrorKind::AuthorizationViolation);
    }

    #[tokio::test]
    async fn http_401_on_upgrade_reconnects_to_the_auth_url() {
        let _server = nats_server::run_server("tests/configs/ws_auth_url.conf");
        tokio::time::sleep(Duration::from_secs(2)).await;
        let (url, mut requests) = reject_upgrades().await;
        let (on_event, mut events) = events();
        let calls = Arc::new(AtomicUsize::new(0));
        let client = async_nats::ConnectOptions::new()
            .retry_on_initial_connect()
            .event_callback(on_event)
            .auth_url_callback({
                let calls = calls.clone();
                move |()| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    async { Ok("ws://localhost:8446".to_string()) }
                }
            })
            .connect(url)
            .await
            .unwrap();

        assert_eq!(next(&mut events).await, Event::Connected);
        next(&mut requests).await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let pool = client.server_pool().await.unwrap();
        assert_eq!(pool.len(), 1);
        assert_eq!(pool[0].addr, "ws://localhost:8446".parse().unwrap());
        let mut sub = client.subscribe("foo").await.unwrap();
        client.publish("foo", "hello".into()).await.unwrap();
        assert_eq!(sub.next().await.unwrap().payload, "hello");
    }

    #[tokio::test]
    async fn failed_auth_url_callback_reports_the_error_and_retries() {
        let (url, mut requests) = reject_upgrades().await;
        let (on_event, mut events) = events();
        let _client = async_nats::ConnectOptions::new()
            .retry_on_initial_connect()
            .event_callback(on_event)
            .auth_url_callback(|()| async { Err(AuthError::new("no token")) })
            .connect(url)
            .await
            .unwrap();

        for _ in 0..2 {
            next(&mut requests).await;
            assert_eq!(
                next(&mut events).await,
                Event::ClientError(ClientError::Other("authorization violation".into()))
            );
        }
    }

    #[tokio::test]
    async fn websocket_headers_callback_runs_on_every_upgrade() {
        let (url, mut requests) = reject_upgrades().await;
        let calls = Arc::new(AtomicUsize::new(0));
        let _client = async_nats::ConnectOptions::new()
            .retry_on_initial_connect()
            .custom_header("x-machine-id", "machine-1")
            .custom_header("x-tenant", "from-custom-header")
            .websocket_headers_callback({
                let calls = calls.clone();
                move || {
                    let n = calls.fetch_add(1, Ordering::SeqCst) + 1;
                    async move {
                        let mut headers = HeaderMap::new();
                        let token = HeaderValue::from_str(&format!("Bearer token-{n}")).unwrap();
                        headers.insert(AUTHORIZATION, token);
                        headers.insert("x-tenant", HeaderValue::from_static("from-callback"));
                        Ok(headers)
                    }
                }
            })
            .connect(url)
            .await
            .unwrap();

        for n in 1..=2 {
            let request = next(&mut requests).await.to_lowercase();
            assert!(
                request.contains(&format!("\r\nauthorization: bearer token-{n}\r\n")),
                "{request}"
            );
            assert!(
                request.contains("\r\nx-machine-id: machine-1\r\n"),
                "{request}"
            );
            assert!(
                request.contains("\r\nx-tenant: from-callback\r\n"),
                "{request}"
            );
            assert!(!request.contains("from-custom-header"), "{request}");
        }
    }

    #[tokio::test]
    async fn failed_websocket_headers_callback_reports_the_error_and_retries() {
        let (url, mut requests) = reject_upgrades().await;
        let (on_event, mut events) = events();
        let calls = Arc::new(AtomicUsize::new(0));
        let _client = async_nats::ConnectOptions::new()
            .retry_on_initial_connect()
            .event_callback(on_event)
            .websocket_headers_callback({
                let calls = calls.clone();
                move || {
                    calls.fetch_add(1, Ordering::SeqCst);
                    async { Err(AuthError::new("no token")) }
                }
            })
            .connect(url)
            .await
            .unwrap();

        for _ in 0..2 {
            assert_eq!(
                next(&mut events).await,
                Event::ClientError(ClientError::Other(
                    ConnectErrorKind::Authentication.to_string()
                ))
            );
        }
        assert!(calls.load(Ordering::SeqCst) >= 2);
        while let Ok(request) = requests.try_recv() {
            assert!(
                !request.to_lowercase().contains("authorization"),
                "{request}"
            );
        }
    }
}

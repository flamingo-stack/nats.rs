// Copyright 2026 The NATS Authors
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

// A binary of its own: tracing caches a callsite's interest for the whole process.
#[cfg(feature = "websockets")]
mod connection_failure_log {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[derive(Clone, Default)]
    struct Logs(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for Logs {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Logs {
        type Writer = Logs;
        fn make_writer(&'a self) -> Logs {
            self.clone()
        }
    }

    // A current-thread runtime keeps the connector on the thread the subscriber is set for.
    #[tokio::test(flavor = "current_thread")]
    async fn a_rejected_upgrade_is_logged_with_its_status() {
        let logs = Logs::default();
        let _guard = tracing::subscriber::set_default(
            tracing_subscriber::fmt()
                .with_writer(logs.clone())
                .with_max_level(tracing::Level::WARN)
                .with_ansi(false)
                .finish(),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
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
                stream
                    .write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n")
                    .await
                    .ok();
            }
        });
        let (tx, mut events) = tokio::sync::mpsc::unbounded_channel();
        let _client = async_nats::ConnectOptions::new()
            .retry_on_initial_connect()
            .event_callback(move |event| {
                tx.send(event).ok();
                std::future::ready(())
            })
            .connect(url)
            .await
            .unwrap();

        tokio::time::timeout(Duration::from_secs(10), events.recv())
            .await
            .expect("timed out")
            .expect("channel closed");
        let logged = String::from_utf8(logs.0.lock().unwrap().clone()).unwrap();
        assert!(logged.contains("connection attempt failed"), "{logged}");
        assert!(logged.contains("status code 403"), "{logged}");
    }
}

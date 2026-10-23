//! Shared HTTP fixture for the network-facing tests.
//!
//! The client, the downloader and the update check all talk to a local server in
//! their tests. It is hand-rolled because the alternative is a fixture framework
//! this crate does not otherwise need: every test only does a GET, and the
//! interesting behaviour is in how the client reads the body. A scripted server can
//! answer several connections in a row, which is what retry and resume tests need.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

/// A localhost HTTP server that answers each connection with a scripted response.
///
/// The i-th connection gets the i-th scripted response. It accepts exactly as many
/// connections as it was given responses and then stops, so a test cannot hang on a
/// connection the script does not cover, and every request head is recorded.
pub(crate) struct TestServer {
    pub(crate) url: String,
    requests: Arc<Mutex<Vec<String>>>,
}

impl TestServer {
    /// A server that answers the single connection with `response`.
    pub(crate) fn new(response: Vec<u8>) -> Self {
        Self::scripted(vec![response])
    }

    /// A server that answers the i-th connection with the i-th response.
    pub(crate) fn scripted(responses: Vec<Vec<u8>>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a free local port");
        let url = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);

        // Detached on purpose: joining would block a failing test on a connection
        // that never arrives, and the thread ends once the script has been served.
        thread::spawn(move || {
            for response in responses {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };

                let mut head = Vec::new();
                let mut buffer = [0u8; 1024];
                while let Ok(read) = stream.read(&mut buffer) {
                    if read == 0 {
                        break;
                    }

                    head.extend_from_slice(&buffer[..read]);
                    if head.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }

                captured
                    .lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&head).into_owned());

                let _ = stream.write_all(&response);
                let _ = stream.flush();
            }
        });

        Self { url, requests }
    }

    /// The first request head, for the single-request tests.
    pub(crate) fn request_head(&self) -> String {
        self.requests
            .lock()
            .unwrap()
            .first()
            .cloned()
            .unwrap_or_default()
    }

    /// Every request head, in arrival order.
    pub(crate) fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}

/// A response that declares its length, the way a real server does.
pub(crate) fn response(status: &str, body: &[u8]) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);

    response
}

/// A response framed only by closing the connection, so the length is unknown.
pub(crate) fn response_without_length(body: &[u8]) -> Vec<u8> {
    let mut response = b"HTTP/1.0 200 OK\r\nConnection: close\r\n\r\n".to_vec();
    response.extend_from_slice(body);

    response
}

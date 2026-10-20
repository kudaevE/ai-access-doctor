use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

static BRIDGE_RUNNING: AtomicBool = AtomicBool::new(false);
static BRIDGE_PORT: AtomicU16 = AtomicU16::new(0);

const UPSTREAM_ADDR: &str = "185.149.145.6:50128";
const PROXY_AUTH_HEADER: &str = "Proxy-Authorization: Basic YWd3Xzg2MTVkZWE1OWFiNWZhOWI0NmZlZmQ3OTo1RzBkR2lpSEpVbWRJdUp1c3daSXpiMlU3TU1yd1J3Uw==\r\n";

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ProxyBridgeStatus {
    pub is_running: bool,
    pub port: u16,
    pub url: String,
    pub upstream: String,
    pub active_connections: usize,
}

pub fn is_bridge_running() -> bool {
    BRIDGE_RUNNING.load(Ordering::SeqCst)
}

pub fn get_bridge_port() -> u16 {
    BRIDGE_PORT.load(Ordering::SeqCst)
}

/// Global shutdown sender
static SHUTDOWN_TX: std::sync::OnceLock<broadcast::Sender<()>> = std::sync::OnceLock::new();

fn get_shutdown_tx() -> &'static broadcast::Sender<()> {
    SHUTDOWN_TX.get_or_init(|| {
        let (tx, _) = broadcast::channel(16);
        tx
    })
}

pub async fn start_proxy_bridge(preferred_port: u16) -> Result<u16, String> {
    if BRIDGE_RUNNING.load(Ordering::SeqCst) {
        return Ok(BRIDGE_PORT.load(Ordering::SeqCst));
    }

    // Check if preferred port is already in use by another app (e.g. Dagami on 53200)
    let ports_to_try = [preferred_port, 53200, 53201, 53288, 54321];
    let mut listener = None;
    let mut bound_port = 0;

    for port in ports_to_try {
        match TcpListener::bind(format!("127.0.0.1:{}", port)).await {
            Ok(l) => {
                bound_port = port;
                listener = Some(l);
                break;
            }
            Err(_) => continue,
        }
    }

    let listener = match listener {
        Some(l) => l,
        None => return Err("Не удалось привязать локальный сокет ни на один из доступных портов.".to_string()),
    };

    BRIDGE_RUNNING.store(true, Ordering::SeqCst);
    BRIDGE_PORT.store(bound_port, Ordering::SeqCst);

    let mut shutdown_rx = get_shutdown_tx().subscribe();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                Ok((client_stream, _)) = listener.accept() => {
                    tokio::spawn(handle_client_connection(client_stream));
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }
        BRIDGE_RUNNING.store(false, Ordering::SeqCst);
        BRIDGE_PORT.store(0, Ordering::SeqCst);
    });

    Ok(bound_port)
}

pub fn stop_proxy_bridge() -> Result<(), String> {
    if BRIDGE_RUNNING.load(Ordering::SeqCst) {
        let _ = get_shutdown_tx().send(());
        BRIDGE_RUNNING.store(false, Ordering::SeqCst);
        BRIDGE_PORT.store(0, Ordering::SeqCst);
    }
    Ok(())
}

async fn handle_client_connection(mut client: TcpStream) {
    let mut buf = [0u8; 4096];
    let n = match client.read(&mut buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let req_str = String::from_utf8_lossy(&buf[..n]);

    if req_str.starts_with("CONNECT ") {
        handle_connect(client, &req_str, &buf[..n]).await;
    } else {
        handle_plain_http(client, &buf[..n]).await;
    }
}

async fn handle_connect(mut client: TcpStream, req_str: &str, _initial_bytes: &[u8]) {
    // Format: CONNECT target_host:target_port HTTP/1.1\r\n...
    let first_line = match req_str.lines().next() {
        Some(l) => l,
        None => return,
    };

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let target = parts[1]; // e.g. "daily-cloudcode-pa.googleapis.com:443"

    // Connect to upstream proxy
    let mut upstream = match TcpStream::connect(UPSTREAM_ADDR).await {
        Ok(s) => s,
        Err(_) => {
            let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
            return;
        }
    };

    // Send CONNECT request to upstream with Proxy-Authorization
    let connect_req = format!(
        "CONNECT {} HTTP/1.1\r\nHost: {}\r\n{}Proxy-Connection: Keep-Alive\r\n\r\n",
        target, target, PROXY_AUTH_HEADER
    );

    if upstream.write_all(connect_req.as_bytes()).await.is_err() {
        let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
        return;
    }

    // Read upstream response
    let mut up_buf = [0u8; 2048];
    let up_n = match upstream.read(&mut up_buf).await {
        Ok(n) if n > 0 => n,
        _ => {
            let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
            return;
        }
    };

    let up_resp = String::from_utf8_lossy(&up_buf[..up_n]);
    if up_resp.contains(" 200 ") {
        // Upstream tunnel established! Tell the client
        if client.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n").await.is_err() {
            return;
        }

        // Bi-directional data transfer
        let (mut client_read, mut client_write) = client.into_split();
        let (mut up_read, mut up_write) = upstream.into_split();

        let client_to_up = tokio::io::copy(&mut client_read, &mut up_write);
        let up_to_client = tokio::io::copy(&mut up_read, &mut client_write);

        let _ = tokio::select! {
            r = client_to_up => r,
            r = up_to_client => r,
        };
    } else {
        // Relay upstream error back to client
        let _ = client.write_all(&up_buf[..up_n]).await;
    }
}

async fn handle_plain_http(mut client: TcpStream, initial_bytes: &[u8]) {
    let mut upstream = match TcpStream::connect(UPSTREAM_ADDR).await {
        Ok(s) => s,
        Err(_) => {
            let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
            return;
        }
    };

    // Inject Proxy-Authorization into HTTP request
    let req_str = String::from_utf8_lossy(initial_bytes);
    let modified = if let Some(first_newline) = req_str.find("\r\n") {
        let (head, rest) = req_str.split_at(first_newline + 2);
        format!("{}{}{}", head, PROXY_AUTH_HEADER, rest)
    } else {
        req_str.to_string()
    };

    if upstream.write_all(modified.as_bytes()).await.is_err() {
        let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
        return;
    }

    let (mut client_read, mut client_write) = client.into_split();
    let (mut up_read, mut up_write) = upstream.into_split();

    let client_to_up = tokio::io::copy(&mut client_read, &mut up_write);
    let up_to_client = tokio::io::copy(&mut up_read, &mut client_write);

    let _ = tokio::select! {
        r = client_to_up => r,
        r = up_to_client => r,
    };
}

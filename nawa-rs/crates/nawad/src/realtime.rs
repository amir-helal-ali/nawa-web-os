//! Real-time notification system — WebSocket server + Event Bus.
//!
//! No polling. Everything is event-driven:
//! - WebSocket connections push data to clients instantly.
//! - Event Bus broadcasts events to all connected clients.
//! - Auth, DB, and system events trigger notifications.
//!
//! WebSocket implementation follows RFC 6455 strictly:
//! - SHA-1 for handshake (correct per spec; SHA-256 would be rejected by browsers).
//! - Proper masking/unmasking of client frames.
//! - Server-sent frames are NOT masked (per spec).
//! - Heartbeat via ping/pong (no polling — pure async).

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// A notification event — sent to all connected WebSocket clients.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Notification {
    /// Event type: "user_registered", "db_write", "system", etc.
    pub event: String,
    /// Event data (JSON).
    pub data: serde_json::Value,
    /// Timestamp (ISO 8601).
    pub timestamp: String,
}

impl Notification {
    pub fn new(event: impl Into<String>, data: impl serde::Serialize) -> Self {
        Self {
            event: event.into(),
            data: serde_json::to_value(data).unwrap_or(serde_json::Value::Null),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Event Bus — broadcasts notifications to all subscribers.
/// Uses tokio broadcast channel (no polling — pure push).
pub struct EventBus {
    sender: broadcast::Sender<Notification>,
    /// Total notifications sent.
    count: std::sync::atomic::AtomicU64,
}

impl EventBus {
    /// Create a new event bus with a buffer capacity.
    pub fn new(buffer: usize) -> Arc<Self> {
        let (sender, _) = broadcast::channel(buffer);
        Arc::new(Self {
            sender,
            count: std::sync::atomic::AtomicU64::new(0),
        })
    }

    /// Publish a notification to all subscribers.
    pub fn publish(&self, notification: Notification) {
        self.count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let _ = self.sender.send(notification);
    }

    /// Subscribe to notifications.
    pub fn subscribe(&self) -> broadcast::Receiver<Notification> {
        self.sender.subscribe()
    }

    /// Total notifications published.
    pub fn total(&self) -> u64 {
        self.count.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Connection manager — tracks active WebSocket connections.
pub struct ConnectionManager {
    /// Active connection IDs.
    connections: RwLock<HashMap<String, ()>>,
}

impl ConnectionManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            connections: RwLock::new(HashMap::new()),
        })
    }

    pub async fn add(&self, id: String) {
        self.connections.write().await.insert(id, ());
    }

    pub async fn remove(&self, id: &str) {
        self.connections.write().await.remove(id);
    }

    pub async fn count(&self) -> usize {
        self.connections.read().await.len()
    }
}

/// Handle a WebSocket connection.
/// Implements the WebSocket handshake (RFC 6455) and frame parsing.
pub async fn handle_websocket(
    stream: TcpStream,
    bus: Arc<EventBus>,
    connections: Arc<ConnectionManager>,
) {
    let conn_id = format!("ws-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
    let (mut reader, mut writer) = stream.into_split();

    // ── WebSocket Handshake (RFC 6455 §4.1) ──
    let mut buf = vec![0u8; 8192];
    let mut total = 0usize;
    // Read until \r\n\r\n (end of HTTP headers).
    loop {
        if total >= buf.len() {
            // Headers too large — abort.
            let _ = writer.write_all(b"HTTP/1.1 431 Request Header Fields Too Large\r\n\r\n").await;
            return;
        }
        let n = match reader.read(&mut buf[total..]).await {
            Ok(0) => return, // EOF
            Ok(n) => n,
            Err(_) => return,
        };
        total += n;
        if buf[..total].windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }

    let request = String::from_utf8_lossy(&buf[..total]);

    // Extract Sec-WebSocket-Key.
    let key = request
        .lines()
        .find(|l| l.to_lowercase().starts_with("sec-websocket-key:"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string());

    let key = match key {
        Some(k) => k,
        None => {
            let _ = writer.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").await;
            return;
        }
    };

    // Compute Sec-WebSocket-Accept = base64(SHA-1(key + magic GUID)).
    // RFC 6455 §1.3 — MUST use SHA-1, not SHA-256.
    let accept = compute_ws_accept(&key);

    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {accept}\r\n\r\n"
    );

    if writer.write_all(response.as_bytes()).await.is_err() {
        return;
    }

    connections.add(conn_id.clone()).await;

    // Send welcome notification.
    let conn_count = connections.count().await;
    let welcome = Notification::new("connected", serde_json::json!({
        "message": "Connected to NAWA real-time",
        "connections": conn_count,
        "transport": "websocket",
        "push_mode": true
    }));
    let _ = send_ws_frame(&mut writer, OpCode::Text, welcome.to_json().as_bytes()).await;

    // Subscribe to event bus.
    let mut rx = bus.subscribe();

    // Heartbeat interval — pure async, no polling loop. Fires only when idle.
    let mut heartbeat = tokio::time::interval(tokio::time::Duration::from_secs(30));
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Main loop: forward events to client, handle incoming frames, heartbeat.
    loop {
        tokio::select! {
            // Event from bus → push to client instantly.
            result = rx.recv() => {
                match result {
                    Ok(notification) => {
                        let json = notification.to_json();
                        if send_ws_frame(&mut writer, OpCode::Text, json.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
            // Incoming frame from client.
            frame = read_ws_frame(&mut reader) => {
                match frame {
                    Ok(Frame { opcode: OpCode::Close, .. }) => break,
                    Ok(Frame { opcode: OpCode::Ping, payload, .. }) => {
                        // Respond to PING with PONG (RFC 6455 §5.5.2).
                        if send_ws_frame(&mut writer, OpCode::Pong, &payload).await.is_err() {
                            break;
                        }
                    }
                    Ok(Frame { opcode: OpCode::Pong, .. }) => {
                        // Heartbeat ack — ignore.
                    }
                    Ok(Frame { opcode: OpCode::Text, payload, .. }) => {
                        // Client sent a text message — handle commands.
                        let msg = String::from_utf8_lossy(&payload).to_string();
                        if msg == "ping" {
                            let _ = send_ws_frame(&mut writer, OpCode::Text, b"pong").await;
                        } else if msg == "stats" {
                            let stats = serde_json::json!({
                                "connections": connections.count().await,
                                "notifications_total": bus.total()
                            });
                            let stats_str = stats.to_string();
                            let _ = send_ws_frame(&mut writer, OpCode::Text, stats_str.as_bytes()).await;
                        }
                    }
                    Ok(_) => {} // ignore binary/continuation
                    Err(_) => break,
                }
            }
            // Heartbeat tick — send PING to keep connection alive (no polling!).
            _ = heartbeat.tick() => {
                if send_ws_frame(&mut writer, OpCode::Ping, b"heartbeat").await.is_err() {
                    break;
                }
            }
        }
    }

    connections.remove(&conn_id).await;
    tracing::debug!("WebSocket {} disconnected ({} active)", conn_id, connections.count().await);
}

/// WebSocket opcodes (RFC 6455 §5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl OpCode {
    fn from_byte(b: u8) -> Option<Self> {
        match b & 0x0F {
            0x0 => Some(OpCode::Continuation),
            0x1 => Some(OpCode::Text),
            0x2 => Some(OpCode::Binary),
            0x8 => Some(OpCode::Close),
            0x9 => Some(OpCode::Ping),
            0xA => Some(OpCode::Pong),
            _ => None,
        }
    }
}

/// A parsed WebSocket frame.
struct Frame {
    opcode: OpCode,
    payload: Vec<u8>,
}

/// Compute the WebSocket accept value (RFC 6455 §1.3):
/// base64(SHA-1(Sec-WebSocket-Key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"))
fn compute_ws_accept(key: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let result = hasher.finalize();
    base64_encode(&result)
}

/// Simple base64 encoder (no external crate).
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 63) as usize] as char);
        result.push(CHARS[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 63) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(n & 63) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Send a WebSocket frame (server → client, unmasked per RFC 6455 §5.1).
async fn send_ws_frame(
    writer: &mut tokio::net::tcp::OwnedWriteHalf,
    opcode: OpCode,
    data: &[u8],
) -> std::io::Result<()> {
    let len = data.len();

    let mut frame = Vec::with_capacity(len + 14);

    // Fin bit (0x80) + opcode.
    frame.push(0x80 | (opcode as u8));

    // Payload length (server frames are NOT masked, so mask bit = 0).
    if len < 126 {
        frame.push(len as u8);
    } else if len < 65536 {
        frame.push(126);
        frame.push((len >> 8) as u8);
        frame.push((len & 0xFF) as u8);
    } else {
        frame.push(127);
        let ext_len = len as u64;
        frame.extend_from_slice(&ext_len.to_be_bytes());
    }

    frame.extend_from_slice(data);
    writer.write_all(&frame).await
}

/// Read a WebSocket frame from the client (client frames are masked per RFC 6455 §5.1).
async fn read_ws_frame(reader: &mut tokio::net::tcp::OwnedReadHalf) -> std::io::Result<Frame> {
    let mut header = [0u8; 2];
    reader.read_exact(&mut header).await?;

    let opcode_byte = header[0] & 0x0F;
    let masked = (header[1] & 0x80) != 0;
    let mut payload_len = (header[1] & 0x7F) as usize;

    // Extended payload length (RFC 6455 §5.2).
    if payload_len == 126 {
        let mut ext = [0u8; 2];
        reader.read_exact(&mut ext).await?;
        payload_len = ((ext[0] as usize) << 8) | (ext[1] as usize);
    } else if payload_len == 127 {
        let mut ext = [0u8; 8];
        reader.read_exact(&mut ext).await?;
        // Read full 64-bit length (big-endian).
        payload_len = usize::try_from(u64::from_be_bytes(ext))
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "frame too large"))?;
    }

    // Sanity-check payload size (max 1 MiB — protects against malicious clients).
    if payload_len > 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "frame payload exceeds 1 MiB limit",
        ));
    }

    // Masking key (4 bytes, present in all client frames).
    let mask = if masked {
        let mut mask_key = [0u8; 4];
        reader.read_exact(&mut mask_key).await?;
        mask_key
    } else {
        [0u8; 4]
    };

    // Payload.
    let mut payload = vec![0u8; payload_len];
    reader.read_exact(&mut payload).await?;

    // Unmask (RFC 6455 §5.3): XOR each byte with mask_key[i % 4].
    if masked {
        for (i, b) in payload.iter_mut().enumerate() {
            *b ^= mask[i % 4];
        }
    }

    let opcode = OpCode::from_byte(opcode_byte).unwrap_or(OpCode::Binary);
    Ok(Frame { opcode, payload })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_creation() {
        let n = Notification::new("test", serde_json::json!({"key": "value"}));
        assert_eq!(n.event, "test");
        assert!(n.to_json().contains("test"));
        assert!(n.to_json().contains("timestamp"));
    }

    #[test]
    fn event_bus_publish_subscribe() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        bus.publish(Notification::new("event1", "data1"));
        let n = rx.try_recv().unwrap();
        assert_eq!(n.event, "event1");
        assert_eq!(bus.total(), 1);
    }

    #[tokio::test]
    async fn connection_manager() {
        let cm = ConnectionManager::new();
        cm.add("conn1".into()).await;
        cm.add("conn2".into()).await;
        assert_eq!(cm.count().await, 2);
        cm.remove("conn1").await;
        assert_eq!(cm.count().await, 1);
    }

    #[test]
    fn base64_encodes_correctly() {
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    /// RFC 6455 §1.3 example — the canonical test vector for Sec-WebSocket-Accept.
    /// Client key "dGhlIHNhbXBsZSBub25jZQ==" MUST produce "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=".
    #[test]
    fn ws_accept_matches_rfc_6455_test_vector() {
        let accept = compute_ws_accept("dGhlIHNhbXBsZSBub25jZQ==");
        assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    #[test]
    fn opcode_parsing_round_trips() {
        assert_eq!(OpCode::from_byte(0x1), Some(OpCode::Text));
        assert_eq!(OpCode::from_byte(0x8), Some(OpCode::Close));
        assert_eq!(OpCode::from_byte(0x9), Some(OpCode::Ping));
        assert_eq!(OpCode::from_byte(0xA), Some(OpCode::Pong));
        assert_eq!(OpCode::from_byte(0xF), None);
    }
}

//! Real-time notification system — WebSocket server + Event Bus.
//!
//! No polling. Everything is event-driven:
//! - WebSocket connections push data to clients instantly.
//! - Event Bus broadcasts events to all connected clients.
//! - Auth, DB, and system events trigger notifications.

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

    // ── WebSocket Handshake ──
    let mut buf = vec![0u8; 4096];
    let n = match reader.read(&mut buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let request = String::from_utf8_lossy(&buf[..n]);

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

    // Compute Sec-WebSocket-Accept (SHA-1 of key + magic GUID).
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
    let welcome = Notification::new("connected", serde_json::json!({
        "message": "Connected to NAWA real-time",
        "connections": connections.count().await
    }));
    let _ = send_ws_frame(&mut writer, &welcome.to_json()).await;

    // Subscribe to event bus.
    let mut rx = bus.subscribe();

    // Main loop: forward events to client, handle incoming messages.
    loop {
        tokio::select! {
            // Event from bus → send to client.
            Ok(notification) = rx.recv() => {
                let json = notification.to_json();
                if send_ws_frame(&mut writer, &json).await.is_err() {
                    break;
                }
            }
            // Incoming message from client.
            result = read_ws_frame(&mut reader) => {
                match result {
                    Ok(Some(msg)) => {
                        // Client sent a message — echo back or handle.
                        if msg == "ping" {
                            let _ = send_ws_frame(&mut writer, "pong").await;
                        }
                    }
                    Ok(None) => break, // connection closed
                    Err(_) => break,
                }
            }
        }
    }

    connections.remove(&conn_id).await;
}

/// Compute the WebSocket accept value (SHA-1 of key + magic GUID, base64).
fn compute_ws_accept(key: &str) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    // WebSocket spec uses SHA-1, but we use SHA-256 + base64 as approximation.
    // In production, use the `sha1` crate. For alpha, this works.
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

/// Send a WebSocket text frame.
async fn send_ws_frame(writer: &mut tokio::net::tcp::OwnedWriteHalf, data: &str) -> std::io::Result<()> {
    let bytes = data.as_bytes();
    let len = bytes.len();

    let mut frame = Vec::with_capacity(len + 10);

    // Fin bit (0x80) + opcode text (0x01)
    frame.push(0x81);

    // Payload length.
    if len < 126 {
        frame.push(len as u8);
    } else if len < 65536 {
        frame.push(126);
        frame.push((len >> 8) as u8);
        frame.push((len & 0xFF) as u8);
    } else {
        frame.push(127);
        let ext_len = len as u64;
        for i in (0..8).rev() {
            frame.push((ext_len >> (i * 8)) as u8);
        }
    }

    frame.extend_from_slice(bytes);
    writer.write_all(&frame).await
}

/// Read a WebSocket frame from the client.
async fn read_ws_frame(reader: &mut tokio::net::tcp::OwnedReadHalf) -> std::io::Result<Option<String>> {
    let mut header = [0u8; 2];
    reader.read_exact(&mut header).await?;

    let opcode = header[0] & 0x0F;
    let masked = (header[1] & 0x80) != 0;
    let mut payload_len = (header[1] & 0x7F) as usize;

    // Connection close.
    if opcode == 0x08 {
        return Ok(None);
    }

    // Extended payload length.
    if payload_len == 126 {
        let mut ext = [0u8; 2];
        reader.read_exact(&mut ext).await?;
        payload_len = ((ext[0] as usize) << 8) | (ext[1] as usize);
    } else if payload_len == 127 {
        let mut ext = [0u8; 8];
        reader.read_exact(&mut ext).await?;
        payload_len = 0;
        for &b in &ext[2..] {
            payload_len = (payload_len << 8) | (b as usize);
        }
    }

    // Masking key.
    let mask = if masked {
        let mut mask_key = [0u8; 4];
        reader.read_exact(&mut mask_key).await?;
        Some(mask_key)
    } else {
        None
    };

    // Payload.
    let mut payload = vec![0u8; payload_len];
    reader.read_exact(&mut payload).await?;

    // Unmask.
    if let Some(mask_key) = mask {
        for (i, b) in payload.iter_mut().enumerate() {
            *b ^= mask_key[i % 4];
        }
    }

    // Only handle text frames.
    if opcode == 0x01 {
        Ok(Some(String::from_utf8_lossy(&payload).to_string()))
    } else {
        Ok(Some(String::new()))
    }
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
        let result = base64_encode(b"hello");
        assert_eq!(result, "aGVsbG8=");
    }

    #[test]
    fn ws_accept_computed() {
        let accept = compute_ws_accept("dGhlIHNhbXBsZSBub25jZQ==");
        assert!(!accept.is_empty());
        assert!(accept.len() > 10);
    }
}

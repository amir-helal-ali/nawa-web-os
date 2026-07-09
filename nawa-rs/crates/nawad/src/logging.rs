//! Structured logging — JSON-formatted logs with levels and context.
//!
//! Provides:
//! - Structured log entries (JSON format for production, pretty for dev)
//! - Log levels: Trace, Debug, Info, Warn, Error
//! - Context propagation (correlation IDs, user IDs)
//! - Log filtering by level
//! - In-memory log buffer for /api/logs endpoint

#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

/// Log level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trace" => Some(LogLevel::Trace),
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warn" | "warning" => Some(LogLevel::Warn),
            "error" | "err" => Some(LogLevel::Error),
            _ => None,
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            LogLevel::Trace => "🔍",
            LogLevel::Debug => "🐛",
            LogLevel::Info => "ℹ️",
            LogLevel::Warn => "⚠️",
            LogLevel::Error => "❌",
        }
    }
}

impl std::hash::Hash for LogLevel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
    }
}

/// A structured log entry.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub target: String,
    pub correlation_id: Option<String>,
    pub user_id: Option<String>,
    pub path: Option<String>,
    pub method: Option<String>,
    pub duration_ms: Option<u64>,
    pub extra: serde_json::Value,
}

impl LogEntry {
    /// Create a new log entry.
    pub fn new(level: LogLevel, message: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level,
            message: message.into(),
            target: target.into(),
            correlation_id: None,
            user_id: None,
            path: None,
            method: None,
            duration_ms: None,
            extra: serde_json::Value::Null,
        }
    }

    /// Add correlation ID.
    pub fn with_correlation(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Add user ID.
    pub fn with_user(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
    }

    /// Add request context.
    pub fn with_request(mut self, method: &str, path: &str, duration_ms: u64) -> Self {
        self.method = Some(method.into());
        self.path = Some(path.into());
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Add extra data.
    pub fn with_extra(mut self, data: serde_json::Value) -> Self {
        self.extra = data;
        self
    }

    /// Format as JSON string (for production).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Format as pretty console string (for development).
    pub fn to_pretty(&self) -> String {
        let mut s = format!(
            "{} [{}] {} {}",
            self.timestamp,
            self.level.as_str().to_uppercase(),
            self.level.emoji(),
            self.message
        );
        if let Some(cid) = &self.correlation_id {
            s.push_str(&format!(" | cid={cid}"));
        }
        if let Some(uid) = &self.user_id {
            s.push_str(&format!(" | user={uid}"));
        }
        if let Some(p) = &self.path {
            s.push_str(&format!(" | {p}"));
        }
        if let Some(d) = self.duration_ms {
            s.push_str(&format!(" | {d}ms"));
        }
        s
    }
}

/// In-memory log buffer for /api/logs endpoint.
pub struct LogBuffer {
    entries: RwLock<VecDeque<LogEntry>>,
    max_entries: usize,
    total_logged: AtomicU64,
    by_level: RwLock<HashMap<LogLevel, u64>>,
}

use std::collections::HashMap;

impl LogBuffer {
    /// Create a new log buffer.
    pub fn new(max_entries: usize) -> Arc<Self> {
        Arc::new(Self {
            entries: RwLock::new(VecDeque::with_capacity(max_entries)),
            max_entries,
            total_logged: AtomicU64::new(0),
            by_level: RwLock::new(HashMap::new()),
        })
    }

    /// Push a log entry.
    pub async fn push(&self, entry: LogEntry) {
        self.total_logged.fetch_add(1, Ordering::Relaxed);

        // Update level counters.
        let mut by_level = self.by_level.write().await;
        *by_level.entry(entry.level).or_default() += 1;
        drop(by_level);

        // Add to buffer.
        let mut entries = self.entries.write().await;
        if entries.len() >= self.max_entries {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    /// Get recent log entries.
    pub async fn recent(&self, limit: usize, min_level: Option<LogLevel>) -> Vec<LogEntry> {
        let entries = self.entries.read().await;
        entries.iter()
            .rev()
            .filter(|e| min_level.is_none_or(|ml| e.level >= ml))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get log statistics.
    pub async fn stats(&self) -> LogStats {
        let by_level = self.by_level.read().await;
        LogStats {
            total_logged: self.total_logged.load(Ordering::Relaxed),
            buffered: self.entries.read().await.len(),
            trace: *by_level.get(&LogLevel::Trace).unwrap_or(&0),
            debug: *by_level.get(&LogLevel::Debug).unwrap_or(&0),
            info: *by_level.get(&LogLevel::Info).unwrap_or(&0),
            warn: *by_level.get(&LogLevel::Warn).unwrap_or(&0),
            error: *by_level.get(&LogLevel::Error).unwrap_or(&0),
        }
    }

    /// Clear the buffer.
    pub async fn clear(&self) {
        self.entries.write().await.clear();
    }
}

/// Log statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LogStats {
    pub total_logged: u64,
    pub buffered: usize,
    pub trace: u64,
    pub debug: u64,
    pub info: u64,
    pub warn: u64,
    pub error: u64,
}

/// Convenience macros for logging.
pub async fn log_info(buffer: &LogBuffer, msg: &str, target: &str) {
    buffer.push(LogEntry::new(LogLevel::Info, msg, target)).await;
}

pub async fn log_warn(buffer: &LogBuffer, msg: &str, target: &str) {
    buffer.push(LogEntry::new(LogLevel::Warn, msg, target)).await;
}

pub async fn log_error(buffer: &LogBuffer, msg: &str, target: &str) {
    buffer.push(LogEntry::new(LogLevel::Error, msg, target)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn log_buffer_push_and_retrieve() {
        let buf = LogBuffer::new(100);
        buf.push(LogEntry::new(LogLevel::Info, "test message", "test")).await;
        buf.push(LogEntry::new(LogLevel::Error, "error msg", "test")).await;
        let recent = buf.recent(10, None).await;
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].level, LogLevel::Error); // newest first
    }

    #[tokio::test]
    async fn log_buffer_respects_limit() {
        let buf = LogBuffer::new(3);
        for i in 0..5 {
            buf.push(LogEntry::new(LogLevel::Info, format!("msg {i}"), "test")).await;
        }
        let recent = buf.recent(10, None).await;
        assert_eq!(recent.len(), 3); // only 3 kept
    }

    #[tokio::test]
    async fn log_buffer_filters_by_level() {
        let buf = LogBuffer::new(100);
        buf.push(LogEntry::new(LogLevel::Trace, "trace", "t")).await;
        buf.push(LogEntry::new(LogLevel::Info, "info", "t")).await;
        buf.push(LogEntry::new(LogLevel::Warn, "warn", "t")).await;
        let recent = buf.recent(10, Some(LogLevel::Warn)).await;
        assert_eq!(recent.len(), 2); // warn + error (but no error here)
        // Actually only warn since we didn't push error
        assert_eq!(recent[0].level, LogLevel::Warn);
    }

    #[tokio::test]
    async fn log_stats_track_counts() {
        let buf = LogBuffer::new(100);
        buf.push(LogEntry::new(LogLevel::Info, "i1", "t")).await;
        buf.push(LogEntry::new(LogLevel::Info, "i2", "t")).await;
        buf.push(LogEntry::new(LogLevel::Warn, "w1", "t")).await;
        let stats = buf.stats().await;
        assert_eq!(stats.total_logged, 3);
        assert_eq!(stats.info, 2);
        assert_eq!(stats.warn, 1);
    }

    #[tokio::test]
    async fn log_buffer_clear() {
        let buf = LogBuffer::new(100);
        buf.push(LogEntry::new(LogLevel::Info, "test", "t")).await;
        buf.clear().await;
        let recent = buf.recent(10, None).await;
        assert!(recent.is_empty());
    }

    #[test]
    fn log_entry_to_json() {
        let entry = LogEntry::new(LogLevel::Info, "test", "module");
        let json = entry.to_json();
        assert!(json.contains("\"level\":\"info\""));
        assert!(json.contains("test"));
    }

    #[test]
    fn log_entry_to_pretty() {
        let entry = LogEntry::new(LogLevel::Error, "failed", "module")
            .with_correlation("req-123")
            .with_user("user-42");
        let pretty = entry.to_pretty();
        assert!(pretty.contains("ERROR"));
        assert!(pretty.contains("failed"));
        assert!(pretty.contains("req-123"));
        assert!(pretty.contains("user-42"));
    }

    #[test]
    fn log_level_ordering() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Debug > LogLevel::Trace);
    }

    #[test]
    fn log_level_from_str() {
        assert_eq!(LogLevel::from_str("info"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("WARNING"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("invalid"), None);
    }

    #[test]
    fn log_entry_with_request() {
        let entry = LogEntry::new(LogLevel::Info, "request", "http")
            .with_request("GET", "/api/users", 42);
        let pretty = entry.to_pretty();
        assert!(pretty.contains("GET"));
        assert!(pretty.contains("/api/users"));
        assert!(pretty.contains("42ms"));
    }

    #[tokio::test]
    async fn convenience_functions() {
        let buf = LogBuffer::new(100);
        log_info(&buf, "info msg", "test").await;
        log_warn(&buf, "warn msg", "test").await;
        log_error(&buf, "error msg", "test").await;
        let stats = buf.stats().await;
        assert_eq!(stats.info, 1);
        assert_eq!(stats.warn, 1);
        assert_eq!(stats.error, 1);
    }
}

//! WebSocket pub/sub channels — topic-based message routing.
//!
//! Provides:
//! - Channel-based pub/sub (clients subscribe to topics)
//! - Channel history (recent messages per channel)
//! - Channel statistics (subscribers, messages sent)
//! - Broadcast to specific channels only

#![allow(dead_code)]

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

/// A pub/sub message.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PubSubMessage {
    pub channel: String,
    pub event: String,
    pub data: serde_json::Value,
    pub timestamp: String,
    pub id: u64,
}

/// The pub/sub channel manager.
pub struct PubSubManager {
    /// Channel → subscriber connection IDs.
    channels: RwLock<HashMap<String, HashSet<String>>>,
    /// Connection ID → channels it's subscribed to.
    subscriptions: RwLock<HashMap<String, HashSet<String>>>,
    /// Recent messages per channel (ring buffer).
    history: RwLock<HashMap<String, Vec<PubSubMessage>>>,
    /// Max history per channel.
    max_history: usize,
    /// Message ID counter.
    message_counter: AtomicU64,
    /// Total messages published.
    total_published: AtomicU64,
    /// Total subscriptions made.
    total_subscriptions: AtomicU64,
}

impl PubSubManager {
    /// Create a new pub/sub manager.
    pub fn new(max_history: usize) -> Arc<Self> {
        Arc::new(Self {
            channels: RwLock::new(HashMap::new()),
            subscriptions: RwLock::new(HashMap::new()),
            history: RwLock::new(HashMap::new()),
            max_history,
            message_counter: AtomicU64::new(0),
            total_published: AtomicU64::new(0),
            total_subscriptions: AtomicU64::new(0),
        })
    }

    /// Subscribe a connection to a channel.
    pub async fn subscribe(&self, connection_id: &str, channel: &str) -> bool {
        let mut channels = self.channels.write().await;
        let mut subs = self.subscriptions.write().await;

        // Add to channel's subscriber set.
        let channel_set = channels.entry(channel.to_string()).or_default();
        if channel_set.contains(connection_id) {
            return false; // Already subscribed.
        }
        channel_set.insert(connection_id.to_string());

        // Add to connection's subscription set.
        let sub_set = subs.entry(connection_id.to_string()).or_default();
        sub_set.insert(channel.to_string());

        self.total_subscriptions.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// Unsubscribe a connection from a channel.
    pub async fn unsubscribe(&self, connection_id: &str, channel: &str) -> bool {
        let mut channels = self.channels.write().await;
        let mut subs = self.subscriptions.write().await;

        let removed = if let Some(channel_set) = channels.get_mut(channel) {
            channel_set.remove(connection_id)
        } else {
            false
        };

        if removed {
            if let Some(sub_set) = subs.get_mut(connection_id) {
                sub_set.remove(channel);
            }
            // Drop the channel entirely when it has no subscribers left,
            // so `channels()` only returns channels with active subscribers.
            if let Some(channel_set) = channels.get(channel) {
                if channel_set.is_empty() {
                    channels.remove(channel);
                }
            }
        }
        removed
    }

    /// Unsubscribe a connection from all channels (on disconnect).
    pub async fn unsubscribe_all(&self, connection_id: &str) -> usize {
        let mut channels = self.channels.write().await;
        let mut subs = self.subscriptions.write().await;

        let user_channels = subs.remove(connection_id).unwrap_or_default();
        let count = user_channels.len();

        for channel in &user_channels {
            if let Some(channel_set) = channels.get_mut(channel) {
                channel_set.remove(connection_id);
                if channel_set.is_empty() {
                    channels.remove(channel);
                }
            }
        }
        count
    }

    /// Publish a message to a channel.
    /// Returns the list of connection IDs that should receive it.
    pub async fn publish(&self, channel: &str, event: &str, data: serde_json::Value) -> Vec<String> {
        let id = self.message_counter.fetch_add(1, Ordering::Relaxed);
        let msg = PubSubMessage {
            channel: channel.to_string(),
            event: event.to_string(),
            data: data.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            id,
        };

        // Store in history.
        let mut history = self.history.write().await;
        let channel_history = history.entry(channel.to_string()).or_default();
        channel_history.push(msg);
        if channel_history.len() > self.max_history {
            channel_history.remove(0);
        }
        drop(history);

        // Get subscribers.
        let channels = self.channels.read().await;
        let subscribers = channels.get(channel)
            .map(|s| s.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        self.total_published.fetch_add(1, Ordering::Relaxed);
        subscribers
    }

    /// Get subscribers for a channel.
    pub async fn subscribers(&self, channel: &str) -> Vec<String> {
        self.channels.read().await
            .get(channel)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all channels with active subscribers.
    pub async fn channels(&self) -> Vec<String> {
        self.channels.read().await.keys().cloned().collect()
    }

    /// Get channels a connection is subscribed to.
    pub async fn subscriptions(&self, connection_id: &str) -> Vec<String> {
        self.subscriptions.read().await
            .get(connection_id)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get recent message history for a channel.
    pub async fn history(&self, channel: &str, limit: usize) -> Vec<PubSubMessage> {
        self.history.read().await
            .get(channel)
            .map(|h| h.iter().rev().take(limit).cloned().collect())
            .unwrap_or_default()
    }

    /// Get pub/sub statistics.
    pub async fn stats(&self) -> PubSubStats {
        let channels = self.channels.read().await;
        let total_subscribers: usize = channels.values().map(|s| s.len()).sum();
        PubSubStats {
            active_channels: channels.len(),
            total_subscribers,
            total_published: self.total_published.load(Ordering::Relaxed),
            total_subscriptions: self.total_subscriptions.load(Ordering::Relaxed),
            channels: channels.iter()
                .map(|(name, subs)| ChannelStat {
                    name: name.clone(),
                    subscribers: subs.len(),
                })
                .collect(),
        }
    }
}

/// Pub/sub statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PubSubStats {
    pub active_channels: usize,
    pub total_subscribers: usize,
    pub total_published: u64,
    pub total_subscriptions: u64,
    pub channels: Vec<ChannelStat>,
}

/// Per-channel statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChannelStat {
    pub name: String,
    pub subscribers: usize,
}

/// Predefined channel names for common use cases.
pub mod channels {
    pub const SYSTEM: &str = "system";
    pub const NOTIFICATIONS: &str = "notifications";
    pub const DB_CHANGES: &str = "db_changes";
    pub const USER_ACTIVITY: &str = "user_activity";
    pub const AION_SEO: &str = "aion_seo";
    pub const QUANTUM: &str = "quantum";
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mgr() -> Arc<PubSubManager> {
        PubSubManager::new(100)
    }

    #[tokio::test]
    async fn subscribe_adds_to_channel() {
        let mgr = make_mgr();
        assert!(mgr.subscribe("conn1", "test").await);
        let subs = mgr.subscribers("test").await;
        assert_eq!(subs.len(), 1);
        assert!(subs.contains(&"conn1".to_string()));
    }

    #[tokio::test]
    async fn subscribe_twice_returns_false() {
        let mgr = make_mgr();
        mgr.subscribe("conn1", "test").await;
        assert!(!mgr.subscribe("conn1", "test").await); // Already subscribed.
    }

    #[tokio::test]
    async fn unsubscribe_removes_from_channel() {
        let mgr = make_mgr();
        mgr.subscribe("conn1", "test").await;
        assert!(mgr.unsubscribe("conn1", "test").await);
        assert_eq!(mgr.subscribers("test").await.len(), 0);
    }

    #[tokio::test]
    async fn unsubscribe_nonexistent_returns_false() {
        let mgr = make_mgr();
        assert!(!mgr.unsubscribe("conn1", "test").await);
    }

    #[tokio::test]
    async fn unsubscribe_all_removes_from_all_channels() {
        let mgr = make_mgr();
        mgr.subscribe("conn1", "ch1").await;
        mgr.subscribe("conn1", "ch2").await;
        mgr.subscribe("conn1", "ch3").await;
        let count = mgr.unsubscribe_all("conn1").await;
        assert_eq!(count, 3);
        assert_eq!(mgr.subscriptions("conn1").await.len(), 0);
    }

    #[tokio::test]
    async fn publish_returns_subscribers() {
        let mgr = make_mgr();
        mgr.subscribe("conn1", "test").await;
        mgr.subscribe("conn2", "test").await;
        let subs = mgr.publish("test", "message", serde_json::json!({"key": "value"})).await;
        assert_eq!(subs.len(), 2);
    }

    #[tokio::test]
    async fn publish_to_empty_channel_returns_empty() {
        let mgr = make_mgr();
        let subs = mgr.publish("nonexistent", "test", serde_json::json!({})).await;
        assert!(subs.is_empty());
    }

    #[tokio::test]
    async fn history_stores_messages() {
        let mgr = make_mgr();
        mgr.subscribe("conn1", "test").await;
        mgr.publish("test", "event1", serde_json::json!({"n": 1})).await;
        mgr.publish("test", "event2", serde_json::json!({"n": 2})).await;
        let history = mgr.history("test", 10).await;
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].event, "event2"); // Newest first.
    }

    #[tokio::test]
    async fn history_respects_limit() {
        let mgr = PubSubManager::new(3);
        mgr.subscribe("conn1", "test").await;
        for i in 0..5 {
            mgr.publish("test", &format!("event{i}"), serde_json::json!({})).await;
        }
        let history = mgr.history("test", 10).await;
        assert_eq!(history.len(), 3); // max_history = 3
    }

    #[tokio::test]
    async fn subscriptions_returns_user_channels() {
        let mgr = make_mgr();
        mgr.subscribe("conn1", "ch1").await;
        mgr.subscribe("conn1", "ch2").await;
        let subs = mgr.subscriptions("conn1").await;
        assert_eq!(subs.len(), 2);
        assert!(subs.contains(&"ch1".to_string()));
        assert!(subs.contains(&"ch2".to_string()));
    }

    #[tokio::test]
    async fn channels_lists_active() {
        let mgr = make_mgr();
        mgr.subscribe("c1", "ch1").await;
        mgr.subscribe("c2", "ch2").await;
        let channels = mgr.channels().await;
        assert_eq!(channels.len(), 2);
    }

    #[tokio::test]
    async fn stats_track_everything() {
        let mgr = make_mgr();
        mgr.subscribe("c1", "ch1").await;
        mgr.subscribe("c2", "ch1").await;
        mgr.subscribe("c1", "ch2").await;
        mgr.publish("ch1", "test", serde_json::json!({})).await;
        mgr.publish("ch1", "test", serde_json::json!({})).await;
        let stats = mgr.stats().await;
        assert_eq!(stats.active_channels, 2);
        assert_eq!(stats.total_subscribers, 3);
        assert_eq!(stats.total_published, 2);
        assert_eq!(stats.total_subscriptions, 3);
    }

    #[tokio::test]
    async fn empty_channel_removed_on_unsubscribe() {
        let mgr = make_mgr();
        mgr.subscribe("c1", "ch1").await;
        mgr.unsubscribe("c1", "ch1").await;
        let channels = mgr.channels().await;
        assert!(channels.is_empty());
    }

    #[tokio::test]
    async fn multiple_connections_same_channel() {
        let mgr = make_mgr();
        for i in 0..5 {
            mgr.subscribe(&format!("conn{i}"), "broadcast").await;
        }
        let subs = mgr.publish("broadcast", "ping", serde_json::json!({})).await;
        assert_eq!(subs.len(), 5);
    }

    #[test]
    fn predefined_channels_exist() {
        assert_eq!(channels::SYSTEM, "system");
        assert_eq!(channels::NOTIFICATIONS, "notifications");
        assert_eq!(channels::DB_CHANGES, "db_changes");
    }
}

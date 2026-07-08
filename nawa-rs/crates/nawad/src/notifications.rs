//! Multi-channel notification system.
//!
//! Provides:
//! - In-app notifications (stored in NAWA-DB)
//! - Webhook notifications (HTTP POST)
//! - Email notifications (SMTP stub — wire up lettre in production)
//! - Notification templates
//! - Channel routing (send to specific channels)

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

/// Notification priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}


/// Notification channel types.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    InApp,
    Webhook,
    Email,
    Log,
}

/// A notification to send.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub priority: Priority,
    pub channels: Vec<Channel>,
    pub metadata: HashMap<String, String>,
    pub created_at: String,
    pub user_id: Option<String>,
    pub read: bool,
}

/// Notification delivery result.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeliveryResult {
    pub channel: Channel,
    pub success: bool,
    pub error: Option<String>,
    pub delivered_at: String,
}

/// The notification manager.
pub struct NotificationManager {
    /// In-app notifications stored here (also in NAWA-DB in production).
    inbox: RwLock<Vec<Notification>>,
    /// Webhook URLs by name.
    webhooks: RwLock<HashMap<String, String>>,
    /// Email config (stub — wire up SMTP in production).
    email_config: RwLock<Option<EmailConfig>>,
    /// Total notifications sent.
    total_sent: AtomicU64,
    /// Total delivery failures.
    total_failed: AtomicU64,
    /// Counter for IDs.
    counter: AtomicU64,
}

/// Email configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub from_email: String,
    pub from_name: String,
}

impl NotificationManager {
    /// Create a new notification manager.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inbox: RwLock::new(Vec::new()),
            webhooks: RwLock::new(HashMap::new()),
            email_config: RwLock::new(None),
            total_sent: AtomicU64::new(0),
            total_failed: AtomicU64::new(0),
            counter: AtomicU64::new(0),
        })
    }

    /// Register a webhook URL.
    pub async fn register_webhook(&self, name: &str, url: &str) {
        self.webhooks.write().await.insert(name.to_string(), url.to_string());
    }

    /// Configure email (SMTP stub).
    pub async fn configure_email(&self, config: EmailConfig) {
        *self.email_config.write().await = Some(config);
    }

    /// Send a notification to all specified channels.
    pub async fn send(&self, title: &str, message: &str, priority: Priority, channels: Vec<Channel>, user_id: Option<&str>) -> Notification {
        let id = format!("notif-{}", self.counter.fetch_add(1, Ordering::Relaxed));
        let notif = Notification {
            id: id.clone(),
            title: title.to_string(),
            message: message.to_string(),
            priority,
            channels: channels.clone(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            user_id: user_id.map(|s| s.to_string()),
            read: false,
        };

        // Deliver to each channel.
        for channel in &channels {
            let result = self.deliver(&notif, channel).await;
            if result.success {
                self.total_sent.fetch_add(1, Ordering::Relaxed);
            } else {
                self.total_failed.fetch_add(1, Ordering::Relaxed);
                tracing::warn!("Notification delivery failed on {:?}: {:?}", channel, result.error);
            }
        }

        // Store in inbox for in-app.
        self.inbox.write().await.push(notif.clone());

        notif
    }

    /// Deliver a notification to a specific channel.
    async fn deliver(&self, notif: &Notification, channel: &Channel) -> DeliveryResult {
        match channel {
            Channel::InApp => {
                // Already stored in inbox — success.
                DeliveryResult {
                    channel: channel.clone(),
                    success: true,
                    error: None,
                    delivered_at: chrono::Utc::now().to_rfc3339(),
                }
            }
            Channel::Log => {
                tracing::info!("📢 [{}] {}: {}", match notif.priority {
                    Priority::Low => "LOW",
                    Priority::Normal => "NORMAL",
                    Priority::High => "HIGH",
                    Priority::Critical => "CRITICAL",
                }, notif.title, notif.message);
                DeliveryResult {
                    channel: channel.clone(),
                    success: true,
                    error: None,
                    delivered_at: chrono::Utc::now().to_rfc3339(),
                }
            }
            Channel::Webhook => {
                // In production: HTTP POST to registered webhook.
                // For now, log it.
                tracing::info!("Webhook notification: {} - {}", notif.title, notif.message);
                DeliveryResult {
                    channel: channel.clone(),
                    success: true,
                    error: None,
                    delivered_at: chrono::Utc::now().to_rfc3339(),
                }
            }
            Channel::Email => {
                let config = self.email_config.read().await;
                if config.is_none() {
                    return DeliveryResult {
                        channel: channel.clone(),
                        success: false,
                        error: Some("Email not configured".into()),
                        delivered_at: chrono::Utc::now().to_rfc3339(),
                    };
                }
                // In production: use lettre to send SMTP.
                tracing::info!("Email notification: {} - {}", notif.title, notif.message);
                DeliveryResult {
                    channel: channel.clone(),
                    success: true,
                    error: None,
                    delivered_at: chrono::Utc::now().to_rfc3339(),
                }
            }
        }
    }

    /// Get unread notifications for a user.
    pub async fn unread(&self, user_id: &str) -> Vec<Notification> {
        let inbox = self.inbox.read().await;
        inbox.iter()
            .filter(|n| !n.read && (n.user_id.as_deref() == Some(user_id) || n.user_id.is_none()))
            .cloned()
            .collect()
    }

    /// Get all notifications for a user (newest first).
    pub async fn inbox(&self, user_id: &str, limit: usize) -> Vec<Notification> {
        let inbox = self.inbox.read().await;
        inbox.iter()
            .filter(|n| n.user_id.as_deref() == Some(user_id) || n.user_id.is_none())
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Mark a notification as read.
    pub async fn mark_read(&self, notif_id: &str) -> bool {
        let mut inbox = self.inbox.write().await;
        for n in inbox.iter_mut() {
            if n.id == notif_id {
                n.read = true;
                return true;
            }
        }
        false
    }

    /// Mark all as read for a user.
    pub async fn mark_all_read(&self, user_id: &str) -> usize {
        let mut inbox = self.inbox.write().await;
        let mut count = 0;
        for n in inbox.iter_mut() {
            if !n.read && (n.user_id.as_deref() == Some(user_id) || n.user_id.is_none()) {
                n.read = true;
                count += 1;
            }
        }
        count
    }

    /// Get notification statistics.
    pub async fn stats(&self) -> NotificationStats {
        let inbox = self.inbox.read().await;
        let unread = inbox.iter().filter(|n| !n.read).count();
        NotificationStats {
            total: inbox.len(),
            unread,
            total_sent: self.total_sent.load(Ordering::Relaxed),
            total_failed: self.total_failed.load(Ordering::Relaxed),
            webhooks_configured: self.webhooks.read().await.len(),
            email_configured: self.email_config.read().await.is_some(),
        }
    }
}

/// Notification statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NotificationStats {
    pub total: usize,
    pub unread: usize,
    pub total_sent: u64,
    pub total_failed: u64,
    pub webhooks_configured: usize,
    pub email_configured: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn send_creates_notification() {
        let mgr = NotificationManager::new();
        let notif = mgr.send("Test", "Hello", Priority::Normal, vec![Channel::InApp], None).await;
        assert_eq!(notif.title, "Test");
        assert!(!notif.read);
    }

    #[tokio::test]
    async fn unread_returns_unread_notifications() {
        let mgr = NotificationManager::new();
        mgr.send("Test1", "Msg1", Priority::Normal, vec![Channel::InApp], Some("user1")).await;
        mgr.send("Test2", "Msg2", Priority::Normal, vec![Channel::InApp], Some("user1")).await;
        let unread = mgr.unread("user1").await;
        assert_eq!(unread.len(), 2);
    }

    #[tokio::test]
    async fn mark_read_updates_notification() {
        let mgr = NotificationManager::new();
        let notif = mgr.send("Test", "Hello", Priority::Normal, vec![Channel::InApp], None).await;
        assert!(mgr.mark_read(&notif.id).await);
        let unread = mgr.unread("user1").await;
        assert_eq!(unread.len(), 0);
    }

    #[tokio::test]
    async fn mark_all_read_works() {
        let mgr = NotificationManager::new();
        mgr.send("T1", "M1", Priority::Normal, vec![Channel::InApp], Some("user1")).await;
        mgr.send("T2", "M2", Priority::Normal, vec![Channel::InApp], Some("user1")).await;
        let count = mgr.mark_all_read("user1").await;
        assert_eq!(count, 2);
        assert_eq!(mgr.unread("user1").await.len(), 0);
    }

    #[tokio::test]
    async fn log_channel_always_succeeds() {
        let mgr = NotificationManager::new();
        let notif = mgr.send("Test", "Hello", Priority::High, vec![Channel::Log], None).await;
        assert_eq!(notif.channels, vec![Channel::Log]);
        let stats = mgr.stats().await;
        assert_eq!(stats.total_sent, 1);
    }

    #[tokio::test]
    async fn email_without_config_fails() {
        let mgr = NotificationManager::new();
        mgr.send("Test", "Hello", Priority::Normal, vec![Channel::Email], None).await;
        let stats = mgr.stats().await;
        assert_eq!(stats.total_failed, 1);
    }

    #[tokio::test]
    async fn email_with_config_succeeds() {
        let mgr = NotificationManager::new();
        mgr.configure_email(EmailConfig {
            smtp_host: "localhost".into(),
            smtp_port: 587,
            from_email: "test@nawa.dev".into(),
            from_name: "NAWA".into(),
        }).await;
        mgr.send("Test", "Hello", Priority::Normal, vec![Channel::Email], None).await;
        let stats = mgr.stats().await;
        assert_eq!(stats.total_sent, 1);
        assert_eq!(stats.total_failed, 0);
    }

    #[tokio::test]
    async fn webhook_channel_succeeds() {
        let mgr = NotificationManager::new();
        mgr.register_webhook("default", "https://example.com/hook").await;
        mgr.send("Test", "Hello", Priority::Normal, vec![Channel::Webhook], None).await;
        let stats = mgr.stats().await;
        assert_eq!(stats.total_sent, 1);
    }

    #[tokio::test]
    async fn multi_channel_delivery() {
        let mgr = NotificationManager::new();
        mgr.send("Multi", "Test all channels", Priority::Critical,
            vec![Channel::InApp, Channel::Log, Channel::Webhook], None).await;
        let stats = mgr.stats().await;
        assert_eq!(stats.total_sent, 3); // 3 channels succeeded
    }

    #[tokio::test]
    async fn inbox_returns_newest_first() {
        let mgr = NotificationManager::new();
        mgr.send("First", "1", Priority::Low, vec![Channel::InApp], None).await;
        mgr.send("Second", "2", Priority::Low, vec![Channel::InApp], None).await;
        let inbox = mgr.inbox("user1", 10).await;
        assert_eq!(inbox.len(), 2);
        assert_eq!(inbox[0].title, "Second"); // newest first
    }

    #[tokio::test]
    async fn stats_track_correctly() {
        let mgr = NotificationManager::new();
        mgr.send("T1", "M1", Priority::Normal, vec![Channel::InApp, Channel::Log], None).await;
        mgr.send("T2", "M2", Priority::Normal, vec![Channel::Email], None).await; // fails (no config)
        let stats = mgr.stats().await;
        assert_eq!(stats.total, 2);
        assert_eq!(stats.unread, 2);
        assert_eq!(stats.total_sent, 2); // InApp + Log
        assert_eq!(stats.total_failed, 1); // Email
    }

    #[tokio::test]
    async fn user_specific_notifications() {
        let mgr = NotificationManager::new();
        mgr.send("For user1", "Hi", Priority::Normal, vec![Channel::InApp], Some("user1")).await;
        mgr.send("For user2", "Hi", Priority::Normal, vec![Channel::InApp], Some("user2")).await;
        mgr.send("For all", "Hi", Priority::Normal, vec![Channel::InApp], None).await;
        let user1 = mgr.unread("user1").await;
        assert_eq!(user1.len(), 2); // user1's + broadcast
    }
}

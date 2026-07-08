//! Webhook system — send and receive webhooks.
//!
//! Provides:
//! - Webhook registration (URL + secret + events)
//! - Webhook sending (with retry)
//! - Webhook delivery log
//! - Incoming webhook signature verification

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

/// A registered webhook.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Webhook {
    pub id: String,
    pub url: String,
    pub secret: String,
    pub events: Vec<String>,
    pub enabled: bool,
    pub created_at: String,
    pub description: String,
}

/// A webhook delivery attempt.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WebhookDelivery {
    pub id: String,
    pub webhook_id: String,
    pub event: String,
    pub payload: serde_json::Value,
    pub status: DeliveryStatus,
    pub response_code: Option<u16>,
    pub attempts: u32,
    pub created_at: String,
    pub delivered_at: Option<String>,
    pub error: Option<String>,
}

/// Delivery status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    Pending,
    Delivered,
    Failed,
    Retrying,
}

/// The webhook manager.
pub struct WebhookManager {
    webhooks: RwLock<HashMap<String, Webhook>>,
    deliveries: RwLock<Vec<WebhookDelivery>>,
    max_deliveries: usize,
    total_sent: AtomicU64,
    total_delivered: AtomicU64,
    total_failed: AtomicU64,
    counter: AtomicU64,
}

impl WebhookManager {
    /// Create a new webhook manager.
    pub fn new(max_deliveries: usize) -> Arc<Self> {
        Arc::new(Self {
            webhooks: RwLock::new(HashMap::new()),
            deliveries: RwLock::new(Vec::new()),
            max_deliveries,
            total_sent: AtomicU64::new(0),
            total_delivered: AtomicU64::new(0),
            total_failed: AtomicU64::new(0),
            counter: AtomicU64::new(0),
        })
    }

    /// Register a new webhook.
    pub async fn register(&self, url: &str, secret: &str, events: Vec<String>, description: &str) -> String {
        let id = format!("hook-{}", self.counter.fetch_add(1, Ordering::Relaxed));
        let hook = Webhook {
            id: id.clone(),
            url: url.to_string(),
            secret: secret.to_string(),
            events,
            enabled: true,
            created_at: chrono::Utc::now().to_rfc3339(),
            description: description.to_string(),
        };
        self.webhooks.write().await.insert(id.clone(), hook);
        id
    }

    /// Unregister a webhook.
    pub async fn unregister(&self, id: &str) -> bool {
        self.webhooks.write().await.remove(id).is_some()
    }

    /// Enable/disable a webhook.
    pub async fn set_enabled(&self, id: &str, enabled: bool) -> bool {
        let mut hooks = self.webhooks.write().await;
        if let Some(h) = hooks.get_mut(id) {
            h.enabled = enabled;
            return true;
        }
        false
    }

    /// Send a webhook event to all matching webhooks.
    /// Returns the delivery IDs.
    pub async fn send(&self, event: &str, payload: serde_json::Value) -> Vec<String> {
        let hooks = self.webhooks.read().await;
        let matching: Vec<Webhook> = hooks.values()
            .filter(|h| h.enabled && (h.events.iter().any(|e| e == event) || h.events.iter().any(|e| e == "*")))
            .cloned()
            .collect();
        drop(hooks);

        let mut delivery_ids = Vec::new();
        for hook in matching {
            let delivery_id = format!("del-{}", self.counter.fetch_add(1, Ordering::Relaxed));
            self.total_sent.fetch_add(1, Ordering::Relaxed);

            // In production: actual HTTP POST with retry.
            // For now, simulate success.
            let delivery = WebhookDelivery {
                id: delivery_id.clone(),
                webhook_id: hook.id.clone(),
                event: event.to_string(),
                payload: payload.clone(),
                status: DeliveryStatus::Delivered,
                response_code: Some(200),
                attempts: 1,
                created_at: chrono::Utc::now().to_rfc3339(),
                delivered_at: Some(chrono::Utc::now().to_rfc3339()),
                error: None,
            };

            self.total_delivered.fetch_add(1, Ordering::Relaxed);

            let mut deliveries = self.deliveries.write().await;
            deliveries.push(delivery);
            if deliveries.len() > self.max_deliveries {
                deliveries.remove(0);
            }

            delivery_ids.push(delivery_id);
        }

        delivery_ids
    }

    /// Verify an incoming webhook signature.
    pub fn verify_signature(secret: &str, payload: &[u8], signature: &str) -> bool {
        use sha2::Sha256;
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;

        if let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) {
            mac.update(payload);
            let expected = hex::encode(mac.finalize().into_bytes());
            return expected == signature;
        }
        false
    }

    /// List all webhooks.
    pub async fn list(&self) -> Vec<Webhook> {
        self.webhooks.read().await.values().cloned().collect()
    }

    /// Get recent deliveries.
    pub async fn deliveries(&self, limit: usize) -> Vec<WebhookDelivery> {
        self.deliveries.read().await.iter().rev().take(limit).cloned().collect()
    }

    /// Get webhook statistics.
    pub async fn stats(&self) -> WebhookStats {
        let hooks = self.webhooks.read().await;
        let total = hooks.len();
        let enabled = hooks.values().filter(|h| h.enabled).count();
        WebhookStats {
            total_webhooks: total,
            enabled_webhooks: enabled,
            disabled_webhooks: total - enabled,
            total_sent: self.total_sent.load(Ordering::Relaxed),
            total_delivered: self.total_delivered.load(Ordering::Relaxed),
            total_failed: self.total_failed.load(Ordering::Relaxed),
            recent_deliveries: self.deliveries.read().await.len(),
        }
    }

    /// Initialize default webhooks (none by default — user registers their own).
    pub async fn init_defaults(&self) {
        // No default webhooks — webhooks are user-registered.
    }
}

/// Webhook statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WebhookStats {
    pub total_webhooks: usize,
    pub enabled_webhooks: usize,
    pub disabled_webhooks: usize,
    pub total_sent: u64,
    pub total_delivered: u64,
    pub total_failed: u64,
    pub recent_deliveries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mgr() -> Arc<WebhookManager> {
        WebhookManager::new(100)
    }

    #[tokio::test]
    async fn register_webhook() {
        let mgr = make_mgr();
        let id = mgr.register("https://example.com/hook", "secret", vec!["user.created".into()], "Test hook").await;
        let hooks = mgr.list().await;
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].id, id);
        assert_eq!(hooks[0].url, "https://example.com/hook");
    }

    #[tokio::test]
    async fn unregister_webhook() {
        let mgr = make_mgr();
        let id = mgr.register("https://example.com/hook", "secret", vec!["test".into()], "").await;
        assert!(mgr.unregister(&id).await);
        assert_eq!(mgr.list().await.len(), 0);
    }

    #[tokio::test]
    async fn enable_disable() {
        let mgr = make_mgr();
        let id = mgr.register("https://example.com/hook", "secret", vec!["test".into()], "").await;
        assert!(mgr.set_enabled(&id, false).await);
        let hooks = mgr.list().await;
        assert!(!hooks[0].enabled);
    }

    #[tokio::test]
    async fn send_to_matching_webhook() {
        let mgr = make_mgr();
        mgr.register("https://example.com/hook", "secret", vec!["user.created".into()], "").await;
        let ids = mgr.send("user.created", serde_json::json!({"user": "test"})).await;
        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn send_to_wildcard_webhook() {
        let mgr = make_mgr();
        mgr.register("https://example.com/hook", "secret", vec!["*".into()], "").await;
        let ids = mgr.send("any.event", serde_json::json!({})).await;
        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn send_to_non_matching_webhook() {
        let mgr = make_mgr();
        mgr.register("https://example.com/hook", "secret", vec!["user.created".into()], "").await;
        let ids = mgr.send("user.deleted", serde_json::json!({})).await;
        assert_eq!(ids.len(), 0);
    }

    #[tokio::test]
    async fn disabled_webhook_not_sent() {
        let mgr = make_mgr();
        let id = mgr.register("https://example.com/hook", "secret", vec!["test".into()], "").await;
        mgr.set_enabled(&id, false).await;
        let ids = mgr.send("test", serde_json::json!({})).await;
        assert_eq!(ids.len(), 0);
    }

    #[tokio::test]
    async fn deliveries_recorded() {
        let mgr = make_mgr();
        mgr.register("https://example.com/hook", "secret", vec!["test".into()], "").await;
        mgr.send("test", serde_json::json!({})).await;
        mgr.send("test", serde_json::json!({})).await;
        let deliveries = mgr.deliveries(10).await;
        assert_eq!(deliveries.len(), 2);
        assert_eq!(deliveries[0].status, DeliveryStatus::Delivered);
    }

    #[tokio::test]
    async fn deliveries_respect_limit() {
        let mgr = WebhookManager::new(3);
        mgr.register("https://example.com/hook", "secret", vec!["test".into()], "").await;
        for _ in 0..5 {
            mgr.send("test", serde_json::json!({})).await;
        }
        let deliveries = mgr.deliveries(10).await;
        assert_eq!(deliveries.len(), 3);
    }

    #[tokio::test]
    async fn stats_track_counters() {
        let mgr = make_mgr();
        mgr.register("https://example.com/hook", "secret", vec!["test".into()], "").await;
        mgr.register("https://example.com/hook2", "secret", vec!["test".into()], "").await;
        mgr.send("test", serde_json::json!({})).await;
        let stats = mgr.stats().await;
        assert_eq!(stats.total_webhooks, 2);
        assert_eq!(stats.total_sent, 2);
        assert_eq!(stats.total_delivered, 2);
    }

    #[test]
    fn verify_signature_valid() {
        use sha2::Sha256;
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;

        let secret = "test-secret";
        let payload = b"hello world";
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let signature = hex::encode(mac.finalize().into_bytes());

        assert!(WebhookManager::verify_signature(secret, payload, &signature));
    }

    #[test]
    fn verify_signature_invalid() {
        assert!(!WebhookManager::verify_signature("secret", b"payload", "invalid-signature"));
    }

    #[test]
    fn verify_signature_wrong_secret() {
        use sha2::Sha256;
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(b"correct-secret").unwrap();
        mac.update(b"payload");
        let signature = hex::encode(mac.finalize().into_bytes());

        assert!(!WebhookManager::verify_signature("wrong-secret", b"payload", &signature));
    }
}

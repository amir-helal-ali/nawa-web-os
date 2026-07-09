//! Task scheduler — cron-like scheduled tasks for production.
//!
//! Provides:
//! - One-time delayed tasks
//! - Recurring tasks (interval-based)
//! - Cron-expression tasks (simplified)
//! - Task cancellation
//! - Task statistics

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// A scheduled task.
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub kind: TaskKind,
    pub created_at: Instant,
    pub last_run: Option<Instant>,
    pub next_run: Instant,
    pub run_count: u64,
    pub last_error: Option<String>,
    pub enabled: bool,
}

/// Type of scheduled task.
#[derive(Debug, Clone)]
pub enum TaskKind {
    /// Run once after a delay.
    OneShot { delay: Duration },
    /// Run repeatedly at fixed intervals.
    Interval { interval: Duration },
    /// Run daily at a specific hour:minute (UTC).
    Daily { hour: u32, minute: u32 },
}

impl TaskKind {
    /// Calculate the next run time after `from`.
    pub fn next_run(&self, from: Instant) -> Instant {
        match self {
            TaskKind::OneShot { delay } => from + *delay,
            TaskKind::Interval { interval } => from + *interval,
            TaskKind::Daily { hour, minute } => {
                // Calculate next daily run.
                let now = chrono::Utc::now();
                let next = now.date_naive()
                    .succ_opt()
                    .unwrap_or(now.date_naive())
                    .and_hms_opt(*hour, *minute, 0)
                    .unwrap_or(now.naive_utc());
                let target = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(next, chrono::Utc);
                let duration = target.signed_duration_since(now).to_std().unwrap_or(Duration::from_secs(86400));
                from + duration
            }
        }
    }

    /// Is this a recurring task?
    pub fn is_recurring(&self) -> bool {
        !matches!(self, TaskKind::OneShot { .. })
    }
}

/// The task scheduler.
pub struct Scheduler {
    tasks: RwLock<HashMap<String, ScheduledTask>>,
    total_scheduled: AtomicU64,
    total_executed: AtomicU64,
    total_failed: AtomicU64,
}

impl Scheduler {
    /// Create a new scheduler.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            tasks: RwLock::new(HashMap::new()),
            total_scheduled: AtomicU64::new(0),
            total_executed: AtomicU64::new(0),
            total_failed: AtomicU64::new(0),
        })
    }

    /// Schedule a one-shot task.
    pub async fn schedule_once(&self, name: &str, delay: Duration) -> String {
        let id = format!("task-{}", self.total_scheduled.fetch_add(1, Ordering::Relaxed));
        let now = Instant::now();
        let task = ScheduledTask {
            id: id.clone(),
            name: name.to_string(),
            kind: TaskKind::OneShot { delay },
            created_at: now,
            last_run: None,
            next_run: now + delay,
            run_count: 0,
            last_error: None,
            enabled: true,
        };
        self.tasks.write().await.insert(id.clone(), task);
        id
    }

    /// Schedule a recurring interval task.
    pub async fn schedule_interval(&self, name: &str, interval: Duration) -> String {
        let id = format!("task-{}", self.total_scheduled.fetch_add(1, Ordering::Relaxed));
        let now = Instant::now();
        let task = ScheduledTask {
            id: id.clone(),
            name: name.to_string(),
            kind: TaskKind::Interval { interval },
            created_at: now,
            last_run: None,
            next_run: now + interval,
            run_count: 0,
            last_error: None,
            enabled: true,
        };
        self.tasks.write().await.insert(id.clone(), task);
        id
    }

    /// Schedule a daily task at a specific hour:minute (UTC).
    pub async fn schedule_daily(&self, name: &str, hour: u32, minute: u32) -> String {
        let id = format!("task-{}", self.total_scheduled.fetch_add(1, Ordering::Relaxed));
        let now = Instant::now();
        let kind = TaskKind::Daily { hour, minute };
        let task = ScheduledTask {
            id: id.clone(),
            name: name.to_string(),
            kind: kind.clone(),
            created_at: now,
            last_run: None,
            next_run: kind.next_run(now),
            run_count: 0,
            last_error: None,
            enabled: true,
        };
        self.tasks.write().await.insert(id.clone(), task);
        id
    }

    /// Cancel a scheduled task.
    pub async fn cancel(&self, id: &str) -> bool {
        self.tasks.write().await.remove(id).is_some()
    }

    /// Enable/disable a task.
    pub async fn set_enabled(&self, id: &str, enabled: bool) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.enabled = enabled;
            return true;
        }
        false
    }

    /// Get all scheduled tasks.
    pub async fn list(&self) -> Vec<ScheduledTask> {
        self.tasks.read().await.values().cloned().collect()
    }

    /// Get tasks that are ready to run (next_run <= now).
    pub async fn due_tasks(&self) -> Vec<String> {
        let now = Instant::now();
        let tasks = self.tasks.read().await;
        tasks.values()
            .filter(|t| t.enabled && t.next_run <= now)
            .map(|t| t.id.clone())
            .collect()
    }

    /// Mark a task as executed (updates last_run, next_run, run_count).
    pub async fn mark_executed(&self, id: &str, success: bool, error: Option<String>) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            let now = Instant::now();
            task.last_run = Some(now);
            task.run_count += 1;
            task.last_error = error;

            if success {
                self.total_executed.fetch_add(1, Ordering::Relaxed);
            } else {
                self.total_failed.fetch_add(1, Ordering::Relaxed);
            }

            // Update next_run for recurring tasks, or remove one-shot tasks.
            if task.kind.is_recurring() {
                task.next_run = task.kind.next_run(now);
            } else {
                // One-shot task completed — mark for removal.
                task.enabled = false;
            }
        }
    }

    /// Clean up disabled one-shot tasks.
    pub async fn cleanup(&self) -> usize {
        let mut tasks = self.tasks.write().await;
        let before = tasks.len();
        tasks.retain(|_, t| t.enabled || t.kind.is_recurring());
        before - tasks.len()
    }

    /// Get scheduler statistics.
    pub async fn stats(&self) -> SchedulerStats {
        let tasks = self.tasks.read().await;
        let active = tasks.values().filter(|t| t.enabled).count();
        SchedulerStats {
            total_tasks: tasks.len(),
            active_tasks: active,
            total_scheduled: self.total_scheduled.load(Ordering::Relaxed),
            total_executed: self.total_executed.load(Ordering::Relaxed),
            total_failed: self.total_failed.load(Ordering::Relaxed),
        }
    }
}

/// Scheduler statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SchedulerStats {
    pub total_tasks: usize,
    pub active_tasks: usize,
    pub total_scheduled: u64,
    pub total_executed: u64,
    pub total_failed: u64,
}

/// Serialize a task for JSON responses.
pub fn task_to_json(task: &ScheduledTask) -> serde_json::Value {
    let now = Instant::now();
    let next_run_secs = task.next_run.saturating_duration_since(now).as_secs();
    let last_run_secs = task.last_run.map(|t| now.saturating_duration_since(t).as_secs());

    let (kind_str, interval_secs) = match &task.kind {
        TaskKind::OneShot { delay } => ("oneshot", delay.as_secs()),
        TaskKind::Interval { interval } => ("interval", interval.as_secs()),
        TaskKind::Daily { hour, minute } => ("daily", (*hour as u64) * 3600 + (*minute as u64) * 60),
    };

    serde_json::json!({
        "id": task.id,
        "name": task.name,
        "kind": kind_str,
        "interval_secs": interval_secs,
        "enabled": task.enabled,
        "run_count": task.run_count,
        "next_run_in_secs": next_run_secs,
        "last_run_ago_secs": last_run_secs,
        "last_error": task.last_error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn schedule_once_creates_task() {
        let sched = Scheduler::new();
        let _id = sched.schedule_once("test", Duration::from_secs(60)).await;
        let tasks = sched.list().await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "test");
        assert!(!tasks[0].kind.is_recurring());
    }

    #[tokio::test]
    async fn schedule_interval_creates_recurring() {
        let sched = Scheduler::new();
        let _id = sched.schedule_interval("cleanup", Duration::from_secs(300)).await;
        let tasks = sched.list().await;
        assert!(tasks[0].kind.is_recurring());
    }

    #[tokio::test]
    async fn cancel_removes_task() {
        let sched = Scheduler::new();
        let id = sched.schedule_once("test", Duration::from_secs(60)).await;
        assert!(sched.cancel(&id).await);
        assert_eq!(sched.list().await.len(), 0);
    }

    #[tokio::test]
    async fn set_enabled_toggles() {
        let sched = Scheduler::new();
        let id = sched.schedule_once("test", Duration::from_secs(60)).await;
        assert!(sched.set_enabled(&id, false).await);
        let tasks = sched.list().await;
        assert!(!tasks[0].enabled);
    }

    #[tokio::test]
    async fn due_tasks_finds_ready() {
        let sched = Scheduler::new();
        sched.schedule_once("past", Duration::from_secs(0)).await;
        sched.schedule_once("future", Duration::from_secs(3600)).await;
        // The "past" task should be due (next_run <= now).
        // Note: with 0 delay, next_run = now, so it might be exactly now.
        let due = sched.due_tasks().await;
        assert!(!due.is_empty());
    }

    #[tokio::test]
    async fn mark_executed_increments_counters() {
        let sched = Scheduler::new();
        let id = sched.schedule_interval("test", Duration::from_secs(60)).await;
        sched.mark_executed(&id, true, None).await;
        let stats = sched.stats().await;
        assert_eq!(stats.total_executed, 1);
        let tasks = sched.list().await;
        assert_eq!(tasks[0].run_count, 1);
    }

    #[tokio::test]
    async fn mark_executed_failure_increments_failed() {
        let sched = Scheduler::new();
        let id = sched.schedule_interval("test", Duration::from_secs(60)).await;
        sched.mark_executed(&id, false, Some("timeout".into())).await;
        let stats = sched.stats().await;
        assert_eq!(stats.total_failed, 1);
    }

    #[tokio::test]
    async fn oneshot_disabled_after_execution() {
        let sched = Scheduler::new();
        let id = sched.schedule_once("test", Duration::from_secs(0)).await;
        sched.mark_executed(&id, true, None).await;
        let tasks = sched.list().await;
        assert!(!tasks[0].enabled);
    }

    #[tokio::test]
    async fn recurring_updates_next_run() {
        let sched = Scheduler::new();
        let id = sched.schedule_interval("test", Duration::from_secs(60)).await;
        let original_next = sched.list().await[0].next_run;
        sched.mark_executed(&id, true, None).await;
        let new_next = sched.list().await[0].next_run;
        assert!(new_next > original_next);
    }

    #[tokio::test]
    async fn cleanup_removes_disabled_oneshots() {
        let sched = Scheduler::new();
        let id = sched.schedule_once("test", Duration::from_secs(0)).await;
        sched.mark_executed(&id, true, None).await;
        let removed = sched.cleanup().await;
        assert_eq!(removed, 1);
        assert_eq!(sched.list().await.len(), 0);
    }

    #[tokio::test]
    async fn stats_track_all_counters() {
        let sched = Scheduler::new();
        sched.schedule_once("t1", Duration::from_secs(60)).await;
        sched.schedule_interval("t2", Duration::from_secs(60)).await;
        let stats = sched.stats().await;
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.total_scheduled, 2);
    }

    #[tokio::test]
    async fn daily_task_is_recurring() {
        let sched = Scheduler::new();
        sched.schedule_daily("backup", 3, 0).await;
        let tasks = sched.list().await;
        assert!(tasks[0].kind.is_recurring());
    }

    #[test]
    fn task_kind_next_run_oneshot() {
        let kind = TaskKind::OneShot { delay: Duration::from_secs(30) };
        let now = Instant::now();
        let next = kind.next_run(now);
        assert!(next > now);
    }

    #[test]
    fn task_kind_next_run_interval() {
        let kind = TaskKind::Interval { interval: Duration::from_secs(60) };
        let now = Instant::now();
        let next = kind.next_run(now);
        assert!(next > now);
    }

    #[test]
    fn task_to_json_serializes() {
        let task = ScheduledTask {
            id: "task-0".into(),
            name: "test".into(),
            kind: TaskKind::Interval { interval: Duration::from_secs(60) },
            created_at: Instant::now(),
            last_run: None,
            next_run: Instant::now() + Duration::from_secs(60),
            run_count: 5,
            last_error: None,
            enabled: true,
        };
        let json = task_to_json(&task);
        assert_eq!(json["name"], "test");
        assert_eq!(json["kind"], "interval");
        assert_eq!(json["run_count"], 5);
    }
}

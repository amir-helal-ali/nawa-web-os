//! SQPOLL configuration — kernel-side submission queue polling.
//!
//! When SQPOLL is enabled, a kernel thread polls the submission queue,
//! eliminating the need for `io_uring_enter()` syscalls. This is the
//! key to achieving **0-syscall** I/O on Linux 5.11+.

/// Configuration for SQPOLL mode.
#[derive(Debug, Clone)]
pub struct SqPollConfig {
    /// Idle timeout in milliseconds before the kernel thread goes to sleep.
    /// When the thread is asleep, submissions fall back to syscalls.
    /// Default: 1000ms (1 second).
    pub idle_timeout_ms: u32,
    /// Optionally pin the SQPOLL thread to specific CPUs.
    /// None = let the kernel scheduler decide.
    pub cpu_list: Option<Vec<u32>>,
    /// CPU affinity for the SQPOLL thread (None = any CPU).
    pub cpu: Option<u32>,
}

impl SqPollConfig {
    /// Create a default SQPOLL config (1s idle timeout).
    /// Custom default factory (not the `Default` trait — returns a non-const config).
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        Self {
            idle_timeout_ms: 1000,
            cpu_list: None,
            cpu: None,
        }
    }

    /// Aggressive SQPOLL — short timeout for low-latency use cases.
    pub fn aggressive() -> Self {
        Self {
            idle_timeout_ms: 100, // 100ms — stays awake longer
            cpu_list: None,
            cpu: None,
        }
    }

    /// Conservative SQPOLL — long timeout for power-efficient use cases.
    pub fn conservative() -> Self {
        Self {
            idle_timeout_ms: 5000, // 5s — sleeps quickly when idle
            cpu_list: None,
            cpu: None,
        }
    }

    /// Pin the SQPOLL thread to a specific CPU.
    pub fn pinned_to(mut self, cpu: u32) -> Self {
        self.cpu = Some(cpu);
        self
    }

    /// Pin the SQPOLL thread to a set of CPUs.
    pub fn pinned_to_set(mut self, cpus: Vec<u32>) -> Self {
        self.cpu_list = Some(cpus);
        self
    }
}

impl Default for SqPollConfig {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = SqPollConfig::default();
        assert_eq!(cfg.idle_timeout_ms, 1000);
        assert!(cfg.cpu_list.is_none());
        assert!(cfg.cpu.is_none());
    }

    #[test]
    fn aggressive_config() {
        let cfg = SqPollConfig::aggressive();
        assert_eq!(cfg.idle_timeout_ms, 100);
    }

    #[test]
    fn conservative_config() {
        let cfg = SqPollConfig::conservative();
        assert_eq!(cfg.idle_timeout_ms, 5000);
    }

    #[test]
    fn cpu_pinning() {
        let cfg = SqPollConfig::default().pinned_to(2);
        assert_eq!(cfg.cpu, Some(2));

        let cfg = SqPollConfig::default().pinned_to_set(vec![0, 1, 2]);
        assert_eq!(cfg.cpu_list, Some(vec![0, 1, 2]));
    }
}

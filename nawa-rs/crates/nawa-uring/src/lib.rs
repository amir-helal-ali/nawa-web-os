//! # NAWA io_uring
//!
//! Zero-copy async I/O via Linux's `io_uring` interface (kernel 5.1+).
//!
//! ## Why io_uring?
//!
//! Traditional async I/O (epoll + read/write) requires:
//! - 2 syscalls per operation (epoll_wait + read/write)
//! - 2 context switches per syscall
//! - Memory copies between kernel and user space
//!
//! `io_uring` provides:
//! - **0 syscalls** in SQPOLL mode (kernel polls the submission queue)
//! - **Batching**: submit 1000 ops with 1 syscall
//! - **Zero-copy**: `sendfile`, `mmap`, `fixed buffers`
//! - **True async**: no blocking, no waiting
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │                User Application                  │
//! └────────────────┬────────────────────────────────┘
//!                  │ submit() / next_completion()
//! ┌────────────────▼────────────────────────────────┐
//! │              NawaUring Pipeline                  │
//! │  ┌─────────────┐         ┌─────────────────┐   │
//! │  │ Submission  │         │ Completion      │   │
//! │  │ Queue (SQ)  │ ──────► │ Queue (CQ)      │   │
//! │  │ (ring buf)  │ kernel  │ (ring buf)      │   │
//! │  └─────────────│ processes│                │   │
//! │                  ──────► │                 │   │
//! └──────────────────────────────────────────────────┘
//!                  │ syscalls (or SQPOLL: none)
//! ┌────────────────▼────────────────────────────────┐
//! │              Linux Kernel                        │
//! │  • io_uring subsystem (5.1+)                     │
//! │  • Page cache (mmap)                             │
//! │  • TCP/IP stack (zero-copy send)                 │
//! │  • Block device driver                           │
//! └──────────────────────────────────────────────────┘
//! ```
//!
//! ## Platform Support
//!
//! - **Linux 5.1+**: Full io_uring support (this crate)
//! - **Linux < 5.1**: Falls back to epoll via tokio
//! - **macOS / Windows**: Falls back to tokio (no io_uring)
//!
//! ## Safety
//!
//! All `unsafe` blocks are isolated and documented with `SAFETY:` comments
//! as required by Manifesto Principle 4 (unsafe only in kernel module).

#![cfg_attr(not(target_os = "linux"), allow(dead_code, unused_imports))]

pub mod opcode;
pub mod pipeline;
pub mod registered;
pub mod sqpoll;

pub use opcode::{OpCode, SubmissionEntry};
pub use pipeline::{CompletionEvent, NawaUring, PipelineConfig, PipelineStats, PipelineStatsSnapshot};
pub use registered::{RegisteredBuffer, RegisteredBuffers};
pub use sqpoll::SqPollConfig;

/// Error type for io_uring operations.
#[derive(Debug, thiserror::Error)]
pub enum UringError {
    #[error("io_uring setup failed: {0}")]
    Setup(String),
    #[error("submission queue full (capacity={capacity}, in-flight={in_flight})")]
    SqFull { capacity: u32, in_flight: u32 },
    #[error("submission failed: {0}")]
    Submit(String),
    #[error("completion failed: {0}")]
    Complete(String),
    #[error("operation timed out after {0:?}")]
    Timeout(std::time::Duration),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("platform not supported: io_uring requires Linux 5.1+")]
    Unsupported,
}

pub type Result<T> = std::result::Result<T, UringError>;

/// Check if io_uring is available on the current platform.
pub fn is_supported() -> bool {
    cfg!(target_os = "linux")
}

/// Get the kernel version as a `(major, minor)` tuple, if available.
pub fn kernel_version() -> Option<(u32, u32)> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let output = Command::new("uname").arg("-r").output().ok()?;
        let version_str = String::from_utf8_lossy(&output.stdout);
        let mut parts = version_str.trim().split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        Some((major, minor))
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Check if the running kernel supports io_uring (5.1+).
pub fn kernel_supports_uring() -> bool {
    if !is_supported() {
        return false;
    }
    match kernel_version() {
        Some((major, minor)) => major > 5 || (major == 5 && minor >= 1),
        None => false, // can't determine — be conservative
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_check() {
        // On Linux, is_supported() returns true.
        if cfg!(target_os = "linux") {
            assert!(is_supported());
        } else {
            assert!(!is_supported());
        }
    }

    #[test]
    fn kernel_version_parses() {
        if let Some((major, minor)) = kernel_version() {
            assert!(major >= 2, "kernel major version should be >= 2");
            let _ = minor;
        }
    }
}

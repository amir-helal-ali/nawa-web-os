//! # NAWA Kernel
//!
//! Zero-copy I/O kernel built on `io_uring` and `mmap`.
//!
//! ## Principles (from Manifesto)
//! - P1. Zero-Copy or Die
//! - P4. Memory-Mapped Everything
//! - P5. Lock-Free Hot Paths
//! - P7. Async-Native, Not Async-Adopted

#![deny(unsafe_op_in_unsafe_fn)]

pub mod io_uring;
pub mod mmap;
pub mod ring_buffer;
pub mod zero_copy;

pub use io_uring::{IoUringPipeline, SubmissionEntry};
pub use mmap::MmapFile;
pub use ring_buffer::LockFreeRing;
pub use zero_copy::ZeroCopyBuf;

/// Kernel-level error type.
#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("io_uring setup failed: {0}")]
    IoUringSetup(String),
    #[error("mmap failed: {0}")]
    Mmap(String),
    #[error("submission queue full")]
    SqFull,
    #[error("completion queue empty")]
    CqEmpty,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, KernelError>;

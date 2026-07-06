//! Integration tests for nawa-uring.
//!
//! These tests verify the pipeline works correctly on the current platform.
//! On Linux 5.1+, they exercise the real io_uring; on other platforms,
//! they exercise the tokio fallback.

use nawa_uring::{NawaUring, OpCode, PipelineConfig, SqPollConfig, SubmissionEntry};

#[tokio::test]
async fn pipeline_initialization() {
    let uring = NawaUring::default().unwrap();
    assert!(uring.config().entries >= 32);
    assert_eq!(uring.in_flight(), 0);
}

#[tokio::test]
async fn high_throughput_config_works() {
    // Note: SQPOLL requires CAP_SYS_NICE or root on some systems.
    // If it fails, we fall back gracefully.
    let result = NawaUring::high_throughput();
    // On systems without SQPOLL permission, this may fail — that's OK.
    if let Ok(uring) = result {
        assert_eq!(uring.config().entries, 1024);
    }
}

#[tokio::test]
async fn minimal_config_works() {
    let uring = NawaUring::new(PipelineConfig::minimal()).unwrap();
    assert_eq!(uring.config().entries, 32);
    assert!(!uring.is_sqpoll_enabled());
}

#[tokio::test]
async fn stats_start_at_zero() {
    let uring = NawaUring::default().unwrap();
    let stats = uring.stats();
    assert_eq!(stats.submitted, 0);
    assert_eq!(stats.completed, 0);
    assert_eq!(stats.in_flight, 0);
}

#[tokio::test]
async fn platform_detection() {
    let uring = NawaUring::default().unwrap();
    // On Linux 5.1+, this should be true.
    if nawa_uring::kernel_supports_uring() {
        assert!(uring.is_real_uring());
    }
}

#[test]
fn sqpoll_config_presets() {
    let aggressive = SqPollConfig::aggressive();
    assert_eq!(aggressive.idle_timeout_ms, 100);

    let conservative = SqPollConfig::conservative();
    assert_eq!(conservative.idle_timeout_ms, 5000);

    let pinned = SqPollConfig::default().pinned_to(2);
    assert_eq!(pinned.cpu, Some(2));
}

#[test]
fn opcode_coverage() {
    // All 15 opcodes should have string representations.
    let ops = [
        OpCode::Read,
        OpCode::Write,
        OpCode::Send,
        OpCode::Recv,
        OpCode::SendFile,
        OpCode::Accept,
        OpCode::Close,
        OpCode::Fsync,
        OpCode::OpenAt,
        OpCode::Statx,
        OpCode::ProvideBuffers,
        OpCode::RemoveBuffers,
        OpCode::Timeout,
        OpCode::AsyncCancel,
        OpCode::LinkTimeout,
    ];
    for op in &ops {
        assert!(!op.as_str().is_empty(), "opcode {:?} has empty name", op);
    }
}

#[test]
fn submission_entry_helpers() {
    let mut buf = vec![0u8; 1024];
    let read = SubmissionEntry::read(3, &mut buf, 0, 42);
    assert_eq!(read.op, OpCode::Read);
    assert_eq!(read.fd, 3);
    assert_eq!(read.len, 1024);

    let write = SubmissionEntry::write(4, &buf, 512, 99);
    assert_eq!(write.op, OpCode::Write);
    assert_eq!(write.offset, 512);

    let sendfile = SubmissionEntry::sendfile(5, 6, 0, 4096, 7);
    assert_eq!(sendfile.op, OpCode::SendFile);
    assert_eq!(sendfile.fd, 5); // socket
    assert_eq!(sendfile.buf_addr, 6); // file_fd
}

#[test]
fn config_cloning() {
    let cfg = PipelineConfig::default();
    let cfg2 = cfg.clone();
    assert_eq!(cfg.entries, cfg2.entries);
}

#[test]
fn kernel_version_detection() {
    // On Linux, we should detect a kernel version.
    if cfg!(target_os = "linux") {
        let v = nawa_uring::kernel_version();
        assert!(v.is_some(), "kernel version should be detectable on Linux");
        let (major, _minor) = v.unwrap();
        assert!(major >= 2);
    }
}

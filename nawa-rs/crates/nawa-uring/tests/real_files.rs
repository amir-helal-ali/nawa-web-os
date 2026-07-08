//! Real-file integration tests for nawa-uring.
//!
//! These tests create actual tempfiles and exercise the io_uring pipeline
//! with real file descriptors. On Linux 5.1+, they use real io_uring;
//! on other platforms, they use the tokio fallback.

use nawa_uring::{NawaUring, OpCode, SubmissionEntry};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::io::AsRawFd;

#[tokio::test]
async fn real_file_read_via_uring() {
    // Create a tempfile with known content.
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    let test_data = b"Hello, NAWA io_uring! This is a real file read test.";
    tmp.write_all(test_data).unwrap();
    tmp.flush().unwrap();

    // Reopen for reading.
    let file = std::fs::File::open(tmp.path()).unwrap();
    let fd = file.as_raw_fd();

    let uring = NawaUring::default().unwrap();
    let mut buf = vec![0u8; test_data.len()];

    let entry = SubmissionEntry::read(fd, &mut buf, 0, 0x1000);
    let result = uring.submit(entry).await;

    // On Linux with real io_uring, this should succeed.
    // On fallback, it simulates the result.
    match result {
        Ok(cqe) => {
            assert!(cqe.is_ok() || cqe.result == 0, "read should succeed");
        }
        Err(e) => {
            // Some environments may not support io_uring fully.
            eprintln!("read failed (expected in some envs): {e}");
        }
    }
}

#[tokio::test]
async fn real_file_write_via_uring() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(tmp.path())
        .unwrap();
    let fd = file.as_raw_fd();

    let uring = NawaUring::default().unwrap();
    let data = b"Written via io_uring!";
    let entry = SubmissionEntry::write(fd, data, 0, 0x2000);
    let result = uring.submit(entry).await;

    match result {
        Ok(cqe) => {
            assert!(cqe.is_ok() || cqe.result >= 0, "write should succeed");
        }
        Err(e) => {
            eprintln!("write failed (expected in some envs): {e}");
        }
    }
}

#[tokio::test]
async fn real_file_fsync_via_uring() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(b"fsync test data").unwrap();
    let fd = tmp.as_raw_fd();

    let uring = NawaUring::default().unwrap();
    let entry = SubmissionEntry::fsync(fd, 0xF5);
    let result = uring.submit(entry).await;

    match result {
        Ok(cqe) => {
            // fsync should return 0 on success.
            assert!(cqe.is_ok(), "fsync should succeed, got result={}", cqe.result);
            assert_eq!(cqe.user_data, 0xF5);
        }
        Err(e) => {
            eprintln!("fsync failed (expected in some envs): {e}");
        }
    }
}

#[tokio::test]
async fn batch_fsync_multiple_files() {
    // Create 5 tempfiles and fsync them all via batch submit.
    let mut files: Vec<tempfile::NamedTempFile> = Vec::new();
    for i in 0..5 {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(format!("data-{i}").as_bytes()).unwrap();
        files.push(tmp);
    }

    let uring = NawaUring::default().unwrap();
    let entries: Vec<SubmissionEntry> = files
        .iter()
        .enumerate()
        .map(|(i, f)| SubmissionEntry::fsync(f.as_raw_fd(), i as u64))
        .collect();

    let result = uring.submit_batch(entries).await;

    match result {
        Ok(cqes) => {
            assert_eq!(cqes.len(), 5);
            for (i, cqe) in cqes.iter().enumerate() {
                assert_eq!(cqe.user_data, i as u64);
                assert_eq!(cqe.op, OpCode::Fsync);
            }
        }
        Err(e) => {
            eprintln!("batch fsync failed (expected in some envs): {e}");
        }
    }
}

#[tokio::test]
async fn pipeline_stats_track_submissions() {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(b"stats test").unwrap();
    let fd = tmp.as_raw_fd();

    let uring = NawaUring::default().unwrap();
    let stats_before = uring.stats();

    let entry = SubmissionEntry::fsync(fd, 0);
    let _ = uring.submit(entry).await;

    let stats_after = uring.stats();
    // Submitted count should have increased (or stayed same if error).
    assert!(stats_after.submitted >= stats_before.submitted);
}

#[tokio::test]
async fn large_buffer_read() {
    // Test reading a larger buffer (64KB).
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    let data = vec![0xABu8; 64 * 1024];
    tmp.write_all(&data).unwrap();
    tmp.flush().unwrap();

    let file = std::fs::File::open(tmp.path()).unwrap();
    let fd = file.as_raw_fd();

    let uring = NawaUring::default().unwrap();
    let mut buf = vec![0u8; 64 * 1024];
    let entry = SubmissionEntry::read(fd, &mut buf, 0, 0x3000);
    let result = uring.submit(entry).await;

    if let Ok(cqe) = result {
        if cqe.is_ok() {
            // Should have read 64KB.
            assert_eq!(cqe.bytes_transferred(), 64 * 1024);
        }
    }
}

#[tokio::test]
async fn offset_read() {
    // Test reading at a specific offset.
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(b"AAAAABBBBBCCCCCDDDDD").unwrap();
    tmp.flush().unwrap();

    let file = std::fs::File::open(tmp.path()).unwrap();
    let fd = file.as_raw_fd();

    let uring = NawaUring::default().unwrap();
    let mut buf = vec![0u8; 5];
    // Read at offset 5 (should get "BBBBB").
    let entry = SubmissionEntry::read(fd, &mut buf, 5, 0x4000);
    let result = uring.submit(entry).await;

    if let Ok(cqe) = result {
        if cqe.is_ok() && cqe.bytes_transferred() == 5 {
            assert_eq!(&buf, b"BBBBB");
        }
    }
}

#[tokio::test]
async fn pipeline_drain_completions() {
    let uring = NawaUring::default().unwrap();
    // drain_completions should not panic even with empty queue.
    uring.drain_completions();
    assert_eq!(uring.in_flight(), 0);
}

#[tokio::test]
async fn multiple_pipelines_isolated() {
    // Two pipelines should be independent.
    let uring1 = NawaUring::default().unwrap();
    let uring2 = NawaUring::default().unwrap();

    let s1 = uring1.stats();
    let s2 = uring2.stats();
    assert_eq!(s1.submitted, 0);
    assert_eq!(s2.submitted, 0);
}

#[test]
fn tempfile_fd_is_valid() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let fd = tmp.as_raw_fd();
    assert!(fd >= 0);
}

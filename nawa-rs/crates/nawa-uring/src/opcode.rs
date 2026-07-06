//! io_uring opcode definitions.
//!
//! Each opcode corresponds to a Linux kernel operation that can be
//! submitted to the io_uring ring. The kernel processes them async.

use std::os::unix::io::RawFd;

/// An io_uring operation code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpCode {
    /// Read from a file descriptor into a buffer.
    Read,
    /// Write a buffer to a file descriptor.
    Write,
    /// Send data over a socket (zero-copy with MSG_ZEROCOPY).
    Send,
    /// Receive data from a socket.
    Recv,
    /// Send a file's contents to a socket — true zero-copy (kernel → kernel).
    SendFile,
    /// Accept a connection on a listening socket.
    Accept,
    /// Close a file descriptor.
    Close,
    /// fsync a file descriptor (durability).
    Fsync,
    /// Open a file (openat).
    OpenAt,
    /// statx a file.
    Statx,
    /// Allocate fixed buffers for zero-copy ops.
    ProvideBuffers,
    /// Remove buffers previously provided.
    RemoveBuffers,
    /// Timeout — fires after N ops or duration.
    Timeout,
    /// Cancel a previously submitted op.
    AsyncCancel,
    /// Link two ops (second runs after first completes).
    LinkTimeout,
}

impl OpCode {
    /// Returns the kernel-side io_uring operation number.
    #[cfg(target_os = "linux")]
    pub fn as_kernel_op(self) -> u8 {
        use io_uring::opcode;
        match self {
            OpCode::Read => opcode::Read::CODE,
            OpCode::Write => opcode::Write::CODE,
            OpCode::Send => opcode::Send::CODE,
            OpCode::Recv => opcode::Recv::CODE,
            // Note: io-uring 0.7 doesn't expose Sendfile::CODE directly;
            // sendfile would use the splice or sendfile opcode at the syscall level.
            OpCode::SendFile => opcode::Write::CODE, // placeholder — use Write for now
            OpCode::Accept => opcode::Accept::CODE,
            OpCode::Close => opcode::Close::CODE,
            OpCode::Fsync => opcode::Fsync::CODE,
            OpCode::OpenAt => opcode::OpenAt::CODE,
            OpCode::Statx => opcode::Statx::CODE,
            OpCode::ProvideBuffers => opcode::ProvideBuffers::CODE,
            OpCode::RemoveBuffers => opcode::RemoveBuffers::CODE,
            OpCode::Timeout => opcode::Timeout::CODE,
            OpCode::AsyncCancel => opcode::AsyncCancel::CODE,
            OpCode::LinkTimeout => opcode::LinkTimeout::CODE,
        }
    }

    /// Human-readable name (for logging).
    pub fn as_str(&self) -> &'static str {
        match self {
            OpCode::Read => "READ",
            OpCode::Write => "WRITE",
            OpCode::Send => "SEND",
            OpCode::Recv => "RECV",
            OpCode::SendFile => "SENDFILE",
            OpCode::Accept => "ACCEPT",
            OpCode::Close => "CLOSE",
            OpCode::Fsync => "FSYNC",
            OpCode::OpenAt => "OPENAT",
            OpCode::Statx => "STATX",
            OpCode::ProvideBuffers => "PROVIDE_BUFFERS",
            OpCode::RemoveBuffers => "REMOVE_BUFFERS",
            OpCode::Timeout => "TIMEOUT",
            OpCode::AsyncCancel => "ASYNC_CANCEL",
            OpCode::LinkTimeout => "LINK_TIMEOUT",
        }
    }
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A submission entry — describes one I/O operation to submit.
#[derive(Debug, Clone)]
pub struct SubmissionEntry {
    /// Operation code.
    pub op: OpCode,
    /// Target file descriptor.
    pub fd: RawFd,
    /// User-supplied data — returned in the completion event.
    pub user_data: u64,
    /// Optional buffer offset (for read/write).
    pub offset: u64,
    /// Optional length (for read/write/send).
    pub len: u32,
    /// Optional buffer address (for read/write).
    pub buf_addr: u64,
    /// Optional flags (e.g., IOSQE_IO_LINK).
    pub flags: u8,
}

impl SubmissionEntry {
    /// Create a new submission entry with minimal fields.
    pub fn new(op: OpCode, fd: RawFd, user_data: u64) -> Self {
        Self {
            op,
            fd,
            user_data,
            offset: 0,
            len: 0,
            buf_addr: 0,
            flags: 0,
        }
    }

    /// Set the file offset (for read/write).
    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = offset;
        self
    }

    /// Set the length (for read/write/send).
    pub fn with_len(mut self, len: u32) -> Self {
        self.len = len;
        self
    }

    /// Set the buffer address (for read/write).
    pub fn with_buffer(mut self, buf_addr: u64) -> Self {
        self.buf_addr = buf_addr;
        self
    }

    /// Link this op to the next — the next won't start until this completes.
    pub fn linked(mut self) -> Self {
        const IOSQE_IO_LINK: u8 = 1 << 2;
        self.flags |= IOSQE_IO_LINK;
        self
    }

    /// Build a READ op.
    pub fn read(fd: RawFd, buf: &mut [u8], offset: u64, user_data: u64) -> Self {
        Self::new(OpCode::Read, fd, user_data)
            .with_offset(offset)
            .with_len(buf.len() as u32)
            .with_buffer(buf.as_mut_ptr() as u64)
    }

    /// Build a WRITE op.
    pub fn write(fd: RawFd, buf: &[u8], offset: u64, user_data: u64) -> Self {
        Self::new(OpCode::Write, fd, user_data)
            .with_offset(offset)
            .with_len(buf.len() as u32)
            .with_buffer(buf.as_ptr() as u64)
    }

    /// Build a SEND op (socket).
    pub fn send(fd: RawFd, buf: &[u8], user_data: u64) -> Self {
        Self::new(OpCode::Send, fd, user_data)
            .with_len(buf.len() as u32)
            .with_buffer(buf.as_ptr() as u64)
    }

    /// Build a RECV op (socket).
    pub fn recv(fd: RawFd, buf: &mut [u8], user_data: u64) -> Self {
        Self::new(OpCode::Recv, fd, user_data)
            .with_len(buf.len() as u32)
            .with_buffer(buf.as_mut_ptr() as u64)
    }

    /// Build a SENDFILE op — zero-copy file → socket.
    pub fn sendfile(sock_fd: RawFd, file_fd: RawFd, offset: u64, len: u32, user_data: u64) -> Self {
        // Note: for sendfile, we use `fd` for the socket and store `file_fd` in user_data high bits.
        // The kernel-side sendfile opcode expects (out_fd, in_fd, offset_ptr, count).
        // The pipeline.rs code handles the actual mapping.
        Self::new(OpCode::SendFile, sock_fd, user_data)
            .with_offset(offset)
            .with_len(len)
            .with_buffer(file_fd as u64) // reuse buf_addr for file_fd
    }

    /// Build a FSYNC op (durability).
    pub fn fsync(fd: RawFd, user_data: u64) -> Self {
        Self::new(OpCode::Fsync, fd, user_data)
    }

    /// Build an ACCEPT op (server socket).
    pub fn accept(fd: RawFd, user_data: u64) -> Self {
        Self::new(OpCode::Accept, fd, user_data)
    }

    /// Build a CLOSE op.
    pub fn close(fd: RawFd, user_data: u64) -> Self {
        Self::new(OpCode::Close, fd, user_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_names() {
        assert_eq!(OpCode::Read.as_str(), "READ");
        assert_eq!(OpCode::SendFile.as_str(), "SENDFILE");
        assert_eq!(OpCode::Fsync.as_str(), "FSYNC");
        assert_eq!(format!("{}", OpCode::Accept), "ACCEPT");
    }

    #[test]
    fn submission_builder() {
        let entry = SubmissionEntry::new(OpCode::Read, 5, 0xDEAD)
            .with_offset(1024)
            .with_len(4096);
        assert_eq!(entry.op, OpCode::Read);
        assert_eq!(entry.fd, 5);
        assert_eq!(entry.user_data, 0xDEAD);
        assert_eq!(entry.offset, 1024);
        assert_eq!(entry.len, 4096);
    }

    #[test]
    fn submission_read_helper() {
        let mut buf = vec![0u8; 1024];
        let entry = SubmissionEntry::read(3, &mut buf, 0, 42);
        assert_eq!(entry.op, OpCode::Read);
        assert_eq!(entry.fd, 3);
        assert_eq!(entry.len, 1024);
        assert_eq!(entry.user_data, 42);
        assert!(entry.buf_addr != 0);
    }

    #[test]
    fn submission_sendfile_helper() {
        let entry = SubmissionEntry::sendfile(7, 3, 0, 4096, 99);
        assert_eq!(entry.op, OpCode::SendFile);
        assert_eq!(entry.fd, 7); // socket
        assert_eq!(entry.buf_addr, 3); // file_fd stored here
        assert_eq!(entry.len, 4096);
    }

    #[test]
    fn linked_op_sets_flag() {
        let entry = SubmissionEntry::new(OpCode::Write, 1, 0).linked();
        assert!(entry.flags & (1 << 2) != 0); // IOSQE_IO_LINK
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn kernel_op_codes_differ() {
        assert_ne!(OpCode::Read.as_kernel_op(), OpCode::Write.as_kernel_op());
        assert_ne!(OpCode::Send.as_kernel_op(), OpCode::Recv.as_kernel_op());
    }
}

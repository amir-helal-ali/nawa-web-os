# nawa-kernel

The foundation layer providing zero-copy primitives.

## Modules

### io_uring (legacy)
The original io_uring simulation from nawa-kernel. Superseded by `nawa-uring` crate for real io_uring support.

### mmap
Memory-mapped file access via `memmap2`.

```rust
use nawa_kernel::MmapFile;

let mapped = MmapFile::open("data.bin")?;
let bytes = mapped.as_bytes(); // zero-copy access
```

### ring_buffer
Lock-free SPMC ring buffer for hot-path message passing.

```rust
use nawa_kernel::LockFreeRing;

let ring = LockFreeRing::<u64>::new(1024);
ring.push(42).unwrap();
assert_eq!(ring.pop(), Some(42));
```

### zero_copy
Reference-counted byte buffers via `bytes::Bytes`.

```rust
use nawa_kernel::ZeroCopyBuf;

let buf = ZeroCopyBuf::from_static(b"hello");
let buf2 = buf.clone(); // O(1) — just refcount bump
```

## Tests
- 9 unit tests covering all modules

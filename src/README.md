# Source Layout

`src` contains shared value types and explicit backend modules.

- `write.rs`: positioned write slices and validated write batches.
- `buffer.rs`: shared owned byte buffers (`Bytes`) and the internal buffer pool.
- `sync/`: `std::fs`-like synchronous backend.
- `tokio/`: `tokio::fs`-like async backend.
- `mmap.rs`: read-only memory mapping file backend.
- `uring.rs`: Linux `io_uring` file backend and ring implementation. Cursor traits and positioned methods use the ring; append mode is unsupported.

The crate root exports backend modules and shared value types, but no default file backend or backend free functions.

Read-capable file backends allocate through the internal `Bytes::allocate` path, which reuses buffers from a process-wide pool. Zero-length reads return an empty `Bytes::Vec` without touching the pool. `mmap` is not pool-backed.

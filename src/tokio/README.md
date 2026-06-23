# tokio Backend

The `tokio` module provides async `File` and `OpenOptions` with positioned I/O methods.

All operations run via `tokio::task::spawn_blocking` over an `Arc<std::fs::File>`. This avoids `tokio::fs` entirely — no per-call `try_clone` or tokio-internal bookkeeping. On Unix, positioned reads/writes use `pread`/`pwrite` through the shared `Arc`; on Windows, `positioned_handle()` clones the OS handle per-op to avoid seek races. Batch writes use bounded worker waves inside one blocking task.

`File` and `OpenOptions` are allocator-generic. Default reads use pooled buffers when `pool` is enabled; call `OpenOptions::allocator(System)` to force heap-backed reads.

This feature intentionally does not depend on Rayon.

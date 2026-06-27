# tokio Backend

The `tokio` module provides a `tokio::fs`-like `File` and `OpenOptions` with positioned I/O methods.

Regular filesystem operations use `tokio::fs` directly. Positioned reads and writes move owned buffers into blocking tasks because Tokio does not expose positioned file I/O; batch writes use bounded worker waves inside one blocking task.

Default reads allocate from the internal buffer pool.

This feature intentionally does not depend on Rayon.

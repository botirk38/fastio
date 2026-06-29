# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `sync` backend with positioned read/write for Linux, macOS, and Windows.
- `tokio` async backend using `spawn_blocking` for positioned I/O.
- `mmap` read-only memory-mapped backend.
- Linux `io-uring` backend with `read_at_batch` support.
- `WriteSlice` and `WriteSlices` for non-overlapping batch writes.
- Internal buffer pooling via `zeropool` for read-capable backends.
- Benchmark suite under `benches/` covering `sync`, `tokio`, `mmap`, and `uring`.

### Changed

- Internalized allocator configuration so buffer pooling is an implementation detail.
- Bumped MSRV to Rust 1.93.

## [0.3.0] - 2026-06-28

### Added

- Initial release of backend-qualified file I/O: `sync`, `tokio`, `mmap`, and `uring`.
- Public `Bytes`, `MmapRegion`, `WriteSlice`, and `WriteSlices` vocabulary.
- `OpenOptions` for each backend matching `std::fs` patterns.
- CI workflow covering Linux, macOS, and Windows with feature matrix testing.

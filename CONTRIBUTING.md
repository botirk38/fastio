# Contributing to fastio

Thank you for your interest in contributing. `fastio` keeps a small, backend-qualified API, so contributions that fit this shape are especially welcome.

## Getting Started

1. Fork the repository and create a feature branch.
2. Ensure you have Rust 1.93 or newer installed.
3. Run the validation commands below before submitting a PR.

## Build and Test

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo check --no-default-features --all-targets
cargo check --no-default-features --features tokio --all-targets
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --all-features --no-deps
```

On Linux, also run:

```bash
cargo check --no-default-features --features io-uring --all-targets
cargo test --no-default-features --features io-uring --all-targets
```

## Architecture Constraints

- Keep the public API small and backend-qualified. Do not add a root default `File` or `OpenOptions`.
- Operations belong on concrete backend file types, not on free functions or backend traits.
- Read-capable backends must allocate through the internal allocator; do not use `Vec` directly in read paths unless the caller explicitly chose system allocation.
- Keep `tokio` independent from Rayon. Use `spawn_blocking` with owned data for positioned I/O.
- Gate optional backends and APIs with their Cargo features (`sync`, `mmap`, `tokio`, `io-uring`).
- Keep `unsafe` limited to platform I/O and memory-mapping boundaries, with a nearby safety comment.

## Pull Request Process

1. Open a PR with a clear description of the problem and the change.
2. Ensure the full validation suite passes locally.
3. Link any related issue.
4. Update README, module READMEs, or AGENTS.md if behavior or workflow changes.
5. Wait for CI to pass before requesting review.

## Code Style

- Rust edition 2024.
- Prefer small, direct implementations over premature helpers.
- Add helpers only when they encapsulate repeated complexity, enforce an invariant, or isolate platform/unsafe code.
- Use `std::io::Error` and `std::io::Result`; do not add custom error types unless necessary.

## Reporting Bugs

Use the bug issue template and include OS, Rust version, feature flags, and minimal reproduction steps.

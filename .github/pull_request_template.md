## Summary

Briefly describe the change and the motivation behind it.

## Test Plan

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-features`
- [ ] `cargo check --no-default-features --all-targets`
- [ ] `cargo check --no-default-features --features tokio --all-targets`

## Backwards Compatibility

Does this change break existing public API or behavior? If so, explain how.

## Related Issues

Fixes #

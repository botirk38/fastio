# Security Policy

## Supported Versions

The following versions of `fastio` are currently supported with security updates:

| Version | Supported          |
| :------ | :----------------- |
| 0.3.x   | :white_check_mark: |
| < 0.3   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in `fastio`, please report it privately.

- **Email:** security@example.com (replace with the project maintainer's security contact)
- Do not open a public issue or pull request for a security vulnerability.
- Include as much detail as possible: affected versions, platform, feature flags, and a minimal reproduction if available.

We aim to acknowledge reports within 5 business days and provide a timeline for a fix within 10 business days. Once a fix is released, we will publish a security advisory and credit the reporter unless they prefer to remain anonymous.

## Security Considerations

`fastio` performs direct filesystem I/O and memory mapping. Callers should:

- Validate file paths and offsets before passing them to `fastio` APIs.
- Avoid operating on untrusted paths with elevated privileges.
- Be aware that memory-mapped regions are backed by the OS page cache and may be shared across processes.

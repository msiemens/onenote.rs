# Security Policy

## Supported Versions

The following versions of `onenote_parser` are currently supported with security updates:

| Version           | Supported |
|-------------------|-----------|
| 1.x (latest only) | ✅         |
| < 1.0             | ❌         |

## Reporting a Vulnerability

We take security issues seriously. If you discover a security vulnerability in `onenote_parser`, please report it
responsibly.

### How to Report

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via one of the following methods:

1. **GitHub Private Vulnerability Reporting**: Use GitHub's private vulnerability reporting feature to submit a report
   directly:  
   https://github.com/msiemens/onenote.rs/security/advisories/new

2. **Email**: Contact the maintainer directly at the email address listed on the GitHub profile:  
   https://github.com/msiemens

### What to Include

Please include as much of the following information as possible:

- A description of the vulnerability
- Steps to reproduce the issue (ideally with a minimal proof-of-concept)
- Affected versions
- Observed and expected behavior
- Any suggested fixes or mitigations (if available)

### What to Expect

- **Acknowledgment**: You should receive an acknowledgment within **7 days** of your report.
- **Updates**: We will keep you informed about the progress of addressing the vulnerability.
- **Resolution**: There is no fixed timeline for resolution; it will depend on severity, complexity, and maintainer
  availability.
- **Credit**: We are happy to credit security researchers who report valid vulnerabilities (unless you prefer to remain
  anonymous).

## Security Considerations

As a file parser library, `onenote_parser` processes potentially untrusted input.

### Resource Exhaustion / Denial of Service

Reports about excessive memory or CPU usage are welcome **when they can be demonstrated with realistically sized
inputs**.

Issues that require **extremely large input files** to trigger memory exhaustion (or similar resource exhaustion) are
generally considered **out of scope**, unless the same behavior can be reproduced with regular-sized files or typical
real-world OneNote content.

## Dependency Security

This project uses `cargo-deny` to audit dependencies. Security advisories for dependencies are monitored through the
RustSec Advisory Database.

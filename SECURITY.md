# Security Policy

## Supported Versions

The latest `0.1.x` release receives security fixes. Unreleased source on `main`
is not a supported distribution.

## Report A Vulnerability

Do not open a public issue or Discussion. Use GitHub private vulnerability
reporting for
[`Tinkora/developer_primitives`](https://github.com/Tinkora/developer_primitives/security/advisories/new).

Include affected versions, the platform, reproduction steps, impact, and any
suggested mitigation. Avoid accessing data that is not yours and do not publish
the report before a coordinated disclosure.

The maintainers aim to acknowledge complete reports within three business days
and provide an initial triage within seven business days. Timing for a fix and
disclosure depends on impact and release complexity.

## Security Boundary

Relevant reports include:

- Predictable UUID v4 or ULID random data.
- Incorrect UUID/ULID validation that crosses the documented strict boundary.
- Batch or input limit bypasses that cause practical denial of service.
- WebAssembly or browser behavior that sends or persists user data.
- CLI output behavior that exposes input or corrupts automation contracts.
- Vulnerable dependencies that are exploitable through this project.

The browser app has no backend, accounts, or stored user data. Identifiers are
not credentials, and guessing an identifier is not by itself a vulnerability
unless this project makes a stronger security claim.

## Design Controls

- Randomness comes from the operating system or Web Crypto and fails closed.
- Time acquisition is fallible and pre-Unix timestamps are rejected.
- Identifier input is bounded before parsing and invalid values are not echoed
  in machine error codes.
- Batch counts are checked before allocation.
- The browser app uses no telemetry, cookies, persistent storage, remote fonts,
  or application network API.
- Release workflows publish checksums, SBOMs, license evidence, and GitHub
  artifact attestations.

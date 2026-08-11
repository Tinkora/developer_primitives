# Changelog

All notable changes are documented here. This project follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and uses semantic
versioning for public releases.

## [Unreleased]

## [0.2.0] - 2026-08-12

### Added

- `timestamp_zone_core` with explicit Unix/RFC 3339 parsing, ordered IANA zone
  conversion, bundled IANA tzdb 2026c, and stable time errors.
- Local civil-time resolution that reports `UNAMBIGUOUS`, `GAP`, or `FOLD`
  without silently shifting gaps or selecting fold candidates.
- `tinkora-time` CLI with `convert`, `resolve`, and `zones` commands plus
  human-readable and schema-versioned JSON output.
- Browser Time module with explicit input kinds, bounded removable comparison
  zones, UTC summaries, gap/fold results, and explicit result copying.
- Additive time conversion, local resolution, zone search, and tzdb version
  exports in the existing WebAssembly package.

### Security

- Time text, IANA names, and comparison counts are validated before parsing or
  result allocation; zone discovery returns at most 50 names.
- Time conversion remains local and performs no browser, CLI, or core network
  access.

## [0.1.0] - 2026-08-12

### Added

- Local identifier workbench for UUID v4, UUID v7, and ULID generation and
  strict inspection.
- `tinkora-id` CLI with text and schema-versioned JSON output.
- Fallible randomness and clock boundaries with stable public error codes.
- Rust, CLI process, Node/WASM, and four-viewport browser validation.
- English-first and Chinese README/product specification entry points.

### Security

- Input length and batch limits before parsing and allocation.
- No browser persistence, telemetry, cookies, or application network requests.

[Unreleased]: https://github.com/Tinkora/developer_primitives/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Tinkora/developer_primitives/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Tinkora/developer_primitives/releases/tag/v0.1.0

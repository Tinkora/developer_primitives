# Developer Primitives Identifier Workbench Design

## Purpose

Developer Primitives starts with one narrow workflow: generate and inspect UUID v4, UUID v7, and ULID identifiers without sending data to a service. The first release serves developers interactively in a browser and gives scripts and AI agents a deterministic CLI contract.

The repository will be published as `Tinkora/developer_primitives`. Existing Rust crate names remain `uuid_factory_core`, `uuid_factory_cli`, and `uuid_factory_web` for this release so the work stays focused. A later time-zone module can add its own core crate and share the browser shell without changing the identifier API.

## User Workflows

1. Generate one identifier and copy it.
2. Generate 1 to 10,000 identifiers for fixtures or migrations and download or copy newline-delimited output.
3. Inspect an identifier to determine its kind, canonical value, UUID version and variant, or embedded millisecond timestamp.
4. Run the same operations from a local CLI and request versioned JSON for automation.

## Product Boundary

The release supports UUID v4, UUID v7, and canonical uppercase ULID. It does not provide a hosted API, database allocation service, UUID v1/v3/v5/v6/v8, custom UUID layouts, or a runnable MCP server. Static Agent schemas are documentation only and must not be described as Agent-callable.

UUID v7 and ULID are time-sortable across different millisecond timestamps. The release does not promise monotonic ordering for multiple identifiers generated within the same millisecond.

## Architecture

- `uuid_factory_core` owns generation, parsing, validation, limits, stable errors, and serializable result types.
- `uuid_factory_cli` exposes `generate` and `inspect`, newline text by default, and schema-versioned JSON with stable exit codes.
- `uuid_factory_web` exposes the core through `wasm-bindgen`; it never generates identifiers in JavaScript.
- The static browser app imports only the local WASM package. It uses no CDN, telemetry, cookies, local storage, or network API.

The core obtains random bytes through a fallible source and current time through a fallible clock boundary. Production uses OS/Web Crypto randomness and system time. Tests inject fixed bytes and timestamps to verify RFC layout, timestamp extraction, ordering boundaries, and failure mapping without probabilistic assertions.

## Public Contracts

Generation accepts `uuid-v4`, `uuid-v7`, or `ulid` and a count from 1 through 10,000. Output preserves generation order.

Inspection returns:

```json
{
  "schema_version": 1,
  "input": "01890f3e-e7c8-7cc3-98c8-4c0a1d2b3c4d",
  "canonical": "01890f3e-e7c8-7cc3-98c8-4c0a1d2b3c4d",
  "kind": "uuid",
  "version": 7,
  "variant": "RFC4122",
  "timestamp_ms": 1688177928136
}
```

ULID inspection uses kind `ulid`, no UUID variant or version, and includes its embedded timestamp. Invalid input fails with an error and is never represented as a successful object with `valid: false`.

Stable error codes are `INVALID_UUID`, `INVALID_ULID`, `INVALID_IDENTIFIER`, `BATCH_OUT_OF_RANGE`, `UNSUPPORTED_KIND`, `RANDOM_UNAVAILABLE`, `CLOCK_UNAVAILABLE`, and `SERIALIZATION_FAILED`.

## Security And Privacy

- Randomness comes from the operating system or Web Crypto and failure is returned, not replaced with weak randomness.
- Input length is capped before parsing; batch count is validated before allocation.
- Browser input stays in memory and is not persisted.
- Clipboard and download actions require an explicit user command.
- Generated identifiers are identifiers, not authentication secrets. UUID v4 randomness does not make an ID an access-control mechanism.

## Interface Design

The browser is a compact workbench, not a landing page. A top segmented control switches between Generate and Inspect. Generate presents identifier kind, count, and output; Inspect presents one labeled input and a structured result. Primary commands use familiar icons with accessible labels and tooltips. Output dimensions remain stable as results change.

The visual system uses a neutral light work surface, dark text, green only for the primary action/status, IBM Plex Sans for UI text, and JetBrains Mono for identifiers. It includes visible keyboard focus, `aria-live` status, 44-pixel touch targets, reduced-motion support, and no horizontal overflow at 375, 768, 1024, or 1440 pixels.

## Verification

- Core tests use fixed RFC-compatible bytes and timestamps, invalid inputs, exact limits, random failure, clock failure, and known UUID/ULID parsing vectors.
- CLI tests execute the built binary and validate stdout, stderr, JSON schema, and exit codes.
- WASM tests call exported functions in Node.
- Playwright runs the built application in Chromium at four widths, covers keyboard generation, copy/download state, inspection, invalid input, no horizontal overflow, accessibility names, and console errors.
- CI pins Actions by full SHA and runs fmt, Clippy, tests, MSRV, WASM, browser, docs, CodeQL, dependency policy, Pages, and Release gates.

## Release

The first public version is `v0.1.0`. GitHub Pages hosts the browser app. The Release contains Linux, macOS, and Windows CLI archives plus the web archive, `SHA256SUMS`, SPDX SBOM, license evidence, and GitHub attestations.

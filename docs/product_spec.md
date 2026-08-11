# Developer Primitives Product Specification

[中文](product_spec.zh-CN.md)

## Purpose

Developer Primitives helps developers and agents generate and inspect UUID v4,
UUID v7, and ULID identifiers locally. It provides a static browser workbench
and a scriptable CLI without introducing a service dependency.

## Supported Workflows

1. Generate one identifier, then copy it.
2. Generate 1 through 10,000 identifiers, then copy or download the ordered
   newline-delimited result.
3. Inspect a UUID or canonical uppercase ULID and view its structured metadata.
4. Run the same generation and inspection behavior through `tinkora-id`.

## Product Boundary

The first release supports UUID v4, UUID v7, and canonical uppercase ULID. It
does not support hosted APIs, persistence, UUID v1/v3/v5/v6/v8, custom UUID
layouts, monotonic same-millisecond generation, or a runnable MCP server.

The static schemas in `skills/mcp-tools.json` are machine-readable drafts for
future integrations. They do not start a process, open a transport, or make
these operations callable by an MCP client.

## Public Contract

Generation accepts `uuid-v4`, `uuid-v7`, or `ulid`, with a count from 1 through
10,000. Batch output preserves generation order.

Inspection returns this versioned shape:

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

ULID inspection omits UUID version and variant and includes its timestamp.
Invalid input fails with a stable error; it is never represented by a successful
`valid: false` response.

## Privacy And Security

- Browser operations execute locally through WebAssembly.
- The page does not use telemetry, cookies, local storage, or remote fonts.
- Random generation fails explicitly if the OS or Web Crypto source is
  unavailable; it never falls back to weak randomness.
- Identifier input is limited to 128 UTF-8 bytes before parsing.
- Batch counts are validated before allocation.
- Clipboard and file download require an explicit user action.

UUID v4 randomness is useful for identifier generation but is not an access
control system. Applications must still authorize access to every resource.

## Verification

- Rust tests cover RFC bit layouts, fixed timestamps, bounds, strict parsing,
  source failures, CLI output and exit codes, and WASM errors.
- Browser tests run each workflow at 375, 768, 1024, and 1440 pixels, including
  keyboard controls, live status, contrast, no overflow, no external requests,
  and zero runtime errors.
- Documentation checks validate UTF-8, local links, bilingual README/spec entry
  points, the draft-schema boundary, and retired repository links.

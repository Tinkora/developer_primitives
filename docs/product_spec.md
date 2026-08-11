# Developer Primitives Product Specification

[中文](product_spec.zh-CN.md)

## Purpose

Developer Primitives helps developers and agents generate and inspect UUID v4,
UUID v7, and ULID identifiers and perform reproducible IANA time conversion
locally. It provides one static browser workbench plus the scriptable
`tinkora-id` and `tinkora-time` CLIs without introducing a service dependency.

## Supported Workflows

1. Generate one identifier, then copy it.
2. Generate 1 through 10,000 identifiers, then copy or download the ordered
   newline-delimited result.
3. Inspect a UUID or canonical uppercase ULID and view its structured metadata.
4. Convert an explicitly typed instant into UTC and 1 through 8 ordered IANA
   zones.
5. Resolve a local civil date and time in one IANA zone as `UNAMBIGUOUS`, `GAP`,
   or `FOLD` without silently changing the input or selecting a fold candidate.
6. Discover bundled IANA zone names by exact lookup or bounded text filter.
7. Run identifier workflows through `tinkora-id` and time workflows through
   `tinkora-time`, using human-readable or schema-versioned JSON output.

## Product Boundary

Version `0.2.0` supports UUID v4, UUID v7, canonical uppercase ULID, explicit
instant conversion, IANA zone comparison, and local civil-time resolution. It
does not support hosted APIs, persistence, UUID v1/v3/v5/v6/v8, custom UUID
layouts, monotonic same-millisecond generation, schedulers, meeting planning,
calendar arithmetic, locale-dependent output, leap-second simulation, NTP, or
a runnable MCP server.

The static schemas in `skills/mcp-tools.json` are machine-readable drafts for
future integrations. They do not start a process, open a transport, or make
these operations callable by an MCP client.

## Architecture

- `uuid_factory_core` owns UUID/ULID generation, parsing, limits, and stable
  identifier errors.
- `timestamp_zone_core` owns explicit instant parsing, IANA lookup, bundled
  tzdb data, ordered conversion, zone discovery, local resolution, limits, and
  stable time errors.
- `uuid_factory_cli` exposes `tinkora-id`; `timestamp_zone_cli` exposes
  `tinkora-time`. Neither CLI performs network or file access for its work.
- `uuid_factory_web` remains the single WASM package. It exposes both cores to
  the static Identifiers and Time browser modules.

## Identifier Contract

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
Invalid input fails with a stable error; it is never represented by a
successful `valid: false` response.

## Time Contract

### Instant Conversion

Callers must select exactly one input kind:

- `unix-seconds`: a signed base-10 integer.
- `unix-milliseconds`: a signed base-10 integer.
- `rfc3339`: RFC 3339 with an explicit `Z` or numeric offset and at most three
  fractional-second digits.

The core does not infer units from numeric magnitude and rejects naive RFC 3339
input. Every successful conversion includes `schema_version`, canonical Unix
seconds and milliseconds, canonical UTC RFC 3339, `tzdb_version`, and one
ordered result for each requested zone. A zone result contains its canonical
name, local date/time, numeric offset, abbreviation, and DST state. Duplicate
zone names are rejected.

### Local Civil-Time Resolution

Local input must use `YYYY-MM-DDTHH:MM:SS` without an offset and must name one
IANA zone. The versioned result contains the canonical zone, local input,
bundled tzdb version, and one discriminated resolution:

- `UNAMBIGUOUS`: one candidate instant.
- `GAP`: no candidate; only the offsets immediately before and after the gap.
- `FOLD`: earlier and later candidate instants with their offsets.

The core never shifts a gap to a valid time or chooses one side of a fold.

### Database And Limits

All time surfaces use the bundled IANA tzdb 2026c. The contract accepts at most
128 UTF-8 bytes after trimming for text input, at most 64 ASCII bytes for a zone
name, and 1 through 8 comparison zones. Zone searches are case-insensitive,
sorted, and limited to 50 results.

## CLI Contract

- `tinkora-id generate` and `tinkora-id inspect` expose identifier behavior.
- `tinkora-time convert` requires exactly one explicit instant flag, repeated
  `--zone` options, and optional `--json`.
- `tinkora-time resolve` requires `--local`, `--zone`, and optional `--json`.
- `tinkora-time zones` accepts `--name` for exact lookup or `--filter` for
  bounded discovery, plus optional `--json`. With neither lookup option, it
  returns the first bounded page of sorted names.

Successful output is written to stdout. Stable error codes and concise messages
are written to stderr with exit code `1`; command-line usage errors exit `2`.
JSON results use `schema_version: 1`.

## Browser Contract

The static workbench retains the product header and provides a top-level switch
between Identifiers and Time. Time has Convert Instant and Resolve Local modes,
explicit input-kind controls, a searchable removable zone list, a primary UTC
summary, an ordered comparison table, separate gap/fold treatments, and an
explicit copy action. Browser behavior uses the same WASM contracts and bundled
tzdb as the CLI.

## Stable Errors

Identifier codes are `INVALID_UUID`, `INVALID_ULID`, `INVALID_IDENTIFIER`,
`BATCH_OUT_OF_RANGE`, `UNSUPPORTED_KIND`, `RANDOM_UNAVAILABLE`,
`CLOCK_UNAVAILABLE`, and `SERIALIZATION_FAILED`.

Time codes are `INVALID_TIMESTAMP`, `INVALID_RFC3339`,
`INVALID_LOCAL_DATETIME`, `INVALID_TIMEZONE`, `DUPLICATE_TIMEZONE`,
`TIMEZONE_LIMIT_EXCEEDED`, `INPUT_TOO_LONG`, and `SERIALIZATION_FAILED`.
Codes, result discriminants, and JSON field meanings are the compatibility
contract; human-readable messages may improve before `1.0`.

## Privacy And Security

- Browser operations execute locally through WebAssembly.
- The page does not use telemetry, cookies, local storage, a CDN, remote fonts,
  or a time-zone API.
- Random generation fails explicitly if the OS or Web Crypto source is
  unavailable; it never falls back to weak randomness.
- Identifier and time input limits are enforced before parsing or allocation.
- Clipboard and file download require an explicit user action.

UUID v4 randomness is useful for identifier generation but is not an access
control system. Applications must still authorize access to every resource.

## Verification

- Rust tests cover identifier bit layouts and parsing plus time parsing,
  published IANA transitions, exact New York gap/fold candidates, bounds, and
  stable errors.
- CLI process tests validate stdout, stderr, exit codes, ordered zones, and
  schema-versioned JSON against the core contracts.
- Node/WASM tests call identifier and time exports and verify structured errors
  and IANA tzdb 2026c.
- Browser tests run identifier and Time workflows at 375, 768, 1024, and 1440
  pixels, including keyboard controls, accessibility names, copy state, no
  overflow, no external requests, and zero runtime errors.
- Documentation checks validate UTF-8, local links, bilingual README/spec entry
  points, public time markers, the draft-schema boundary, and retired links.

# Developer Primitives Time Conversion Design

## Purpose

Developer Primitives will add a narrow time conversion module for log analysis,
API debugging, and cross-time-zone incident coordination. The module converts
explicit Unix seconds, Unix milliseconds, and RFC 3339 instants into UTC and up
to eight selected IANA time zones. It also resolves a local civil date and time
in an IANA zone without hiding daylight-saving gaps or folds.

The module extends the existing browser-local product and adds a separate
`tinkora-time` CLI. It does not change the public UUID/ULID core, WASM exports,
or `tinkora-id` CLI behavior released in `v0.1.0`.

## User Workflows

1. Convert an explicit Unix-seconds, Unix-milliseconds, or RFC 3339 value into
   one instant and compare its representation across selected IANA zones.
2. Enter a local civil date and time plus an IANA zone and determine whether it
   maps to one instant, no instant during a DST gap, or two instants during a
   DST fold.
3. Use the same contracts from `tinkora-time` with human-readable output or
   versioned JSON suitable for scripts and agents.
4. Inspect the bundled tzdb version so a result can be reproduced against the
   same rule set.

## Approaches Considered

### Bundled Jiff time-zone database

Use Jiff with its bundled tzdb and expose the bundled database version. This
provides deterministic offline behavior, first-class gap/fold inspection, and
matches the already validated time-zone implementation used by Tinkora Cron
Maker. This is the selected approach.

### `chrono-tz`

`chrono-tz` also compiles IANA rules into Rust and can represent ambiguous or
missing local times. It would be viable, but it would introduce a second
time-zone stack in the organization without a demonstrated product benefit.

### Browser `Intl`

Browser `Intl` avoids bundling tzdb, but results depend on each browser and
operating system. The tzdb version is not reliably available, CLI parity is
poor, and local-time ambiguity handling is not an explicit contract. It is not
suitable for reproducible Developer Primitives results.

## Architecture

- A new `timestamp_zone_core` crate owns parsing, IANA lookup, conversion,
  ambiguity classification, limits, serializable results, and stable errors.
- The existing `uuid_factory_web` crate remains the single WASM package. It
  adds thin time conversion exports backed by `timestamp_zone_core`; identifier
  exports stay unchanged.
- A new `timestamp_zone_cli` crate provides the `tinkora-time` binary. It uses
  the core directly and does not duplicate parsing or time-zone behavior.
- The existing static browser shell gains top-level Identifiers and Time
  modules. The Time module has Convert Instant and Resolve Local modes.
- Release archives contain both `tinkora-id` and `tinkora-time`. The Web archive
  continues to contain one static workbench and one local WASM package.

## Public Contracts

### Instant conversion input

The caller must select one of these kinds:

- `unix-seconds`: signed base-10 integer seconds.
- `unix-milliseconds`: signed base-10 integer milliseconds.
- `rfc3339`: RFC 3339 with an explicit `Z` or numeric offset.

Naive date/time strings are rejected for instant conversion. The core does not
guess seconds versus milliseconds from magnitude. RFC 3339 input with more
than three fractional-second digits is rejected rather than silently losing
precision in the millisecond contract.

### Instant conversion result

The result contains:

- schema version;
- canonical Unix seconds and milliseconds;
- canonical UTC RFC 3339 value;
- bundled tzdb version;
- one ordered entry per requested IANA zone with local date/time, numeric
  offset, abbreviation, and DST state when known.

Zone input order is preserved. Duplicate zone names are rejected so a caller
cannot mistake repeated output for separate comparisons.

### Local civil-time resolution

The caller provides `YYYY-MM-DDTHH:MM:SS` and one canonical IANA zone name.
The result is a discriminated union:

- `UNAMBIGUOUS`: one candidate instant;
- `GAP`: no candidate and the adjacent valid offsets;
- `FOLD`: earlier and later candidate instants with their offsets.

The core never silently shifts a gap or chooses one side of a fold. A caller
that needs one fold candidate must explicitly select `earlier` or `later`
outside the resolution operation.

### Limits

- Text input: at most 128 UTF-8 bytes after trimming.
- Zone name: at most 64 ASCII bytes.
- Comparison zones: 1 through 8.
- Numeric input must fit the Jiff timestamp range and convert safely between
  seconds and milliseconds.

### Stable errors

The time module adds these machine-readable codes without changing identifier
codes:

- `INVALID_TIMESTAMP`
- `INVALID_RFC3339`
- `INVALID_LOCAL_DATETIME`
- `INVALID_TIMEZONE`
- `DUPLICATE_TIMEZONE`
- `TIMEZONE_LIMIT_EXCEEDED`
- `INPUT_TOO_LONG`
- `SERIALIZATION_FAILED`

Human-readable messages may improve between pre-`1.0` releases; error codes,
result discriminants, and JSON field meanings are the compatibility contract.

## CLI

`tinkora-time convert` accepts exactly one explicit input flag, repeated
`--zone` options, and optional `--json`. `tinkora-time resolve` accepts one
`--local` value, one `--zone`, and optional `--json`. `tinkora-time zones`
supports exact-name lookup and a bounded text filter for discovery.

Successful JSON is written to stdout. Stable error codes and a concise message
are written to stderr with a nonzero exit code. The CLI performs no network or
file access.

## Browser Interface

The workbench retains its current product header and introduces a compact
top-level module switch. Time conversion uses explicit input-kind controls, a
searchable IANA zone selector, a bounded selected-zone list, a primary result,
and a scan-friendly comparison table. Local resolution clearly distinguishes
valid, gap, and fold states and shows both fold candidates.

All computation stays in the local WASM module. There is no telemetry, cookie,
storage, CDN, or time-zone API. Clipboard actions remain explicit user actions.
Frontend changes must follow `ui-ux-pro-max` and pass browser verification at
375, 768, 1024, and 1440 pixels.

## Verification

- Unit tests start from published transition instants and verify winter/summer
  offsets for New York, London, Sydney, Shanghai, and Kolkata.
- Gap and fold tests use `America/New_York` 2026 transitions and assert exact
  candidate Unix values and offsets.
- Parser tests cover negative epochs, millisecond precision, offset-bearing
  RFC 3339, overflow, overlong input, unknown zones, duplicates, and zone
  limits.
- CLI integration tests execute `tinkora-time`, validate stdout/stderr and exit
  codes, and compare JSON with direct core results.
- WASM tests call each time export in Node and verify structured errors.
- Browser tests cover both time modes, keyboard operation, selected-zone
  limits, gap/fold rendering, copy state, accessibility names, console output,
  overflow, and the existing identifier workflows.

## Release Boundary

The module is targeted for `v0.2.0` because it adds a new public core, CLI, WASM
surface, Web workflow, and release artifact contract. It does not add a hosted
API, scheduler, meeting planner, calendar arithmetic, locale-dependent output,
leap-second simulation, NTP, or a runnable MCP server.

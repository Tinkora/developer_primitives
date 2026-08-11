# Repository Guide for AI Agents

## Project Overview

Developer Primitives is a local-first identifier and time conversion workbench
powered by Rust WASM. It generates UUID v4, UUID v7, and canonical uppercase
ULID identifiers, converts explicit instants across IANA time zones, and
resolves local civil times without hiding daylight-saving gaps or folds. The
browser app has zero server dependencies. `tinkora-id` exposes identifier
contracts and `tinkora-time` exposes time contracts to scripts and agents.

## Architecture

```text
uuid_factory/
├── crates/
│   ├── timestamp_zone_core/       # Explicit time parsing, bundled IANA conversion, DST resolution
│   ├── timestamp_zone_cli/        # CLI binary: tinkora-time
│   ├── uuid_factory_core/         # Generation, validation, parsing core
│   ├── uuid_factory_cli/          # CLI binary: tinkora-id
│   └── uuid_factory_web/          # Shared WASM bridge + static workbench
├── docs/                           # English and Chinese product specifications
├── scripts/                        # Local documentation and release checks
├── skills/                         # Documentation-only draft schemas
└── index.html                      # Local redirect to the workbench
```

## Key Files for AI Context

| File | Purpose |
|------|---------|
| `crates/uuid_factory_core/src/generate.rs` | UUID v4/v7 and ULID generation, batch generation |
| `crates/uuid_factory_core/src/validate.rs` | UUID/ULID parsing, validation, timestamp extraction |
| `crates/uuid_factory_core/src/error.rs` | CoreError enum with stable machine codes |
| `crates/uuid_factory_cli/src/main.rs` | `tinkora-id` CLI commands |
| `crates/timestamp_zone_core/src/instant.rs` | Explicit Unix/RFC 3339 parsing and canonical UTC values |
| `crates/timestamp_zone_core/src/zone.rs` | Bundled IANA conversion, discovery, and gap/fold resolution |
| `crates/timestamp_zone_core/src/error.rs` | TimeError enum with stable machine codes |
| `crates/timestamp_zone_cli/src/main.rs` | `tinkora-time` convert, resolve, and zones commands |
| `crates/uuid_factory_web/src/lib.rs` | Additive identifier and time WASM bindings (7 JS exports) |
| `crates/uuid_factory_web/static/index.html` | Local Identifiers and Time browser workbench |
| `skills/uuid_factory.md` | CLI agent reference |
| `skills/mcp-tools.json` | Non-runnable draft schemas |

## Build & Test Commands

```bash
# Run all tests
cargo test --workspace --locked

# Format check
cargo fmt --all -- --check

# Lint (strict)
cargo clippy --workspace --all-targets --locked -- -D warnings

# WASM compilation check
cargo check -p uuid_factory_web --target wasm32-unknown-unknown --locked

# Build Web WASM for deployment
wasm-pack build --target web crates/uuid_factory_web -- --locked

# Run Node/WASM tests
wasm-pack test --node crates/uuid_factory_web --locked

# Run four-viewport browser tests
cd crates/uuid_factory_web && npm run test:browser

# Check public documentation contracts
ruby scripts/check_docs.rb
```

## Design Principles

1. **Browser-first**: Identifier and time operations run in-browser via WASM with no server calls
2. **Reproducible time rules**: Every time surface uses the bundled IANA tzdb 2026c instead of host rules
3. **Zero runtime services**: The WASM module is self-contained and the HTML UI has no external JavaScript dependencies
4. **Bounded input**: Identifier batches, text, zone names, zone comparisons, and zone searches have explicit limits
5. **Explicit ambiguity**: Local time resolution reports `UNAMBIGUOUS`, `GAP`, or `FOLD` without silent adjustment
6. **Strict validation**: Identifier inspection and time parsing return stable errors instead of partial success
7. **Explicit output actions**: Clipboard and download operations require a user action

## Supported ID Types

| Type | Format | Length | Use Case |
|------|--------|--------|----------|
| UUID v4 | 8-4-4-4-12 hex (lowercase) | 36 chars | Random, unguessable IDs |
| UUID v7 | 8-4-4-4-12 hex (lowercase) | 36 chars | Time-ordered, sortable IDs |
| ULID | Crockford Base32 | 26 chars | URL-safe, time-ordered, shorter |

## Supported Time Operations

| Operation | Input | Result |
|------|---------|--------|
| Convert instant | Explicit Unix seconds, Unix milliseconds, or offset-bearing RFC 3339 plus 1-8 IANA zones | Canonical UTC and ordered zoned representations |
| Resolve local | `YYYY-MM-DDTHH:MM:SS` plus one IANA zone | `UNAMBIGUOUS`, `GAP`, or `FOLD` |
| Search zones | Exact name or bounded case-insensitive filter | Sorted names from IANA tzdb 2026c |

## Error Codes (Stable Machine-Readable)

| Code | Meaning |
|------|---------|
| `INVALID_UUID` | String is not a valid UUID |
| `INVALID_ULID` | String is not a valid ULID |
| `INVALID_IDENTIFIER` | String is neither a valid UUID nor canonical ULID |
| `BATCH_OUT_OF_RANGE` | Batch count is outside 1 through 10,000 |
| `RANDOM_UNAVAILABLE` | Secure random source is unavailable |
| `CLOCK_UNAVAILABLE` | System clock cannot provide a Unix timestamp |
| `UNSUPPORTED_KIND` | Requested identifier kind is unsupported |
| `INVALID_TIMESTAMP` | Unix timestamp input is invalid or out of range |
| `INVALID_RFC3339` | RFC 3339 input is invalid, naive, or too precise |
| `INVALID_LOCAL_DATETIME` | Local civil date and time is invalid |
| `INVALID_TIMEZONE` | IANA time-zone name is invalid |
| `DUPLICATE_TIMEZONE` | A comparison repeats a time-zone name |
| `TIMEZONE_LIMIT_EXCEEDED` | Comparison zone count is outside 1 through 8 |
| `INPUT_TOO_LONG` | Time text input exceeds its supported length |
| `SERIALIZATION_FAILED` | A public identifier or time result could not be serialized |

## WASM Exports

| Function | Signature | Returns |
|----------|-----------|---------|
| `generate` | `(kind: &str) -> Result<String, JsValue>` | One identifier |
| `batch_generate` | `(kind: &str, count: u32) -> Result<JsValue, JsValue>` | Ordered JavaScript array |
| `inspect_identifier` | `(input: &str) -> Result<JsValue, JsValue>` | `IdentifierInspection` object |
| `convert_timestamp` | `(kind: &str, input: &str, zones: JsValue) -> Result<JsValue, JsValue>` | Versioned ordered time conversion |
| `resolve_local_timestamp` | `(local_datetime: &str, zone: &str) -> Result<JsValue, JsValue>` | Explicit local-time resolution |
| `search_time_zones` | `(filter: &str) -> Result<JsValue, JsValue>` | Bounded sorted IANA names |
| `time_zone_database_version` | `() -> String` | Bundled IANA tzdb version |

## Commit Language

- Write commit subjects and bodies in English and follow Conventional Commits.
- This repository-level rule overrides any global preference for another commit-message language.

## Frontend Design Requirement

- Before creating, modifying, reviewing, or debugging any HTML page or user-facing frontend, invoke the `ui-ux-pro-max` skill.
- Run the skill's required `--design-system` search before editing, followed by relevant stack and UX searches.
- If `ui-ux-pro-max` is unavailable, stop frontend work and report the missing prerequisite.
- Verify the rendered result in a real browser at 375, 768, 1024, and 1440 pixel widths, including console, keyboard, accessibility, and overflow checks.

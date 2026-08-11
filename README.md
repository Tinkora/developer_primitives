# Developer Primitives

[中文](README.zh-CN.md) | [Product specification](docs/product_spec.md) |
[Changelog](CHANGELOG.md) | [Contributing](CONTRIBUTING.md)

Developer Primitives is a local-first browser workbench and pair of CLIs for
identifier generation, identifier inspection, and reproducible time-zone
conversion. The browser application runs Rust compiled to WebAssembly and
makes no application network requests. `tinkora-id` handles UUID and ULID
workflows; `tinkora-time` handles timestamp and IANA time-zone workflows.

## What It Does

- Generates UUID v4, UUID v7, and canonical uppercase ULIDs, individually or
  in ordered batches of 1 through 10,000.
- Strictly inspects UUID and ULID input, including UUID version, variant, and
  embedded v7/ULID timestamps when available.
- Converts explicit Unix seconds, Unix milliseconds, or RFC 3339 instants into
  UTC and 1 through 8 ordered IANA time zones.
- Resolves a local civil date and time as `UNAMBIGUOUS`, `GAP`, or `FOLD`
  without shifting a gap or choosing one side of a fold.
- Uses the bundled IANA tzdb 2026c in the Rust core, CLI, and browser so results
  do not depend on the host time-zone database.
- Returns versioned results and stable machine-readable errors through the
  Rust cores, both CLIs, and the WebAssembly bridge.

It is not an identifier allocation service, hosted API, database, scheduler,
meeting planner, or runnable MCP server. The schemas in [`skills/`](skills/)
are documentation drafts only.

## Use The Workbench

Run the static workbench locally:

```bash
cd crates/uuid_factory_web
npm ci
npm run build:wasm
python3 -m http.server 8080 --bind 127.0.0.1 --directory static
```

Open `http://127.0.0.1:8080`. The Identifiers module generates and inspects
UUIDs and ULIDs. The Time module converts explicit instants and resolves local
times against IANA rules, with a primary UTC summary and ordered comparison
zones. Generated values, entered input, results, and explicit clipboard actions
stay in the browser. The page uses no telemetry, cookies, persistent storage,
CDN, remote fonts, or time-zone API.

## Use The CLIs

Both CLIs write successful output to standard output and diagnostics to
standard error.

```bash
# Generate and inspect identifiers.
cargo run -p uuid_factory_cli --bin tinkora-id -- generate --kind uuid-v7 --count 3
cargo run -p uuid_factory_cli --bin tinkora-id -- generate --kind ulid --count 2 --json
printf '%s\n' '550e8400-e29b-41d4-a716-446655440000' \
  | cargo run -p uuid_factory_cli --bin tinkora-id -- inspect --json

# Convert one explicit instant into ordered zones.
cargo run -p timestamp_zone_cli --bin tinkora-time -- convert \
  --unix-seconds 0 --zone UTC --zone Asia/Shanghai --json

# Resolve a local civil time without hiding a DST fold.
cargo run -p timestamp_zone_cli --bin tinkora-time -- resolve \
  --local 2026-11-01T01:30:00 --zone America/New_York --json

# Discover bundled IANA names with an exact lookup or bounded filter.
cargo run -p timestamp_zone_cli --bin tinkora-time -- zones --name Asia/Shanghai
cargo run -p timestamp_zone_cli --bin tinkora-time -- zones --filter shanghai --json
```

`tinkora-id generate` accepts `uuid-v4`, `uuid-v7`, or `ulid`.
`tinkora-time convert` requires exactly one of `--unix-seconds`,
`--unix-milliseconds`, or `--rfc3339`, plus 1 through 8 repeated `--zone`
options. `tinkora-time resolve` requires one `--local` value and one `--zone`.
All successful JSON uses `schema_version: 1`. Successful commands exit `0`;
malformed command-line usage exits `2`; operational failures exit `1` and
include a stable error code.

## Identifier Semantics

| Kind | Canonical output | Timestamp | Intended fit |
| --- | --- | --- | --- |
| UUID v4 | Lowercase hyphenated UUID | None | Random identifiers |
| UUID v7 | Lowercase hyphenated UUID | Unix milliseconds | Time-sortable identifiers |
| ULID | Uppercase Crockford Base32 | Unix milliseconds | Shorter time-sortable identifiers |

UUID v7 and ULID sort by their embedded millisecond timestamp. This project
does not promise monotonic ordering for identifiers generated within the same
millisecond. Identifiers are not authentication or authorization secrets.

## Time Semantics

Instant conversion never guesses seconds versus milliseconds from magnitude.
RFC 3339 input must include `Z` or a numeric offset and may contain at most
three fractional-second digits. Zone order is preserved, duplicate names are
rejected, and every result includes canonical Unix seconds, Unix milliseconds,
UTC RFC 3339, and the bundled tzdb version.

Local resolution accepts `YYYY-MM-DDTHH:MM:SS` and one IANA zone:

- `UNAMBIGUOUS` contains one candidate instant.
- `GAP` contains no candidate and reports the adjacent valid offsets.
- `FOLD` contains explicit earlier and later candidate instants and offsets.

## Stable Errors

Identifier errors include `INVALID_UUID`, `INVALID_ULID`,
`INVALID_IDENTIFIER`, `BATCH_OUT_OF_RANGE`, `UNSUPPORTED_KIND`,
`RANDOM_UNAVAILABLE`, `CLOCK_UNAVAILABLE`, and `SERIALIZATION_FAILED`.

Time errors include `INVALID_TIMESTAMP`, `INVALID_RFC3339`,
`INVALID_LOCAL_DATETIME`, `INVALID_TIMEZONE`, `DUPLICATE_TIMEZONE`,
`TIMEZONE_LIMIT_EXCEEDED`, `INPUT_TOO_LONG`, and `SERIALIZATION_FAILED`.
Invalid input is never returned as a successful `{ valid: false }` object.

## Architecture

- `uuid_factory_core` owns identifier generation and strict inspection.
- `timestamp_zone_core` owns explicit instant parsing, bundled IANA conversion,
  zone discovery, and local gap/fold resolution.
- `uuid_factory_cli` provides `tinkora-id`; `timestamp_zone_cli` provides
  `tinkora-time`. Both call their core directly.
- `uuid_factory_web` is the single WASM package and static workbench. It exposes
  additive identifier and time bindings without a server runtime.

## Development

Rust `1.95.0`, the `wasm32-unknown-unknown` target, `wasm-pack`, Node.js 20 or
newer, and npm are required for the complete local gate.

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p uuid_factory_web --target wasm32-unknown-unknown --locked
wasm-pack test --node crates/uuid_factory_web --locked
cd crates/uuid_factory_web && npm run test:browser
ruby scripts/check_docs.rb
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow and
[SECURITY.md](SECURITY.md) for private vulnerability reporting.

## License

MIT. See [LICENSE](LICENSE) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

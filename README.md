# Developer Primitives

[中文](README.zh-CN.md) | [Product specification](docs/product_spec.md) |
[Changelog](CHANGELOG.md) | [Contributing](CONTRIBUTING.md)

Developer Primitives is a local-first workbench and CLI for generating and
inspecting UUID v4, UUID v7, and ULID identifiers. The browser application
runs Rust compiled to WebAssembly and makes no application network requests.
The `tinkora-id` CLI exposes the same contracts for scripts and AI agents.

## What It Does

- Generates UUID v4, UUID v7, and canonical uppercase ULIDs.
- Generates ordered batches from 1 through 10,000 identifiers.
- Inspects strict UUID and ULID input, including UUID version, variant, and
  embedded v7/ULID timestamps when available.
- Returns stable machine-readable error codes through the Rust core, CLI, and
  WebAssembly bridge.

It is not an identifier allocation service, hosted API, database, or runnable
MCP server. The schemas in [`skills/`](skills/) are documentation drafts only.

## Use The Workbench

The published workbench is served by GitHub Pages after the first release. For
local use:

```bash
cd crates/uuid_factory_web
npm ci
npm run build:wasm
python3 -m http.server 8080 --bind 127.0.0.1 --directory static
```

Open `http://127.0.0.1:8080`. Generated values, inspected input, and clipboard
actions stay in the browser. The page uses no telemetry, cookies, storage, or
external fonts.

## Use The CLI

The CLI writes values to standard output and diagnostics to standard error.

```bash
cargo run -p uuid_factory_cli --bin tinkora-id -- generate --kind uuid-v7 --count 3
cargo run -p uuid_factory_cli --bin tinkora-id -- generate --kind ulid --count 2 --json
printf '%s\n' '550e8400-e29b-41d4-a716-446655440000' \
  | cargo run -p uuid_factory_cli --bin tinkora-id -- inspect --json
```

Supported generation kinds are `uuid-v4`, `uuid-v7`, and `ulid`. JSON output
uses `schema_version: 1`. Successful commands exit `0`; malformed command-line
usage exits `2`; operational failures exit `1` and include a stable error code.

## Identifier Semantics

| Kind | Canonical output | Timestamp | Intended fit |
| --- | --- | --- | --- |
| UUID v4 | Lowercase hyphenated UUID | None | Random identifiers |
| UUID v7 | Lowercase hyphenated UUID | Unix milliseconds | Time-sortable identifiers |
| ULID | Uppercase Crockford Base32 | Unix milliseconds | Shorter time-sortable identifiers |

UUID v7 and ULID sort by different millisecond timestamps. This project does
not promise monotonic ordering for identifiers generated within the same
millisecond. Identifiers are not authentication or authorization secrets.

## Stable Errors

`INVALID_UUID`, `INVALID_ULID`, `INVALID_IDENTIFIER`,
`BATCH_OUT_OF_RANGE`, `UNSUPPORTED_KIND`, `RANDOM_UNAVAILABLE`,
`CLOCK_UNAVAILABLE`, and `SERIALIZATION_FAILED` are part of the public
contract. Invalid input is not returned as a successful `{ valid: false }`
object.

## Development

Rust `1.95.0`, the `wasm32-unknown-unknown` target, `wasm-pack`, Node.js 20 or
newer, and npm are required for the complete local gate.

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
wasm-pack test --node crates/uuid_factory_web --locked
cd crates/uuid_factory_web && npm run test:browser
ruby scripts/check_docs.rb
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow and
[SECURITY.md](SECURITY.md) for private vulnerability reporting.

## License

MIT. See [LICENSE](LICENSE) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

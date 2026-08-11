# Repository Guide for AI Agents

## Project Overview

Developer Primitives is a local-first UUID/ULID workbench powered by Rust WASM.
It generates UUID v4 (random), UUID v7 (time-ordered), and canonical uppercase
ULID identifiers, with batch generation and strict inspection. The browser app
has zero server dependencies and the `tinkora-id` CLI exposes the same contract
to scripts and agents.

## Architecture

```text
uuid_factory/
├── crates/
│   ├── uuid_factory_core/         # Generation, validation, parsing core
│   ├── uuid_factory_cli/          # CLI binary: tinkora-id
│   └── uuid_factory_web/          # WASM bridge + static workbench
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
| `crates/uuid_factory_web/src/lib.rs` | WASM bindings (3 JS exports) |
| `crates/uuid_factory_cli/src/main.rs` | `tinkora-id` CLI commands |
| `crates/uuid_factory_web/src/lib.rs` | WASM bridge re-export layer |
| `crates/uuid_factory_web/static/index.html` | Local browser workbench |
| `skills/uuid_factory.md` | CLI agent reference |
| `skills/mcp-tools.json` | Non-runnable draft schemas |

## Build & Test Commands

```bash
# Run all tests
cargo test --workspace

# Format check
cargo fmt --all -- --check

# Lint (strict)
cargo clippy --workspace --all-targets -- -D warnings

# WASM compilation check
cargo check -p uuid_factory_web --target wasm32-unknown-unknown

# Build Web WASM for deployment
wasm-pack build --target web crates/uuid_factory_web
```

## Design Principles

1. **Browser-first**: All generation and validation runs in-browser via WASM — no server calls, no network latency
2. **Zero dependencies at runtime**: The WASM module is self-contained; the HTML UI has no external JS dependencies
3. **Batch-safe**: Batch generation is capped at 10,000 to prevent browser tab freezes
4. **Validation is comprehensive**: UUID inspector extracts version, variant, and v7 timestamps
5. **Copy-friendly**: Every generated ID comes with a one-click copy button

## Supported ID Types

| Type | Format | Length | Use Case |
|------|--------|--------|----------|
| UUID v4 | 8-4-4-4-12 hex (lowercase) | 36 chars | Random, unguessable IDs |
| UUID v7 | 8-4-4-4-12 hex (lowercase) | 36 chars | Time-ordered, sortable IDs |
| ULID | Crockford Base32 | 26 chars | URL-safe, time-ordered, shorter |

## Error Codes (Stable Machine-Readable)

| Code | Meaning |
|------|---------|
| `INVALID_UUID` | String is not a valid UUID |
| `INVALID_ULID` | String is not a valid ULID |
| `BATCH_OUT_OF_RANGE` | Batch count is outside 1 through 10,000 |
| `RANDOM_UNAVAILABLE` | Secure random source is unavailable |
| `CLOCK_UNAVAILABLE` | System clock cannot provide a Unix timestamp |
| `SERIALIZATION_FAILED` | A public result could not be serialized |
| `UNSUPPORTED_KIND` | Requested identifier kind is unsupported |

## WASM Exports

| Function | Signature | Returns |
|----------|-----------|---------|
| `generate` | `(kind: &str) -> Result<String, JsValue>` | One identifier |
| `batch_generate` | `(kind: &str, count: u32) -> Result<JsValue, JsValue>` | Ordered JavaScript array |
| `inspect_identifier` | `(input: &str) -> Result<JsValue, JsValue>` | `IdentifierInspection` object |

## Commit Language

- Write commit subjects and bodies in English and follow Conventional Commits.
- This repository-level rule overrides any global preference for another commit-message language.

## Frontend Design Requirement

- Before creating, modifying, reviewing, or debugging any HTML page or user-facing frontend, invoke the `ui-ux-pro-max` skill.
- Run the skill's required `--design-system` search before editing, followed by relevant stack and UX searches.
- If `ui-ux-pro-max` is unavailable, stop frontend work and report the missing prerequisite.
- Verify the rendered result in a real browser at 375, 768, 1024, and 1440 pixel widths, including console, keyboard, accessibility, and overflow checks.

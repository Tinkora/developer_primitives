# Developer Primitives Time Conversion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add reproducible IANA time conversion, explicit DST gap/fold resolution, a `tinkora-time` CLI, and a browser Time module to Developer Primitives.

**Architecture:** A new `timestamp_zone_core` crate owns all time semantics with Jiff's bundled tzdb. The existing WASM crate and a new `timestamp_zone_cli` crate are thin adapters, while the static workbench adds a Time module without changing released identifier contracts.

**Tech Stack:** Rust 1.95, Jiff 0.2.35, jiff-tzdb 0.1.8, Serde, Clap, wasm-bindgen, vanilla HTML/CSS/JavaScript, Playwright, GitHub Actions.

---

## File Map

- `crates/timestamp_zone_core/src/error.rs`: stable time error variants and codes.
- `crates/timestamp_zone_core/src/instant.rs`: explicit instant parsing and canonical UTC representation.
- `crates/timestamp_zone_core/src/zone.rs`: IANA conversion, zone discovery, and gap/fold resolution.
- `crates/timestamp_zone_core/src/lib.rs`: the public core surface and shared limits.
- `crates/timestamp_zone_cli/src/main.rs`: `tinkora-time` argument and output adapter.
- `crates/uuid_factory_web/src/lib.rs`: additive WASM exports for time operations.
- `crates/uuid_factory_web/static/app.js`: module navigation and Time workflow state.
- `crates/uuid_factory_web/static/index.html`: Time controls and result regions.
- `crates/uuid_factory_web/static/styles.css`: responsive workbench layout.
- `tests/browser/workbench.spec.js`: identifier regression and Time user-flow coverage.
- `.github/workflows/release.yml`: archives both CLI binaries per platform.
- `scripts/validate_release.rb`: validates every versioned crate.

### Task 1: Time Core Contract and Instant Parsing

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Create: `crates/timestamp_zone_core/Cargo.toml`
- Create: `crates/timestamp_zone_core/src/error.rs`
- Create: `crates/timestamp_zone_core/src/instant.rs`
- Create: `crates/timestamp_zone_core/src/lib.rs`
- Test: `crates/timestamp_zone_core/tests/instant_contract.rs`

- [ ] **Step 1: Add failing public-contract tests**

Create tests that call this exact surface:

```rust
use timestamp_zone_core::{InstantInputKind, TimeError, parse_instant};

#[test]
fn parses_explicit_seconds_without_unit_guessing() {
    let value = parse_instant(InstantInputKind::UnixSeconds, "0").unwrap();
    assert_eq!(value.unix_seconds, 0);
    assert_eq!(value.unix_milliseconds, 0);
    assert_eq!(value.utc_rfc3339, "1970-01-01T00:00:00Z");
}

#[test]
fn preserves_rfc3339_millisecond_precision() {
    let value = parse_instant(
        InstantInputKind::Rfc3339,
        "2026-11-01T01:30:00.125-04:00",
    )
    .unwrap();
    assert_eq!(value.unix_milliseconds, 1_793_511_000_125);
}

#[test]
fn rejects_naive_rfc3339_input() {
    let error = parse_instant(InstantInputKind::Rfc3339, "2026-11-01T01:30:00")
        .unwrap_err();
    assert_eq!(error, TimeError::InvalidRfc3339);
}
```

Also cover negative epoch values, integer overflow, whitespace-only input, and
129-byte input returning `INPUT_TOO_LONG`.

- [ ] **Step 2: Run the focused tests and verify RED**

Run:

```bash
cargo test -p timestamp_zone_core --test instant_contract
```

Expected: compilation fails because `timestamp_zone_core` and the public types
do not exist.

- [ ] **Step 3: Add the crate and minimal parsing implementation**

Add exact workspace dependencies:

```toml
jiff = { version = "=0.2.35", default-features = false, features = ["std", "tzdb-bundle-always"] }
jiff-tzdb = "=0.1.8"
```

Expose:

```rust
pub const MAX_TIME_INPUT_BYTES: usize = 128;

pub enum InstantInputKind { UnixSeconds, UnixMilliseconds, Rfc3339 }

pub struct ParsedInstant {
    pub schema_version: u32,
    pub unix_seconds: i64,
    pub unix_milliseconds: i64,
    pub utc_rfc3339: String,
}

pub fn parse_instant(kind: InstantInputKind, input: &str)
    -> Result<ParsedInstant, TimeError>;
```

Parse integer kinds with checked arithmetic. Parse RFC 3339 with Jiff
`Timestamp`, require an explicit `Z` or numeric offset, and produce UTC with at
most millisecond precision. Do not import the fixed-offset implementation from
the local `timestamp_zone` checkout.

- [ ] **Step 4: Verify GREEN and core hygiene**

Run:

```bash
cargo test -p timestamp_zone_core --test instant_contract
cargo fmt --all -- --check
cargo clippy -p timestamp_zone_core --all-targets -- -D warnings
```

Expected: all instant contract tests pass with no formatting or lint errors.

- [ ] **Step 5: Commit the core parsing milestone**

```bash
git add Cargo.toml Cargo.lock crates/timestamp_zone_core
git commit -m "feat: add explicit timestamp parsing core"
```

### Task 2: IANA Conversion and DST Resolution

**Files:**
- Create: `crates/timestamp_zone_core/src/zone.rs`
- Modify: `crates/timestamp_zone_core/src/lib.rs`
- Modify: `crates/timestamp_zone_core/src/error.rs`
- Test: `crates/timestamp_zone_core/tests/timezone_contract.rs`

- [ ] **Step 1: Add failing IANA and DST tests**

Cover this exact public surface:

```rust
use timestamp_zone_core::{
    InstantInputKind, LocalResolution, convert_instant, resolve_local_time,
    time_zone_database_version,
};

#[test]
fn uses_bundled_iana_2026c_rules() {
    assert_eq!(time_zone_database_version(), "2026c");
    let result = convert_instant(
        InstantInputKind::UnixSeconds,
        "1780300800",
        &["America/New_York"],
    ).unwrap();
    assert_eq!(result.zones[0].offset, "-04:00");
}

#[test]
fn reports_new_york_spring_gap_without_shifting() {
    let result = resolve_local_time("2026-03-08T02:30:00", "America/New_York")
        .unwrap();
    assert!(matches!(result.resolution, LocalResolution::Gap { .. }));
}

#[test]
fn reports_both_new_york_fall_fold_candidates() {
    let result = resolve_local_time("2026-11-01T01:30:00", "America/New_York")
        .unwrap();
    let LocalResolution::Fold { earlier, later, .. } = result.resolution else {
        panic!("expected fold");
    };
    assert_eq!(earlier.offset, "-04:00");
    assert_eq!(later.offset, "-05:00");
    assert!(earlier.unix_seconds < later.unix_seconds);
}
```

Also assert winter/summer offsets for London and Sydney, fixed offsets for
Shanghai and Kolkata, unknown-zone rejection, duplicate-zone rejection,
preserved zone order, zero zones, and more than eight zones.

- [ ] **Step 2: Run the focused tests and verify RED**

```bash
cargo test -p timestamp_zone_core --test timezone_contract
```

Expected: compilation fails because conversion and resolution are not exposed.

- [ ] **Step 3: Implement IANA result contracts**

Use `jiff::tz::TimeZone::get` and `to_ambiguous_timestamp`. Define serializable
structures with these stable fields:

```rust
pub struct TimeConversion {
    pub schema_version: u32,
    pub tzdb_version: String,
    pub instant: ParsedInstant,
    pub zones: Vec<ZonedTime>,
}

#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LocalResolution {
    Unambiguous { instant: CandidateInstant },
    Gap { before_offset: String, after_offset: String },
    Fold { earlier: CandidateInstant, later: CandidateInstant },
}
```

`ZonedTime` contains `zone`, `local_datetime`, `offset`, `abbreviation`, and
`is_dst: Option<bool>`. If Jiff cannot prove DST state, serialize `null` rather
than infer it from abbreviation or offset.

- [ ] **Step 4: Verify exact DST behavior and the full core**

```bash
cargo test -p timestamp_zone_core
cargo fmt --all -- --check
cargo clippy -p timestamp_zone_core --all-targets -- -D warnings
```

Expected: all time core tests pass, including exact gap/fold candidates.

- [ ] **Step 5: Commit the IANA milestone**

```bash
git add crates/timestamp_zone_core
git commit -m "feat: add IANA time zone conversion"
```

### Task 3: `tinkora-time` CLI

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/timestamp_zone_cli/Cargo.toml`
- Create: `crates/timestamp_zone_cli/src/main.rs`
- Test: `crates/timestamp_zone_cli/tests/cli.rs`

- [ ] **Step 1: Add failing black-box CLI tests**

Use `assert_cmd::cargo::cargo_bin_cmd!("tinkora-time")` and cover:

```text
tinkora-time convert --unix-seconds 0 --zone UTC --zone Asia/Shanghai --json
tinkora-time convert --rfc3339 2026-11-01T01:30:00-04:00 --zone America/New_York
tinkora-time resolve --local 2026-11-01T01:30:00 --zone America/New_York --json
tinkora-time zones --filter shanghai --json
```

Assert schema version, ordered zone output, `FOLD` with two candidates,
case-insensitive bounded zone search, stderr error codes, and exit code 1 for
invalid time input. Assert Clap rejects multiple instant input flags with exit
code 2.

- [ ] **Step 2: Run the CLI tests and verify RED**

```bash
cargo test -p timestamp_zone_cli --test cli
```

Expected: Cargo cannot find the new CLI package.

- [ ] **Step 3: Implement the thin Clap adapter**

Define `Convert`, `Resolve`, and `Zones` subcommands. Use a required Clap
argument group so Convert accepts exactly one of `--unix-seconds`,
`--unix-milliseconds`, or `--rfc3339`. Call only public core functions. JSON
goes to stdout; failures use `CODE: message` on stderr and exit 1.

- [ ] **Step 4: Verify CLI and workspace behavior**

```bash
cargo test -p timestamp_zone_cli
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Expected: CLI and the existing `tinkora-id` tests pass unchanged.

- [ ] **Step 5: Commit the CLI milestone**

```bash
git add Cargo.toml Cargo.lock crates/timestamp_zone_cli
git commit -m "feat: add time conversion CLI"
```

### Task 4: WASM Time Bridge

**Files:**
- Modify: `crates/uuid_factory_web/Cargo.toml`
- Modify: `crates/uuid_factory_web/src/lib.rs`
- Modify: `crates/uuid_factory_web/tests/web.rs`

- [ ] **Step 1: Add failing WASM contract tests**

Add calls for:

```rust
convert_timestamp(
    "unix-seconds",
    "0",
    serde_wasm_bindgen::to_value(&vec!["UTC"]).unwrap(),
)
resolve_local_timestamp("2026-03-08T02:30:00", "America/New_York")
search_time_zones("shanghai")
time_zone_database_version()
```

Assert serialized fields, `GAP`, exact stable error objects, and the `2026c`
database version. Keep all six identifier WASM tests unchanged.

- [ ] **Step 2: Run WASM tests and verify RED**

```bash
wasm-pack test --node crates/uuid_factory_web
```

Expected: compilation fails because the four time exports are absent.

- [ ] **Step 3: Add additive WASM exports**

Depend on `timestamp_zone_core`. Deserialize the zone array with
`serde_wasm_bindgen`, call the core, and serialize results. Use a time-specific
error adapter so the existing identifier `CoreError` mapping stays unchanged.

- [ ] **Step 4: Verify WASM and identifier regression tests**

```bash
cargo check -p uuid_factory_web --target wasm32-unknown-unknown --locked
wasm-pack test --node crates/uuid_factory_web
```

Expected: all existing identifier and new time WASM tests pass.

- [ ] **Step 5: Commit the WASM milestone**

```bash
git add Cargo.lock crates/uuid_factory_web
git commit -m "feat: expose time conversion to WebAssembly"
```

### Task 5: Browser Time Workbench

**Files:**
- Modify: `crates/uuid_factory_web/static/index.html`
- Modify: `crates/uuid_factory_web/static/app.js`
- Modify: `crates/uuid_factory_web/static/styles.css`
- Modify: `tests/browser/workbench.spec.js`

- [ ] **Step 1: Invoke the required frontend design skill**

Read `ui-ux-pro-max/SKILL.md`, run its `--design-system` search for a quiet,
dense developer utility, then run relevant vanilla HTML, forms, accessibility,
table, and responsive-layout searches before editing any frontend file.

- [ ] **Step 2: Add failing browser workflows**

At 375, 768, 1024, and 1440 pixels, test:

- switch between Identifiers and Time without losing the product header;
- convert Unix seconds into UTC, New York, and Shanghai in selected order;
- render an invalid IANA zone as an inline error with the stable code;
- render a New York gap with no invented instant;
- render both earlier and later candidates for a New York fold;
- enforce eight selected zones without layout shift;
- navigate and submit using the keyboard;
- preserve all identifier generation and inspection flows;
- emit no console errors, external network requests, or horizontal overflow.

Run the focused tests and confirm they fail because the Time module is absent:

```bash
npm run test:browser -- --grep "time workbench" --reporter=line
```

- [ ] **Step 3: Implement the smallest complete Time interface**

Add a compact top-level module switch and two modes: Convert Instant and
Resolve Local. Use explicit input-kind controls, a searchable IANA selector,
removable selected-zone rows, a primary UTC result, a comparison table, and
separate gap/fold result treatments. Reuse existing status, focus, copy, and
responsive patterns. Do not add a hero, dashboard cards, locale formatting,
browser storage, or network calls.

- [ ] **Step 4: Verify browser quality**

```bash
npm run test:browser -- --reporter=line
```

Expected: identifier and Time tests pass at all four widths with clean console,
network, accessibility, keyboard, and overflow assertions. Capture desktop and
mobile screenshots for visual inspection.

- [ ] **Step 5: Commit the Web milestone**

```bash
git add crates/uuid_factory_web/static tests/browser/workbench.spec.js
git commit -m "feat: add browser time workbench"
```

### Task 6: Public Documentation and Release Contract

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/product_spec.md`
- Modify: `docs/product_spec.zh-CN.md`
- Modify: `AGENTS.md`
- Modify: `CHANGELOG.md`
- Modify: `THIRD_PARTY_NOTICES.md`
- Modify: `.github/workflows/release.yml`
- Modify: `scripts/validate_release.rb`
- Modify: `scripts/check_workflows.rb`
- Modify: `scripts/check_docs.rb`

- [ ] **Step 1: Add failing release and documentation contracts**

Require all five crates to report version `0.2.0`, release archives to contain
both `tinkora-id` and `tinkora-time`, English README to remain default, Chinese
README to link back, Jiff and bundled tzdb notices to be present, and public
claims to describe gap/fold and the bundled tzdb version without claiming a
runnable MCP server.

```bash
ruby scripts/check_docs.rb
ruby scripts/check_workflows.rb
ruby scripts/validate_release.rb --tag v0.2.0
```

Expected: at least the version, changelog, and two-binary archive assertions
fail before documentation and workflow changes.

- [ ] **Step 2: Update bilingual product documentation**

Document the two CLI names, exact input kinds, zone limit, gap/fold behavior,
tzdb version, privacy boundary, machine-readable errors, and pre-`1.0`
compatibility. Keep English default and Chinese parity. Remove obsolete wording
that describes the product as UUID/ULID-only.

- [ ] **Step 3: Extend version and packaging contracts**

Set every crate to `0.2.0`. Build `uuid_factory_cli` and `timestamp_zone_cli` in
the same platform matrix. Package both executables in one
`tinkora-developer-primitives-0.2.0-<target>` archive, keep the Web archive, and
leave attestations, SBOM evidence, fixed Action SHAs, and release Environment
permissions unchanged.

- [ ] **Step 4: Verify documentation and release contracts**

```bash
ruby scripts/check_docs.rb
ruby scripts/check_workflows.rb
ruby scripts/validate_release.rb --tag v0.2.0
cargo deny check advisories bans licenses sources
```

Expected: all checks pass and release validation emits `Release metadata
validated for v0.2.0.`

- [ ] **Step 5: Commit the release-ready documentation milestone**

```bash
git add README.md README.zh-CN.md docs AGENTS.md CHANGELOG.md \
  THIRD_PARTY_NOTICES.md Cargo.toml Cargo.lock crates/*/Cargo.toml \
  .github/workflows/release.yml scripts
git commit -m "docs: prepare Developer Primitives v0.2.0"
```

### Task 7: Full Verification, Review, and Publication

**Files:**
- Modify only files required by verified findings.
- Update after publication: organization `.github` repository files and local
  `TOOL_MATRIX.md` / `TINKORA_ROADMAP.md`.

- [ ] **Step 1: Run the complete local gate**

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p uuid_factory_web --target wasm32-unknown-unknown --locked
wasm-pack test --node crates/uuid_factory_web
npm run test:browser -- --reporter=line
cargo deny check advisories bans licenses sources
ruby scripts/check_docs.rb
ruby scripts/check_workflows.rb
ruby scripts/check_runtime_contracts.rb
actionlint
zizmor --pedantic --offline .github/workflows
git diff --check
```

Expected: every applicable check passes; any unavailable host tool is reported
and replaced by the existing hosted workflow before publication.

- [ ] **Step 2: Review the complete diff**

Confirm the `v0.1.0` identifier APIs and CLI output remain compatible, local
time ambiguity is never resolved implicitly, public claims match tests, no
external browser request exists, and no secret or generated build output is
tracked.

- [ ] **Step 3: Push and verify hosted checks**

Push `main`, then require green Rust/WASM quality, browser contracts, CodeQL,
and supply-chain workflow runs for the exact commit.

- [ ] **Step 4: Publish and verify `v0.2.0`**

Create the annotated tag only after hosted checks pass. Verify all platform
archives, the Web archive, `SHA256SUMS`, `SBOM.spdx.json`, and `LICENSES.json`.
Download every asset, validate checksums, and run `gh attestation verify` for
each attested subject.

- [ ] **Step 5: Synchronize organization facts**

Update the bilingual organization Profile, settings policy release count,
release/access/incident records, Changelog, `TOOL_MATRIX.md`, and
`TINKORA_ROADMAP.md`. Run the full organization contracts and live read-only
audit before an English Conventional Commit to `Tinkora/.github`.

## Completion Checkpoint

- [ ] Core returns reproducible results with a visible bundled tzdb version.
- [ ] Gap and fold behavior is explicit in core, CLI, WASM, and browser output.
- [ ] Existing identifier workflows and contracts remain green.
- [ ] Both CLI binaries and the Web app ship in verified release assets.
- [ ] Pages, hosted CI, CodeQL, release evidence, and organization audit pass.
- [ ] Public English and Chinese documentation describes only implemented behavior.

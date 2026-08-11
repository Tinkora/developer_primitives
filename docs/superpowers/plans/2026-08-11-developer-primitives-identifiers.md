# Developer Primitives Identifier Workbench Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish a local-first UUID v4, UUID v7, and ULID browser workbench plus a stable automation CLI as the first Developer Primitives release.

**Architecture:** A fallible Rust core owns identifier semantics and stable errors. A thin CLI and WASM bridge consume the same serializable contracts, while a static browser workbench provides Generate and Inspect modes without network or persistence.

**Tech Stack:** Rust 1.95, `uuid`, `ulid`, `getrandom`, `clap`, `wasm-bindgen`, vanilla HTML/CSS/JavaScript, Playwright, GitHub Actions.

---

### Task 1: Reproducible Workspace And Compile Regression

**Files:**

- Modify: `Cargo.toml`
- Create: `Cargo.lock`
- Create: `rust-toolchain.toml`
- Modify: `crates/uuid_factory_core/src/validate.rs`
- Test: `crates/uuid_factory_core/src/validate.rs`

- [ ] Add a compile-time test assignment proving `ParsedUuid.version` remains `Option<u32>`.
- [ ] Run `cargo test --workspace --locked` and confirm the existing `Option<usize>` to `Option<u32>` mismatch fails.
- [ ] Convert the upstream version nibble with a checked, documented conversion at the core boundary.
- [ ] Pin the supported dependency range through `Cargo.lock`, align the repository URL to `Tinkora/developer_primitives`, and add the Rust 1.95 minimal toolchain.
- [ ] Run `cargo fmt --all -- --check` and `cargo test --workspace --locked`.

### Task 2: Fallible Identifier Generation

**Files:**

- Modify: `crates/uuid_factory_core/src/error.rs`
- Modify: `crates/uuid_factory_core/src/generate.rs`
- Modify: `crates/uuid_factory_core/src/lib.rs`
- Modify: `crates/uuid_factory_core/Cargo.toml`
- Modify: `Cargo.toml`

- [ ] Add failing tests for count `0`, count `10_001`, fixed UUID v4 version/variant bits, fixed UUID v7 timestamp/layout, fixed ULID timestamp/layout, random-source failure, and pre-epoch clock failure.
- [ ] Run the focused core tests and confirm each new behavior fails for the intended reason.
- [ ] Add `GenerationContext` boundaries for timestamp milliseconds and random bytes; production uses `SystemTime` and `getrandom::fill`.
- [ ] Build UUID v4 with `uuid::Builder::from_random_bytes`, UUID v7 with `uuid::Builder::from_unix_timestamp_millis`, and ULID with `ulid::Ulid::from_parts`.
- [ ] Return `Result` from all single and batch generation functions and expose stable `BATCH_OUT_OF_RANGE`, `RANDOM_UNAVAILABLE`, and `CLOCK_UNAVAILABLE` errors.
- [ ] Run all core tests and Clippy.

### Task 3: Unified Inspection Contract

**Files:**

- Modify: `crates/uuid_factory_core/src/validate.rs`
- Modify: `crates/uuid_factory_core/src/lib.rs`

- [ ] Add failing tests for canonical UUID, UUID v7 timestamp, canonical ULID, lowercase ULID rejection, overlong input, malformed UUID-like input, and unknown input.
- [ ] Introduce `IdentifierInspection` with schema version, input, canonical, kind, optional version, optional variant, and optional timestamp.
- [ ] Implement strict `inspect_identifier` without logging or echoing invalid values in machine error codes.
- [ ] Keep narrow UUID/ULID validators as compatibility helpers and remove ambiguous successful `{valid:false}` results from public bridges.
- [ ] Run core tests, fmt, and Clippy.

### Task 4: Automation CLI

**Files:**

- Create: `crates/uuid_factory_cli/Cargo.toml`
- Create: `crates/uuid_factory_cli/src/main.rs`
- Create: `crates/uuid_factory_cli/tests/cli.rs`
- Modify: `Cargo.toml`

- [ ] Add failing process tests for `--version`, single and batch text generation, JSON generation, stdin inspection, invalid kind, invalid count, invalid identifier, stdout/stderr separation, and exit codes.
- [ ] Implement `tinkora-id generate --kind KIND --count N [--json]` and `tinkora-id inspect [IDENTIFIER] [--json]` with stdin fallback.
- [ ] Emit JSON with `schema_version: 1`; usage errors exit `2`, operational errors exit `1`, and success exits `0`.
- [ ] Run CLI process tests and the full workspace tests.

### Task 5: WASM Contract

**Files:**

- Modify: `crates/uuid_factory_core/src/wasm.rs`
- Modify: `crates/uuid_factory_web/src/lib.rs`
- Modify: `crates/uuid_factory_web/Cargo.toml`
- Create: `crates/uuid_factory_web/tests/web.rs`

- [ ] Add failing WASM tests for generation success, invalid batch counts, inspection success, invalid inspection, and stable error objects.
- [ ] Map core results to `{code,message}` JavaScript errors without localized strings.
- [ ] Export `generate`, `batch_generate`, and `inspect_identifier`; retain existing names only when compatibility costs are trivial.
- [ ] Run `cargo check -p uuid_factory_web --target wasm32-unknown-unknown --locked` and `wasm-pack test --node crates/uuid_factory_web --locked`.

### Task 6: Browser Workbench

**Files:**

- Replace: `crates/uuid_factory_web/static/index.html`
- Create: `crates/uuid_factory_web/static/app.js`
- Create: `crates/uuid_factory_web/static/styles.css`
- Create: `crates/uuid_factory_web/package.json`
- Create: `crates/uuid_factory_web/package-lock.json`
- Create: `crates/uuid_factory_web/playwright.config.js`
- Create: `crates/uuid_factory_web/tests/browser.spec.js`
- Replace: `index.html`

- [ ] Add failing Playwright tests for Generate/Inspect mode semantics, kind/count controls, keyboard generation, result copy/download feedback, valid and invalid inspection, `aria-live`, focus visibility, no external requests, no console errors, and no horizontal overflow at 375/768/1024/1440.
- [ ] Implement the compact workbench using the approved `ui-ux-pro-max` accessibility and responsive constraints; do not add a marketing landing page.
- [ ] Use local system font fallbacks if font downloads would violate the no-network boundary; use inline Lucide-derived icons only if licensing and source are documented, otherwise use familiar text-free Unicode-free CSS/HTML symbols.
- [ ] Build WASM, serve the static directory, and run all browser tests.

### Task 7: Public Documentation And Repository Contracts

**Files:**

- Replace: `README.md`
- Create: `README.zh-CN.md`
- Modify: `CHANGELOG.md`
- Modify: `CONTRIBUTING.md`
- Modify: `SECURITY.md`
- Modify: `SUPPORT.md`
- Replace: `docs/product_spec.zh-CN.md`
- Create: `docs/product_spec.md`
- Modify: `skills/mcp-tools.json`
- Modify: `skills/uuid_factory.md`
- Create: `.markdownlint-cli2.jsonc`
- Create: `scripts/check_docs.rb`

- [ ] Write English-first and Chinese-equivalent documentation for browser, CLI, limits, privacy, exact semantics, non-goals, and verification.
- [ ] Mark static tool schemas as machine-readable drafts, not a runnable MCP server.
- [ ] Add a UTF-8/local-link/bilingual contract checker and run it with Markdown lint.
- [ ] Record `v0.1.0` release notes without claiming external adoption.

### Task 8: CI, Pages, Security, And Release

**Files:**

- Replace: `.github/workflows/test.yml`
- Create: `.github/workflows/ci.yml`
- Create: `.github/workflows/pages.yml`
- Create: `.github/workflows/supply-chain.yml`
- Create: `.github/workflows/codeql.yml`
- Create: `.github/workflows/release.yml`
- Create: `.github/dependabot.yml`
- Create: `.github/ISSUE_TEMPLATE/bug.yml`
- Create: `.github/ISSUE_TEMPLATE/feature.yml`
- Create: `.github/ISSUE_TEMPLATE/config.yml`
- Create: `.github/PULL_REQUEST_TEMPLATE.md`
- Create: `deny.toml`
- Create: `scripts/assemble_pages.rb`
- Create: `scripts/validate_release.rb`

- [ ] Add source-contract tests before workflow implementation for pinned Action SHAs, least permissions, stable artifact names, Pages assembly whitelist, release metadata, checksums, SPDX SBOM, license evidence, and attestations.
- [ ] Implement workflows using reviewed Tinkora reusable workflows and project-owned final publication jobs.
- [ ] Run actionlint, zizmor, cargo-deny, cargo-audit, documentation checks, and workflow contract tests locally.
- [ ] Run the complete release-equivalent local gate.

### Task 9: Clean Publication

**Files:**

- Update: `AGENTS.md`
- Update: all release-facing files after hosted evidence

- [ ] Review the complete diff and ensure all public commits and code comments are English.
- [ ] Create a clean English root history for the previously unpublished repository, preserving no internal setup history.
- [ ] Create `Tinkora/developer_primitives`, configure About, topics, Issues, Discussions, merge policy, security, Pages, release Environment, and minimal `main` protection.
- [ ] Push `main`, wait for all hosted checks, and fix failures through test-first commits.
- [ ] Tag `v0.1.0`, verify Release assets/checksums/attestations and Pages in a real browser.
- [ ] Register the repository in the Tinkora organization Profile, governance policy, root roadmap, and tool matrix.

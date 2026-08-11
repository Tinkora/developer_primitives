# Contributing To Developer Primitives

Contributions that solve a concrete developer or agent workflow are welcome.
Keep changes focused: this repository is a small local tool, not a hosted
platform.

## Before You Start

- Use [GitHub Discussions](https://github.com/Tinkora/developer_primitives/discussions)
  for broad proposals and questions.
- Open an issue for a reproducible bug or a narrowly scoped feature.
- Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).
- Do not add UUID variants, persistence, a hosted API, or an MCP server without
  an accepted design change.

## Development Setup

Install Rust `1.95.0` with the repository toolchain, `wasm-pack`, Node.js 20 or
newer, npm, and Chromium for Playwright.

```bash
rustup show
cargo test --workspace --locked
cd crates/uuid_factory_web && npm ci
```

## Change Workflow

1. Fork the repository and create a short-lived branch from `main`.
2. Add a failing outcome-focused test for behavior changes and bug fixes.
3. Implement the smallest coherent solution. Keep public API errors and JSON
   shapes backward compatible unless the change is explicitly breaking.
4. Write code, code comments, commit messages, and default documentation in
   English. Update the Chinese README or product specification when their
   English counterpart changes materially.
5. Run the relevant local checks and open a focused pull request.

Use English Conventional Commit subjects such as `feat: add ...`,
`fix: reject ...`, or `docs: clarify ...`. One commit should represent one
validated logical change.

## Local Verification

Run the complete gate before requesting review:

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
wasm-pack test --node crates/uuid_factory_web --locked
cd crates/uuid_factory_web && npm run test:browser
cd ../..
ruby scripts/check_docs.rb
```

Frontend changes must also be inspected in a real browser at 375, 768, 1024,
and 1440 pixels with keyboard navigation, accessibility, console, and overflow
checks. Do not introduce external runtime requests.

## Pull Requests

A pull request should explain the user problem, the chosen boundary, and the
verification performed. Link the issue when one exists. Screenshots are useful
for visual changes, but automated browser coverage is still required.

Maintainers merge only after required checks pass and review findings are
resolved. Release artifacts, tags, attestations, and Pages deployment are
created by repository workflows rather than contributor machines.

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

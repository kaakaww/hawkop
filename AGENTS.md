# Repository Guidelines

## Project Structure & Module Organization
- Core CLI entrypoint: `src/main.rs` wires subcommands and output.
- Commands live in `src/cli/` (`init`, `status`, `org`, future `app`), keeping argument parsing and handlers close.
- HTTP integration is in `src/client/` (StackHawk API, auth refresh, rate limiting).
- Configuration management sits in `src/config/`; shared errors in `src/error.rs`.
- Output formatting is split in `src/output/` (`table.rs`, `json.rs`).
- Reference architecture docs in `docs/` (e.g., `docs/plans/2025-11-09-hawkop-rust-design.md`).

## Build, Test, and Development Commands
- `cargo build` — debug build for local development.
- `cargo build --release` — optimized binary in `target/release/hawkop`.
- `cargo run -- --help` — view CLI help; run subcommands locally (`cargo run -- init`, `cargo run -- org list`).
- `cargo fmt` — format to project standards; run before committing.
- `cargo clippy -- -D warnings` — lint; treat warnings as errors.
- `cargo test` — execute unit/integration tests (add as you build features).

## Coding Style & Naming Conventions
- Rust 2021 edition; prefer idiomatic ownership, `?` for error propagation, and `anyhow::Result` for fallible CLI paths.
- Define user-facing error types in `src/error.rs` using `thiserror`; keep API errors actionable.
- Module/variable names use `snake_case`; types and enums use `PascalCase`; CLI flag/env names mirror existing ones (`--org`, `HAWKOP_ORG_ID`).
- Keep command handlers small; move API calls/config logic into `client` or `config` modules.

## Testing Guidelines
- Add unit tests alongside modules or integration tests under `tests/`; use `assert_cmd` for CLI flows and `mockito` for HTTP stubs.
- Prefer deterministic fixtures; avoid hitting real StackHawk APIs. Use `tempfile` when writing configs.
- Name tests after behavior (`init_sets_default_org`, `org_list_prints_table`); include both table and JSON output cases when relevant.

## Commit & Pull Request Guidelines
- Use concise, imperative commit subjects (`add init prompt`, `fix org list pagination`). Keep body focused on motivation and impact.
- For PRs, include: summary of changes, linked issues, key screenshots or sample CLI output when UI/formatting changes, and validation steps (`cargo fmt && cargo clippy -- -D warnings && cargo test`).
- Ensure new commands/flags are documented in `README.md` and align with existing global flag/env naming.

## Security & Configuration Tips
- Never log API keys or JWTs. Config defaults to `~/.hawkop/config.yaml`; ensure file permissions remain restrictive (`chmod 600` on Unix).
- When adding network calls, enforce rate limiting via the shared client and honor `--debug` only for non-sensitive diagnostics.

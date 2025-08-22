# Repository Guidelines

## Project Structure & Module Organization
- Root crate `miso` (Rust 2024). Library-only crate.
- Source: `src/` — `lib.rs` exposes `pub mod miso;`, main type in `src/miso.rs` (`miso::miso::Miso`). Control-byte helpers in `src/control.rs`.
- Tests: unit tests live inline under `#[cfg(test)] mod tests` in `src/miso.rs`.
- Benchmarks: Criterion in `benches/hashmap_comparison.rs`.
- Docs: `README.md`, roadmap in `TODO.md`.

## Build, Test, and Development Commands
- Build: `cargo build` — compiles the library.
- Tests: `cargo test` — runs unit tests in `src/`.
- Benches: `cargo bench` or `cargo bench --bench hashmap_comparison` — runs Criterion benches (HTML reports enabled).
- Lint: `cargo clippy --all-targets --all-features` — static checks; fix warnings where possible.
- Format: `cargo fmt` — apply standard Rust formatting.

## Coding Style & Naming Conventions
- Use rustfmt defaults (4-space indent, 100+ cols OK if readable).
- Prefer clear, explicit names: modules/files `snake_case`, types/traits `CamelCase`, functions/vars `snake_case`.
- Document unsafe: add brief “Safety:” comments for each `unsafe` block (see TODO).
- Keep API minimal; prefer `pub(crate)` over `pub` unless needed externally.

## Testing Guidelines
- Unit tests colocated with code for private access; prefer behavioral assertions via public API.
- Name tests descriptively (e.g., `test_overwrite`, `test_repeated_insert_delete`).
- Run: `cargo test`. For perf validation, compare with benches against `std::collections::HashMap`.
- Optional: add property tests and miri checks as outlined in `TODO.md`.

## Commit & Pull Request Guidelines
- Commits: follow Conventional Commits seen in history (`feat:`, `chore:`, `fix:`, `wip:`). Keep changes focused.
- PRs: include a clear description, rationale, and links to related TODO items. Add before/after benchmarks when performance-affecting. Note any public API changes.
- CI hygiene: ensure `cargo fmt`, `cargo clippy`, and `cargo test` pass locally.

## Architecture Overview
- SwissTable-style open addressing with linear probing, control-byte array, tombstones, and growth/rehash policy. Benchmarks compare `Miso` to `HashMap` for inserts/lookups/deletes.

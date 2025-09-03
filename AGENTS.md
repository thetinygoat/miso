# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Library crate (`lib.rs`) exposing `table::HashMap`.
  - Core modules: `bitmask.rs`, `control.rs`, `group.rs`, `scalar.rs`, `neon.rs`, `table.rs`.
- `benches/`: Criterion benchmarks (e.g., `hashmap_workloads.rs`).
- `docs/`: Project notes and design docs (if any).
- Tests live inline with modules (e.g., `src/table.rs` has `#[test]` blocks).

## Build, Test, and Development Commands
- `cargo build`: Compile the library (Rust edition 2024).
- `cargo test`: Run unit tests. Example: `cargo test test_resize -- --nocapture`.
- `cargo bench --bench hashmap_workloads`: Run Criterion benches; HTML at `target/criterion/<group>/report/index.html`.
- `cargo fmt`: Format code; run before committing.
- `cargo clippy --all-targets --all-features -D warnings`: Lint and deny warnings.
- `cargo doc --open`: Build and view API docs locally.

## Coding Style & Naming Conventions
- Use `rustfmt` defaults (4-space indent, trailing commas where idiomatic).
- Naming: `snake_case` for functions/modules, `UpperCamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for consts.
- Documentation: Prefer `///` doc comments for public items.
- Unsafe code: Include a brief "Safety:" note and state invariants (e.g., control-byte layout, alignment, probe guarantees).

## Testing Guidelines
- Add focused `#[test]` cases alongside the code they verify; mirror existing `test_*` naming.
- Cover: inserts/updates/deletes, growth/rehash thresholds, wraparound probing, ZST keys/values.
- Run `cargo test` locally and ensure zero warnings under Clippy.
- Property tests or fuzzing are welcome but keep deterministic seeds for repeatability.

## Commit & Pull Request Guidelines
- Use Conventional Commits: `feat:`, `fix:`, `chore:`, `refactor:`, `docs:` (matches current history).
- PRs should include: purpose/summary, notable design choices, any `unsafe` changes with rationale, and links to issues or `TODO.md` items.
- For perf-impacting changes, include before/after numbers from `cargo bench` and point to the HTML report path.
- Pre-push checklist: `cargo fmt`, `cargo clippy --all-targets --all-features -D warnings`, `cargo test`, and (if relevant) `cargo bench`.

## Platform & SIMD Notes (Optional)
- SIMD group ops auto-select NEON on `aarch64`/ARM with NEON; scalar fallback elsewhere. Validate both paths where feasible.

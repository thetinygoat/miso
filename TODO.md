# Production Readiness TODO — Miso (SwissTable)

Notes

- Each task is scoped to roughly 2–3 hours.
- Prefer small, verifiable increments; run tests and benches after each cluster of changes.
- Keep scalar baseline correct while adding SIMD; guard with feature flags.

## Recommended Sequence

1. Correctness & safety → 2) Group/scalar baseline → 3) SIMD → 4) Hashing/API → 5) Resize/tombstones → 6) Tests/benches → 7) CI/docs → 8) Release.

## 1. Correctness & Safety

- [x] Add invariant checks and asserts (power-of-two capacity, buffer lengths match, group alignment) [2h]
- [x] Refactor `insert` to loop-after-grow (remove recursion) [2h]
- [ ] Audit all `unsafe` blocks; add “Safety:” docs; ensure reads/writes are strictly guarded by control byte state [2–3h]
- [x] Handle ZST keys/values explicitly; add unit tests for ZSTs [2h]
- [x] Panic-safety audit for `grow`/rehash paths; ensure no leaks/double-drops on early returns [2–3h]

## 2. Control-Byte Layout & Grouped Probing (Scalar Baseline)

- [ ] Append sentinel tail to control bytes (len = capacity + 16; fill with 0xFF) [2h]
- [ ] Ensure 16-byte alignment for control array; add debug assertions [2h]
- [ ] Implement `Group` (scalar): load 16 control bytes; `match_fingerprint(h2)`, `match_empty()`, `match_deleted()` → u16 bitmasks [2–3h]
- [ ] Rewrite `probe_for_lookup` to iterate by groups using bitmasks, early-exit on empty [2–3h]
- [ ] Rewrite `probe_for_insert` to groups with first-tombstone tracking and early-exit on empty [2–3h]

## 3. SIMD Implementations

- [ ] AArch64 NEON group ops behind `cfg(target_arch = "aarch64")` and a `simd` feature flag [2–3h]
- [ ] Implement NEON movemask-equivalent (compare vs splat(h2), extract u16 mask) and validate vs scalar [2–3h]
- [ ] Optional: x86_64 SSE2/SSSE3 path behind feature flag with movemask [2–3h]
- [ ] Feature toggles and runtime fallback; tests ensure scalar/SIMD produce identical results [2h]

## 4. Hashing & API Generics

- [ ] Generalize hasher: `Miso<K, V, S = RandomState> where S: BuildHasher` [2–3h]
- [ ] Add constructors: `with_hasher`, `with_capacity_and_hasher` [2h]
- [ ] Add optional `fast-hash` feature (e.g., `ahash`) for benchmarks; RandomState remains default [2h]

## 5. Resizing & Tombstone Management

- [ ] Extract growth policy constants; document 7/8 load-factor rationale [2h]
- [ ] Add rehash-without-growth when tombstones exceed threshold (e.g., > size/2) [2–3h]
- [ ] Implement `reserve(additional)` and `shrink_to_fit()`; verify invariants post-move [2–3h]

## 6. API Surface & Ergonomics

- [ ] Rename `size()` → `len()`; keep `size()` as alias temporarily [2h]
- [ ] Add `is_empty()`, `contains_key(&K)` [2h]
- [ ] Add `get_mut(&K) -> Option<&mut V>`, `remove(&K)` (alias `delete`) [2h]
- [ ] Implement `clear()` freeing/reinitializing control bytes safely [2h]
- [ ] Minimal `entry()` API (Occupied/Vacant) for in-place updates/default inserts [2–3h]
- [ ] Iterators: `iter()`, `iter_mut()`, `into_iter()`, plus `keys()`/`values()` [2–3h]
- [ ] Derive/impl `Debug`, `Default`, `Clone`, `PartialEq` where K,V permit [2h]

## 7. Tests & Verification

- [ ] Duplicate insert returns old value and keeps new stored [2h]
- [ ] Tombstone reuse: delete then insert reuses slots; counts accurate [2h]
- [ ] Wraparound clustering tests (near end-of-table) for probe termination correctness [2h]
- [ ] Rehash tests: heavy delete then insert; validate contents and load factors [2–3h]
- [ ] Collision-heavy custom hasher to stress equality and tombstones [2h]
- [ ] Property tests with `proptest` against `HashMap` on random op sequences [2–3h]
- [ ] Run under Miri; fix any UB; document unsafe invariants in code [2–3h]

## 8. Benchmarks & Profiling

- [ ] Probe-length distribution benchmark; record average/worst-case probe counts [2h]
- [ ] Group-based vs scalar insert/lookup benches (sequential and random keys) [2–3h]
- [ ] Delete/tombstone-heavy workload benches; before/after rehash-without-growth [2h]
- [ ] Feature matrix benches: RandomState vs `ahash`; SIMD on/off [2h]

## 9. Tooling & CI

- [ ] CI: `cargo clippy --all-targets --all-features`, `cargo fmt -- --check`, `RUSTFLAGS='-D warnings'` [2h]
- [ ] Matrix: stable/beta/nightly; Linux/macOS; features (simd on/off, fast-hash on/off) [2–3h]
- [ ] MSRV policy (e.g., 1.74+); enforce in CI and document [2–3h]
- [ ] Optional: qemu-based aarch64 job to validate NEON builds/tests [2–3h]

## 10. Documentation & Examples

- [ ] Crate-level docs: design overview (control bytes, groups, tombstones, grow policy) [2–3h]
- [ ] “Safety” section documenting invariants per unsafe block/function [2h]
- [ ] API examples: insert/get/remove, entry, reserve/shrink [2h]
- [ ] Performance guide: load factors, reserving capacity, SIMD availability [2–3h]
- [ ] README updates with feature flags, target support, benchmark snapshot [2h]

## 11. Release & Maintenance

- [ ] CHANGELOG and semantic versioning policy [2h]
- [ ] Document default/optional features and stability [2h]
- [ ] CONTRIBUTING: dev setup, test/bench/profiling workflow [2h]
- [ ] Optional: `no_std + alloc` exploration behind feature gate [2–3h]

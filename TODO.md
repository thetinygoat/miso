# Swiss Table with AArch64 SIMD Implementation Plan

## Phase 1: Foundation & Infrastructure

### 1. Setup ARM NEON Type Definitions
- [ ] Add proper ARM NEON intrinsic imports (uint8x16_t, uint16x8_t, etc.)
- [ ] Define constants for SIMD operations (GROUP_SIZE = 16)
- [ ] Add target architecture guards (#[cfg(target_arch = "aarch64")])
- [ ] Create fallback scalar implementation for non-AArch64 targets

### 2. Create SIMD Utility Functions
- [ ] Implement `simd_match_byte()` - compare 16 bytes against target value
- [ ] Implement `simd_to_bitmask()` - convert SIMD comparison result to u16 bitmask
- [ ] Add `find_first_set_bit()` helper for bitmask processing
- [ ] Create `is_aligned_16()` helper for checking memory alignment

### 3. Refactor Metadata Layout
- [ ] Ensure metadata vector is 16-byte aligned in memory
- [ ] Add padding to capacity calculations to ensure alignment
- [ ] Update `with_capacity()` to allocate aligned metadata
- [ ] Add debug assertions for alignment verification

## Phase 2: Group-Based Probing Infrastructure

### 4. Implement Group Structure
- [ ] Create `Group` struct wrapping 16-byte metadata slice
- [ ] Add `Group::load()` method with bounds checking
- [ ] Implement `Group::match_fingerprint()` using SIMD
- [ ] Add `Group::match_empty()` and `Group::match_deleted()` methods

### 5. Create Group Iterator
- [ ] Implement `GroupIterator` that processes 16-byte chunks
- [ ] Add bounds checking and scalar fallback for partial groups
- [ ] Ensure proper wraparound handling for ring buffer behavior
- [ ] Add early termination when full table is scanned

### 6. Bitmask Processing Utilities
- [ ] Implement `BitMask` wrapper for u16 with iteration methods
- [ ] Add `first_set_bit()`, `clear_lowest_bit()` methods
- [ ] Create `for_each_set_bit()` iterator for processing matches
- [ ] Add debug formatting for bitmask visualization

## Phase 3: SIMD-Enabled Core Operations

### 7. Rewrite `probe_for_insert()` with Group-Based Approach
- [ ] Remove existing SIMD code and scalar loop separation
- [ ] Implement group-by-group scanning with early termination
- [ ] Add SIMD fingerprint matching with fallback to key comparison
- [ ] Properly handle deleted slot reuse with SIMD detection

### 8. Update `probe_for_lookup()` with SIMD
- [ ] Convert to group-based iteration
- [ ] Use SIMD for empty slot detection (early exit)
- [ ] Optimize fingerprint comparison path
- [ ] Ensure consistent behavior with insert probe

### 9. Optimize `delete()` Operation
- [ ] Use SIMD probe for deletion target location
- [ ] Ensure tombstone marking works with group-based approach
- [ ] Verify deleted slot detection in subsequent operations

## Phase 4: SIMD Bitmask Optimization (AArch64 Specific)

### 10. Implement Efficient Bitmask Conversion
- [ ] Research ARM NEON equivalent of `_mm_movemask_epi8`
- [ ] Implement using `vshrn_n_u16` and `vmovn_u16` approach
- [ ] Add alternative implementation using `vaddv` if needed
- [ ] Benchmark different approaches for best performance

### 11. Optimize Match Detection Pipeline
- [ ] Minimize register pressure in SIMD operations
- [ ] Use single SIMD load for multiple comparison types
- [ ] Pipeline fingerprint and empty/deleted checks
- [ ] Add prefetch hints for next group when beneficial

### 12. Handle Edge Cases
- [ ] Proper handling of table sizes not divisible by 16
- [ ] Boundary checking for wraparound scenarios
- [ ] Ensure alignment requirements don't break on resize
- [ ] Add safety checks for debug builds

## Phase 5: Testing & Validation

### 13. Create Comprehensive Unit Tests
- [ ] Test SIMD utilities in isolation
- [ ] Verify group-based operations match scalar behavior
- [ ] Add tests for edge cases (small tables, alignment issues)
- [ ] Create property-based tests with random operations

### 14. Add Integration Tests
- [ ] Large-scale insertion/deletion cycles
- [ ] Stress test with high load factors
- [ ] Verify tombstone management under heavy delete patterns
- [ ] Test resize behavior with SIMD-optimized operations

### 15. Performance Benchmarks
- [ ] Benchmark against std::collections::HashMap
- [ ] Compare SIMD vs scalar performance on AArch64
- [ ] Profile memory access patterns and cache behavior
- [ ] Add benchmarks for different key/value types

## Phase 6: Performance Optimization

### 16. Memory Layout Optimizations
- [ ] Experiment with interleaved vs separate metadata storage
- [ ] Optimize data structure padding and alignment
- [ ] Consider cache line alignment for metadata groups
- [ ] Profile and optimize memory access patterns

### 17. Algorithm Tuning
- [ ] Optimize load factor thresholds for resize
- [ ] Tune fingerprint extraction (currently top 7 bits)
- [ ] Experiment with different probing strategies
- [ ] Optimize tombstone cleanup frequency

### 18. Advanced SIMD Optimizations
- [ ] Investigate wider SIMD operations where applicable
- [ ] Add vectorized hash computation if beneficial
- [ ] Consider batch operations for multiple key lookups
- [ ] Explore prefetching strategies for sequential access

## Phase 7: Documentation & Polish

### 19. Code Documentation
- [ ] Add comprehensive rustdoc comments
- [ ] Document SIMD implementation details and requirements
- [ ] Create usage examples and best practices
- [ ] Add performance characteristics documentation

### 20. API Refinement
- [ ] Ensure consistent error handling
- [ ] Add Debug and Display implementations where needed
- [ ] Consider additional convenience methods
- [ ] Finalize public API surface

## Notes:
- Each task should be completable in 30-60 minutes
- Run tests after each major change
- Profile performance after each phase
- Keep scalar fallback working throughout development
- Focus on correctness first, then optimize for performance
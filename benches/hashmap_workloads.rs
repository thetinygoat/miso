use std::collections::HashMap as StdHashMap;
use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rand::{Rng, SeedableRng, rngs::StdRng};

type MisoHashMap<K, V> = miso::table::HashMap<K, V>;

trait MapLike<K, V> {
    fn with_capacity(cap: usize) -> Self;
    fn insert(&mut self, k: K, v: V) -> Option<V>;
    fn get(&self, k: &K) -> Option<&V>;
    fn delete(&mut self, k: &K) -> Option<V>;
}

impl<K: std::hash::Hash + Eq, V> MapLike<K, V> for StdHashMap<K, V> {
    #[inline]
    fn with_capacity(cap: usize) -> Self {
        StdHashMap::with_capacity(cap)
    }
    #[inline]
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        StdHashMap::insert(self, k, v)
    }
    #[inline]
    fn get(&self, k: &K) -> Option<&V> {
        StdHashMap::get(self, k)
    }
    #[inline]
    fn delete(&mut self, k: &K) -> Option<V> {
        StdHashMap::remove(self, k)
    }
}

impl<K: std::hash::Hash + Eq, V> MapLike<K, V> for MisoHashMap<K, V> {
    #[inline]
    fn with_capacity(cap: usize) -> Self {
        MisoHashMap::with_capacity(cap)
    }
    #[inline]
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        MisoHashMap::insert(self, k, v)
    }
    #[inline]
    fn get(&self, k: &K) -> Option<&V> {
        MisoHashMap::get(self, k)
    }
    #[inline]
    fn delete(&mut self, k: &K) -> Option<V> {
        MisoHashMap::delete(self, k)
    }
}

const SIZES: &[usize] = &[1_000, 10_000, 100_000];
const STRING_KEY_LEN: usize = 16;
const SEED: u64 = 0xCAFE_F00D_DEAD_BEEF;

fn gen_u64_keys(n: usize, seed: u64) -> Vec<u64> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut v: Vec<u64> = (0..n).map(|_| rng.random::<u64>()).collect();
    // Ensure uniqueness by sorting and dedup, then top-up if needed
    v.sort_unstable();
    v.dedup();
    while v.len() < n {
        let x = rng.random::<u64>();
        if v.binary_search(&x).is_err() {
            v.push(x);
        }
    }
    v
}

fn gen_string_keys(n: usize, len: usize, seed: u64) -> Vec<String> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..n)
        .map(|i| {
            // Fixed-length base36-ish string with index salt
            let mut s = format!("k{:010}_{:08x}", i, rng.random::<u32>());
            s.truncate(len);
            if s.len() < len {
                s.push_str(&"x".repeat(len - s.len()));
            }
            s
        })
        .collect()
}

fn bench_insert_u64<M: MapLike<u64, u64>>(c: &mut Criterion, impl_name: &str) {
    let mut group = c.benchmark_group("insert/u64");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(BenchmarkId::new(impl_name, n), |b| {
            b.iter_batched(
                || gen_u64_keys(n, SEED ^ (n as u64)),
                |keys| {
                    let mut map = M::with_capacity(n);
                    for (i, k) in keys.into_iter().enumerate() {
                        black_box(map.insert(k, (i as u64) ^ 0xDEADBEEF));
                    }
                    black_box(map)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_insert_string<M: MapLike<String, u64>>(c: &mut Criterion, impl_name: &str) {
    let mut group = c.benchmark_group("insert/string16");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(BenchmarkId::new(impl_name, n), |b| {
            b.iter_batched(
                || gen_string_keys(n, STRING_KEY_LEN, SEED ^ (n as u64)),
                |keys| {
                    let mut map = M::with_capacity(n);
                    for (i, k) in keys.into_iter().enumerate() {
                        black_box(map.insert(k, i as u64));
                    }
                    black_box(map)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

// (removed earlier broken variant of get_hit to avoid compilation & fairness issues)

// Fix the above by performing setup that returns both the populated map and the keys.
fn bench_get_hit_u64_fixed<M: MapLike<u64, u64>>(c: &mut Criterion, impl_name: &str) {
    let mut group = c.benchmark_group("get_hit/u64");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(BenchmarkId::new(impl_name, n), |b| {
            b.iter_batched(
                || {
                    let keys = gen_u64_keys(n, SEED ^ 0x1234 ^ (n as u64));
                    let mut map = M::with_capacity(n);
                    for (i, &k) in keys.iter().enumerate() {
                        map.insert(k, i as u64);
                    }
                    (map, keys)
                },
                |(map, keys)| {
                    let mut rng = StdRng::seed_from_u64(SEED ^ 0xBEEF ^ (n as u64));
                    let mut sum = 0u64;
                    for _ in 0..keys.len() {
                        let idx = rng.random_range(0..keys.len());
                        let k = &keys[idx];
                        if let Some(v) = map.get(k) {
                            sum ^= *v;
                        }
                    }
                    black_box(sum)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_get_miss_u64<M: MapLike<u64, u64>>(c: &mut Criterion, impl_name: &str) {
    let mut group = c.benchmark_group("get_miss/u64");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(BenchmarkId::new(impl_name, n), |b| {
            b.iter_batched(
                || {
                    let keys = gen_u64_keys(n, SEED ^ 0x9E37 ^ (n as u64));
                    let mut map = M::with_capacity(n);
                    for (i, &k) in keys.iter().enumerate() {
                        map.insert(k, i as u64);
                    }
                    // Miss keys from a disjoint space: add a large odd constant
                    let miss: Vec<u64> = keys
                        .iter()
                        .map(|&k| k.wrapping_add(0x9E3779B97F4A7C15))
                        .collect();
                    (map, miss)
                },
                |(map, miss)| {
                    let mut cnt = 0usize;
                    for k in &miss {
                        if map.get(k).is_none() {
                            cnt ^= 1;
                        }
                    }
                    black_box(cnt)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_update_u64<M: MapLike<u64, u64>>(c: &mut Criterion, impl_name: &str) {
    let mut group = c.benchmark_group("update/u64");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(BenchmarkId::new(impl_name, n), |b| {
            b.iter_batched(
                || {
                    let keys = gen_u64_keys(n, SEED ^ 0xA5A5 ^ (n as u64));
                    let mut map = M::with_capacity(n);
                    for (i, &k) in keys.iter().enumerate() {
                        map.insert(k, i as u64);
                    }
                    (map, keys)
                },
                |(mut map, keys)| {
                    for (i, &k) in keys.iter().enumerate() {
                        black_box(map.insert(k, (i as u64) ^ 0xFFFF_FFFF));
                    }
                    black_box(keys.len());
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_delete_u64<M: MapLike<u64, u64>>(c: &mut Criterion, impl_name: &str) {
    let mut group = c.benchmark_group("delete/u64");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(BenchmarkId::new(impl_name, n), |b| {
            b.iter_batched(
                || {
                    let keys = gen_u64_keys(n, SEED ^ 0xD1CE ^ (n as u64));
                    let mut map = M::with_capacity(n);
                    for (i, &k) in keys.iter().enumerate() {
                        map.insert(k, i as u64);
                    }
                    (map, keys)
                },
                |(mut map, keys)| {
                    for k in keys {
                        black_box(map.delete(&k));
                    }
                    black_box(())
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn bench_mixed_u64<M: MapLike<u64, u64>>(c: &mut Criterion, impl_name: &str) {
    let mut group = c.benchmark_group("mixed/u64");
    for &n in SIZES {
        group.throughput(Throughput::Elements((n as u64) * 10)); // approx operations
        group.bench_function(BenchmarkId::new(impl_name, n), |b| {
            b.iter_batched(
                || {
                    let keys = gen_u64_keys(n, SEED ^ 0xFEED ^ (n as u64));
                    let mut map = M::with_capacity(n * 2);
                    for (i, &k) in keys.iter().enumerate() {
                        map.insert(k, i as u64);
                    }
                    // Pre-generate operations: 80% gets (70% hit, 30% miss), 15% inserts, 5% deletes
                    let mut rng = StdRng::seed_from_u64(SEED ^ 0xFACE ^ (n as u64));
                    let miss_base: u64 = 0x9E3779B97F4A7C15;
                    let mut ops: Vec<(u8, u64)> = Vec::with_capacity(n * 10);
                    for _ in 0..(n * 10) {
                        let r = rng.random::<u8>();
                        match r % 100 {
                            0..=79 => {
                                // get
                                let rr = rng.random::<u8>();
                                if rr < 70 {
                                    // hit
                                    let idx = rng.random_range(0..n);
                                    ops.push((0, keys[idx]));
                                } else {
                                    // miss
                                    let idx = rng.random_range(0..n);
                                    ops.push((0, keys[idx].wrapping_add(miss_base)));
                                }
                            }
                            80..=94 => {
                                // insert
                                ops.push((1, rng.random::<u64>()));
                            }
                            _ => {
                                // delete
                                let idx = rng.random_range(0..n);
                                ops.push((2, keys[idx]));
                            }
                        }
                    }
                    (map, ops)
                },
                |(mut map, ops)| {
                    let mut acc = 0u64;
                    for (op, k) in ops {
                        match op {
                            0 => {
                                if let Some(v) = map.get(&k) {
                                    acc ^= *v;
                                }
                            }
                            1 => {
                                let _ = map.insert(k, 1);
                            }
                            2 => {
                                let _ = map.delete(&k);
                            }
                            _ => unreachable!(),
                        }
                    }
                    black_box(acc)
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn benches(c: &mut Criterion) {
    // Inserts
    bench_insert_u64::<StdHashMap<u64, u64>>(c, "std");
    bench_insert_u64::<MisoHashMap<u64, u64>>(c, "miso");
    bench_insert_string::<StdHashMap<String, u64>>(c, "std");
    bench_insert_string::<MisoHashMap<String, u64>>(c, "miso");

    // Gets (hit/miss)
    bench_get_hit_u64_fixed::<StdHashMap<u64, u64>>(c, "std");
    bench_get_hit_u64_fixed::<MisoHashMap<u64, u64>>(c, "miso");
    bench_get_miss_u64::<StdHashMap<u64, u64>>(c, "std");
    bench_get_miss_u64::<MisoHashMap<u64, u64>>(c, "miso");

    // Updates & deletes
    bench_update_u64::<StdHashMap<u64, u64>>(c, "std");
    bench_update_u64::<MisoHashMap<u64, u64>>(c, "miso");
    bench_delete_u64::<StdHashMap<u64, u64>>(c, "std");
    bench_delete_u64::<MisoHashMap<u64, u64>>(c, "miso");

    // Mixed
    bench_mixed_u64::<StdHashMap<u64, u64>>(c, "std");
    bench_mixed_u64::<MisoHashMap<u64, u64>>(c, "miso");
}

criterion_group!(name = hashmap_workloads; config = Criterion::default().configure_from_args(); targets = benches);
criterion_main!(hashmap_workloads);

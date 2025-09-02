use std::collections::HashMap as StdHashMap;
use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rand::{Rng, SeedableRng, rngs::StdRng};

type MisoHashMap<K, V> = miso::table::HashMap<K, V>;

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

fn bench_insert_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert/u64");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));

        // std
        group.bench_function(BenchmarkId::new("std", n), |b| {
            b.iter_batched(
                || gen_u64_keys(n, SEED ^ (n as u64)),
                |keys| {
                    let mut map = StdHashMap::<u64, u64>::with_capacity(n);
                    for (i, k) in keys.into_iter().enumerate() {
                        black_box(map.insert(k, (i as u64) ^ 0xDEADBEEF));
                    }
                    black_box(map)
                },
                BatchSize::SmallInput,
            )
        });

        // miso
        group.bench_function(BenchmarkId::new("miso", n), |b| {
            b.iter_batched(
                || gen_u64_keys(n, SEED ^ (n as u64)),
                |keys| {
                    let mut map = MisoHashMap::<u64, u64>::with_capacity(n);
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

fn bench_insert_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert/string16");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));

        // std
        group.bench_function(BenchmarkId::new("std", n), |b| {
            b.iter_batched(
                || gen_string_keys(n, STRING_KEY_LEN, SEED ^ (n as u64)),
                |keys| {
                    let mut map = StdHashMap::<String, u64>::with_capacity(n);
                    for (i, k) in keys.into_iter().enumerate() {
                        black_box(map.insert(k, i as u64));
                    }
                    black_box(map)
                },
                BatchSize::SmallInput,
            )
        });

        // miso
        group.bench_function(BenchmarkId::new("miso", n), |b| {
            b.iter_batched(
                || gen_string_keys(n, STRING_KEY_LEN, SEED ^ (n as u64)),
                |keys| {
                    let mut map = MisoHashMap::<String, u64>::with_capacity(n);
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
fn bench_get_hit_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_hit/u64");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));

        // std
        group.bench_function(BenchmarkId::new("std", n), |b| {
            b.iter_batched(
                || {
                    let keys = gen_u64_keys(n, SEED ^ 0x1234 ^ (n as u64));
                    let mut map = StdHashMap::<u64, u64>::with_capacity(n);
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

        // miso
        group.bench_function(BenchmarkId::new("miso", n), |b| {
            b.iter_batched(
                || {
                    let keys = gen_u64_keys(n, SEED ^ 0x1234 ^ (n as u64));
                    let mut map = MisoHashMap::<u64, u64>::with_capacity(n);
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

fn bench_get_miss_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_miss/u64");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));

        // std
        group.bench_function(BenchmarkId::new("std", n), |b| {
            b.iter_batched(
                || {
                    let keys = gen_u64_keys(n, SEED ^ 0x9E37 ^ (n as u64));
                    let mut map = StdHashMap::<u64, u64>::with_capacity(n);
                    for (i, &k) in keys.iter().enumerate() {
                        map.insert(k, i as u64);
                    }
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

        // miso
        group.bench_function(BenchmarkId::new("miso", n), |b| {
            b.iter_batched(
                || {
                    let keys = gen_u64_keys(n, SEED ^ 0x9E37 ^ (n as u64));
                    let mut map = MisoHashMap::<u64, u64>::with_capacity(n);
                    for (i, &k) in keys.iter().enumerate() {
                        map.insert(k, i as u64);
                    }
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

fn bench_update_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("update/u64");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));

        // std
        group.bench_function(BenchmarkId::new("std", n), |b| {
            b.iter_batched(
                || {
                    let keys = gen_u64_keys(n, SEED ^ 0xA5A5 ^ (n as u64));
                    let mut map = StdHashMap::<u64, u64>::with_capacity(n);
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

        // miso
        group.bench_function(BenchmarkId::new("miso", n), |b| {
            b.iter_batched(
                || {
                    let keys = gen_u64_keys(n, SEED ^ 0xA5A5 ^ (n as u64));
                    let mut map = MisoHashMap::<u64, u64>::with_capacity(n);
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

fn bench_delete_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("delete/u64");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));

        // std
        group.bench_function(BenchmarkId::new("std", n), |b| {
            b.iter_batched(
                || {
                    let keys = gen_u64_keys(n, SEED ^ 0xD1CE ^ (n as u64));
                    let mut map = StdHashMap::<u64, u64>::with_capacity(n);
                    for (i, &k) in keys.iter().enumerate() {
                        map.insert(k, i as u64);
                    }
                    (map, keys)
                },
                |(mut map, keys)| {
                    for k in keys {
                        black_box(map.remove(&k));
                    }
                    black_box(())
                },
                BatchSize::SmallInput,
            )
        });

        // miso
        group.bench_function(BenchmarkId::new("miso", n), |b| {
            b.iter_batched(
                || {
                    let keys = gen_u64_keys(n, SEED ^ 0xD1CE ^ (n as u64));
                    let mut map = MisoHashMap::<u64, u64>::with_capacity(n);
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

fn bench_mixed_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed/u64");
    for &n in SIZES {
        group.throughput(Throughput::Elements((n as u64) * 10));

        let setup_ops = |n: usize| {
            let keys = gen_u64_keys(n, SEED ^ 0xFEED ^ (n as u64));
            let miss_base: u64 = 0x9E3779B97F4A7C15;
            let mut rng = StdRng::seed_from_u64(SEED ^ 0xFACE ^ (n as u64));
            let mut ops: Vec<(u8, u64)> = Vec::with_capacity(n * 10);
            for _ in 0..(n * 10) {
                let r = rng.random::<u8>();
                match r % 100 {
                    0..=79 => {
                        let rr = rng.random::<u8>();
                        if rr < 70 {
                            let idx = rng.random_range(0..n);
                            ops.push((0, keys[idx]));
                        } else {
                            let idx = rng.random_range(0..n);
                            ops.push((0, keys[idx].wrapping_add(miss_base)));
                        }
                    }
                    80..=94 => ops.push((1, rng.random::<u64>())),
                    _ => {
                        let idx = rng.random_range(0..n);
                        ops.push((2, keys[idx]));
                    }
                }
            }
            (keys, ops)
        };

        // std
        group.bench_function(BenchmarkId::new("std", n), |b| {
            b.iter_batched(
                || {
                    let (keys, ops) = setup_ops(n);
                    let mut map = StdHashMap::<u64, u64>::with_capacity(n * 2);
                    for (i, &k) in keys.iter().enumerate() {
                        map.insert(k, i as u64);
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
                                let _ = map.remove(&k);
                            }
                            _ => unreachable!(),
                        }
                    }
                    black_box(acc)
                },
                BatchSize::SmallInput,
            )
        });

        // miso
        group.bench_function(BenchmarkId::new("miso", n), |b| {
            b.iter_batched(
                || {
                    let (keys, ops) = setup_ops(n);
                    let mut map = MisoHashMap::<u64, u64>::with_capacity(n * 2);
                    for (i, &k) in keys.iter().enumerate() {
                        map.insert(k, i as u64);
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
    bench_insert_u64(c);
    bench_insert_string(c);
    bench_get_hit_u64(c);
    bench_get_miss_u64(c);
    bench_update_u64(c);
    bench_delete_u64(c);
    bench_mixed_u64(c);
}

criterion_group!(name = hashmap_workloads; config = Criterion::default().configure_from_args(); targets = benches);
criterion_main!(hashmap_workloads);

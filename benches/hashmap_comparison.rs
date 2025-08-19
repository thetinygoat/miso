use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use miso::miso::Miso;
use std::collections::HashMap;
use std::hint::black_box as std_black_box;

// Test data generation
fn generate_sequential_keys(n: usize) -> Vec<u64> {
    (0..n as u64).collect()
}

fn generate_random_keys(n: usize, seed: u64) -> Vec<u64> {
    // Simple LCG for deterministic random numbers
    let mut rng = seed;
    let mut keys = Vec::with_capacity(n);
    
    for _ in 0..n {
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        keys.push(rng);
    }
    
    keys
}

// Insert benchmarks
fn bench_insert_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_sequential");
    
    for size in [100, 1000, 10000, 100000] {
        let keys = generate_sequential_keys(size);
        
        group.bench_with_input(BenchmarkId::new("miso", size), &size, |b, &size| {
            b.iter(|| {
                let mut map = Miso::new();
                for &key in &keys[..size] {
                    std_black_box(map.insert(key, key));
                }
                std_black_box(map)
            });
        });
        
        group.bench_with_input(BenchmarkId::new("std_hashmap", size), &size, |b, &size| {
            b.iter(|| {
                let mut map = HashMap::new();
                for &key in &keys[..size] {
                    std_black_box(map.insert(key, key));
                }
                std_black_box(map)
            });
        });
    }
    
    group.finish();
}

fn bench_insert_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_random");
    
    for size in [100, 1000, 10000, 100000] {
        let keys = generate_random_keys(size, 12345);
        
        group.bench_with_input(BenchmarkId::new("miso", size), &size, |b, &size| {
            b.iter(|| {
                let mut map = Miso::new();
                for &key in &keys[..size] {
                    std_black_box(map.insert(key, key));
                }
                std_black_box(map)
            });
        });
        
        group.bench_with_input(BenchmarkId::new("std_hashmap", size), &size, |b, &size| {
            b.iter(|| {
                let mut map = HashMap::new();
                for &key in &keys[..size] {
                    std_black_box(map.insert(key, key));
                }
                std_black_box(map)
            });
        });
    }
    
    group.finish();
}

fn bench_insert_presized(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_presized");
    
    for size in [100, 1000, 10000, 100000] {
        let keys = generate_random_keys(size, 12345);
        
        group.bench_with_input(BenchmarkId::new("miso", size), &size, |b, &size| {
            b.iter(|| {
                let mut map = Miso::with_capacity(size);
                for &key in &keys[..size] {
                    std_black_box(map.insert(key, key));
                }
                std_black_box(map)
            });
        });
        
        group.bench_with_input(BenchmarkId::new("std_hashmap", size), &size, |b, &size| {
            b.iter(|| {
                let mut map = HashMap::with_capacity(size);
                for &key in &keys[..size] {
                    std_black_box(map.insert(key, key));
                }
                std_black_box(map)
            });
        });
    }
    
    group.finish();
}

// Lookup benchmarks
fn bench_lookup_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_hit");
    
    for size in [100, 1000, 10000, 100000] {
        let keys = generate_random_keys(size, 12345);
        
        // Pre-populate maps
        let mut miso_map = Miso::with_capacity(size);
        let mut std_map = HashMap::with_capacity(size);
        
        for &key in &keys {
            miso_map.insert(key, key);
            std_map.insert(key, key);
        }
        
        group.bench_with_input(BenchmarkId::new("miso", size), &keys, |b, keys| {
            b.iter(|| {
                for &key in keys {
                    std_black_box(miso_map.get(&key));
                }
            });
        });
        
        group.bench_with_input(BenchmarkId::new("std_hashmap", size), &keys, |b, keys| {
            b.iter(|| {
                for &key in keys {
                    std_black_box(std_map.get(&key));
                }
            });
        });
    }
    
    group.finish();
}

fn bench_lookup_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_miss");
    
    for size in [100, 1000, 10000, 100000] {
        let keys = generate_random_keys(size, 12345);
        let miss_keys = generate_random_keys(size, 54321); // Different seed for misses
        
        // Pre-populate maps with original keys
        let mut miso_map = Miso::with_capacity(size);
        let mut std_map = HashMap::with_capacity(size);
        
        for &key in &keys {
            miso_map.insert(key, key);
            std_map.insert(key, key);
        }
        
        group.bench_with_input(BenchmarkId::new("miso", size), &miss_keys, |b, miss_keys| {
            b.iter(|| {
                for &key in miss_keys {
                    std_black_box(miso_map.get(&key));
                }
            });
        });
        
        group.bench_with_input(BenchmarkId::new("std_hashmap", size), &miss_keys, |b, miss_keys| {
            b.iter(|| {
                for &key in miss_keys {
                    std_black_box(std_map.get(&key));
                }
            });
        });
    }
    
    group.finish();
}

fn bench_lookup_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_mixed");
    
    for size in [100, 1000, 10000, 100000] {
        let keys = generate_random_keys(size, 12345);
        let miss_keys = generate_random_keys(size / 2, 54321);
        
        // Create mixed lookup keys (50% hit, 50% miss)
        let mut mixed_keys = keys[..size / 2].to_vec();
        mixed_keys.extend_from_slice(&miss_keys);
        
        // Pre-populate maps
        let mut miso_map = Miso::with_capacity(size);
        let mut std_map = HashMap::with_capacity(size);
        
        for &key in &keys {
            miso_map.insert(key, key);
            std_map.insert(key, key);
        }
        
        group.bench_with_input(BenchmarkId::new("miso", size), &mixed_keys, |b, mixed_keys| {
            b.iter(|| {
                for &key in mixed_keys {
                    std_black_box(miso_map.get(&key));
                }
            });
        });
        
        group.bench_with_input(BenchmarkId::new("std_hashmap", size), &mixed_keys, |b, mixed_keys| {
            b.iter(|| {
                for &key in mixed_keys {
                    std_black_box(std_map.get(&key));
                }
            });
        });
    }
    
    group.finish();
}

// Delete benchmarks
fn bench_delete_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("delete_all");
    
    for size in [100, 1000, 10000] { // Smaller sizes for delete since it's destructive
        let keys = generate_random_keys(size, 12345);
        
        group.bench_with_input(BenchmarkId::new("miso", size), &keys, |b, keys| {
            b.iter_batched(
                || {
                    let mut map = Miso::with_capacity(size);
                    for &key in keys {
                        map.insert(key, key);
                    }
                    map
                },
                |mut map| {
                    for &key in keys {
                        std_black_box(map.delete(&key));
                    }
                    std_black_box(map)
                },
                criterion::BatchSize::SmallInput,
            );
        });
        
        group.bench_with_input(BenchmarkId::new("std_hashmap", size), &keys, |b, keys| {
            b.iter_batched(
                || {
                    let mut map = HashMap::with_capacity(size);
                    for &key in keys {
                        map.insert(key, key);
                    }
                    map
                },
                |mut map| {
                    for &key in keys {
                        std_black_box(map.remove(&key));
                    }
                    std_black_box(map)
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    
    group.finish();
}

fn bench_delete_half(c: &mut Criterion) {
    let mut group = c.benchmark_group("delete_half");
    
    for size in [100, 1000, 10000] {
        let keys = generate_random_keys(size, 12345);
        let delete_keys = &keys[..size / 2];
        
        group.bench_with_input(BenchmarkId::new("miso", size), &(keys.clone(), delete_keys), |b, (keys, delete_keys)| {
            b.iter_batched(
                || {
                    let mut map = Miso::with_capacity(size);
                    for &key in keys {
                        map.insert(key, key);
                    }
                    map
                },
                |mut map| {
                    for &key in *delete_keys {
                        std_black_box(map.delete(&key));
                    }
                    std_black_box(map)
                },
                criterion::BatchSize::SmallInput,
            );
        });
        
        group.bench_with_input(BenchmarkId::new("std_hashmap", size), &(keys.clone(), delete_keys), |b, (keys, delete_keys)| {
            b.iter_batched(
                || {
                    let mut map = HashMap::with_capacity(size);
                    for &key in keys {
                        map.insert(key, key);
                    }
                    map
                },
                |mut map| {
                    for &key in *delete_keys {
                        std_black_box(map.remove(&key));
                    }
                    std_black_box(map)
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    
    group.finish();
}

// Mixed workload benchmark
fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");
    
    for size in [1000, 10000] {
        let keys = generate_random_keys(size, 12345);
        let operations_count = size * 2; // 2x operations per key
        
        // Generate operation sequence: 70% lookup, 20% insert, 10% delete
        let mut operations = Vec::with_capacity(operations_count);
        for i in 0..operations_count {
            let op_type = i % 10;
            let key_idx = i % size;
            
            if op_type < 7 {
                operations.push(("lookup", keys[key_idx]));
            } else if op_type < 9 {
                operations.push(("insert", keys[key_idx]));
            } else {
                operations.push(("delete", keys[key_idx]));
            }
        }
        
        group.bench_with_input(BenchmarkId::new("miso", size), &operations, |b, operations| {
            b.iter_batched(
                || {
                    let mut map = Miso::with_capacity(size);
                    // Pre-populate with half the keys
                    for &key in &keys[..size / 2] {
                        map.insert(key, key);
                    }
                    map
                },
                |mut map| {
                    for &(op, key) in operations {
                        match op {
                            "lookup" => { std_black_box(map.get(&key)); }
                            "insert" => { std_black_box(map.insert(key, key)); }
                            "delete" => { std_black_box(map.delete(&key)); }
                            _ => unreachable!()
                        }
                    }
                    std_black_box(map)
                },
                criterion::BatchSize::SmallInput,
            );
        });
        
        group.bench_with_input(BenchmarkId::new("std_hashmap", size), &operations, |b, operations| {
            b.iter_batched(
                || {
                    let mut map = HashMap::with_capacity(size);
                    // Pre-populate with half the keys
                    for &key in &keys[..size / 2] {
                        map.insert(key, key);
                    }
                    map
                },
                |mut map| {
                    for &(op, key) in operations {
                        match op {
                            "lookup" => { std_black_box(map.get(&key)); }
                            "insert" => { std_black_box(map.insert(key, key)); }
                            "delete" => { std_black_box(map.remove(&key)); }
                            _ => unreachable!()
                        }
                    }
                    std_black_box(map)
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    
    group.finish();
}

// Memory analysis benchmark
fn bench_memory_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_analysis");
    
    // This is more of a measurement than a benchmark
    group.bench_function("struct_sizes", |b| {
        b.iter(|| {
            // Measure the size of the structs themselves
            let miso_size = std::mem::size_of::<Miso<u64, u64>>();
            let hashmap_size = std::mem::size_of::<HashMap<u64, u64>>();
            
            std_black_box((miso_size, hashmap_size))
        });
    });
    
    // Memory usage per element (approximate)
    for size in [1000, 10000, 100000] {
        let keys = generate_random_keys(size, 12345);
        
        group.bench_with_input(BenchmarkId::new("miso_memory_per_element", size), &keys, |b, keys| {
            b.iter(|| {
                let mut map = Miso::with_capacity(size);
                for &key in keys {
                    map.insert(key, key);
                }
                
                // Approximate memory calculation
                let capacity = map.capacity();
                let element_size = std::mem::size_of::<Option<(u64, u64)>>();
                let metadata_size = std::mem::size_of::<u8>();
                let total_memory = capacity * (element_size + metadata_size);
                let memory_per_element = total_memory / map.size();
                
                std_black_box((map, memory_per_element))
            });
        });
        
        group.bench_with_input(BenchmarkId::new("std_memory_per_element", size), &keys, |b, keys| {
            b.iter(|| {
                let mut map = HashMap::with_capacity(size);
                for &key in keys {
                    map.insert(key, key);
                }
                
                // This is harder to measure precisely for HashMap
                // but we can estimate based on capacity and load factor
                let approximate_memory = map.capacity() * std::mem::size_of::<(u64, u64)>();
                let memory_per_element = approximate_memory / map.len();
                
                std_black_box((map, memory_per_element))
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_insert_sequential,
    bench_insert_random,
    bench_insert_presized,
    bench_lookup_hit,
    bench_lookup_miss,
    bench_lookup_mixed,
    bench_delete_all,
    bench_delete_half,
    bench_mixed_workload,
    bench_memory_analysis
);
criterion_main!(benches);
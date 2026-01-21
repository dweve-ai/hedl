// HEDL WASM Memory Optimization Benchmarks
//
// Benchmarks for memory-efficient operations in WASM environment

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use hedl_core::parse as core_parse;
use std::collections::HashMap;

/// Generate test HEDL document with specified node count
fn generate_test_doc(node_count: usize) -> String {
    let mut doc = String::from("%VERSION: 1.0\n");
    doc.push_str("%STRUCT: Entity: [id, name, value, timestamp]\n");
    doc.push_str("---\n");
    doc.push_str("entities: @Entity\n");

    for i in 0..node_count {
        doc.push_str(&format!(
            "  | entity_{}, Name {}, {}, 2024-01-01T00:00:00Z\n",
            i,
            i,
            i * 100
        ));
    }

    doc
}

fn bench_parse_memory_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_memory/parse");

    for size in [100, 1_000, 10_000].iter() {
        let input = generate_test_doc(*size);
        let bytes = input.len();

        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| {
                let doc = core_parse(black_box(input.as_bytes())).unwrap();
                black_box(doc);
            });
        });
    }

    group.finish();
}

fn bench_partial_parse_header_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_memory/partial_parse");

    for size in [1_000, 10_000, 100_000].iter() {
        let input = generate_test_doc(*size);

        group.bench_with_input(BenchmarkId::new("header_only", size), &input, |b, input| {
            b.iter(|| {
                let mut doc = core_parse(black_box(input.as_bytes())).unwrap();
                // Simulate header-only parsing by clearing entities
                doc.root.clear();
                black_box(doc);
            });
        });

        group.bench_with_input(BenchmarkId::new("full", size), &input, |b, input| {
            b.iter(|| {
                let doc = core_parse(black_box(input.as_bytes())).unwrap();
                black_box(doc);
            });
        });
    }

    group.finish();
}

fn bench_entity_counting(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_memory/counting");

    let input = generate_test_doc(10_000);
    let doc = core_parse(input.as_bytes()).unwrap();

    // Benchmark with String keys (cloning)
    group.bench_function("string_keys", |b| {
        b.iter(|| {
            let mut counts: HashMap<String, usize> = HashMap::new();
            for item in doc.root.values() {
                if let hedl_core::Item::List(list) = item {
                    *counts.entry(list.type_name.clone()).or_default() += list.rows.len();
                }
            }
            black_box(counts);
        });
    });

    // Benchmark with &str keys (no cloning)
    group.bench_function("str_ref_keys", |b| {
        b.iter(|| {
            let mut counts: HashMap<&str, usize> = HashMap::new();
            for item in doc.root.values() {
                if let hedl_core::Item::List(list) = item {
                    *counts.entry(list.type_name.as_str()).or_default() += list.rows.len();
                }
            }
            black_box(counts);
        });
    });

    group.finish();
}

fn bench_truncation_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_memory/truncation");

    let input = generate_test_doc(10_000);

    group.bench_function("truncate_only", |b| {
        b.iter(|| {
            let mut doc = core_parse(black_box(input.as_bytes())).unwrap();
            for (_key, item) in doc.root.iter_mut() {
                if let hedl_core::Item::List(list) = item {
                    list.rows.truncate(100);
                }
            }
            black_box(doc);
        });
    });

    group.bench_function("truncate_and_shrink", |b| {
        b.iter(|| {
            let mut doc = core_parse(black_box(input.as_bytes())).unwrap();
            for (_key, item) in doc.root.iter_mut() {
                if let hedl_core::Item::List(list) = item {
                    list.rows.truncate(100);
                    list.rows.shrink_to_fit();
                }
            }
            black_box(doc);
        });
    });

    group.finish();
}

fn bench_paginated_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_memory/query");

    let input = generate_test_doc(10_000);
    let doc = core_parse(input.as_bytes()).unwrap();

    // Full query (no pagination)
    group.bench_function("full_query", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for item in doc.root.values() {
                if let hedl_core::Item::List(list) = item {
                    for node in &list.rows {
                        results.push(&node.id);
                    }
                }
            }
            black_box(results);
        });
    });

    // Paginated query (offset=0, limit=100)
    group.bench_function("paginated_100", |b| {
        b.iter(|| {
            let limit = 100;
            let mut results = Vec::with_capacity(limit);
            'outer: for item in doc.root.values() {
                if let hedl_core::Item::List(list) = item {
                    for node in &list.rows {
                        results.push(&node.id);
                        if results.len() >= limit {
                            break 'outer;
                        }
                    }
                }
            }
            black_box(results);
        });
    });

    // Paginated query (offset=1000, limit=100)
    group.bench_function("paginated_offset_1000", |b| {
        b.iter(|| {
            let offset = 1000;
            let limit = 100;
            let mut results = Vec::with_capacity(limit);
            let mut count = 0;
            'outer: for item in doc.root.values() {
                if let hedl_core::Item::List(list) = item {
                    for node in &list.rows {
                        if count >= offset && results.len() < limit {
                            results.push(&node.id);
                        }
                        count += 1;
                        if results.len() >= limit {
                            break 'outer;
                        }
                    }
                }
            }
            black_box(results);
        });
    });

    group.finish();
}

fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_memory/batch_processing");

    let input = generate_test_doc(10_000);
    let doc = core_parse(input.as_bytes()).unwrap();

    for batch_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let mut processed = 0;
                    for item in doc.root.values() {
                        if let hedl_core::Item::List(list) = item {
                            for batch in list.rows.chunks(batch_size) {
                                // Simulate processing batch
                                processed += batch.len();
                                black_box(&batch);
                            }
                        }
                    }
                    black_box(processed);
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_efficiency_ratios(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_memory/efficiency");

    // Compare memory usage patterns
    for size in [1_000, 10_000].iter() {
        let input = generate_test_doc(*size);

        // Full parse
        group.bench_with_input(BenchmarkId::new("full_parse", size), &input, |b, input| {
            b.iter(|| {
                let doc = core_parse(black_box(input.as_bytes())).unwrap();
                black_box(doc);
            });
        });

        // Parse + truncate to 10%
        group.bench_with_input(
            BenchmarkId::new("parse_truncate_10pct", size),
            &input,
            |b, input| {
                b.iter(|| {
                    let mut doc = core_parse(black_box(input.as_bytes())).unwrap();
                    let max = size / 10;
                    for (_key, item) in doc.root.iter_mut() {
                        if let hedl_core::Item::List(list) = item {
                            list.rows.truncate(max);
                            list.rows.shrink_to_fit();
                        }
                    }
                    black_box(doc);
                });
            },
        );

        // Parse + header only
        group.bench_with_input(
            BenchmarkId::new("parse_header", size),
            &input,
            |b, input| {
                b.iter(|| {
                    let mut doc = core_parse(black_box(input.as_bytes())).unwrap();
                    doc.root.clear();
                    black_box(doc);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_memory_sizes,
    bench_partial_parse_header_only,
    bench_entity_counting,
    bench_truncation_strategies,
    bench_paginated_query,
    bench_batch_processing,
    bench_memory_efficiency_ratios,
);
criterion_main!(benches);

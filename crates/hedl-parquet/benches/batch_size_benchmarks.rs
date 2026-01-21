// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for batch size optimization.
//!
//! These benchmarks measure the impact of batch size on throughput and memory usage
//! across different table shapes and data types.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_parquet::{
    from_parquet_bytes_with_config, to_parquet_bytes, BatchSize, FromParquetConfig,
};
use std::hint::black_box;
use std::time::Duration;

/// Generate a test document with specified rows and columns.
fn generate_test_document(
    num_rows: usize,
    num_columns: usize,
    column_type: ColumnType,
) -> Document {
    let mut doc = Document::new((1, 0));

    // Build schema
    let mut schema = vec!["id".to_string()];
    for i in 1..num_columns {
        schema.push(format!("col{i}"));
    }

    let mut matrix_list = MatrixList::new("TestEntity", schema);

    // Generate rows
    for row_idx in 0..num_rows {
        let mut fields = vec![Value::String(format!("row{row_idx}").into())];

        // Generate column values based on type
        for col_idx in 1..num_columns {
            let value = match column_type {
                ColumnType::Integer => Value::Int((row_idx * 1000 + col_idx) as i64),
                ColumnType::Float => Value::Float((row_idx as f64) + (col_idx as f64) / 1000.0),
                ColumnType::String => Value::String(format!("value_{row_idx}_{col_idx}").into()),
                ColumnType::Mixed => {
                    if col_idx % 3 == 0 {
                        Value::Int((row_idx * col_idx) as i64)
                    } else if col_idx % 3 == 1 {
                        Value::Float((row_idx + col_idx) as f64)
                    } else {
                        Value::String(format!("mixed_{row_idx}_{col_idx}").into())
                    }
                }
            };
            fields.push(value);
        }

        let node = Node::new("TestEntity", format!("row{row_idx}"), fields);
        matrix_list.add_row(node);
    }

    doc.root
        .insert("test_data".to_string(), Item::List(matrix_list));
    doc
}

#[derive(Debug, Clone, Copy)]
enum ColumnType {
    Integer,
    #[allow(dead_code)]
    Float,
    String,
    Mixed,
}

/// Benchmark reading with different batch sizes on narrow tables (< 20 columns).
fn bench_narrow_table_batch_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("narrow_table_batch_sizes");
    group.measurement_time(Duration::from_secs(10));

    // Generate test data: 1M rows × 10 columns
    let num_rows = 1_000_000;
    let num_columns = 10;

    let doc = generate_test_document(num_rows, num_columns, ColumnType::Integer);
    let parquet_bytes = to_parquet_bytes(&doc).unwrap();

    // Set throughput based on uncompressed data size
    group.throughput(Throughput::Bytes(parquet_bytes.len() as u64));

    // Test different batch sizes
    for batch_size in &[1_000, 10_000, 65_536, 100_000, 500_000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let config =
                        FromParquetConfig::default().with_batch_size(BatchSize::Fixed(size));
                    black_box(from_parquet_bytes_with_config(&parquet_bytes, &config).unwrap())
                });
            },
        );
    }

    // Test Auto mode
    group.bench_function("auto", |b| {
        b.iter(|| {
            let config = FromParquetConfig::default().with_batch_size(BatchSize::Auto);
            black_box(from_parquet_bytes_with_config(&parquet_bytes, &config).unwrap())
        });
    });

    group.finish();
}

/// Benchmark reading with different batch sizes on wide tables (> 50 columns).
fn bench_wide_table_batch_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("wide_table_batch_sizes");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(20); // Fewer samples for slower benchmarks

    // Generate test data: 100K rows × 100 columns
    let num_rows = 100_000;
    let num_columns = 100;

    let doc = generate_test_document(num_rows, num_columns, ColumnType::Mixed);
    let parquet_bytes = to_parquet_bytes(&doc).unwrap();

    group.throughput(Throughput::Bytes(parquet_bytes.len() as u64));

    // Test different batch sizes
    for batch_size in &[1_000, 8_192, 16_384, 32_768, 65_536] {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let config =
                        FromParquetConfig::default().with_batch_size(BatchSize::Fixed(size));
                    black_box(from_parquet_bytes_with_config(&parquet_bytes, &config).unwrap())
                });
            },
        );
    }

    // Test Auto mode
    group.bench_function("auto", |b| {
        b.iter(|| {
            let config = FromParquetConfig::default().with_batch_size(BatchSize::Auto);
            black_box(from_parquet_bytes_with_config(&parquet_bytes, &config).unwrap())
        });
    });

    group.finish();
}

/// Benchmark string-heavy tables (high memory variability).
fn bench_string_heavy_batch_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_heavy_batch_sizes");
    group.measurement_time(Duration::from_secs(12));

    // Generate test data: 500K rows × 20 columns (all strings)
    let num_rows = 500_000;
    let num_columns = 20;

    let doc = generate_test_document(num_rows, num_columns, ColumnType::String);
    let parquet_bytes = to_parquet_bytes(&doc).unwrap();

    group.throughput(Throughput::Bytes(parquet_bytes.len() as u64));

    // Test different batch sizes
    for batch_size in &[5_000, 16_384, 32_768, 65_536, 131_072] {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let config =
                        FromParquetConfig::default().with_batch_size(BatchSize::Fixed(size));
                    black_box(from_parquet_bytes_with_config(&parquet_bytes, &config).unwrap())
                });
            },
        );
    }

    // Test Auto mode
    group.bench_function("auto", |b| {
        b.iter(|| {
            let config = FromParquetConfig::default().with_batch_size(BatchSize::Auto);
            black_box(from_parquet_bytes_with_config(&parquet_bytes, &config).unwrap())
        });
    });

    group.finish();
}

/// Benchmark small tables to verify overhead is acceptable.
fn bench_small_table_batch_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_table_batch_sizes");
    group.measurement_time(Duration::from_secs(5));

    // Generate test data: 1K rows × 5 columns
    let num_rows = 1_000;
    let num_columns = 5;

    let doc = generate_test_document(num_rows, num_columns, ColumnType::Integer);
    let parquet_bytes = to_parquet_bytes(&doc).unwrap();

    group.throughput(Throughput::Bytes(parquet_bytes.len() as u64));

    // Test different batch sizes
    for batch_size in &[100, 500, 1_000, 5_000, 10_000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let config =
                        FromParquetConfig::default().with_batch_size(BatchSize::Fixed(size));
                    black_box(from_parquet_bytes_with_config(&parquet_bytes, &config).unwrap())
                });
            },
        );
    }

    // Test Auto mode
    group.bench_function("auto", |b| {
        b.iter(|| {
            let config = FromParquetConfig::default().with_batch_size(BatchSize::Auto);
            black_box(from_parquet_bytes_with_config(&parquet_bytes, &config).unwrap())
        });
    });

    group.finish();
}

/// Benchmark adaptive batch sizing strategy.
fn bench_adaptive_batch_sizing(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_vs_fixed");
    group.measurement_time(Duration::from_secs(10));

    // Generate mixed workload
    let num_rows = 500_000;
    let num_columns = 30;

    let doc = generate_test_document(num_rows, num_columns, ColumnType::Mixed);
    let parquet_bytes = to_parquet_bytes(&doc).unwrap();

    group.throughput(Throughput::Bytes(parquet_bytes.len() as u64));

    // Fixed batch size
    group.bench_function("fixed_32k", |b| {
        b.iter(|| {
            let config = FromParquetConfig::default().with_batch_size(BatchSize::Fixed(32_768));
            black_box(from_parquet_bytes_with_config(&parquet_bytes, &config).unwrap())
        });
    });

    // Adaptive batch size
    group.bench_function("adaptive_32k", |b| {
        b.iter(|| {
            let config = FromParquetConfig::default().with_batch_size(BatchSize::Adaptive(32_768));
            black_box(from_parquet_bytes_with_config(&parquet_bytes, &config).unwrap())
        });
    });

    // Auto batch size
    group.bench_function("auto", |b| {
        b.iter(|| {
            let config = FromParquetConfig::default().with_batch_size(BatchSize::Auto);
            black_box(from_parquet_bytes_with_config(&parquet_bytes, &config).unwrap())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_narrow_table_batch_sizes,
    bench_wide_table_batch_sizes,
    bench_string_heavy_batch_sizes,
    bench_small_table_batch_sizes,
    bench_adaptive_batch_sizing,
);
criterion_main!(benches);

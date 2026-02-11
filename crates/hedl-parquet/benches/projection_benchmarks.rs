// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Performance benchmarks for Parquet column projection (projection pushdown).
//!
//! These benchmarks measure the performance improvement from reading only
//! selected columns versus reading all columns from Parquet files.
//!
//! Expected results (based on plan analysis):
//! - 100-column table, read 10 columns: 8-10x speedup, 90% memory reduction
//! - 50-column table, read 5 columns: 8-10x speedup, 90% memory reduction
//! - 50-column table, read 2 columns: 20-25x speedup, 96% memory reduction
//! - 100-column table, read 1 column: 80-100x speedup, 99% memory reduction
//!
//! These benchmarks validate that projection pushdown provides the expected
//! performance characteristics for real-world analytics workloads.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_parquet::{from_parquet_bytes, from_parquet_bytes_select, to_parquet_bytes};
use std::hint::black_box;

/// Generate a test document with specified dimensions.
///
/// Creates a table with `num_rows` rows and `num_columns` columns.
/// Data types are varied to simulate real-world tables.
fn generate_table(num_rows: usize, num_columns: usize) -> Document {
    let mut doc = Document::new((2, 0));

    // Build schema: id + col1, col2, col3, ...
    let mut schema = vec!["id".to_string()];
    for i in 1..num_columns {
        schema.push(format!("col{i}"));
    }

    let mut matrix_list = MatrixList::new("Data", schema);

    // Generate rows with varied data types
    for row_idx in 0..num_rows {
        let mut fields = vec![Value::String(format!("row{row_idx}").into())];

        for col_idx in 1..num_columns {
            let value = match col_idx % 4 {
                0 => Value::Int((row_idx * col_idx) as i64),
                1 => Value::Float((row_idx * col_idx) as f64 * 0.5),
                2 => Value::String(format!("data_{row_idx}_{col_idx}").into()),
                _ => Value::Bool(row_idx % 2 == 0),
            };
            fields.push(value);
        }

        let node = Node::new("Data", format!("row{row_idx}"), fields);
        matrix_list.add_row(node);
    }

    doc.root.insert("data".to_string(), Item::List(matrix_list));
    doc
}

/// Benchmark: Wide table (100 columns) with selective reads (10 columns = 10% selectivity).
///
/// Expected: 8-10x speedup for projected read vs full read.
fn bench_wide_table_selective_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("wide_table_100_cols");

    // Generate 100-column table with 10K rows
    let doc = generate_table(10_000, 100);
    let parquet_bytes = to_parquet_bytes(&doc).unwrap();

    // Calculate throughput based on rows processed
    group.throughput(Throughput::Elements(10_000));

    // Benchmark: Full read (all 100 columns)
    group.bench_function("full_read_100_columns", |b| {
        b.iter(|| {
            let result = from_parquet_bytes(black_box(&parquet_bytes));
            black_box(result.unwrap());
        });
    });

    // Benchmark: Projected read (10 columns = 10% selectivity)
    let projection_10 = (0..10)
        .map(|i| {
            if i == 0 {
                "id".to_string()
            } else {
                format!("col{i}")
            }
        })
        .collect::<Vec<_>>();

    group.bench_function("projected_read_10_columns", |b| {
        b.iter(|| {
            let result = from_parquet_bytes_select(
                black_box(&parquet_bytes),
                black_box(projection_10.clone()),
            );
            black_box(result.unwrap());
        });
    });

    // Benchmark: Very selective (5 columns = 5% selectivity)
    let projection_5 = (0..5)
        .map(|i| {
            if i == 0 {
                "id".to_string()
            } else {
                format!("col{i}")
            }
        })
        .collect::<Vec<_>>();

    group.bench_function("projected_read_5_columns", |b| {
        b.iter(|| {
            let result = from_parquet_bytes_select(
                black_box(&parquet_bytes),
                black_box(projection_5.clone()),
            );
            black_box(result.unwrap());
        });
    });

    // Benchmark: Single column (1% selectivity)
    let projection_1 = vec!["id".to_string()];

    group.bench_function("projected_read_1_column", |b| {
        b.iter(|| {
            let result = from_parquet_bytes_select(
                black_box(&parquet_bytes),
                black_box(projection_1.clone()),
            );
            black_box(result.unwrap());
        });
    });

    group.finish();
}

/// Benchmark: Medium-width table (50 columns) with various selectivity ratios.
///
/// Expected: 10-20x speedup for 10% selectivity, 20-25x for 4% selectivity.
fn bench_medium_table_selectivity(c: &mut Criterion) {
    let mut group = c.benchmark_group("medium_table_50_cols");

    // Generate 50-column table with 10K rows
    let doc = generate_table(10_000, 50);
    let parquet_bytes = to_parquet_bytes(&doc).unwrap();

    group.throughput(Throughput::Elements(10_000));

    // Full read baseline
    group.bench_function("full_read_50_columns", |b| {
        b.iter(|| {
            let result = from_parquet_bytes(black_box(&parquet_bytes));
            black_box(result.unwrap());
        });
    });

    // 20% selectivity (10 columns)
    let projection_20pct = (0..10)
        .map(|i| {
            if i == 0 {
                "id".to_string()
            } else {
                format!("col{i}")
            }
        })
        .collect::<Vec<_>>();

    group.bench_function("projected_20pct_selectivity", |b| {
        b.iter(|| {
            let result = from_parquet_bytes_select(
                black_box(&parquet_bytes),
                black_box(projection_20pct.clone()),
            );
            black_box(result.unwrap());
        });
    });

    // 10% selectivity (5 columns)
    let projection_10pct = (0..5)
        .map(|i| {
            if i == 0 {
                "id".to_string()
            } else {
                format!("col{i}")
            }
        })
        .collect::<Vec<_>>();

    group.bench_function("projected_10pct_selectivity", |b| {
        b.iter(|| {
            let result = from_parquet_bytes_select(
                black_box(&parquet_bytes),
                black_box(projection_10pct.clone()),
            );
            black_box(result.unwrap());
        });
    });

    // 4% selectivity (2 columns)
    let projection_4pct = vec!["id".to_string(), "col1".to_string()];

    group.bench_function("projected_4pct_selectivity", |b| {
        b.iter(|| {
            let result = from_parquet_bytes_select(
                black_box(&parquet_bytes),
                black_box(projection_4pct.clone()),
            );
            black_box(result.unwrap());
        });
    });

    group.finish();
}

/// Benchmark: String-heavy table (simulating text analytics workload).
///
/// String columns are more expensive to decompress than numeric columns.
/// Expected: 5-8x speedup for 20% selectivity with strings.
fn bench_string_heavy_table(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_heavy_table");

    // Generate table with mostly string columns
    let mut doc = Document::new((2, 0));
    let num_columns = 30;
    let num_rows = 5_000;

    let mut schema = vec!["id".to_string()];
    for i in 1..num_columns {
        schema.push(format!("col{i}"));
    }

    let mut matrix_list = MatrixList::new("Data", schema);

    for row_idx in 0..num_rows {
        let mut fields = vec![Value::String(format!("row{row_idx}").into())];

        for col_idx in 1..num_columns {
            // 80% strings, 20% numbers
            let value = if col_idx % 5 == 0 {
                Value::Int(i64::from(row_idx * col_idx))
            } else {
                // Longer strings (average 50 chars) to simulate real text data
                Value::String(
                    format!(
                        "This is a longer text field for row {row_idx} column {col_idx} with some data"
                    )
                    .into(),
                )
            };
            fields.push(value);
        }

        let node = Node::new("Data", format!("row{row_idx}"), fields);
        matrix_list.add_row(node);
    }

    doc.root.insert("data".to_string(), Item::List(matrix_list));
    let parquet_bytes = to_parquet_bytes(&doc).unwrap();

    group.throughput(Throughput::Elements(num_rows as u64));

    // Full read
    group.bench_function("full_read_30_string_cols", |b| {
        b.iter(|| {
            let result = from_parquet_bytes(black_box(&parquet_bytes));
            black_box(result.unwrap());
        });
    });

    // Read 6 columns (20% selectivity, mix of strings and ints)
    let projection = vec![
        "id".to_string(),
        "col1".to_string(),
        "col5".to_string(),
        "col10".to_string(),
        "col15".to_string(),
        "col20".to_string(),
    ];

    group.bench_function("projected_6_columns_mixed", |b| {
        b.iter(|| {
            let result =
                from_parquet_bytes_select(black_box(&parquet_bytes), black_box(projection.clone()));
            black_box(result.unwrap());
        });
    });

    group.finish();
}

/// Benchmark: Analytics aggregation pattern (GROUP BY simulation).
///
/// Simulates reading only aggregation columns from a wide fact table.
/// Common in OLAP workloads.
fn bench_analytics_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("analytics_aggregation");

    // Generate fact table: 80 columns, 20K rows
    let doc = generate_table(20_000, 80);
    let parquet_bytes = to_parquet_bytes(&doc).unwrap();

    group.throughput(Throughput::Elements(20_000));

    // Full read (wasteful for aggregation)
    group.bench_function("full_read_fact_table", |b| {
        b.iter(|| {
            let result = from_parquet_bytes(black_box(&parquet_bytes));
            black_box(result.unwrap());
        });
    });

    // Aggregation query: SELECT region, product, SUM(sales)
    // Only need 3 columns (group key + aggregation columns)
    let aggregation_projection = vec![
        "id".to_string(),
        "col1".to_string(), // region
        "col2".to_string(), // product
    ];

    group.bench_function("aggregation_3_columns", |b| {
        b.iter(|| {
            let result = from_parquet_bytes_select(
                black_box(&parquet_bytes),
                black_box(aggregation_projection.clone()),
            );
            black_box(result.unwrap());
        });
    });

    group.finish();
}

/// Benchmark: Join key extraction (graph construction pattern).
///
/// Simulates extracting foreign keys for graph edge construction.
/// Needs only ID + foreign key columns.
fn bench_join_key_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("join_key_extraction");

    // Wide table: 100 columns, but we only need 2 for joins
    let doc = generate_table(50_000, 100);
    let parquet_bytes = to_parquet_bytes(&doc).unwrap();

    group.throughput(Throughput::Elements(50_000));

    // Full read (extremely wasteful)
    group.bench_function("full_read_for_join", |b| {
        b.iter(|| {
            let result = from_parquet_bytes(black_box(&parquet_bytes));
            black_box(result.unwrap());
        });
    });

    // Join extraction: only primary key + foreign key
    let join_projection = vec![
        "id".to_string(),    // primary key
        "col10".to_string(), // foreign key
    ];

    group.bench_function("extract_2_join_keys", |b| {
        b.iter(|| {
            let result = from_parquet_bytes_select(
                black_box(&parquet_bytes),
                black_box(join_projection.clone()),
            );
            black_box(result.unwrap());
        });
    });

    group.finish();
}

/// Benchmark: Time-series sensor data (`IoT` pattern).
///
/// Many sensor columns, but analysis focuses on subset of sensors.
fn bench_timeseries_sensors(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeseries_sensors");

    // Sensor table: 120 sensor columns, 10K timestamps
    let doc = generate_table(10_000, 120);
    let parquet_bytes = to_parquet_bytes(&doc).unwrap();

    group.throughput(Throughput::Elements(10_000));

    // Full read (all sensors)
    group.bench_function("full_read_120_sensors", |b| {
        b.iter(|| {
            let result = from_parquet_bytes(black_box(&parquet_bytes));
            black_box(result.unwrap());
        });
    });

    // Analyze 10 specific sensors (8.3% selectivity)
    let sensor_projection = vec![
        "id".to_string(),
        "col1".to_string(),
        "col5".to_string(),
        "col10".to_string(),
        "col15".to_string(),
        "col20".to_string(),
        "col25".to_string(),
        "col30".to_string(),
        "col35".to_string(),
        "col40".to_string(),
    ];

    group.bench_function("projected_10_sensors", |b| {
        b.iter(|| {
            let result = from_parquet_bytes_select(
                black_box(&parquet_bytes),
                black_box(sensor_projection.clone()),
            );
            black_box(result.unwrap());
        });
    });

    group.finish();
}

/// Benchmark: ML feature selection (data science pattern).
///
/// Training dataset with many features, but model uses only selected features.
fn bench_ml_feature_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_feature_selection");

    // Feature table: 200 features, 5K samples
    let doc = generate_table(5_000, 200);
    let parquet_bytes = to_parquet_bytes(&doc).unwrap();

    group.throughput(Throughput::Elements(5_000));

    // Full read (all features)
    group.bench_function("full_read_200_features", |b| {
        b.iter(|| {
            let result = from_parquet_bytes(black_box(&parquet_bytes));
            black_box(result.unwrap());
        });
    });

    // Model uses 15 selected features (7.5% selectivity)
    let feature_projection = (0..15)
        .map(|i| {
            if i == 0 {
                "id".to_string()
            } else {
                format!("col{}", i * 10)
            }
        })
        .collect::<Vec<_>>();

    group.bench_function("projected_15_features", |b| {
        b.iter(|| {
            let result = from_parquet_bytes_select(
                black_box(&parquet_bytes),
                black_box(feature_projection.clone()),
            );
            black_box(result.unwrap());
        });
    });

    group.finish();
}

/// Benchmark: Projection overhead (verify no regression for full reads).
///
/// Ensures that projecting all columns has negligible overhead vs non-projected read.
fn bench_projection_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("projection_overhead");

    let doc = generate_table(10_000, 20);
    let parquet_bytes = to_parquet_bytes(&doc).unwrap();

    group.throughput(Throughput::Elements(10_000));

    // Non-projected read
    group.bench_function("no_projection", |b| {
        b.iter(|| {
            let result = from_parquet_bytes(black_box(&parquet_bytes));
            black_box(result.unwrap());
        });
    });

    // Projected read with all columns
    let all_columns = (0..20)
        .map(|i| {
            if i == 0 {
                "id".to_string()
            } else {
                format!("col{i}")
            }
        })
        .collect::<Vec<_>>();

    group.bench_function("projection_all_columns", |b| {
        b.iter(|| {
            let result = from_parquet_bytes_select(
                black_box(&parquet_bytes),
                black_box(all_columns.clone()),
            );
            black_box(result.unwrap());
        });
    });

    group.finish();
}

/// Benchmark: Column count scaling (understand speedup across various table widths).
fn bench_column_count_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("column_count_scaling");

    // Test various table widths with constant selectivity (10%)
    let table_widths = vec![20, 50, 100, 150, 200];
    let num_rows = 5_000;

    for width in table_widths {
        let doc = generate_table(num_rows, width);
        let parquet_bytes = to_parquet_bytes(&doc).unwrap();

        // Read 10% of columns
        let num_projected = (width / 10).max(2);
        let projection = (0..num_projected)
            .map(|i| {
                if i == 0 {
                    "id".to_string()
                } else {
                    format!("col{i}")
                }
            })
            .collect::<Vec<_>>();

        group.bench_with_input(
            BenchmarkId::new("full_read", width),
            &parquet_bytes,
            |b, bytes| {
                b.iter(|| {
                    let result = from_parquet_bytes(black_box(bytes));
                    black_box(result.unwrap());
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("projected_10pct", width),
            &(parquet_bytes, projection),
            |b, (bytes, proj)| {
                b.iter(|| {
                    let result =
                        from_parquet_bytes_select(black_box(bytes), black_box(proj.clone()));
                    black_box(result.unwrap());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Row count scaling (verify linear scaling behavior).
fn bench_row_count_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("row_count_scaling");

    let row_counts = vec![1_000, 5_000, 10_000, 20_000];
    let num_columns = 50;

    for row_count in row_counts {
        let doc = generate_table(row_count, num_columns);
        let parquet_bytes = to_parquet_bytes(&doc).unwrap();

        // Project 5 columns (10% selectivity)
        let projection = (0..5)
            .map(|i| {
                if i == 0 {
                    "id".to_string()
                } else {
                    format!("col{i}")
                }
            })
            .collect::<Vec<_>>();

        group.throughput(Throughput::Elements(row_count as u64));

        group.bench_with_input(
            BenchmarkId::new("projected_5_cols", row_count),
            &(parquet_bytes, projection),
            |b, (bytes, proj)| {
                b.iter(|| {
                    let result =
                        from_parquet_bytes_select(black_box(bytes), black_box(proj.clone()));
                    black_box(result.unwrap());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_wide_table_selective_read,
    bench_medium_table_selectivity,
    bench_string_heavy_table,
    bench_analytics_aggregation,
    bench_join_key_extraction,
    bench_timeseries_sensors,
    bench_ml_feature_selection,
    bench_projection_overhead,
    bench_column_count_scaling,
    bench_row_count_scaling,
);

criterion_main!(benches);

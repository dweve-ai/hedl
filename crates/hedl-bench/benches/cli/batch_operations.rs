// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Benchmarks for hedl-cli batch processing operations.
//!
//! These benchmarks measure the performance and scalability of batch processing
//! operations (validate, format, lint) across different file counts and sizes.
//!
//! # Benchmark Groups
//!
//! - **`batch_validate_scaling`**: Validation throughput vs file count
//! - **`batch_format_scaling`**: Formatting throughput vs file count
//! - **`batch_lint_scaling`**: Linting throughput vs file count
//! - **`batch_operation_comparison`**: Compare different operations
//! - **`batch_file_size_impact`**: Performance vs file size
//! - **`batch_parallel_efficiency`**: Parallel speedup measurement
//!
//! # Running Benchmarks
//!
//! ```bash
//! # Run all batch benchmarks
//! cargo bench --bench batch_operations
//!
//! # Run specific benchmark group
//! cargo bench --bench batch_operations -- batch_validate_scaling
//!
//! # Generate detailed HTML report
//! cargo bench --bench batch_operations -- --verbose
//! ```
//!
//! # Performance Expectations
//!
//! Based on the implementation analysis:
//!
//! - **Small batches (< 10 files)**: Serial processing, minimal overhead
//! - **Medium batches (10-100)**: Parallel with 6-8x speedup on 8-core machines
//! - **Large batches (> 100)**: Sustained throughput of 100-200 files/second
//!
//! # Regression Detection
//!
//! Thresholds for regression alerts:
//! - Throughput drop > 10% = investigate
//! - Memory increase > 20% = investigate
//! - Error rate increase > 1% = block merge

use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, PlotConfiguration, Throughput,
};
use hedl_bench::generators::hierarchical::generate_deep_nesting;
use hedl_bench::generators::simple::{generate_flat_struct, generate_list_simple};
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Test Data Generation
// ============================================================================

/// Test file configuration for benchmarks
#[derive(Debug, Clone)]
struct TestFileConfig {
    /// Number of files to generate
    count: usize,
    /// Average size of each file in bytes
    avg_size: usize,
    /// Complexity level (simple, medium, complex)
    complexity: Complexity,
}

#[derive(Debug, Clone, Copy)]
enum Complexity {
    Simple,  // Flat structures, minimal nesting
    Medium,  // Some nesting, typical real-world documents
    Complex, // Deep nesting, many references
}

/// Generate test files for benchmarking
fn generate_test_files(config: &TestFileConfig) -> (TempDir, Vec<PathBuf>) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut paths = Vec::with_capacity(config.count);

    for i in 0..config.count {
        let content = generate_hedl_content(config.avg_size, config.complexity);
        let file_path = temp_dir.path().join(format!("test_{i}.hedl"));

        fs::write(&file_path, content).expect("Failed to write test file");
        paths.push(file_path);
    }

    (temp_dir, paths)
}

/// Generate HEDL content based on size and complexity
fn generate_hedl_content(target_size: usize, complexity: Complexity) -> String {
    match complexity {
        Complexity::Simple => {
            // Generate simple flat structures
            let field_count = target_size / 50; // ~50 bytes per field
            generate_flat_struct(field_count.max(5))
        }
        Complexity::Medium => {
            // Generate list structures
            let item_count = target_size / 100; // ~100 bytes per item
            generate_list_simple(item_count.max(10))
        }
        Complexity::Complex => {
            // Generate complex nested documents with deep hierarchy
            let depth = (target_size / 2000).clamp(3, 10); // Depth based on size
            let fields_per_level = 5;
            generate_deep_nesting(depth, fields_per_level)
        }
    }
}

// ============================================================================
// Batch Validation Benchmarks
// ============================================================================

/// Benchmark batch validation scaling across different file counts
fn batch_validate_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_validate_scaling");
    group
        .plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));

    for file_count in [1, 5, 10, 20, 50, 100] {
        let config = TestFileConfig {
            count: file_count,
            avg_size: 5_000, // 5KB files
            complexity: Complexity::Medium,
        };

        let (_temp_dir, files) = generate_test_files(&config);

        group.throughput(Throughput::Elements(file_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(file_count),
            &files,
            |b, files| {
                b.iter(|| {
                    // Import inside to avoid linking issues if hedl-cli isn't available
                    use hedl_cli::batch::{BatchConfig, BatchExecutor, ValidationOperation};

                    let processor = BatchExecutor::new(BatchConfig::default());
                    let operation = ValidationOperation { strict: false };

                    processor
                        .process(black_box(files), operation, false)
                        .expect("Validation should succeed");
                });
            },
        );
    }

    group.finish();
}

/// Benchmark batch validation with strict mode
fn batch_validate_strict(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_validate_strict");

    for file_count in [10, 50, 100] {
        let config = TestFileConfig {
            count: file_count,
            avg_size: 5_000,
            complexity: Complexity::Medium,
        };

        let (_temp_dir, files) = generate_test_files(&config);

        group.throughput(Throughput::Elements(file_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(file_count),
            &files,
            |b, files| {
                b.iter(|| {
                    use hedl_cli::batch::{BatchConfig, BatchExecutor, ValidationOperation};

                    let processor = BatchExecutor::new(BatchConfig::default());
                    let operation = ValidationOperation { strict: true };

                    processor
                        .process(black_box(files), operation, false)
                        .expect("Validation should succeed");
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Batch Format Benchmarks
// ============================================================================

/// Benchmark batch formatting scaling across different file counts
fn batch_format_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_format_scaling");
    group
        .plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));

    for file_count in [1, 5, 10, 20, 50, 100] {
        let config = TestFileConfig {
            count: file_count,
            avg_size: 5_000,
            complexity: Complexity::Medium,
        };

        let (_temp_dir, files) = generate_test_files(&config);

        group.throughput(Throughput::Elements(file_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(file_count),
            &files,
            |b, files| {
                b.iter(|| {
                    use hedl_cli::batch::{BatchConfig, BatchExecutor, FormatOperation};

                    let processor = BatchExecutor::new(BatchConfig::default());
                    let operation = FormatOperation {
                        check: false,
                        ditto: false,
                        with_counts: false,
                    };

                    processor
                        .process(black_box(files), operation, false)
                        .expect("Formatting should succeed");
                });
            },
        );
    }

    group.finish();
}

/// Benchmark batch formatting with count hints
fn batch_format_with_counts(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_format_with_counts");

    for file_count in [10, 50, 100] {
        let config = TestFileConfig {
            count: file_count,
            avg_size: 10_000, // Larger files to make count hints impact visible
            complexity: Complexity::Medium,
        };

        let (_temp_dir, files) = generate_test_files(&config);

        group.throughput(Throughput::Elements(file_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(file_count),
            &files,
            |b, files| {
                b.iter(|| {
                    use hedl_cli::batch::{BatchConfig, BatchExecutor, FormatOperation};

                    let processor = BatchExecutor::new(BatchConfig::default());
                    let operation = FormatOperation {
                        check: false,
                        ditto: false,
                        with_counts: true,
                    };

                    processor
                        .process(black_box(files), operation, false)
                        .expect("Formatting should succeed");
                });
            },
        );
    }

    group.finish();
}

/// Benchmark batch format check mode (CI usage)
fn batch_format_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_format_check");

    for file_count in [10, 50, 100] {
        let config = TestFileConfig {
            count: file_count,
            avg_size: 5_000,
            complexity: Complexity::Medium,
        };

        let (_temp_dir, files) = generate_test_files(&config);

        group.throughput(Throughput::Elements(file_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(file_count),
            &files,
            |b, files| {
                b.iter(|| {
                    use hedl_cli::batch::{BatchConfig, BatchExecutor, FormatOperation};

                    let processor = BatchExecutor::new(BatchConfig::default());
                    let operation = FormatOperation {
                        check: true,
                        ditto: false,
                        with_counts: false,
                    };

                    // Check mode may fail if files aren't canonical, that's ok
                    let _ = processor.process(black_box(files), operation, false);
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Batch Lint Benchmarks
// ============================================================================

/// Benchmark batch linting scaling across different file counts
fn batch_lint_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_lint_scaling");
    group
        .plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));

    for file_count in [1, 5, 10, 20, 50, 100] {
        let config = TestFileConfig {
            count: file_count,
            avg_size: 5_000,
            complexity: Complexity::Medium,
        };

        let (_temp_dir, files) = generate_test_files(&config);

        group.throughput(Throughput::Elements(file_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(file_count),
            &files,
            |b, files| {
                b.iter(|| {
                    use hedl_cli::batch::{BatchConfig, BatchExecutor, LintOperation};

                    let processor = BatchExecutor::new(BatchConfig::default());
                    let operation = LintOperation { warn_error: false };

                    processor
                        .process(black_box(files), operation, false)
                        .expect("Linting should succeed");
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// File Size Impact Benchmarks
// ============================================================================

/// Benchmark the impact of file size on batch processing performance
fn batch_file_size_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_file_size_impact");

    // Test with fixed file count but varying sizes
    let file_count = 20;

    for file_size in [1_000, 5_000, 10_000, 50_000, 100_000] {
        let config = TestFileConfig {
            count: file_count,
            avg_size: file_size,
            complexity: Complexity::Medium,
        };

        let (_temp_dir, files) = generate_test_files(&config);

        // Report throughput in bytes
        group.throughput(Throughput::Bytes((file_count * file_size) as u64));
        group.bench_with_input(
            BenchmarkId::new("validate", file_size),
            &files,
            |b, files| {
                b.iter(|| {
                    use hedl_cli::batch::{BatchConfig, BatchExecutor, ValidationOperation};

                    let processor = BatchExecutor::new(BatchConfig::default());
                    let operation = ValidationOperation { strict: false };

                    processor
                        .process(black_box(files), operation, false)
                        .expect("Validation should succeed");
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("format", file_size), &files, |b, files| {
            b.iter(|| {
                use hedl_cli::batch::{BatchConfig, BatchExecutor, FormatOperation};

                let processor = BatchExecutor::new(BatchConfig::default());
                let operation = FormatOperation {
                    check: false,
                    ditto: false,
                    with_counts: false,
                };

                processor
                    .process(black_box(files), operation, false)
                    .expect("Formatting should succeed");
            });
        });
    }

    group.finish();
}

// ============================================================================
// Parallel Efficiency Benchmarks
// ============================================================================

/// Benchmark parallel efficiency by comparing serial vs parallel execution
fn batch_parallel_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_parallel_efficiency");

    let file_count = 50;
    let config = TestFileConfig {
        count: file_count,
        avg_size: 5_000,
        complexity: Complexity::Medium,
    };

    let (_temp_dir, files) = generate_test_files(&config);

    // Serial execution (threshold high to force serial)
    group.bench_function("serial", |b| {
        b.iter(|| {
            use hedl_cli::batch::{BatchConfig, BatchExecutor, ValidationOperation};

            let processor = BatchExecutor::new(BatchConfig {
                parallel_threshold: 1000, // Force serial
                ..Default::default()
            });
            let operation = ValidationOperation { strict: false };

            processor
                .process(black_box(&files), operation, false)
                .expect("Validation should succeed");
        });
    });

    // Parallel execution (default threshold)
    group.bench_function("parallel", |b| {
        b.iter(|| {
            use hedl_cli::batch::{BatchConfig, BatchExecutor, ValidationOperation};

            let processor = BatchExecutor::new(BatchConfig::default());
            let operation = ValidationOperation { strict: false };

            processor
                .process(black_box(&files), operation, false)
                .expect("Validation should succeed");
        });
    });

    // Parallel with limited threads
    group.bench_function("parallel_4_threads", |b| {
        b.iter(|| {
            use hedl_cli::batch::{BatchConfig, BatchExecutor, ValidationOperation};

            let processor = BatchExecutor::new(BatchConfig {
                max_threads: Some(4),
                ..Default::default()
            });
            let operation = ValidationOperation { strict: false };

            processor
                .process(black_box(&files), operation, false)
                .expect("Validation should succeed");
        });
    });

    group.finish();
}

// ============================================================================
// Operation Comparison Benchmarks
// ============================================================================

/// Compare different batch operations on the same dataset
fn batch_operation_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operation_comparison");

    let file_count = 50;
    let config = TestFileConfig {
        count: file_count,
        avg_size: 5_000,
        complexity: Complexity::Medium,
    };

    let (_temp_dir, files) = generate_test_files(&config);

    group.throughput(Throughput::Elements(file_count as u64));

    group.bench_function("validate", |b| {
        b.iter(|| {
            use hedl_cli::batch::{BatchConfig, BatchExecutor, ValidationOperation};

            let processor = BatchExecutor::new(BatchConfig::default());
            let operation = ValidationOperation { strict: false };

            processor
                .process(black_box(&files), operation, false)
                .expect("Validation should succeed");
        });
    });

    group.bench_function("format", |b| {
        b.iter(|| {
            use hedl_cli::batch::{BatchConfig, BatchExecutor, FormatOperation};

            let processor = BatchExecutor::new(BatchConfig::default());
            let operation = FormatOperation {
                check: false,
                ditto: false,
                with_counts: false,
            };

            processor
                .process(black_box(&files), operation, false)
                .expect("Formatting should succeed");
        });
    });

    group.bench_function("lint", |b| {
        b.iter(|| {
            use hedl_cli::batch::{BatchConfig, BatchExecutor, LintOperation};

            let processor = BatchExecutor::new(BatchConfig::default());
            let operation = LintOperation { warn_error: false };

            processor
                .process(black_box(&files), operation, false)
                .expect("Linting should succeed");
        });
    });

    group.finish();
}

// ============================================================================
// Complexity Impact Benchmarks
// ============================================================================

/// Benchmark the impact of document complexity on batch processing
fn batch_complexity_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_complexity_impact");

    let file_count = 20;
    let file_size = 10_000;

    for (name, complexity) in [
        ("simple", Complexity::Simple),
        ("medium", Complexity::Medium),
        ("complex", Complexity::Complex),
    ] {
        let config = TestFileConfig {
            count: file_count,
            avg_size: file_size,
            complexity,
        };

        let (_temp_dir, files) = generate_test_files(&config);

        group.bench_with_input(BenchmarkId::new("validate", name), &files, |b, files| {
            b.iter(|| {
                use hedl_cli::batch::{BatchConfig, BatchExecutor, ValidationOperation};

                let processor = BatchExecutor::new(BatchConfig::default());
                let operation = ValidationOperation { strict: false };

                processor
                    .process(black_box(files), operation, false)
                    .expect("Validation should succeed");
            });
        });

        group.bench_with_input(BenchmarkId::new("format", name), &files, |b, files| {
            b.iter(|| {
                use hedl_cli::batch::{BatchConfig, BatchExecutor, FormatOperation};

                let processor = BatchExecutor::new(BatchConfig::default());
                let operation = FormatOperation {
                    check: false,
                    ditto: false,
                    with_counts: false,
                };

                processor
                    .process(black_box(files), operation, false)
                    .expect("Formatting should succeed");
            });
        });
    }

    group.finish();
}

// ============================================================================
// Threshold Configuration Benchmarks
// ============================================================================

/// Benchmark the impact of `parallel_threshold` configuration
fn batch_threshold_tuning(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_threshold_tuning");

    let file_count = 15; // Around the default threshold

    let config = TestFileConfig {
        count: file_count,
        avg_size: 5_000,
        complexity: Complexity::Medium,
    };

    let (_temp_dir, files) = generate_test_files(&config);

    for threshold in [1, 5, 10, 20, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(threshold),
            &threshold,
            |b, &threshold| {
                b.iter(|| {
                    use hedl_cli::batch::{BatchConfig, BatchExecutor, ValidationOperation};

                    let processor = BatchExecutor::new(BatchConfig {
                        parallel_threshold: threshold,
                        ..Default::default()
                    });
                    let operation = ValidationOperation { strict: false };

                    processor
                        .process(black_box(&files), operation, false)
                        .expect("Validation should succeed");
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(
    batch_validation_benches,
    batch_validate_scaling,
    batch_validate_strict,
);

criterion_group!(
    batch_format_benches,
    batch_format_scaling,
    batch_format_with_counts,
    batch_format_check,
);

criterion_group!(batch_lint_benches, batch_lint_scaling,);

criterion_group!(
    batch_analysis_benches,
    batch_file_size_impact,
    batch_parallel_efficiency,
    batch_operation_comparison,
    batch_complexity_impact,
    batch_threshold_tuning,
);

criterion_main!(
    batch_validation_benches,
    batch_format_benches,
    batch_lint_benches,
    batch_analysis_benches,
);

// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Benchmarks for batch processing strategies.
//!
//! Compares serial vs parallel processing at different scales.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use hedl_cli::batch::{BatchConfig, BatchOperation, BatchProcessor};
use hedl_cli::error::CliError;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// Mock operation for benchmarking
struct MockOperation;

impl BatchOperation for MockOperation {
    type Output = String;

    fn process_file(&self, path: &Path) -> Result<Self::Output, CliError> {
        // Simulate lightweight processing
        Ok(path.to_string_lossy().to_string())
    }

    fn name(&self) -> &str {
        "mock"
    }
}

fn create_test_files(count: usize) -> (TempDir, Vec<PathBuf>) {
    let temp_dir = TempDir::new().unwrap();
    let mut paths = Vec::new();

    for i in 0..count {
        let path = temp_dir.path().join(format!("file{}.hedl", i));
        std::fs::write(&path, format!("test content {}", i)).unwrap();
        paths.push(path);
    }

    (temp_dir, paths)
}

fn bench_serial_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("serial_processing");

    for size in [10, 50, 100, 500] {
        let (_temp_dir, paths) = create_test_files(size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            let processor = BatchProcessor::new(BatchConfig {
                parallel_threshold: usize::MAX, // Force serial
                ..Default::default()
            });

            b.iter(|| {
                processor
                    .process(black_box(&paths), MockOperation, false)
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_parallel_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_processing");

    for size in [10, 50, 100, 500] {
        let (_temp_dir, paths) = create_test_files(size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            let processor = BatchProcessor::new(BatchConfig {
                parallel_threshold: 1, // Force parallel
                ..Default::default()
            });

            b.iter(|| {
                processor
                    .process(black_box(&paths), MockOperation, false)
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_strategy_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategy_comparison");

    for size in [10, 50, 100, 500, 1000] {
        let (_temp_dir, paths) = create_test_files(size);

        // Serial
        group.bench_with_input(
            BenchmarkId::new("serial", size),
            &size,
            |b, _| {
                let processor = BatchProcessor::new(BatchConfig {
                    parallel_threshold: usize::MAX,
                    ..Default::default()
                });

                b.iter(|| {
                    processor
                        .process(black_box(&paths), MockOperation, false)
                        .unwrap()
                });
            },
        );

        // Parallel
        group.bench_with_input(
            BenchmarkId::new("parallel", size),
            &size,
            |b, _| {
                let processor = BatchProcessor::new(BatchConfig {
                    parallel_threshold: 1,
                    ..Default::default()
                });

                b.iter(|| {
                    processor
                        .process(black_box(&paths), MockOperation, false)
                        .unwrap()
                });
            },
        );

        // Auto (uses default threshold of 10)
        group.bench_with_input(
            BenchmarkId::new("auto", size),
            &size,
            |b, _| {
                let processor = BatchProcessor::new(BatchConfig::default());

                b.iter(|| {
                    processor
                        .process(black_box(&paths), MockOperation, false)
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

fn bench_threshold_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("threshold_optimization");

    let size = 100;
    let (_temp_dir, paths) = create_test_files(size);

    for threshold in [1, 5, 10, 20, 50, 100] {
        group.bench_with_input(
            BenchmarkId::from_parameter(threshold),
            &threshold,
            |b, &threshold| {
                let processor = BatchProcessor::new(BatchConfig {
                    parallel_threshold: threshold,
                    ..Default::default()
                });

                b.iter(|| {
                    processor
                        .process(black_box(&paths), MockOperation, false)
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_serial_processing,
    bench_parallel_processing,
    bench_strategy_comparison,
    bench_threshold_optimization
);
criterion_main!(benches);

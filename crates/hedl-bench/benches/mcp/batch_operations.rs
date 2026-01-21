// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Benchmarks for batch operation performance.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use hedl_mcp::batch::{BatchMode, BatchOperation, BatchRequest};
use hedl_mcp::{BatchExecutor, OperationCache};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;

fn create_validation_batch(size: usize) -> BatchRequest {
    let operations: Vec<_> = (0..size)
        .map(|i| BatchOperation {
            id: format!("val_{}", i),
            tool: "hedl_validate".to_string(),
            arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
            depends_on: vec![],
        })
        .collect();

    BatchRequest {
        operations,
        mode: BatchMode::ContinueOnError,
        parallel: true,
        transaction: false,
        timeout: None,
    }
}

fn bench_batch_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_execution");

    for size in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("serial", size), size, |b, &size| {
            let executor = BatchExecutor::new(Path::new("."), None);
            b.iter(|| {
                let mut batch = create_validation_batch(size);
                batch.parallel = false;
                executor.execute(black_box(batch)).unwrap()
            });
        });

        group.bench_with_input(BenchmarkId::new("parallel", size), size, |b, &size| {
            let executor = BatchExecutor::new(Path::new("."), None);
            b.iter(|| {
                let batch = create_validation_batch(size);
                executor.execute(black_box(batch)).unwrap()
            });
        });

        group.bench_with_input(BenchmarkId::new("with_cache", size), size, |b, &size| {
            let cache = Arc::new(OperationCache::new(1000));
            let executor = BatchExecutor::new(Path::new("."), Some(cache));
            b.iter(|| {
                let batch = create_validation_batch(size);
                executor.execute(black_box(batch)).unwrap()
            });
        });
    }

    group.finish();
}

fn bench_dependency_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("dependency_resolution");

    for depth in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("linear_chain", depth),
            depth,
            |b, &depth| {
                let executor = BatchExecutor::new(Path::new("."), None);
                b.iter(|| {
                    let mut operations = vec![BatchOperation {
                        id: "op_0".to_string(),
                        tool: "hedl_validate".to_string(),
                        arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                        depends_on: vec![],
                    }];

                    for i in 1..depth {
                        operations.push(BatchOperation {
                            id: format!("op_{}", i),
                            tool: "hedl_validate".to_string(),
                            arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                            depends_on: vec![format!("op_{}", i - 1)],
                        });
                    }

                    let batch = BatchRequest {
                        operations,
                        mode: BatchMode::ContinueOnError,
                        parallel: false,
                        transaction: false,
                        timeout: None,
                    };

                    executor.execute(black_box(batch)).unwrap()
                });
            },
        );
    }

    group.finish();
}

fn bench_batch_vs_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_vs_sequential");
    let size = 100;

    group.throughput(Throughput::Elements(size as u64));

    group.bench_function("batch_parallel", |b| {
        let executor = BatchExecutor::new(Path::new("."), None);
        b.iter(|| {
            let batch = create_validation_batch(size);
            executor.execute(black_box(batch)).unwrap()
        });
    });

    group.bench_function("batch_serial", |b| {
        let executor = BatchExecutor::new(Path::new("."), None);
        b.iter(|| {
            let mut batch = create_validation_batch(size);
            batch.parallel = false;
            executor.execute(black_box(batch)).unwrap()
        });
    });

    group.bench_function("sequential_calls", |b| {
        use hedl_mcp::tools::execute_tool;
        b.iter(|| {
            for _i in 0..size {
                let args = json!({"hedl": "%VERSION 1.0\n---"});
                execute_tool("hedl_validate", Some(args), black_box(Path::new("."))).unwrap();
            }
        });
    });

    group.finish();
}

fn bench_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");
    let size = 100;

    group.bench_function("no_cache", |b| {
        let executor = BatchExecutor::new(Path::new("."), None);
        b.iter(|| {
            let batch = create_validation_batch(size);
            executor.execute(black_box(batch)).unwrap()
        });
    });

    group.bench_function("with_cache_first_run", |b| {
        let cache = Arc::new(OperationCache::new(1000));
        let executor = BatchExecutor::new(Path::new("."), Some(cache.clone()));
        b.iter(|| {
            cache.clear();
            let batch = create_validation_batch(size);
            executor.execute(black_box(batch)).unwrap()
        });
    });

    group.bench_function("with_cache_warm", |b| {
        let cache = Arc::new(OperationCache::new(1000));
        let executor = BatchExecutor::new(Path::new("."), Some(cache.clone()));

        // Prime cache
        let batch = create_validation_batch(size);
        executor.execute(batch).unwrap();

        b.iter(|| {
            let batch = create_validation_batch(size);
            executor.execute(black_box(batch)).unwrap()
        });
    });

    group.finish();
}

fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    group.bench_function("continue_on_error", |b| {
        let executor = BatchExecutor::new(Path::new("."), None);
        b.iter(|| {
            let operations = vec![
                BatchOperation {
                    id: "val1".to_string(),
                    tool: "hedl_validate".to_string(),
                    arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                    depends_on: vec![],
                },
                BatchOperation {
                    id: "invalid".to_string(),
                    tool: "hedl_validate".to_string(),
                    arguments: Some(json!({"hedl": "invalid"})),
                    depends_on: vec![],
                },
                BatchOperation {
                    id: "val2".to_string(),
                    tool: "hedl_validate".to_string(),
                    arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                    depends_on: vec![],
                },
            ];

            let batch = BatchRequest {
                operations,
                mode: BatchMode::ContinueOnError,
                parallel: false,
                transaction: false,
                timeout: None,
            };

            executor.execute(black_box(batch)).unwrap()
        });
    });

    group.bench_function("stop_on_error", |b| {
        let executor = BatchExecutor::new(Path::new("."), None);
        b.iter(|| {
            let operations = vec![
                BatchOperation {
                    id: "val1".to_string(),
                    tool: "hedl_validate".to_string(),
                    arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                    depends_on: vec![],
                },
                BatchOperation {
                    id: "invalid".to_string(),
                    tool: "hedl_validate".to_string(),
                    arguments: Some(json!({"hedl": "invalid"})),
                    depends_on: vec![],
                },
                BatchOperation {
                    id: "val2".to_string(),
                    tool: "hedl_validate".to_string(),
                    arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                    depends_on: vec![],
                },
            ];

            let batch = BatchRequest {
                operations,
                mode: BatchMode::StopOnError,
                parallel: false,
                transaction: false,
                timeout: None,
            };

            executor.execute(black_box(batch)).unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_batch_execution,
    bench_dependency_resolution,
    bench_batch_vs_sequential,
    bench_cache_performance,
    bench_error_handling
);
criterion_main!(benches);

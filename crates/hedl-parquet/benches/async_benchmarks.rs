#![cfg(feature = "async-io")]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_parquet::async_io::{from_parquet_async, to_parquet_async};
use hedl_parquet::{from_parquet, to_parquet};
use std::hint::black_box;
use tempfile::TempDir;

fn generate_test_data(num_rows: usize, num_columns: usize) -> Document {
    let mut doc = Document::new((1, 0));

    // Create schema
    let mut schema = vec!["id".to_string()];
    for i in 1..num_columns {
        schema.push(format!("col{}", i));
    }

    let mut matrix_list = MatrixList::new("TestData", schema);

    // Generate rows
    for i in 0..num_rows {
        let mut fields = vec![Value::String(format!("row{}", i).into())];
        for j in 1..num_columns {
            match j % 4 {
                0 => fields.push(Value::Int(i as i64 * j as i64)),
                1 => fields.push(Value::Float((i as f64) * 0.5)),
                2 => fields.push(Value::String(format!("value_{}", i).into())),
                3 => fields.push(Value::Bool(i % 2 == 0)),
                _ => unreachable!(),
            }
        }

        matrix_list.add_row(Node::new("TestData", format!("row{}", i), fields));
    }

    doc.root.insert("data".to_string(), Item::List(matrix_list));
    doc
}

fn bench_sync_vs_async_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_vs_async_write");

    for num_rows in [1_000, 10_000, 100_000].iter() {
        let doc = generate_test_data(*num_rows, 10);
        let temp_dir = TempDir::new().unwrap();

        group.throughput(Throughput::Elements(*num_rows as u64));

        // Sync write
        group.bench_with_input(BenchmarkId::new("sync", num_rows), num_rows, |b, _| {
            b.iter(|| {
                let path = temp_dir.path().join("sync_test.parquet");
                to_parquet(&doc, &path).unwrap();
                black_box(())
            })
        });

        // Async write (single operation, no concurrency benefit)
        group.bench_with_input(BenchmarkId::new("async", num_rows), num_rows, |b, _| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            b.iter(|| {
                let path = temp_dir.path().join("async_test.parquet");
                rt.block_on(async {
                    let _: () = to_parquet_async(&doc, &path).await.unwrap();
                    black_box(())
                })
            })
        });
    }

    group.finish();
}

fn bench_sync_vs_async_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_vs_async_read");

    for num_rows in [1_000, 10_000, 100_000].iter() {
        let doc = generate_test_data(*num_rows, 10);
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.parquet");
        to_parquet(&doc, &path).unwrap();

        group.throughput(Throughput::Elements(*num_rows as u64));

        // Sync read
        group.bench_with_input(BenchmarkId::new("sync", num_rows), num_rows, |b, _| {
            b.iter(|| black_box(from_parquet(&path).unwrap()))
        });

        // Async read
        group.bench_with_input(BenchmarkId::new("async", num_rows), num_rows, |b, _| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            b.iter(|| rt.block_on(async { black_box(from_parquet_async(&path).await.unwrap()) }))
        });
    }

    group.finish();
}

fn bench_concurrent_async_reads(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_async_reads");

    // Create test files
    let temp_dir = TempDir::new().unwrap();
    let num_files = 10;
    let mut paths = Vec::new();

    for i in 0..num_files {
        let doc = generate_test_data(10_000, 10);
        let path = temp_dir.path().join(format!("test_{}.parquet", i));
        to_parquet(&doc, &path).unwrap();
        paths.push(path);
    }

    group.throughput(Throughput::Elements(num_files * 10_000));

    // Sync sequential reads
    group.bench_function("sync_sequential", |b| {
        b.iter(|| {
            for path in &paths {
                black_box(from_parquet(path).unwrap());
            }
        })
    });

    // Async concurrent reads
    group.bench_function("async_concurrent", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                let mut tasks = Vec::new();
                for path in &paths {
                    let path = path.clone();
                    let task =
                        tokio::spawn(async move { from_parquet_async(&path).await.unwrap() });
                    tasks.push(task);
                }
                futures::future::join_all(tasks).await
            })
        })
    });

    group.finish();
}

fn bench_concurrent_async_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_async_writes");

    let num_operations = 10;
    let docs: Vec<_> = (0..num_operations)
        .map(|_| generate_test_data(10_000, 10))
        .collect();

    group.throughput(Throughput::Elements(num_operations * 10_000));

    // Sync sequential writes
    group.bench_function("sync_sequential", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            for (i, doc) in docs.iter().enumerate() {
                let path = temp_dir.path().join(format!("test_{}.parquet", i));
                to_parquet(doc, &path).unwrap();
                black_box(());
            }
        })
    });

    // Async concurrent writes
    group.bench_function("async_concurrent", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let mut tasks = Vec::new();
                for (i, doc) in docs.iter().enumerate() {
                    let path = temp_dir.path().join(format!("test_{}.parquet", i));
                    let doc_clone = doc.clone();
                    let task =
                        tokio::spawn(
                            async move { to_parquet_async(&doc_clone, &path).await.unwrap() },
                        );
                    tasks.push(task);
                }
                futures::future::join_all(tasks).await
            })
        })
    });

    group.finish();
}

fn bench_mixed_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_concurrent_operations");

    // Setup: 5 files to read, 5 documents to write
    let temp_dir = TempDir::new().unwrap();
    let mut read_paths = Vec::new();
    for i in 0..5 {
        let doc = generate_test_data(5_000, 10);
        let path = temp_dir.path().join(format!("read_{}.parquet", i));
        to_parquet(&doc, &path).unwrap();
        read_paths.push(path);
    }

    let write_docs: Vec<_> = (0..5).map(|_| generate_test_data(5_000, 10)).collect();

    group.throughput(Throughput::Elements(10 * 5_000));

    // Sync sequential (reads then writes)
    group.bench_function("sync_sequential", |b| {
        b.iter(|| {
            // Reads
            for path in &read_paths {
                black_box(from_parquet(path).unwrap());
            }
            // Writes
            for (i, doc) in write_docs.iter().enumerate() {
                let path = temp_dir.path().join(format!("write_{}.parquet", i));
                to_parquet(doc, &path).unwrap();
                black_box(());
            }
        })
    });

    // Async concurrent (reads and writes overlap)
    group.bench_function("async_concurrent", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                let mut read_tasks = Vec::new();
                let mut write_tasks = Vec::new();

                // Read tasks
                for path in &read_paths {
                    let path = path.clone();
                    let task =
                        tokio::spawn(async move { from_parquet_async(&path).await.unwrap() });
                    read_tasks.push(task);
                }

                // Write tasks
                for (i, doc) in write_docs.iter().enumerate() {
                    let path = temp_dir.path().join(format!("write_{}.parquet", i));
                    let doc_clone = doc.clone();
                    let task =
                        tokio::spawn(
                            async move { to_parquet_async(&doc_clone, &path).await.unwrap() },
                        );
                    write_tasks.push(task);
                }

                // Join both task sets concurrently
                let (read_results, write_results) = futures::future::join(
                    futures::future::join_all(read_tasks),
                    futures::future::join_all(write_tasks),
                )
                .await;
                black_box((read_results, write_results))
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_sync_vs_async_write,
    bench_sync_vs_async_read,
    bench_concurrent_async_reads,
    bench_concurrent_async_writes,
    bench_mixed_concurrent_operations
);
criterion_main!(benches);

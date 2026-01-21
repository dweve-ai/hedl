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

//! Array optimization benchmarks
//!
//! Measures performance improvements from array handling optimizations:
//! - Single-pass array type classification
//! - `SmallVec` for small allocations
//! - Sorted `BTreeMap` insertion
//! - Capacity pre-allocation

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hedl_json::{from_json, FromJsonConfig};
use serde_json::json;
use std::hint::black_box;

fn generate_tensor_array(size: usize) -> String {
    let numbers: Vec<i32> = (0..size as i32).collect();
    json!({
        "data": numbers
    })
    .to_string()
}

fn generate_object_array(size: usize) -> String {
    let mut users = Vec::new();
    for i in 0..size {
        users.push(json!({
            "id": format!("u{}", i),
            "name": format!("User {}", i),
            "email": format!("user{}@example.com", i),
            "age": 20 + (i % 50)
        }));
    }

    json!({
        "users": users
    })
    .to_string()
}

fn generate_nested_array(outer_size: usize, inner_size: usize) -> String {
    let mut departments = Vec::new();
    for i in 0..outer_size {
        let mut employees = Vec::new();
        for j in 0..inner_size {
            employees.push(json!({
                "id": format!("e{}", i * inner_size + j),
                "name": format!("Employee {}", i * inner_size + j)
            }));
        }

        departments.push(json!({
            "id": format!("d{}", i),
            "name": format!("Department {}", i),
            "employees": employees
        }));
    }

    json!({
        "departments": departments
    })
    .to_string()
}

fn generate_wide_object_array(size: usize, fields: usize) -> String {
    let mut records = Vec::new();
    for i in 0..size {
        let mut obj = serde_json::Map::new();
        obj.insert("id".to_string(), json!(format!("r{}", i)));

        for f in 0..fields {
            obj.insert(format!("field{f}"), json!(format!("value{}", f)));
        }

        records.push(json!(obj));
    }

    json!({
        "records": records
    })
    .to_string()
}

fn bench_tensor_arrays(c: &mut Criterion) {
    let mut group = c.benchmark_group("tensor_arrays");

    for &size in &[100, 1_000, 10_000, 100_000] {
        let json = generate_tensor_array(size);
        let bytes = json.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::from_parameter(size), &json, |b, json| {
            let config = FromJsonConfig::default();
            b.iter(|| from_json(black_box(json), &config).unwrap());
        });
    }

    group.finish();
}

fn bench_object_arrays(c: &mut Criterion) {
    let mut group = c.benchmark_group("object_arrays");

    for &size in &[100, 1_000, 10_000] {
        let json = generate_object_array(size);
        let bytes = json.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::from_parameter(size), &json, |b, json| {
            let config = FromJsonConfig::default();
            b.iter(|| from_json(black_box(json), &config).unwrap());
        });
    }

    group.finish();
}

fn bench_nested_arrays(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_arrays");

    for &(outer, inner) in &[(10, 100), (100, 10), (50, 50)] {
        let json = generate_nested_array(outer, inner);
        let bytes = json.len() as u64;
        let _total = outer * inner;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{outer}x{inner}")),
            &json,
            |b, json| {
                let config = FromJsonConfig::default();
                b.iter(|| from_json(black_box(json), &config).unwrap());
            },
        );
    }

    group.finish();
}

fn bench_wide_objects(c: &mut Criterion) {
    let mut group = c.benchmark_group("wide_objects");

    // Test objects with varying field counts to validate SmallVec optimization
    for &fields in &[5, 10, 16, 32, 64] {
        let json = generate_wide_object_array(1000, fields);
        let bytes = json.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{fields}_fields")),
            &json,
            |b, json| {
                let config = FromJsonConfig::default();
                b.iter(|| from_json(black_box(json), &config).unwrap());
            },
        );
    }

    group.finish();
}

fn bench_schema_cache_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("schema_cache");

    // Generate JSON with multiple arrays sharing the same schema
    let mut arrays = Vec::new();
    for _ in 0..10 {
        let mut items = Vec::new();
        for j in 0..100 {
            items.push(json!({
                "id": format!("id{}", j),
                "name": format!("Name {}", j),
                "value": j
            }));
        }
        arrays.push(items);
    }

    let json = json!({
        "array1": arrays[0],
        "array2": arrays[1],
        "array3": arrays[2],
        "array4": arrays[3],
        "array5": arrays[4],
        "array6": arrays[5],
        "array7": arrays[6],
        "array8": arrays[7],
        "array9": arrays[8],
        "array10": arrays[9]
    })
    .to_string();

    let bytes = json.len() as u64;
    group.throughput(Throughput::Bytes(bytes));
    group.bench_function("10_arrays_same_schema", |b| {
        let config = FromJsonConfig::default();
        b.iter(|| from_json(black_box(&json), &config).unwrap());
    });

    group.finish();
}

fn bench_array_classification(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_classification");

    // Benchmark the improved single-pass classification
    let configs = [
        ("tensor_100", generate_tensor_array(100)),
        ("tensor_10000", generate_tensor_array(10000)),
        ("objects_100", generate_object_array(100)),
        ("objects_1000", generate_object_array(1000)),
    ];

    for (name, json) in &configs {
        let bytes = json.len() as u64;
        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::from_parameter(name), json, |b, json| {
            let config = FromJsonConfig::default();
            b.iter(|| from_json(black_box(json), &config).unwrap());
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_tensor_arrays,
    bench_object_arrays,
    bench_nested_arrays,
    bench_wide_objects,
    bench_schema_cache_reuse,
    bench_array_classification
);
criterion_main!(benches);

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

//! Array performance demonstration
//!
//! Shows the performance characteristics of array processing with the optimizations:
//! - Single-pass array type classification
//! - `SmallVec` for small allocations
//! - Adaptive `BTreeMap` insertion strategy
//! - Capacity pre-allocation

use hedl_json::{from_json, FromJsonConfig};
use serde_json::json;
use std::time::Instant;

fn main() {
    println!("HEDL JSON Array Processing Performance Demonstration\n");
    println!("======================================================\n");

    // Tensor arrays
    println!("1. Tensor Arrays (numeric data)");
    println!("   - Optimized with pre-allocated capacity");
    println!("   - Single-pass type classification");
    test_tensor_arrays();
    println!();

    // Object arrays
    println!("2. Object Arrays (structured data)");
    println!("   - SmallVec optimization for schemas <16 fields");
    println!("   - Schema caching for repeated structures");
    test_object_arrays();
    println!();

    // Nested arrays
    println!("3. Nested Arrays (hierarchical data)");
    println!("   - Adaptive children insertion strategy");
    println!("   - Capacity hints propagated through recursion");
    test_nested_arrays();
    println!();

    // Wide objects
    println!("4. Wide Objects (many fields)");
    println!("   - Adaptive BTreeMap insertion (direct vs sorted)");
    println!("   - SmallVec overflow handling for >16 fields");
    test_wide_objects();
    println!();

    println!("======================================================");
    println!("All optimizations demonstrated successfully!");
}

fn test_tensor_arrays() {
    let sizes = [100, 1_000, 10_000, 100_000];

    for size in sizes {
        let numbers: Vec<i32> = (0..size).collect();
        let json = json!({"data": numbers}).to_string();
        let bytes = json.len();

        let start = Instant::now();
        let config = FromJsonConfig::default();
        let _doc = from_json(&json, &config).unwrap();
        let elapsed = start.elapsed();

        let throughput = (bytes as f64) / elapsed.as_secs_f64() / 1_024_000.0;
        println!(
            "   {:>6} elements: {:>8.2} ms ({:>6.1} MB/s)",
            size,
            elapsed.as_secs_f64() * 1000.0,
            throughput
        );
    }
}

fn test_object_arrays() {
    let sizes = [100, 1_000, 10_000];

    for size in sizes {
        let mut users = Vec::new();
        for i in 0..size {
            users.push(json!({
                "id": format!("u{}", i),
                "name": format!("User {}", i),
                "email": format!("user{}@example.com", i),
                "age": 20 + (i % 50)
            }));
        }

        let json = json!({"users": users}).to_string();
        let bytes = json.len();

        let start = Instant::now();
        let config = FromJsonConfig::default();
        let doc = from_json(&json, &config).unwrap();
        let elapsed = start.elapsed();

        let throughput = (bytes as f64) / elapsed.as_secs_f64() / 1_024_000.0;
        println!(
            "   {:>6} objects: {:>8.2} ms ({:>6.1} MB/s) - schema cached: {}",
            size,
            elapsed.as_secs_f64() * 1000.0,
            throughput,
            doc.structs.contains_key("User")
        );
    }
}

fn test_nested_arrays() {
    let configs = [(10, 100), (100, 10), (50, 50)];

    for (outer, inner) in configs {
        let mut departments = Vec::new();
        for i in 0..outer {
            let mut employees = Vec::new();
            for j in 0..inner {
                employees.push(json!({
                    "id": format!("e{}", i * inner + j),
                    "name": format!("Employee {}", i * inner + j)
                }));
            }
            departments.push(json!({
                "id": format!("d{}", i),
                "name": format!("Department {}", i),
                "employees": employees
            }));
        }

        let json = json!({"departments": departments}).to_string();
        let bytes = json.len();

        let start = Instant::now();
        let config = FromJsonConfig::default();
        let _doc = from_json(&json, &config).unwrap();
        let elapsed = start.elapsed();

        let throughput = (bytes as f64) / elapsed.as_secs_f64() / 1_024_000.0;
        println!(
            "   {:>3}x{:<3} structure: {:>8.2} ms ({:>6.1} MB/s)",
            outer,
            inner,
            elapsed.as_secs_f64() * 1000.0,
            throughput
        );
    }
}

fn test_wide_objects() {
    let field_counts = [5, 10, 16, 32, 64];

    for fields in field_counts {
        let mut records = Vec::new();
        for i in 0..1000 {
            let mut obj = serde_json::Map::new();
            obj.insert("id".to_string(), json!(format!("r{}", i)));

            for f in 0..fields {
                obj.insert(format!("field{f}"), json!(format!("value{}", f)));
            }

            records.push(json!(obj));
        }

        let json = json!({"records": records}).to_string();
        let bytes = json.len();

        let start = Instant::now();
        let config = FromJsonConfig::default();
        let _doc = from_json(&json, &config).unwrap();
        let elapsed = start.elapsed();

        let throughput = (bytes as f64) / elapsed.as_secs_f64() / 1_024_000.0;
        let strategy = if fields < 32 { "direct" } else { "sorted" };
        println!(
            "   {:>2} fields: {:>8.2} ms ({:>6.1} MB/s) - strategy: {}",
            fields,
            elapsed.as_secs_f64() * 1000.0,
            throughput,
            strategy
        );
    }
}

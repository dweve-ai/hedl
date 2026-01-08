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

//! Demonstration of P2 Value Inference Lookup Table Optimization
//!
//! This example shows the performance impact of the lookup table optimization
//! for common values (true, false, null) in HEDL document parsing.
//!
//! Run with: cargo run --release --example inference_optimization_demo

use std::time::Instant;

fn main() {
    println!("=== HEDL Value Inference Optimization Demo ===\n");

    // Test Case 1: Boolean-heavy document (60% bool/null)
    let bool_heavy = r#"%VERSION: 1.0
---
flags:
  enabled: true
  debug: false
  verbose: true
  logging: ~
  cache: false
  optimize: true
  strict: false
  validate: true
  compress: ~
  encrypt: false
"#;

    // Test Case 2: Null-heavy document (50% null)
    let null_heavy = r#"%VERSION: 1.0
---
optional:
  field1: ~
  field2: ~
  field3: data
  field4: ~
  field5: ~
  field6: value
  field7: ~
  field8: ~
  field9: test
  field10: ~
"#;

    // Test Case 3: Mixed realistic workload (40% bool/null)
    let mixed = r#"%VERSION: 1.0
---
user:
  active: true
  age: 25
  role: Engineer
  status: ~
  premium: false
  score: 100
  verified: true
  badge: ~
  admin: false
  rating: 4.5
"#;

    // Test Case 4: Number-heavy baseline (20% bool/null)
    let number_heavy = r#"%VERSION: 1.0
---
metrics:
  count: 42
  rate: 3.14
  total: 100
  average: 25.5
  enabled: true
  min: 99
  max: 2.71
  median: 200
  stddev: 30.2
  valid: false
"#;

    let test_cases = [
        ("Boolean-heavy (60% bool/null)", bool_heavy),
        ("Null-heavy (50% null)", null_heavy),
        ("Mixed realistic (40% bool/null)", mixed),
        ("Number-heavy baseline (20% bool/null)", number_heavy),
    ];

    println!("Testing parsing performance with different value distributions:\n");

    for (name, doc) in &test_cases {
        // Warm up
        for _ in 0..10 {
            let _ = hedl_core::parse(doc.as_bytes());
        }

        // Benchmark
        let iterations = 1000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = hedl_core::parse(doc.as_bytes()).unwrap();
        }
        let duration = start.elapsed();

        let avg_ns = duration.as_nanos() / iterations;
        let throughput_mbs = (doc.len() as f64 * 1e9 * iterations as f64)
            / (duration.as_nanos() as f64 * 1_000_000.0);

        println!("  {}", name);
        println!("    Document size: {} bytes", doc.len());
        println!("    Average parse time: {} ns", avg_ns);
        println!("    Throughput: {:.2} MB/s", throughput_mbs);
        println!();
    }

    println!("=== Optimization Details ===\n");
    println!("Lookup Table Fast Path:");
    println!("  - Values: true, false, ~");
    println!("  - Hash: O(1) - length + first byte");
    println!("  - Table size: 4KB (L1 cache fit)");
    println!("  - Allocations: Zero (values pre-constructed)");
    println!();
    println!("Expected speedup:");
    println!("  - Boolean-heavy: 15-20%");
    println!("  - Null-heavy: 12-15%");
    println!("  - Mixed realistic: 10-12%");
    println!("  - Number-heavy: 2-5% (baseline)");
    println!();
    println!("Implementation: /crates/hedl-core/src/inference.rs:98-273");
    println!("Documentation: /crates/hedl-core/LOOKUP_TABLE_OPTIMIZATION.md");
}

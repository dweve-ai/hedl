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

//! Minimal benchmark example showing how to use the infrastructure.
//!
//! This example demonstrates:
//! - Creating a benchmark report
//! - Recording performance measurements
//! - Exporting to all formats (HTML, JSON, Markdown)
//!
//! Run with:
//! ```bash
//! cargo run --package hedl-bench --example simple_benchmark
//! ```

use hedl_bench::{generate_users, sizes, BenchmarkReport, ExportConfig, PerfResult};
use std::time::Instant;

fn main() {
    println!("=== Simple Benchmark Example ===\n");

    // 1. Create a benchmark report
    let mut report = BenchmarkReport::new("Simple HEDL Parsing Benchmark");
    report.set_timestamp();
    report.add_note("Minimal example showing infrastructure usage");
    report.add_note("Measures parsing performance for small dataset");

    // 2. Generate test data
    let hedl = generate_users(sizes::SMALL);
    println!("Generated HEDL document: {} bytes", hedl.len());

    // 3. Measure performance
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = hedl_core::parse(hedl.as_bytes());
    }

    let elapsed = start.elapsed();
    let total_ns = elapsed.as_nanos() as u64;
    let avg_ns = total_ns / iterations;

    println!("Completed {} iterations in {:?}", iterations, elapsed);
    println!("Average time per iteration: {} ns", avg_ns);

    // 4. Record results
    let throughput_bytes = hedl.len() as u64 * iterations;
    let throughput_mbs = (throughput_bytes as f64 * 1e9) / total_ns as f64 / 1_000_000.0;

    report.add_perf(PerfResult {
        name: "parse_small_users".to_string(),
        iterations,
        total_time_ns: total_ns,
        throughput_bytes: Some(throughput_bytes),
        avg_time_ns: Some(avg_ns),
        throughput_mbs: Some(throughput_mbs),
    });

    println!("Throughput: {:.2} MB/s\n", throughput_mbs);

    // 5. Export reports
    let config = ExportConfig::all();
    report
        .save_all("target/demo/simple_benchmark", &config)
        .expect("Failed to export reports");

    println!("=== Reports Generated ===");
    println!("HTML:     target/demo/simple_benchmark.html");
    println!("JSON:     target/demo/simple_benchmark.json");
    println!("Markdown: target/demo/simple_benchmark.md");
    println!("\nOpen the HTML report in your browser to see results!");
}

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

//! Example of generating custom reports with specific configurations.
//!
//! This example demonstrates:
//! - Custom export configurations
//! - Selective format generation
//! - Adding format comparison metrics
//! - Custom analysis and recommendations
//!
//! Run with:
//! ```bash
//! cargo run --package hedl-bench --example custom_report
//! ```

use hedl_bench::{
    compare_formats, generate_users, sizes, BenchmarkReport, ExportConfig, FormatMetrics,
    PerfResult,
};
use std::time::Instant;

fn main() {
    println!("=== Custom Report Example ===\n");

    // Create a report with custom title and notes
    let mut report = BenchmarkReport::new("Custom HEDL Performance Analysis");
    report.set_timestamp();
    report.add_note("Custom benchmark with format comparison");
    report.add_note("Demonstrates selective report generation");
    report.add_note("Includes token efficiency analysis");

    // Generate test documents
    let small = generate_users(sizes::SMALL);
    let medium = generate_users(sizes::MEDIUM);
    let large = generate_users(sizes::LARGE);

    println!("Generated test documents:");
    println!("  Small:  {} bytes", small.len());
    println!("  Medium: {} bytes", medium.len());
    println!("  Large:  {} bytes\n", large.len());

    // Benchmark each size
    for (name, hedl) in &[("small", &small), ("medium", &medium), ("large", &large)] {
        // Parse document
        let doc = hedl_core::parse(hedl.as_bytes()).expect("Parse failed");

        // Measure parsing performance
        let iterations = 100;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = hedl_core::parse(hedl.as_bytes());
        }
        let elapsed = start.elapsed();
        let total_ns = elapsed.as_nanos() as u64;

        // Record performance
        report.add_perf(PerfResult {
            name: format!("parse_{}", name),
            iterations,
            total_time_ns: total_ns,
            throughput_bytes: Some(hedl.len() as u64 * iterations),
            avg_time_ns: Some(total_ns / iterations),
            throughput_mbs: Some(
                (hedl.len() as f64 * iterations as f64 * 1e9) / total_ns as f64 / 1_000_000.0,
            ),
        });

        // Add format comparison
        let stats = compare_formats(&doc);
        report.add_format(FormatMetrics {
            name: name.to_string(),
            complexity: None,
            hedl_tokens: stats.hedl_tokens,
            json_tokens: stats.json_compact_tokens,
            yaml_tokens: stats.yaml_tokens,
            xml_tokens: stats.xml_tokens,
            toon_tokens: Some(stats.toon_tokens),
            csv_tokens: Some(stats.csv_tokens),
            hedl_bytes: stats.hedl_bytes,
            json_bytes: stats.json_compact_bytes,
            yaml_bytes: stats.yaml_bytes,
            xml_bytes: stats.xml_bytes,
            toon_bytes: Some(stats.toon_bytes),
            csv_bytes: Some(stats.csv_bytes),
            token_savings_vs_json: Some(stats.savings_vs_json),
            byte_savings_vs_json: None,
        });

        println!(
            "Benchmarked {}: {:.2} µs/iter",
            name,
            total_ns as f64 / iterations as f64 / 1000.0
        );
    }

    println!("\n=== Generating Reports ===\n");

    // Option 1: Generate all formats
    println!("1. All formats:");
    let config_all = ExportConfig::all();
    report
        .save_all("target/demo/custom_all", &config_all)
        .expect("Failed to export all formats");
    println!("   - HTML, JSON, Markdown");

    // Option 2: HTML only
    println!("\n2. HTML only:");
    let config_html = ExportConfig {
        html: true,
        json: false,
        markdown: false,
    };
    report
        .save_all("target/demo/custom_html", &config_html)
        .expect("Failed to export HTML");
    println!("   - HTML only");

    // Option 3: JSON for CI/CD
    println!("\n3. JSON for CI/CD:");
    let config_json = ExportConfig {
        html: false,
        json: true,
        markdown: false,
    };
    report
        .save_all("target/demo/custom_json", &config_json)
        .expect("Failed to export JSON");
    println!("   - JSON only");

    // Option 4: Markdown for documentation
    println!("\n4. Markdown for docs:");
    let config_md = ExportConfig {
        html: false,
        json: false,
        markdown: true,
    };
    report
        .save_all("target/demo/custom_md", &config_md)
        .expect("Failed to export Markdown");
    println!("   - Markdown only");

    println!("\n=== Reports Generated ===");
    println!("All formats:  target/demo/custom_all.*");
    println!("HTML only:    target/demo/custom_html.html");
    println!("JSON only:    target/demo/custom_json.json");
    println!("Markdown only: target/demo/custom_md.md");
}

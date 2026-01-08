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

//! Example of checking for performance regressions.
//!
//! This example demonstrates:
//! - Loading baseline metrics
//! - Running current benchmarks
//! - Comparing against baselines
//! - Detecting regressions and improvements
//! - Generating regression reports
//!
//! Run with:
//! ```bash
//! cargo run --package hedl-bench --example regression_check
//! ```

use hedl_bench::{generate_users, sizes};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

// ============================================================================
// Baseline Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BaselineMetrics {
    mean_ns: u64,
    std_dev_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Baseline {
    version: String,
    timestamp: String,
    benchmarks: HashMap<String, BaselineMetrics>,
}

#[derive(Debug)]
struct RegressionResult {
    name: String,
    baseline_ns: u64,
    current_ns: u64,
    change_percent: f64,
    status: Status,
}

#[derive(Debug)]
enum Status {
    Regression,
    Improvement,
    Stable,
}

// ============================================================================
// Main Example
// ============================================================================

fn main() {
    println!("=== Regression Check Example ===\n");

    // 1. Create or load baseline
    let baseline_path = Path::new("target/demo/baseline_example.json");
    let mut baseline = if baseline_path.exists() {
        println!("Loading existing baseline from {:?}", baseline_path);
        load_baseline(baseline_path)
    } else {
        println!("Creating new baseline");
        let mut b = Baseline {
            version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            benchmarks: HashMap::new(),
        };

        // Run initial benchmarks to create baseline
        for (name, size) in &[
            ("small", sizes::SMALL),
            ("medium", sizes::MEDIUM),
            ("large", sizes::LARGE),
        ] {
            let avg_ns = run_benchmark(name, *size);
            b.benchmarks.insert(
                name.to_string(),
                BaselineMetrics {
                    mean_ns: avg_ns,
                    std_dev_ns: avg_ns / 10, // Approximate
                },
            );
            println!("  {}: {} ns", name, avg_ns);
        }

        save_baseline(&b, baseline_path);
        b
    };

    println!("\n=== Running Current Benchmarks ===\n");

    // 2. Run current benchmarks
    let mut results = Vec::new();

    for (name, size) in &[
        ("small", sizes::SMALL),
        ("medium", sizes::MEDIUM),
        ("large", sizes::LARGE),
    ] {
        let current_ns = run_benchmark(name, *size);
        println!("  {}: {} ns", name, current_ns);

        // Compare against baseline
        if let Some(baseline_metrics) = baseline.benchmarks.get(*name) {
            let baseline_ns = baseline_metrics.mean_ns;
            let change_percent =
                ((current_ns as f64 - baseline_ns as f64) / baseline_ns as f64) * 100.0;

            let status = if change_percent > 5.0 {
                Status::Regression
            } else if change_percent < -5.0 {
                Status::Improvement
            } else {
                Status::Stable
            };

            results.push(RegressionResult {
                name: name.to_string(),
                baseline_ns,
                current_ns,
                change_percent,
                status,
            });
        }
    }

    // 3. Analyze results
    println!("\n=== Regression Analysis ===\n");

    let mut has_regressions = false;
    let mut has_improvements = false;

    for result in &results {
        let status_str = match result.status {
            Status::Regression => {
                has_regressions = true;
                "REGRESSION"
            }
            Status::Improvement => {
                has_improvements = true;
                "IMPROVEMENT"
            }
            Status::Stable => "STABLE",
        };

        println!(
            "{:12} {}: {:+.2}% (baseline: {} ns, current: {} ns)",
            status_str, result.name, result.change_percent, result.baseline_ns, result.current_ns
        );
    }

    // 4. Summary
    println!("\n=== Summary ===\n");

    if has_regressions {
        println!("WARNING: Performance regressions detected!");
        println!("Consider investigating the changes that caused slowdowns.");
    } else if has_improvements {
        println!("SUCCESS: Performance improvements detected!");
        println!("Consider updating baseline to capture improvements.");
    } else {
        println!("STABLE: Performance is consistent with baseline.");
    }

    // 5. Optional: Update baseline with current results
    if has_improvements && !has_regressions {
        println!("\nUpdating baseline with improvements...");
        for result in &results {
            baseline.benchmarks.insert(
                result.name.clone(),
                BaselineMetrics {
                    mean_ns: result.current_ns,
                    std_dev_ns: result.current_ns / 10,
                },
            );
        }
        baseline.timestamp = chrono::Utc::now().to_rfc3339();
        save_baseline(&baseline, baseline_path);
        println!("Baseline updated: {:?}", baseline_path);
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn run_benchmark(name: &str, size: usize) -> u64 {
    let hedl = generate_users(size);
    let iterations = 100;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = hedl_core::parse(hedl.as_bytes());
    }
    let elapsed = start.elapsed();

    elapsed.as_nanos() as u64 / iterations
}

fn load_baseline(path: &Path) -> Baseline {
    let contents = fs::read_to_string(path).expect("Failed to read baseline");
    serde_json::from_str(&contents).expect("Failed to parse baseline")
}

fn save_baseline(baseline: &Baseline, path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let contents = serde_json::to_string_pretty(baseline).expect("Failed to serialize baseline");
    fs::write(path, contents).expect("Failed to write baseline");
}

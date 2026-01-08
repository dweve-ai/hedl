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

//! Console reporter for benchmark results.
//!
//! Formats and prints benchmark reports to the console.

use crate::harness::Regression;
use crate::reporters::types::BenchmarkReport;

/// Prints the full benchmark report to console.
pub fn print_report(report: &BenchmarkReport) {
    println!("\n{}", "=".repeat(80));
    println!("BENCHMARK REPORT: {}", report.title);
    println!("{}", "=".repeat(80));
    println!("Timestamp: {}", report.timestamp);
    println!("Results: {}", report.result_count());

    if !report.notes.is_empty() {
        println!("\nNotes:");
        for note in &report.notes {
            println!("  - {}", note);
        }
    }

    println!("\n{}", "-".repeat(80));
    println!("RESULTS:");
    println!("{}", "-".repeat(80));

    for result in &report.results {
        let size_str = result
            .size
            .map(|s| format!(" (size: {})", s))
            .unwrap_or_default();
        println!(
            "{}{}: {:?} ({} iterations)",
            result.name,
            size_str,
            result.avg_duration(),
            result.iterations
        );

        if let Some(throughput) = result.throughput_mbs() {
            println!("  Throughput: {:.2} MB/s", throughput);
        }
    }

    if !report.analysis.bottlenecks.is_empty() {
        println!("\n{}", "-".repeat(80));
        println!("BOTTLENECKS:");
        println!("{}", "-".repeat(80));

        for bottleneck in &report.analysis.bottlenecks {
            println!(
                "[{}] {}: {}",
                bottleneck.severity.as_str().to_uppercase(),
                bottleneck.location,
                bottleneck.description
            );
            if bottleneck.impact_pct > 0.0 {
                println!("  Impact: {:.1}% of total time", bottleneck.impact_pct);
            }
        }
    }

    if !report.analysis.regressions.is_empty() {
        print_regressions(&report.analysis.regressions);
    }

    if !report.recommendations.is_empty() {
        println!("\n{}", "-".repeat(80));
        println!("RECOMMENDATIONS:");
        println!("{}", "-".repeat(80));

        for (i, rec) in report.recommendations.iter().enumerate() {
            println!(
                "{}. [{}] {}",
                i + 1,
                rec.severity.as_str().to_uppercase(),
                rec.message
            );
            println!(
                "   Estimated impact: {:.1}% improvement, {:.1} hours effort, {:.0}% confidence",
                rec.impact.improvement_pct,
                rec.impact.effort_hours,
                rec.impact.confidence * 100.0
            );
        }
    }

    println!("{}\n", "=".repeat(80));
}

/// Prints a summary of the benchmark report.
pub fn print_summary(report: &BenchmarkReport) {
    println!("\n{}", "=".repeat(60));
    println!("SUMMARY: {}", report.title);
    println!("{}", "=".repeat(60));
    println!("Benchmarks: {}", report.result_count());
    println!("Bottlenecks: {}", report.analysis.bottlenecks.len());
    println!("Regressions: {}", report.analysis.regressions.len());
    println!(
        "High-priority recommendations: {}",
        report.high_priority_count()
    );
    println!("{}\n", "=".repeat(60));
}

/// Prints regression details.
pub fn print_regressions(regressions: &[Regression]) {
    println!("\n{}", "-".repeat(80));
    println!("REGRESSIONS:");
    println!("{}", "-".repeat(80));

    for regression in regressions {
        println!(
            "[{}] {}: {}% slower than baseline",
            regression.status.severity().to_uppercase(),
            regression.name,
            regression.status.percentage()
        );
        println!(
            "  Current: {:.2}ms, Baseline: {:.2}ms",
            regression.current_ns as f64 / 1_000_000.0,
            regression.baseline_ns as f64 / 1_000_000.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_summary() {
        let report = BenchmarkReport::new("Test");
        print_summary(&report);
        // Visual test - just ensure it doesn't panic
    }
}

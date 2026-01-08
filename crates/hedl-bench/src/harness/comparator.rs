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

//! Benchmark comparison and regression detection.
//!
//! Compares benchmark results against baselines and identifies regressions.

use crate::core::{check_regression, Baseline, RegressionStatus};
use crate::report::BenchmarkReport;

/// Comparison result between current and baseline.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Comparison {
    /// Benchmark name.
    pub name: String,
    /// Current duration in nanoseconds.
    pub current_ns: u64,
    /// Baseline duration in nanoseconds.
    pub baseline_ns: u64,
    /// Regression status.
    pub status: RegressionStatus,
    /// Percentage change (positive = slower, negative = faster).
    pub change_pct: f64,
}

/// Regression detected in a benchmark.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Regression {
    /// Benchmark name.
    pub name: String,
    /// Regression status and severity.
    pub status: RegressionStatus,
    /// Current duration in nanoseconds.
    pub current_ns: u64,
    /// Baseline duration in nanoseconds.
    pub baseline_ns: u64,
}

/// Format comparison result.
#[derive(Debug, Clone)]
pub struct FormatResult {
    /// Format name (e.g., "JSON", "YAML").
    pub format: String,
    /// Parse time in nanoseconds.
    pub parse_ns: u64,
    /// Serialize time in nanoseconds.
    pub serialize_ns: u64,
    /// Size in bytes.
    pub size_bytes: usize,
}

/// Comparison between different formats.
#[derive(Debug, Clone)]
pub struct FormatComparison {
    /// Results per format.
    pub results: Vec<FormatResult>,
    /// Fastest format for parsing.
    pub fastest_parse: String,
    /// Fastest format for serialization.
    pub fastest_serialize: String,
    /// Most compact format.
    pub most_compact: String,
}

/// Compares benchmark results to baseline.
///
/// # Arguments
///
/// * `results` - Current benchmark results
/// * `baseline` - Baseline to compare against
///
/// # Returns
///
/// Vector of comparisons for each benchmark.
pub fn compare_to_baseline(results: &BenchmarkReport, baseline: &Baseline) -> Vec<Comparison> {
    let mut comparisons = Vec::new();

    for perf_result in &results.perf_results {
        if let Some(baseline_bench) = baseline.get_benchmark(&perf_result.name) {
            let current_ns = perf_result.avg_time_ns.unwrap_or(perf_result.total_time_ns);
            let baseline_ns = baseline_bench.mean;

            let status = check_regression(current_ns, baseline_bench);

            let change_pct = if baseline_ns > 0 {
                ((current_ns as f64 - baseline_ns as f64) / baseline_ns as f64) * 100.0
            } else {
                0.0
            };

            comparisons.push(Comparison {
                name: perf_result.name.clone(),
                current_ns,
                baseline_ns,
                status,
                change_pct,
            });
        }
    }

    comparisons
}

/// Compares different format conversion results.
///
/// # Arguments
///
/// * `results` - Format conversion results
///
/// # Returns
///
/// Format comparison summary.
pub fn compare_formats(results: &[FormatResult]) -> FormatComparison {
    if results.is_empty() {
        return FormatComparison {
            results: Vec::new(),
            fastest_parse: String::new(),
            fastest_serialize: String::new(),
            most_compact: String::new(),
        };
    }

    let fastest_parse = results
        .iter()
        .min_by_key(|r| r.parse_ns)
        .map(|r| r.format.clone())
        .unwrap_or_default();

    let fastest_serialize = results
        .iter()
        .min_by_key(|r| r.serialize_ns)
        .map(|r| r.format.clone())
        .unwrap_or_default();

    let most_compact = results
        .iter()
        .min_by_key(|r| r.size_bytes)
        .map(|r| r.format.clone())
        .unwrap_or_default();

    FormatComparison {
        results: results.to_vec(),
        fastest_parse,
        fastest_serialize,
        most_compact,
    }
}

/// Identifies regressions from comparison results.
///
/// # Arguments
///
/// * `comparisons` - Comparison results
///
/// # Returns
///
/// Vector of detected regressions.
pub fn identify_regressions(comparisons: &[Comparison]) -> Vec<Regression> {
    comparisons
        .iter()
        .filter(|c| c.status.is_regression())
        .map(|c| Regression {
            name: c.name.clone(),
            status: c.status,
            current_ns: c.current_ns,
            baseline_ns: c.baseline_ns,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{BenchmarkBaseline, Percentiles};
    use crate::report::PerfResult;

    fn create_baseline() -> Baseline {
        let mut baseline = Baseline::new("test");
        baseline.add_benchmark(
            "bench1",
            BenchmarkBaseline {
                mean: 1_000_000,
                std_dev: 100_000,
                percentiles: Percentiles {
                    p50: 1_000_000,
                    p95: 1_100_000,
                    p99: 1_200_000,
                },
            },
        );
        baseline
    }

    fn create_report(avg_ns: u64) -> BenchmarkReport {
        let mut report = BenchmarkReport::new("Test");
        report.add_perf(PerfResult {
            name: "bench1".to_string(),
            iterations: 100,
            total_time_ns: avg_ns * 100,
            throughput_bytes: None,
            avg_time_ns: Some(avg_ns),
            throughput_mbs: None,
        });
        report
    }

    #[test]
    fn test_compare_to_baseline_no_regression() {
        let baseline = create_baseline();
        let report = create_report(1_000_000);

        let comparisons = compare_to_baseline(&report, &baseline);
        assert_eq!(comparisons.len(), 1);
        assert_eq!(comparisons[0].status, RegressionStatus::None);
    }

    #[test]
    fn test_compare_to_baseline_with_regression() {
        let baseline = create_baseline();
        let report = create_report(1_200_000); // 20% slower

        let comparisons = compare_to_baseline(&report, &baseline);
        assert_eq!(comparisons.len(), 1);
        assert!(comparisons[0].status.is_regression());
        assert!(comparisons[0].change_pct > 15.0);
    }

    #[test]
    fn test_identify_regressions() {
        let comparisons = vec![
            Comparison {
                name: "bench1".to_string(),
                current_ns: 1_000_000,
                baseline_ns: 1_000_000,
                status: RegressionStatus::None,
                change_pct: 0.0,
            },
            Comparison {
                name: "bench2".to_string(),
                current_ns: 1_200_000,
                baseline_ns: 1_000_000,
                status: RegressionStatus::Moderate(20),
                change_pct: 20.0,
            },
        ];

        let regressions = identify_regressions(&comparisons);
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].name, "bench2");
    }

    #[test]
    fn test_compare_formats() {
        let results = vec![
            FormatResult {
                format: "JSON".to_string(),
                parse_ns: 1_000_000,
                serialize_ns: 800_000,
                size_bytes: 1000,
            },
            FormatResult {
                format: "HEDL".to_string(),
                parse_ns: 500_000,
                serialize_ns: 600_000,
                size_bytes: 800,
            },
        ];

        let comparison = compare_formats(&results);
        assert_eq!(comparison.fastest_parse, "HEDL");
        assert_eq!(comparison.fastest_serialize, "HEDL");
        assert_eq!(comparison.most_compact, "HEDL");
    }
}

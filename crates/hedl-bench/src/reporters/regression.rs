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

//! Regression detection and reporting.

use crate::core::{check_regression, Baseline};
use crate::harness::Regression;
use crate::reporters::types::{BenchmarkReport, Severity};

/// Detects regressions by comparing results to baseline.
///
/// # Arguments
///
/// * `results` - Current benchmark results
/// * `baseline` - Baseline to compare against
///
/// # Returns
///
/// Vector of detected regressions.
pub fn detect_regressions(results: &BenchmarkReport, baseline: &Baseline) -> Vec<Regression> {
    let mut regressions = Vec::new();

    for result in &results.results {
        if let Some(baseline_bench) = baseline.get_benchmark(&result.name) {
            let current_ns = result.measurement.as_nanos();
            let status = check_regression(current_ns, baseline_bench);

            if status.is_regression() {
                regressions.push(Regression {
                    name: result.name.clone(),
                    status,
                    current_ns,
                    baseline_ns: baseline_bench.mean,
                });
            }
        }
    }

    regressions
}

/// Classifies the severity of a regression.
///
/// # Arguments
///
/// * `regression` - The regression to classify
///
/// # Returns
///
/// Severity level.
pub fn classify_severity(regression: &Regression) -> Severity {
    match regression.status.severity() {
        "severe" => Severity::Critical,
        "moderate" => Severity::High,
        "minor" => Severity::Medium,
        _ => Severity::Low,
    }
}

/// Formats a regression report as a string.
///
/// # Arguments
///
/// * `regressions` - Regressions to format
///
/// # Returns
///
/// Formatted string report.
pub fn format_regression_report(regressions: &[Regression]) -> String {
    if regressions.is_empty() {
        return "No regressions detected.".to_string();
    }

    let mut report = String::new();
    report.push_str(&format!(
        "REGRESSION REPORT: {} detected\n\n",
        regressions.len()
    ));

    for regression in regressions {
        let severity = classify_severity(regression);
        report.push_str(&format!(
            "[{}] {}: {}% slower than baseline\n",
            severity.as_str().to_uppercase(),
            regression.name,
            regression.status.percentage()
        ));
        report.push_str(&format!(
            "  Current: {:.2}ms, Baseline: {:.2}ms\n\n",
            regression.current_ns as f64 / 1_000_000.0,
            regression.baseline_ns as f64 / 1_000_000.0
        ));
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{BenchmarkBaseline, Measurement, Percentiles, RegressionStatus};
    use crate::harness::BenchResult;
    use std::time::Duration;

    #[test]
    fn test_detect_regressions() {
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

        let mut report = BenchmarkReport::new("Test");
        report.add_result(BenchResult::new(
            "bench1",
            100,
            Measurement::new(Duration::from_nanos(1_200_000)), // 20% slower
        ));

        let regressions = detect_regressions(&report, &baseline);
        assert_eq!(regressions.len(), 1);
    }

    #[test]
    fn test_classify_severity() {
        let regression = Regression {
            name: "test".to_string(),
            status: RegressionStatus::Severe(25),
            current_ns: 1_250_000,
            baseline_ns: 1_000_000,
        };

        assert_eq!(classify_severity(&regression), Severity::Critical);
    }

    #[test]
    fn test_format_regression_report() {
        let regressions = vec![Regression {
            name: "test".to_string(),
            status: RegressionStatus::Moderate(10),
            current_ns: 1_100_000,
            baseline_ns: 1_000_000,
        }];

        let report = format_regression_report(&regressions);
        assert!(report.contains("REGRESSION REPORT"));
        assert!(report.contains("10% slower"));
    }
}

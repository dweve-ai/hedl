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

//! Baseline management and regression detection.
//!
//! Manages performance baselines for regression detection across benchmark runs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Regression severity classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RegressionStatus {
    /// No regression detected.
    None,
    /// Minor regression (<5% slower).
    Minor(u8),
    /// Moderate regression (5-15% slower).
    Moderate(u8),
    /// Severe regression (>15% slower).
    Severe(u8),
}

impl RegressionStatus {
    /// Returns the regression percentage.
    pub fn percentage(&self) -> u8 {
        match self {
            RegressionStatus::None => 0,
            RegressionStatus::Minor(p)
            | RegressionStatus::Moderate(p)
            | RegressionStatus::Severe(p) => *p,
        }
    }

    /// Returns whether this represents a regression.
    pub fn is_regression(&self) -> bool {
        !matches!(self, RegressionStatus::None)
    }

    /// Returns the severity level as a string.
    pub fn severity(&self) -> &str {
        match self {
            RegressionStatus::None => "none",
            RegressionStatus::Minor(_) => "minor",
            RegressionStatus::Moderate(_) => "moderate",
            RegressionStatus::Severe(_) => "severe",
        }
    }
}

/// Percentile measurements.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Percentiles {
    /// 50th percentile (median).
    pub p50: u64,
    /// 95th percentile.
    pub p95: u64,
    /// 99th percentile.
    pub p99: u64,
}

impl Percentiles {
    /// Creates percentiles from a sorted vector of durations in nanoseconds.
    pub fn from_sorted(sorted_ns: &[u64]) -> Self {
        if sorted_ns.is_empty() {
            return Self {
                p50: 0,
                p95: 0,
                p99: 0,
            };
        }

        let len = sorted_ns.len();
        Self {
            p50: sorted_ns[len / 2],
            p95: sorted_ns[(len * 95) / 100],
            p99: sorted_ns[(len * 99) / 100],
        }
    }
}

/// Baseline data for a single benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkBaseline {
    /// Mean duration in nanoseconds.
    pub mean: u64,
    /// Standard deviation in nanoseconds.
    pub std_dev: u64,
    /// Percentile measurements.
    pub percentiles: Percentiles,
}

impl BenchmarkBaseline {
    /// Creates a new baseline from duration statistics.
    pub fn new(mean: Duration, std_dev: Duration, percentiles: Percentiles) -> Self {
        Self {
            mean: mean.as_nanos() as u64,
            std_dev: std_dev.as_nanos() as u64,
            percentiles,
        }
    }

    /// Returns the mean as a Duration.
    pub fn mean_duration(&self) -> Duration {
        Duration::from_nanos(self.mean)
    }

    /// Returns the standard deviation as a Duration.
    pub fn std_dev_duration(&self) -> Duration {
        Duration::from_nanos(self.std_dev)
    }
}

/// Complete baseline data for a version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    /// Version identifier (e.g., "1.0.0").
    pub version: String,
    /// Timestamp when baseline was created.
    pub timestamp: String,
    /// Map of benchmark name to baseline data.
    pub benchmarks: HashMap<String, BenchmarkBaseline>,
}

impl Baseline {
    /// Creates a new baseline for a version.
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            benchmarks: HashMap::new(),
        }
    }

    /// Adds a benchmark baseline.
    pub fn add_benchmark(&mut self, name: impl Into<String>, baseline: BenchmarkBaseline) {
        self.benchmarks.insert(name.into(), baseline);
    }

    /// Gets a benchmark baseline by name.
    pub fn get_benchmark(&self, name: &str) -> Option<&BenchmarkBaseline> {
        self.benchmarks.get(name)
    }
}

/// Loads a baseline from a JSON file.
///
/// # Arguments
///
/// * `version` - Version identifier or path to baseline file
///
/// # Returns
///
/// Result containing the loaded baseline or an error.
pub fn load_baseline(version: &str) -> Result<Baseline, Box<dyn std::error::Error>> {
    let path = if version.ends_with(".json") {
        version.to_string()
    } else {
        format!("baselines/{}.json", version)
    };

    let contents = fs::read_to_string(&path)?;
    let baseline = serde_json::from_str(&contents)?;
    Ok(baseline)
}

/// Saves a baseline to a JSON file.
///
/// # Arguments
///
/// * `baseline` - The baseline to save
///
/// # Returns
///
/// Result indicating success or failure.
pub fn save_baseline(baseline: &Baseline) -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("baselines/{}.json", baseline.version);

    // Ensure baselines directory exists
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(baseline)?;
    fs::write(&path, json)?;
    Ok(())
}

/// Checks for regression between current measurement and baseline.
///
/// # Arguments
///
/// * `current_ns` - Current measurement duration in nanoseconds
/// * `baseline` - Baseline to compare against
///
/// # Returns
///
/// `RegressionStatus` indicating the regression severity.
pub fn check_regression(current_ns: u64, baseline: &BenchmarkBaseline) -> RegressionStatus {
    if current_ns <= baseline.mean {
        return RegressionStatus::None;
    }

    let diff = current_ns - baseline.mean;
    let percentage = ((diff as f64 / baseline.mean as f64) * 100.0) as u8;

    match percentage {
        0..=4 => RegressionStatus::None,
        5..=14 => RegressionStatus::Minor(percentage),
        15..=49 => RegressionStatus::Moderate(percentage),
        _ => RegressionStatus::Severe(percentage),
    }
}

/// Updates the current baseline with new measurement.
///
/// # Arguments
///
/// * `current_path` - Path to current baseline file
/// * `benchmark_name` - Name of the benchmark
/// * `baseline_data` - New baseline data to add/update
pub fn update_current_baseline(
    current_path: &str,
    benchmark_name: &str,
    baseline_data: BenchmarkBaseline,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut baseline = load_baseline(current_path).unwrap_or_else(|_| Baseline::new("current"));
    baseline.add_benchmark(benchmark_name, baseline_data);
    baseline.timestamp = chrono::Utc::now().to_rfc3339();

    let json = serde_json::to_string_pretty(&baseline)?;
    fs::write(current_path, json)?;
    Ok(())
}

// We need chrono for timestamps
use chrono;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regression_status() {
        let status = RegressionStatus::Minor(7);
        assert_eq!(status.percentage(), 7);
        assert!(status.is_regression());
        assert_eq!(status.severity(), "minor");

        let none = RegressionStatus::None;
        assert!(!none.is_regression());
    }

    #[test]
    fn test_percentiles() {
        let values = vec![100, 200, 300, 400, 500];
        let p = Percentiles::from_sorted(&values);
        assert_eq!(p.p50, 300);
        assert_eq!(p.p95, 500);
        assert_eq!(p.p99, 500);
    }

    #[test]
    fn test_benchmark_baseline() {
        let percentiles = Percentiles {
            p50: 1000,
            p95: 2000,
            p99: 3000,
        };
        let baseline = BenchmarkBaseline::new(
            Duration::from_millis(1),
            Duration::from_micros(100),
            percentiles,
        );

        assert_eq!(baseline.mean_duration(), Duration::from_millis(1));
        assert!(baseline.std_dev_duration() > Duration::ZERO);
    }

    #[test]
    fn test_baseline_management() {
        let mut baseline = Baseline::new("test");
        let bench_baseline = BenchmarkBaseline {
            mean: 1_000_000,
            std_dev: 100_000,
            percentiles: Percentiles {
                p50: 1_000_000,
                p95: 1_200_000,
                p99: 1_500_000,
            },
        };

        baseline.add_benchmark("test_bench", bench_baseline);
        assert!(baseline.get_benchmark("test_bench").is_some());
        assert_eq!(baseline.version, "test");
    }

    #[test]
    fn test_check_regression() {
        let baseline = BenchmarkBaseline {
            mean: 1_000_000,
            std_dev: 50_000,
            percentiles: Percentiles {
                p50: 1_000_000,
                p95: 1_100_000,
                p99: 1_200_000,
            },
        };

        // No regression
        assert_eq!(
            check_regression(1_000_000, &baseline),
            RegressionStatus::None
        );

        // Minor regression (7%)
        let status = check_regression(1_070_000, &baseline);
        assert!(matches!(status, RegressionStatus::Minor(_)));

        // Minor regression (10% - still in 5-14% range)
        let status = check_regression(1_100_000, &baseline);
        assert!(matches!(status, RegressionStatus::Minor(_)));

        // Moderate regression (20% - in 15-49% range)
        let status = check_regression(1_200_000, &baseline);
        assert!(matches!(status, RegressionStatus::Moderate(_)));

        // Severe regression (50%+)
        let status = check_regression(1_500_000, &baseline);
        assert!(matches!(status, RegressionStatus::Severe(_)));
    }
}

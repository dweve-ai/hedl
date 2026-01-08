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

//! Result aggregation and statistics computation.
//!
//! Aggregates benchmark results and computes statistical summaries.

use crate::core::registry::Category;
use crate::harness::runner::BenchResult;
use std::collections::HashMap;
use std::time::Duration;

/// Aggregated results from multiple benchmark runs.
#[derive(Debug, Clone)]
pub struct AggregatedResults {
    /// Total number of benchmarks.
    pub total_benchmarks: usize,
    /// Total time across all benchmarks.
    pub total_duration: Duration,
    /// Average time per benchmark.
    pub avg_duration: Duration,
    /// Fastest benchmark.
    pub fastest: Option<String>,
    /// Slowest benchmark.
    pub slowest: Option<String>,
}

/// Statistical summary of benchmark results.
#[derive(Debug, Clone)]
pub struct Statistics {
    /// Mean duration.
    pub mean: Duration,
    /// Standard deviation.
    pub std_dev: Duration,
    /// Minimum duration.
    pub min: Duration,
    /// Maximum duration.
    pub max: Duration,
    /// Median duration.
    pub median: Duration,
    /// Total number of samples.
    pub count: usize,
}

/// Aggregates results from multiple benchmark runs.
///
/// # Arguments
///
/// * `results` - Slice of benchmark results
///
/// # Returns
///
/// Aggregated results summary.
pub fn aggregate_results(results: &[BenchResult]) -> AggregatedResults {
    if results.is_empty() {
        return AggregatedResults {
            total_benchmarks: 0,
            total_duration: Duration::ZERO,
            avg_duration: Duration::ZERO,
            fastest: None,
            slowest: None,
        };
    }

    let total_duration: Duration = results.iter().map(|r| r.measurement.duration).sum();
    let avg_duration = total_duration / results.len() as u32;

    let fastest = results
        .iter()
        .min_by_key(|r| r.measurement.duration)
        .map(|r| r.name.clone());

    let slowest = results
        .iter()
        .max_by_key(|r| r.measurement.duration)
        .map(|r| r.name.clone());

    AggregatedResults {
        total_benchmarks: results.len(),
        total_duration,
        avg_duration,
        fastest,
        slowest,
    }
}

/// Computes statistical summary from benchmark results.
///
/// # Arguments
///
/// * `results` - Slice of benchmark results
///
/// # Returns
///
/// Statistical summary.
pub fn compute_statistics(results: &[BenchResult]) -> Statistics {
    if results.is_empty() {
        return Statistics {
            mean: Duration::ZERO,
            std_dev: Duration::ZERO,
            min: Duration::ZERO,
            max: Duration::ZERO,
            median: Duration::ZERO,
            count: 0,
        };
    }

    let mut durations: Vec<Duration> = results.iter().map(|r| r.measurement.duration).collect();
    durations.sort();

    let total: Duration = durations.iter().sum();
    let mean = total / durations.len() as u32;

    let variance: f64 = durations
        .iter()
        .map(|d: &Duration| {
            let diff = d.as_nanos() as f64 - mean.as_nanos() as f64;
            diff * diff
        })
        .sum::<f64>()
        / durations.len() as f64;

    let std_dev = Duration::from_nanos(variance.sqrt() as u64);

    Statistics {
        mean,
        std_dev,
        min: durations[0],
        max: durations[durations.len() - 1],
        median: durations[durations.len() / 2],
        count: durations.len(),
    }
}

/// Groups benchmark results by category.
///
/// # Arguments
///
/// * `results` - Slice of benchmark results
///
/// # Returns
///
/// HashMap of category to results.
pub fn group_by_category(results: &[BenchResult]) -> HashMap<Category, Vec<BenchResult>> {
    let mut grouped: HashMap<Category, Vec<BenchResult>> = HashMap::new();

    for result in results {
        let category = infer_category(&result.name);
        grouped
            .entry(category)
            .or_default()
            .push(result.clone());
    }

    grouped
}

/// Infers the category from a benchmark name.
fn infer_category(name: &str) -> Category {
    let lower = name.to_lowercase();
    if lower.contains("parse") {
        Category::Parsing
    } else if lower.contains("convert") || lower.contains("json") || lower.contains("yaml") {
        Category::Conversion
    } else if lower.contains("stream") {
        Category::Streaming
    } else if lower.contains("memory") || lower.contains("mem") {
        Category::Memory
    } else if lower.contains("validate") || lower.contains("valid") {
        Category::Validation
    } else if lower.contains("lsp") {
        Category::Lsp
    } else if lower.contains("mcp") {
        Category::Mcp
    } else if lower.contains("ffi") {
        Category::Ffi
    } else {
        Category::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Measurement;

    fn create_test_result(name: &str, millis: u64) -> BenchResult {
        BenchResult::new(name, 1, Measurement::new(Duration::from_millis(millis)))
    }

    #[test]
    fn test_aggregate_results() {
        let results = vec![
            create_test_result("bench1", 100),
            create_test_result("bench2", 200),
            create_test_result("bench3", 150),
        ];

        let agg = aggregate_results(&results);
        assert_eq!(agg.total_benchmarks, 3);
        assert_eq!(agg.total_duration, Duration::from_millis(450));
        assert_eq!(agg.fastest, Some("bench1".to_string()));
        assert_eq!(agg.slowest, Some("bench2".to_string()));
    }

    #[test]
    fn test_compute_statistics() {
        let results = vec![
            create_test_result("b1", 100),
            create_test_result("b2", 200),
            create_test_result("b3", 300),
        ];

        let stats = compute_statistics(&results);
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, Duration::from_millis(200));
        assert_eq!(stats.min, Duration::from_millis(100));
        assert_eq!(stats.max, Duration::from_millis(300));
        assert_eq!(stats.median, Duration::from_millis(200));
    }

    #[test]
    fn test_group_by_category() {
        let results = vec![
            create_test_result("parse_test", 100),
            create_test_result("convert_json", 200),
            create_test_result("parse_yaml", 150),
        ];

        let grouped = group_by_category(&results);
        assert!(grouped.contains_key(&Category::Parsing));
        assert!(grouped.contains_key(&Category::Conversion));
        assert_eq!(grouped[&Category::Parsing].len(), 2);
        assert_eq!(grouped[&Category::Conversion].len(), 1);
    }

    #[test]
    fn test_infer_category() {
        assert_eq!(infer_category("parse_users"), Category::Parsing);
        assert_eq!(infer_category("convert_to_json"), Category::Conversion);
        assert_eq!(infer_category("stream_data"), Category::Streaming);
        assert_eq!(infer_category("memory_test"), Category::Memory);
        assert_eq!(infer_category("validate_schema"), Category::Validation);
        assert_eq!(infer_category("lsp_completion"), Category::Lsp);
        assert_eq!(infer_category("unknown"), Category::Other);
    }
}

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

//! Performance analysis and recommendation generation.
//!
//! Analyzes benchmark results to identify bottlenecks and generate
//! actionable optimization recommendations.

use crate::core::registry::Category;
use crate::harness::BenchResult;
use crate::reporters::types::{
    Bottleneck, EstimatedImpact, PerformanceAnalysis, Recommendation, Severity,
};
use std::time::Duration;

/// Analyzes performance from benchmark results.
///
/// # Arguments
///
/// * `results` - Benchmark results to analyze
///
/// # Returns
///
/// Performance analysis with bottlenecks identified.
pub fn analyze_performance(results: &[BenchResult]) -> PerformanceAnalysis {
    let bottlenecks = identify_bottlenecks(results);

    PerformanceAnalysis {
        bottlenecks,
        regressions: Vec::new(), // Filled by comparator
        comparisons: Vec::new(), // Filled by comparator
    }
}

/// Identifies performance bottlenecks from results.
///
/// # Arguments
///
/// * `results` - Benchmark results
///
/// # Returns
///
/// Vector of identified bottlenecks.
pub fn identify_bottlenecks(results: &[BenchResult]) -> Vec<Bottleneck> {
    if results.is_empty() {
        return Vec::new();
    }

    let mut bottlenecks = Vec::new();
    let total_time: Duration = results.iter().map(|r| r.measurement.duration).sum();

    // Find slowest operations (>10% of total time)
    for result in results {
        let pct = (result.measurement.duration.as_secs_f64() / total_time.as_secs_f64()) * 100.0;

        if pct > 10.0 {
            let severity = if pct > 30.0 {
                Severity::High
            } else if pct > 20.0 {
                Severity::Medium
            } else {
                Severity::Low
            };

            bottlenecks.push(Bottleneck {
                location: result.name.clone(),
                category: infer_category(&result.name),
                severity,
                description: format!("Takes {:.1}% of total execution time", pct),
                impact_pct: pct,
            });
        }
    }

    // Find operations with poor scaling
    let scaling_bottlenecks = find_scaling_issues(results);
    bottlenecks.extend(scaling_bottlenecks);

    bottlenecks
}

/// Finds operations with poor scaling characteristics.
fn find_scaling_issues(results: &[BenchResult]) -> Vec<Bottleneck> {
    let mut bottlenecks = Vec::new();

    // Group by base name
    let mut groups: std::collections::HashMap<String, Vec<&BenchResult>> =
        std::collections::HashMap::new();

    for result in results {
        if let Some(size) = result.size {
            let base_name = result
                .name
                .strip_suffix(&format!("_{}", size))
                .unwrap_or(&result.name);
            groups
                .entry(base_name.to_string())
                .or_default()
                .push(result);
        }
    }

    // Check for worse than O(n) scaling
    for (name, mut group_results) in groups {
        if group_results.len() < 2 {
            continue;
        }

        group_results.sort_by_key(|r: &&BenchResult| r.size.unwrap_or(0));

        let first: &BenchResult = group_results[0];
        let last: &BenchResult = group_results[group_results.len() - 1];

        let size_ratio = last.size.unwrap_or(1) as f64 / first.size.unwrap_or(1).max(1) as f64;
        let time_ratio = last.measurement.duration.as_secs_f64()
            / first.measurement.duration.as_secs_f64().max(0.001);

        // If time grows faster than O(n log n), flag it
        let expected_max = size_ratio * size_ratio.log2();
        if time_ratio > expected_max * 1.5 {
            bottlenecks.push(Bottleneck {
                location: name.clone(),
                category: infer_category(&name),
                severity: Severity::Medium,
                description: format!(
                    "Poor scaling: {}x size increase causes {}x time increase",
                    size_ratio as usize, time_ratio as usize
                ),
                impact_pct: 0.0,
            });
        }
    }

    bottlenecks
}

/// Generates optimization recommendations from analysis.
///
/// Generates at least 3 specific, actionable recommendations per report.
///
/// # Arguments
///
/// * `analysis` - Performance analysis results
///
/// # Returns
///
/// Vector of recommendations (minimum 3).
pub fn generate_recommendations(analysis: &PerformanceAnalysis) -> Vec<Recommendation> {
    let mut recommendations = Vec::new();

    // Generate recommendations from bottlenecks
    for bottleneck in &analysis.bottlenecks {
        let rec = generate_bottleneck_recommendation(bottleneck);
        recommendations.push(rec);
    }

    // Generate recommendations from regressions
    for regression in &analysis.regressions {
        recommendations.push(Recommendation {
            severity: match regression.status.severity() {
                "severe" => Severity::Critical,
                "moderate" => Severity::High,
                "minor" => Severity::Medium,
                _ => Severity::Low,
            },
            category: Category::Other,
            message: format!(
                "Performance regression in '{}': {}% slower than baseline",
                regression.name,
                regression.status.percentage()
            ),
            impact: EstimatedImpact {
                improvement_pct: regression.status.percentage() as f64,
                effort_hours: 4.0,
                confidence: 0.9,
            },
        });
    }

    // Ensure minimum 3 recommendations
    while recommendations.len() < 3 {
        recommendations.push(generate_general_recommendation(recommendations.len()));
    }

    recommendations
}

/// Generates a recommendation for a specific bottleneck.
fn generate_bottleneck_recommendation(bottleneck: &Bottleneck) -> Recommendation {
    let (message, impact) = match bottleneck.category {
        Category::Parsing => (
            format!(
                "Optimize parsing in '{}': Consider SIMD vectorization or lazy parsing",
                bottleneck.location
            ),
            EstimatedImpact {
                improvement_pct: 20.0,
                effort_hours: 8.0,
                confidence: 0.7,
            },
        ),
        Category::Conversion => (
            format!(
                "Optimize conversion in '{}': Use zero-copy techniques or buffer pooling",
                bottleneck.location
            ),
            EstimatedImpact {
                improvement_pct: 15.0,
                effort_hours: 6.0,
                confidence: 0.75,
            },
        ),
        Category::Memory => (
            format!(
                "Reduce memory usage in '{}': Apply arena allocation or memory pooling",
                bottleneck.location
            ),
            EstimatedImpact {
                improvement_pct: 25.0,
                effort_hours: 10.0,
                confidence: 0.8,
            },
        ),
        _ => (
            format!(
                "Optimize '{}': Profile to identify hot paths and apply targeted optimizations",
                bottleneck.location
            ),
            EstimatedImpact {
                improvement_pct: 10.0,
                effort_hours: 4.0,
                confidence: 0.6,
            },
        ),
    };

    Recommendation {
        severity: bottleneck.severity,
        category: bottleneck.category,
        message,
        impact,
    }
}

/// Generates a general recommendation when specific ones are insufficient.
fn generate_general_recommendation(index: usize) -> Recommendation {
    let recommendations = [
        (
            "Enable link-time optimization (LTO) in release builds for 5-15% performance improvement",
            Category::Other,
            EstimatedImpact {
                improvement_pct: 10.0,
                effort_hours: 1.0,
                confidence: 0.9,
            },
        ),
        (
            "Profile with perf/flamegraph to identify CPU hotspots and optimize hot paths",
            Category::Other,
            EstimatedImpact {
                improvement_pct: 15.0,
                effort_hours: 4.0,
                confidence: 0.75,
            },
        ),
        (
            "Consider caching frequently accessed data structures to reduce recomputation",
            Category::Other,
            EstimatedImpact {
                improvement_pct: 12.0,
                effort_hours: 6.0,
                confidence: 0.7,
            },
        ),
    ];

    let (message, category, impact) = &recommendations[index % recommendations.len()];

    Recommendation {
        severity: Severity::Medium,
        category: *category,
        message: message.to_string(),
        impact: *impact,
    }
}

/// Infers category from benchmark name.
fn infer_category(name: &str) -> Category {
    let lower = name.to_lowercase();
    if lower.contains("parse") {
        Category::Parsing
    } else if lower.contains("convert") {
        Category::Conversion
    } else if lower.contains("memory") {
        Category::Memory
    } else if lower.contains("stream") {
        Category::Streaming
    } else {
        Category::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Measurement;

    fn create_result(name: &str, millis: u64, size: Option<usize>) -> BenchResult {
        let mut result = BenchResult::new(name, 1, Measurement::new(Duration::from_millis(millis)));
        if let Some(s) = size {
            result = result.with_size(s);
        }
        result
    }

    #[test]
    fn test_identify_bottlenecks() {
        let results = vec![
            create_result("fast", 10, None),
            create_result("slow", 100, None), // >10% of total
        ];

        let bottlenecks = identify_bottlenecks(&results);
        assert!(!bottlenecks.is_empty());
    }

    #[test]
    fn test_generate_recommendations_minimum() {
        let analysis = PerformanceAnalysis {
            bottlenecks: Vec::new(),
            regressions: Vec::new(),
            comparisons: Vec::new(),
        };

        let recommendations = generate_recommendations(&analysis);
        assert!(recommendations.len() >= 3);
    }

    #[test]
    fn test_generate_recommendations_from_bottleneck() {
        let analysis = PerformanceAnalysis {
            bottlenecks: vec![Bottleneck {
                location: "parse_test".to_string(),
                category: Category::Parsing,
                severity: Severity::High,
                description: "Test".to_string(),
                impact_pct: 50.0,
            }],
            regressions: Vec::new(),
            comparisons: Vec::new(),
        };

        let recommendations = generate_recommendations(&analysis);
        assert!(recommendations.len() >= 3);
        assert!(recommendations[0].message.contains("parsing"));
    }
}

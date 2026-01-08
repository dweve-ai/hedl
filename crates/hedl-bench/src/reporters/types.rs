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

//! Report types and structures.
//!
//! Defines data structures for benchmark reports, performance analysis,
//! and optimization recommendations.

use crate::core::registry::Category;
use crate::harness::{BenchResult, Comparison, Regression};
use serde::{Deserialize, Serialize};

/// Severity level for issues and recommendations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Informational only.
    Info,
    /// Low priority.
    Low,
    /// Medium priority.
    Medium,
    /// High priority - should be addressed.
    High,
    /// Critical - must be addressed immediately.
    Critical,
}

impl Severity {
    /// Returns the severity as a string.
    pub fn as_str(&self) -> &str {
        match self {
            Severity::Info => "info",
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }
}

/// Performance bottleneck identification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    /// Location/name of the bottleneck.
    pub location: String,
    /// Category of bottleneck.
    pub category: Category,
    /// Severity level.
    pub severity: Severity,
    /// Description of the issue.
    pub description: String,
    /// Impact as percentage of total time.
    pub impact_pct: f64,
}

/// Estimated impact of an optimization.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EstimatedImpact {
    /// Expected performance improvement (percentage).
    pub improvement_pct: f64,
    /// Implementation effort (person-hours).
    pub effort_hours: f64,
    /// Confidence level (0.0 - 1.0).
    pub confidence: f64,
}

/// Optimization recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Severity/priority of this recommendation.
    pub severity: Severity,
    /// Category this applies to.
    pub category: Category,
    /// Detailed recommendation message.
    pub message: String,
    /// Estimated impact if implemented.
    pub impact: EstimatedImpact,
}

/// Performance analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// Identified bottlenecks.
    pub bottlenecks: Vec<Bottleneck>,
    /// Detected regressions.
    pub regressions: Vec<Regression>,
    /// Baseline comparisons.
    pub comparisons: Vec<Comparison>,
}

/// Complete benchmark report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    /// Report title.
    pub title: String,
    /// Benchmark results.
    pub results: Vec<BenchResult>,
    /// Performance analysis.
    pub analysis: PerformanceAnalysis,
    /// Optimization recommendations.
    pub recommendations: Vec<Recommendation>,
    /// Report timestamp.
    pub timestamp: String,
    /// Additional notes.
    pub notes: Vec<String>,
}

impl BenchmarkReport {
    /// Creates a new benchmark report.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            results: Vec::new(),
            analysis: PerformanceAnalysis {
                bottlenecks: Vec::new(),
                regressions: Vec::new(),
                comparisons: Vec::new(),
            },
            recommendations: Vec::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            notes: Vec::new(),
        }
    }

    /// Adds a benchmark result.
    pub fn add_result(&mut self, result: BenchResult) {
        self.results.push(result);
    }

    /// Adds a bottleneck to the analysis.
    pub fn add_bottleneck(&mut self, bottleneck: Bottleneck) {
        self.analysis.bottlenecks.push(bottleneck);
    }

    /// Adds a regression to the analysis.
    pub fn add_regression(&mut self, regression: Regression) {
        self.analysis.regressions.push(regression);
    }

    /// Adds a comparison to the analysis.
    pub fn add_comparison(&mut self, comparison: Comparison) {
        self.analysis.comparisons.push(comparison);
    }

    /// Adds a recommendation.
    pub fn add_recommendation(&mut self, recommendation: Recommendation) {
        self.recommendations.push(recommendation);
    }

    /// Adds a note.
    pub fn add_note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }

    /// Returns the number of results.
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Returns whether any regressions were detected.
    pub fn has_regressions(&self) -> bool {
        !self.analysis.regressions.is_empty()
    }

    /// Returns the number of high-severity recommendations.
    pub fn high_priority_count(&self) -> usize {
        self.recommendations
            .iter()
            .filter(|r| matches!(r.severity, Severity::High | Severity::Critical))
            .count()
    }
}

use chrono;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity() {
        assert_eq!(Severity::High.as_str(), "high");
        assert_eq!(Severity::Critical.as_str(), "critical");
    }

    #[test]
    fn test_benchmark_report() {
        let mut report = BenchmarkReport::new("Test Report");
        report.add_note("Test note");

        assert_eq!(report.title, "Test Report");
        assert_eq!(report.notes.len(), 1);
        assert!(!report.has_regressions());
    }

    #[test]
    fn test_high_priority_count() {
        let mut report = BenchmarkReport::new("Test");

        report.add_recommendation(Recommendation {
            severity: Severity::High,
            category: Category::Parsing,
            message: "High priority".to_string(),
            impact: EstimatedImpact {
                improvement_pct: 20.0,
                effort_hours: 4.0,
                confidence: 0.8,
            },
        });

        report.add_recommendation(Recommendation {
            severity: Severity::Low,
            category: Category::Parsing,
            message: "Low priority".to_string(),
            impact: EstimatedImpact {
                improvement_pct: 5.0,
                effort_hours: 1.0,
                confidence: 0.9,
            },
        });

        assert_eq!(report.high_priority_count(), 1);
    }
}

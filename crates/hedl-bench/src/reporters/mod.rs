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

//! Benchmark reporters for various output formats.
//!
//! Provides reporting infrastructure for benchmark results including
//! console output, JSON/Markdown/HTML export, performance analysis,
//! and regression detection.
//!
//! # Modules
//!
//! - `types`: Core report data structures
//! - `analysis`: Performance analysis and recommendations
//! - `console`: Console output formatting
//! - `json`: JSON export
//! - `markdown`: Markdown export
//! - `html`: HTML export
//! - `regression`: Regression detection and reporting

pub mod analysis;
pub mod console;
pub mod html;
pub mod json;
pub mod markdown;
pub mod regression;
pub mod types;

// Re-export commonly used types
pub use analysis::{analyze_performance, generate_recommendations, identify_bottlenecks};
pub use console::{print_regressions, print_report, print_summary};
pub use html::export_html;
pub use json::export_json;
pub use markdown::export_markdown;
pub use regression::{classify_severity, detect_regressions, format_regression_report};
pub use types::{
    BenchmarkReport, Bottleneck, EstimatedImpact, PerformanceAnalysis, Recommendation, Severity,
};

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

//! Benchmark harness for unified reporting.
//!
//! Provides standardized infrastructure for collecting and reporting benchmark results.
//!
//! # Modules
//!
//! - `runner`: Benchmark execution and result collection
//! - `collector`: Metric collection infrastructure
//! - `aggregator`: Result aggregation and statistics
//! - `comparator`: Baseline comparison and regression detection

pub mod aggregator;
pub mod collector;
pub mod comparator;
pub mod runner;

pub use aggregator::{
    aggregate_results, compute_statistics, group_by_category, AggregatedResults, Statistics,
};
pub use collector::{
    collect_memory, collect_performance, collect_throughput, MemMetrics, MetricCollector,
    PerfMetrics, ThroughputMetrics,
};
pub use comparator::{
    compare_formats, compare_to_baseline, identify_regressions, Comparison, FormatComparison,
    FormatResult, Regression,
};
pub use runner::{BenchResult, BenchmarkRunner};

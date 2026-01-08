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

//! Core benchmark infrastructure.
//!
//! Provides centralized configuration, measurement primitives, baseline management,
//! and benchmark registry for the HEDL benchmarking framework.
//!
//! # Modules
//!
//! - `config`: Centralized benchmark configuration
//! - `measurement`: Low-overhead timing and measurement
//! - `baselines`: Performance baseline management and regression detection
//! - `registry`: Benchmark discovery and metadata

pub mod baselines;
pub mod config;
pub mod measurement;
pub mod registry;

// Re-export commonly used types
pub use baselines::{
    check_regression, load_baseline, save_baseline, update_current_baseline, Baseline,
    BenchmarkBaseline, Percentiles, RegressionStatus,
};
pub use config::{BenchConfig, ExportFormat, LARGE_SIZES, STANDARD_SIZES};
pub use measurement::{
    compute_statistics, measure, measure_memory, measure_with_throughput, Measurement, Statistics,
};
pub use registry::{
    discover_benchmarks, filter_by_category, filter_by_tag, register_benchmark, BenchmarkInfo,
    BenchmarkMetadata, Category, Coverage,
};

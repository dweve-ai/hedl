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

// Allow deprecated usage in tests - the legacy module tests test deprecated functionality
#![cfg_attr(not(test), warn(missing_docs))]
#![cfg_attr(test, allow(deprecated))]

//! HEDL Benchmark Framework
//!
//! Comprehensive benchmarking and performance testing for HEDL.
//!
//! ## Features
//!
//! - **Dataset generators**: Create realistic test data of various sizes
//! - **Token counting**: Compare HEDL token efficiency vs JSON/YAML/XML
//! - **Performance metrics**: Parse, convert, and stream benchmarks
//!
//! ## Usage
//!
//! Run all benchmarks:
//! ```bash
//! cargo bench --package hedl-bench
//! ```
//!
//! Run specific benchmark:
//! ```bash
//! cargo bench --package hedl-bench --bench parsing
//! ```

// Core modules (Phase 1 infrastructure - COMPLETE)
/// Core benchmark infrastructure.
pub mod core;
/// Dataset generators for benchmarks.
pub mod datasets;
/// Error types for HEDL benchmarking operations.
pub mod error;
/// Benchmark harness for unified reporting.
pub mod harness;
/// Comprehensive reporting module for HEDL benchmarks.
pub mod report;
/// Benchmark reporters for various output formats.
pub mod reporters;
/// Token counting utilities for comparing HEDL efficiency vs other formats.
pub mod token_counter;

// New modular structure (Phase 2)
/// Helper utilities for HEDL benchmarks.
pub mod benchmark_utilities;
/// Fixture management for benchmark data.
pub mod fixtures;
/// Comprehensive data generation for HEDL benchmarks.
pub mod generators;

/// HEDL Accuracy Benchmark Framework v2.0.
pub mod accuracy;

/// Real-world datasets with verified ground truth for LLM accuracy benchmarks.
pub mod real_datasets;

// Re-export key types for convenience
pub use datasets::{
    generate_analytics, generate_blog, generate_config, generate_deep_hierarchy,
    generate_ditto_heavy, generate_events, generate_graph, generate_nested, generate_orders,
    generate_products, generate_reference_heavy, generate_users, generate_users_safe, validation,
    DatasetSize,
};
pub use error::{validate_dataset_size, BenchError, Result, MAX_DATASET_SIZE};

// New module re-exports
pub use benchmark_utilities::{convert_to_json, convert_to_yaml, parse_unchecked};
pub use fixtures::{load_all_fixtures, load_fixture as load_fixture_new, FixtureCache};
pub use generators::{ComplexityLevel as GenComplexityLevel, GeneratorConfig};
pub use report::{
    BenchmarkReport, ComparisonRow, ComplexityLevel, CustomTable, ExportConfig,
    FormatDatasetResult, FormatMetrics, Insight, PerfResult, SummaryReport, TableCell,
};
pub use token_counter::{compare_formats, count_tokens, TokenStats};

/// Standard fixture sizes for benchmarks
pub mod sizes {
    /// Small dataset: < 1KB, ~10 entities
    pub const SMALL: usize = 10;
    /// Medium dataset: ~10KB, ~100 entities
    pub const MEDIUM: usize = 100;
    /// Large dataset: ~100KB, ~1,000 entities
    pub const LARGE: usize = 1_000;
    /// Stress test: ~1MB, ~10,000 entities
    pub const STRESS: usize = 10_000;
    /// Extreme test: ~10MB, ~100,000 entities
    pub const EXTREME: usize = 100_000;
}

/// Load a fixture file as a string
pub fn load_fixture(name: &str) -> Result<String> {
    use core::name_validation::validate_benchmark_name;

    validate_benchmark_name(name)?;
    let path = format!("{}/fixtures/{}.hedl", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).or_else(|_| {
        // Fallback to generated fixture
        Ok(match name {
            "small" => generate_users(sizes::SMALL),
            "medium" => generate_users(sizes::MEDIUM),
            "large" => generate_users(sizes::LARGE),
            "stress" => generate_users(sizes::STRESS),
            _ => return Err(BenchError::IoError(format!("Unknown fixture: {name}"))),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_users() {
        let hedl = generate_users(10);
        assert!(hedl.contains("%V:2.0"));
        // v2.0 uses separate %S and %C directives
        assert!(hedl.contains("%S:User:[id,name,email,role,created_at]"));
        assert!(hedl.contains("%C:User.total=10"));
        assert!(hedl.contains("users:@User"));
    }

    #[test]
    fn test_token_stats() {
        let hedl = generate_users(10);
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
        let stats = compare_formats(&doc);

        // HEDL should be more token-efficient than JSON
        assert!(stats.savings_vs_json > 0.0);
    }

    #[test]
    fn test_dataset_sizes() {
        // Verify datasets scale appropriately
        let small = generate_users(sizes::SMALL);
        let medium = generate_users(sizes::MEDIUM);

        assert!(medium.len() > small.len() * 5);
    }
}

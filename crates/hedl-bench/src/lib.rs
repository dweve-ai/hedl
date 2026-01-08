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
pub mod core;
pub mod datasets;
pub mod error;
pub mod harness;
pub mod report;
pub mod reporters;
pub mod token_counter;

// New modular structure (Phase 2)
pub mod fixtures;
pub mod generators;
pub mod helpers;

// Legacy modules (deprecated, moved to legacy/)
#[allow(deprecated)]
pub mod legacy;

// Re-export key types for convenience
pub use datasets::{
    generate_analytics, generate_blog, generate_config, generate_deep_hierarchy,
    generate_ditto_heavy, generate_events, generate_graph, generate_nested, generate_orders,
    generate_products, generate_reference_heavy, generate_users, generate_users_safe, validation,
    DatasetSize,
};
pub use error::{validate_dataset_size, BenchError, Result, MAX_DATASET_SIZE};

// Legacy re-exports for backward compatibility (temporarily disabled)
// #[allow(deprecated)]
// pub use legacy::normalize::{compare, normalize};
// #[allow(deprecated)]
// pub use legacy::questions::{
//     all_questions, all_questions_flat, generate_blog_questions, generate_event_questions,
//     generate_product_questions, generate_user_questions, generate_validation_questions,
//     question_counts, AnswerType, Question, QuestionType,
// };

// New module re-exports (temporarily disabled)
// pub use fixtures::{FixtureCache, load_fixture as load_fixture_new, load_all_fixtures};
// pub use generators::{ComplexityLevel as GenComplexityLevel, GeneratorConfig};
// pub use helpers::{parse_unchecked, convert_to_json, convert_to_yaml};
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
pub fn load_fixture(name: &str) -> String {
    let path = format!("{}/fixtures/{}.hedl", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|_| {
        // Fallback to generated fixture
        match name {
            "small" => generate_users(sizes::SMALL),
            "medium" => generate_users(sizes::MEDIUM),
            "large" => generate_users(sizes::LARGE),
            "stress" => generate_users(sizes::STRESS),
            _ => panic!("Unknown fixture: {}", name),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_users() {
        let hedl = generate_users(10);
        assert!(hedl.contains("%VERSION: 1.0"));
        // Uses %STRUCT with count in header for LLM comprehension
        assert!(hedl.contains("%STRUCT: User (10): [id,name,email,role,created_at]"));
        assert!(hedl.contains("users: @User"));
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

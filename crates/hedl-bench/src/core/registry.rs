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

//! Benchmark discovery and metadata registry.
//!
//! Provides registration and discovery of benchmarks with metadata
//! for categorization, tagging, and coverage tracking.

use std::collections::HashMap;
use std::sync::RwLock;

/// Benchmark category classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Category {
    /// Parsing benchmarks.
    Parsing,
    /// Serialization/conversion benchmarks.
    Conversion,
    /// Streaming I/O benchmarks.
    Streaming,
    /// Memory usage benchmarks.
    Memory,
    /// Validation benchmarks.
    Validation,
    /// LSP operation benchmarks.
    Lsp,
    /// MCP operation benchmarks.
    Mcp,
    /// FFI operation benchmarks.
    Ffi,
    /// Other/custom category.
    Other,
}

impl Category {
    /// Returns the category as a string.
    pub fn as_str(&self) -> &str {
        match self {
            Category::Parsing => "parsing",
            Category::Conversion => "conversion",
            Category::Streaming => "streaming",
            Category::Memory => "memory",
            Category::Validation => "validation",
            Category::Lsp => "lsp",
            Category::Mcp => "mcp",
            Category::Ffi => "ffi",
            Category::Other => "other",
        }
    }
}

/// Code coverage information for a benchmark.
#[derive(Debug, Clone)]
pub struct Coverage {
    /// Percentage of code covered by this benchmark.
    pub percentage: f64,
    /// Lines covered.
    pub lines_covered: usize,
    /// Total lines.
    pub total_lines: usize,
}

impl Coverage {
    /// Creates new coverage information.
    pub fn new(lines_covered: usize, total_lines: usize) -> Self {
        let percentage = if total_lines > 0 {
            (lines_covered as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        Self {
            percentage,
            lines_covered,
            total_lines,
        }
    }

    /// Returns whether coverage meets a threshold.
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.percentage >= threshold
    }
}

/// Metadata for a benchmark.
#[derive(Debug, Clone)]
pub struct BenchmarkMetadata {
    /// Category classification.
    pub category: Category,
    /// Tags for filtering and grouping.
    pub tags: Vec<String>,
    /// Coverage information.
    pub coverage: Option<Coverage>,
    /// Description of the benchmark.
    pub description: String,
}

impl BenchmarkMetadata {
    /// Creates new benchmark metadata.
    pub fn new(category: Category, description: impl Into<String>) -> Self {
        Self {
            category,
            tags: Vec::new(),
            coverage: None,
            description: description.into(),
        }
    }

    /// Adds a tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Adds multiple tags.
    pub fn with_tags(mut self, tags: &[&str]) -> Self {
        self.tags.extend(tags.iter().map(|s| s.to_string()));
        self
    }

    /// Sets coverage information.
    pub fn with_coverage(mut self, coverage: Coverage) -> Self {
        self.coverage = Some(coverage);
        self
    }

    /// Returns whether this benchmark has a specific tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

/// Information about a registered benchmark.
#[derive(Debug, Clone)]
pub struct BenchmarkInfo {
    /// Unique name of the benchmark.
    pub name: String,
    /// Benchmark metadata.
    pub metadata: BenchmarkMetadata,
}

/// Global benchmark registry.
static REGISTRY: RwLock<Option<BenchmarkRegistry>> = RwLock::new(None);

/// The benchmark registry implementation.
#[derive(Debug, Default)]
struct BenchmarkRegistry {
    benchmarks: HashMap<String, BenchmarkMetadata>,
}

/// Registers a benchmark with metadata.
///
/// # Arguments
///
/// * `name` - Unique name for the benchmark
/// * `metadata` - Metadata describing the benchmark
///
/// # Example
///
/// ```no_run
/// use hedl_bench::core::registry::{register_benchmark, BenchmarkMetadata, Category};
///
/// let metadata = BenchmarkMetadata::new(Category::Parsing, "Parse HEDL documents")
///     .with_tag("core")
///     .with_tag("performance");
///
/// register_benchmark("parse_users", metadata);
/// ```
pub fn register_benchmark(name: &str, metadata: BenchmarkMetadata) {
    let mut lock = REGISTRY.write().unwrap();
    let registry = lock.get_or_insert_with(BenchmarkRegistry::default);
    registry.benchmarks.insert(name.to_string(), metadata);
}

/// Discovers all registered benchmarks.
///
/// # Returns
///
/// A vector of `BenchmarkInfo` for all registered benchmarks.
pub fn discover_benchmarks() -> Vec<BenchmarkInfo> {
    let lock = REGISTRY.read().unwrap();
    if let Some(registry) = lock.as_ref() {
        registry
            .benchmarks
            .iter()
            .map(|(name, metadata)| BenchmarkInfo {
                name: name.clone(),
                metadata: metadata.clone(),
            })
            .collect()
    } else {
        Vec::new()
    }
}

/// Gets metadata for a specific benchmark.
///
/// # Arguments
///
/// * `name` - Name of the benchmark
///
/// # Returns
///
/// Option containing the metadata if found.
pub fn get_benchmark_metadata(name: &str) -> Option<BenchmarkMetadata> {
    let lock = REGISTRY.read().unwrap();
    lock.as_ref()
        .and_then(|registry| registry.benchmarks.get(name).cloned())
}

/// Filters benchmarks by category.
///
/// # Arguments
///
/// * `category` - The category to filter by
///
/// # Returns
///
/// Vector of benchmarks in the specified category.
pub fn filter_by_category(category: Category) -> Vec<BenchmarkInfo> {
    discover_benchmarks()
        .into_iter()
        .filter(|info| info.metadata.category == category)
        .collect()
}

/// Filters benchmarks by tag.
///
/// # Arguments
///
/// * `tag` - The tag to filter by
///
/// # Returns
///
/// Vector of benchmarks with the specified tag.
pub fn filter_by_tag(tag: &str) -> Vec<BenchmarkInfo> {
    discover_benchmarks()
        .into_iter()
        .filter(|info| info.metadata.has_tag(tag))
        .collect()
}

/// Clears all registered benchmarks (mainly for testing).
pub fn clear_registry() {
    let mut lock = REGISTRY.write().unwrap();
    *lock = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_as_str() {
        assert_eq!(Category::Parsing.as_str(), "parsing");
        assert_eq!(Category::Conversion.as_str(), "conversion");
    }

    #[test]
    fn test_coverage() {
        let coverage = Coverage::new(80, 100);
        assert_eq!(coverage.percentage, 80.0);
        assert!(coverage.meets_threshold(75.0));
        assert!(!coverage.meets_threshold(90.0));
    }

    #[test]
    fn test_benchmark_metadata() {
        let metadata = BenchmarkMetadata::new(Category::Parsing, "Test benchmark")
            .with_tag("fast")
            .with_tag("core");

        assert_eq!(metadata.category, Category::Parsing);
        assert!(metadata.has_tag("fast"));
        assert!(metadata.has_tag("core"));
        assert!(!metadata.has_tag("slow"));
    }

    // Combined test to avoid race conditions with global registry
    #[test]
    fn test_registry_operations() {
        // Test basic registry
        clear_registry();
        let metadata = BenchmarkMetadata::new(Category::Parsing, "Parse test");
        register_benchmark("test1", metadata);
        let benchmarks = discover_benchmarks();
        assert_eq!(benchmarks.len(), 1);
        assert_eq!(benchmarks[0].name, "test1");

        // Test filter by category
        clear_registry();
        register_benchmark(
            "parse1",
            BenchmarkMetadata::new(Category::Parsing, "Parse 1"),
        );
        register_benchmark(
            "parse2",
            BenchmarkMetadata::new(Category::Parsing, "Parse 2"),
        );
        register_benchmark(
            "convert1",
            BenchmarkMetadata::new(Category::Conversion, "Convert 1"),
        );
        let parsing = filter_by_category(Category::Parsing);
        assert_eq!(parsing.len(), 2);
        let conversion = filter_by_category(Category::Conversion);
        assert_eq!(conversion.len(), 1);

        // Test filter by tag
        clear_registry();
        register_benchmark(
            "bench1",
            BenchmarkMetadata::new(Category::Parsing, "Bench 1").with_tag("fast"),
        );
        register_benchmark(
            "bench2",
            BenchmarkMetadata::new(Category::Parsing, "Bench 2").with_tag("slow"),
        );
        register_benchmark(
            "bench3",
            BenchmarkMetadata::new(Category::Parsing, "Bench 3").with_tag("fast"),
        );
        let fast = filter_by_tag("fast");
        assert_eq!(fast.len(), 2);

        clear_registry();
    }
}

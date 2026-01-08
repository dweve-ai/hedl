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

//! Centralized benchmark configuration.
//!
//! Provides standardized configuration for benchmark execution including
//! sizes, iterations, warmup periods, and export settings.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Standard dataset sizes for benchmarks.
pub const STANDARD_SIZES: &[usize] = &[10, 100, 1_000, 10_000, 100_000];

/// Large dataset sizes for stress testing.
pub const LARGE_SIZES: &[usize] = &[100_000, 500_000, 1_000_000];

/// Default warmup duration for stable measurements.
pub const DEFAULT_WARMUP: Duration = Duration::from_millis(100);

/// Default iteration count for small datasets.
pub const DEFAULT_ITERATIONS_SMALL: u64 = 1_000;

/// Default iteration count for medium datasets.
pub const DEFAULT_ITERATIONS_MEDIUM: u64 = 100;

/// Default iteration count for large datasets.
pub const DEFAULT_ITERATIONS_LARGE: u64 = 10;

/// Export format options for benchmark reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportFormat {
    /// JSON format for machine-readable reports.
    Json,
    /// Markdown format for documentation.
    Markdown,
    /// HTML format for visual reports.
    Html,
    /// Console output for immediate feedback.
    Console,
}

/// Centralized benchmark configuration.
///
/// Manages all configuration for benchmark execution including dataset sizes,
/// iteration counts, warmup periods, baseline paths, and export formats.
///
/// # Example
///
/// ```no_run
/// use hedl_bench::core::config::{BenchConfig, ExportFormat, STANDARD_SIZES};
///
/// let config = BenchConfig::default()
///     .with_sizes(STANDARD_SIZES)
///     .with_export_format(ExportFormat::Html);
/// ```
#[derive(Debug, Clone)]
pub struct BenchConfig {
    /// Dataset sizes to test.
    pub sizes: Vec<usize>,
    /// Iteration count per dataset size.
    pub iterations: HashMap<usize, u64>,
    /// Warmup duration before measurements.
    pub warmup: Duration,
    /// Path to baseline data for regression detection.
    pub baseline_path: PathBuf,
    /// Export formats for reports.
    pub export_formats: Vec<ExportFormat>,
}

impl BenchConfig {
    /// Creates a new benchmark configuration with specified sizes.
    pub fn new(sizes: &[usize]) -> Self {
        let mut iterations = HashMap::new();
        for &size in sizes {
            iterations.insert(size, Self::default_iterations_for_size(size));
        }

        Self {
            sizes: sizes.to_vec(),
            iterations,
            warmup: DEFAULT_WARMUP,
            baseline_path: PathBuf::from("baselines/current.json"),
            export_formats: vec![ExportFormat::Console, ExportFormat::Json],
        }
    }

    /// Returns the default iteration count for a given dataset size.
    fn default_iterations_for_size(size: usize) -> u64 {
        match size {
            s if s <= 100 => DEFAULT_ITERATIONS_SMALL,
            s if s <= 10_000 => DEFAULT_ITERATIONS_MEDIUM,
            _ => DEFAULT_ITERATIONS_LARGE,
        }
    }

    /// Sets custom dataset sizes.
    pub fn with_sizes(mut self, sizes: &[usize]) -> Self {
        self.sizes = sizes.to_vec();
        for &size in sizes {
            self.iterations.entry(size).or_insert_with(|| Self::default_iterations_for_size(size));
        }
        self
    }

    /// Sets custom iteration count for a specific size.
    pub fn with_iterations(mut self, size: usize, iterations: u64) -> Self {
        self.iterations.insert(size, iterations);
        self
    }

    /// Sets warmup duration.
    pub fn with_warmup(mut self, warmup: Duration) -> Self {
        self.warmup = warmup;
        self
    }

    /// Sets baseline path for regression detection.
    pub fn with_baseline_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.baseline_path = path.into();
        self
    }

    /// Adds an export format.
    pub fn with_export_format(mut self, format: ExportFormat) -> Self {
        if !self.export_formats.contains(&format) {
            self.export_formats.push(format);
        }
        self
    }

    /// Sets all export formats.
    pub fn with_export_formats(mut self, formats: Vec<ExportFormat>) -> Self {
        self.export_formats = formats;
        self
    }

    /// Gets the iteration count for a specific size.
    pub fn get_iterations(&self, size: usize) -> u64 {
        *self
            .iterations
            .get(&size)
            .unwrap_or(&DEFAULT_ITERATIONS_MEDIUM)
    }

    /// Returns whether a specific export format is enabled.
    pub fn has_format(&self, format: ExportFormat) -> bool {
        self.export_formats.contains(&format)
    }
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self::new(STANDARD_SIZES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BenchConfig::default();
        assert_eq!(config.sizes.len(), STANDARD_SIZES.len());
        assert!(config.has_format(ExportFormat::Console));
        assert!(config.has_format(ExportFormat::Json));
    }

    #[test]
    fn test_custom_sizes() {
        let config = BenchConfig::default().with_sizes(&[50, 500]);
        assert_eq!(config.sizes, vec![50, 500]);
        assert_eq!(config.get_iterations(50), DEFAULT_ITERATIONS_SMALL);
        assert_eq!(config.get_iterations(500), DEFAULT_ITERATIONS_MEDIUM);
    }

    #[test]
    fn test_custom_iterations() {
        let config = BenchConfig::default().with_iterations(1_000, 50);
        assert_eq!(config.get_iterations(1_000), 50);
    }

    #[test]
    fn test_export_formats() {
        let config = BenchConfig::default()
            .with_export_format(ExportFormat::Html)
            .with_export_format(ExportFormat::Markdown);
        assert!(config.has_format(ExportFormat::Html));
        assert!(config.has_format(ExportFormat::Markdown));
    }

    #[test]
    fn test_baseline_path() {
        let config = BenchConfig::default().with_baseline_path("custom/baseline.json");
        assert_eq!(config.baseline_path, PathBuf::from("custom/baseline.json"));
    }
}

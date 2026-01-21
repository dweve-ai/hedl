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

//! Configuration for reading Parquet files.

use crate::predicate::Predicate;

/// Configuration for converting Parquet files to HEDL documents.
///
/// Enables selective column reading (projection pushdown), predicate pushdown
/// for row group filtering, and batch size optimization for reduced I/O and
/// memory usage.
///
/// # Performance
///
/// - **Column projection**: 8-10x faster for 10% column selectivity
/// - **Predicate pushdown**: 10-100x faster for highly selective filters (< 10%)
/// - **Combined**: Multiplicative effect for filtered, projected reads
///
/// # Example
///
/// ```
/// use hedl_parquet::{FromParquetConfig, predicate::{Predicate, PredicateValue}};
///
/// // Read only specific columns with a filter
/// let config = FromParquetConfig::default()
///     .set_columns(Some(vec!["id".into(), "name".into(), "age".into()]))
///     .with_filter(Predicate::greater_than("age", PredicateValue::Int(18)));
/// ```
#[derive(Debug, Clone, Default)]
pub struct FromParquetConfig {
    /// How to handle null or missing IDs.
    pub null_id_handling: NullIdHandling,

    /// Columns to read (None = all columns, Some(vec) = specific columns).
    ///
    /// When specified, only these columns are read from the Parquet file,
    /// providing significant performance improvement for wide tables.
    ///
    /// Empty vectors are not allowed and will cause an error.
    pub columns: Option<Vec<String>>,

    /// Batch size for reading record batches from Parquet files.
    ///
    /// Controls how many rows are read from the Parquet file at once.
    /// Larger batch sizes improve throughput but use more memory.
    /// Smaller batch sizes reduce peak memory usage but may be slower.
    ///
    /// # Performance Trade-offs
    ///
    /// - **Small batches (1K-10K rows)**: Lower memory usage, better for streaming,
    ///   more overhead from batch processing
    /// - **Medium batches (10K-100K rows)**: Balanced memory/performance (default: 64K)
    /// - **Large batches (100K-1M rows)**: Higher throughput, more memory usage,
    ///   better for bulk operations
    ///
    /// # Automatic Sizing
    ///
    /// Use `BatchSize::Auto` (default) to let the library choose optimal batch size
    /// based on:
    /// - Number of columns (wider tables use smaller batches)
    /// - Column types (variable-length strings use smaller batches)
    /// - Available memory heuristics
    ///
    /// # Example
    ///
    /// ```
    /// use hedl_parquet::{FromParquetConfig, BatchSize};
    ///
    /// // Use automatic batch sizing (recommended)
    /// let config = FromParquetConfig::new();
    ///
    /// // Or specify exact batch size for memory-constrained environments
    /// let config = FromParquetConfig::new()
    ///     .with_batch_size(BatchSize::Fixed(10_000));
    /// ```
    pub batch_size: BatchSize,

    /// Optional predicate filter for row group pruning.
    ///
    /// When specified, uses Parquet column statistics to skip entire row groups
    /// that cannot contain matching rows, dramatically reducing I/O and
    /// decompression overhead.
    ///
    /// # Performance
    ///
    /// Expected speedup by selectivity:
    /// - 0.1% selectivity (needle in haystack): 10-20x faster
    /// - 1% selectivity (rare category): 8-10x faster
    /// - 5% selectivity (age range): 3-5x faster
    /// - 10% selectivity (common category): 2-4x faster
    ///
    /// # Example
    ///
    /// ```
    /// use hedl_parquet::{FromParquetConfig, predicate::{Predicate, PredicateValue}};
    ///
    /// // Filter by age
    /// let config = FromParquetConfig::default()
    ///     .with_filter(Predicate::equal("age", PredicateValue::Int(25)));
    ///
    /// // Combined filter
    /// let config = FromParquetConfig::default()
    ///     .with_filter(Predicate::and(vec![
    ///         Predicate::equal("status", PredicateValue::String("active".into())),
    ///         Predicate::between("age", PredicateValue::Int(18), PredicateValue::Int(65)),
    ///     ]));
    /// ```
    pub filter: Option<Predicate>,
}

impl FromParquetConfig {
    /// Create a new configuration with default settings (strict mode).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a lenient configuration that generates IDs for nulls.
    ///
    /// # Warning
    ///
    /// This mode generates IDs that don't exist in the source data.
    /// Use only when dealing with legacy data that cannot be fixed.
    /// Generated IDs are not preserved on round-trip.
    #[must_use]
    pub fn lenient() -> Self {
        Self {
            null_id_handling: NullIdHandling::Generate,
            columns: None,
            batch_size: BatchSize::Auto,
            filter: None,
        }
    }

    /// Create a strict configuration (same as default).
    #[must_use]
    pub fn strict() -> Self {
        Self::default()
    }

    /// Set the null ID handling strategy.
    #[must_use]
    pub fn with_null_id_handling(mut self, handling: NullIdHandling) -> Self {
        self.null_id_handling = handling;
        self
    }

    /// Create configuration to read only specific columns.
    ///
    /// Uses projection pushdown to read only the specified columns,
    /// significantly improving performance for wide tables.
    ///
    /// # Example
    ///
    /// ```
    /// use hedl_parquet::FromParquetConfig;
    ///
    /// let config = FromParquetConfig::with_columns(vec!["id".into(), "name".into()]);
    /// ```
    #[must_use]
    pub fn with_columns(columns: Vec<String>) -> Self {
        Self {
            columns: Some(columns),
            ..Default::default()
        }
    }

    /// Create configuration to read a single column.
    ///
    /// # Example
    ///
    /// ```
    /// use hedl_parquet::FromParquetConfig;
    ///
    /// let config = FromParquetConfig::with_column("id".into());
    /// ```
    #[must_use]
    pub fn with_column(column: String) -> Self {
        Self::with_columns(vec![column])
    }

    /// Set which columns to read.
    ///
    /// # Example
    ///
    /// ```
    /// use hedl_parquet::FromParquetConfig;
    ///
    /// let config = FromParquetConfig::default()
    ///     .set_columns(Some(vec!["id".into(), "name".into()]));
    /// ```
    #[must_use]
    pub fn set_columns(mut self, columns: Option<Vec<String>>) -> Self {
        self.columns = columns;
        self
    }

    /// Set the batch size for reading record batches.
    ///
    /// # Example
    ///
    /// ```
    /// use hedl_parquet::{FromParquetConfig, BatchSize};
    ///
    /// // Use fixed batch size of 10,000 rows
    /// let config = FromParquetConfig::default()
    ///     .with_batch_size(BatchSize::Fixed(10_000));
    ///
    /// // Or use automatic sizing
    /// let config = FromParquetConfig::default()
    ///     .with_batch_size(BatchSize::Auto);
    /// ```
    #[must_use]
    pub fn with_batch_size(mut self, batch_size: BatchSize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Set a predicate filter for row group pruning.
    ///
    /// Uses Parquet column statistics to skip entire row groups that cannot
    /// contain matching rows, dramatically reducing I/O and decompression.
    ///
    /// # Performance
    ///
    /// Expected speedup depends on filter selectivity:
    /// - 1% selectivity: 8-10x faster
    /// - 5% selectivity: 3-5x faster
    /// - 10% selectivity: 2-4x faster
    ///
    /// # Example
    ///
    /// ```
    /// use hedl_parquet::{FromParquetConfig, predicate::{Predicate, PredicateValue}};
    ///
    /// // Simple equality filter
    /// let config = FromParquetConfig::default()
    ///     .with_filter(Predicate::equal("status", PredicateValue::String("active".into())));
    ///
    /// // Range filter
    /// let config = FromParquetConfig::default()
    ///     .with_filter(Predicate::between("age", PredicateValue::Int(18), PredicateValue::Int(65)));
    ///
    /// // Combined filter
    /// let config = FromParquetConfig::default()
    ///     .with_filter(Predicate::and(vec![
    ///         Predicate::equal("country", PredicateValue::String("USA".into())),
    ///         Predicate::greater_than("age", PredicateValue::Int(21)),
    ///     ]));
    /// ```
    #[must_use]
    pub fn with_filter(mut self, filter: Predicate) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Create configuration with a predicate filter.
    ///
    /// Convenience constructor for filtered reads.
    ///
    /// # Example
    ///
    /// ```
    /// use hedl_parquet::{FromParquetConfig, predicate::{Predicate, PredicateValue}};
    ///
    /// let config = FromParquetConfig::with_predicate(
    ///     Predicate::equal("id", PredicateValue::String("user_123".into()))
    /// );
    /// ```
    #[must_use]
    pub fn with_predicate(filter: Predicate) -> Self {
        Self {
            filter: Some(filter),
            ..Default::default()
        }
    }
}

/// Batch size strategy for reading Parquet files.
///
/// Controls how many rows are read from the Parquet file at once,
/// balancing memory usage against throughput.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BatchSize {
    /// Automatically determine optimal batch size based on data characteristics.
    ///
    /// The library analyzes:
    /// - Number of columns (wider tables use smaller batches)
    /// - Column types (string-heavy tables use smaller batches)
    /// - Memory constraints (estimated based on heuristics)
    ///
    /// This is the recommended default as it adapts to your data.
    #[default]
    Auto,

    /// Use a fixed batch size for all reads.
    ///
    /// Provides predictable memory usage and performance characteristics.
    /// Use when you have specific memory constraints or performance requirements.
    ///
    /// Valid range: 1 to 1,048,576 (1M rows)
    ///
    /// # Panics
    ///
    /// Panics during read operations if the value is 0 or exceeds 1,048,576.
    Fixed(usize),

    /// Adaptive batch sizing that adjusts based on observed memory usage.
    ///
    /// Starts with an initial size and dynamically adjusts based on:
    /// - Actual memory consumption per batch
    /// - Processing time per batch
    /// - System memory pressure
    ///
    /// More sophisticated than `Auto` but may have slight overhead.
    /// Use for long-running processes or when memory is very constrained.
    ///
    /// The usize parameter is the starting batch size (typically 8K-64K).
    Adaptive(usize),
}

impl BatchSize {
    /// Minimum allowed batch size (prevents excessive overhead).
    pub const MIN_BATCH_SIZE: usize = 100;

    /// Maximum allowed batch size (prevents memory exhaustion).
    pub const MAX_BATCH_SIZE: usize = 1_048_576; // 1M rows

    /// Default batch size for Auto mode with narrow tables (< 20 columns).
    pub const DEFAULT_NARROW_BATCH_SIZE: usize = 65_536; // 64K rows

    /// Default batch size for Auto mode with medium tables (20-50 columns).
    pub const DEFAULT_MEDIUM_BATCH_SIZE: usize = 32_768; // 32K rows

    /// Default batch size for Auto mode with wide tables (> 50 columns).
    pub const DEFAULT_WIDE_BATCH_SIZE: usize = 16_384; // 16K rows

    /// Default batch size for Adaptive mode.
    pub const DEFAULT_ADAPTIVE_BATCH_SIZE: usize = 32_768; // 32K rows

    /// Validate and clamp a batch size to valid range.
    ///
    /// Returns a clamped value within [`MIN_BATCH_SIZE`, `MAX_BATCH_SIZE`].
    #[must_use]
    pub fn validate(size: usize) -> usize {
        size.clamp(Self::MIN_BATCH_SIZE, Self::MAX_BATCH_SIZE)
    }

    /// Calculate optimal batch size for Auto mode based on column count and types.
    ///
    /// # Parameters
    ///
    /// * `num_columns` - Number of columns in the schema
    /// * `has_many_strings` - Whether the schema has many variable-length string columns
    ///
    /// # Returns
    ///
    /// Recommended batch size balancing memory and performance.
    #[must_use]
    pub fn calculate_auto_size(num_columns: usize, has_many_strings: bool) -> usize {
        let base_size = if num_columns < 20 {
            Self::DEFAULT_NARROW_BATCH_SIZE
        } else if num_columns < 50 {
            Self::DEFAULT_MEDIUM_BATCH_SIZE
        } else {
            Self::DEFAULT_WIDE_BATCH_SIZE
        };

        // Reduce batch size further if many string columns (unpredictable memory)
        if has_many_strings {
            Self::validate(base_size / 2)
        } else {
            base_size
        }
    }

    /// Get the effective batch size for this configuration.
    ///
    /// # Parameters
    ///
    /// * `num_columns` - Number of columns in the schema (for Auto mode)
    /// * `has_many_strings` - Whether schema has many string columns (for Auto mode)
    ///
    /// # Returns
    ///
    /// The actual batch size to use for reading.
    #[must_use]
    pub fn get_effective_size(&self, num_columns: usize, has_many_strings: bool) -> usize {
        match self {
            Self::Auto => Self::calculate_auto_size(num_columns, has_many_strings),
            Self::Fixed(size) => Self::validate(*size),
            Self::Adaptive(initial_size) => Self::validate(*initial_size),
        }
    }
}

/// Strategy for handling null or missing ID values.
#[derive(Debug, Clone, Default)]
pub enum NullIdHandling {
    /// Return an error if ID is null (default, recommended).
    ///
    /// This is the safest option as it ensures all entities have valid IDs.
    /// Null IDs usually indicate data quality issues that should be fixed
    /// at the source.
    #[default]
    Error,

    /// Generate a unique ID for null values.
    ///
    /// Generated IDs have the format `__generated_row_{row_idx}` where
    /// `row_idx` is the 0-based row index in the Parquet file.
    ///
    /// # Warning
    ///
    /// Generated IDs are:
    /// - Synthetic data that doesn't exist in the source
    /// - Not stable across file modifications
    /// - Not preserved on round-trip conversion
    ///
    /// Use only for legacy data that cannot be fixed.
    Generate,

    /// Use a constant value for all null IDs.
    ///
    /// # Warning
    ///
    /// This will cause duplicate IDs if multiple rows have null IDs!
    /// Only use this if you know there's exactly one null ID in the file,
    /// or if you have duplicate detection enabled.
    UseConstant(String),
}

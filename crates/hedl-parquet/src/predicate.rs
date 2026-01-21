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

//! Predicate pushdown support for efficient Parquet filtering.
//!
//! This module provides predicate expressions that enable row group pruning
//! by leveraging Parquet column statistics (min/max values, null counts).
//!
//! # Performance
//!
//! Predicate pushdown can provide 10-100x performance improvement for selective
//! queries by eliminating unnecessary I/O and decompression.
//!
//! # Example
//!
//! ```
//! use hedl_parquet::predicate::{Predicate, PredicateValue};
//!
//! // Filter users where age = 25
//! let pred = Predicate::equal("age", PredicateValue::Int(25));
//!
//! // Filter users where age BETWEEN 30 AND 40
//! let pred = Predicate::between("age", PredicateValue::Int(30), PredicateValue::Int(40));
//!
//! // Combine predicates with AND
//! let pred = Predicate::and(vec![
//!     Predicate::equal("status", PredicateValue::String("active".into())),
//!     Predicate::greater_than("age", PredicateValue::Int(18)),
//! ]);
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{
    Array, BooleanArray, Float32Array, Float64Array, Int16Array, Int32Array, Int64Array, Int8Array,
    RecordBatch, Scalar, StringArray, UInt16Array, UInt32Array, UInt64Array, UInt8Array,
};
use arrow::compute;
use arrow::compute::kernels::cmp;
use arrow::datatypes::{DataType, Schema};
use parquet::file::metadata::ParquetMetaData;
use parquet::file::statistics::Statistics;

use hedl_core::{HedlError, HedlErrorKind};

/// A value used in predicate comparisons.
///
/// This is separate from `hedl_core::Value` to support efficient comparison
/// with Parquet column statistics without requiring full HEDL value semantics.
#[derive(Debug, Clone, PartialEq)]
pub enum PredicateValue {
    /// Null value for IS NULL / IS NOT NULL comparisons.
    Null,
    /// Boolean value.
    Bool(bool),
    /// 64-bit signed integer.
    Int(i64),
    /// 64-bit floating point.
    Float(f64),
    /// String value.
    String(String),
}

impl PredicateValue {
    /// Compare two predicate values.
    ///
    /// Returns None if types are incompatible.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => Some(a.cmp(b)),
            (Self::Float(a), Self::Float(b)) => a.partial_cmp(b),
            (Self::String(a), Self::String(b)) => Some(a.cmp(b)),
            (Self::Bool(a), Self::Bool(b)) => Some(a.cmp(b)),
            // Cross-type: int can be compared to float
            (Self::Int(a), Self::Float(b)) => (*a as f64).partial_cmp(b),
            (Self::Float(a), Self::Int(b)) => a.partial_cmp(&(*b as f64)),
            _ => None,
        }
    }

    /// Check if this value is less than another.
    #[must_use]
    pub fn lt(&self, other: &Self) -> Option<bool> {
        self.partial_cmp(other)
            .map(|o| o == std::cmp::Ordering::Less)
    }

    /// Check if this value is less than or equal to another.
    #[must_use]
    pub fn le(&self, other: &Self) -> Option<bool> {
        self.partial_cmp(other)
            .map(|o| o != std::cmp::Ordering::Greater)
    }

    /// Check if this value is greater than another.
    #[must_use]
    pub fn gt(&self, other: &Self) -> Option<bool> {
        self.partial_cmp(other)
            .map(|o| o == std::cmp::Ordering::Greater)
    }

    /// Check if this value is greater than or equal to another.
    #[must_use]
    pub fn ge(&self, other: &Self) -> Option<bool> {
        self.partial_cmp(other)
            .map(|o| o != std::cmp::Ordering::Less)
    }
}

impl std::fmt::Display for PredicateValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "NULL"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "'{s}'"),
        }
    }
}

/// Predicate expression for filtering Parquet data.
///
/// Predicates enable efficient row group pruning by leveraging Parquet column
/// statistics. Only row groups that might contain matching rows are read.
#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    /// Equality comparison: column = value
    Equal(String, PredicateValue),
    /// Inequality comparison: column != value
    NotEqual(String, PredicateValue),
    /// Less than comparison: column < value
    LessThan(String, PredicateValue),
    /// Less than or equal comparison: column <= value
    LessThanOrEqual(String, PredicateValue),
    /// Greater than comparison: column > value
    GreaterThan(String, PredicateValue),
    /// Greater than or equal comparison: column >= value
    GreaterThanOrEqual(String, PredicateValue),
    /// Range comparison: column BETWEEN min AND max (inclusive)
    Between(String, PredicateValue, PredicateValue),
    /// Set membership: column IN (value1, value2, ...)
    In(String, Vec<PredicateValue>),
    /// Negated set membership: column NOT IN (value1, value2, ...)
    NotIn(String, Vec<PredicateValue>),
    /// Null check: column IS NULL
    IsNull(String),
    /// Not null check: column IS NOT NULL
    IsNotNull(String),
    /// Logical AND: all predicates must match
    And(Vec<Predicate>),
    /// Logical OR: any predicate must match
    Or(Vec<Predicate>),
    /// Logical NOT: negate the predicate
    Not(Box<Predicate>),
}

impl Predicate {
    // ==================== Constructors ====================

    /// Create an equality predicate: column = value
    pub fn equal(column: impl Into<String>, value: PredicateValue) -> Self {
        Self::Equal(column.into(), value)
    }

    /// Create an inequality predicate: column != value
    pub fn not_equal(column: impl Into<String>, value: PredicateValue) -> Self {
        Self::NotEqual(column.into(), value)
    }

    /// Create a less than predicate: column < value
    pub fn less_than(column: impl Into<String>, value: PredicateValue) -> Self {
        Self::LessThan(column.into(), value)
    }

    /// Create a less than or equal predicate: column <= value
    pub fn less_than_or_equal(column: impl Into<String>, value: PredicateValue) -> Self {
        Self::LessThanOrEqual(column.into(), value)
    }

    /// Create a greater than predicate: column > value
    pub fn greater_than(column: impl Into<String>, value: PredicateValue) -> Self {
        Self::GreaterThan(column.into(), value)
    }

    /// Create a greater than or equal predicate: column >= value
    pub fn greater_than_or_equal(column: impl Into<String>, value: PredicateValue) -> Self {
        Self::GreaterThanOrEqual(column.into(), value)
    }

    /// Create a range predicate: column BETWEEN min AND max (inclusive)
    pub fn between(column: impl Into<String>, min: PredicateValue, max: PredicateValue) -> Self {
        Self::Between(column.into(), min, max)
    }

    /// Create a set membership predicate: column IN (values...)
    pub fn in_set(column: impl Into<String>, values: Vec<PredicateValue>) -> Self {
        Self::In(column.into(), values)
    }

    /// Create a negated set membership predicate: column NOT IN (values...)
    pub fn not_in_set(column: impl Into<String>, values: Vec<PredicateValue>) -> Self {
        Self::NotIn(column.into(), values)
    }

    /// Create a null check predicate: column IS NULL
    pub fn is_null(column: impl Into<String>) -> Self {
        Self::IsNull(column.into())
    }

    /// Create a not null check predicate: column IS NOT NULL
    pub fn is_not_null(column: impl Into<String>) -> Self {
        Self::IsNotNull(column.into())
    }

    /// Create a logical AND of predicates.
    ///
    /// All predicates must match for a row to pass.
    #[must_use]
    pub fn and(predicates: Vec<Predicate>) -> Self {
        Self::And(predicates)
    }

    /// Create a logical OR of predicates.
    ///
    /// Any predicate matching is sufficient for a row to pass.
    #[must_use]
    pub fn or(predicates: Vec<Predicate>) -> Self {
        Self::Or(predicates)
    }

    /// Create a logical NOT of a predicate.
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn not(predicate: Predicate) -> Self {
        Self::Not(Box::new(predicate))
    }

    // ==================== Row Group Pruning ====================

    /// Determine if a row group can be skipped based on column statistics.
    ///
    /// Returns `true` if the row group CANNOT contain matching rows and can be safely skipped.
    /// Returns `false` if the row group MIGHT contain matching rows and must be read.
    ///
    /// This method is conservative: it never returns `true` unless it's certain
    /// no rows in the row group can match. False negatives (returning `false` when
    /// the row group could be skipped) are acceptable; false positives (returning
    /// `true` when matching rows exist) are NOT acceptable.
    #[must_use]
    pub fn can_skip_row_group(&self, stats: &RowGroupStatistics) -> bool {
        match self {
            Self::Equal(col, val) => {
                // Skip if value is outside [min, max] range
                if let Some(col_stats) = stats.columns.get(col) {
                    // If all values are null in this row group, and we're looking for non-null, skip
                    if col_stats.all_null() {
                        return true;
                    }
                    // Check if value is outside the range
                    if let (Some(min), Some(max)) = (&col_stats.min_value, &col_stats.max_value) {
                        // Skip if val < min or val > max
                        if val.lt(min).unwrap_or(false) || val.gt(max).unwrap_or(false) {
                            return true;
                        }
                    }
                }
                false
            }
            Self::NotEqual(col, val) => {
                // Skip only if min == max == val (all values equal to the value we're excluding)
                if let Some(col_stats) = stats.columns.get(col) {
                    if let (Some(min), Some(max)) = (&col_stats.min_value, &col_stats.max_value) {
                        if min == val && max == val && col_stats.null_count == 0 {
                            return true;
                        }
                    }
                }
                false
            }
            Self::LessThan(col, val) => {
                // Skip if min >= val (all values are >= val, none are < val)
                if let Some(col_stats) = stats.columns.get(col) {
                    if col_stats.all_null() {
                        return true;
                    }
                    if let Some(min) = &col_stats.min_value {
                        if min.ge(val).unwrap_or(false) {
                            return true;
                        }
                    }
                }
                false
            }
            Self::LessThanOrEqual(col, val) => {
                // Skip if min > val
                if let Some(col_stats) = stats.columns.get(col) {
                    if col_stats.all_null() {
                        return true;
                    }
                    if let Some(min) = &col_stats.min_value {
                        if min.gt(val).unwrap_or(false) {
                            return true;
                        }
                    }
                }
                false
            }
            Self::GreaterThan(col, val) => {
                // Skip if max <= val
                if let Some(col_stats) = stats.columns.get(col) {
                    if col_stats.all_null() {
                        return true;
                    }
                    if let Some(max) = &col_stats.max_value {
                        if max.le(val).unwrap_or(false) {
                            return true;
                        }
                    }
                }
                false
            }
            Self::GreaterThanOrEqual(col, val) => {
                // Skip if max < val
                if let Some(col_stats) = stats.columns.get(col) {
                    if col_stats.all_null() {
                        return true;
                    }
                    if let Some(max) = &col_stats.max_value {
                        if max.lt(val).unwrap_or(false) {
                            return true;
                        }
                    }
                }
                false
            }
            Self::Between(col, min_val, max_val) => {
                // Skip if no overlap: row_group.max < min_val OR row_group.min > max_val
                if let Some(col_stats) = stats.columns.get(col) {
                    if col_stats.all_null() {
                        return true;
                    }
                    if let (Some(rg_min), Some(rg_max)) =
                        (&col_stats.min_value, &col_stats.max_value)
                    {
                        // No overlap if rg_max < min_val or rg_min > max_val
                        if rg_max.lt(min_val).unwrap_or(false)
                            || rg_min.gt(max_val).unwrap_or(false)
                        {
                            return true;
                        }
                    }
                }
                false
            }
            Self::In(col, values) => {
                // Skip if no values overlap with [min, max]
                if values.is_empty() {
                    return true; // Empty IN set matches nothing
                }
                if let Some(col_stats) = stats.columns.get(col) {
                    if col_stats.all_null() {
                        return true;
                    }
                    if let (Some(rg_min), Some(rg_max)) =
                        (&col_stats.min_value, &col_stats.max_value)
                    {
                        // Skip if all values are outside the range
                        let any_in_range = values.iter().any(|v| {
                            !(v.lt(rg_min).unwrap_or(false) || v.gt(rg_max).unwrap_or(false))
                        });
                        if !any_in_range {
                            return true;
                        }
                    }
                }
                false
            }
            Self::NotIn(col, values) => {
                // Skip only if min == max and that value is in the excluded set
                if values.is_empty() {
                    return false; // Empty NOT IN matches everything
                }
                if let Some(col_stats) = stats.columns.get(col) {
                    if let (Some(rg_min), Some(rg_max)) =
                        (&col_stats.min_value, &col_stats.max_value)
                    {
                        // If all values in row group are the same and that value is in the exclusion set
                        if rg_min == rg_max && col_stats.null_count == 0 && values.contains(rg_min)
                        {
                            return true;
                        }
                    }
                }
                false
            }
            Self::IsNull(col) => {
                // Skip if null_count == 0 (no nulls in this row group)
                if let Some(col_stats) = stats.columns.get(col) {
                    if col_stats.null_count == 0 {
                        return true;
                    }
                }
                false
            }
            Self::IsNotNull(col) => {
                // Skip if all values are null
                if let Some(col_stats) = stats.columns.get(col) {
                    if col_stats.all_null() {
                        return true;
                    }
                }
                false
            }
            Self::And(predicates) => {
                // Skip if ANY predicate can skip (AND requires all to match)
                predicates.iter().any(|p| p.can_skip_row_group(stats))
            }
            Self::Or(predicates) => {
                // Skip only if ALL predicates can skip (OR needs only one to match)
                if predicates.is_empty() {
                    return true; // Empty OR matches nothing
                }
                predicates.iter().all(|p| p.can_skip_row_group(stats))
            }
            Self::Not(pred) => {
                // NOT is complex for row group pruning - be conservative
                // We can only skip if the inner predicate would match ALL rows
                // This is hard to determine, so we don't skip
                match pred.as_ref() {
                    // NOT IsNull -> IsNotNull
                    Predicate::IsNull(col) => {
                        Self::IsNotNull(col.clone()).can_skip_row_group(stats)
                    }
                    // NOT IsNotNull -> IsNull
                    Predicate::IsNotNull(col) => {
                        Self::IsNull(col.clone()).can_skip_row_group(stats)
                    }
                    _ => false,
                }
            }
        }
    }
}

impl std::fmt::Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Equal(col, val) => write!(f, "{col} = {val}"),
            Self::NotEqual(col, val) => write!(f, "{col} != {val}"),
            Self::LessThan(col, val) => write!(f, "{col} < {val}"),
            Self::LessThanOrEqual(col, val) => write!(f, "{col} <= {val}"),
            Self::GreaterThan(col, val) => write!(f, "{col} > {val}"),
            Self::GreaterThanOrEqual(col, val) => write!(f, "{col} >= {val}"),
            Self::Between(col, min, max) => write!(f, "{col} BETWEEN {min} AND {max}"),
            Self::In(col, vals) => {
                let vals_str: Vec<String> =
                    vals.iter().map(std::string::ToString::to_string).collect();
                write!(f, "{} IN ({})", col, vals_str.join(", "))
            }
            Self::NotIn(col, vals) => {
                let vals_str: Vec<String> =
                    vals.iter().map(std::string::ToString::to_string).collect();
                write!(f, "{} NOT IN ({})", col, vals_str.join(", "))
            }
            Self::IsNull(col) => write!(f, "{col} IS NULL"),
            Self::IsNotNull(col) => write!(f, "{col} IS NOT NULL"),
            Self::And(preds) => {
                let strs: Vec<String> = preds.iter().map(|p| format!("({p})")).collect();
                write!(f, "{}", strs.join(" AND "))
            }
            Self::Or(preds) => {
                let strs: Vec<String> = preds.iter().map(|p| format!("({p})")).collect();
                write!(f, "{}", strs.join(" OR "))
            }
            Self::Not(pred) => write!(f, "NOT ({pred})"),
        }
    }
}

/// Column statistics from a Parquet row group.
#[derive(Debug, Clone, Default)]
pub struct ColumnStatistics {
    /// Minimum value in the column (if available).
    pub min_value: Option<PredicateValue>,
    /// Maximum value in the column (if available).
    pub max_value: Option<PredicateValue>,
    /// Number of null values in the column.
    pub null_count: u64,
    /// Total number of rows in the row group.
    pub total_rows: u64,
    /// Whether statistics are present and valid.
    pub has_statistics: bool,
}

impl ColumnStatistics {
    /// Check if all values in this column are null.
    #[must_use]
    pub fn all_null(&self) -> bool {
        self.total_rows > 0 && self.null_count >= self.total_rows
    }

    /// Check if the column has no null values.
    #[must_use]
    pub fn no_nulls(&self) -> bool {
        self.null_count == 0
    }
}

/// Row group statistics for all columns.
#[derive(Debug, Clone, Default)]
pub struct RowGroupStatistics {
    /// Row group index (0-based).
    pub row_group_id: usize,
    /// Total number of rows in this row group.
    pub total_rows: u64,
    /// Column statistics indexed by column name.
    pub columns: HashMap<String, ColumnStatistics>,
}

/// Extract row group statistics from Parquet file metadata.
///
/// This function reads the column statistics from each row group in the Parquet
/// file metadata. These statistics are used for predicate pushdown to determine
/// which row groups can be skipped.
#[must_use]
pub fn extract_row_group_statistics(
    metadata: &ParquetMetaData,
    schema: &Arc<Schema>,
) -> Vec<RowGroupStatistics> {
    let mut all_stats = Vec::with_capacity(metadata.num_row_groups());

    for (rg_idx, row_group) in metadata.row_groups().iter().enumerate() {
        let total_rows = row_group.num_rows() as u64;
        let mut columns = HashMap::new();

        // Build a mapping from column index to column name
        let field_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

        for (col_idx, column_chunk) in row_group.columns().iter().enumerate() {
            // Get column name from schema
            let col_name = if col_idx < field_names.len() {
                field_names[col_idx].to_string()
            } else {
                continue; // Skip if we can't map the column
            };

            // Get the data type for this column
            let data_type = if col_idx < schema.fields().len() {
                schema.field(col_idx).data_type()
            } else {
                continue;
            };

            let col_stats = if let Some(stats) = column_chunk.statistics() {
                extract_column_statistics(stats, data_type, total_rows)
            } else {
                // No statistics available for this column
                ColumnStatistics {
                    min_value: None,
                    max_value: None,
                    null_count: 0,
                    total_rows,
                    has_statistics: false,
                }
            };

            columns.insert(col_name, col_stats);
        }

        all_stats.push(RowGroupStatistics {
            row_group_id: rg_idx,
            total_rows,
            columns,
        });
    }

    all_stats
}

/// Extract column statistics from Parquet statistics object.
fn extract_column_statistics(
    stats: &Statistics,
    data_type: &DataType,
    total_rows: u64,
) -> ColumnStatistics {
    let null_count = stats.null_count_opt().unwrap_or(0);

    // Extract min/max values based on data type
    let (min_value, max_value) = extract_min_max(stats, data_type);

    ColumnStatistics {
        min_value,
        max_value,
        null_count,
        total_rows,
        has_statistics: true,
    }
}

/// Extract min and max values from Parquet statistics.
fn extract_min_max(
    stats: &Statistics,
    data_type: &DataType,
) -> (Option<PredicateValue>, Option<PredicateValue>) {
    // Only process if statistics have both min and max
    // Use min_bytes_opt() and max_bytes_opt() which return Option<&[u8]>
    if stats.min_bytes_opt().is_none() || stats.max_bytes_opt().is_none() {
        return (None, None);
    }

    match data_type {
        DataType::Boolean => {
            if let Statistics::Boolean(bool_stats) = stats {
                let min = bool_stats.min_opt().map(|v| PredicateValue::Bool(*v));
                let max = bool_stats.max_opt().map(|v| PredicateValue::Bool(*v));
                (min, max)
            } else {
                (None, None)
            }
        }
        DataType::Int8 => {
            if let Statistics::Int32(int_stats) = stats {
                let min = int_stats
                    .min_opt()
                    .map(|v| PredicateValue::Int(i64::from(*v)));
                let max = int_stats
                    .max_opt()
                    .map(|v| PredicateValue::Int(i64::from(*v)));
                (min, max)
            } else {
                (None, None)
            }
        }
        DataType::Int16 => {
            if let Statistics::Int32(int_stats) = stats {
                let min = int_stats
                    .min_opt()
                    .map(|v| PredicateValue::Int(i64::from(*v)));
                let max = int_stats
                    .max_opt()
                    .map(|v| PredicateValue::Int(i64::from(*v)));
                (min, max)
            } else {
                (None, None)
            }
        }
        DataType::Int32 => {
            if let Statistics::Int32(int_stats) = stats {
                let min = int_stats
                    .min_opt()
                    .map(|v| PredicateValue::Int(i64::from(*v)));
                let max = int_stats
                    .max_opt()
                    .map(|v| PredicateValue::Int(i64::from(*v)));
                (min, max)
            } else {
                (None, None)
            }
        }
        DataType::Int64 => {
            if let Statistics::Int64(int_stats) = stats {
                let min = int_stats.min_opt().map(|v| PredicateValue::Int(*v));
                let max = int_stats.max_opt().map(|v| PredicateValue::Int(*v));
                (min, max)
            } else {
                (None, None)
            }
        }
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 => {
            if let Statistics::Int32(int_stats) = stats {
                let min = int_stats
                    .min_opt()
                    .map(|v| PredicateValue::Int(i64::from(*v)));
                let max = int_stats
                    .max_opt()
                    .map(|v| PredicateValue::Int(i64::from(*v)));
                (min, max)
            } else {
                (None, None)
            }
        }
        DataType::UInt64 => {
            if let Statistics::Int64(int_stats) = stats {
                // Handle potential overflow for very large u64 values
                let min = int_stats.min_opt().map(|v| PredicateValue::Int(*v));
                let max = int_stats.max_opt().map(|v| PredicateValue::Int(*v));
                (min, max)
            } else {
                (None, None)
            }
        }
        DataType::Float32 => {
            if let Statistics::Float(float_stats) = stats {
                let min = float_stats.min_opt().and_then(|v| {
                    if v.is_nan() {
                        None
                    } else {
                        Some(PredicateValue::Float(f64::from(*v)))
                    }
                });
                let max = float_stats.max_opt().and_then(|v| {
                    if v.is_nan() {
                        None
                    } else {
                        Some(PredicateValue::Float(f64::from(*v)))
                    }
                });
                (min, max)
            } else {
                (None, None)
            }
        }
        DataType::Float64 => {
            if let Statistics::Double(double_stats) = stats {
                let min = double_stats.min_opt().and_then(|v| {
                    if v.is_nan() {
                        None
                    } else {
                        Some(PredicateValue::Float(*v))
                    }
                });
                let max = double_stats.max_opt().and_then(|v| {
                    if v.is_nan() {
                        None
                    } else {
                        Some(PredicateValue::Float(*v))
                    }
                });
                (min, max)
            } else {
                (None, None)
            }
        }
        DataType::Utf8 | DataType::LargeUtf8 => {
            if let Statistics::ByteArray(byte_stats) = stats {
                let min = byte_stats.min_opt().and_then(|v| {
                    std::str::from_utf8(v.data())
                        .ok()
                        .map(|s| PredicateValue::String(s.to_string()))
                });
                let max = byte_stats.max_opt().and_then(|v| {
                    std::str::from_utf8(v.data())
                        .ok()
                        .map(|s| PredicateValue::String(s.to_string()))
                });
                (min, max)
            } else {
                (None, None)
            }
        }
        _ => (None, None),
    }
}

/// Select row groups that might contain matching rows.
///
/// Uses the predicate to determine which row groups can be skipped based on
/// column statistics. Returns indices of row groups that must be read.
#[must_use]
pub fn select_row_groups(
    predicate: &Predicate,
    all_statistics: &[RowGroupStatistics],
) -> Vec<usize> {
    let mut selected = Vec::new();

    for stats in all_statistics {
        // If we can't skip this row group, include it
        if !predicate.can_skip_row_group(stats) {
            selected.push(stats.row_group_id);
        }
    }

    // If no row groups selected (all skipped) but there are row groups,
    // this is the expected behavior - return empty vector for no matches
    selected
}

// ==================== In-Memory Predicate Evaluation ====================

/// Evaluate a predicate on a record batch, returning a boolean mask.
///
/// Each element in the returned array indicates whether the corresponding
/// row in the batch matches the predicate.
pub fn evaluate_predicate(
    batch: &RecordBatch,
    predicate: &Predicate,
) -> Result<BooleanArray, HedlError> {
    match predicate {
        Predicate::Equal(col, val) => evaluate_comparison(batch, col, val, ComparisonOp::Eq),
        Predicate::NotEqual(col, val) => evaluate_comparison(batch, col, val, ComparisonOp::Ne),
        Predicate::LessThan(col, val) => evaluate_comparison(batch, col, val, ComparisonOp::Lt),
        Predicate::LessThanOrEqual(col, val) => {
            evaluate_comparison(batch, col, val, ComparisonOp::Le)
        }
        Predicate::GreaterThan(col, val) => evaluate_comparison(batch, col, val, ComparisonOp::Gt),
        Predicate::GreaterThanOrEqual(col, val) => {
            evaluate_comparison(batch, col, val, ComparisonOp::Ge)
        }
        Predicate::Between(col, min_val, max_val) => {
            // BETWEEN is: col >= min AND col <= max
            let ge_min = evaluate_comparison(batch, col, min_val, ComparisonOp::Ge)?;
            let le_max = evaluate_comparison(batch, col, max_val, ComparisonOp::Le)?;
            compute::and(&ge_min, &le_max).map_err(|e| {
                HedlError::new(
                    HedlErrorKind::IO,
                    format!("Failed to combine predicates: {e}"),
                    0,
                )
            })
        }
        Predicate::In(col, values) => evaluate_in(batch, col, values, false),
        Predicate::NotIn(col, values) => evaluate_in(batch, col, values, true),
        Predicate::IsNull(col) => evaluate_is_null(batch, col, true),
        Predicate::IsNotNull(col) => evaluate_is_null(batch, col, false),
        Predicate::And(predicates) => {
            if predicates.is_empty() {
                // Empty AND matches everything
                return Ok(BooleanArray::from(vec![true; batch.num_rows()]));
            }
            let mut result = evaluate_predicate(batch, &predicates[0])?;
            for pred in &predicates[1..] {
                let mask = evaluate_predicate(batch, pred)?;
                result = compute::and(&result, &mask).map_err(|e| {
                    HedlError::new(
                        HedlErrorKind::IO,
                        format!("Failed to combine predicates: {e}"),
                        0,
                    )
                })?;
            }
            Ok(result)
        }
        Predicate::Or(predicates) => {
            if predicates.is_empty() {
                // Empty OR matches nothing
                return Ok(BooleanArray::from(vec![false; batch.num_rows()]));
            }
            let mut result = evaluate_predicate(batch, &predicates[0])?;
            for pred in &predicates[1..] {
                let mask = evaluate_predicate(batch, pred)?;
                result = compute::or(&result, &mask).map_err(|e| {
                    HedlError::new(
                        HedlErrorKind::IO,
                        format!("Failed to combine predicates: {e}"),
                        0,
                    )
                })?;
            }
            Ok(result)
        }
        Predicate::Not(pred) => {
            let mask = evaluate_predicate(batch, pred)?;
            compute::not(&mask).map_err(|e| {
                HedlError::new(
                    HedlErrorKind::IO,
                    format!("Failed to negate predicate: {e}"),
                    0,
                )
            })
        }
    }
}

/// Apply a predicate filter to a record batch, returning filtered batch.
pub fn filter_batch(batch: &RecordBatch, predicate: &Predicate) -> Result<RecordBatch, HedlError> {
    let mask = evaluate_predicate(batch, predicate)?;

    // Use Arrow's filter kernel to select matching rows
    let filtered_columns: Result<Vec<Arc<dyn Array>>, _> = batch
        .columns()
        .iter()
        .map(|col| compute::filter(col, &mask))
        .collect();

    let filtered_columns = filtered_columns.map_err(|e| {
        HedlError::new(HedlErrorKind::IO, format!("Failed to filter batch: {e}"), 0)
    })?;

    RecordBatch::try_new(batch.schema(), filtered_columns).map_err(|e| {
        HedlError::new(
            HedlErrorKind::IO,
            format!("Failed to create filtered batch: {e}"),
            0,
        )
    })
}

// ==================== Helper Functions ====================

#[derive(Clone, Copy)]
enum ComparisonOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

fn evaluate_comparison(
    batch: &RecordBatch,
    col_name: &str,
    value: &PredicateValue,
    op: ComparisonOp,
) -> Result<BooleanArray, HedlError> {
    let col_idx = batch.schema().index_of(col_name).map_err(|_| {
        HedlError::new(
            HedlErrorKind::Syntax,
            format!("Column not found: {col_name}"),
            0,
        )
    })?;

    let array = batch.column(col_idx);

    match (array.data_type(), value) {
        (DataType::Int64, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<Int64Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected Int64 array", 0))?;
            compare_int64(int_array, *v, op)
        }
        (DataType::Int32, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<Int32Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected Int32 array", 0))?;
            compare_int32(int_array, *v as i32, op)
        }
        (DataType::Int16, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<Int16Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected Int16 array", 0))?;
            compare_int16(int_array, *v as i16, op)
        }
        (DataType::Int8, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<Int8Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected Int8 array", 0))?;
            compare_int8(int_array, *v as i8, op)
        }
        (DataType::UInt64, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<UInt64Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected UInt64 array", 0))?;
            if *v < 0 {
                // All uint64 values are >= 0, so comparison with negative is straightforward
                match op {
                    ComparisonOp::Eq => Ok(BooleanArray::from(vec![false; batch.num_rows()])),
                    ComparisonOp::Ne => Ok(BooleanArray::from(vec![true; batch.num_rows()])),
                    ComparisonOp::Lt => Ok(BooleanArray::from(vec![false; batch.num_rows()])),
                    ComparisonOp::Le => Ok(BooleanArray::from(vec![false; batch.num_rows()])),
                    ComparisonOp::Gt => Ok(BooleanArray::from(vec![true; batch.num_rows()])),
                    ComparisonOp::Ge => Ok(BooleanArray::from(vec![true; batch.num_rows()])),
                }
            } else {
                compare_uint64(int_array, *v as u64, op)
            }
        }
        (DataType::UInt32, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<UInt32Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected UInt32 array", 0))?;
            compare_uint32(int_array, *v as u32, op)
        }
        (DataType::UInt16, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<UInt16Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected UInt16 array", 0))?;
            compare_uint16(int_array, *v as u16, op)
        }
        (DataType::UInt8, PredicateValue::Int(v)) => {
            let int_array = array
                .as_any()
                .downcast_ref::<UInt8Array>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected UInt8 array", 0))?;
            compare_uint8(int_array, *v as u8, op)
        }
        (DataType::Float64, PredicateValue::Float(v)) => {
            let float_array = array
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| {
                    HedlError::new(HedlErrorKind::Syntax, "Expected Float64 array", 0)
                })?;
            compare_float64(float_array, *v, op)
        }
        (DataType::Float32, PredicateValue::Float(v)) => {
            let float_array = array
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or_else(|| {
                    HedlError::new(HedlErrorKind::Syntax, "Expected Float32 array", 0)
                })?;
            compare_float32(float_array, *v as f32, op)
        }
        (DataType::Float64, PredicateValue::Int(v)) => {
            let float_array = array
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| {
                    HedlError::new(HedlErrorKind::Syntax, "Expected Float64 array", 0)
                })?;
            compare_float64(float_array, *v as f64, op)
        }
        (DataType::Float32, PredicateValue::Int(v)) => {
            let float_array = array
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or_else(|| {
                    HedlError::new(HedlErrorKind::Syntax, "Expected Float32 array", 0)
                })?;
            compare_float32(float_array, *v as f32, op)
        }
        (DataType::Utf8, PredicateValue::String(v)) => {
            let str_array = array
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| HedlError::new(HedlErrorKind::Syntax, "Expected String array", 0))?;
            compare_string(str_array, v, op)
        }
        (DataType::Boolean, PredicateValue::Bool(v)) => {
            let bool_array = array
                .as_any()
                .downcast_ref::<BooleanArray>()
                .ok_or_else(|| {
                    HedlError::new(HedlErrorKind::Syntax, "Expected Boolean array", 0)
                })?;
            compare_bool(bool_array, *v, op)
        }
        _ => Err(HedlError::new(
            HedlErrorKind::Syntax,
            format!(
                "Type mismatch: column '{}' has type {:?}, predicate value is {:?}",
                col_name,
                array.data_type(),
                value
            ),
            0,
        )),
    }
}

fn evaluate_in(
    batch: &RecordBatch,
    col_name: &str,
    values: &[PredicateValue],
    negate: bool,
) -> Result<BooleanArray, HedlError> {
    if values.is_empty() {
        let result = vec![negate; batch.num_rows()];
        return Ok(BooleanArray::from(result));
    }

    // Evaluate equality for each value and OR them together
    let mut result = evaluate_comparison(batch, col_name, &values[0], ComparisonOp::Eq)?;
    for val in &values[1..] {
        let mask = evaluate_comparison(batch, col_name, val, ComparisonOp::Eq)?;
        result = compute::or(&result, &mask).map_err(|e| {
            HedlError::new(
                HedlErrorKind::IO,
                format!("Failed to combine IN predicates: {e}"),
                0,
            )
        })?;
    }

    if negate {
        compute::not(&result).map_err(|e| {
            HedlError::new(
                HedlErrorKind::IO,
                format!("Failed to negate predicate: {e}"),
                0,
            )
        })
    } else {
        Ok(result)
    }
}

fn evaluate_is_null(
    batch: &RecordBatch,
    col_name: &str,
    is_null: bool,
) -> Result<BooleanArray, HedlError> {
    let col_idx = batch.schema().index_of(col_name).map_err(|_| {
        HedlError::new(
            HedlErrorKind::Syntax,
            format!("Column not found: {col_name}"),
            0,
        )
    })?;

    let array = batch.column(col_idx);

    if is_null {
        compute::is_null(array).map_err(|e| {
            HedlError::new(HedlErrorKind::IO, format!("Failed to check nulls: {e}"), 0)
        })
    } else {
        compute::is_not_null(array).map_err(|e| {
            HedlError::new(
                HedlErrorKind::IO,
                format!("Failed to check not nulls: {e}"),
                0,
            )
        })
    }
}

// Comparison helper macros using Arrow 57+ Datum-based API
macro_rules! impl_compare {
    ($fn_name:ident, $array_type:ty, $value_type:ty, $wrapper_array:ty) => {
        fn $fn_name(
            array: &$array_type,
            value: $value_type,
            op: ComparisonOp,
        ) -> Result<BooleanArray, HedlError> {
            // Create a scalar value as a single-element array wrapped in Scalar
            let scalar_array = <$wrapper_array>::from(vec![value]);
            let scalar = Scalar::new(&scalar_array);

            let result = match op {
                ComparisonOp::Eq => cmp::eq(array, &scalar),
                ComparisonOp::Ne => cmp::neq(array, &scalar),
                ComparisonOp::Lt => cmp::lt(array, &scalar),
                ComparisonOp::Le => cmp::lt_eq(array, &scalar),
                ComparisonOp::Gt => cmp::gt(array, &scalar),
                ComparisonOp::Ge => cmp::gt_eq(array, &scalar),
            };
            result.map_err(|e| {
                HedlError::new(HedlErrorKind::IO, format!("Comparison failed: {}", e), 0)
            })
        }
    };
}

impl_compare!(compare_int64, Int64Array, i64, Int64Array);
impl_compare!(compare_int32, Int32Array, i32, Int32Array);
impl_compare!(compare_int16, Int16Array, i16, Int16Array);
impl_compare!(compare_int8, Int8Array, i8, Int8Array);
impl_compare!(compare_uint64, UInt64Array, u64, UInt64Array);
impl_compare!(compare_uint32, UInt32Array, u32, UInt32Array);
impl_compare!(compare_uint16, UInt16Array, u16, UInt16Array);
impl_compare!(compare_uint8, UInt8Array, u8, UInt8Array);
impl_compare!(compare_float64, Float64Array, f64, Float64Array);
impl_compare!(compare_float32, Float32Array, f32, Float32Array);

fn compare_string(
    array: &StringArray,
    value: &str,
    op: ComparisonOp,
) -> Result<BooleanArray, HedlError> {
    let scalar_array = StringArray::from(vec![value]);
    let scalar = Scalar::new(&scalar_array);

    let result = match op {
        ComparisonOp::Eq => cmp::eq(array, &scalar),
        ComparisonOp::Ne => cmp::neq(array, &scalar),
        ComparisonOp::Lt => cmp::lt(array, &scalar),
        ComparisonOp::Le => cmp::lt_eq(array, &scalar),
        ComparisonOp::Gt => cmp::gt(array, &scalar),
        ComparisonOp::Ge => cmp::gt_eq(array, &scalar),
    };
    result.map_err(|e| {
        HedlError::new(
            HedlErrorKind::IO,
            format!("String comparison failed: {e}"),
            0,
        )
    })
}

fn compare_bool(
    array: &BooleanArray,
    value: bool,
    op: ComparisonOp,
) -> Result<BooleanArray, HedlError> {
    let scalar_array = BooleanArray::from(vec![value]);
    let scalar = Scalar::new(&scalar_array);

    let result = match op {
        ComparisonOp::Eq => cmp::eq(array, &scalar),
        ComparisonOp::Ne => cmp::neq(array, &scalar),
        // For boolean, Lt/Le/Gt/Ge use Arrow's default ordering (false < true)
        ComparisonOp::Lt => cmp::lt(array, &scalar),
        ComparisonOp::Le => cmp::lt_eq(array, &scalar),
        ComparisonOp::Gt => cmp::gt(array, &scalar),
        ComparisonOp::Ge => cmp::gt_eq(array, &scalar),
    };
    result.map_err(|e| {
        HedlError::new(
            HedlErrorKind::IO,
            format!("Boolean comparison failed: {e}"),
            0,
        )
    })
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use super::*;
    use arrow::datatypes::Field;

    #[test]
    fn test_predicate_value_comparison() {
        // Int comparisons
        assert!(PredicateValue::Int(5).lt(&PredicateValue::Int(10)).unwrap());
        assert!(!PredicateValue::Int(10).lt(&PredicateValue::Int(5)).unwrap());
        assert!(PredicateValue::Int(5).le(&PredicateValue::Int(5)).unwrap());

        // String comparisons
        assert!(PredicateValue::String("a".into())
            .lt(&PredicateValue::String("b".into()))
            .unwrap());

        // Float comparisons
        assert!(PredicateValue::Float(1.5)
            .lt(&PredicateValue::Float(2.5))
            .unwrap());

        // Cross-type int/float
        assert!(PredicateValue::Int(5)
            .lt(&PredicateValue::Float(5.5))
            .unwrap());
    }

    #[test]
    fn test_predicate_constructors() {
        let p = Predicate::equal("age", PredicateValue::Int(25));
        assert!(matches!(p, Predicate::Equal(col, PredicateValue::Int(25)) if col == "age"));

        let p = Predicate::between("age", PredicateValue::Int(18), PredicateValue::Int(65));
        assert!(matches!(p, Predicate::Between(col, _, _) if col == "age"));

        let p = Predicate::and(vec![
            Predicate::equal("a", PredicateValue::Int(1)),
            Predicate::equal("b", PredicateValue::Int(2)),
        ]);
        assert!(matches!(p, Predicate::And(preds) if preds.len() == 2));
    }

    #[test]
    fn test_row_group_pruning_equal() {
        let mut stats = RowGroupStatistics {
            row_group_id: 0,
            total_rows: 1000,
            columns: HashMap::new(),
        };

        stats.columns.insert(
            "age".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::Int(20)),
                max_value: Some(PredicateValue::Int(40)),
                null_count: 0,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // Value in range - cannot skip
        let pred = Predicate::equal("age", PredicateValue::Int(25));
        assert!(!pred.can_skip_row_group(&stats));

        // Value below range - can skip
        let pred = Predicate::equal("age", PredicateValue::Int(10));
        assert!(pred.can_skip_row_group(&stats));

        // Value above range - can skip
        let pred = Predicate::equal("age", PredicateValue::Int(50));
        assert!(pred.can_skip_row_group(&stats));
    }

    #[test]
    fn test_row_group_pruning_range() {
        let mut stats = RowGroupStatistics {
            row_group_id: 0,
            total_rows: 1000,
            columns: HashMap::new(),
        };

        stats.columns.insert(
            "age".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::Int(30)),
                max_value: Some(PredicateValue::Int(50)),
                null_count: 0,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // Overlapping range - cannot skip
        let pred = Predicate::between("age", PredicateValue::Int(25), PredicateValue::Int(35));
        assert!(!pred.can_skip_row_group(&stats));

        // Range entirely below - can skip
        let pred = Predicate::between("age", PredicateValue::Int(10), PredicateValue::Int(20));
        assert!(pred.can_skip_row_group(&stats));

        // Range entirely above - can skip
        let pred = Predicate::between("age", PredicateValue::Int(60), PredicateValue::Int(70));
        assert!(pred.can_skip_row_group(&stats));
    }

    #[test]
    fn test_row_group_pruning_null_checks() {
        let mut stats = RowGroupStatistics {
            row_group_id: 0,
            total_rows: 1000,
            columns: HashMap::new(),
        };

        // Column with some nulls
        stats.columns.insert(
            "email".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::String("a@b.com".into())),
                max_value: Some(PredicateValue::String("z@b.com".into())),
                null_count: 100,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // IS NULL - cannot skip (there are nulls)
        let pred = Predicate::is_null("email");
        assert!(!pred.can_skip_row_group(&stats));

        // IS NOT NULL - cannot skip (not all are null)
        let pred = Predicate::is_not_null("email");
        assert!(!pred.can_skip_row_group(&stats));

        // Column with no nulls
        stats.columns.insert(
            "id".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::Int(1)),
                max_value: Some(PredicateValue::Int(1000)),
                null_count: 0,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // IS NULL on no-null column - can skip
        let pred = Predicate::is_null("id");
        assert!(pred.can_skip_row_group(&stats));

        // Column with all nulls
        stats.columns.insert(
            "optional".into(),
            ColumnStatistics {
                min_value: None,
                max_value: None,
                null_count: 1000,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // IS NOT NULL on all-null column - can skip
        let pred = Predicate::is_not_null("optional");
        assert!(pred.can_skip_row_group(&stats));
    }

    #[test]
    fn test_row_group_pruning_and() {
        let mut stats = RowGroupStatistics {
            row_group_id: 0,
            total_rows: 1000,
            columns: HashMap::new(),
        };

        stats.columns.insert(
            "age".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::Int(30)),
                max_value: Some(PredicateValue::Int(50)),
                null_count: 0,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        stats.columns.insert(
            "status".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::String("active".into())),
                max_value: Some(PredicateValue::String("pending".into())),
                null_count: 0,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // Both predicates could match - cannot skip
        let pred = Predicate::and(vec![
            Predicate::equal("age", PredicateValue::Int(35)),
            Predicate::equal("status", PredicateValue::String("active".into())),
        ]);
        assert!(!pred.can_skip_row_group(&stats));

        // One predicate fails (age outside range) - can skip
        let pred = Predicate::and(vec![
            Predicate::equal("age", PredicateValue::Int(10)), // Outside range
            Predicate::equal("status", PredicateValue::String("active".into())),
        ]);
        assert!(pred.can_skip_row_group(&stats));
    }

    #[test]
    fn test_row_group_pruning_or() {
        let mut stats = RowGroupStatistics {
            row_group_id: 0,
            total_rows: 1000,
            columns: HashMap::new(),
        };

        stats.columns.insert(
            "age".into(),
            ColumnStatistics {
                min_value: Some(PredicateValue::Int(30)),
                max_value: Some(PredicateValue::Int(50)),
                null_count: 0,
                total_rows: 1000,
                has_statistics: true,
            },
        );

        // One predicate matches - cannot skip
        let pred = Predicate::or(vec![
            Predicate::equal("age", PredicateValue::Int(10)), // Outside range
            Predicate::equal("age", PredicateValue::Int(35)), // Inside range
        ]);
        assert!(!pred.can_skip_row_group(&stats));

        // Both predicates fail - can skip
        let pred = Predicate::or(vec![
            Predicate::equal("age", PredicateValue::Int(10)), // Outside range
            Predicate::equal("age", PredicateValue::Int(60)), // Also outside range
        ]);
        assert!(pred.can_skip_row_group(&stats));
    }

    #[test]
    fn test_evaluate_predicate_int() {
        let schema = Arc::new(Schema::new(vec![Field::new("age", DataType::Int64, false)]));

        let age_array = Int64Array::from(vec![20, 25, 30, 35, 40]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(age_array)]).unwrap();

        // Test equality
        let pred = Predicate::equal("age", PredicateValue::Int(25));
        let mask = evaluate_predicate(&batch, &pred).unwrap();
        assert_eq!(mask.value(0), false);
        assert_eq!(mask.value(1), true);
        assert_eq!(mask.value(2), false);

        // Test greater than
        let pred = Predicate::greater_than("age", PredicateValue::Int(30));
        let mask = evaluate_predicate(&batch, &pred).unwrap();
        assert_eq!(mask.value(0), false);
        assert_eq!(mask.value(1), false);
        assert_eq!(mask.value(2), false);
        assert_eq!(mask.value(3), true);
        assert_eq!(mask.value(4), true);

        // Test between
        let pred = Predicate::between("age", PredicateValue::Int(25), PredicateValue::Int(35));
        let mask = evaluate_predicate(&batch, &pred).unwrap();
        assert_eq!(mask.value(0), false);
        assert_eq!(mask.value(1), true);
        assert_eq!(mask.value(2), true);
        assert_eq!(mask.value(3), true);
        assert_eq!(mask.value(4), false);
    }

    #[test]
    fn test_evaluate_predicate_string() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "status",
            DataType::Utf8,
            false,
        )]));

        let status_array = StringArray::from(vec!["active", "inactive", "pending", "active"]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(status_array)]).unwrap();

        let pred = Predicate::equal("status", PredicateValue::String("active".into()));
        let mask = evaluate_predicate(&batch, &pred).unwrap();
        assert_eq!(mask.value(0), true);
        assert_eq!(mask.value(1), false);
        assert_eq!(mask.value(2), false);
        assert_eq!(mask.value(3), true);
    }

    #[test]
    fn test_evaluate_predicate_in() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "status",
            DataType::Utf8,
            false,
        )]));

        let status_array = StringArray::from(vec!["active", "inactive", "pending", "suspended"]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(status_array)]).unwrap();

        let pred = Predicate::in_set(
            "status",
            vec![
                PredicateValue::String("active".into()),
                PredicateValue::String("pending".into()),
            ],
        );
        let mask = evaluate_predicate(&batch, &pred).unwrap();
        assert_eq!(mask.value(0), true); // active
        assert_eq!(mask.value(1), false); // inactive
        assert_eq!(mask.value(2), true); // pending
        assert_eq!(mask.value(3), false); // suspended
    }

    #[test]
    fn test_filter_batch() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let id_array = Int64Array::from(vec![1, 2, 3, 4, 5]);
        let name_array = StringArray::from(vec!["alice", "bob", "charlie", "diana", "eve"]);
        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(id_array), Arc::new(name_array)]).unwrap();

        let pred = Predicate::greater_than("id", PredicateValue::Int(2));
        let filtered = filter_batch(&batch, &pred).unwrap();

        assert_eq!(filtered.num_rows(), 3);

        let id_col = filtered
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        assert_eq!(id_col.value(0), 3);
        assert_eq!(id_col.value(1), 4);
        assert_eq!(id_col.value(2), 5);
    }

    #[test]
    fn test_predicate_display() {
        let pred = Predicate::equal("age", PredicateValue::Int(25));
        assert_eq!(pred.to_string(), "age = 25");

        let pred = Predicate::between("age", PredicateValue::Int(18), PredicateValue::Int(65));
        assert_eq!(pred.to_string(), "age BETWEEN 18 AND 65");

        let pred = Predicate::and(vec![
            Predicate::equal("status", PredicateValue::String("active".into())),
            Predicate::greater_than("age", PredicateValue::Int(18)),
        ]);
        assert_eq!(pred.to_string(), "(status = 'active') AND (age > 18)");
    }
}

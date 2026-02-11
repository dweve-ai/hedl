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

//! Row group pruning using Parquet column statistics.

use std::collections::HashMap;
use std::sync::Arc;

use arrow::datatypes::{DataType, Schema};
use parquet::file::metadata::ParquetMetaData;
use parquet::file::statistics::Statistics;

use super::types::{Predicate, PredicateValue};

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

impl Predicate {
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

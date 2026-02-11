// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Schema inference for CSV columns

// Note: hedl_core::Value is used by ColumnType enum for type inference

#[derive(Debug, Clone, Copy)]
pub(crate) enum ColumnType {
    /// All sampled values are null/empty
    Null,
    /// All sampled values are "true" or "false"
    Bool,
    /// All sampled values parse as integers
    Int,
    /// All sampled values parse as floats (but not all as integers)
    Float,
    /// Default fallback for mixed or string data
    String,
}

/// Infer the type of a single column from sampled values.
///
/// # Type Inference Rules
///
/// The function examines non-null values and determines the most specific type:
///
/// 1. If all values are null → `ColumnType::Null`
/// 2. If all values are "true"/"false" → `ColumnType::Bool`
/// 3. If all values parse as i64 → `ColumnType::Int`
/// 4. If all values parse as f64 → `ColumnType::Float`
/// 5. Otherwise → `ColumnType::String`
///
/// # Arguments
///
/// * `values` - Iterator over string values from a column
///
/// # Examples
///
/// ```text
/// let values = vec!["1", "2", "3"];
/// let col_type = infer_column_type(values.iter().map(|s| s.as_str()));
/// assert_eq!(col_type, ColumnType::Int);
/// ```
fn infer_column_type<'a, I>(values: I) -> ColumnType
where
    I: Iterator<Item = &'a str>,
{
    let mut all_null = true;
    let mut all_bool = true;
    let mut all_int = true;
    let mut all_float = true;

    for value in values {
        let trimmed = value.trim();

        // Skip null values (don't affect type inference)
        if trimmed.is_empty() || trimmed == "~" || trimmed == "null" {
            continue;
        }

        all_null = false;

        // Check bool
        if trimmed != "true" && trimmed != "false" {
            all_bool = false;
        }

        // Check int
        if trimmed.parse::<i64>().is_err() {
            all_int = false;
        }

        // Check float
        if trimmed.parse::<f64>().is_err() {
            all_float = false;
        }

        // Early exit if we know it's a string
        if !all_bool && !all_int && !all_float {
            return ColumnType::String;
        }
    }

    // Determine type based on inference (most specific to least)
    if all_null {
        ColumnType::Null
    } else if all_bool {
        ColumnType::Bool
    } else if all_int {
        ColumnType::Int
    } else if all_float {
        ColumnType::Float
    } else {
        ColumnType::String
    }
}

/// Infer types for all columns by sampling CSV records.
///
/// # Arguments
///
/// * `records` - Slice of CSV records (each record is a Vec<String>)
/// * `sample_size` - Maximum number of records to sample
///
/// # Returns
///
/// A vector of `ColumnType` for each column in the CSV.
///
/// # Examples
///
/// ```text
/// let records = vec![
///     vec!["1".to_string(), "Alice".to_string(), "30".to_string()],
///     vec!["2".to_string(), "Bob".to_string(), "25".to_string()],
/// ];
/// let types = infer_column_types(&records, 100);
/// assert_eq!(types, vec![ColumnType::Int, ColumnType::String, ColumnType::Int]);
/// ```
pub(crate) fn infer_column_types(records: &[Vec<String>], sample_size: usize) -> Vec<ColumnType> {
    if records.is_empty() {
        return Vec::new();
    }

    let num_columns = records[0].len();
    let sample_count = sample_size.min(records.len());

    (0..num_columns)
        .map(|col_idx| {
            let column_values = records
                .iter()
                .take(sample_count)
                .filter_map(|row| row.get(col_idx).map(std::string::String::as_str));

            infer_column_type(column_values)
        })
        .collect()
}

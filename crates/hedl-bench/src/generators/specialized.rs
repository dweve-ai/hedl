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

//! Domain-specific data generators.
//!
//! Specialized generators for specific use cases like tensor data,
//! row operations, CSV-like structures, and ditto-heavy patterns.

use crate::datasets::generate_ditto_heavy;

/// Generates tensor/matrix data with specified dimensions.
///
/// Creates HEDL representation of multi-dimensional arrays suitable
/// for machine learning and scientific computing benchmarks.
///
/// # Arguments
///
/// * `dimensions` - Slice of dimension sizes (e.g., [100, 50, 3] for 100x50x3 tensor)
///
/// # Returns
///
/// HEDL document string with tensor data.
pub fn generate_tensor_data(dimensions: &[usize]) -> String {
    if dimensions.is_empty() {
        return "%VERSION: 1.0\n---\n".to_string();
    }

    let total_elements: usize = dimensions.iter().product();
    if total_elements == 0 {
        return "%VERSION: 1.0\n---\n".to_string();
    }

    let mut doc = String::from("%VERSION: 1.0\n");
    doc.push_str("%STRUCT: TensorElement: [id,value]\n");
    doc.push_str("---\n");
    doc.push_str("tensor: @TensorElement\n");

    // Generate elements with coordinate-based IDs
    let limit = total_elements.min(1000); // Limit for reasonable size
    for flat_idx in 0..limit {
        // Convert flat index to multi-dimensional coordinates
        let mut coords = Vec::new();
        let mut remaining = flat_idx;

        for dim_size in dimensions.iter().rev() {
            coords.push(remaining % dim_size);
            remaining /= dim_size;
        }
        coords.reverse();

        // Create ID from coordinates (e.g., "0_5_3" for [0,5,3])
        let id = coords
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join("_");

        doc.push_str(&format!("  |elem_{},{:.2}\n", id, flat_idx as f64 * 0.1));
    }

    doc
}

/// Generates row-oriented data for row operation benchmarks.
///
/// Creates CSV-like tabular data optimized for row parsing tests.
///
/// # Arguments
///
/// * `row_count` - Number of rows
/// * `col_count` - Number of columns
///
/// # Returns
///
/// HEDL document string with row data.
pub fn generate_row_data(row_count: usize, col_count: usize) -> String {
    if row_count == 0 || col_count == 0 {
        return "%VERSION: 1.0\n".to_string();
    }

    let mut doc = String::from("%VERSION: 1.0\n");

    // Generate column headers
    let headers: Vec<String> = (0..col_count).map(|i| format!("col{}", i)).collect();

    doc.push_str(&format!("%STRUCT: Row: [{}]\n", headers.join(",")));
    doc.push_str("---\n");
    doc.push_str("data: @Row\n");

    // Generate rows
    for row in 0..row_count {
        doc.push_str("  |");
        for col in 0..col_count {
            if col > 0 {
                doc.push(',');
            }
            doc.push_str(&format!("r{}c{}", row, col));
        }
        doc.push('\n');
    }

    doc
}

/// Generates CSV-like data for compatibility testing.
///
/// Alias for generate_row_data with clearer naming for CSV contexts.
///
/// # Arguments
///
/// * `rows` - Number of rows
/// * `cols` - Number of columns
///
/// # Returns
///
/// HEDL document string with CSV-like structure.
pub fn generate_csv_like(rows: usize, cols: usize) -> String {
    generate_row_data(rows, cols)
}

/// Generates ditto-heavy data for token efficiency testing.
///
/// Creates documents with extensive use of ditto markers (^) for
/// repeated values, testing HEDL's token efficiency.
///
/// # Arguments
///
/// * `entity_count` - Number of entities with repeated values
///
/// # Returns
///
/// HEDL document string with ditto-heavy pattern.
pub fn generate_ditto_data(entity_count: usize) -> String {
    generate_ditto_heavy(entity_count)
}

/// Generates wide rows with many columns.
///
/// Creates rows with a large number of fields, useful for testing
/// row parsing performance with varying field counts.
///
/// # Arguments
///
/// * `row_count` - Number of rows
/// * `field_count` - Number of fields per row (columns)
///
/// # Returns
///
/// HEDL document string with wide rows.
pub fn generate_wide_rows(row_count: usize, field_count: usize) -> String {
    generate_row_data(row_count, field_count)
}

/// Generates time-series data for analytics benchmarks.
///
/// # Arguments
///
/// * `data_points` - Number of time-series data points
///
/// # Returns
///
/// HEDL document string with time-series data.
pub fn generate_time_series(data_points: usize) -> String {
    if data_points == 0 {
        return "%VERSION: 1.0\n".to_string();
    }

    let mut doc = String::from("%VERSION: 1.0\n");
    doc.push_str("%STRUCT: Metric: [timestamp,value,tags]\n");
    doc.push_str("---\n");
    doc.push_str("metrics: @Metric\n");

    for i in 0..data_points {
        let timestamp = 1704067200 + (i * 60); // Starting from 2024-01-01
        let value = (i as f64 * 0.5).sin() * 100.0 + 100.0;
        doc.push_str(&format!(
            "  |{},{:.2},[server:web1,env:prod]\n",
            timestamp, value
        ));
    }

    doc
}

/// Generates key-value pairs for configuration data.
///
/// # Arguments
///
/// * `pair_count` - Number of key-value pairs
///
/// # Returns
///
/// HEDL document string with key-value structure.
pub fn generate_key_value(pair_count: usize) -> String {
    if pair_count == 0 {
        return "%VERSION: 1.0\n".to_string();
    }

    let mut doc = String::from("%VERSION: 1.0\n");
    doc.push_str("%STRUCT: Config: [key,value]\n");
    doc.push_str("---\n");
    doc.push_str("config: @Config\n");

    for i in 0..pair_count {
        doc.push_str(&format!("  |config_key_{},config_value_{}\n", i, i));
    }

    doc
}

/// Generates sparse matrix data.
///
/// # Arguments
///
/// * `rows` - Number of rows
/// * `cols` - Number of columns
/// * `density` - Sparsity (0.0 = all zeros, 1.0 = all non-zero)
///
/// # Returns
///
/// HEDL document string with sparse matrix.
pub fn generate_sparse_matrix(rows: usize, cols: usize, density: f32) -> String {
    if rows == 0 || cols == 0 {
        return "%VERSION: 1.0\n".to_string();
    }

    let mut doc = String::from("%VERSION: 1.0\n");
    doc.push_str("%STRUCT: SparseEntry: [row,col,value]\n");
    doc.push_str("---\n");
    doc.push_str("sparse: @SparseEntry\n");

    let total_entries = (rows * cols) as f32 * density.min(1.0);
    for i in 0..total_entries as usize {
        let row = i % rows;
        let col = (i / rows) % cols;
        let value = (i as f64 + 1.0) * 0.1;
        doc.push_str(&format!("  |{},{},{:.2}\n", row, col, value));
    }

    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_data() {
        let doc = generate_tensor_data(&[10, 5, 3]);
        assert!(doc.contains("TensorElement"));
        assert!(doc.contains("tensor:"));
        assert!(doc.contains("value"));
    }

    #[test]
    fn test_row_data() {
        let doc = generate_row_data(5, 3);
        assert!(doc.contains("%STRUCT: Row"));
        assert!(doc.contains("col0"));
        assert!(doc.contains("r0c0"));
    }

    #[test]
    fn test_csv_like() {
        let doc = generate_csv_like(10, 5);
        assert!(doc.contains("%VERSION: 1.0"));
    }

    #[test]
    fn test_wide_rows() {
        let doc = generate_wide_rows(5, 20);
        assert!(doc.contains("col19"));
    }

    #[test]
    fn test_time_series() {
        let doc = generate_time_series(10);
        assert!(doc.contains("Metric"));
        assert!(doc.contains("timestamp"));
    }

    #[test]
    fn test_key_value() {
        let doc = generate_key_value(5);
        assert!(doc.contains("Config"));
        assert!(doc.contains("config_key_"));
    }

    #[test]
    fn test_sparse_matrix() {
        let doc = generate_sparse_matrix(10, 10, 0.1);
        assert!(doc.contains("SparseEntry"));
    }
}

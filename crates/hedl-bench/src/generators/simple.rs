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

//! Simple data generators for flat and basic structures.
//!
//! These generators produce straightforward HEDL documents without deep nesting,
//! references, or complex relationships. Ideal for baseline benchmarks.

use crate::datasets::{generate_analytics, generate_events, generate_products, generate_users};

/// Generates a flat structure with specified field count.
///
/// Creates a simple HEDL document with a single struct type containing
/// the specified number of fields.
///
/// # Arguments
///
/// * `field_count` - Number of fields in the struct
///
/// # Returns
///
/// HEDL document string with flat structure.
pub fn generate_flat_struct(field_count: usize) -> String {
    let mut fields = Vec::new();
    for i in 0..field_count {
        fields.push(format!("field{}", i));
    }

    format!(
        "%VERSION: 1.0\n%STRUCT: Record (1): [{}]\ndata: @Record\n| {}",
        fields.join(","),
        (0..field_count)
            .map(|i| format!("value{}", i))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Generates simple nested structure with minimal depth.
///
/// Creates a document with 1-2 levels of nesting, useful for testing
/// basic nesting scenarios without deep recursion.
///
/// # Arguments
///
/// * `depth` - Nesting depth (1-2 recommended for simple structures)
///
/// # Returns
///
/// HEDL document string with simple nesting.
pub fn generate_nested_simple(depth: usize) -> String {
    if depth == 0 {
        return generate_flat_struct(3);
    }

    let mut doc = String::from("%VERSION: 1.0\n");
    doc.push_str("%STRUCT: Level0: [id,name]\n");
    doc.push_str("root: @Level0\n");
    doc.push_str("| 0, root\n");

    for level in 1..=depth {
        doc.push_str(&format!(
            "%NEST: Level{} in Level{}: [id,name]\n",
            level,
            level - 1
        ));
        doc.push_str(&format!("  | {}, child_{}\n", level, level));
    }

    doc
}

/// Generates a simple list structure with specified item count.
///
/// Creates a flat list of items without complex relationships.
///
/// # Arguments
///
/// * `item_count` - Number of items in the list
///
/// # Returns
///
/// HEDL document string with list structure.
pub fn generate_list_simple(item_count: usize) -> String {
    generate_users(item_count)
}

/// Generates simple tabular data from users dataset.
///
/// # Arguments
///
/// * `row_count` - Number of rows to generate
///
/// # Returns
///
/// HEDL document string with tabular data.
pub fn generate_users_simple(row_count: usize) -> String {
    generate_users(row_count)
}

/// Generates simple product catalog data.
///
/// # Arguments
///
/// * `row_count` - Number of products to generate
///
/// # Returns
///
/// HEDL document string with product data.
pub fn generate_products_simple(row_count: usize) -> String {
    generate_products(row_count)
}

/// Generates simple analytics/metrics data.
///
/// # Arguments
///
/// * `row_count` - Number of metric entries to generate
///
/// # Returns
///
/// HEDL document string with analytics data.
pub fn generate_analytics_simple(row_count: usize) -> String {
    generate_analytics(row_count)
}

/// Generates simple event log data.
///
/// # Arguments
///
/// * `row_count` - Number of events to generate
///
/// # Returns
///
/// HEDL document string with event data.
pub fn generate_events_simple(row_count: usize) -> String {
    generate_events(row_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_struct() {
        let doc = generate_flat_struct(5);
        assert!(doc.contains("%STRUCT: Record"));
        assert!(doc.contains("field0"));
        assert!(doc.contains("field4"));
    }

    #[test]
    fn test_nested_simple() {
        let doc = generate_nested_simple(2);
        assert!(doc.contains("%VERSION: 1.0"));
        assert!(doc.contains("%NEST:"));
    }

    #[test]
    fn test_list_simple() {
        let doc = generate_list_simple(10);
        assert!(doc.contains("%STRUCT: User"));
    }
}

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

//! Security limits for HEDL parsing.

/// Configurable limits for parser security.
///
/// These limits protect against denial-of-service attacks and memory exhaustion
/// by bounding the resources consumed during parsing.
#[derive(Debug, Clone)]
pub struct Limits {
    /// Maximum file size in bytes (default: 1GB).
    pub max_file_size: usize,
    /// Maximum line length in bytes (default: 1MB).
    pub max_line_length: usize,
    /// Maximum indent depth (default: 50).
    pub max_indent_depth: usize,
    /// Maximum number of nodes (default: 10M).
    pub max_nodes: usize,
    /// Maximum number of aliases (default: 10k).
    pub max_aliases: usize,
    /// Maximum columns per schema (default: 100).
    pub max_columns: usize,
    /// Maximum NEST hierarchy depth (default: 100).
    pub max_nest_depth: usize,
    /// Maximum block string size in bytes (default: 10MB).
    pub max_block_string_size: usize,
    /// Maximum number of keys in a single object/map (default: 10k).
    pub max_object_keys: usize,
    /// Maximum total number of keys across all objects (default: 10M).
    ///
    /// This prevents DoS attacks where an attacker creates many small objects,
    /// each under the max_object_keys limit, but collectively consuming excessive
    /// memory. Provides defense-in-depth against memory exhaustion attacks.
    ///
    /// Default is 10,000,000 keys, which allows for large documents while still
    /// providing protection against memory exhaustion. For very large datasets,
    /// this can be increased via `ParseOptions`.
    pub max_total_keys: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_file_size: 1024 * 1024 * 1024,      // 1GB
            max_line_length: 1024 * 1024,           // 1MB
            max_indent_depth: 50,
            max_nodes: 10_000_000,
            max_aliases: 10_000,
            max_columns: 100,
            max_nest_depth: 100,
            max_block_string_size: 10 * 1024 * 1024, // 10MB
            max_object_keys: 10_000,
            max_total_keys: 10_000_000,             // 10M
        }
    }
}

impl Limits {
    /// Create limits with no restrictions (for testing).
    pub fn unlimited() -> Self {
        Self {
            max_file_size: usize::MAX,
            max_line_length: usize::MAX,
            max_indent_depth: usize::MAX,
            max_nodes: usize::MAX,
            max_aliases: usize::MAX,
            max_columns: usize::MAX,
            max_nest_depth: usize::MAX,
            max_block_string_size: usize::MAX,
            max_object_keys: usize::MAX,
            max_total_keys: usize::MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Default limits tests ====================

    #[test]
    fn test_default_max_file_size() {
        let limits = Limits::default();
        assert_eq!(limits.max_file_size, 1024 * 1024 * 1024); // 1GB
    }

    #[test]
    fn test_default_max_line_length() {
        let limits = Limits::default();
        assert_eq!(limits.max_line_length, 1024 * 1024); // 1MB
    }

    #[test]
    fn test_default_max_indent_depth() {
        let limits = Limits::default();
        assert_eq!(limits.max_indent_depth, 50);
    }

    #[test]
    fn test_default_max_nodes() {
        let limits = Limits::default();
        assert_eq!(limits.max_nodes, 10_000_000); // 10M
    }

    #[test]
    fn test_default_max_aliases() {
        let limits = Limits::default();
        assert_eq!(limits.max_aliases, 10_000); // 10k
    }

    #[test]
    fn test_default_max_columns() {
        let limits = Limits::default();
        assert_eq!(limits.max_columns, 100);
    }

    // ==================== Unlimited limits tests ====================

    #[test]
    fn test_unlimited_max_file_size() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_file_size, usize::MAX);
    }

    #[test]
    fn test_unlimited_max_line_length() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_line_length, usize::MAX);
    }

    #[test]
    fn test_unlimited_max_indent_depth() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_indent_depth, usize::MAX);
    }

    #[test]
    fn test_unlimited_max_nodes() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_nodes, usize::MAX);
    }

    #[test]
    fn test_unlimited_max_aliases() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_aliases, usize::MAX);
    }

    #[test]
    fn test_unlimited_max_columns() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_columns, usize::MAX);
    }

    // ==================== Clone and Debug tests ====================

    #[test]
    fn test_limits_clone() {
        let original = Limits::default();
        let cloned = original.clone();
        assert_eq!(original.max_file_size, cloned.max_file_size);
        assert_eq!(original.max_line_length, cloned.max_line_length);
        assert_eq!(original.max_indent_depth, cloned.max_indent_depth);
        assert_eq!(original.max_nodes, cloned.max_nodes);
        assert_eq!(original.max_aliases, cloned.max_aliases);
        assert_eq!(original.max_columns, cloned.max_columns);
    }

    #[test]
    fn test_limits_debug() {
        let limits = Limits::default();
        let debug = format!("{:?}", limits);
        assert!(debug.contains("max_file_size"));
        assert!(debug.contains("max_line_length"));
        assert!(debug.contains("max_indent_depth"));
        assert!(debug.contains("max_nodes"));
        assert!(debug.contains("max_aliases"));
        assert!(debug.contains("max_columns"));
    }

    // ==================== Custom limits tests ====================

    #[test]
    fn test_custom_limits() {
        let limits = Limits {
            max_file_size: 100,
            max_line_length: 200,
            max_indent_depth: 5,
            max_nodes: 1000,
            max_aliases: 50,
            max_columns: 10,
            max_nest_depth: 20,
            max_block_string_size: 5000,
            max_object_keys: 100,
            max_total_keys: 500,
        };
        assert_eq!(limits.max_file_size, 100);
        assert_eq!(limits.max_line_length, 200);
        assert_eq!(limits.max_indent_depth, 5);
        assert_eq!(limits.max_nodes, 1000);
        assert_eq!(limits.max_aliases, 50);
        assert_eq!(limits.max_columns, 10);
        assert_eq!(limits.max_nest_depth, 20);
        assert_eq!(limits.max_block_string_size, 5000);
        assert_eq!(limits.max_object_keys, 100);
        assert_eq!(limits.max_total_keys, 500);
    }

    #[test]
    fn test_limits_zero_values() {
        let limits = Limits {
            max_file_size: 0,
            max_line_length: 0,
            max_indent_depth: 0,
            max_nodes: 0,
            max_aliases: 0,
            max_columns: 0,
            max_nest_depth: 0,
            max_block_string_size: 0,
            max_object_keys: 0,
            max_total_keys: 0,
        };
        assert_eq!(limits.max_file_size, 0);
        assert_eq!(limits.max_columns, 0);
        assert_eq!(limits.max_nest_depth, 0);
        assert_eq!(limits.max_block_string_size, 0);
        assert_eq!(limits.max_object_keys, 0);
        assert_eq!(limits.max_total_keys, 0);
    }

    // ==================== New limits tests ====================

    #[test]
    fn test_default_max_nest_depth() {
        let limits = Limits::default();
        assert_eq!(limits.max_nest_depth, 100);
    }

    #[test]
    fn test_default_max_block_string_size() {
        let limits = Limits::default();
        assert_eq!(limits.max_block_string_size, 10 * 1024 * 1024); // 10MB
    }

    #[test]
    fn test_unlimited_max_nest_depth() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_nest_depth, usize::MAX);
    }

    #[test]
    fn test_unlimited_max_block_string_size() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_block_string_size, usize::MAX);
    }

    #[test]
    fn test_default_max_total_keys() {
        let limits = Limits::default();
        assert_eq!(limits.max_total_keys, 10_000_000);
    }

    #[test]
    fn test_unlimited_max_total_keys() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_total_keys, usize::MAX);
    }

    #[test]
    fn test_max_total_keys_greater_than_max_object_keys() {
        let limits = Limits::default();
        assert!(limits.max_total_keys > limits.max_object_keys,
            "max_total_keys ({}) should be greater than max_object_keys ({})",
            limits.max_total_keys, limits.max_object_keys);
    }
}

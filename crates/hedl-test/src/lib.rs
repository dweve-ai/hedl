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

//! Shared test fixtures and utilities for HEDL format converters.
//!
//! This crate provides canonical test documents, builders, error fixtures,
//! and utilities to eliminate test code duplication across all HEDL format
//! converters (JSON, XML, YAML, CSV, Neo4j, Parquet, etc.).
//!
//! # Features
//!
//! - **Pre-built Fixtures**: Comprehensive test documents covering all HEDL features
//! - **Builder Pattern**: Fluent API for creating customized test data
//! - **Error Fixtures**: Invalid documents and error cases for testing error handling
//! - **Utilities**: Expression helpers, counting functions, and more
//!
//! # Quick Start
//!
//! ```rust
//! use hedl_test::fixtures;
//!
//! // Use pre-built fixtures
//! let doc = fixtures::scalars();           // All scalar types
//! let doc = fixtures::user_list();         // MatrixList with users
//! let doc = fixtures::with_references();   // Cross-references
//! let doc = fixtures::comprehensive();     // Everything together
//!
//! // Build custom fixtures
//! use hedl_test::fixtures::builders::{DocumentBuilder, ValueBuilder};
//!
//! let doc = DocumentBuilder::new()
//!     .scalar("name", ValueBuilder::string("Alice"))
//!     .scalar("age", ValueBuilder::int(30))
//!     .build();
//!
//! // Test error handling
//! use hedl_test::fixtures::errors;
//!
//! for (name, invalid) in errors::invalid_hedl_samples() {
//!     // Test parser with invalid input
//! }
//!
//! // Use utilities
//! use hedl_test::{expr, count_nodes};
//!
//! let e = expr("now()");               // Create expression
//! let n = count_nodes(&doc);           // Count nodes in document
//! ```
//!
//! # See Also
//!
//! - [Fixtures Guide](FIXTURES_GUIDE.md) - Comprehensive usage guide
//! - Module documentation for detailed API information

use hedl_core::Document;

/// Type alias for a list of fixture functions (name, generator).
pub type FixtureList = Vec<(&'static str, fn() -> Document)>;

/// Returns all fixtures as (name, hedl_text) pairs.
pub fn fixtures_as_hedl() -> Vec<(&'static str, String)> {
    fixtures::all()
        .into_iter()
        .map(|(name, fixture_fn)| {
            let doc = fixture_fn();
            let hedl_text = hedl_c14n::canonicalize(&doc)
                .unwrap_or_else(|e| format!("# Error serializing: {}", e));
            (name, hedl_text)
        })
        .collect()
}

/// Write all fixtures to a directory as .hedl files.
#[cfg(feature = "generate")]
pub fn write_fixtures_to_dir(dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    for (name, hedl_text) in fixtures_as_hedl() {
        let path = dir.join(format!("{}.hedl", name));
        std::fs::write(path, hedl_text)?;
    }
    Ok(())
}

/// Canonical test fixtures covering all HEDL features.
pub mod fixtures;

/// Expression utility functions for tests.
pub mod expr_utils;

/// Fixture counting utilities.
pub mod counts;

// Re-export all fixtures for backward compatibility
pub use fixtures::*;

// Re-export commonly used utilities
pub use expr_utils::{expr, expr_value, try_expr, try_expr_value, ExprError};
pub use counts::{count_nodes, count_references};


#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::Item;

    #[test]
    fn test_all_fixtures_valid() {
        for (name, fixture_fn) in fixtures::all() {
            let doc = fixture_fn();
            assert_eq!(
                doc.version,
                (1, 0),
                "Fixture {} should have version 1.0",
                name
            );
        }
    }

    #[test]
    fn test_scalars_fixture() {
        let doc = fixtures::scalars();
        assert!(doc.root.contains_key("null_val"));
        assert!(doc.root.contains_key("bool_true"));
        assert!(doc.root.contains_key("int_positive"));
        assert!(doc.root.contains_key("float_positive"));
        assert!(doc.root.contains_key("string_simple"));
    }

    #[test]
    fn test_user_list_fixture() {
        let doc = fixtures::user_list();
        if let Some(Item::List(list)) = doc.root.get("users") {
            assert_eq!(list.type_name, "User");
            assert_eq!(list.rows.len(), 3);
        } else {
            panic!("Expected users list");
        }
    }

    #[test]
    fn test_with_nest_fixture() {
        let doc = fixtures::with_nest();
        assert!(doc.nests.contains_key("User"));
        assert_eq!(doc.nests.get("User"), Some(&"Post".to_string()));

        // Check that alice has children
        if let Some(Item::List(list)) = doc.root.get("users") {
            let alice = list.rows.iter().find(|n| n.id == "alice").unwrap();
            assert!(!alice.children.is_empty());
        }
    }

    #[test]
    fn test_comprehensive_fixture() {
        let doc = fixtures::comprehensive();
        assert!(!doc.root.is_empty());
        assert!(!doc.structs.is_empty());
        assert!(!doc.nests.is_empty());

        // Should have various item types
        let has_scalar = doc.root.values().any(|i| matches!(i, Item::Scalar(_)));
        let has_list = doc.root.values().any(|i| matches!(i, Item::List(_)));
        assert!(has_scalar);
        assert!(has_list);
    }

    #[test]
    fn test_count_nodes() {
        let doc = fixtures::user_list();
        assert_eq!(counts::count_nodes(&doc), 3);

        let doc = fixtures::with_nest();
        // 2 users + 3 posts
        assert_eq!(counts::count_nodes(&doc), 5);
    }

    #[test]
    fn test_count_references() {
        let doc = fixtures::with_references();
        assert_eq!(counts::count_references(&doc), 3); // 3 posts with author refs
    }
}

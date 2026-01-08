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

//! Lexical analysis subsystem for HEDL.
//!
//! This module provides the complete lexical analysis infrastructure for HEDL,
//! consolidating functionality from `hedl-lex`, `hedl-row`, and `hedl-tensor`
//! into a unified, DRY, and modular architecture.
//!
//! # Module Structure
//!
//! - [`error`] - Unified error types for all lexer operations
//! - [`span`] - Source position and span tracking for error reporting
//! - [`tokens`] - Token validation and parsing (keys, types, IDs, references)
//! - [`row`] - CSV/matrix row parsing with expression and tensor support
//! - [`tensor`] - Multi-dimensional tensor literal parsing
//! - [`incremental`] - Incremental parsing for IDE integration
//!
//! # Examples
//!
//! ## Token Validation
//!
//! ```
//! use hedl_core::lex::{is_valid_key_token, is_valid_type_name, is_valid_id_token};
//!
//! assert!(is_valid_key_token("user_name"));  // snake_case
//! assert!(is_valid_type_name("UserProfile")); // PascalCase
//! assert!(is_valid_id_token("SKU-4020"));     // IDs with hyphens
//! ```
//!
//! ## Reference Parsing
//!
//! ```
//! use hedl_core::lex::{parse_reference, Reference};
//!
//! let local = parse_reference("@user_1").unwrap();
//! assert!(local.is_local());
//!
//! let qualified = parse_reference("@User:user_1").unwrap();
//! assert!(qualified.is_qualified());
//! assert_eq!(qualified.type_name, Some("User".to_string()));
//! ```
//!
//! ## CSV Row Parsing
//!
//! ```
//! use hedl_core::lex::parse_csv_row;
//!
//! let fields = parse_csv_row("id, name, [1, 2, 3]").unwrap();
//! assert_eq!(fields.len(), 3);
//! assert_eq!(fields[2].value, "[1, 2, 3]");
//! ```
//!
//! ## Tensor Parsing
//!
//! ```
//! use hedl_core::lex::{parse_tensor, is_tensor_literal, Tensor};
//!
//! assert!(is_tensor_literal("[1, 2, 3]"));
//!
//! let tensor = parse_tensor("[[1, 2], [3, 4]]").unwrap();
//! assert_eq!(tensor.shape(), vec![2, 2]);
//! assert_eq!(tensor.flatten(), vec![1.0, 2.0, 3.0, 4.0]);
//! ```
//!
//! # Security
//!
//! All parsing operations enforce resource limits to prevent DoS attacks:
//! - Maximum string lengths
//! - Maximum recursion depths
//! - Maximum element counts
//! - Rejection of NaN/Infinity values
//!
//! # Migration from Separate Crates
//!
//! This module consolidates the following crates:
//! - `hedl-lex` -> `hedl_core::lex` (tokens, spans, config, regions, etc.)
//! - `hedl-row` -> `hedl_core::lex::row` (CSV parsing)
//! - `hedl-tensor` -> `hedl_core::lex::tensor` (tensor parsing)
//!
//! For a detailed migration guide, see `LEXER_CONSOLIDATION.md`.

// Core modules
pub mod error;
pub mod span;
pub mod tokens;

// Extended modules from hedl-lex
pub mod arena;
pub mod config;
pub mod csv;
pub mod directives;
pub mod expression;
pub mod incremental;
pub mod indent;
pub mod lex_inference;
pub mod regions;
pub mod strings;

// Data modules from hedl-row and hedl-tensor
pub mod row;
pub mod tensor;

// Re-export error types
pub use error::{LexError, LexResult};

// Re-export span types
pub use span::{SourcePos, Span};

// Re-export token types and functions
pub use tokens::{
    is_valid_id_token, is_valid_key_token, is_valid_type_name, parse_reference, parse_reference_at,
    Reference,
};

// Re-export expression types and functions
pub use expression::{parse_expression, parse_expression_token, ExprLiteral, Expression};

// Re-export configuration
pub use config::LexConfig;

// Re-export string utilities
pub use strings::singularize_and_capitalize;

// Re-export indent handling
pub use indent::{calculate_indent, validate_indent, IndentInfo};

// Re-export directive parsing
pub use directives::{
    parse_alias, parse_nest, parse_struct, AliasDirective, NestDirective, StructDirective,
};

// Re-export region scanning
pub use regions::{scan_regions, strip_comment, Region, RegionType};

// Re-export value inference
pub use lex_inference::{infer_cell_value, infer_value, TensorValue, Value};

// Re-export CSV parsing (from row module which handles tensors correctly)
pub use row::{parse_csv_row, CsvField};

// Re-export incremental parsing
pub use incremental::{IncrementalParser, ParseResult, TextEdit};

// Re-export tensor types and functions
pub use tensor::{is_tensor_literal, parse_tensor, Tensor};

// Re-export arena allocation
pub use arena::ExpressionArena;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_integration() {
        // Test that all re-exports work correctly
        assert!(is_valid_key_token("test_key"));
        assert!(is_valid_type_name("TestType"));
        assert!(is_valid_id_token("test-id"));

        let reference = parse_reference("@User:user_1").unwrap();
        assert!(reference.is_qualified());

        let fields = parse_csv_row("a, b, c").unwrap();
        assert_eq!(fields.len(), 3);

        let tensor = parse_tensor("[1, 2, 3]").unwrap();
        assert_eq!(tensor.shape(), vec![3]);
    }

    #[test]
    fn test_error_types_unified() {
        // Verify unified error handling
        let csv_err = parse_csv_row("a, b,").unwrap_err();
        // CSV trailing comma returns TrailingComma error
        assert!(matches!(csv_err, LexError::TrailingComma));
        assert!(csv_err.is_csv_error());

        let tensor_err = parse_tensor("[]").unwrap_err();
        assert!(tensor_err.is_tensor_error());

        let ref_err = parse_reference("@123invalid").unwrap_err();
        assert!(ref_err.position().is_some());
    }
}

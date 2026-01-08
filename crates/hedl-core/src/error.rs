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

//! Error types for HEDL parsing.

use std::fmt;
use thiserror::Error;

/// The kind of error that occurred during parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HedlErrorKind {
    /// Lexical or structural violation.
    Syntax,
    /// Unsupported version.
    Version,
    /// Schema violation or mismatch.
    Schema,
    /// Duplicate or invalid alias.
    Alias,
    /// Wrong number of cells in row.
    Shape,
    /// Logical error (ditto in ID, null in ID, etc).
    Semantic,
    /// Child row without NEST rule.
    OrphanRow,
    /// Duplicate ID within type.
    Collision,
    /// Unresolved reference in strict mode.
    Reference,
    /// Security limit exceeded.
    Security,
    /// Error during format conversion (JSON, YAML, XML, etc.).
    Conversion,
    /// I/O error (file operations, network, etc.).
    IO,
}

impl fmt::Display for HedlErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax => write!(f, "SyntaxError"),
            Self::Version => write!(f, "VersionError"),
            Self::Schema => write!(f, "SchemaError"),
            Self::Alias => write!(f, "AliasError"),
            Self::Shape => write!(f, "ShapeError"),
            Self::Semantic => write!(f, "SemanticError"),
            Self::OrphanRow => write!(f, "OrphanRowError"),
            Self::Collision => write!(f, "CollisionError"),
            Self::Reference => write!(f, "ReferenceError"),
            Self::Security => write!(f, "SecurityError"),
            Self::Conversion => write!(f, "ConversionError"),
            Self::IO => write!(f, "IOError"),
        }
    }
}

/// An error that occurred during HEDL parsing.
#[derive(Debug, Clone, Error)]
#[error("{kind} at line {line}: {message}")]
pub struct HedlError {
    /// The kind of error.
    pub kind: HedlErrorKind,
    /// Human-readable error message.
    pub message: String,
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based, optional).
    pub column: Option<usize>,
    /// Additional context (e.g., "in list User started at line 5").
    pub context: Option<String>,
}

impl HedlError {
    /// Create a new error.
    pub fn new(kind: HedlErrorKind, message: impl Into<String>, line: usize) -> Self {
        Self {
            kind,
            message: message.into(),
            line,
            column: None,
            context: None,
        }
    }

    /// Add column information.
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Add context information.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    // Convenience constructors for each error kind
    pub fn syntax(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::Syntax, message, line)
    }

    pub fn version(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::Version, message, line)
    }

    pub fn schema(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::Schema, message, line)
    }

    pub fn alias(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::Alias, message, line)
    }

    pub fn shape(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::Shape, message, line)
    }

    pub fn semantic(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::Semantic, message, line)
    }

    pub fn orphan_row(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::OrphanRow, message, line)
    }

    pub fn collision(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::Collision, message, line)
    }

    pub fn reference(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::Reference, message, line)
    }

    pub fn security(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::Security, message, line)
    }

    pub fn conversion(message: impl Into<String>) -> Self {
        Self::new(HedlErrorKind::Conversion, message, 0)
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(HedlErrorKind::IO, message, 0)
    }
}

/// Result type for HEDL operations.
pub type HedlResult<T> = Result<T, HedlError>;

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== HedlErrorKind Display tests ====================

    #[test]
    fn test_error_kind_display_syntax() {
        assert_eq!(format!("{}", HedlErrorKind::Syntax), "SyntaxError");
    }

    #[test]
    fn test_error_kind_display_version() {
        assert_eq!(format!("{}", HedlErrorKind::Version), "VersionError");
    }

    #[test]
    fn test_error_kind_display_schema() {
        assert_eq!(format!("{}", HedlErrorKind::Schema), "SchemaError");
    }

    #[test]
    fn test_error_kind_display_alias() {
        assert_eq!(format!("{}", HedlErrorKind::Alias), "AliasError");
    }

    #[test]
    fn test_error_kind_display_shape() {
        assert_eq!(format!("{}", HedlErrorKind::Shape), "ShapeError");
    }

    #[test]
    fn test_error_kind_display_semantic() {
        assert_eq!(format!("{}", HedlErrorKind::Semantic), "SemanticError");
    }

    #[test]
    fn test_error_kind_display_orphan_row() {
        assert_eq!(format!("{}", HedlErrorKind::OrphanRow), "OrphanRowError");
    }

    #[test]
    fn test_error_kind_display_collision() {
        assert_eq!(format!("{}", HedlErrorKind::Collision), "CollisionError");
    }

    #[test]
    fn test_error_kind_display_reference() {
        assert_eq!(format!("{}", HedlErrorKind::Reference), "ReferenceError");
    }

    #[test]
    fn test_error_kind_display_security() {
        assert_eq!(format!("{}", HedlErrorKind::Security), "SecurityError");
    }

    // ==================== HedlErrorKind equality tests ====================

    #[test]
    fn test_error_kind_equality() {
        assert_eq!(HedlErrorKind::Syntax, HedlErrorKind::Syntax);
        assert_ne!(HedlErrorKind::Syntax, HedlErrorKind::Schema);
    }

    #[test]
    fn test_error_kind_clone() {
        let kind = HedlErrorKind::Reference;
        let cloned = kind.clone();
        assert_eq!(kind, cloned);
    }

    // ==================== HedlError Display tests ====================

    #[test]
    fn test_error_display() {
        let err = HedlError::new(HedlErrorKind::Syntax, "unexpected token", 42);
        let msg = format!("{}", err);
        assert!(msg.contains("SyntaxError"));
        assert!(msg.contains("line 42"));
        assert!(msg.contains("unexpected token"));
    }

    #[test]
    fn test_error_with_column() {
        let err = HedlError::syntax("error", 5).with_column(10);
        assert_eq!(err.column, Some(10));
    }

    #[test]
    fn test_error_with_context() {
        let err = HedlError::syntax("error", 5).with_context("in struct User");
        assert_eq!(err.context, Some("in struct User".to_string()));
    }

    // ==================== Convenience constructor tests ====================

    #[test]
    fn test_error_syntax() {
        let err = HedlError::syntax("test", 1);
        assert_eq!(err.kind, HedlErrorKind::Syntax);
        assert_eq!(err.line, 1);
    }

    #[test]
    fn test_error_version() {
        let err = HedlError::version("test", 2);
        assert_eq!(err.kind, HedlErrorKind::Version);
    }

    #[test]
    fn test_error_schema() {
        let err = HedlError::schema("test", 3);
        assert_eq!(err.kind, HedlErrorKind::Schema);
    }

    #[test]
    fn test_error_alias() {
        let err = HedlError::alias("test", 4);
        assert_eq!(err.kind, HedlErrorKind::Alias);
    }

    #[test]
    fn test_error_shape() {
        let err = HedlError::shape("test", 5);
        assert_eq!(err.kind, HedlErrorKind::Shape);
    }

    #[test]
    fn test_error_semantic() {
        let err = HedlError::semantic("test", 6);
        assert_eq!(err.kind, HedlErrorKind::Semantic);
    }

    #[test]
    fn test_error_orphan_row() {
        let err = HedlError::orphan_row("test", 7);
        assert_eq!(err.kind, HedlErrorKind::OrphanRow);
    }

    #[test]
    fn test_error_collision() {
        let err = HedlError::collision("test", 8);
        assert_eq!(err.kind, HedlErrorKind::Collision);
    }

    #[test]
    fn test_error_reference() {
        let err = HedlError::reference("test", 9);
        assert_eq!(err.kind, HedlErrorKind::Reference);
    }

    #[test]
    fn test_error_security() {
        let err = HedlError::security("test", 10);
        assert_eq!(err.kind, HedlErrorKind::Security);
    }

    #[test]
    fn test_error_conversion() {
        let err = HedlError::conversion("JSON serialization failed");
        assert_eq!(err.kind, HedlErrorKind::Conversion);
        assert_eq!(err.line, 0);
    }

    #[test]
    fn test_error_io() {
        let err = HedlError::io("Failed to read file");
        assert_eq!(err.kind, HedlErrorKind::IO);
        assert_eq!(err.line, 0);
    }

    #[test]
    fn test_error_kind_display_conversion() {
        assert_eq!(format!("{}", HedlErrorKind::Conversion), "ConversionError");
    }

    #[test]
    fn test_error_kind_display_io() {
        assert_eq!(format!("{}", HedlErrorKind::IO), "IOError");
    }

    // ==================== Error trait tests ====================

    #[test]
    fn test_error_is_std_error() {
        fn accepts_error<E: std::error::Error>(_: E) {}
        accepts_error(HedlError::syntax("test", 1));
    }

    #[test]
    fn test_error_clone() {
        let original = HedlError::syntax("message", 5).with_column(10);
        let cloned = original.clone();
        assert_eq!(original.kind, cloned.kind);
        assert_eq!(original.message, cloned.message);
        assert_eq!(original.line, cloned.line);
        assert_eq!(original.column, cloned.column);
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_error_with_empty_message() {
        let err = HedlError::syntax("", 1);
        assert_eq!(err.message, "");
    }

    #[test]
    fn test_error_with_unicode_message() {
        let err = HedlError::syntax("日本語エラー 🎉", 1);
        assert!(err.message.contains("🎉"));
    }

    #[test]
    fn test_error_line_zero() {
        // Line 0 is technically invalid but should still work
        let err = HedlError::syntax("test", 0);
        assert_eq!(err.line, 0);
    }

    #[test]
    fn test_error_large_line() {
        let err = HedlError::syntax("test", usize::MAX);
        assert_eq!(err.line, usize::MAX);
    }

    #[test]
    fn test_error_chained_builders() {
        let err = HedlError::syntax("error", 5)
            .with_column(10)
            .with_context("in list");
        assert_eq!(err.column, Some(10));
        assert_eq!(err.context, Some("in list".to_string()));
    }

    #[test]
    fn test_error_debug() {
        let err = HedlError::syntax("test", 1);
        let debug = format!("{:?}", err);
        assert!(debug.contains("Syntax"));
        assert!(debug.contains("test"));
    }
}

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

//! Unified error types for lexical analysis, CSV parsing, and tensor parsing.
//!
//! This module consolidates all parsing-related error types into a single hierarchy,
//! providing consistent error handling across the lexer subsystem.

use thiserror::Error;

// Re-export SourcePos from span module for backward compatibility
pub use crate::lex::span::SourcePos;

/// Unified error type for all lexical analysis operations.
///
/// This enum consolidates errors from:
/// - Token scanning and validation
/// - CSV/matrix row parsing
/// - Tensor literal parsing
/// - Resource limit enforcement
#[derive(Debug, Clone, Error, PartialEq)]
pub enum LexError {
    // ==================== Indentation errors ====================
    /// Invalid indentation (must be multiple of 2 spaces).
    #[error("line {}, column {}: invalid indentation: {} spaces (must be multiple of 2)", .pos.line(), .pos.column(), .spaces)]
    InvalidIndentation { spaces: usize, pos: SourcePos },

    /// Tab character in indentation (tabs not allowed).
    #[error("line {}, column {}: tab character not allowed for indentation", .pos.line(), .pos.column())]
    TabInIndentation { pos: SourcePos },

    /// Indentation too deep.
    #[error("line {}, column {}: indent depth {} exceeds maximum {}", .pos.line(), .pos.column(), .depth, .max)]
    IndentTooDeep {
        depth: usize,
        max: usize,
        pos: SourcePos,
    },

    // ==================== String/expression errors ====================
    /// Unclosed quoted string.
    #[error("line {}, column {}: unclosed quoted string", .pos.line(), .pos.column())]
    UnclosedQuote { pos: SourcePos },

    /// Unclosed expression.
    #[error("line {}, column {}: unclosed expression", .pos.line(), .pos.column())]
    UnclosedExpression { pos: SourcePos },

    // ==================== Token errors ====================
    /// Invalid reference format.
    #[error("line {}, column {}: invalid reference format: {}", .pos.line(), .pos.column(), .message)]
    InvalidReference { message: String, pos: SourcePos },

    /// Invalid token.
    #[error("line {}, column {}: invalid token: {}", .pos.line(), .pos.column(), .message)]
    InvalidToken { message: String, pos: SourcePos },

    // ==================== Resource limit errors ====================
    /// String too long.
    #[error("line {}, column {}: string length {} exceeds maximum {}", .pos.line(), .pos.column(), .length, .max)]
    StringTooLong {
        length: usize,
        max: usize,
        pos: SourcePos,
    },

    /// Recursion too deep.
    #[error("line {}, column {}: recursion depth {} exceeds maximum {}", .pos.line(), .pos.column(), .depth, .max)]
    RecursionTooDeep {
        depth: usize,
        max: usize,
        pos: SourcePos,
    },

    /// Too many fields.
    #[error("line {}, column {}: field count {} exceeds maximum {}", .pos.line(), .pos.column(), .count, .max)]
    TooManyFields {
        count: usize,
        max: usize,
        pos: SourcePos,
    },

    /// Parenthesis depth exceeded.
    #[error("line {}, column {}: parenthesis depth {} exceeds maximum {}", .pos.line(), .pos.column(), .depth, .max)]
    ParenthesisDepthExceeded {
        depth: usize,
        max: usize,
        pos: SourcePos,
    },

    // ==================== CSV/Row parsing errors ====================
    /// Trailing comma in CSV row.
    #[error("trailing comma not allowed in matrix row")]
    TrailingComma,

    /// Expected comma after closing quote.
    #[error("expected comma after closing quote, got '{0}'")]
    ExpectedCommaAfterQuote(char),

    /// Quote character in unquoted field.
    #[error("quote character found in unquoted CSV field: '{0}'")]
    QuoteInUnquotedField(String),

    // ==================== Tensor parsing errors ====================
    /// Unbalanced brackets in tensor literal.
    #[error("unbalanced brackets in tensor literal")]
    UnbalancedBrackets,

    /// Invalid number in tensor.
    #[error("invalid number in tensor: {0}")]
    InvalidNumber(String),

    /// Empty tensor not allowed.
    #[error("empty tensor not allowed")]
    EmptyTensor,

    /// Mixed types in tensor.
    #[error("mixed types in tensor (expected number, got '{0}')")]
    MixedTypes(String),

    /// Inconsistent dimensions in tensor.
    #[error("inconsistent dimensions in tensor")]
    InconsistentDimensions,

    /// Unexpected character in tensor.
    #[error("unexpected character in tensor: '{0}'")]
    UnexpectedChar(char),

    /// Invalid tensor structure.
    #[error("invalid tensor structure: {0}")]
    InvalidStructure(String),
}

impl LexError {
    /// Get the position where this error occurred, if available.
    ///
    /// Returns `None` for errors that don't have position information
    /// (e.g., some CSV and tensor parsing errors).
    #[inline]
    pub fn position(&self) -> Option<SourcePos> {
        match self {
            LexError::InvalidIndentation { pos, .. } => Some(*pos),
            LexError::TabInIndentation { pos } => Some(*pos),
            LexError::IndentTooDeep { pos, .. } => Some(*pos),
            LexError::UnclosedQuote { pos } => Some(*pos),
            LexError::UnclosedExpression { pos } => Some(*pos),
            LexError::InvalidReference { pos, .. } => Some(*pos),
            LexError::InvalidToken { pos, .. } => Some(*pos),
            LexError::StringTooLong { pos, .. } => Some(*pos),
            LexError::RecursionTooDeep { pos, .. } => Some(*pos),
            LexError::TooManyFields { pos, .. } => Some(*pos),
            LexError::ParenthesisDepthExceeded { pos, .. } => Some(*pos),
            // Errors without position information
            LexError::TrailingComma
            | LexError::ExpectedCommaAfterQuote(_)
            | LexError::QuoteInUnquotedField(_)
            | LexError::UnbalancedBrackets
            | LexError::InvalidNumber(_)
            | LexError::EmptyTensor
            | LexError::MixedTypes(_)
            | LexError::InconsistentDimensions
            | LexError::UnexpectedChar(_)
            | LexError::InvalidStructure(_) => None,
        }
    }

    /// Returns `true` if this is a resource limit error.
    #[inline]
    pub fn is_resource_limit(&self) -> bool {
        matches!(
            self,
            LexError::StringTooLong { .. }
                | LexError::RecursionTooDeep { .. }
                | LexError::TooManyFields { .. }
                | LexError::ParenthesisDepthExceeded { .. }
                | LexError::IndentTooDeep { .. }
        )
    }

    /// Returns `true` if this is a tensor parsing error.
    #[inline]
    pub fn is_tensor_error(&self) -> bool {
        matches!(
            self,
            LexError::UnbalancedBrackets
                | LexError::InvalidNumber(_)
                | LexError::EmptyTensor
                | LexError::MixedTypes(_)
                | LexError::InconsistentDimensions
                | LexError::UnexpectedChar(_)
                | LexError::InvalidStructure(_)
        )
    }

    /// Returns `true` if this is a CSV/row parsing error.
    #[inline]
    pub fn is_csv_error(&self) -> bool {
        matches!(
            self,
            LexError::TrailingComma
                | LexError::ExpectedCommaAfterQuote(_)
                | LexError::QuoteInUnquotedField(_)
                | LexError::UnclosedQuote { .. }
                | LexError::UnclosedExpression { .. }
        )
    }
}

/// Result type for lexer operations.
pub type LexResult<T> = Result<T, LexError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_pos_new() {
        let pos = SourcePos::new(10, 5);
        assert_eq!(pos.line(), 10);
        assert_eq!(pos.column(), 5);
    }

    #[test]
    fn test_source_pos_display() {
        let pos = SourcePos::new(42, 15);
        assert_eq!(format!("{}", pos), "line 42, column 15");
    }

    #[test]
    fn test_source_pos_default() {
        let pos = SourcePos::default();
        assert_eq!(pos.line(), 0);
        assert_eq!(pos.column(), 0);
    }

    #[test]
    fn test_error_position_extraction() {
        let pos = SourcePos::new(10, 20);

        assert_eq!(LexError::InvalidIndentation { spaces: 3, pos }.position(), Some(pos));
        assert_eq!(LexError::TabInIndentation { pos }.position(), Some(pos));
        assert_eq!(LexError::UnclosedQuote { pos }.position(), Some(pos));

        // Errors without position
        assert_eq!(LexError::TrailingComma.position(), None);
        assert_eq!(LexError::EmptyTensor.position(), None);
        assert_eq!(LexError::UnbalancedBrackets.position(), None);
    }

    #[test]
    fn test_is_resource_limit() {
        let pos = SourcePos::new(1, 1);

        assert!(LexError::StringTooLong { length: 100, max: 50, pos }.is_resource_limit());
        assert!(LexError::RecursionTooDeep { depth: 10, max: 5, pos }.is_resource_limit());
        assert!(LexError::TooManyFields { count: 100, max: 50, pos }.is_resource_limit());
        assert!(LexError::IndentTooDeep { depth: 10, max: 5, pos }.is_resource_limit());

        assert!(!LexError::TrailingComma.is_resource_limit());
        assert!(!LexError::EmptyTensor.is_resource_limit());
    }

    #[test]
    fn test_is_tensor_error() {
        assert!(LexError::UnbalancedBrackets.is_tensor_error());
        assert!(LexError::EmptyTensor.is_tensor_error());
        assert!(LexError::InconsistentDimensions.is_tensor_error());
        assert!(LexError::InvalidNumber("abc".to_string()).is_tensor_error());
        assert!(LexError::UnexpectedChar('x').is_tensor_error());

        assert!(!LexError::TrailingComma.is_tensor_error());
        assert!(!LexError::UnclosedQuote { pos: SourcePos::new(1, 1) }.is_tensor_error());
    }

    #[test]
    fn test_is_csv_error() {
        assert!(LexError::TrailingComma.is_csv_error());
        assert!(LexError::ExpectedCommaAfterQuote('x').is_csv_error());
        assert!(LexError::QuoteInUnquotedField("test".to_string()).is_csv_error());
        assert!(LexError::UnclosedQuote { pos: SourcePos::new(1, 1) }.is_csv_error());
        assert!(LexError::UnclosedExpression { pos: SourcePos::new(1, 1) }.is_csv_error());

        assert!(!LexError::EmptyTensor.is_csv_error());
        assert!(!LexError::UnbalancedBrackets.is_csv_error());
    }

    #[test]
    fn test_error_display() {
        let pos = SourcePos::new(5, 10);

        let err = LexError::InvalidIndentation { spaces: 3, pos };
        let msg = format!("{}", err);
        assert!(msg.contains("line 5"));
        assert!(msg.contains("column 10"));
        assert!(msg.contains("3 spaces"));

        let err = LexError::TrailingComma;
        assert_eq!(format!("{}", err), "trailing comma not allowed in matrix row");

        let err = LexError::EmptyTensor;
        assert_eq!(format!("{}", err), "empty tensor not allowed");
    }

    #[test]
    fn test_error_equality() {
        let pos = SourcePos::new(1, 1);

        assert_eq!(
            LexError::UnclosedQuote { pos },
            LexError::UnclosedQuote { pos }
        );
        assert_ne!(
            LexError::UnclosedQuote { pos },
            LexError::UnclosedExpression { pos }
        );
        assert_eq!(LexError::TrailingComma, LexError::TrailingComma);
        assert_eq!(LexError::EmptyTensor, LexError::EmptyTensor);
    }

    #[test]
    fn test_error_clone() {
        let pos = SourcePos::new(5, 10);
        let original = LexError::InvalidToken {
            message: "test".to_string(),
            pos,
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_error_is_std_error() {
        fn accepts_error<E: std::error::Error>(_: E) {}
        accepts_error(LexError::TrailingComma);
        accepts_error(LexError::EmptyTensor);
        accepts_error(LexError::UnclosedQuote { pos: SourcePos::new(1, 1) });
    }
}

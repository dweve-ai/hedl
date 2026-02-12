// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Error types for TOON conversion

use thiserror::Error;

/// Maximum nesting depth allowed (security limit)
pub const MAX_NESTING_DEPTH: usize = 100;

/// Result type for TOON operations
pub type Result<T> = std::result::Result<T, ToonError>;

/// Errors that can occur during TOON conversion
#[derive(Error, Debug)]
pub enum ToonError {
    /// TOON parsing error from the official parser
    #[error("TOON parse error: {0}")]
    ParseError(String),

    /// TOON encoding error
    #[error("TOON encode error: {0}")]
    EncodeError(String),

    /// Maximum nesting depth exceeded
    #[error("Maximum nesting depth of {max} exceeded at depth {depth}")]
    MaxDepthExceeded {
        /// Current depth
        depth: usize,
        /// Maximum allowed depth
        max: usize,
    },

    /// JSON serialization error (intermediate format)
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// HEDL core error
    #[error("HEDL error: {0}")]
    HedlError(#[from] hedl_core::HedlError),
}

impl From<toon_format::ToonError> for ToonError {
    fn from(e: toon_format::ToonError) -> Self {
        ToonError::ParseError(e.to_string())
    }
}

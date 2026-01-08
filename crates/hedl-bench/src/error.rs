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

//! Error types for HEDL benchmarking operations.
//!
//! This module provides structured error handling for dataset generation
//! and benchmark operations, replacing ad-hoc string-based errors with
//! type-safe error variants.

use std::fmt;

/// Maximum dataset size to prevent DoS attacks (10 million entities)
///
/// This limit prevents memory exhaustion from maliciously large dataset requests.
/// Benchmarks should use a reasonable subset of data for meaningful results.
pub const MAX_DATASET_SIZE: usize = 10_000_000;

/// Result type for benchmarking operations
pub type Result<T> = std::result::Result<T, BenchError>;

/// Errors that can occur during benchmarking operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BenchError {
    /// Dataset size exceeds maximum allowed limit
    DatasetTooLarge {
        /// Requested size
        requested: usize,
        /// Maximum allowed size
        max: usize,
    },

    /// Invalid configuration parameter
    InvalidConfig {
        /// Parameter name
        parameter: String,
        /// Reason for invalidity
        reason: String,
    },

    /// Failed to generate dataset
    GenerationFailed {
        /// Dataset type being generated
        dataset_type: String,
        /// Error message
        message: String,
    },

    /// Token counting operation failed
    TokenCountFailed {
        /// Reason for failure
        reason: String,
    },

    /// Question generation failed
    QuestionGenFailed {
        /// Question type
        question_type: String,
        /// Error message
        message: String,
    },

    /// Accuracy measurement failed
    AccuracyFailed {
        /// Reason for failure
        reason: String,
    },

    /// Normalization failed
    NormalizationFailed {
        /// Value being normalized
        value: String,
        /// Reason for failure
        reason: String,
    },

    /// Comparison failed
    ComparisonFailed {
        /// Reason for failure
        reason: String,
    },

    /// Conversion operation failed
    ConversionError(String),

    /// I/O error
    IoError(String),

    /// Parse error
    ParseError(String),

    /// Validation error
    ValidationError(String),

    /// Streaming operation failed
    StreamError(String),
}

impl fmt::Display for BenchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BenchError::DatasetTooLarge { requested, max } => {
                write!(
                    f,
                    "Dataset size {} exceeds maximum allowed limit of {}",
                    requested, max
                )
            }
            BenchError::InvalidConfig { parameter, reason } => {
                write!(
                    f,
                    "Invalid configuration parameter '{}': {}",
                    parameter, reason
                )
            }
            BenchError::GenerationFailed {
                dataset_type,
                message,
            } => {
                write!(
                    f,
                    "Failed to generate {} dataset: {}",
                    dataset_type, message
                )
            }
            BenchError::TokenCountFailed { reason } => {
                write!(f, "Token counting failed: {}", reason)
            }
            BenchError::QuestionGenFailed {
                question_type,
                message,
            } => {
                write!(
                    f,
                    "Failed to generate {} questions: {}",
                    question_type, message
                )
            }
            BenchError::AccuracyFailed { reason } => {
                write!(f, "Accuracy measurement failed: {}", reason)
            }
            BenchError::NormalizationFailed { value, reason } => {
                write!(f, "Failed to normalize '{}': {}", value, reason)
            }
            BenchError::ComparisonFailed { reason } => {
                write!(f, "Comparison failed: {}", reason)
            }
            BenchError::ConversionError(msg) => {
                write!(f, "Conversion error: {}", msg)
            }
            BenchError::IoError(msg) => {
                write!(f, "I/O error: {}", msg)
            }
            BenchError::ParseError(msg) => {
                write!(f, "Parse error: {}", msg)
            }
            BenchError::ValidationError(msg) => {
                write!(f, "Validation error: {}", msg)
            }
            BenchError::StreamError(msg) => {
                write!(f, "Stream error: {}", msg)
            }
        }
    }
}

impl std::error::Error for BenchError {}

/// Validate that a dataset size is within acceptable limits
///
/// # Arguments
///
/// * `size` - The requested dataset size
///
/// # Returns
///
/// `Ok(())` if the size is valid, or a [`BenchError::DatasetTooLarge`] error
///
/// # Examples
///
/// ```no_run
/// use hedl_bench::error::{validate_dataset_size, MAX_DATASET_SIZE};
///
/// assert!(validate_dataset_size(1000).is_ok());
/// assert!(validate_dataset_size(MAX_DATASET_SIZE + 1).is_err());
/// ```
#[inline]
pub fn validate_dataset_size(size: usize) -> Result<()> {
    if size > MAX_DATASET_SIZE {
        Err(BenchError::DatasetTooLarge {
            requested: size,
            max: MAX_DATASET_SIZE,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_dataset_size_success() {
        assert!(validate_dataset_size(100).is_ok());
        assert!(validate_dataset_size(10_000).is_ok());
        assert!(validate_dataset_size(MAX_DATASET_SIZE).is_ok());
    }

    #[test]
    fn test_validate_dataset_size_failure() {
        let result = validate_dataset_size(MAX_DATASET_SIZE + 1);
        assert!(result.is_err());

        if let Err(BenchError::DatasetTooLarge { requested, max }) = result {
            assert_eq!(requested, MAX_DATASET_SIZE + 1);
            assert_eq!(max, MAX_DATASET_SIZE);
        } else {
            panic!("Expected DatasetTooLarge error");
        }
    }

    #[test]
    fn test_error_display() {
        let err = BenchError::DatasetTooLarge {
            requested: 20_000_000,
            max: MAX_DATASET_SIZE,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("20000000"));
        assert!(msg.contains("10000000"));

        let err = BenchError::InvalidConfig {
            parameter: "indent".to_string(),
            reason: "must be positive".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("indent"));
        assert!(msg.contains("must be positive"));
    }

    #[test]
    fn test_error_equality() {
        let err1 = BenchError::DatasetTooLarge {
            requested: 100,
            max: 10,
        };
        let err2 = BenchError::DatasetTooLarge {
            requested: 100,
            max: 10,
        };
        let err3 = BenchError::DatasetTooLarge {
            requested: 200,
            max: 10,
        };

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }
}

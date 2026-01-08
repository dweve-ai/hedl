// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Error types for YAML conversion operations.

use thiserror::Error;

/// Errors that can occur during YAML to HEDL conversion.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum YamlError {
    /// YAML parsing failed
    #[error("YAML parse error: {0}")]
    ParseError(String),

    /// Root element must be a mapping/object
    #[error("Root must be a YAML mapping, found {found}")]
    InvalidRootType { found: String },

    /// Non-string key encountered in mapping
    #[error("Non-string keys not supported, found {key_type} at path {path}")]
    NonStringKey { key_type: String, path: String },

    /// Invalid number format
    #[error("Invalid number format: {value}")]
    InvalidNumber { value: String },

    /// Invalid expression syntax
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),

    /// Invalid reference format
    #[error("Invalid reference format: {0}")]
    InvalidReference(String),

    /// Nested objects not allowed in scalar context
    #[error("Nested objects not allowed in scalar context at path {path}")]
    NestedObjectInScalar { path: String },

    /// Invalid tensor element type
    #[error("Invalid tensor element at path {path}: must be number or sequence")]
    InvalidTensorElement { path: String },

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {limit_type} (limit: {limit}, actual: {actual})")]
    ResourceLimitExceeded {
        limit_type: String,
        limit: usize,
        actual: usize,
    },

    /// Maximum nesting depth exceeded
    #[error("Maximum nesting depth of {max_depth} exceeded at depth {actual_depth}")]
    MaxDepthExceeded {
        max_depth: usize,
        actual_depth: usize,
    },

    /// Document too large
    #[error("Document size {size} bytes exceeds maximum of {max_size} bytes")]
    DocumentTooLarge { size: usize, max_size: usize },

    /// Array too long
    #[error("Array length {length} exceeds maximum of {max_length} at path {path}")]
    ArrayTooLong {
        length: usize,
        max_length: usize,
        path: String,
    },

    /// Generic conversion error
    #[error("Conversion error: {0}")]
    Conversion(String),
}

impl From<serde_yaml::Error> for YamlError {
    fn from(err: serde_yaml::Error) -> Self {
        YamlError::ParseError(err.to_string())
    }
}

impl From<String> for YamlError {
    fn from(err: String) -> Self {
        YamlError::Conversion(err)
    }
}

impl From<&str> for YamlError {
    fn from(err: &str) -> Self {
        YamlError::Conversion(err.to_string())
    }
}

impl From<hedl_core::lex::LexError> for YamlError {
    fn from(err: hedl_core::lex::LexError) -> Self {
        YamlError::InvalidExpression(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = YamlError::ParseError("invalid syntax".to_string());
        assert_eq!(err.to_string(), "YAML parse error: invalid syntax");
    }

    #[test]
    fn test_invalid_root_type_display() {
        let err = YamlError::InvalidRootType {
            found: "sequence".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Root must be a YAML mapping, found sequence"
        );
    }

    #[test]
    fn test_non_string_key_display() {
        let err = YamlError::NonStringKey {
            key_type: "number".to_string(),
            path: "root.config".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Non-string keys not supported, found number at path root.config"
        );
    }

    #[test]
    fn test_resource_limit_exceeded_display() {
        let err = YamlError::ResourceLimitExceeded {
            limit_type: "array_length".to_string(),
            limit: 1000,
            actual: 2000,
        };
        assert_eq!(
            err.to_string(),
            "Resource limit exceeded: array_length (limit: 1000, actual: 2000)"
        );
    }

    #[test]
    fn test_max_depth_exceeded_display() {
        let err = YamlError::MaxDepthExceeded {
            max_depth: 100,
            actual_depth: 150,
        };
        assert_eq!(
            err.to_string(),
            "Maximum nesting depth of 100 exceeded at depth 150"
        );
    }

    #[test]
    fn test_document_too_large_display() {
        let err = YamlError::DocumentTooLarge {
            size: 20_000_000,
            max_size: 10_000_000,
        };
        assert_eq!(
            err.to_string(),
            "Document size 20000000 bytes exceeds maximum of 10000000 bytes"
        );
    }

    #[test]
    fn test_array_too_long_display() {
        let err = YamlError::ArrayTooLong {
            length: 2000,
            max_length: 1000,
            path: "root.items".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Array length 2000 exceeds maximum of 1000 at path root.items"
        );
    }

    #[test]
    fn test_error_clone() {
        let err1 = YamlError::ParseError("test".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_error_equality() {
        let err1 = YamlError::ParseError("test".to_string());
        let err2 = YamlError::ParseError("test".to_string());
        let err3 = YamlError::ParseError("different".to_string());

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_from_string() {
        let err: YamlError = "test error".to_string().into();
        match err {
            YamlError::Conversion(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Expected Conversion error"),
        }
    }

    #[test]
    fn test_from_str() {
        let err: YamlError = "test error".into();
        match err {
            YamlError::Conversion(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Expected Conversion error"),
        }
    }
}

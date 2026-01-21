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

//! Error types for fix operations

use thiserror::Error;

/// Errors that can occur during fix application
#[derive(Debug, Clone, Error)]
pub enum FixError {
    /// Invalid source range
    #[error("Invalid source range: {0}")]
    InvalidRange(String),

    /// Parse failed after applying fix
    #[error("Parse failed after fix: {0}")]
    ParseFailure(String),

    /// New violations introduced by fix
    #[error("New violations introduced: {0}")]
    NewViolations(String),

    /// Fix dependency not satisfied
    #[error("Dependency not satisfied: {0}")]
    DependencyError(String),

    /// Circular dependency detected
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// Source text encoding error
    #[error("Source encoding error: {0}")]
    EncodingError(String),

    /// Fix application failed
    #[error("Fix application failed: {0}")]
    ApplicationFailed(String),

    /// Conflict resolution failed
    #[error("Conflict resolution failed: {0}")]
    ConflictResolutionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = FixError::InvalidRange("test range".to_string());
        assert_eq!(err.to_string(), "Invalid source range: test range");
    }

    #[test]
    fn test_parse_failure_error() {
        let err = FixError::ParseFailure("syntax error".to_string());
        assert!(err.to_string().contains("Parse failed"));
        assert!(err.to_string().contains("syntax error"));
    }

    #[test]
    fn test_new_violations_error() {
        let err = FixError::NewViolations("rule violation".to_string());
        assert!(err.to_string().contains("New violations"));
    }

    #[test]
    fn test_dependency_error() {
        let err = FixError::DependencyError("missing dep".to_string());
        assert!(err.to_string().contains("Dependency not satisfied"));
    }

    #[test]
    fn test_circular_dependency_error() {
        let err = FixError::CircularDependency("A -> B -> A".to_string());
        assert!(err.to_string().contains("Circular dependency"));
    }

    #[test]
    fn test_encoding_error() {
        let err = FixError::EncodingError("invalid UTF-8".to_string());
        assert!(err.to_string().contains("encoding"));
    }

    #[test]
    fn test_application_failed_error() {
        let err = FixError::ApplicationFailed("unknown reason".to_string());
        assert!(err.to_string().contains("application failed"));
    }

    #[test]
    fn test_conflict_resolution_failed_error() {
        let err = FixError::ConflictResolutionFailed("no strategy".to_string());
        assert!(err.to_string().contains("Conflict resolution failed"));
    }

    #[test]
    fn test_error_clone() {
        let err1 = FixError::InvalidRange("test".to_string());
        let err2 = err1.clone();
        assert_eq!(err1.to_string(), err2.to_string());
    }
}

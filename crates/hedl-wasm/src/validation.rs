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

//! Validation result types.

use serde::Serialize;

/// Validation result.
#[derive(Serialize)]
pub struct ValidationResult {
    /// Whether the document is valid.
    pub(crate) valid: bool,
    /// List of validation errors.
    pub(crate) errors: Vec<ValidationError>,
    /// List of validation warnings.
    pub(crate) warnings: Vec<ValidationWarning>,
}

/// A validation error.
#[derive(Serialize)]
pub struct ValidationError {
    /// Line number where the error occurred.
    pub(crate) line: usize,
    /// Error message.
    pub(crate) message: String,
    /// Error type identifier.
    #[serde(rename = "type")]
    pub(crate) error_type: String,
}

/// A validation warning.
#[derive(Serialize)]
pub struct ValidationWarning {
    /// Line number where the warning occurred.
    pub(crate) line: usize,
    /// Warning message.
    pub(crate) message: String,
    /// Lint rule that generated this warning.
    pub(crate) rule: String,
}

impl ValidationResult {
    /// Create a new validation result.
    pub(crate) fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create an invalid validation result with an error.
    pub(crate) fn with_error(line: usize, message: String, error_type: String) -> Self {
        Self {
            valid: false,
            errors: vec![ValidationError {
                line,
                message,
                error_type,
            }],
            warnings: Vec::new(),
        }
    }
}

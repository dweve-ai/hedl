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

//! Request size validation limits.

use super::error::ResourceLimitError;
use crate::protocol::JsonRpcRequest;
use serde_json::Value;

/// Request size validation limits.
///
/// Enforces size constraints on incoming JSON-RPC requests to prevent
/// memory exhaustion and parsing `DoS` attacks.
#[derive(Debug, Clone)]
pub struct RequestSizeLimits {
    /// Maximum total request size in bytes.
    max_total_size: usize,

    /// Maximum individual parameter size in bytes.
    max_param_size: usize,

    /// Maximum array element count.
    max_array_elements: usize,

    /// Maximum JSON object nesting depth.
    max_object_depth: usize,
}

impl RequestSizeLimits {
    /// Create new request size limits with specified values.
    ///
    /// # Arguments
    ///
    /// * `max_total_size` - Maximum total request size in bytes
    /// * `max_param_size` - Maximum individual parameter size in bytes
    /// * `max_array_elements` - Maximum array element count
    /// * `max_object_depth` - Maximum JSON object nesting depth
    #[must_use]
    pub fn new(
        max_total_size: usize,
        max_param_size: usize,
        max_array_elements: usize,
        max_object_depth: usize,
    ) -> Self {
        Self {
            max_total_size,
            max_param_size,
            max_array_elements,
            max_object_depth,
        }
    }

    /// Get default request size limits.
    ///
    /// Returns limits suitable for most production environments:
    /// - 10 MB total request size
    /// - 5 MB per parameter
    /// - 10,000 array elements
    /// - 32 object nesting depth
    #[must_use]
    pub fn default_limits() -> Self {
        Self {
            max_total_size: 10_485_760, // 10 MB
            max_param_size: 5_242_880,  // 5 MB
            max_array_elements: 10_000,
            max_object_depth: 32,
        }
    }

    /// Check raw request byte size before parsing.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw request bytes
    ///
    /// # Returns
    ///
    /// `Ok(())` if size is within limits, `Err` if exceeded.
    pub fn check_raw_size(&self, bytes: &[u8]) -> Result<(), ResourceLimitError> {
        if bytes.len() > self.max_total_size {
            return Err(ResourceLimitError::RequestTooLarge {
                size: bytes.len(),
                limit: self.max_total_size,
                exceeded_by: bytes.len() - self.max_total_size,
            });
        }
        Ok(())
    }

    /// Validate parsed JSON-RPC request structure.
    ///
    /// # Arguments
    ///
    /// * `request` - Parsed JSON-RPC request
    ///
    /// # Returns
    ///
    /// `Ok(())` if request structure is valid, `Err` if limits exceeded.
    pub fn check_parsed_request(&self, request: &JsonRpcRequest) -> Result<(), ResourceLimitError> {
        if let Some(params) = &request.params {
            self.validate_json_value(params, 0)?;
        }
        Ok(())
    }

    /// Recursively validate JSON value against size limits.
    fn validate_json_value(&self, value: &Value, depth: usize) -> Result<(), ResourceLimitError> {
        if depth > self.max_object_depth {
            return Err(ResourceLimitError::JsonTooDeep {
                depth,
                limit: self.max_object_depth,
            });
        }

        match value {
            Value::String(s) if s.len() > self.max_param_size => {
                Err(ResourceLimitError::StringTooLarge {
                    size: s.len(),
                    limit: self.max_param_size,
                })
            }
            Value::Array(arr) if arr.len() > self.max_array_elements => {
                Err(ResourceLimitError::ArrayTooLarge {
                    size: arr.len(),
                    limit: self.max_array_elements,
                })
            }
            Value::Array(arr) => {
                for item in arr {
                    self.validate_json_value(item, depth + 1)?;
                }
                Ok(())
            }
            Value::Object(obj) => {
                for (key, val) in obj {
                    // Check key size
                    if key.len() > self.max_param_size {
                        return Err(ResourceLimitError::StringTooLarge {
                            size: key.len(),
                            limit: self.max_param_size,
                        });
                    }
                    self.validate_json_value(val, depth + 1)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Get the maximum total request size.
    #[must_use]
    pub fn max_total_size(&self) -> usize {
        self.max_total_size
    }

    /// Get the maximum parameter size.
    #[must_use]
    pub fn max_param_size(&self) -> usize {
        self.max_param_size
    }

    /// Get the maximum array elements.
    #[must_use]
    pub fn max_array_elements(&self) -> usize {
        self.max_array_elements
    }

    /// Get the maximum object depth.
    #[must_use]
    pub fn max_object_depth(&self) -> usize {
        self.max_object_depth
    }
}

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

//! Response size validation limits.

use super::error::ResourceLimitError;
use crate::protocol::{CallToolResult, Content};

/// Response size validation limits.
///
/// Enforces size constraints on outgoing responses to prevent excessive
/// memory allocation and network saturation.
#[derive(Debug, Clone)]
pub struct ResponseSizeLimits {
    /// Maximum total response size in bytes.
    max_total_size: usize,

    /// Maximum number of result items for array responses.
    max_result_items: usize,

    /// Whether streaming is enabled for large results.
    enable_streaming: bool,
}

impl ResponseSizeLimits {
    /// Create new response size limits with specified values.
    ///
    /// # Arguments
    ///
    /// * `max_total_size` - Maximum total response size in bytes
    /// * `max_result_items` - Maximum result count for array responses
    /// * `enable_streaming` - Whether to enable streaming for large results
    #[must_use]
    pub fn new(max_total_size: usize, max_result_items: usize, enable_streaming: bool) -> Self {
        Self {
            max_total_size,
            max_result_items,
            enable_streaming,
        }
    }

    /// Get default response size limits.
    ///
    /// Returns limits suitable for most production environments:
    /// - 50 MB total response size
    /// - 100,000 result items
    /// - Streaming enabled
    #[must_use]
    pub fn default_limits() -> Self {
        Self {
            max_total_size: 50_000_000, // 50 MB
            max_result_items: 100_000,
            enable_streaming: true,
        }
    }

    /// Estimate the size of a tool call result.
    ///
    /// # Arguments
    ///
    /// * `result` - Tool call result to estimate
    ///
    /// # Returns
    ///
    /// Estimated size in bytes, or error if exceeds limit.
    pub fn estimate_size(&self, result: &CallToolResult) -> Result<usize, ResourceLimitError> {
        let mut size = 0;
        for content in &result.content {
            size += match content {
                Content::Text { text } => text.len(),
                Content::Resource { resource } => {
                    resource.text.as_ref().map_or(0, std::string::String::len)
                }
            };
        }

        if size > self.max_total_size {
            return Err(ResourceLimitError::ResponseTooLarge {
                estimated_size: size,
                limit: self.max_total_size,
            });
        }

        Ok(size)
    }

    /// Check if result count is within limits.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of results
    ///
    /// # Returns
    ///
    /// `Ok(())` if count is within limits, `Err` if exceeded.
    pub fn check_result_count(&self, count: usize) -> Result<(), ResourceLimitError> {
        if count > self.max_result_items {
            return Err(ResourceLimitError::TooManyResults {
                count,
                limit: self.max_result_items,
            });
        }
        Ok(())
    }

    /// Get the maximum total response size.
    #[must_use]
    pub fn max_total_size(&self) -> usize {
        self.max_total_size
    }

    /// Get the maximum result items.
    #[must_use]
    pub fn max_result_items(&self) -> usize {
        self.max_result_items
    }

    /// Check if streaming is enabled.
    #[must_use]
    pub fn is_streaming_enabled(&self) -> bool {
        self.enable_streaming
    }
}

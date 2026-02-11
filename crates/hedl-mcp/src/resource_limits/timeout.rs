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

//! Operation timeout limits.

use super::error::ResourceLimitError;
use std::collections::HashMap;
use std::time::Duration;

/// Operation timeout limits.
///
/// Enforces maximum execution time for operations to prevent resource
/// exhaustion from long-running or hung operations.
pub struct TimeoutLimits {
    /// Default timeout for all operations.
    default_timeout: Duration,

    /// Per-tool timeout overrides.
    per_tool_timeouts: HashMap<String, Duration>,
}

impl TimeoutLimits {
    /// Create new timeout limits with default timeout.
    ///
    /// # Arguments
    ///
    /// * `default_timeout` - Default operation timeout
    #[must_use]
    pub fn new(default_timeout: Duration) -> Self {
        Self {
            default_timeout,
            per_tool_timeouts: HashMap::new(),
        }
    }

    /// Create with default timeout configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut limits = Self::new(Duration::from_millis(30_000)); // 30 seconds

        // Add per-tool overrides
        limits
            .per_tool_timeouts
            .insert("hedl_validate".to_string(), Duration::from_millis(5_000)); // 5 seconds
        limits
            .per_tool_timeouts
            .insert("hedl_query".to_string(), Duration::from_millis(10_000)); // 10 seconds
        limits.per_tool_timeouts.insert(
            "hedl_convert_to".to_string(),
            Duration::from_millis(60_000), // 60 seconds
        );
        limits
            .per_tool_timeouts
            .insert("hedl_stream".to_string(), Duration::from_millis(120_000)); // 120 seconds

        limits
    }

    /// Execute an operation with timeout.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name of the tool being executed
    /// * `operation` - Async operation to execute
    ///
    /// # Returns
    ///
    /// `Ok(T)` if operation completes within timeout, `Err` if timeout.
    pub async fn execute_with_timeout<F, T>(
        &self,
        tool_name: &str,
        operation: F,
    ) -> Result<T, ResourceLimitError>
    where
        F: std::future::Future<Output = T>,
    {
        let timeout = self
            .per_tool_timeouts
            .get(tool_name)
            .copied()
            .unwrap_or(self.default_timeout);

        tokio::time::timeout(timeout, operation).await.map_err(|_| {
            ResourceLimitError::OperationTimeout {
                tool_name: tool_name.to_string(),
                timeout_ms: timeout.as_millis() as u64,
            }
        })
    }

    /// Get timeout for a specific tool.
    #[must_use]
    pub fn get_timeout(&self, tool_name: &str) -> Duration {
        self.per_tool_timeouts
            .get(tool_name)
            .copied()
            .unwrap_or(self.default_timeout)
    }

    /// Get the default timeout.
    #[must_use]
    pub fn default_timeout(&self) -> Duration {
        self.default_timeout
    }

    /// Set timeout for a specific tool.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name of the tool
    /// * `timeout` - Timeout duration for the tool
    pub fn set_tool_timeout(&mut self, tool_name: String, timeout: Duration) {
        self.per_tool_timeouts.insert(tool_name, timeout);
    }
}

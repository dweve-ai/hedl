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

//! Resource limit error types.

use crate::error::McpError;

/// Resource limit enforcement error.
///
/// Represents various types of resource limit violations with actionable
/// error messages and context.
#[derive(Debug, thiserror::Error)]
pub enum ResourceLimitError {
    /// Request size exceeds configured limit.
    #[error(
        "Request size {size} bytes exceeds limit {limit} bytes (exceeded by {exceeded_by} bytes)"
    )]
    RequestTooLarge {
        /// Actual request size in bytes.
        size: usize,
        /// Maximum allowed request size in bytes.
        limit: usize,
        /// Amount by which the limit was exceeded.
        exceeded_by: usize,
    },

    /// String parameter exceeds size limit.
    #[error("String parameter size {size} bytes exceeds limit {limit} bytes")]
    StringTooLarge {
        /// Actual string size in bytes.
        size: usize,
        /// Maximum allowed string size in bytes.
        limit: usize,
    },

    /// Array element count exceeds limit.
    #[error("Array has {size} elements, exceeds limit {limit}")]
    ArrayTooLarge {
        /// Actual array element count.
        size: usize,
        /// Maximum allowed array elements.
        limit: usize,
    },

    /// JSON object nesting depth exceeds limit.
    #[error("JSON depth {depth} exceeds limit {limit}")]
    JsonTooDeep {
        /// Actual nesting depth.
        depth: usize,
        /// Maximum allowed nesting depth.
        limit: usize,
    },

    /// Response size exceeds configured limit.
    #[error("Response estimated size {estimated_size} bytes exceeds limit {limit} bytes")]
    ResponseTooLarge {
        /// Estimated response size in bytes.
        estimated_size: usize,
        /// Maximum allowed response size in bytes.
        limit: usize,
    },

    /// Result count exceeds configured limit.
    #[error("Result count {count} exceeds limit {limit}")]
    TooManyResults {
        /// Actual result count.
        count: usize,
        /// Maximum allowed result count.
        limit: usize,
    },

    /// Rate limit exceeded for a specific client.
    #[error("Rate limit exceeded for client '{client_id}': burst={burst}, rate={rate}/s")]
    RateLimitExceeded {
        /// Client identifier that exceeded the limit.
        client_id: String,
        /// Maximum burst capacity.
        burst: usize,
        /// Refill rate per second.
        rate: usize,
    },

    /// Cache memory usage exceeds limit.
    #[error("Cache memory {current} bytes exceeds limit {limit} bytes, needs {needed} bytes")]
    CacheMemoryExceeded {
        /// Current cache memory usage in bytes.
        current: usize,
        /// Maximum allowed cache memory in bytes.
        limit: usize,
        /// Additional memory needed for the operation.
        needed: usize,
    },

    /// Global concurrency limit exceeded.
    #[error("Global concurrency limit {limit} exceeded, queue timeout")]
    GlobalConcurrencyExceeded {
        /// Maximum concurrent requests allowed globally.
        limit: usize,
    },

    /// Per-client concurrency limit exceeded.
    #[error("Client concurrency limit {limit} exceeded for client '{client_id}', queue timeout")]
    ClientConcurrencyExceeded {
        /// Client identifier that exceeded the limit.
        client_id: String,
        /// Maximum concurrent requests allowed per client.
        limit: usize,
    },

    /// Per-tool concurrency limit exceeded.
    #[error("Tool concurrency limit {limit} exceeded for tool '{tool_name}', queue timeout")]
    ToolConcurrencyExceeded {
        /// Name of the tool that exceeded the limit.
        tool_name: String,
        /// Maximum concurrent requests allowed per tool.
        limit: usize,
    },

    /// Operation execution timeout.
    #[error("Operation '{tool_name}' exceeded timeout {timeout_ms}ms")]
    OperationTimeout {
        /// Name of the tool that timed out.
        tool_name: String,
        /// Timeout duration in milliseconds.
        timeout_ms: u64,
    },

    /// Semaphore closed unexpectedly.
    #[error("Semaphore closed unexpectedly")]
    SemaphoreClosed,
}

impl ResourceLimitError {
    /// Get the JSON-RPC error code for this resource limit error.
    #[must_use]
    pub fn error_code(&self) -> i32 {
        match self {
            Self::RequestTooLarge { .. }
            | Self::StringTooLarge { .. }
            | Self::ArrayTooLarge { .. }
            | Self::JsonTooDeep { .. } => -32006,
            Self::ResponseTooLarge { .. } | Self::TooManyResults { .. } => -32009,
            Self::RateLimitExceeded { .. } => -32005,
            Self::CacheMemoryExceeded { .. } => -32010,
            Self::GlobalConcurrencyExceeded { .. }
            | Self::ClientConcurrencyExceeded { .. }
            | Self::ToolConcurrencyExceeded { .. } => -32007,
            Self::OperationTimeout { .. } => -32008,
            Self::SemaphoreClosed => -32011,
        }
    }
}

impl From<ResourceLimitError> for McpError {
    fn from(err: ResourceLimitError) -> Self {
        McpError::InvalidRequest(err.to_string())
    }
}

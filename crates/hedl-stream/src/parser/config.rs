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

//! Configuration for streaming parser

use crate::buffer_config::BufferSizeHint;
use crate::buffer_pool::MemoryLimits;
use std::time::Duration;

/// Configuration options for the streaming parser.
///
/// Controls memory limits, buffer sizes, timeout behavior, and buffer pooling.
#[derive(Debug, Clone)]
pub struct StreamingParserConfig {
    /// Maximum line length in bytes.
    ///
    /// Lines exceeding this length will cause a parsing error. This protects against
    /// malformed input with extremely long lines that could exhaust memory.
    ///
    /// Default: 1,000,000 bytes (1MB)
    pub max_line_length: usize,

    /// Maximum indentation depth.
    ///
    /// Indentation levels exceeding this depth will cause a parsing error. This
    /// protects against deeply nested structures that could cause stack overflow
    /// or performance issues.
    ///
    /// Default: 100 levels
    pub max_indent_depth: usize,

    /// Buffer size for reading input.
    ///
    /// Larger buffers can improve performance for large files by reducing the
    /// number of system calls, but use more memory.
    ///
    /// Default: 64KB
    pub buffer_size: usize,

    /// Timeout for parsing operations.
    ///
    /// If set, the parser will return a `StreamError::Timeout` if parsing takes
    /// longer than the specified duration. This protects against infinite loops
    /// from malicious or malformed input.
    ///
    /// Set to `None` to disable timeout checking (default for trusted input).
    ///
    /// Default: None (no timeout)
    ///
    /// # Performance Note
    ///
    /// Timeout checking is performed periodically (every 100 operations) to minimize
    /// overhead. For very fast parsing, the actual timeout may slightly exceed the
    /// configured limit.
    pub timeout: Option<Duration>,

    /// Memory limits for buffer management.
    ///
    /// Controls maximum buffer sizes, line lengths, and pool configuration.
    /// See [`MemoryLimits`] for preset configurations.
    ///
    /// Default: `MemoryLimits::default()`
    pub memory_limits: MemoryLimits,

    /// Enable buffer pooling for high-throughput scenarios.
    ///
    /// When enabled, the parser reuses string and value buffers across operations,
    /// reducing allocation overhead. Beneficial for processing many files in sequence
    /// or high-throughput server workloads.
    ///
    /// Default: false (for backward compatibility)
    pub enable_pooling: bool,
}

impl Default for StreamingParserConfig {
    fn default() -> Self {
        Self {
            max_line_length: 1_000_000,
            max_indent_depth: 100,
            buffer_size: 64 * 1024,
            timeout: None,
            memory_limits: MemoryLimits::default(),
            enable_pooling: false,
        }
    }
}

impl StreamingParserConfig {
    /// Config with no limits (use for trusted input only).
    ///
    /// # Security Warning
    ///
    /// This configuration removes the line length limit, which can expose
    /// your application to denial-of-service attacks if processing untrusted input.
    /// Only use this for trusted, controlled environments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::StreamingParserConfig;
    ///
    /// // For trusted input where you want to allow arbitrarily long lines
    /// let config = StreamingParserConfig::unlimited();
    /// ```
    #[must_use]
    pub fn unlimited() -> Self {
        Self {
            max_line_length: usize::MAX,
            ..Default::default()
        }
    }

    /// Configure buffer size using a size hint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::{StreamingParserConfig, BufferSizeHint};
    ///
    /// let config = StreamingParserConfig::default()
    ///     .with_buffer_hint(BufferSizeHint::Large);
    /// assert_eq!(config.buffer_size, 256 * 1024);
    /// ```
    #[must_use]
    pub fn with_buffer_hint(mut self, hint: BufferSizeHint) -> Self {
        self.buffer_size = hint.size();
        self
    }

    /// Enable or disable buffer pooling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::StreamingParserConfig;
    ///
    /// let config = StreamingParserConfig::default()
    ///     .with_buffer_pooling(true);
    /// assert_eq!(config.enable_pooling, true);
    /// ```
    #[must_use]
    pub fn with_buffer_pooling(mut self, enabled: bool) -> Self {
        self.enable_pooling = enabled;
        self
    }

    /// Configure memory limits.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::{StreamingParserConfig, MemoryLimits};
    ///
    /// let config = StreamingParserConfig::default()
    ///     .with_memory_limits(MemoryLimits::high_throughput());
    /// ```
    #[must_use]
    pub fn with_memory_limits(mut self, limits: MemoryLimits) -> Self {
        self.memory_limits = limits;
        // Sync max_line_length with memory limits
        self.max_line_length = limits.max_line_length;
        self
    }

    /// Configure buffer pool size (when pooling is enabled).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::StreamingParserConfig;
    ///
    /// let config = StreamingParserConfig::default()
    ///     .with_buffer_pooling(true)
    ///     .with_pool_size(50);
    /// assert_eq!(config.memory_limits.max_pool_size, 50);
    /// ```
    #[must_use]
    pub fn with_pool_size(mut self, size: usize) -> Self {
        self.memory_limits.max_pool_size = size;
        self
    }
}

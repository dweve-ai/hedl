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

//! Parser configuration options.

use crate::limits::Limits;
use crate::reference::ReferenceMode;

/// Parsing options for configuring HEDL document parsing behavior.
///
/// ParseOptions provides both direct field access and a fluent builder API
/// for convenient configuration. All parsing functions accept ParseOptions
/// to customize limits, security settings, and error handling behavior.
///
/// # Creating ParseOptions
///
/// ## Using the builder pattern (recommended)
///
/// ```text
/// use hedl_core::ParseOptions;
///
/// // Typical strict parsing with custom depth limit
/// let opts = ParseOptions::builder()
///     .max_depth(100)
///     .strict(true)
///     .build();
///
/// // Lenient parsing for large datasets
/// let opts = ParseOptions::builder()
///     .max_array_length(50_000)
///     .strict(false)
///     .max_block_string_size(50 * 1024 * 1024)
///     .build();
///
/// // Restrictive parsing for security
/// let opts = ParseOptions::builder()
///     .max_file_size(10 * 1024 * 1024)
///     .max_line_length(64 * 1024)
///     .max_depth(20)
///     .max_array_length(1000)
///     .strict(true)
///     .build();
/// ```
///
/// ## Using defaults
///
/// ```text
/// use hedl_core::{ParseOptions, parse_with_limits};
///
/// // Default options: strict refs, normal limits
/// let opts = ParseOptions::default();
///
/// // Parse with defaults
/// let doc = parse_with_limits(input, opts)?;
/// ```
///
/// ## Direct field access
///
/// ```text
/// use hedl_core::{ParseOptions, Limits};
///
/// let mut opts = ParseOptions::default();
/// opts.reference_mode = false;
/// opts.limits.max_nodes = 5000;
/// ```
///
/// # Security Considerations
///
/// ParseOptions includes multiple security limits to prevent denial-of-service attacks:
///
/// - `max_file_size`: Prevents loading extremely large files
/// - `max_line_length`: Prevents regex DOS via extremely long lines
/// - `max_indent_depth`: Prevents stack overflow via deep nesting
/// - `max_nodes`: Prevents memory exhaustion via large matrix lists
/// - `max_object_keys` and `max_total_keys`: Prevent memory exhaustion via many objects
/// - `max_nest_depth`: Prevents stack overflow via deeply nested NEST hierarchies
/// - `max_block_string_size`: Prevents memory exhaustion via large block strings
///
/// # Fields
///
/// - `limits`: Security limits for parser resources
/// - `reference_mode`: Reference resolution mode (strict or lenient)
#[derive(Debug, Clone)]
pub struct ParseOptions {
    /// Security limits.
    pub limits: Limits,
    /// Reference resolution mode (strict or lenient).
    ///
    /// Controls how unresolved references are handled:
    /// - `ReferenceMode::Strict`: Errors on unresolved references (default)
    /// - `ReferenceMode::Lenient`: Ignores unresolved references
    ///
    /// Note: Ambiguous references always error regardless of mode.
    pub reference_mode: ReferenceMode,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            reference_mode: ReferenceMode::Strict,
        }
    }
}

impl ParseOptions {
    /// Create a new builder for ParseOptions.
    ///
    /// # Examples
    ///
    /// ```text
    /// let opts = ParseOptions::builder()
    ///     .max_depth(100)
    ///     .strict(true)
    ///     .build();
    /// ```
    pub fn builder() -> ParseOptionsBuilder {
        ParseOptionsBuilder::new()
    }
}

/// Builder for ergonomic construction of ParseOptions.
///
/// Provides a fluent API for configuring parser options with sensible defaults.
///
/// # Examples
///
/// ```text
/// // Using builder with custom limits
/// let opts = ParseOptions::builder()
///     .max_depth(200)
///     .max_array_length(5000)
///     .strict(false)
///     .build();
///
/// // Using builder with defaults
/// let opts = ParseOptions::builder().build();
/// ```
#[derive(Debug, Clone)]
pub struct ParseOptionsBuilder {
    limits: Limits,
    reference_mode: ReferenceMode,
}

impl ParseOptionsBuilder {
    /// Create a new builder with default options.
    pub fn new() -> Self {
        Self {
            limits: Limits::default(),
            reference_mode: ReferenceMode::Strict,
        }
    }

    /// Set the maximum nesting depth (indent depth).
    ///
    /// # Parameters
    ///
    /// - `depth`: Maximum nesting level (default: 50)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_depth(100)
    /// ```
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.limits.max_indent_depth = depth;
        self
    }

    /// Set the maximum array length (nodes in matrix lists).
    ///
    /// # Parameters
    ///
    /// - `length`: Maximum number of nodes (default: 10M)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_array_length(5000)
    /// ```
    pub fn max_array_length(mut self, length: usize) -> Self {
        self.limits.max_nodes = length;
        self
    }
    /// Set reference resolution mode.
    ///
    /// # Arguments
    /// - `mode`: The reference resolution mode to use
    ///
    /// # Examples
    ///
    /// ```text
    /// use hedl_core::{ParseOptionsBuilder, ReferenceMode};
    ///
    /// let opts = ParseOptionsBuilder::new()
    ///     .reference_mode(ReferenceMode::Lenient)
    ///     .build();
    /// ```
    pub fn reference_mode(mut self, mode: ReferenceMode) -> Self {
        self.reference_mode = mode;
        self
    }

    /// Enable strict reference resolution (error on unresolved).
    ///
    /// Shorthand for `.reference_mode(ReferenceMode::Strict)`.
    ///
    /// # Examples
    ///
    /// ```text
    /// let opts = ParseOptions::builder()
    ///     .strict_refs()
    ///     .build();
    /// ```
    pub fn strict_refs(mut self) -> Self {
        self.reference_mode = ReferenceMode::Strict;
        self
    }

    /// Enable lenient reference resolution (ignore unresolved).
    ///
    /// Shorthand for `.reference_mode(ReferenceMode::Lenient)`.
    ///
    /// # Examples
    ///
    /// ```text
    /// let opts = ParseOptions::builder()
    ///     .lenient_refs()
    ///     .build();
    /// ```
    pub fn lenient_refs(mut self) -> Self {
        self.reference_mode = ReferenceMode::Lenient;
        self
    }

    /// Set strict reference resolution mode (legacy compatibility).
    ///
    pub fn strict(mut self, strict: bool) -> Self {
        self.reference_mode = ReferenceMode::from(strict);
        self
    }

    /// Set the maximum file size in bytes.
    ///
    /// # Parameters
    ///
    /// - `size`: Maximum file size in bytes (default: 1GB)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_file_size(500 * 1024 * 1024)
    /// ```
    pub fn max_file_size(mut self, size: usize) -> Self {
        self.limits.max_file_size = size;
        self
    }

    /// Set the maximum line length in bytes.
    ///
    /// # Parameters
    ///
    /// - `length`: Maximum line length in bytes (default: 1MB)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_line_length(512 * 1024)
    /// ```
    pub fn max_line_length(mut self, length: usize) -> Self {
        self.limits.max_line_length = length;
        self
    }

    /// Set the maximum number of aliases.
    ///
    /// # Parameters
    ///
    /// - `count`: Maximum number of aliases (default: 10k)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_aliases(5000)
    /// ```
    pub fn max_aliases(mut self, count: usize) -> Self {
        self.limits.max_aliases = count;
        self
    }

    /// Set the maximum columns per schema.
    ///
    /// # Parameters
    ///
    /// - `count`: Maximum columns (default: 100)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_columns(50)
    /// ```
    pub fn max_columns(mut self, count: usize) -> Self {
        self.limits.max_columns = count;
        self
    }

    /// Set the maximum NEST hierarchy depth.
    ///
    /// # Parameters
    ///
    /// - `depth`: Maximum nesting depth (default: 100)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_nest_depth(50)
    /// ```
    pub fn max_nest_depth(mut self, depth: usize) -> Self {
        self.limits.max_nest_depth = depth;
        self
    }

    /// Set the maximum block string size in bytes.
    ///
    /// # Parameters
    ///
    /// - `size`: Maximum block string size (default: 10MB)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_block_string_size(5 * 1024 * 1024)
    /// ```
    pub fn max_block_string_size(mut self, size: usize) -> Self {
        self.limits.max_block_string_size = size;
        self
    }

    /// Set the maximum keys per object.
    ///
    /// # Parameters
    ///
    /// - `count`: Maximum keys per object (default: 10k)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_object_keys(5000)
    /// ```
    pub fn max_object_keys(mut self, count: usize) -> Self {
        self.limits.max_object_keys = count;
        self
    }

    /// Set the maximum total keys across all objects.
    ///
    /// This provides defense-in-depth against memory exhaustion attacks.
    ///
    /// # Parameters
    ///
    /// - `count`: Maximum total keys (default: 10M)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_total_keys(5_000_000)
    /// ```
    pub fn max_total_keys(mut self, count: usize) -> Self {
        self.limits.max_total_keys = count;
        self
    }

    /// Build the ParseOptions.
    pub fn build(self) -> ParseOptions {
        ParseOptions {
            limits: self.limits,
            reference_mode: self.reference_mode,
        }
    }
}

impl Default for ParseOptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

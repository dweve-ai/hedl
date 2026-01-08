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

//! HEDL Canonicalization
//!
//! Provides deterministic output generation for HEDL documents.
//! Canonical output ensures stable hashing, diffing, and round-trips.
//!
//! # Overview
//!
//! This crate implements the canonical serialization format for HEDL documents,
//! as specified in SPEC.md Section 13.2. Canonicalization ensures:
//!
//! - **Deterministic output**: Same document always produces same output
//! - **Idempotency**: `canonicalize(canonicalize(x)) == canonicalize(x)`
//! - **Round-trip preservation**: Parsing canonical output preserves semantics
//! - **Stable hashing**: Enables content-addressable storage and diffing
//!
//! # Features
//!
//! - Minimal or always-quote string formatting strategies
//! - Ditto optimization for repeated values in matrix lists
//! - Proper escaping of quotes and control characters
//! - Alphabetically sorted keys, aliases, and struct declarations
//! - Count hints in STRUCT directives for performance optimization
//! - Security: Recursion depth limits prevent stack overflow DoS attacks
//!
//! # Examples
//!
//! ```no_run
//! use hedl_c14n::{canonicalize, CanonicalConfig, CanonicalConfigBuilder, QuotingStrategy};
//! use hedl_core::Document;
//!
//! # fn example(doc: Document) -> Result<(), hedl_core::HedlError> {
//! // Simple canonicalization with defaults
//! let output = canonicalize(&doc)?;
//!
//! // Custom configuration using fluent API
//! let config = CanonicalConfig::new()
//!     .with_quoting(QuotingStrategy::Always)
//!     .with_ditto(false);
//! let output = hedl_c14n::canonicalize_with_config(&doc, &config)?;
//!
//! // Custom configuration using builder pattern
//! let config = CanonicalConfig::builder()
//!     .quoting(QuotingStrategy::Always)
//!     .use_ditto(false)
//!     .sort_keys(true)
//!     .build();
//! let output = hedl_c14n::canonicalize_with_config(&doc, &config)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Security
//!
//! This crate implements protection against denial-of-service attacks:
//!
//! - **Recursion depth limit**: Maximum nesting depth of 1000 levels prevents stack overflow
//! - **Proper escaping**: All special characters are escaped to prevent injection attacks
//! - **Type safety**: Rust's type system prevents memory safety issues
//!
//! # Performance
//!
//! Several optimizations are implemented:
//!
//! - **P0**: Direct BTreeMap iteration eliminates key cloning (1.15x speedup, 10-15% fewer allocations)
//! - **P1**: Pre-allocated output buffer (1.2-1.3x speedup)
//! - **P1**: Cell buffer reuse across rows (1.05-1.1x speedup for large matrices)

mod config;
mod ditto;
mod writer;

pub use config::{CanonicalConfig, CanonicalConfigBuilder, QuotingStrategy};
pub use ditto::can_use_ditto;
pub use writer::CanonicalWriter;

use hedl_core::{Document, HedlError};

/// Canonicalize a HEDL document to a string.
///
/// Uses default configuration with minimal quoting, ditto optimization enabled,
/// and STRUCT directives in header (per SPEC.md Section 13.2).
///
/// # Arguments
///
/// * `doc` - The HEDL document to canonicalize
///
/// # Returns
///
/// Canonical string representation of the document, or an error if writing fails.
///
/// # Errors
///
/// Returns `HedlError::Syntax` if:
/// - Writing to output buffer fails (extremely rare)
/// - Document nesting exceeds maximum depth of 1000 levels
///
/// # Examples
///
/// ```no_run
/// use hedl_c14n::canonicalize;
/// use hedl_core::Document;
///
/// # fn example(doc: Document) -> Result<(), hedl_core::HedlError> {
/// let canonical_output = canonicalize(&doc)?;
/// println!("{}", canonical_output);
/// # Ok(())
/// # }
/// ```
///
/// # Security
///
/// - Protected against stack overflow via recursion depth limit
/// - All special characters properly escaped
/// - No unsafe code
pub fn canonicalize(doc: &Document) -> Result<String, HedlError> {
    canonicalize_with_config(doc, &CanonicalConfig::default())
}

/// Canonicalize a HEDL document with custom configuration.
///
/// Allows fine-grained control over output format, including quoting strategy,
/// ditto optimization, and schema placement.
///
/// # Arguments
///
/// * `doc` - The HEDL document to canonicalize
/// * `config` - Configuration controlling output format
///
/// # Returns
///
/// Canonical string representation according to configuration, or an error if writing fails.
///
/// # Errors
///
/// Returns `HedlError::Syntax` if:
/// - Writing to output buffer fails
/// - Document nesting exceeds maximum depth of 1000 levels
///
/// # Examples
///
/// ```no_run
/// use hedl_c14n::{canonicalize_with_config, CanonicalConfig, QuotingStrategy};
/// use hedl_core::Document;
///
/// # fn example(doc: Document) -> Result<(), hedl_core::HedlError> {
/// let config = CanonicalConfig::new()
///     .with_quoting(QuotingStrategy::Always)
///     .with_ditto(false)
///     .with_sort_keys(true)
///     .with_inline_schemas(true);
/// let output = canonicalize_with_config(&doc, &config)?;
/// # Ok(())
/// # }
/// ```
///
/// # Performance
///
/// Pre-allocates 4KB output buffer to minimize reallocations for typical documents.
pub fn canonicalize_with_config(
    doc: &Document,
    config: &CanonicalConfig,
) -> Result<String, HedlError> {
    let mut writer = CanonicalWriter::new(config.clone());
    writer.write_document(doc)
}

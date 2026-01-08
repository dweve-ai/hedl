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

//! HEDL to TOON Conversion
//!
//! Converts HEDL documents to TOON (Token-Oriented Object Notation) format.
//! TOON is a compact, line-oriented format optimized for LLM consumption.
//!
//! # Overview
//!
//! This crate provides high-quality conversion from HEDL documents to TOON format,
//! implementing the full TOON v3.0 specification. TOON is designed for efficient
//! processing by Large Language Models while maintaining human readability.
//!
//! # Features
//!
//! - **Specification Compliance**: Full TOON v3.0 specification adherence
//! - **Reference Preservation**: Maintains HEDL references as `@Type:id` strings
//! - **Format Optimization**: Intelligent selection between tabular and expanded formats
//! - **Security**: Depth limit protection against stack overflow attacks
//! - **Type Safety**: Proper error types with detailed error messages
//!
//! # Quick Start
//!
//! ```rust
//! use hedl_toon::hedl_to_toon;
//! use hedl_core::Document;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let hedl = r#"%VERSION: 1.0
//! %STRUCT: User: [id, name]
//! ---
//! users: @User
//!   | u1, Alice
//!   | u2, Bob
//! "#;
//!
//! let doc = hedl_core::parse(hedl.as_bytes())?;
//! let toon = hedl_to_toon(&doc)?;
//! println!("{}", toon);
//! # Ok(())
//! # }
//! ```
//!
//! # Configuration
//!
//! For advanced use cases, customize the output format:
//!
//! ```rust
//! use hedl_toon::{to_toon, ToToonConfig, Delimiter};
//!
//! let config = ToToonConfig {
//!     indent: 4,
//!     delimiter: Delimiter::Tab,
//! };
//!
//! // Use config for conversion
//! // let toon = to_toon(&doc, &config)?;
//! ```
//!
//! # Security
//!
//! This crate includes protection against stack overflow attacks by limiting
//! nesting depth to 100 levels. Documents exceeding this limit will return
//! a [`ToonError::MaxDepthExceeded`] error.
//!
//! # TOON Format
//!
//! TOON supports two array representations:
//!
//! **Tabular Format** (for primitive values only):
//! ```toon
//! users[2]{id,name}:
//!   u1,Alice
//!   u2,Bob
//! ```
//!
//! **Expanded Format** (for complex/nested structures):
//! ```toon
//! orders[1]:
//!   - id: ord1
//!     customer: @User:u1
//!     items[2]{product,quantity}:
//!       prod1,5
//!       prod2,3
//! ```
//!
//! TOON Spec: <https://github.com/toon-format/spec>

mod error;
mod to_toon;

pub use error::{ToonError, Result, MAX_NESTING_DEPTH};
pub use to_toon::{to_toon, Delimiter, ToToonConfig, ToToonConfigBuilder};

use hedl_core::Document;

/// Convert HEDL document to TOON string with default configuration
///
/// This is a convenience function that uses the default TOON configuration:
/// - 2-space indentation
/// - Comma delimiter
///
/// # Arguments
///
/// * `doc` - The HEDL document to convert
///
/// # Returns
///
/// A TOON-formatted string, or a [`ToonError`] if conversion fails.
///
/// # Errors
///
/// Returns [`ToonError::MaxDepthExceeded`] if the document nesting exceeds
/// the maximum allowed depth of [`MAX_NESTING_DEPTH`] (100 levels).
///
/// # Examples
///
/// ```rust
/// use hedl_toon::hedl_to_toon;
/// use hedl_core::Document;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let hedl = r#"%VERSION: 1.0
/// ---
/// name: MyApp
/// version: 1.0
/// "#;
///
/// let doc = hedl_core::parse(hedl.as_bytes())?;
/// let toon = hedl_to_toon(&doc)?;
/// assert!(toon.contains("name: MyApp"));
/// # Ok(())
/// # }
/// ```
///
/// # Performance
///
/// - Time Complexity: O(n) where n is the total number of nodes
/// - Space Complexity: O(n) for the output string
///
/// # Thread Safety
///
/// This function is thread-safe and can be called concurrently with different
/// documents. It takes an immutable borrow of the document.
pub fn hedl_to_toon(doc: &Document) -> Result<String> {
    to_toon(doc, &ToToonConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_object() {
        let hedl = r#"%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
  | u2, Bob, bob@example.com
"#;
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
        let toon = hedl_to_toon(&doc).unwrap();

        // Field order follows schema definition: id, name, email
        assert!(toon.contains("users[2]{id,name,email}:"));
        assert!(toon.contains("u1,Alice,alice@example.com"));
        assert!(toon.contains("u2,Bob,bob@example.com"));
    }

    #[test]
    fn test_nested_object() {
        let hedl = r#"%VERSION: 1.0
---
config:
  name: MyApp
  version: 1.0
  settings:
    debug: true
    timeout: 30
"#;
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
        let toon = hedl_to_toon(&doc).unwrap();

        assert!(toon.contains("config:"));
        assert!(toon.contains("name: MyApp"));
        assert!(toon.contains("settings:"));
        assert!(toon.contains("debug: true"));
    }

    #[test]
    fn test_quoting() {
        let hedl = r#"%VERSION: 1.0
---
data:
  message: "Hello, world"
  empty: ""
  colon: "has:colon"
"#;
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
        let toon = hedl_to_toon(&doc).unwrap();

        // "Hello, world" needs quoting due to comma
        assert!(toon.contains("\"Hello, world\"") || toon.contains("message: \"Hello, world\""));
        // Empty string must be quoted
        assert!(toon.contains("\"\""));
        // Colon requires quoting
        assert!(toon.contains("\"has:colon\""));
    }
}

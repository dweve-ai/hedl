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

//! HEDL ↔ TOON Conversion
//!
//! Bidirectional conversion between HEDL documents and TOON (Token-Oriented Object Notation) format.
//! Uses the official `toon-format` crate for spec-compliant parsing and serialization.
//!
//! # Overview
//!
//! This crate provides conversion between HEDL and TOON using the official TOON parser.
//! TOON is designed for efficient processing by Large Language Models while maintaining
//! human readability.
//!
//! # Quick Start
//!
//! ```rust
//! use hedl_toon::hedl_to_toon;
//! use hedl_core::Document;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let hedl = r#"%V:2.0
//! %NULL:~
//! %QUOTE:"
//! %S:User:[id, name]
//! ---
//! users:@User
//!  |u1, Alice
//!  |u2, Bob
//! "#;
//!
//! let doc = hedl_core::parse(hedl.as_bytes())?;
//! let toon = hedl_to_toon(&doc)?;
//! println!("{}", toon);
//! # Ok(())
//! # }
//! ```
//!
//! TOON Spec: <https://github.com/toon-format/spec>

#![cfg_attr(not(test), warn(missing_docs))]

mod encoder;
mod error;
mod from_toon;
mod to_toon;

pub use error::{Result, ToonError, MAX_NESTING_DEPTH};
pub use from_toon::{from_toon, from_toon_with_config, FromToonConfig};
pub use to_toon::{to_toon, Delimiter, ToToonConfig, ToToonConfigBuilder};

use hedl_core::Document;

/// Convert HEDL document to TOON string with default configuration
///
/// Uses the official toon-format crate for spec-compliant output.
///
/// # Arguments
///
/// * `doc` - The HEDL document to convert
///
/// # Returns
///
/// A TOON-formatted string, or a [`ToonError`] if conversion fails.
pub fn hedl_to_toon(doc: &Document) -> Result<String> {
    to_toon(doc, &ToToonConfig::default())
}

/// Parse TOON string to HEDL document
///
/// Uses the official toon-format crate for spec-compliant parsing.
///
/// # Arguments
///
/// * `toon` - The TOON formatted string to parse
///
/// # Returns
///
/// A HEDL Document, or a [`ToonError`] if parsing fails.
pub fn toon_to_hedl(toon: &str) -> Result<Document> {
    from_toon(toon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_roundtrip() {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
---
users:@User
 |u1, Alice, alice@example.com
 |u2, Bob, bob@example.com
"#;
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
        let toon = hedl_to_toon(&doc).unwrap();

        // Parse TOON back
        let doc2 = toon_to_hedl(&toon).unwrap();

        // Verify structure preserved
        assert!(doc2.root.contains_key("users"));
    }

    #[test]
    fn test_nested_object() {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
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
    }

    #[test]
    fn test_arrays() {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
numbers: [1, 2, 3, 4, 5]
tags: (rust, python, go)
"#;
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
        let toon = hedl_to_toon(&doc).unwrap();

        // Parse back and verify
        let doc2 = toon_to_hedl(&toon).unwrap();
        assert!(doc2.root.contains_key("numbers"));
        assert!(doc2.root.contains_key("tags"));
    }
}

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

//! Token validation and parsing for HEDL.
//!
//! This module provides utilities for validating and parsing HEDL tokens:
//! - Key tokens (lowercase snake_case identifiers)
//! - Type names (PascalCase identifiers)
//! - ID tokens (alphanumeric with hyphens)
//! - Reference tokens (@Type:id or @id format)

use crate::lex::error::{LexError, SourcePos};

/// A parsed reference token.
///
/// References in HEDL can be either local (just an ID) or qualified (Type:id).
///
/// # Examples
///
/// ```
/// use hedl_core::lex::Reference;
///
/// // Local reference
/// let local = Reference::local("user_1");
/// assert_eq!(local.id, "user_1");
/// assert!(local.type_name.is_none());
///
/// // Qualified reference
/// let qualified = Reference::qualified("User", "user_1");
/// assert_eq!(qualified.type_name, Some("User".to_string()));
/// assert_eq!(qualified.id, "user_1");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// Optional type qualifier (e.g., "User" in "@User:id").
    pub type_name: Option<String>,
    /// The ID being referenced.
    pub id: String,
}

impl Reference {
    /// Creates a local reference (no type qualifier).
    ///
    /// # Arguments
    ///
    /// * `id` - The ID to reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::Reference;
    ///
    /// let r = Reference::local("user_1");
    /// assert_eq!(r.id, "user_1");
    /// assert!(r.type_name.is_none());
    /// ```
    #[inline]
    pub fn local(id: impl Into<String>) -> Self {
        Self {
            type_name: None,
            id: id.into(),
        }
    }

    /// Creates a qualified reference with a type qualifier.
    ///
    /// # Arguments
    ///
    /// * `type_name` - The type name qualifier (e.g., "User").
    /// * `id` - The ID being referenced.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::Reference;
    ///
    /// let r = Reference::qualified("User", "user_1");
    /// assert_eq!(r.type_name, Some("User".to_string()));
    /// assert_eq!(r.id, "user_1");
    /// ```
    #[inline]
    pub fn qualified(type_name: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            type_name: Some(type_name.into()),
            id: id.into(),
        }
    }

    /// Returns `true` if this is a qualified reference (has a type name).
    #[inline]
    pub fn is_qualified(&self) -> bool {
        self.type_name.is_some()
    }

    /// Returns `true` if this is a local reference (no type name).
    #[inline]
    pub fn is_local(&self) -> bool {
        self.type_name.is_none()
    }
}

impl std::fmt::Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.type_name {
            Some(type_name) => write!(f, "@{}:{}", type_name, self.id),
            None => write!(f, "@{}", self.id),
        }
    }
}

/// Checks if a string is a valid Key Token: `[a-z_][a-z0-9_]*`
///
/// Key tokens are lowercase snake_case identifiers used for field names.
///
/// # Examples
///
/// ```
/// use hedl_core::lex::is_valid_key_token;
///
/// assert!(is_valid_key_token("name"));
/// assert!(is_valid_key_token("user_id"));
/// assert!(is_valid_key_token("_private"));
/// assert!(is_valid_key_token("item123"));
///
/// assert!(!is_valid_key_token("Name"));     // No uppercase
/// assert!(!is_valid_key_token("123item"));  // No leading digit
/// assert!(!is_valid_key_token("my-key"));   // No hyphens
/// ```
#[inline]
pub fn is_valid_key_token(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let first = bytes[0];
    if !first.is_ascii_lowercase() && first != b'_' {
        return false;
    }
    bytes[1..]
        .iter()
        .all(|&b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// Checks if a string is a valid TypeName Token: `[A-Z][A-Za-z0-9]*`
///
/// Type names are PascalCase identifiers used for entity types.
///
/// # Examples
///
/// ```
/// use hedl_core::lex::is_valid_type_name;
///
/// assert!(is_valid_type_name("User"));
/// assert!(is_valid_type_name("Post123"));
/// assert!(is_valid_type_name("MyType"));
///
/// assert!(!is_valid_type_name("user"));      // Must start uppercase
/// assert!(!is_valid_type_name("User_Type")); // No underscores
/// assert!(!is_valid_type_name("123User"));   // No leading digit
/// ```
#[inline]
pub fn is_valid_type_name(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    if !bytes[0].is_ascii_uppercase() {
        return false;
    }
    bytes[1..].iter().all(|&b| b.is_ascii_alphanumeric())
}

/// Checks if a string is a valid ID Token: `[a-zA-Z_][a-zA-Z0-9_\-]*`
///
/// IDs can start with any letter (upper or lower) or underscore, followed by
/// letters, digits, underscores, or hyphens. This allows real-world IDs like
/// "SKU-4020", "User123", "ABC-DEF-001", etc.
///
/// # Examples
///
/// ```
/// use hedl_core::lex::is_valid_id_token;
///
/// assert!(is_valid_id_token("user_1"));
/// assert!(is_valid_id_token("item-two"));
/// assert!(is_valid_id_token("SKU-4020"));
/// assert!(is_valid_id_token("ABC-DEF-001"));
///
/// assert!(!is_valid_id_token("123item"));  // No leading digit
/// assert!(!is_valid_id_token("-item"));    // No leading hyphen
/// assert!(!is_valid_id_token("id.name"));  // No dots
/// ```
#[inline]
pub fn is_valid_id_token(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let first = bytes[0];
    if !first.is_ascii_alphabetic() && first != b'_' {
        return false;
    }
    bytes[1..]
        .iter()
        .all(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// Parses a reference token (with or without leading `@`).
///
/// Accepts formats:
/// - `@id` or `id` -> Reference { type_name: None, id }
/// - `@Type:id` or `Type:id` -> Reference { type_name: Some(Type), id }
///
/// # Examples
///
/// ```
/// use hedl_core::lex::parse_reference;
///
/// // Local references
/// let r = parse_reference("@user_1").unwrap();
/// assert!(r.type_name.is_none());
/// assert_eq!(r.id, "user_1");
///
/// // Qualified references
/// let r = parse_reference("@User:user_1").unwrap();
/// assert_eq!(r.type_name, Some("User".to_string()));
/// assert_eq!(r.id, "user_1");
///
/// // Without @ prefix
/// let r = parse_reference("Post:post-123").unwrap();
/// assert_eq!(r.type_name, Some("Post".to_string()));
/// assert_eq!(r.id, "post-123");
/// ```
///
/// # Errors
///
/// Returns `LexError::InvalidReference` if the reference format is invalid.
pub fn parse_reference(s: &str) -> Result<Reference, LexError> {
    parse_reference_at(s, SourcePos::default())
}

/// Parses a reference token with position information for error reporting.
///
/// Same as `parse_reference`, but allows specifying the source position for
/// error messages.
pub fn parse_reference_at(s: &str, pos: SourcePos) -> Result<Reference, LexError> {
    let s = s.strip_prefix('@').unwrap_or(s);

    if let Some((type_part, id_part)) = s.split_once(':') {
        // Qualified reference
        if !is_valid_type_name(type_part) {
            return Err(LexError::InvalidReference {
                message: format!("invalid type name: {}", type_part),
                pos,
            });
        }
        if !is_valid_id_token(id_part) {
            return Err(LexError::InvalidReference {
                message: format!("invalid ID: {}", id_part),
                pos,
            });
        }
        Ok(Reference {
            type_name: Some(type_part.to_string()),
            id: id_part.to_string(),
        })
    } else {
        // Local reference
        if !is_valid_id_token(s) {
            return Err(LexError::InvalidReference {
                message: format!("invalid ID: {}", s),
                pos,
            });
        }
        Ok(Reference {
            type_name: None,
            id: s.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== is_valid_key_token tests ====================

    #[test]
    fn test_key_token_valid_basic() {
        assert!(is_valid_key_token("name"));
        assert!(is_valid_key_token("user_id"));
        assert!(is_valid_key_token("_private"));
        assert!(is_valid_key_token("item123"));
        assert!(is_valid_key_token("a"));
        assert!(is_valid_key_token("_"));
        assert!(is_valid_key_token("abc_def_ghi"));
        assert!(is_valid_key_token("a1b2c3"));
    }

    #[test]
    fn test_key_token_valid_edge_cases() {
        assert!(is_valid_key_token("a"));
        assert!(is_valid_key_token("z"));
        assert!(is_valid_key_token("_"));
        assert!(is_valid_key_token("_a"));
        assert!(is_valid_key_token("_1"));
        assert!(is_valid_key_token("__"));
        assert!(is_valid_key_token("a_very_long_key_name_with_numbers_123"));
        assert!(is_valid_key_token("abcdefghijklmnopqrstuvwxyz"));
        assert!(is_valid_key_token("a0123456789"));
    }

    #[test]
    fn test_key_token_invalid() {
        assert!(!is_valid_key_token(""));
        assert!(!is_valid_key_token("Name"));
        assert!(!is_valid_key_token("NAME"));
        assert!(!is_valid_key_token("123item"));
        assert!(!is_valid_key_token("my-key"));
        assert!(!is_valid_key_token("my.key"));
        assert!(!is_valid_key_token("my key"));
    }

    // ==================== is_valid_type_name tests ====================

    #[test]
    fn test_type_name_valid() {
        assert!(is_valid_type_name("User"));
        assert!(is_valid_type_name("Post123"));
        assert!(is_valid_type_name("A"));
        assert!(is_valid_type_name("MyType"));
        assert!(is_valid_type_name("ABCDef"));
        assert!(is_valid_type_name("ABCDEFGHIJKLMNOPQRSTUVWXYZ"));
    }

    #[test]
    fn test_type_name_invalid() {
        assert!(!is_valid_type_name(""));
        assert!(!is_valid_type_name("user"));
        assert!(!is_valid_type_name("myType"));
        assert!(!is_valid_type_name("User_Type"));
        assert!(!is_valid_type_name("_User"));
        assert!(!is_valid_type_name("123User"));
    }

    // ==================== is_valid_id_token tests ====================

    #[test]
    fn test_id_token_valid() {
        assert!(is_valid_id_token("user_1"));
        assert!(is_valid_id_token("item-two"));
        assert!(is_valid_id_token("_system"));
        assert!(is_valid_id_token("a"));
        assert!(is_valid_id_token("_"));
        assert!(is_valid_id_token("my-id"));
        assert!(is_valid_id_token("User1"));
        assert!(is_valid_id_token("SKU-4020"));
        assert!(is_valid_id_token("ABC-DEF-001"));
    }

    #[test]
    fn test_id_token_invalid() {
        assert!(!is_valid_id_token(""));
        assert!(!is_valid_id_token("123"));
        assert!(!is_valid_id_token("-id"));
        assert!(!is_valid_id_token("id.name"));
        assert!(!is_valid_id_token("id name"));
    }

    // ==================== parse_reference tests ====================

    #[test]
    fn test_parse_reference_local() {
        let r = parse_reference("@user_1").unwrap();
        assert_eq!(r.type_name, None);
        assert_eq!(r.id, "user_1");

        let r = parse_reference("user_1").unwrap();
        assert_eq!(r.type_name, None);
        assert_eq!(r.id, "user_1");
    }

    #[test]
    fn test_parse_reference_qualified() {
        let r = parse_reference("@User:user_1").unwrap();
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "user_1");

        let r = parse_reference("Post:post-123").unwrap();
        assert_eq!(r.type_name, Some("Post".to_string()));
        assert_eq!(r.id, "post-123");
    }

    #[test]
    fn test_parse_reference_invalid() {
        assert!(parse_reference("@").is_err());
        assert!(parse_reference("").is_err());
        assert!(parse_reference("@:id").is_err());
        assert!(parse_reference("@Type:").is_err());
        assert!(parse_reference("@user:id").is_err()); // Lowercase type
        assert!(parse_reference("@123User").is_err()); // Digit-start ID
    }

    // ==================== Reference struct tests ====================

    #[test]
    fn test_reference_constructors() {
        let r = Reference::local("my_id");
        assert!(r.is_local());
        assert!(!r.is_qualified());

        let r = Reference::qualified("User", "user_1");
        assert!(!r.is_local());
        assert!(r.is_qualified());
    }

    #[test]
    fn test_reference_display() {
        let r = Reference::local("user_1");
        assert_eq!(format!("{}", r), "@user_1");

        let r = Reference::qualified("User", "user_1");
        assert_eq!(format!("{}", r), "@User:user_1");
    }

    #[test]
    fn test_reference_equality() {
        let r1 = Reference::local("id");
        let r2 = Reference::local("id");
        assert_eq!(r1, r2);

        let r3 = Reference::qualified("Type", "id");
        assert_ne!(r1, r3);
    }

    #[test]
    fn test_reference_clone() {
        let r1 = Reference::qualified("User", "user_1");
        let r2 = r1.clone();
        assert_eq!(r1, r2);
    }
}

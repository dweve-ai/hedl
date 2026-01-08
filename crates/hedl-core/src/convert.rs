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

//! Common conversion utilities and traits for HEDL format converters.
//!
//! This module provides shared functionality used by all HEDL format converters
//! (JSON, YAML, XML, CSV, etc.) to ensure consistent behavior and reduce code
//! duplication.

use crate::{Document, Reference};

/// Default schema used when no schema can be inferred from data.
/// This is the standard fallback for all format converters.
pub const DEFAULT_SCHEMA: &[&str] = &["id", "value"];

/// Configuration trait for import operations.
///
/// All format-specific import configurations should include these common fields.
pub trait ImportConfig {
    /// Get the default type name for collections without explicit metadata.
    fn default_type_name(&self) -> &str;

    /// Get the HEDL version to use for the imported document.
    fn version(&self) -> (u32, u32);
}

/// Configuration trait for export operations.
///
/// All format-specific export configurations should include these common fields.
pub trait ExportConfig {
    /// Whether to include version metadata in the output.
    fn include_metadata(&self) -> bool;

    /// Whether to use pretty-printed/indented output.
    fn pretty(&self) -> bool;
}

/// Trait for types that can import HEDL documents from a string format.
pub trait FromFormat: Sized {
    /// The configuration type for this format.
    type Config: ImportConfig;

    /// The error type returned on conversion failure.
    type Error;

    /// Convert from the format string to a HEDL document.
    fn from_format(input: &str, config: &Self::Config) -> Result<Document, Self::Error>;
}

/// Trait for types that can export HEDL documents to a string format.
pub trait ToFormat {
    /// The configuration type for this format.
    type Config: ExportConfig;

    /// The error type returned on conversion failure.
    type Error;

    /// Convert from a HEDL document to the format string.
    fn to_format(doc: &Document, config: &Self::Config) -> Result<String, Self::Error>;
}

/// Parse a reference string in the format "@id" or "@Type:id".
///
/// # Arguments
///
/// * `s` - The reference string to parse. Must start with '@'.
///
/// # Returns
///
/// * `Ok(Reference)` - The parsed reference.
/// * `Err(String)` - An error message if the format is invalid.
///
/// # Examples
///
/// ```
/// use hedl_core::convert::parse_reference;
///
/// let local = parse_reference("@user123").unwrap();
/// assert_eq!(local.type_name, None);
/// assert_eq!(local.id, "user123");
///
/// let qualified = parse_reference("@User:123").unwrap();
/// assert_eq!(qualified.type_name, Some("User".to_string()));
/// assert_eq!(qualified.id, "123");
/// ```
pub fn parse_reference(s: &str) -> Result<Reference, String> {
    if let Some(stripped) = s.strip_prefix('@') {
        if let Some((type_name, id)) = stripped.split_once(':') {
            Ok(Reference::qualified(type_name, id))
        } else {
            Ok(Reference::local(stripped))
        }
    } else {
        Err(format!("Invalid reference format: expected '@' prefix, got '{}'", s))
    }
}

/// Common base configuration for import operations.
///
/// Format-specific configurations can embed this to get common fields
/// with consistent defaults.
#[derive(Debug, Clone)]
pub struct BaseImportConfig {
    /// Default type name for arrays/lists without metadata.
    pub default_type_name: String,
    /// HEDL version to use for the imported document.
    pub version: (u32, u32),
}

impl Default for BaseImportConfig {
    fn default() -> Self {
        Self {
            default_type_name: "Item".to_string(),
            version: (1, 0),
        }
    }
}

impl ImportConfig for BaseImportConfig {
    fn default_type_name(&self) -> &str {
        &self.default_type_name
    }

    fn version(&self) -> (u32, u32) {
        self.version
    }
}

/// Common base configuration for export operations.
///
/// Format-specific configurations can embed this to get common fields
/// with consistent defaults.
#[derive(Debug, Clone)]
pub struct BaseExportConfig {
    /// Whether to include version metadata in output.
    pub include_metadata: bool,
    /// Whether to use pretty-printed/indented output.
    pub pretty: bool,
    /// Indentation string (when pretty is true).
    pub indent: String,
}

impl Default for BaseExportConfig {
    fn default() -> Self {
        Self {
            include_metadata: true,
            pretty: true,
            indent: "  ".to_string(),
        }
    }
}

impl ExportConfig for BaseExportConfig {
    fn include_metadata(&self) -> bool {
        self.include_metadata
    }

    fn pretty(&self) -> bool {
        self.pretty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== parse_reference tests ====================

    #[test]
    fn test_parse_reference_local() {
        let r = parse_reference("@user123").unwrap();
        assert_eq!(r.type_name, None);
        assert_eq!(r.id, "user123");
    }

    #[test]
    fn test_parse_reference_qualified() {
        let r = parse_reference("@User:123").unwrap();
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "123");
    }

    #[test]
    fn test_parse_reference_with_dashes() {
        let r = parse_reference("@my-item_123").unwrap();
        assert_eq!(r.type_name, None);
        assert_eq!(r.id, "my-item_123");
    }

    #[test]
    fn test_parse_reference_qualified_with_dashes() {
        let r = parse_reference("@My-Type:item-123").unwrap();
        assert_eq!(r.type_name, Some("My-Type".to_string()));
        assert_eq!(r.id, "item-123");
    }

    #[test]
    fn test_parse_reference_invalid_no_at() {
        let result = parse_reference("user123");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid reference format"));
    }

    #[test]
    fn test_parse_reference_empty_after_at() {
        let r = parse_reference("@").unwrap();
        assert_eq!(r.type_name, None);
        assert_eq!(r.id, "");
    }

    #[test]
    fn test_parse_reference_empty_type_and_id() {
        let r = parse_reference("@:").unwrap();
        assert_eq!(r.type_name, Some("".to_string()));
        assert_eq!(r.id, "");
    }

    // ==================== BaseImportConfig tests ====================

    #[test]
    fn test_base_import_config_default() {
        let config = BaseImportConfig::default();
        assert_eq!(config.default_type_name, "Item");
        assert_eq!(config.version, (1, 0));
    }

    #[test]
    fn test_base_import_config_trait() {
        let config = BaseImportConfig::default();
        assert_eq!(config.default_type_name(), "Item");
        assert_eq!(config.version(), (1, 0));
    }

    #[test]
    fn test_base_import_config_custom() {
        let config = BaseImportConfig {
            default_type_name: "Custom".to_string(),
            version: (2, 5),
        };
        assert_eq!(config.default_type_name(), "Custom");
        assert_eq!(config.version(), (2, 5));
    }

    #[test]
    fn test_base_import_config_clone() {
        let config = BaseImportConfig::default();
        let cloned = config.clone();
        assert_eq!(config.default_type_name, cloned.default_type_name);
        assert_eq!(config.version, cloned.version);
    }

    // ==================== BaseExportConfig tests ====================

    #[test]
    fn test_base_export_config_default() {
        let config = BaseExportConfig::default();
        assert!(config.include_metadata);
        assert!(config.pretty);
        assert_eq!(config.indent, "  ");
    }

    #[test]
    fn test_base_export_config_trait() {
        let config = BaseExportConfig::default();
        assert!(config.include_metadata());
        assert!(config.pretty());
    }

    #[test]
    fn test_base_export_config_custom() {
        let config = BaseExportConfig {
            include_metadata: false,
            pretty: false,
            indent: "\t".to_string(),
        };
        assert!(!config.include_metadata());
        assert!(!config.pretty());
        assert_eq!(config.indent, "\t");
    }

    #[test]
    fn test_base_export_config_clone() {
        let config = BaseExportConfig::default();
        let cloned = config.clone();
        assert_eq!(config.include_metadata, cloned.include_metadata);
        assert_eq!(config.pretty, cloned.pretty);
        assert_eq!(config.indent, cloned.indent);
    }

    // ==================== DEFAULT_SCHEMA tests ====================

    #[test]
    fn test_default_schema() {
        assert_eq!(DEFAULT_SCHEMA, &["id", "value"]);
        assert_eq!(DEFAULT_SCHEMA.len(), 2);
    }
}

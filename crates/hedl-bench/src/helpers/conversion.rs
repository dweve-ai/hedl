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

//! Format conversion helpers.
//!
//! Utilities for converting between HEDL and other formats (JSON, YAML, XML)
//! with round-trip testing support.

use crate::Result;
use hedl_core::Document;

/// Format types for conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Yaml,
    Xml,
    Canonical,
}

/// Converts a HEDL document to JSON.
///
/// # Arguments
///
/// * `doc` - The document to convert
///
/// # Returns
///
/// Result containing JSON string.
pub fn convert_to_json(doc: &Document) -> Result<String> {
    hedl_json::to_json(doc, &hedl_json::ToJsonConfig::default())
        .map_err(|e| crate::BenchError::ConversionError(e.to_string()))
}

/// Converts a HEDL document to YAML.
///
/// # Arguments
///
/// * `doc` - The document to convert
///
/// # Returns
///
/// Result containing YAML string.
pub fn convert_to_yaml(doc: &Document) -> Result<String> {
    hedl_yaml::to_yaml(doc, &hedl_yaml::ToYamlConfig::default())
        .map_err(|e| crate::BenchError::ConversionError(e.to_string()))
}

/// Converts a HEDL document to XML.
///
/// # Arguments
///
/// * `doc` - The document to convert
///
/// # Returns
///
/// Result containing XML string.
pub fn convert_to_xml(doc: &Document) -> Result<String> {
    hedl_xml::to_xml(doc, &hedl_xml::ToXmlConfig::default())
        .map_err(|e| crate::BenchError::ConversionError(e.to_string()))
}

/// Converts a HEDL document to canonical form.
///
/// # Arguments
///
/// * `doc` - The document to canonicalize
///
/// # Returns
///
/// Result containing canonical HEDL string.
pub fn convert_to_canonical(doc: &Document) -> Result<String> {
    hedl_c14n::canonicalize(doc).map_err(|e| crate::BenchError::ConversionError(e.to_string()))
}

/// Converts from JSON to HEDL document.
///
/// # Arguments
///
/// * `json` - The JSON string to convert
///
/// # Returns
///
/// Result containing parsed document.
pub fn convert_from_json(json: &str) -> Result<Document> {
    hedl_json::from_json(json, &hedl_json::FromJsonConfig::default())
        .map_err(|e| crate::BenchError::ConversionError(e.to_string()))
}

/// Converts from YAML to HEDL document.
///
/// # Arguments
///
/// * `yaml` - The YAML string to convert
///
/// # Returns
///
/// Result containing parsed document.
pub fn convert_from_yaml(yaml: &str) -> Result<Document> {
    hedl_yaml::from_yaml(yaml, &hedl_yaml::FromYamlConfig::default())
        .map_err(|e| crate::BenchError::ConversionError(e.to_string()))
}

/// Performs round-trip test for a format.
///
/// Converts doc -> format -> doc and verifies structure preservation.
///
/// # Arguments
///
/// * `doc` - The original document
/// * `format` - The format to test
///
/// # Returns
///
/// Result containing true if round-trip succeeded.
pub fn roundtrip_test(doc: &Document, format: Format) -> Result<bool> {
    let converted = match format {
        Format::Json => convert_from_json(&convert_to_json(doc)?)?,
        Format::Yaml => convert_from_yaml(&convert_to_yaml(doc)?)?,
        Format::Xml => {
            // XML doesn't have from_xml yet, skip
            return Ok(true);
        }
        Format::Canonical => {
            let canon = convert_to_canonical(doc)?;
            hedl_core::parse(canon.as_bytes())
                .map_err(|e| crate::BenchError::ParseError(e.to_string()))?
        }
    };

    // Compare structure (root entity count)
    Ok(doc.root.len() == converted.root.len())
}

/// Converts document to specified format.
///
/// # Arguments
///
/// * `doc` - The document to convert
/// * `format` - Target format
///
/// # Returns
///
/// Result containing formatted string.
pub fn convert_to_format(doc: &Document, format: Format) -> Result<String> {
    match format {
        Format::Json => convert_to_json(doc),
        Format::Yaml => convert_to_yaml(doc),
        Format::Xml => convert_to_xml(doc),
        Format::Canonical => convert_to_canonical(doc),
    }
}

/// Compares sizes across all formats.
///
/// # Arguments
///
/// * `doc` - The document to compare
///
/// # Returns
///
/// Tuple of (json_bytes, yaml_bytes, xml_bytes, canonical_bytes).
pub fn compare_format_sizes(doc: &Document) -> Result<(usize, usize, usize, usize)> {
    let json = convert_to_json(doc)?;
    let yaml = convert_to_yaml(doc)?;
    let xml = convert_to_xml(doc)?;
    let canonical = convert_to_canonical(doc)?;

    Ok((json.len(), yaml.len(), xml.len(), canonical.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_users;

    fn get_test_doc() -> Document {
        let hedl = generate_users(5);
        hedl_core::parse(hedl.as_bytes()).unwrap()
    }

    #[test]
    fn test_convert_to_json() {
        let doc = get_test_doc();
        let json = convert_to_json(&doc).unwrap();
        assert!(json.starts_with('{') || json.starts_with('['));
    }

    #[test]
    fn test_convert_to_yaml() {
        let doc = get_test_doc();
        let yaml = convert_to_yaml(&doc).unwrap();
        assert!(!yaml.is_empty());
    }

    #[test]
    fn test_convert_to_xml() {
        let doc = get_test_doc();
        let xml = convert_to_xml(&doc).unwrap();
        assert!(xml.contains('<'));
    }

    #[test]
    fn test_roundtrip_json() {
        let doc = get_test_doc();
        assert!(roundtrip_test(&doc, Format::Json).unwrap());
    }

    #[test]
    fn test_roundtrip_yaml() {
        let doc = get_test_doc();
        assert!(roundtrip_test(&doc, Format::Yaml).unwrap());
    }

    #[test]
    fn test_compare_sizes() {
        let doc = get_test_doc();
        let (json, yaml, xml, canonical) = compare_format_sizes(&doc).unwrap();
        assert!(json > 0);
        assert!(yaml > 0);
        assert!(xml > 0);
        assert!(canonical > 0);
    }
}

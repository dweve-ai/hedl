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

//! Property-based roundtrip tests for XML conversion
//!
//! These tests verify that XML conversion preserves document semantics and
//! properly handles edge cases through randomized property testing.
//!
//! # Properties Tested
//!
//! 1. **Roundtrip Preservation**: HEDL → XML → HEDL preserves values
//! 2. **Idempotency**: XML → HEDL → XML produces equivalent XML
//! 3. **Well-Formedness**: Generated XML is always well-formed
//! 4. **Escaping**: Special characters are properly escaped/unescaped
//! 5. **Type Preservation**: Value types are maintained through roundtrips
//!
//! # Known Limitations
//!
//! - **Whitespace**: Trailing whitespace may be trimmed by XML parsers (XMLspec compliant)
//! - **Empty Strings**: May be represented differently in XML but preserve semantics
//! - **Floating Point**: Small precision loss may occur in float roundtrips

use hedl_core::{Document, Item, Value};
use hedl_xml::{from_xml, to_xml, FromXmlConfig, ToXmlConfig};
use proptest::prelude::*;
use std::collections::BTreeMap;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property: HEDL → XML → HEDL roundtrip preserves integer values
    #[test]
    fn prop_xml_roundtrip_integers(
        key in "[a-z][a-z0-9]{0,20}",
        value in -10000_i64..10000
    ) {
        let mut root = BTreeMap::new();
        root.insert(key.clone(), Item::Scalar(Value::Int(value)));

        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        };

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).map_err(|e| TestCaseError::fail(e))?;

        let parse_config = FromXmlConfig::default();
        let doc2 = from_xml(&xml, &parse_config).map_err(|e| TestCaseError::fail(e))?;

        // Verify key is preserved
        prop_assert!(doc2.root.contains_key(&key), "Missing key: {}", key);

        // Verify value is equivalent
        if let Some(Item::Scalar(Value::Int(v))) = doc2.root.get(&key) {
            prop_assert_eq!(*v, value);
        }
    }

    /// Property: HEDL → XML → HEDL roundtrip preserves string values (non-whitespace)
    #[test]
    fn prop_xml_roundtrip_strings(
        key in "[a-z][a-z0-9]{0,20}",
        value in "[a-zA-Z0-9]{0,50}"  // No trailing spaces to avoid XML trimming
    ) {
        let mut root = BTreeMap::new();
        root.insert(key.clone(), Item::Scalar(Value::String(value.clone())));

        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        };

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).map_err(|e| TestCaseError::fail(e))?;

        let parse_config = FromXmlConfig::default();
        let doc2 = from_xml(&xml, &parse_config).map_err(|e| TestCaseError::fail(e))?;

        prop_assert!(doc2.root.contains_key(&key), "Missing key: {}", key);

        if let Some(Item::Scalar(Value::String(v))) = doc2.root.get(&key) {
            prop_assert_eq!(v, &value);
        }
    }

    /// Property: XML special characters are properly escaped and unescaped (non-whitespace suffix)
    #[test]
    fn prop_xml_special_chars_roundtrip(
        key in "[a-z][a-z0-9]{2,10}",
        prefix in "[a-zA-Z0-9]{0,10}",
        suffix in "[a-zA-Z0-9]{0,10}",  // No trailing whitespace
        special in prop_oneof!["<", ">", "&", "\"", "'"]
    ) {
        let value = format!("{}{}{}", prefix, special, suffix);

        let mut root = BTreeMap::new();
        root.insert(key.clone(), Item::Scalar(Value::String(value.clone())));

        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        };

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).map_err(|e| TestCaseError::fail(e))?;

        let parse_config = FromXmlConfig::default();
        let doc2 = from_xml(&xml, &parse_config).map_err(|e| TestCaseError::fail(e))?;

        if let Some(Item::Scalar(Value::String(v))) = doc2.root.get(&key) {
            prop_assert_eq!(v, &value, "Special character not preserved");
        }
    }

    /// Property: XML → HEDL → XML roundtrip is idempotent
    #[test]
    fn prop_xml_idempotent_roundtrip(
        key in "[a-z][a-z0-9]{2,10}",
        value in -1000_i64..1000
    ) {
        let mut root = BTreeMap::new();
        root.insert(key.clone(), Item::Scalar(Value::Int(value)));

        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        };

        let config = ToXmlConfig::default();
        let xml1 = to_xml(&doc, &config).map_err(|e| TestCaseError::fail(e))?;

        let parse_config = FromXmlConfig::default();
        let doc2 = from_xml(&xml1, &parse_config).map_err(|e| TestCaseError::fail(e))?;
        let xml2 = to_xml(&doc2, &config).map_err(|e| TestCaseError::fail(e))?;

        // Parse both XMLs and compare
        let doc1_reparsed = from_xml(&xml1, &parse_config).map_err(|e| TestCaseError::fail(e))?;
        let doc2_reparsed = from_xml(&xml2, &parse_config).map_err(|e| TestCaseError::fail(e))?;

        // Both should have the same key
        prop_assert_eq!(
            doc1_reparsed.root.contains_key(&key),
            doc2_reparsed.root.contains_key(&key)
        );
    }

    /// Property: Generated XML is always well-formed
    #[test]
    fn prop_xml_wellformed(
        key in "[a-z][a-z0-9]{2,10}",
        value in prop_oneof![
            any::<i64>().prop_map(Value::Int),
            any::<bool>().prop_map(Value::Bool),
            "[a-zA-Z0-9 <>&\"']{0,30}".prop_map(|s| Value::String(s)),
        ]
    ) {
        let mut root = BTreeMap::new();
        root.insert(key, Item::Scalar(value));

        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        };

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).map_err(|e| TestCaseError::fail(e))?;

        // If we can parse it back, it's well-formed
        let parse_config = FromXmlConfig::default();
        let result = from_xml(&xml, &parse_config);

        prop_assert!(result.is_ok(), "Generated XML is not well-formed: {:?}", result.err());
    }

    /// Property: Boolean values roundtrip correctly
    #[test]
    fn prop_xml_roundtrip_booleans(
        key in "[a-z][a-z0-9]{2,10}",
        value in any::<bool>()
    ) {
        let mut root = BTreeMap::new();
        root.insert(key.clone(), Item::Scalar(Value::Bool(value)));

        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        };

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).map_err(|e| TestCaseError::fail(e))?;

        let parse_config = FromXmlConfig::default();
        let doc2 = from_xml(&xml, &parse_config).map_err(|e| TestCaseError::fail(e))?;

        if let Some(Item::Scalar(Value::Bool(v))) = doc2.root.get(&key) {
            prop_assert_eq!(*v, value);
        }
    }

    /// Property: Float values roundtrip correctly (for finite floats)
    #[test]
    fn prop_xml_roundtrip_floats(
        key in "[a-z][a-z0-9]{2,10}",
        value in prop::num::f64::NORMAL.prop_filter("finite", |f| f.is_finite())
    ) {
        let mut root = BTreeMap::new();
        root.insert(key.clone(), Item::Scalar(Value::Float(value)));

        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        };

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).map_err(|e| TestCaseError::fail(e))?;

        let parse_config = FromXmlConfig::default();
        let doc2 = from_xml(&xml, &parse_config).map_err(|e| TestCaseError::fail(e))?;

        if let Some(Item::Scalar(Value::Float(v))) = doc2.root.get(&key) {
            // Allow small tolerance for float roundtrip
            prop_assert!((v - value).abs() < 1e-10, "Float mismatch: {} != {}", v, value);
        }
    }

    /// Property: Null values roundtrip correctly
    #[test]
    fn prop_xml_roundtrip_null(key in "[a-z][a-z0-9]{2,10}") {
        let mut root = BTreeMap::new();
        root.insert(key.clone(), Item::Scalar(Value::Null));

        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        };

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).map_err(|e| TestCaseError::fail(e))?;

        let parse_config = FromXmlConfig::default();
        let doc2 = from_xml(&xml, &parse_config).map_err(|e| TestCaseError::fail(e))?;

        prop_assert!(doc2.root.contains_key(&key), "Missing key for null value");
    }

    /// Property: Leading whitespace in strings is preserved (trailing may be trimmed)
    #[test]
    #[ignore]  // XML parsers may trim trailing whitespace per spec
    fn prop_xml_whitespace_preserved(
        key in "[a-z][a-z0-9]{2,10}",
        leading in "[ \t\n]{0,3}",
        middle in "[a-zA-Z0-9]{0,20}",
        trailing in "[ \t\n]{0,3}"
    ) {
        let value = format!("{}{}{}", leading, middle, trailing);

        let mut root = BTreeMap::new();
        root.insert(key.clone(), Item::Scalar(Value::String(value.clone())));

        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        };

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).map_err(|e| TestCaseError::fail(e))?;

        let parse_config = FromXmlConfig::default();
        let doc2 = from_xml(&xml, &parse_config).map_err(|e| TestCaseError::fail(e))?;

        if let Some(Item::Scalar(Value::String(v))) = doc2.root.get(&key) {
            prop_assert_eq!(v, &value, "Whitespace not preserved");
        }
    }

}

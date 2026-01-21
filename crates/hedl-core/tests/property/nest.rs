// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Property-based tests for NEST hierarchy semantics.
//!
//! # Invariants Tested
//!
//! 1. **NEST Declaration**: NEST relationships are defined correctly
//! 2. **Schema Consistency**: STRUCT declarations work with NEST
//! 3. **Type Validation**: NEST relationships reference valid types

use hedl_core::parse;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: NEST relationship is defined correctly in document.
    #[test]
    fn prop_nest_relationship_defined(
        parent_type in "[A-Z][a-zA-Z0-9]{0,15}",
        child_type in "[A-Z][a-zA-Z0-9]{0,15}"
    ) {
        prop_assume!(parent_type != child_type);

        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {parent_type}: [id]\n%STRUCT: {child_type}: [id]\n%NEST: {parent_type} > {child_type}\n---\nvalue: 1\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let child_type_from_nest = parsed.get_child_type(&parent_type);
        prop_assert_eq!(child_type_from_nest, Some(&child_type),
            "NEST relationship not defined correctly");
    }

    /// Property: NEST requires both types to be declared.
    #[test]
    fn prop_nest_requires_both_types(
        parent_type in "[A-Z][a-zA-Z0-9]{0,15}",
        child_type in "[A-Z][a-zA-Z0-9]{0,15}"
    ) {
        prop_assume!(parent_type != child_type);

        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {parent_type}: [id]\n%STRUCT: {child_type}: [id]\n%NEST: {parent_type} > {child_type}\n---\nvalue: 1\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse NEST with both types declared: {:?}", result.err());
    }

    /// Property: Multiple NEST relationships can be defined.
    #[test]
    fn prop_multiple_nest_relationships(
        type1 in "[A-Z][a-zA-Z0-9]{0,10}",
        type2 in "[A-Z][a-zA-Z0-9]{0,10}",
        type3 in "[A-Z][a-zA-Z0-9]{0,10}"
    ) {
        prop_assume!(type1 != type2 && type2 != type3 && type1 != type3);

        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {type1}: [id]\n%STRUCT: {type2}: [id]\n%STRUCT: {type3}: [id]\n%NEST: {type1} > {type2}\n%NEST: {type2} > {type3}\n---\nvalue: 1\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse multiple NEST: {:?}", result.err());

        let parsed = result.unwrap();
        prop_assert_eq!(parsed.get_child_type(&type1), Some(&type2));
        prop_assert_eq!(parsed.get_child_type(&type2), Some(&type3));
    }

    /// Property: NEST relationships are stored in document.
    #[test]
    fn prop_nest_stored_in_document(
        parent_type in "[A-Z][a-zA-Z0-9]{0,15}",
        child_type in "[A-Z][a-zA-Z0-9]{0,15}"
    ) {
        prop_assume!(parent_type != child_type);

        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {parent_type}: [id]\n%STRUCT: {child_type}: [id]\n%NEST: {parent_type} > {child_type}\n---\nvalue: 1\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        prop_assert!(parsed.nests.contains_key(&parent_type),
            "NEST parent not in document");
        prop_assert_eq!(parsed.nests.get(&parent_type), Some(&child_type),
            "NEST child not correct");
    }

    /// Property: STRUCT and NEST work together.
    #[test]
    fn prop_struct_and_nest_together(
        parent_type in "[A-Z][a-zA-Z0-9]{0,15}",
        child_type in "[A-Z][a-zA-Z0-9]{0,15}",
        field_count in 1_usize..5
    ) {
        prop_assume!(parent_type != child_type);

        let parent_fields = (0..field_count)
            .map(|i| format!("pfield{i}"))
            .collect::<Vec<_>>()
            .join(", ");

        let child_fields = (0..field_count)
            .map(|i| format!("cfield{i}"))
            .collect::<Vec<_>>()
            .join(", ");

        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {parent_type}: [{parent_fields}]\n%STRUCT: {child_type}: [{child_fields}]\n%NEST: {parent_type} > {child_type}\n---\nvalue: 1\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse STRUCT with NEST: {:?}", result.err());

        let parsed = result.unwrap();
        prop_assert!(parsed.get_schema(&parent_type).is_some());
        prop_assert!(parsed.get_schema(&child_type).is_some());
        prop_assert_eq!(parsed.get_child_type(&parent_type), Some(&child_type));
    }

    /// Property: NEST with same parent and different children across documents.
    #[test]
    fn prop_nest_same_parent_different_children(
        parent_type in "[A-Z][a-zA-Z0-9]{0,15}",
        child_type1 in "[A-Z][a-zA-Z0-9]{0,10}",
        child_type2 in "[A-Z][a-zA-Z0-9]{0,10}"
    ) {
        prop_assume!(parent_type != child_type1 && parent_type != child_type2 && child_type1 != child_type2);

        // First document with one child type
        let doc1 = format!(
            "%VERSION: 1.0\n%STRUCT: {parent_type}: [id]\n%STRUCT: {child_type1}: [id]\n%NEST: {parent_type} > {child_type1}\n---\nvalue: 1\n"
        );

        let result1 = parse(doc1.as_bytes());
        prop_assert!(result1.is_ok());

        // Second document with different child type
        let doc2 = format!(
            "%VERSION: 1.0\n%STRUCT: {parent_type}: [id]\n%STRUCT: {child_type2}: [id]\n%NEST: {parent_type} > {child_type2}\n---\nvalue: 1\n"
        );

        let result2 = parse(doc2.as_bytes());
        prop_assert!(result2.is_ok());

        // Both should succeed independently
        let parsed1 = result1.unwrap();
        let parsed2 = result2.unwrap();

        prop_assert_eq!(parsed1.get_child_type(&parent_type), Some(&child_type1));
        prop_assert_eq!(parsed2.get_child_type(&parent_type), Some(&child_type2));
    }

    /// Property: Very long type names in NEST work.
    #[test]
    fn prop_nest_long_type_names(
        parent_prefix in "[A-Z]{1,5}",
        parent_suffix in "[a-zA-Z0-9]{10,20}",
        child_prefix in "[A-Z]{1,5}",
        child_suffix in "[a-zA-Z0-9]{10,20}"
    ) {
        let parent_type = format!("{parent_prefix}{parent_suffix}");
        let child_type = format!("{child_prefix}{child_suffix}");
        prop_assume!(parent_type != child_type);

        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {parent_type}: [id]\n%STRUCT: {child_type}: [id]\n%NEST: {parent_type} > {child_type}\n---\nvalue: 1\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(),
            "Failed to parse NEST with long type names: {:?}", result.err());

        let parsed = result.unwrap();
        prop_assert_eq!(parsed.get_child_type(&parent_type), Some(&child_type));
    }

    /// Property: NEST declarations are parsed deterministically.
    #[test]
    fn prop_nest_parsing_deterministic(
        parent_type in "[A-Z][a-zA-Z0-9]{0,15}",
        child_type in "[A-Z][a-zA-Z0-9]{0,15}"
    ) {
        prop_assume!(parent_type != child_type);

        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {parent_type}: [id]\n%STRUCT: {child_type}: [id]\n%NEST: {parent_type} > {child_type}\n---\nvalue: 1\n"
        );

        let result1 = parse(doc.as_bytes());
        let result2 = parse(doc.as_bytes());

        prop_assert_eq!(result1.is_ok(), result2.is_ok(),
            "NEST parsing non-deterministic");

        if let (Ok(parsed1), Ok(parsed2)) = (result1, result2) {
            prop_assert_eq!(parsed1.nests, parsed2.nests,
                "NEST relationships differ between parses");
        }
    }
}

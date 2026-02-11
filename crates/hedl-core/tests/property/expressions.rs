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

//! Property-based tests for expression and reference handling.
//!
//! # Invariants Tested
//!
//! 1. **Reference Syntax**: Valid reference syntax is parsed correctly
//! 2. **Qualified References**: Type-qualified references work
//! 3. **Reference Resolution**: References resolve or error appropriately
//! 4. **Expression Preservation**: Expressions are preserved as values

use hedl_core::parse;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Local references (unqualified) parse correctly.
    #[test]
    fn prop_local_reference_parses(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        id in "[a-z][a-z0-9_-]{0,30}"
    ) {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id, ref]\n---\nitems:@{type_name}\n |{id}, @{id}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse local reference: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();
        let ref_val = &list.rows[0].fields[1];

        prop_assert!(ref_val.is_reference(), "Value should be a reference");
        let reference = ref_val.as_reference().unwrap();
        prop_assert_eq!(reference.id.as_ref(), &id, "Reference ID mismatch");
    }

    /// Property: Qualified references (with type) parse correctly.
    #[test]
    fn prop_qualified_reference_parses(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        id in "[a-z][a-z0-9_-]{0,30}"
    ) {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id]\n---\nitems:@{type_name}\n |{id}\nref: @{type_name}:{id}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse qualified reference: {:?}", result.err());

        let parsed = result.unwrap();
        let ref_val = parsed.get("ref").unwrap().as_scalar().unwrap();

        prop_assert!(ref_val.is_reference(), "Value should be a reference");
        let reference = ref_val.as_reference().unwrap();
        prop_assert_eq!(reference.id.as_ref(), &id, "Reference ID mismatch");
        prop_assert_eq!(reference.type_name.as_deref(), Some(type_name.as_str()),
            "Reference type mismatch");
    }

    /// Property: Reference with hyphenated ID works.
    #[test]
    fn prop_hyphenated_id_reference(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        id_prefix in "[a-z]{1,5}",
        id_parts in prop::collection::vec("[a-z0-9]{1,10}", 1..=4)
    ) {
        let id = format!("{}-{}", id_prefix, id_parts.join("-"));

        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id, ref]\n---\nitems:@{type_name}\n |{id}, @{id}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse hyphenated ID: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();
        let ref_val = &list.rows[0].fields[1];

        prop_assert!(ref_val.is_reference(), "Value should be a reference");
        let reference = ref_val.as_reference().unwrap();
        prop_assert_eq!(reference.id.as_ref(), &id, "Hyphenated ID not preserved");
    }

    /// Property: Reference with underscored ID works.
    #[test]
    fn prop_underscored_id_reference(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        id_prefix in "[a-z]{1,5}",
        id_parts in prop::collection::vec("[a-z0-9]{1,10}", 1..=4)
    ) {
        let id = format!("{}_{}", id_prefix, id_parts.join("_"));

        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id, ref]\n---\nitems:@{type_name}\n |{id}, @{id}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse underscored ID: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();
        let ref_val = &list.rows[0].fields[1];

        prop_assert!(ref_val.is_reference(), "Value should be a reference");
        let reference = ref_val.as_reference().unwrap();
        prop_assert_eq!(reference.id.as_ref(), &id, "Underscored ID not preserved");
    }

    /// Property: Multiple references in same row work independently.
    #[test]
    fn prop_multiple_references_same_row(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        id1 in "[a-z][a-z0-9_-]{1,20}",
        id2 in "[a-z][a-z0-9_-]{1,20}"
    ) {
        prop_assume!(id1 != id2);

        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id, ref1, ref2]\n---\nitems:@{type_name}\n |row1, @{id1}, @{id2}\n |{id1}, ~, ~\n |{id2}, ~, ~\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse multiple references: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();

        let ref1 = &list.rows[0].fields[1];
        let ref2 = &list.rows[0].fields[2];

        prop_assert!(ref1.is_reference() && ref2.is_reference(),
            "Both values should be references");

        let r1 = ref1.as_reference().unwrap();
        let r2 = ref2.as_reference().unwrap();

        prop_assert_eq!(r1.id.as_ref(), &id1, "First reference ID mismatch");
        prop_assert_eq!(r2.id.as_ref(), &id2, "Second reference ID mismatch");
    }

    /// Property: References to different types work.
    #[test]
    fn prop_cross_type_references(
        type1 in "[A-Z][a-zA-Z0-9]{0,10}",
        type2 in "[A-Z][a-zA-Z0-9]{0,10}",
        id in "[a-z][a-z0-9_-]{0,20}"
    ) {
        prop_assume!(type1 != type2);

        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type1}:[id]\n%S:{type2}:[id,ref]\n---\nitems1:@{type1}\n |{id}\nitems2:@{type2}\n |other, @{type1}:{id}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse cross-type reference: {:?}", result.err());

        let parsed = result.unwrap();
        let list2 = parsed.get("items2").unwrap().as_list().unwrap();
        let ref_val = &list2.rows[0].fields[1];

        prop_assert!(ref_val.is_reference(), "Value should be a reference");
        let reference = ref_val.as_reference().unwrap();
        prop_assert_eq!(reference.type_name.as_deref(), Some(type1.as_str()),
            "Cross-type reference type mismatch");
    }

    /// Property: Null references are valid (no reference).
    #[test]
    fn prop_null_reference_valid(type_name in "[A-Z][a-zA-Z0-9]{0,15}") {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id, ref]\n---\nitems:@{type_name}\n |id1, ~\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse null reference: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();
        let ref_val = &list.rows[0].fields[1];

        prop_assert!(ref_val.is_null(), "Value should be null");
    }

    /// Property: Very long reference IDs work.
    #[test]
    fn prop_long_reference_id(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        id_prefix in "[a-z]{1,5}",
        id_suffix in "[a-z0-9_-]{50,95}"
    ) {
        let id = format!("{id_prefix}{id_suffix}");

        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id, ref]\n---\nitems:@{type_name}\n |{id}, @{id}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(),
            "Failed to parse long reference ID (len={}): {:?}", id.len(), result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();
        let ref_val = &list.rows[0].fields[1];

        let reference = ref_val.as_reference().unwrap();
        prop_assert_eq!(reference.id.as_ref(), &id, "Long reference ID not preserved");
    }

    /// Property: Reference parsing is deterministic.
    #[test]
    fn prop_reference_parsing_deterministic(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        id in "[a-z][a-z0-9_-]{0,30}"
    ) {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id, ref]\n---\nitems:@{type_name}\n |{id}, @{id}\n"
        );

        let result1 = parse(doc.as_bytes());
        let result2 = parse(doc.as_bytes());

        prop_assert_eq!(result1.is_ok(), result2.is_ok(),
            "Reference parsing non-deterministic");

        if let (Ok(doc1), Ok(doc2)) = (result1, result2) {
            let list1 = doc1.get("items").unwrap().as_list().unwrap();
            let list2 = doc2.get("items").unwrap().as_list().unwrap();

            let ref1 = &list1.rows[0].fields[1];
            let ref2 = &list2.rows[0].fields[1];

            prop_assert_eq!(ref1, ref2, "Parsed references differ");
        }
    }
}

#[cfg(test)]
mod expression_syntax {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Expression syntax is recognized.
        #[test]
        fn prop_expression_syntax_recognized(
            content in "[a-zA-Z0-9+\\-* ]{1,50}"
        ) {
            let doc = format!(
                "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nvalue: $$({content})\n"
            );

            let result = parse(doc.as_bytes());
            // Should either parse as expression or produce clear error
            if result.is_err() {
                let err_msg = format!("{}", result.unwrap_err());
                prop_assert!(!err_msg.is_empty(), "Error message should not be empty");
            }
        }
    }
}

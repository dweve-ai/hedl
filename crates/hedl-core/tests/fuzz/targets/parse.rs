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

//! Fuzz target for HEDL parser.
//!
//! This fuzz target ensures the parser never panics on arbitrary input,
//! only returning errors for invalid documents.
//!
//! To run this fuzz target:
//! ```bash
//! cargo install cargo-fuzz
//! cd crates/hedl-core
//! cargo fuzz run parse
//! ```
//!
//! The fuzzer will generate random byte sequences and feed them to the parser,
//! looking for panics, crashes, or infinite loops.

#![no_main]

use libfuzzer_sys::fuzz_target;
use hedl_core::parse;

fuzz_target!(|data: &[u8]| {
    // The parser should never panic - it should only return errors
    // for invalid input. We don't care about the result, only that
    // it doesn't crash.
    let _ = parse(data);
});

/// Fuzz target for parser with size limits to catch issues faster.
///
/// This variant limits input size to find bugs more quickly.
#[cfg(fuzzing_fast)]
fuzz_target!(|data: &[u8]| {
    // Limit to 1KB for faster fuzzing cycles
    if data.len() <= 1024 {
        let _ = parse(data);
    }
});

/// Fuzz target that checks invariants after successful parsing.
///
/// This verifies that successfully parsed documents maintain
/// expected invariants (e.g., all references resolve, schema consistency).
#[cfg(fuzzing_invariants)]
fuzz_target!(|data: &[u8]| {
    if let Ok(doc) = parse(data) {
        // Invariant 1: Version should be valid
        assert!(doc.version.0 > 0, "Major version must be positive");

        // Invariant 2: All struct schemas should have at least one column
        for (type_name, schema) in &doc.structs {
            assert!(!schema.is_empty(),
                "Type '{}' has empty schema", type_name);
        }

        // Invariant 3: All NEST relationships should reference valid types
        for (parent_type, child_type) in &doc.nests {
            assert!(doc.structs.contains_key(parent_type),
                "NEST parent type '{}' not in structs", parent_type);
            assert!(doc.structs.contains_key(child_type),
                "NEST child type '{}' not in structs", child_type);
        }

        // Invariant 4: All list types should have schemas
        fn check_items(items: &std::collections::BTreeMap<String, hedl_core::Item>,
                      structs: &std::collections::BTreeMap<String, Vec<String>>) {
            for item in items.values() {
                if let hedl_core::Item::List(list) = item {
                    assert!(structs.contains_key(&list.type_name),
                        "List type '{}' not in structs", list.type_name);

                    // Check that all rows have correct field count
                    let expected_fields = list.schema.len();
                    for (idx, row) in list.rows.iter().enumerate() {
                        assert_eq!(row.fields.len(), expected_fields,
                            "Row {} has {} fields, expected {}",
                            idx, row.fields.len(), expected_fields);
                    }
                }
            }
        }

        check_items(&doc.root, &doc.structs);
    }
});

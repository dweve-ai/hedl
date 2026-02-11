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

//! Comprehensive tests for all fixture functions.
//!
//! This test file ensures all fixtures produce valid documents
//! and have expected properties.

use hedl_core::{Item, Value};
use hedl_test::fixtures;

#[test]
fn test_all_fixtures_are_valid() {
    for (name, fixture_fn) in fixtures::all() {
        let doc = fixture_fn();
        assert_eq!(
            doc.version,
            (1, 2),
            "Fixture '{name}' should have version 1.0"
        );
    }
}

#[test]
fn test_scalars_fixture() {
    let doc = fixtures::scalars();

    // Check for all expected scalar types
    assert!(doc.root.contains_key("null_val"));
    assert!(doc.root.contains_key("bool_true"));
    assert!(doc.root.contains_key("bool_false"));
    assert!(doc.root.contains_key("int_positive"));
    assert!(doc.root.contains_key("int_negative"));
    assert!(doc.root.contains_key("int_zero"));
    assert!(doc.root.contains_key("float_positive"));
    assert!(doc.root.contains_key("float_negative"));
    assert!(doc.root.contains_key("float_zero"));
    assert!(doc.root.contains_key("string_simple"));
    assert!(doc.root.contains_key("string_empty"));

    // Verify types
    assert!(matches!(
        doc.root.get("null_val"),
        Some(Item::Scalar(Value::Null))
    ));

    assert!(matches!(
        doc.root.get("bool_true"),
        Some(Item::Scalar(Value::Bool(true)))
    ));

    assert!(matches!(
        doc.root.get("int_positive"),
        Some(Item::Scalar(Value::Int(42)))
    ));
}

#[test]
fn test_special_strings_fixture() {
    let doc = fixtures::special_strings();

    assert!(doc.root.contains_key("with_quotes"));
    assert!(doc.root.contains_key("with_backslash"));
    assert!(doc.root.contains_key("with_newline"));
    assert!(doc.root.contains_key("with_tab"));
    assert!(doc.root.contains_key("with_unicode"));
    assert!(doc.root.contains_key("with_mixed"));

    // Verify string content preservation
    if let Some(Item::Scalar(Value::String(s))) = doc.root.get("with_unicode") {
        assert!(s.contains("日本語"));
        assert!(s.contains("🎉"));
    } else {
        panic!("Expected unicode string");
    }
}

#[test]
fn test_references_fixture() {
    let doc = fixtures::references();

    assert!(doc.root.contains_key("local_ref"));
    assert!(doc.root.contains_key("typed_ref"));

    // Check reference structure
    if let Some(Item::Scalar(Value::Reference(r))) = doc.root.get("local_ref") {
        assert!(r.type_name.is_none());
        assert_eq!(r.id.as_ref(), "some_id");
    } else {
        panic!("Expected local reference");
    }

    if let Some(Item::Scalar(Value::Reference(r))) = doc.root.get("typed_ref") {
        assert!(r.type_name.is_some());
        assert_eq!(r.type_name.as_ref().unwrap().as_ref(), "User");
        assert_eq!(r.id.as_ref(), "alice");
    } else {
        panic!("Expected typed reference");
    }
}

#[test]
fn test_expressions_fixture() {
    let doc = fixtures::expressions();

    assert!(doc.root.contains_key("simple_expr"));
    assert!(doc.root.contains_key("var_expr"));
    assert!(doc.root.contains_key("complex_expr"));

    // Verify all are expressions
    for key in &["simple_expr", "var_expr", "complex_expr"] {
        assert!(
            matches!(doc.root.get(*key), Some(Item::Scalar(Value::Expression(_)))),
            "Expected expression for {key}"
        );
    }
}

#[test]
fn test_tensors_fixture() {
    let doc = fixtures::tensors();

    assert!(doc.root.contains_key("tensor_1d"));
    assert!(doc.root.contains_key("tensor_2d"));
    assert!(doc.root.contains_key("tensor_3d"));
    assert!(doc.root.contains_key("tensor_empty"));

    // Verify tensor types
    for key in &["tensor_1d", "tensor_2d", "tensor_3d", "tensor_empty"] {
        assert!(
            matches!(doc.root.get(*key), Some(Item::Scalar(Value::Tensor(_)))),
            "Expected tensor for {key}"
        );
    }
}

#[test]
fn test_named_values_fixture() {
    let doc = fixtures::named_values();

    assert_eq!(doc.root.len(), 6);
    assert!(doc.root.contains_key("app_name"));
    assert!(doc.root.contains_key("version"));
    assert!(doc.root.contains_key("debug_mode"));
    assert!(doc.root.contains_key("max_connections"));
    assert!(doc.root.contains_key("timeout_seconds"));
    assert!(doc.root.contains_key("deprecated_feature"));
}

#[test]
fn test_user_list_fixture() {
    let doc = fixtures::user_list();

    assert!(doc.root.contains_key("users"));
    assert!(doc.structs.contains_key("User"));

    if let Some(Item::List(list)) = doc.root.get("users") {
        assert_eq!(list.type_name, "User");
        assert_eq!(list.schema, vec!["id", "name", "email"]);
        assert_eq!(list.rows.len(), 3);

        // Check first user
        let alice = &list.rows[0];
        assert_eq!(alice.id, "alice");
        assert_eq!(alice.type_name, "User");
        assert_eq!(alice.fields.len(), 3);
    } else {
        panic!("Expected users list");
    }
}

#[test]
fn test_mixed_type_list_fixture() {
    let doc = fixtures::mixed_type_list();

    if let Some(Item::List(list)) = doc.root.get("items") {
        assert_eq!(list.rows.len(), 2);

        // First item has all field types
        let item = &list.rows[0];
        assert_eq!(item.fields.len(), 6);

        // Verify mixed types in fields
        assert!(matches!(item.fields[0], Value::String(_)));
        assert!(matches!(item.fields[1], Value::String(_)));
        assert!(matches!(item.fields[2], Value::Int(_)));
        assert!(matches!(item.fields[3], Value::Float(_)));
        assert!(matches!(item.fields[4], Value::Bool(_)));
        assert!(matches!(item.fields[5], Value::String(_)));
    } else {
        panic!("Expected items list");
    }
}

#[test]
fn test_with_references_fixture() {
    let doc = fixtures::with_references();

    assert!(doc.root.contains_key("users"));
    assert!(doc.root.contains_key("posts"));
    assert_eq!(hedl_test::count_references(&doc), 3);

    if let Some(Item::List(posts)) = doc.root.get("posts") {
        // All posts should have author references
        for post in &posts.rows {
            assert!(matches!(post.fields[2], Value::Reference(_)));
        }
    }
}

#[test]
fn test_with_nest_fixture() {
    let doc = fixtures::with_nest();

    assert!(doc.nests.contains_key("User"));
    assert_eq!(doc.nests.get("User"), Some(&vec!["Post".to_string()]));

    if let Some(Item::List(users)) = doc.root.get("users") {
        // Check alice has children
        let alice = users.rows.iter().find(|n| n.id == "alice").unwrap();
        assert!(alice.children.is_some());

        if let Some(ref children) = alice.children {
            assert!(children.contains_key("posts"));
            assert_eq!(children.get("posts").unwrap().len(), 2);
        }
    }
}

#[test]
fn test_deep_nest_fixture() {
    let doc = fixtures::deep_nest();

    // Check 3 levels of nesting
    assert!(doc.structs.contains_key("Organization"));
    assert!(doc.structs.contains_key("Department"));
    assert!(doc.structs.contains_key("Employee"));

    assert_eq!(
        doc.nests.get("Organization"),
        Some(&vec!["Department".to_string()])
    );
    assert_eq!(
        doc.nests.get("Department"),
        Some(&vec!["Employee".to_string()])
    );

    // Verify nested structure
    if let Some(Item::List(orgs)) = doc.root.get("organizations") {
        assert_eq!(orgs.rows.len(), 1);
        let org = &orgs.rows[0];

        if let Some(ref org_children) = org.children {
            assert!(org_children.contains_key("departments"));

            let dept = &org_children.get("departments").unwrap()[0];
            if let Some(ref dept_children) = dept.children {
                assert!(dept_children.contains_key("employees"));
                assert_eq!(dept_children.get("employees").unwrap().len(), 2);
            }
        }
    }
}

#[test]
fn test_edge_cases_fixture() {
    let doc = fixtures::edge_cases();

    assert!(doc.root.contains_key("large_int"));
    assert!(doc.root.contains_key("small_int"));
    assert!(doc.root.contains_key("tiny_float"));
    assert!(doc.root.contains_key("large_float"));
    assert!(doc.root.contains_key("long_string"));
    assert!(doc.root.contains_key("special_only"));

    // Verify extreme values
    if let Some(Item::Scalar(Value::Int(i))) = doc.root.get("large_int") {
        assert_eq!(*i, i64::MAX);
    }

    if let Some(Item::Scalar(Value::String(s))) = doc.root.get("long_string") {
        assert_eq!(s.len(), 10000);
    }
}

#[test]
fn test_comprehensive_fixture() {
    let doc = fixtures::comprehensive();

    // Should have multiple entity types
    assert!(!doc.root.is_empty());
    assert!(!doc.structs.is_empty());
    assert!(!doc.nests.is_empty());

    // Check for various content types
    let has_scalar = doc.root.values().any(|i| matches!(i, Item::Scalar(_)));
    let has_list = doc.root.values().any(|i| matches!(i, Item::List(_)));

    assert!(has_scalar, "Should have scalar values");
    assert!(has_list, "Should have lists");

    // Should have users, comments, tags
    assert!(doc.root.contains_key("users"));
    assert!(doc.root.contains_key("comments"));
    assert!(doc.root.contains_key("tags"));
}

#[test]
fn test_blog_fixture() {
    let doc = fixtures::blog();

    // Verify all entity types present
    assert!(doc.root.contains_key("users"));
    assert!(doc.root.contains_key("categories"));
    assert!(doc.root.contains_key("tags"));
    assert!(doc.root.contains_key("posts"));
    assert!(doc.root.contains_key("comments"));
    assert!(doc.root.contains_key("reactions"));
    assert!(doc.root.contains_key("post_tags"));
    assert!(doc.root.contains_key("followers"));

    // Verify struct definitions
    assert!(doc.structs.contains_key("User"));
    assert!(doc.structs.contains_key("Post"));
    assert!(doc.structs.contains_key("Comment"));

    // Check for references
    let ref_count = hedl_test::count_references(&doc);
    assert!(ref_count > 0, "Blog should have references");
}

#[test]
fn test_empty_fixture() {
    let doc = fixtures::empty();

    assert_eq!(doc.version, (1, 2));
    assert!(doc.root.is_empty());
    assert!(doc.structs.is_empty());
    assert!(doc.nests.is_empty());
    assert!(doc.aliases.is_empty());
}

#[test]
fn test_fixtures_as_hedl() {
    let hedl_texts = hedl_test::fixtures_as_hedl();

    assert!(!hedl_texts.is_empty());

    for (name, text) in hedl_texts {
        assert!(!name.is_empty(), "Fixture name should not be empty");
        assert!(!text.is_empty(), "HEDL text should not be empty for {name}");
        assert!(!text.contains("Error"), "Should not contain error: {name}");
    }
}

#[test]
fn test_all_fixtures_have_unique_names() {
    let fixtures = fixtures::all();
    let mut names = std::collections::HashSet::new();

    for (name, _) in fixtures {
        assert!(names.insert(name), "Duplicate fixture name found: {name}");
    }
}

#[test]
fn test_fixture_count_is_complete() {
    let fixtures = fixtures::all();

    // Should have all documented fixtures
    let expected = vec![
        "scalars",
        "special_strings",
        "references",
        "expressions",
        "tensors",
        "named_values",
        "user_list",
        "mixed_type_list",
        "with_references",
        "with_nest",
        "deep_nest",
        "edge_cases",
        "comprehensive",
        "blog",
        "empty",
    ];

    assert_eq!(
        fixtures.len(),
        expected.len(),
        "Fixture count mismatch. Expected {}, got {}",
        expected.len(),
        fixtures.len()
    );

    for expected_name in expected {
        assert!(
            fixtures.iter().any(|(name, _)| *name == expected_name),
            "Missing fixture: {expected_name}"
        );
    }
}

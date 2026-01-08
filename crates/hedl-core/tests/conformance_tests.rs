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

//! HEDL Conformance Tests
//!
//! These tests verify compliance with the HEDL 1.0 specification.
//! Based on Appendix B: Conformance Test Suite.

use hedl_core::{parse, HedlErrorKind, Value};

// =============================================================================
// B.1 Syntax Validation
// =============================================================================

/// B.1.1: Odd indentation -> Syntax Error
#[test]
fn test_odd_indentation_error() {
    let doc = "%VERSION: 1.0\n---\na:\n   b: 1\n"; // 3 spaces
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}

/// B.1.2: Tab character for indentation -> Syntax Error
#[test]
fn test_tab_indentation_error() {
    let doc = "%VERSION: 1.0\n---\na:\n\tb: 1\n"; // tab
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}

/// B.1.3: Missing separator -> Syntax Error
#[test]
fn test_missing_separator_error() {
    let doc = "%VERSION: 1.0\na: 1\n"; // no ---
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}

/// B.1.4: Multiple separators -> Syntax Error
#[test]
fn test_multiple_separators_error() {
    let doc = "%VERSION: 1.0\n---\na: 1\n---\nb: 2\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}

/// B.1.5: Body missing space after colon -> Syntax Error
#[test]
fn test_missing_space_after_colon_error() {
    let doc = "%VERSION: 1.0\n---\na:1\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}

/// B.1.6: Uppercase IDs are now valid (real-world IDs like SKU-4020)
#[test]
fn test_valid_id_uppercase_ok() {
    // Uppercase IDs are valid
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | SKU-4020, test\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

#[test]
fn test_invalid_reference_starts_digit_error() {
    // IDs cannot start with a digit
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | 123User, test\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
}

/// B.1.7: Control characters -> Syntax Error
#[test]
fn test_control_character_error() {
    let doc = "%VERSION: 1.0\n---\na: test\x01value\n"; // SOH control char
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}

/// B.1.8: Bare CR -> Syntax Error
#[test]
fn test_bare_cr_error() {
    let doc = "%VERSION: 1.0\r---\ra: 1\r"; // CR only, no LF
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}

// =============================================================================
// B.2 Schema Validation
// =============================================================================

/// B.2.1: Unknown type without inline schema -> Schema Error
#[test]
fn test_unknown_type_error() {
    let doc = "%VERSION: 1.0\n---\ndata: @UnknownType\n  | x, 1\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Schema));
}

/// B.2.2: Schema mismatch -> Schema Error
#[test]
fn test_schema_mismatch_error() {
    let doc = "%VERSION: 1.0\n%STRUCT: User: [id,name,email]\n---\nusers: @User[id, name]\n  | u1, Alice\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Schema));
}

/// B.2.3: Duplicate struct with different columns -> Schema Error
#[test]
fn test_duplicate_struct_different_columns_error() {
    let doc = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: User: [id, email]\n---\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Schema));
}

/// B.2.4: Nest to undefined type -> Schema Error
#[test]
fn test_nest_undefined_type_error() {
    let doc = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%NEST: User > Post\n---\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Schema));
}

/// B.2.5: Duplicate struct with identical columns -> OK (idempotent)
#[test]
fn test_duplicate_struct_identical_columns_ok() {
    let doc = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: User: [id,name]\n---\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

// =============================================================================
// B.3 Data Validation
// =============================================================================

/// B.3.1: Shape mismatch (wrong cell count) -> Shape Error
#[test]
fn test_shape_mismatch_error() {
    let doc = "%VERSION: 1.0\n%STRUCT: User: [id,name,email]\n---\nusers: @User\n  | u1, Alice\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Shape));
}

/// B.3.2: First row ditto -> Semantic Error
#[test]
fn test_first_row_ditto_error() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | x, ^\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Semantic));
}

/// B.3.3: Orphan child row -> Orphan Row Error
#[test]
fn test_orphan_child_row_error() {
    // Child row without %NEST directive
    let doc = "%VERSION: 1.0\n%STRUCT: Parent: [id]\n%STRUCT: Child: [id]\n---\nparents: @Parent\n  | p1\n    | c1\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::OrphanRow));
}

/// B.3.4: Duplicate ID within type -> Collision Error
#[test]
fn test_duplicate_id_collision_error() {
    let doc =
        "%VERSION: 1.0\n%STRUCT: User: [id,name]\n---\nusers: @User\n  | u1, Alice\n  | u1, Bob\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Collision));
}

/// B.3.5: Different ID across types -> Success
#[test]
fn test_different_id_across_types_ok() {
    let doc = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: Role: [id, name]\n---\nusers: @User\n  | admin, Alice\nroles: @Role\n  | admin, Administrator\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

/// B.3.6: Invalid ID type (number as ID) -> Semantic Error
#[test]
fn test_invalid_id_type_number_error() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | 123, test\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Semantic));
}

/// B.3.7: Uppercase ID format -> Now valid (real-world IDs like SKU-4020)
#[test]
fn test_valid_id_format_uppercase_ok() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | User1, test\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

#[test]
fn test_invalid_id_format_starts_digit_error() {
    // IDs cannot start with a digit
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | 123User, test\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Semantic));
}

/// B.3.8: Valid ID with dash -> Success
#[test]
fn test_valid_id_with_dash_ok() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | config-file, test\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

/// B.3.9: Ditto in ID column -> Semantic Error
#[test]
fn test_ditto_in_id_column_error() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | a, 1\n  | ^, 2\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Semantic));
}

/// B.3.10: Null in ID column -> Semantic Error
#[test]
fn test_null_in_id_column_error() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | ~, test\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Semantic));
}

// =============================================================================
// B.4 Reference Validation
// =============================================================================

/// B.4.1: Forward reference in same type -> Success
#[test]
fn test_forward_reference_ok() {
    let doc = "%VERSION: 1.0\n%STRUCT: Task: [id, depends_on]\n---\ntasks: @Task\n  | t1, @t2\n  | t2, ~\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

/// B.4.2: Missing reference -> Reference Error
#[test]
fn test_missing_reference_error() {
    let doc =
        "%VERSION: 1.0\n%STRUCT: Task: [id, depends_on]\n---\ntasks: @Task\n  | t1, @missing\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Reference));
}

/// B.4.3: Self reference -> Success
#[test]
fn test_self_reference_ok() {
    let doc = "%VERSION: 1.0\n%STRUCT: Task: [id, depends_on]\n---\ntasks: @Task\n  | t1, @t1\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

/// B.4.4: Circular reference -> Success (allowed)
#[test]
fn test_circular_reference_ok() {
    let doc = "%VERSION: 1.0\n%STRUCT: Task: [id, depends_on]\n---\ntasks: @Task\n  | t1, @t2\n  | t2, @t1\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

/// B.4.5: Qualified reference to other type
#[test]
fn test_qualified_reference_ok() {
    let doc = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: Post: [id,author]\n---\nusers: @User\n  | u1, Alice\nposts: @Post\n  | p1, @User:u1\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

/// B.4.6: Unqualified reference in matrix context searches only current type (SPEC 10.2-10.3)
#[test]
fn test_unqualified_reference_scoped_to_current_type() {
    // Same ID 'admin' exists in both User and Role types
    // Unqualified reference @admin in Post matrix should fail because 'admin' doesn't exist in Post
    let doc = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: Role: [id, name]\n%STRUCT: Post: [id, author_ref]\n---\nusers: @User\n  | admin, Alice\nroles: @Role\n  | admin, Administrator\nposts: @Post\n  | p1, @admin\n";
    let result = parse(doc.as_bytes());
    // Should fail: @admin doesn't exist in Post type registry
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Reference));
}

/// B.4.7: Ambiguous unqualified reference in key-value context (SPEC 10.3.1)
#[test]
fn test_ambiguous_unqualified_reference_error() {
    // Same ID 'admin' exists in both User and Role types
    // Unqualified reference @admin in key-value context should error (ambiguous)
    let doc = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: Role: [id, name]\n---\nusers: @User\n  | admin, Alice\nroles: @Role\n  | admin, Administrator\nconfig:\n  ref: @admin\n";
    let result = parse(doc.as_bytes());
    // Should fail: @admin is ambiguous (exists in both User and Role)
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Reference));
    assert!(err.to_string().contains("Ambiguous"));
}

// =============================================================================
// B.5 Parsing Correctness
// =============================================================================

/// B.5.1: Ditto scoping - doesn't copy from different list
#[test]
fn test_ditto_scoping() {
    let doc = "%VERSION: 1.0\n%STRUCT: A: [id, value]\n%STRUCT: B: [id, value]\n---\nlist_a: @A\n  | a1, apple\nlist_b: @B\n  | b1, ^\n"; // ^ in first row of list_b
    let result = parse(doc.as_bytes());
    assert!(result.is_err()); // ^ on first row is error
}

/// B.5.2: Child attachment via NEST
#[test]
fn test_child_attachment() {
    let doc = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: Post: [id,content]\n%NEST: User > Post\n---\nusers: @User\n  | u1, Alice\n    | p1, Hello\n    | p2, World\n  | u2, Bob\n    | p3, Hi\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("users").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
    // First user should have 2 children
    assert_eq!(list.rows[0].children.get("Post").map(|c| c.len()), Some(2));
    // Second user should have 1 child
    assert_eq!(list.rows[1].children.get("Post").map(|c| c.len()), Some(1));
}

/// B.5.3: Alias expansion
#[test]
fn test_alias_expansion() {
    let doc = "%VERSION: 1.0\n%ALIAS: %active: \"true\"\n%STRUCT: T: [id, status]\n---\ndata: @T\n  | x, %active\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("data").unwrap().as_list().unwrap();
    // fields[0] is ID, fields[1] is status
    assert_eq!(list.rows[0].fields[1], Value::Bool(true));
}

/// B.5.4: Hash in quoted CSV field is data
#[test]
fn test_hash_in_quoted_field() {
    let doc =
        "%VERSION: 1.0\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | x, \"value # with hash\"\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("data").unwrap().as_list().unwrap();
    // fields[0] is ID, fields[1] is value
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("value # with hash".to_string())
    );
}

/// B.5.5: Matrix row comment stripped before CSV parse
#[test]
fn test_matrix_row_comment_stripped() {
    let doc =
        "%VERSION: 1.0\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | x, test # this is a comment\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("data").unwrap().as_list().unwrap();
    // fields[0] is ID, fields[1] is value
    assert_eq!(list.rows[0].fields[1], Value::String("test".to_string()));
}

/// B.5.6: Quoted string escaping
#[test]
fn test_quoted_string_escaping() {
    let doc =
        "%VERSION: 1.0\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | x, \"escaped \"\"quote\"\"\"\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("data").unwrap().as_list().unwrap();
    // fields[0] is ID, fields[1] is value
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("escaped \"quote\"".to_string())
    );
}

/// B.5.7: Number inference
#[test]
fn test_number_inference() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id, int_val, float_val, explicit_float]\n---\ndata: @T\n  | x, 42, 3.25, 42.0\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("data").unwrap().as_list().unwrap();
    // fields[0] is ID, fields[1] is int_val, fields[2] is float_val, fields[3] is explicit_float
    assert_eq!(list.rows[0].fields[1], Value::Int(42));
    match &list.rows[0].fields[2] {
        Value::Float(f) => assert!((f - 3.25).abs() < 0.001),
        _ => panic!("expected float"),
    }
    match &list.rows[0].fields[3] {
        Value::Float(f) => assert!((f - 42.0).abs() < 0.001),
        _ => panic!("expected float"),
    }
}

/// B.5.8: Tensor literal parsing
#[test]
fn test_tensor_literal() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id, tensor1, tensor2]\n---\ndata: @T\n  | x, [1, 2, 3], [[1, 2], [3, 4]]\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("data").unwrap().as_list().unwrap();
    // fields[0] is ID, fields[1] is tensor1, fields[2] is tensor2
    assert!(matches!(list.rows[0].fields[1], Value::Tensor(_)));
    assert!(matches!(list.rows[0].fields[2], Value::Tensor(_)));
}

/// B.5.9: @ and $ in strings are not special when not at start
#[test]
fn test_at_and_dollar_in_strings() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id, email, price]\n---\ndata: @T\n  | x, alice@example.com, 100$\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("data").unwrap().as_list().unwrap();
    // fields[0] is ID, fields[1] is email, fields[2] is price
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("alice@example.com".to_string())
    );
    assert_eq!(list.rows[0].fields[2], Value::String("100$".to_string()));
}

// =============================================================================
// B.6 Edge Cases and Truncation Detection
// =============================================================================

/// B.6.1: Only header + separator -> Success (empty root object)
#[test]
fn test_empty_document_ok() {
    let doc = "%VERSION: 1.0\n---\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

/// B.6.2: Empty matrix list -> Success
#[test]
fn test_empty_matrix_list_ok() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id]\n---\ndata: @T\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("data").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 0);
}

/// B.6.3: Object start with comment
#[test]
fn test_object_start_with_comment() {
    let doc = "%VERSION: 1.0\n---\nconfig: # this is a comment\n  key: value\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let obj = doc.get("config").unwrap().as_object().unwrap();
    assert!(obj.contains_key("key"));
}

/// B.6.4: Empty alias
#[test]
fn test_empty_alias() {
    let doc = "%VERSION: 1.0\n%ALIAS: %empty: \"\"\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | x, %empty\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("data").unwrap().as_list().unwrap();
    // fields[0] is ID, fields[1] is value
    assert_eq!(list.rows[0].fields[1], Value::String("".to_string()));
}

/// B.6.5: Whitespace preservation in quoted strings
#[test]
fn test_whitespace_preservation() {
    let doc = "%VERSION: 1.0\n---\nkey: \"  spaces  \"\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let val = doc.get("key").unwrap().as_scalar().unwrap();
    assert_eq!(*val, Value::String("  spaces  ".to_string()));
}

/// B.6.6: Boolean case sensitivity
#[test]
fn test_boolean_case_sensitivity() {
    let doc = "%VERSION: 1.0\n---\na: true\nb: True\nc: TRUE\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    // Only lowercase 'true' should be boolean
    assert_eq!(
        *doc.get("a").unwrap().as_scalar().unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        *doc.get("b").unwrap().as_scalar().unwrap(),
        Value::String("True".to_string())
    );
    assert_eq!(
        *doc.get("c").unwrap().as_scalar().unwrap(),
        Value::String("TRUE".to_string())
    );
}

/// B.6.7: Expression with nested function call
#[test]
fn test_expression_nested_call() {
    use hedl_core::Expression;
    let doc = "%VERSION: 1.0\n---\nexpr: $(outer(inner(x)))\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let val = doc.get("expr").unwrap().as_scalar().unwrap();
    match val {
        Value::Expression(e) => {
            assert!(matches!(e, Expression::Call { name, .. } if name == "outer"));
        }
        _ => panic!("Expected expression"),
    }
}

/// B.6.8: Unclosed quote -> Syntax Error
#[test]
fn test_unclosed_quote_error() {
    let doc = "%VERSION: 1.0\n---\nkey: \"unclosed\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}

/// B.6.9: Tab in quoted string -> OK
#[test]
fn test_tab_in_quoted_string_ok() {
    let doc = "%VERSION: 1.0\n---\nkey: \"a\tb\"\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let val = doc.get("key").unwrap().as_scalar().unwrap();
    assert_eq!(*val, Value::String("a\tb".to_string()));
}

/// B.6.10: CRLF line endings -> OK
#[test]
fn test_crlf_line_endings_ok() {
    let doc = "%VERSION: 1.0\r\n---\r\na: 1\r\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

// =============================================================================
// B.7 Test Document (from spec)
// =============================================================================

/// Full conformance test document from spec
#[test]
fn test_conformance_document() {
    let doc = r#"%VERSION: 1.0
%ALIAS: %true: "true"
%STRUCT: Test: [id, value, ref]
%STRUCT: Child: [id, data]
%NEST: Test > Child
---
tests: @Test
  | t1, "simple", ~
    | c1, child
  | t2, 42, @t1
    | c2, child
  | t3, %true, @t2
  | t4, ^, ^
tensor_test: @TensorTest[id, data]
  | t5, [1, 2, 3]
  | t6, [[1, 2], [3, 4]]
"#;
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    // Verify tests list
    let tests = doc.get("tests").unwrap().as_list().unwrap();
    assert_eq!(tests.rows.len(), 4);

    // t1 with child c1
    // fields[0]=id, fields[1]=value, fields[2]=ref
    assert_eq!(tests.rows[0].id, "t1");
    assert_eq!(tests.rows[0].fields[1], Value::String("simple".to_string()));
    assert_eq!(tests.rows[0].fields[2], Value::Null);
    assert_eq!(
        tests.rows[0].children.get("Child").map(|c| c.len()),
        Some(1)
    );

    // t2 with child c2
    assert_eq!(tests.rows[1].id, "t2");
    assert_eq!(tests.rows[1].fields[1], Value::Int(42));
    assert!(matches!(tests.rows[1].fields[2], Value::Reference(_)));
    assert_eq!(
        tests.rows[1].children.get("Child").map(|c| c.len()),
        Some(1)
    );

    // t3 value = true (via alias)
    assert_eq!(tests.rows[2].id, "t3");
    assert_eq!(tests.rows[2].fields[1], Value::Bool(true));

    // t4 with ditto
    assert_eq!(tests.rows[3].id, "t4");
    assert_eq!(tests.rows[3].fields[1], Value::Bool(true)); // ditto from t3

    // Verify tensor_test list
    // fields[0]=id, fields[1]=data
    let tensors = doc.get("tensor_test").unwrap().as_list().unwrap();
    assert_eq!(tensors.rows.len(), 2);
    assert!(matches!(tensors.rows[0].fields[1], Value::Tensor(_)));
    assert!(matches!(tensors.rows[1].fields[1], Value::Tensor(_)));
}

// =============================================================================
// Additional Edge Cases
// =============================================================================

/// Nested objects
#[test]
fn test_nested_objects() {
    let doc = "%VERSION: 1.0\n---\nconfig:\n  database:\n    host: localhost\n    port: 5432\n  logging:\n    level: info\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let config = doc.get("config").unwrap().as_object().unwrap();
    let db = config.get("database").unwrap().as_object().unwrap();
    assert_eq!(
        *db.get("host").unwrap().as_scalar().unwrap(),
        Value::String("localhost".to_string())
    );
    assert_eq!(
        *db.get("port").unwrap().as_scalar().unwrap(),
        Value::Int(5432)
    );
}

/// Mixed object and list
#[test]
fn test_mixed_object_and_list() {
    let doc = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n---\nconfig:\n  name: Test\nusers: @User\n  | u1, Alice\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

/// Inline schema (without %STRUCT)
#[test]
fn test_inline_schema() {
    let doc = "%VERSION: 1.0\n---\nitems: @Item[id, name, price]\n  | i1, Apple, 1.99\n  | i2, Banana, 0.99\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
}

/// Ditto operator preserves type
#[test]
fn test_ditto_preserves_type() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id, ref, null_val, bool_val]\n---\ndata: @T\n  | a, @a, ~, true\n  | b, ^, ^, ^\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("data").unwrap().as_list().unwrap();

    // Row b should have copied values with their types
    // fields[0]=id, fields[1]=ref, fields[2]=null_val, fields[3]=bool_val
    assert!(matches!(list.rows[1].fields[1], Value::Reference(_)));
    assert_eq!(list.rows[1].fields[2], Value::Null);
    assert_eq!(list.rows[1].fields[3], Value::Bool(true));
}

/// Key-value ditto is string
#[test]
fn test_key_value_ditto_is_string() {
    let doc = "%VERSION: 1.0\n---\ncaret: ^\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let val = doc.get("caret").unwrap().as_scalar().unwrap();
    assert_eq!(*val, Value::String("^".to_string()));
}

/// Alias that expands to number
#[test]
fn test_alias_number_expansion() {
    let doc = "%VERSION: 1.0\n%ALIAS: %rate: \"1.23456\"\n%STRUCT: T: [id,value]\n---\ndata: @T\n  | x, %rate\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("data").unwrap().as_list().unwrap();
    // fields[0]=id, fields[1]=value
    match &list.rows[0].fields[1] {
        Value::Float(f) => assert!((f - 1.23456).abs() < 0.00001),
        _ => panic!("expected float from alias expansion"),
    }
}

/// Scientific notation is string (not number)
#[test]
fn test_scientific_notation_is_string() {
    let doc = "%VERSION: 1.0\n---\nvalue: 1e10\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let val = doc.get("value").unwrap().as_scalar().unwrap();
    assert_eq!(*val, Value::String("1e10".to_string()));
}

/// Underscore in numbers is string
#[test]
fn test_underscore_in_numbers_is_string() {
    let doc = "%VERSION: 1.0\n---\nvalue: 1_000\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let val = doc.get("value").unwrap().as_scalar().unwrap();
    assert_eq!(*val, Value::String("1_000".to_string()));
}

/// Leading zeros are allowed in numbers
#[test]
fn test_leading_zeros_in_numbers() {
    let doc = "%VERSION: 1.0\n---\nvalue: 001\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let val = doc.get("value").unwrap().as_scalar().unwrap();
    assert_eq!(*val, Value::Int(1));
}

/// Empty quoted string
#[test]
fn test_empty_quoted_string() {
    let doc = "%VERSION: 1.0\n---\nvalue: \"\"\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let val = doc.get("value").unwrap().as_scalar().unwrap();
    assert_eq!(*val, Value::String("".to_string()));
}

/// Multiple spaces after colon is OK
#[test]
fn test_multiple_spaces_after_colon() {
    let doc = "%VERSION: 1.0\n---\nvalue:   test\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let val = doc.get("value").unwrap().as_scalar().unwrap();
    assert_eq!(*val, Value::String("test".to_string()));
}

/// Blank lines are ignored
#[test]
fn test_blank_lines_ignored() {
    let doc = "%VERSION: 1.0\n\n---\n\na: 1\n\nb: 2\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(*doc.get("a").unwrap().as_scalar().unwrap(), Value::Int(1));
    assert_eq!(*doc.get("b").unwrap().as_scalar().unwrap(), Value::Int(2));
}

/// Comments are ignored
#[test]
fn test_comments_ignored() {
    let doc = "%VERSION: 1.0\n# header comment\n---\n# body comment\na: 1 # inline comment\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(*doc.get("a").unwrap().as_scalar().unwrap(), Value::Int(1));
}

/// VERSION must be first
#[test]
fn test_version_must_be_first() {
    let doc = "%STRUCT: T: [id]\n%VERSION: 1.0\n---\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
}

/// Duplicate key in object is error
#[test]
fn test_duplicate_object_key_error() {
    let doc = "%VERSION: 1.0\n---\na: 1\na: 2\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
}

// =============================================================================
// Block String Tests (""")
// =============================================================================

/// Single-line block string - MUST be rejected per SPEC Section 8.2
#[test]
fn test_block_string_single_line() {
    let doc = r#"%VERSION: 1.0
---
text: """hello world"""
"#;
    let result = parse(doc.as_bytes());
    assert!(
        result.is_err(),
        "Single-line block strings should be rejected per SPEC Section 8.2"
    );
    let err = result.unwrap_err();
    assert!(err.message.contains("newline after opening"));
}

/// Multi-line block string
#[test]
fn test_block_string_multiline() {
    let doc = r#"%VERSION: 1.0
---
text: """
This is line 1.
This is line 2.
"""
"#;
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let val = doc.get("text").unwrap().as_scalar().unwrap();
    let expected = "\nThis is line 1.\nThis is line 2.\n";
    assert_eq!(*val, Value::String(expected.to_string()));
}

/// Block string with internal quotes - single-line format rejected per SPEC Section 8.2
#[test]
fn test_block_string_preserves_quotes() {
    // This test now verifies multi-line format with internal quotes
    let doc = r#"%VERSION: 1.0
---
text: """
She said "hello" loudly
"""
"#;
    let result = parse(doc.as_bytes());
    assert!(result.is_ok(), "Multi-line block string should be accepted");
    let doc = result.unwrap();
    let val = doc.get("text").unwrap().as_scalar().unwrap();
    // Note: leading newline is preserved per SPEC
    assert_eq!(
        *val,
        Value::String("\nShe said \"hello\" loudly\n".to_string())
    );
}

/// Block string with blank lines
#[test]
fn test_block_string_with_blank_lines() {
    let doc = "%VERSION: 1.0\n---\ntext: \"\"\"\nPara 1\n\nPara 2\n\"\"\"\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let val = doc.get("text").unwrap().as_scalar().unwrap();
    assert!(val.as_str().unwrap().contains("\n\n"));
}

/// Block string followed by another key - using multi-line format per SPEC Section 8.2
#[test]
fn test_block_string_followed_by_key() {
    let doc = r#"%VERSION: 1.0
---
text: """
hello
"""
other: 42
"#;
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(
        *doc.get("text").unwrap().as_scalar().unwrap(),
        Value::String("\nhello\n".to_string())
    );
    assert_eq!(
        *doc.get("other").unwrap().as_scalar().unwrap(),
        Value::Int(42)
    );
}

/// Multiline block string followed by another key
#[test]
fn test_block_string_multiline_followed_by_key() {
    let doc = "%VERSION: 1.0\n---\ntext: \"\"\"\nline 1\nline 2\n\"\"\"\nother: 42\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    assert!(doc
        .get("text")
        .unwrap()
        .as_scalar()
        .unwrap()
        .as_str()
        .is_some());
    assert_eq!(
        *doc.get("other").unwrap().as_scalar().unwrap(),
        Value::Int(42)
    );
}

/// Unclosed block string is error
#[test]
fn test_block_string_unclosed_error() {
    let doc = "%VERSION: 1.0\n---\ntext: \"\"\"\nno closing\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
}

/// Block string with comment after closing quotes
#[test]
fn test_block_string_with_comment() {
    let doc = r#"%VERSION: 1.0
---
text: """
hello
""" # comment after closing is allowed
"#;
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let val = doc.get("text").unwrap().as_scalar().unwrap();
    assert_eq!(*val, Value::String("\nhello\n".to_string()));
}

// =============================================================================
// Elastic Alignment Tests (Internal Spacing)
// =============================================================================

/// Matrix rows with extra internal spacing for column alignment
#[test]
fn test_elastic_alignment_internal_spacing() {
    // Extra spaces within row content for visual column alignment
    let doc = r#"%VERSION: 1.0
%STRUCT: Point: [id,x,y]
---
points: @Point
  | p1,     1,     2
  | p2,    10,    20
  | p3,   100,   200
"#;
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("points").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 3);
    // Values should be correctly parsed despite extra spacing
    assert_eq!(list.rows[0].fields[1], Value::Int(1));
    assert_eq!(list.rows[1].fields[1], Value::Int(10));
    assert_eq!(list.rows[2].fields[1], Value::Int(100));
}

/// Normal matrix rows still work
#[test]
fn test_elastic_alignment_normal() {
    let doc = r#"%VERSION: 1.0
%STRUCT: Point: [id,x,y]
---
points: @Point
  | p1, 1, 2
  | p2, 3, 4
"#;
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("points").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
}

// Test SPEC 9.2: Escape sequences ARE processed in quoted matrix cell fields
// \n → newline, \t → tab, \r → CR, \\ → backslash, \" → quote
#[test]
fn test_escape_sequences_in_matrix_cells() {
    let doc = r#"%VERSION: 1.0
%STRUCT: Msg: [id, content]
---
msgs: @Msg
  | m1, "Hello\nWorld"
  | m2, "Tab\there"
  | m3, "Path\\to\\file"
"#;
    let result = parse(doc.as_bytes());
    assert!(result.is_ok(), "Parse failed: {:?}", result.err());
    let doc = result.unwrap();
    let list = doc.get("msgs").unwrap().as_list().unwrap();

    // m1: \n is converted to actual newline character
    if let Value::String(s) = &list.rows[0].fields[1] {
        assert!(
            s.contains('\n'),
            "Expected actual newline in m1 content, got: {:?}",
            s
        );
        assert_eq!(s, "Hello\nWorld");
    } else {
        panic!("Expected string for m1");
    }

    // m2: \t is converted to actual tab character
    if let Value::String(s) = &list.rows[1].fields[1] {
        assert!(
            s.contains('\t'),
            "Expected actual tab in m2 content, got: {:?}",
            s
        );
        assert_eq!(s, "Tab\there");
    } else {
        panic!("Expected string for m2");
    }

    // m3: \\ is converted to single backslash
    if let Value::String(s) = &list.rows[2].fields[1] {
        assert_eq!(
            s, "Path\\to\\file",
            "Expected single backslashes, got: {:?}",
            s
        );
    } else {
        panic!("Expected string for m3");
    }
}

// =============================================================================
// SPEC Compliance Tests - Section 8.2 and 14.5
// =============================================================================

/// Test that single-line block strings are rejected per SPEC Section 8.2
#[test]
fn test_spec_8_2_single_line_block_string_rejected() {
    let doc = r#"%VERSION: 1.0
---
text: """content on same line"""
"#;
    let result = parse(doc.as_bytes());
    assert!(
        result.is_err(),
        "Single-line block strings must be rejected per SPEC Section 8.2"
    );
    let err = result.unwrap_err();
    assert!(
        err.message.contains("newline after opening"),
        "Error message should mention newline requirement, got: {}",
        err.message
    );
}

/// Test that truncated object (no children) is detected per SPEC Section 14.5
#[test]
fn test_spec_14_5_truncated_object_detected() {
    let doc = r#"%VERSION: 1.0
---
config:
"#;
    let result = parse(doc.as_bytes());
    assert!(
        result.is_err(),
        "Truncated object (no children) must be detected per SPEC Section 14.5"
    );
    let err = result.unwrap_err();
    assert!(
        err.message.contains("truncated") || err.message.contains("no children"),
        "Error message should mention truncation, got: {}",
        err.message
    );
}

/// Test that complete objects are not flagged as truncated
#[test]
fn test_spec_14_5_complete_object_not_truncated() {
    let doc = r#"%VERSION: 1.0
---
config:
  port: 8080
"#;
    let result = parse(doc.as_bytes());
    assert!(
        result.is_ok(),
        "Complete objects should not be flagged as truncated: {:?}",
        result.err()
    );
}

/// Test that empty lists are allowed (not considered truncated)
#[test]
fn test_spec_14_5_empty_list_allowed() {
    let doc = r#"%VERSION: 1.0
%STRUCT: User: [id,name]
---
users: @User
"#;
    let result = parse(doc.as_bytes());
    assert!(
        result.is_ok(),
        "Empty lists should be allowed: {:?}",
        result.err()
    );
}

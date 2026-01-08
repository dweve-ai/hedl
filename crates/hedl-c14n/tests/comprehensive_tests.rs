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

//! Comprehensive tests for hedl-c14n canonicalization
//!
//! Tests for canonical output generation, ditto optimization, and round-trip stability.

use hedl_c14n::{canonicalize, canonicalize_with_config, CanonicalConfig};
use hedl_core::{parse, Document, Item, MatrixList, Node, Reference, Value};

// =============================================================================
// Basic Canonicalization Tests
// =============================================================================

#[test]
fn test_empty_document() {
    let doc = Document::new((1, 0));
    let output = canonicalize(&doc).unwrap();
    assert!(output.starts_with("%VERSION: 1.0"));
    assert!(output.contains("---"));
}

#[test]
fn test_simple_key_value() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "name".to_string(),
        Item::Scalar(Value::String("test".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("name: test"));
}

#[test]
fn test_version_output() {
    let doc = Document::new((1, 5));
    let output = canonicalize(&doc).unwrap();
    assert!(output.starts_with("%VERSION: 1.5"));
}

// =============================================================================
// Type Formatting Tests
// =============================================================================

#[test]
fn test_null_formatting() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Null));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("value: ~"));
}

#[test]
fn test_boolean_formatting() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("active".to_string(), Item::Scalar(Value::Bool(true)));
    doc.root
        .insert("inactive".to_string(), Item::Scalar(Value::Bool(false)));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("active: true"));
    assert!(output.contains("inactive: false"));
}

#[test]
fn test_integer_formatting() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("count".to_string(), Item::Scalar(Value::Int(42)));
    doc.root
        .insert("negative".to_string(), Item::Scalar(Value::Int(-100)));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("count: 42"));
    assert!(output.contains("negative: -100"));
}

#[test]
fn test_float_formatting() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("rate".to_string(), Item::Scalar(Value::Float(3.25)));
    doc.root
        .insert("whole".to_string(), Item::Scalar(Value::Float(42.0)));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("rate: 3.25"));
    // Whole float should have .0 to preserve type
    assert!(output.contains("whole: 42.0"));
}

#[test]
fn test_reference_formatting() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("target"))),
    );
    doc.root.insert(
        "qualified_ref".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("Type", "target"))),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("ref: @target"));
    assert!(output.contains("qualified_ref: @Type:target"));
}

#[test]
fn test_expression_formatting() {
    use hedl_core::Expression;
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "expr".to_string(),
        Item::Scalar(Value::Expression(Expression::Call {
            name: "add".to_string(),
            args: vec![
                Expression::Identifier { name: "x".to_string(), span: Default::default() },
                Expression::Identifier { name: "y".to_string(), span: Default::default() },
            ],
            span: Default::default(),
        })),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("expr: $(add(x, y))"));
}

// =============================================================================
// String Quoting Tests
// =============================================================================

#[test]
fn test_simple_string_no_quote() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "name".to_string(),
        Item::Scalar(Value::String("hello".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("name: hello"));
}

#[test]
fn test_string_with_hash_quoted() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "value".to_string(),
        Item::Scalar(Value::String("test#comment".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("\"test#comment\""));
}

#[test]
fn test_string_with_whitespace_quoted() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "value".to_string(),
        Item::Scalar(Value::String("  spaces  ".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("\"  spaces  \""));
}

#[test]
fn test_string_with_embedded_quote_escaped() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "value".to_string(),
        Item::Scalar(Value::String("say \"hi\"".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("\"say \"\"hi\"\"\""));
}

#[test]
fn test_numeric_string_quoted() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "code".to_string(),
        Item::Scalar(Value::String("123".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    // Numeric strings should be quoted to prevent inference
    assert!(output.contains("\"123\""));
}

#[test]
fn test_boolean_string_quoted() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "label".to_string(),
        Item::Scalar(Value::String("true".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    // Boolean strings should be quoted to prevent inference
    assert!(output.contains("\"true\""));
}

// =============================================================================
// Directive Order Tests
// =============================================================================

#[test]
fn test_alias_order() {
    let mut doc = Document::new((1, 0));
    doc.aliases.insert("zebra".to_string(), "z".to_string());
    doc.aliases.insert("apple".to_string(), "a".to_string());
    doc.aliases.insert("mango".to_string(), "m".to_string());

    let output = canonicalize(&doc).unwrap();
    let apple_pos = output.find("%ALIAS: %apple").unwrap();
    let mango_pos = output.find("%ALIAS: %mango").unwrap();
    let zebra_pos = output.find("%ALIAS: %zebra").unwrap();

    assert!(apple_pos < mango_pos);
    assert!(mango_pos < zebra_pos);
}

#[test]
fn test_struct_order() {
    let mut doc = Document::new((1, 0));
    doc.structs
        .insert("Zebra".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Apple".to_string(), vec!["id".to_string()]);

    // Structs are only output when inline_schemas is false
    let config = CanonicalConfig::new().with_inline_schemas(false);
    let output = canonicalize_with_config(&doc, &config).unwrap();
    let apple_pos = output.find("%STRUCT: Apple:").unwrap();
    let zebra_pos = output.find("%STRUCT: Zebra:").unwrap();

    assert!(apple_pos < zebra_pos);
}

// =============================================================================
// Object Nesting Tests
// =============================================================================

#[test]
fn test_nested_object() {
    let mut doc = Document::new((1, 0));
    let mut inner = std::collections::BTreeMap::new();
    inner.insert(
        "key".to_string(),
        Item::Scalar(Value::String("value".to_string())),
    );
    doc.root.insert("outer".to_string(), Item::Object(inner));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("outer:"));
    assert!(output.contains("  key: value"));
}

#[test]
fn test_key_sorting() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("zebra".to_string(), Item::Scalar(Value::Int(3)));
    doc.root
        .insert("apple".to_string(), Item::Scalar(Value::Int(1)));
    doc.root
        .insert("banana".to_string(), Item::Scalar(Value::Int(2)));

    let config = CanonicalConfig::new().with_sort_keys(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    let apple_pos = output.find("apple:").unwrap();
    let banana_pos = output.find("banana:").unwrap();
    let zebra_pos = output.find("zebra:").unwrap();

    assert!(apple_pos < banana_pos);
    assert!(banana_pos < zebra_pos);
}

// =============================================================================
// Matrix List Tests
// =============================================================================

#[test]
fn test_matrix_list_output() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    list.add_row(Node::new(
        "User",
        "u1",
        vec![
            Value::String("u1".to_string()),
            Value::String("Alice".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "User",
        "u2",
        vec![
            Value::String("u2".to_string()),
            Value::String("Bob".to_string()),
        ],
    ));

    doc.root.insert("users".to_string(), Item::List(list));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("users: @User"));
    assert!(output.contains("|u1,Alice"));
    assert!(output.contains("|u2,Bob"));
}

#[test]
fn test_matrix_list_inline_schema() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Int(1)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    assert!(output.contains("items: @Item[id,value]"));
}

// =============================================================================
// Ditto Optimization Tests
// =============================================================================

#[test]
fn test_ditto_optimization() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "category".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("fruit".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![
            Value::String("i2".to_string()),
            Value::String("fruit".to_string()),
        ],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_ditto(true).with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Second row should use ditto for category
    assert!(output.contains("|i2,^"));
}

#[test]
fn test_no_ditto_in_id_column() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "same",
        vec![Value::String("same".to_string()), Value::Int(1)],
    ));
    list.add_row(Node::new(
        "Item",
        "same2",
        vec![Value::String("same2".to_string()), Value::Int(1)],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_ditto(true).with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // ID column should NOT use ditto (even if values were same)
    assert!(output.contains("|same,"));
    assert!(output.contains("|same2,"));
}

#[test]
fn test_no_ditto_first_row() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Int(42)],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_ditto(true).with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // First row never has ditto
    assert!(output.contains("|i1,42"));
    assert!(!output.contains("^"));
}

#[test]
fn test_ditto_deep_equality() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "active".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Bool(true)],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![Value::String("i2".to_string()), Value::Bool(true)],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_ditto(true).with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Boolean values should ditto
    assert!(output.contains("|i2,^"));
}

#[test]
fn test_ditto_disabled() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("same".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![
            Value::String("i2".to_string()),
            Value::String("same".to_string()),
        ],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_ditto(false).with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // No ditto when disabled
    assert!(!output.contains("^"));
    assert!(output.contains("|i2,same"));
}

// =============================================================================
// Round-Trip Tests
// =============================================================================

#[test]
fn test_round_trip_simple() {
    let input = r#"%VERSION: 1.0
---
name: test
count: 42
"#;
    let doc = parse(input.as_bytes()).unwrap();
    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    assert_eq!(
        doc.root.get("name").unwrap().as_scalar().unwrap(),
        doc2.root.get("name").unwrap().as_scalar().unwrap()
    );
    assert_eq!(
        doc.root.get("count").unwrap().as_scalar().unwrap(),
        doc2.root.get("count").unwrap().as_scalar().unwrap()
    );
}

#[test]
fn test_round_trip_matrix_list() {
    let input = r#"%VERSION: 1.0
%STRUCT: User: [id,name]
---
users: @User
  | u1, Alice
  | u2, Bob
"#;
    let doc = parse(input.as_bytes()).unwrap();
    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    let list1 = doc.root.get("users").unwrap().as_list().unwrap();
    let list2 = doc2.root.get("users").unwrap().as_list().unwrap();

    assert_eq!(list1.rows.len(), list2.rows.len());
    assert_eq!(list1.rows[0].id, list2.rows[0].id);
}

// =============================================================================
// Tensor Formatting Tests
// =============================================================================

#[test]
fn test_tensor_formatting() {
    let mut doc = Document::new((1, 0));
    use hedl_core::Tensor;

    let tensor = Tensor::Array(vec![
        Tensor::Scalar(1.0),
        Tensor::Scalar(2.0),
        Tensor::Scalar(3.0),
    ]);
    doc.root
        .insert("data".to_string(), Item::Scalar(Value::Tensor(tensor)));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("data: [1.0, 2.0, 3.0]"));
}

#[test]
fn test_nested_tensor_formatting() {
    let mut doc = Document::new((1, 0));
    use hedl_core::Tensor;

    let tensor = Tensor::Array(vec![
        Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
        Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
    ]);
    doc.root
        .insert("matrix".to_string(), Item::Scalar(Value::Tensor(tensor)));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("[[1.0, 2.0], [3.0, 4.0]]"));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_string_value() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "empty".to_string(),
        Item::Scalar(Value::String("".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    // Empty string needs quoting
    assert!(output.contains("empty: \"\""));
}

#[test]
fn test_at_sign_in_middle_not_quoted() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "email".to_string(),
        Item::Scalar(Value::String("user@example.com".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    // @ in middle of string does NOT need quoting - only @ at start triggers reference parsing
    assert!(output.contains("email: user@example.com"));
}

#[test]
fn test_at_sign_at_start_quoted() {
    let mut doc = Document::new((1, 0));
    // String that starts with @ should be quoted to prevent reference interpretation
    doc.root.insert(
        "label".to_string(),
        Item::Scalar(Value::String("@handle".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("\"@handle\""));
}

// =============================================================================
// Empty Trailing Cell Tests (Section 13.2 of SPEC.md)
// =============================================================================

#[test]
fn test_empty_middle_column() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec!["id".to_string(), "middle".to_string(), "last".to_string()],
    );
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("".to_string()), // Empty middle column
            Value::String("value".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Empty middle column should NOT have quotes (just empty field)
    assert!(output.contains("|i1,,value"));
}

#[test]
fn test_empty_last_column() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec!["id".to_string(), "value".to_string(), "last".to_string()],
    );
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("data".to_string()),
            Value::String("".to_string()), // Empty last column
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Empty last column MUST have quotes to avoid trailing comma syntax error
    // Per SPEC.md Section 13.2
    assert!(output.contains("|i1,data,\"\""));
}

#[test]
fn test_empty_last_column_only_two_columns() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "last".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("".to_string()), // Empty last column (second column)
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Empty last column must be quoted
    assert!(output.contains("|i1,\"\""));
}

#[test]
fn test_multiple_empty_columns() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec![
            "id".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ],
    );
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("".to_string()), // Empty middle
            Value::String("".to_string()), // Empty middle
            Value::String("".to_string()), // Empty last
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Middle empties: no quotes. Last empty: must have quotes
    assert!(output.contains("|i1,,,\"\""));
}

#[test]
fn test_empty_last_column_with_ditto() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec!["id".to_string(), "value".to_string(), "last".to_string()],
    );
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("data".to_string()),
            Value::String("".to_string()), // Empty last column
        ],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![
            Value::String("i2".to_string()),
            Value::String("data".to_string()),
            Value::String("".to_string()), // Empty last column (same as previous)
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true).with_ditto(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // First row: empty last column must be quoted
    assert!(output.contains("|i1,data,\"\""));
    // Second row: should use ditto for last column since both are empty strings
    assert!(output.contains("|i2,^,^"));
}

#[test]
fn test_empty_trailing_cell_round_trip() {
    // This test verifies that the fix for empty trailing cells works end-to-end
    // per SPEC.md Section 13.2
    let input = r#"%VERSION: 1.0
%STRUCT: TestItem: [id, middle, last]
---
items: @TestItem
  | i1, , ""
  | i2, value, ""
  | i3, , data
"#;

    // Parse the input
    let doc1 = parse(input.as_bytes()).unwrap();

    // Canonicalize it
    let canonical = canonicalize(&doc1).unwrap();

    // Verify the canonical output has proper empty last column handling
    // Row 1: empty middle and empty last - last must be quoted as ""
    assert!(
        canonical.contains("|i1,,\"\""),
        "Empty middle and empty last"
    );
    // Row 2: uses ditto for last column (both i1 and i2 have empty last)
    assert!(
        canonical.contains("|i2,value,^"),
        "Non-empty middle, ditto for empty last"
    );
    // Row 3: empty middle and non-empty last - no quotes needed for last
    assert!(
        canonical.contains("|i3,,data"),
        "Empty middle and non-empty last"
    );

    // Parse the canonical output (round-trip)
    let doc2 = parse(canonical.as_bytes()).unwrap();

    // Verify the data is preserved
    let list1 = doc1.root.get("items").unwrap().as_list().unwrap();
    let list2 = doc2.root.get("items").unwrap().as_list().unwrap();

    assert_eq!(list1.rows.len(), 3);
    assert_eq!(list2.rows.len(), 3);

    // Verify each row preserved correctly
    for i in 0..3 {
        assert_eq!(list1.rows[i].fields.len(), list2.rows[i].fields.len());
        for j in 0..list1.rows[i].fields.len() {
            assert_eq!(
                list1.rows[i].fields[j], list2.rows[i].fields[j],
                "Mismatch at row {} column {}",
                i, j
            );
        }
    }
}

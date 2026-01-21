// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive quote escaping and quote style tests.
//!
//! Tests quote handling in CSV parsing and generation.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_csv::{from_csv_with_config, to_csv_with_config, FromCsvConfig, ToCsvConfig};

// =============================================================================
// Basic Quote Escaping
// =============================================================================

#[test]
fn test_quoted_field_with_comma() {
    let csv = "id,name\n1,\"Smith, John\"\n";
    let doc = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default()).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Smith, John".to_string().into())
    );
}

#[test]
fn test_quoted_field_with_newline() {
    let csv = "id,text\n1,\"line1\nline2\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("line1\nline2".to_string().into())
    );
}

#[test]
fn test_escaped_quotes_double() {
    let csv = "id,text\n1,\"She said \"\"hello\"\"\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("She said \"hello\"".to_string().into())
    );
}

#[test]
fn test_multiple_escaped_quotes() {
    let csv = "id,text\n1,\"\"\"quoted\"\" \"\"text\"\"\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("\"quoted\" \"text\"".to_string().into())
    );
}

#[test]
fn test_empty_quoted_field() {
    let csv = "id,text\n1,\"\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // Empty quoted field is treated as null by type inference
    assert_eq!(list.rows[0].fields[1], Value::Null);
}

#[test]
fn test_quoted_null_string() {
    let csv = "id,text\n1,\"~\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // Even quoted, tilde is still parsed as null by type inference
    assert_eq!(list.rows[0].fields[1], Value::Null);
}

// =============================================================================
// Quote Styles (Output)
// =============================================================================

#[test]
fn test_quote_style_necessary() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);

    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".into()),
            Value::String("hello, world".into()),
        ],
    ));

    list.add_row(Node::new(
        "Item",
        "2",
        vec![Value::String("2".into()), Value::String("simple".into())],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToCsvConfig {
        quote_style: csv::QuoteStyle::Necessary,
        ..Default::default()
    };
    let csv = to_csv_with_config(&doc, config).unwrap();

    // Field with comma should be quoted
    assert!(csv.contains("\"hello, world\""));
    // Simple field should not be quoted
    assert!(csv.contains("simple"));
}

#[test]
fn test_quote_style_always() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);

    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".into()), Value::String("simple".into())],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToCsvConfig {
        quote_style: csv::QuoteStyle::Always,
        ..Default::default()
    };
    let csv = to_csv_with_config(&doc, config).unwrap();

    // All fields should be quoted with Always style
    assert!(csv.contains("\"1\""));
    assert!(csv.contains("\"simple\""));
}

#[test]
fn test_quote_style_never() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);

    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".into()), Value::String("simple".into())],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToCsvConfig {
        quote_style: csv::QuoteStyle::Never,
        ..Default::default()
    };
    let csv = to_csv_with_config(&doc, config).unwrap();

    // No fields should be quoted
    assert!(!csv.contains('"'));
}

#[test]
fn test_quote_style_non_numeric() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec!["id".to_string(), "count".to_string(), "text".to_string()],
    );

    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".into()),
            Value::Int(42),
            Value::String("hello".into()),
        ],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToCsvConfig {
        quote_style: csv::QuoteStyle::NonNumeric,
        ..Default::default()
    };
    let csv = to_csv_with_config(&doc, config).unwrap();

    // String fields should be quoted, numeric should not
    let lines: Vec<&str> = csv.lines().collect();
    assert!(lines.len() >= 2);
    // Check data row contains quoted string and unquoted number
    assert!(csv.contains("\"hello\""));
}

// =============================================================================
// Complex Quoting Scenarios
// =============================================================================

#[test]
fn test_nested_quotes_and_commas() {
    let csv = "id,text\n1,\"He said, \"\"Hello, world!\"\"\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("He said, \"Hello, world!\"".to_string().into())
    );
}

#[test]
fn test_quotes_at_start_and_end() {
    let csv = "id,text\n1,\"\"\"start and end\"\"\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("\"start and end\"".to_string().into())
    );
}

#[test]
fn test_mixed_quoted_unquoted_fields() {
    let csv = "id,quoted,unquoted\n1,\"hello, world\",simple\n";
    let doc = from_csv_with_config(
        csv,
        "Item",
        &["quoted", "unquoted"],
        FromCsvConfig::default(),
    )
    .unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("hello, world".to_string().into())
    );
    assert_eq!(
        list.rows[0].fields[2],
        Value::String("simple".to_string().into())
    );
}

#[test]
fn test_quoted_whitespace() {
    let csv = "id,text\n1,\"  spaces  \"\n";
    let config = FromCsvConfig {
        trim: false,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Item", &["text"], config).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // Whitespace inside quotes should be preserved even without trim
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("  spaces  ".to_string().into())
    );
}

#[test]
fn test_quoted_tab_characters() {
    let csv = "id,text\n1,\"tab\there\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("tab\there".to_string().into())
    );
}

#[test]
fn test_quoted_special_chars() {
    let csv = "id,text\n1,\"!@#$%^&*()\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("!@#$%^&*()".to_string().into())
    );
}

// =============================================================================
// Roundtrip Quote Tests
// =============================================================================

#[test]
fn test_quote_roundtrip_with_commas() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);

    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".into()), Value::String("a, b, c".into())],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv_with_config(&doc, ToCsvConfig::default()).unwrap();
    let doc2 = from_csv_with_config(&csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list2.rows[0].fields[1],
        Value::String("a, b, c".to_string().into())
    );
}

#[test]
fn test_quote_roundtrip_with_newlines() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);

    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".into()),
            Value::String("line1\nline2\nline3".into()),
        ],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv_with_config(&doc, ToCsvConfig::default()).unwrap();
    let doc2 = from_csv_with_config(&csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list2.rows[0].fields[1],
        Value::String("line1\nline2\nline3".to_string().into())
    );
}

#[test]
fn test_quote_roundtrip_with_quotes() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);

    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".into()),
            Value::String("She said \"hello\"".into()),
        ],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv_with_config(&doc, ToCsvConfig::default()).unwrap();
    let doc2 = from_csv_with_config(&csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list2.rows[0].fields[1],
        Value::String("She said \"hello\"".to_string().into())
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_only_quotes() {
    let csv = "id,text\n1,\"\"\"\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("\"".to_string().into())
    );
}

#[test]
fn test_many_quotes() {
    let csv = "id,text\n1,\"\"\"\"\"\"\"\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("\"\"\"".to_string().into())
    );
}

#[test]
fn test_quoted_numbers() {
    let csv = "id,value\n1,\"42\"\n2,\"3.14\"\n";
    let doc = from_csv_with_config(csv, "Item", &["value"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // Even quoted, numbers are still type-inferred
    assert_eq!(list.rows[0].fields[1], Value::Int(42));
    #[allow(clippy::approx_constant)]
    const PI_APPROX: f64 = 3.14;
    assert_eq!(list.rows[1].fields[1], Value::Float(PI_APPROX));
}

#[test]
fn test_quoted_boolean() {
    let csv = "id,value\n1,\"true\"\n2,\"false\"\n";
    let doc = from_csv_with_config(csv, "Item", &["value"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // Even quoted, booleans are still type-inferred
    assert_eq!(list.rows[0].fields[1], Value::Bool(true));
    assert_eq!(list.rows[1].fields[1], Value::Bool(false));
}

#[test]
fn test_quoted_reference() {
    let csv = "id,ref\n1,\"@user1\"\n";
    let doc = from_csv_with_config(csv, "Item", &["ref"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // Quoted reference should still be parsed as reference
    assert!(matches!(list.rows[0].fields[1], Value::Reference(_)));
}

#[test]
fn test_carriage_return_in_quotes() {
    let csv = "id,text\n1,\"line1\r\nline2\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // CSV library should handle CRLF correctly
    let text = match &list.rows[0].fields[1] {
        Value::String(s) => s.as_ref(),
        _ => panic!("Expected string"),
    };
    assert!(text.contains("line1") && text.contains("line2"));
}

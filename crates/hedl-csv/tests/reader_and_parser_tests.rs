// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Reader variant and parser edge case tests.
//!
//! Tests `from_csv_reader` functions and various parser edge cases.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_csv::{
    from_csv_reader, from_csv_reader_with_config, from_csv_with_config, to_csv_list,
    to_csv_list_writer, FromCsvConfig,
};
use std::io::Cursor;

// =============================================================================
// from_csv_reader Tests
// =============================================================================

#[test]
fn test_from_csv_reader_basic() {
    let csv = "id,name,age\n1,Alice,30\n2,Bob,25\n";
    let reader = Cursor::new(csv);

    let doc = from_csv_reader(reader, "Person", &["name", "age"]).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Alice".to_string().into())
    );
}

#[test]
fn test_from_csv_reader_with_config() {
    let csv = "id\tname\tage\n1\tAlice\t30\n";
    let reader = Cursor::new(csv);

    let config = FromCsvConfig {
        delimiter: b'\t',
        ..Default::default()
    };

    let doc = from_csv_reader_with_config(reader, "Person", &["name", "age"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1);
}

#[test]
fn test_from_csv_reader_empty() {
    let csv = "";
    let reader = Cursor::new(csv);

    let result = from_csv_reader(reader, "Person", &["name"]);
    // Empty CSV may succeed with empty list or error
    let _ = result;
}

#[test]
fn test_from_csv_reader_large() {
    let mut csv = String::from("id,value\n");
    for i in 1..=1000 {
        csv.push_str(&format!("{},{}\n", i, i * 10));
    }
    let reader = Cursor::new(csv);

    let doc = from_csv_reader(reader, "Item", &["value"]).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1000);
}

#[test]
fn test_from_csv_reader_bytes() {
    let csv = b"id,name\n1,Alice\n";
    let reader = Cursor::new(csv.as_slice());

    let doc = from_csv_reader(reader, "Person", &["name"]).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1);
}

// =============================================================================
// Writer Tests
// =============================================================================

#[test]
fn test_to_csv_list_writer_basic() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Person", vec!["id".to_string(), "name".to_string()]);

    list.add_row(Node::new(
        "Person",
        "1",
        vec![Value::String("1".into()), Value::String("Alice".into())],
    ));

    doc.root.insert("people".to_string(), Item::List(list));

    let mut buffer = Vec::new();
    to_csv_list_writer(&doc, "people", &mut buffer).unwrap();

    let csv = String::from_utf8(buffer).unwrap();
    assert!(csv.contains("Alice"));
}

#[test]
fn test_to_csv_list_writer_large() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    for i in 1..=1000 {
        let id_str = i.to_string();
        list.add_row(Node::new(
            "Item",
            &id_str,
            vec![Value::String(id_str.clone().into()), Value::Int(i)],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));

    let mut buffer = Vec::new();
    to_csv_list_writer(&doc, "items", &mut buffer).unwrap();

    let csv = String::from_utf8(buffer).unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 1001); // header + 1000 rows
}

// =============================================================================
// Parser Edge Cases
// =============================================================================

#[test]
fn test_single_column_csv() {
    let csv = "id\n1\n2\n3\n";
    let doc = from_csv_with_config(csv, "Item", &[], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 3);
    assert_eq!(list.schema.len(), 1); // Only id column
}

#[test]
fn test_single_row_csv() {
    let csv = "id,name,age\n1,Alice,30\n";
    let doc =
        from_csv_with_config(csv, "Person", &["name", "age"], FromCsvConfig::default()).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1);
}

#[test]
fn test_single_cell_csv() {
    let csv = "id\n1\n";
    let doc = from_csv_with_config(csv, "Item", &[], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1);
}

#[test]
fn test_headers_only_no_data() {
    let csv = "id,name,age\n";
    let doc =
        from_csv_with_config(csv, "Person", &["name", "age"], FromCsvConfig::default()).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 0);
}

#[test]
fn test_blank_lines_in_data() {
    let csv = "id,name\n1,Alice\n\n2,Bob\n";
    let result = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default());

    // CSV library should handle blank lines (may skip or include)
    // Just ensure it doesn't panic
    let _ = result;
}

#[test]
fn test_whitespace_only_row() {
    let csv = "id,name\n1,Alice\n   \n2,Bob\n";
    let result = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default());

    // Should handle gracefully
    let _ = result;
}

#[test]
fn test_comment_like_lines() {
    let csv = "id,name\n# comment\n1,Alice\n";
    let result = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default());

    // CSV format doesn't support comments, so this should error or include as data
    let _ = result;
}

// =============================================================================
// Type Inference Edge Cases
// =============================================================================

#[test]
fn test_leading_zeros_not_octal() {
    let csv = "id,code\n1,0123\n";
    let doc = from_csv_with_config(csv, "Item", &["code"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // Leading zeros: could be int 123 or string "0123"
    // Current implementation should treat as int
    assert!(matches!(
        list.rows[0].fields[1],
        Value::Int(123) | Value::String(_)
    ));
}

#[test]
fn test_scientific_notation() {
    let csv = "id,value\n1,1.5e10\n2,3.14E-5\n";
    let doc = from_csv_with_config(csv, "Item", &["value"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert!(matches!(list.rows[0].fields[1], Value::Float(_)));
    assert!(matches!(list.rows[1].fields[1], Value::Float(_)));
}

#[test]
fn test_hexadecimal_not_parsed() {
    let csv = "id,value\n1,0xFF\n";
    let doc = from_csv_with_config(csv, "Item", &["value"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // Hex should be treated as string, not parsed
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("0xFF".to_string().into())
    );
}

#[test]
fn test_plus_sign_in_number() {
    let csv = "id,value\n1,+42\n2,+3.14\n";
    let doc = from_csv_with_config(csv, "Item", &["value"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // Plus sign may or may not be accepted
    let _ = list.rows[0].fields[1].clone();
}

#[test]
fn test_underscore_in_number() {
    let csv = "id,value\n1,1_000_000\n";
    let doc = from_csv_with_config(csv, "Item", &["value"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // Underscores not valid in standard number format, should be string
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("1_000_000".to_string().into())
    );
}

#[test]
fn test_very_large_integer() {
    let csv = "id,value\n1,999999999999999999\n";
    let doc = from_csv_with_config(csv, "Item", &["value"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // Should parse as i64
    assert!(matches!(list.rows[0].fields[1], Value::Int(_)));
}

#[test]
fn test_integer_overflow() {
    let csv = "id,value\n1,99999999999999999999\n";
    let doc = from_csv_with_config(csv, "Item", &["value"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // Too large for i64: may parse as float or string depending on implementation
    let _ = list.rows[0].fields[1].clone();
}

#[test]
fn test_float_precision() {
    let csv = "id,value\n1,3.141592653589793238\n";
    let doc = from_csv_with_config(csv, "Item", &["value"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert!(matches!(list.rows[0].fields[1], Value::Float(_)));
}

// =============================================================================
// Special Value Tests
// =============================================================================

#[test]
fn test_null_representations() {
    let csv = "id,a,b,c,d\n1,~,,null,NULL\n";
    let doc =
        from_csv_with_config(csv, "Item", &["a", "b", "c", "d"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // ~ and empty should be null
    assert_eq!(list.rows[0].fields[1], Value::Null);
    assert_eq!(list.rows[0].fields[2], Value::Null);
    // "null" and "NULL" should be strings (not special)
    assert_eq!(
        list.rows[0].fields[3],
        Value::String("null".to_string().into())
    );
    assert_eq!(
        list.rows[0].fields[4],
        Value::String("NULL".to_string().into())
    );
}

#[test]
fn test_boolean_case_sensitivity() {
    let csv = "id,a,b,c,d\n1,true,True,TRUE,false\n2,False,FALSE,tRuE,fAlSe\n";
    let doc =
        from_csv_with_config(csv, "Item", &["a", "b", "c", "d"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // Only lowercase "true" and "false" should be booleans
    assert_eq!(list.rows[0].fields[1], Value::Bool(true));
    assert_eq!(list.rows[0].fields[4], Value::Bool(false));
}

#[test]
fn test_reference_formats() {
    let csv = "id,ref1,ref2,ref3\n1,@user1,@User:user2,@Type:id:subid\n";
    let doc = from_csv_with_config(
        csv,
        "Item",
        &["ref1", "ref2", "ref3"],
        FromCsvConfig::default(),
    )
    .unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    // All @ prefixed values should be references
    assert!(matches!(list.rows[0].fields[1], Value::Reference(_)));
    assert!(matches!(list.rows[0].fields[2], Value::Reference(_)));
    // Complex reference might not parse
    let _ = list.rows[0].fields[3].clone();
}

#[test]
fn test_expression_syntax() {
    let csv = "id,expr1\n1,$(x)\n";
    let doc = from_csv_with_config(csv, "Item", &["expr1"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert!(matches!(list.rows[0].fields[1], Value::Expression(_)));
}

#[test]
fn test_invalid_expression() {
    let csv = "id,expr\n1,$(unclosed\n";
    let result = from_csv_with_config(csv, "Item", &["expr"], FromCsvConfig::default());

    // Invalid expression should error or fallback to string
    let _ = result;
}

// =============================================================================
// Custom List Key Tests
// =============================================================================

#[test]
fn test_custom_list_key_people() {
    let csv = "id,name\n1,Alice\n";
    let config = FromCsvConfig {
        list_key: Some("people".to_string()),
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name"], config).unwrap();

    assert!(doc.get("people").is_some());
    assert!(doc.get("persons").is_none());
}

#[test]
fn test_custom_list_key_dataset() {
    let csv = "id,value\n1,42\n";
    let config = FromCsvConfig {
        list_key: Some("dataset".to_string()),
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Data", &["value"], config).unwrap();

    assert!(doc.get("dataset").is_some());
    assert!(doc.get("datas").is_none());
}

#[test]
fn test_custom_list_key_case_sensitive() {
    let csv = "id,name\n1,test\n";
    let config = FromCsvConfig {
        list_key: Some("MyCustomList".to_string()),
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Item", &["name"], config).unwrap();

    assert!(doc.get("MyCustomList").is_some());
    assert!(doc.get("mycustomlist").is_none());
}

#[test]
fn test_list_key_roundtrip() {
    let csv = "id,name\n1,Alice\n";
    let config = FromCsvConfig {
        list_key: Some("people".to_string()),
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name"], config).unwrap();

    let csv_out = to_csv_list(&doc, "people").unwrap();
    assert!(csv_out.contains("Alice"));
}

// =============================================================================
// Whitespace and Trimming Tests
// =============================================================================

#[test]
fn test_leading_whitespace_with_trim() {
    let csv = "id,name\n1,  Alice\n";
    let config = FromCsvConfig {
        trim: true,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Alice".to_string().into())
    );
}

#[test]
fn test_trailing_whitespace_with_trim() {
    let csv = "id,name\n1,Alice  \n";
    let config = FromCsvConfig {
        trim: true,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Alice".to_string().into())
    );
}

#[test]
fn test_whitespace_preserved_without_trim() {
    let csv = "id,name\n1,  Alice  \n";
    let config = FromCsvConfig {
        trim: false,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    let name = match &list.rows[0].fields[1] {
        Value::String(s) => s.as_ref(),
        _ => panic!("Expected string"),
    };
    assert!(name.starts_with(' ') && name.ends_with(' '));
}

#[test]
fn test_tab_whitespace_with_trim() {
    let csv = "id,name\n1,\t\tAlice\t\t\n";
    let config = FromCsvConfig {
        trim: true,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Alice".to_string().into())
    );
}

#[test]
fn test_mixed_whitespace_chars() {
    let csv = "id,name\n1,  \t Alice \t  \n";
    let config = FromCsvConfig {
        trim: true,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Alice".to_string().into())
    );
}

// =============================================================================
// No Headers Mode Tests
// =============================================================================

#[test]
fn test_no_headers_basic() {
    let csv = "1,Alice,30\n2,Bob,25\n";
    let config = FromCsvConfig {
        has_headers: false,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name", "age"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
}

#[test]
fn test_no_headers_single_column() {
    let csv = "1\n2\n3\n";
    let config = FromCsvConfig {
        has_headers: false,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Item", &[], config).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 3);
}

#[test]
fn test_no_headers_many_columns() {
    let csv = "1,a,b,c,d\n2,e,f,g,h\n";
    let config = FromCsvConfig {
        has_headers: false,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Item", &["col1", "col2", "col3", "col4"], config).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
}

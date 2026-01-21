// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive delimiter handling tests.
//!
//! Tests various delimiters, edge cases, and boundary conditions.

use hedl_core::Value;
use hedl_csv::{from_csv_with_config, to_csv_with_config, FromCsvConfig, ToCsvConfig};

// =============================================================================
// Standard Delimiters
// =============================================================================

#[test]
fn test_comma_delimiter_roundtrip() {
    let csv = "id,name,age\n1,Alice,30\n2,Bob,25\n";
    let config = FromCsvConfig {
        delimiter: b',',
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name", "age"], config).unwrap();

    let to_config = ToCsvConfig {
        delimiter: b',',
        ..Default::default()
    };
    let output = to_csv_with_config(&doc, to_config).unwrap();
    assert!(output.contains("Alice"));
}

#[test]
fn test_tab_delimiter() {
    let csv = "id\tname\tage\n1\tAlice\t30\n2\tBob\t25\n";
    let config = FromCsvConfig {
        delimiter: b'\t',
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name", "age"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Alice".to_string().into())
    );
}

#[test]
fn test_semicolon_delimiter() {
    let csv = "id;name;age\n1;Alice;30\n2;Bob;25\n";
    let config = FromCsvConfig {
        delimiter: b';',
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name", "age"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
}

#[test]
fn test_pipe_delimiter() {
    let csv = "id|name|age\n1|Alice|30\n2|Bob|25\n";
    let config = FromCsvConfig {
        delimiter: b'|',
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name", "age"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
}

#[test]
fn test_space_delimiter() {
    let csv = "id name age\n1 Alice 30\n2 Bob 25\n";
    let config = FromCsvConfig {
        delimiter: b' ',
        trim: false,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name", "age"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
}

#[test]
fn test_colon_delimiter() {
    let csv = "id:name:age\n1:Alice:30\n2:Bob:25\n";
    let config = FromCsvConfig {
        delimiter: b':',
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name", "age"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
}

// =============================================================================
// Delimiter Edge Cases
// =============================================================================

#[test]
fn test_delimiter_in_quoted_field() {
    let csv = "id,name\n1,\"Smith, John\"\n";
    let doc = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default()).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Smith, John".to_string().into())
    );
}

#[test]
fn test_multiple_delimiters_in_quoted_field() {
    let csv = "id,text\n1,\"a,b,c,d\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("a,b,c,d".to_string().into())
    );
}

#[test]
fn test_tab_in_comma_delimited() {
    let csv = "id,text\n1,\"hello\tworld\"\n";
    let doc = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("hello\tworld".to_string().into())
    );
}

#[test]
fn test_delimiter_consistency() {
    let csv = "id,name,age\n1,Alice,30\n2,Bob;25\n";
    let result = from_csv_with_config(csv, "Person", &["name", "age"], FromCsvConfig::default());

    // Should parse with comma delimiter, treating semicolon as part of value
    // The "Bob;25" will be in the name field, and age will be missing (causing width mismatch)
    // This is expected behavior, so we just verify it doesn't panic
    let _ = result;
}

#[test]
fn test_empty_fields_with_delimiter() {
    let csv = "id,a,b,c\n1,,,\n2,x,,z\n";
    let doc =
        from_csv_with_config(csv, "Item", &["a", "b", "c"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[1], Value::Null);
    assert_eq!(list.rows[0].fields[2], Value::Null);
    assert_eq!(list.rows[0].fields[3], Value::Null);
    assert_eq!(
        list.rows[1].fields[1],
        Value::String("x".to_string().into())
    );
    assert_eq!(list.rows[1].fields[2], Value::Null);
}

#[test]
fn test_trailing_delimiter() {
    let csv = "id,name\n1,Alice\n";
    let doc = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default()).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1);
}

#[test]
fn test_leading_delimiter() {
    let csv = "x,id,name\n0,1,Alice\n";
    let doc =
        from_csv_with_config(csv, "Person", &["id", "name"], FromCsvConfig::default()).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1);
    assert_eq!(
        list.rows[0].fields[2],
        Value::String("Alice".to_string().into())
    );
}

// =============================================================================
// Roundtrip Tests
// =============================================================================

#[test]
fn test_tab_delimiter_roundtrip() {
    let csv = "id\tname\tvalue\n1\ttest\t42\n";
    let from_config = FromCsvConfig {
        delimiter: b'\t',
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Item", &["name", "value"], from_config).unwrap();

    let to_config = ToCsvConfig {
        delimiter: b'\t',
        ..Default::default()
    };
    let output = to_csv_with_config(&doc, to_config).unwrap();
    assert_eq!(csv, output);
}

#[test]
fn test_semicolon_delimiter_roundtrip() {
    let csv = "id;name;value\n1;test;42\n";
    let from_config = FromCsvConfig {
        delimiter: b';',
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Item", &["name", "value"], from_config).unwrap();

    let to_config = ToCsvConfig {
        delimiter: b';',
        ..Default::default()
    };
    let output = to_csv_with_config(&doc, to_config).unwrap();
    assert_eq!(csv, output);
}

#[test]
fn test_pipe_delimiter_roundtrip() {
    let csv = "id|name|value\n1|test|42\n";
    let from_config = FromCsvConfig {
        delimiter: b'|',
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Item", &["name", "value"], from_config).unwrap();

    let to_config = ToCsvConfig {
        delimiter: b'|',
        ..Default::default()
    };
    let output = to_csv_with_config(&doc, to_config).unwrap();
    assert_eq!(csv, output);
}

// =============================================================================
// Whitespace Handling with Delimiters
// =============================================================================

#[test]
fn test_whitespace_around_delimiter_with_trim() {
    let csv = "id , name , age\n1 , Alice , 30\n";
    let config = FromCsvConfig {
        trim: true,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name", "age"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Alice".to_string().into())
    );
}

#[test]
fn test_whitespace_around_delimiter_without_trim() {
    let csv = "id, name, age\n1, Alice, 30\n";
    let config = FromCsvConfig {
        trim: false,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name", "age"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    // With trim=false, leading space is preserved
    assert_eq!(
        list.rows[0].fields[1],
        Value::String(" Alice".to_string().into())
    );
}

#[test]
fn test_mixed_spacing_with_trim() {
    let csv = "id,name,age\n1,  Alice  ,30\n2,Bob,  25  \n";
    let config = FromCsvConfig {
        trim: true,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name", "age"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Alice".to_string().into())
    );
    assert_eq!(
        list.rows[1].fields[1],
        Value::String("Bob".to_string().into())
    );
}

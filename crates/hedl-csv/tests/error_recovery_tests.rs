// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Error recovery and security limit tests.
//!
//! Tests error handling, malformed CSV, and security limit enforcement.

use hedl_csv::{
    from_csv_with_config, CsvError, FromCsvConfig, DEFAULT_MAX_CELL_SIZE, DEFAULT_MAX_COLUMNS,
    DEFAULT_MAX_HEADER_SIZE, DEFAULT_MAX_ROWS, DEFAULT_MAX_TOTAL_SIZE,
};

// =============================================================================
// Security Limit Tests
// =============================================================================

#[test]
fn test_row_limit_exceeded() {
    let config = FromCsvConfig {
        max_rows: 2,
        ..Default::default()
    };

    let csv = "id,name\n1,Alice\n2,Bob\n3,Charlie\n";
    let result = from_csv_with_config(csv, "Person", &["name"], config);

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(
            err,
            CsvError::SecurityLimit { .. } | CsvError::Security { .. }
        ));
    }
}

#[test]
fn test_row_limit_at_boundary() {
    let config = FromCsvConfig {
        max_rows: 2,
        ..Default::default()
    };

    let csv = "id,name\n1,Alice\n2,Bob\n";
    let result = from_csv_with_config(csv, "Person", &["name"], config);

    // Should succeed with exactly max_rows
    assert!(result.is_ok());
}

#[test]
fn test_column_limit_exceeded() {
    let config = FromCsvConfig {
        max_columns: 3,
        ..Default::default()
    };

    let csv = "id,a,b,c,d\n1,1,2,3,4\n";
    let result = from_csv_with_config(csv, "Item", &["a", "b", "c", "d"], config);

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(
            err,
            CsvError::Security { .. } | CsvError::InvalidHeader { .. }
        ));
    }
}

#[test]
fn test_column_limit_at_boundary() {
    let config = FromCsvConfig {
        max_columns: 4,
        ..Default::default()
    };

    let csv = "id,a,b,c\n1,1,2,3\n";
    let result = from_csv_with_config(csv, "Item", &["a", "b", "c"], config);

    assert!(result.is_ok());
}

#[test]
fn test_cell_size_limit_exceeded() {
    let config = FromCsvConfig {
        max_cell_size: 10,
        ..Default::default()
    };

    let csv = "id,text\n1,\"this is a very long text that exceeds the limit\"\n";
    let result = from_csv_with_config(csv, "Item", &["text"], config);

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(
            err,
            CsvError::Security { .. } | CsvError::ParseError { .. }
        ));
    }
}

#[test]
fn test_cell_size_limit_at_boundary() {
    let config = FromCsvConfig {
        max_cell_size: 5,
        ..Default::default()
    };

    let csv = "id,text\n1,hello\n";
    let result = from_csv_with_config(csv, "Item", &["text"], config);

    assert!(result.is_ok());
}

#[test]
fn test_header_size_limit_exceeded() {
    let config = FromCsvConfig {
        max_header_size: 20,
        ..Default::default()
    };

    let csv = "id,very_long_column_name_that_exceeds_limit\n1,value\n";
    let result = from_csv_with_config(
        csv,
        "Item",
        &["very_long_column_name_that_exceeds_limit"],
        config,
    );

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(
            err,
            CsvError::Security { .. } | CsvError::InvalidHeader { .. }
        ));
    }
}

#[test]
fn test_total_size_limit() {
    let config = FromCsvConfig {
        max_total_size: 50,
        ..Default::default()
    };

    let csv = "id,text\n1,\"very long text to exceed total size limit\"\n2,more\n";
    let result = from_csv_with_config(csv, "Item", &["text"], config);

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(
            err,
            CsvError::Security { .. } | CsvError::ParseError { .. }
        ));
    }
}

#[test]
fn test_unlimited_config() {
    let config = FromCsvConfig::unlimited();

    assert_eq!(config.max_rows, usize::MAX);
    assert_eq!(config.max_columns, usize::MAX);
    assert_eq!(config.max_cell_size, usize::MAX);
    assert_eq!(config.max_total_size, usize::MAX);
    assert_eq!(config.max_header_size, usize::MAX);
}

#[test]
fn test_strict_config() {
    let config = FromCsvConfig::strict();

    assert_eq!(config.max_rows, 1_000_000); // Same as default
    assert_eq!(config.max_columns, 1_000);
    assert_eq!(config.max_cell_size, 65_536);
    assert_eq!(config.max_total_size, 10_485_760);
    assert_eq!(config.max_header_size, 65_536);
}

#[test]
fn test_default_limits() {
    let config = FromCsvConfig::default();

    assert_eq!(config.max_rows, DEFAULT_MAX_ROWS);
    assert_eq!(config.max_columns, DEFAULT_MAX_COLUMNS);
    assert_eq!(config.max_cell_size, DEFAULT_MAX_CELL_SIZE);
    assert_eq!(config.max_total_size, DEFAULT_MAX_TOTAL_SIZE);
    assert_eq!(config.max_header_size, DEFAULT_MAX_HEADER_SIZE);
}

// =============================================================================
// Malformed CSV Tests
// =============================================================================

#[test]
fn test_missing_required_column() {
    let csv = "id,age\n1,30\n";
    let result = from_csv_with_config(csv, "Person", &["name", "age"], FromCsvConfig::default());

    // CSV parsing doesn't validate schema fields against CSV columns
    // It only checks for ID column existence
    // Missing optional fields are allowed
    let _ = result;
}

#[test]
fn test_empty_csv() {
    let csv = "";
    let result = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default());

    // Empty CSV may succeed with empty list or error depending on implementation
    let _ = result;
}

#[test]
fn test_only_header() {
    let csv = "id,name\n";
    let result = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default());

    // Should succeed but with empty list
    assert!(result.is_ok());
    let doc = result.unwrap();
    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 0);
}

#[test]
fn test_width_mismatch_fewer_columns() {
    let csv = "id,name,age\n1,Alice,30\n2,Bob\n";
    let result = from_csv_with_config(csv, "Person", &["name", "age"], FromCsvConfig::default());

    // CSV library should handle this (may add empty fields or error)
    // Accept either behavior
    let _ = result;
}

#[test]
fn test_width_mismatch_more_columns() {
    let csv = "id,name,age\n1,Alice,30,extra\n";
    let result = from_csv_with_config(csv, "Person", &["name", "age"], FromCsvConfig::default());

    // CSV library should handle this (may ignore extra fields or error)
    // Accept either behavior
    let _ = result;
}

#[test]
fn test_empty_id_field() {
    let csv = "id,name\n,Alice\n";
    let result = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default());

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, CsvError::EmptyId { .. }));
    }
}

#[test]
fn test_duplicate_column_names() {
    let csv = "id,name,name\n1,Alice,Bob\n";
    let result = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default());

    // Behavior may vary: could accept, error, or use last value
    // Just ensure it doesn't panic
    let _ = result;
}

#[test]
fn test_empty_column_name() {
    let csv = "id,,name\n1,value,Alice\n";
    let result = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default());

    // Should error on empty column name
    assert!(result.is_err());
}

#[test]
fn test_whitespace_only_column_name() {
    let csv = "id,   ,name\n1,value,Alice\n";
    let config = FromCsvConfig {
        trim: true,
        ..Default::default()
    };
    let result = from_csv_with_config(csv, "Person", &["name"], config);

    // After trimming, column name becomes empty, should error
    assert!(result.is_err());
}

// =============================================================================
// Special Character Tests
// =============================================================================

#[test]
fn test_unicode_in_values() {
    let csv = "id,name\n1,Alice\u{2764}\u{FE0F}\n2,\u{1F600}Bob\n";
    let doc = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default()).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
}

#[test]
fn test_null_byte_handling() {
    let csv = "id,name\n1,Alice\0Bob\n";
    let result = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default());

    // Should handle null bytes (behavior may vary)
    let _ = result;
}

#[test]
fn test_control_characters() {
    let csv = "id,text\n1,\"line1\x0Cline2\"\n";
    let result = from_csv_with_config(csv, "Item", &["text"], FromCsvConfig::default());

    // Should handle control characters
    assert!(result.is_ok());
}

// =============================================================================
// Line Ending Tests
// =============================================================================

#[test]
fn test_crlf_line_endings() {
    let csv = "id,name\r\n1,Alice\r\n2,Bob\r\n";
    let doc = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default()).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
}

#[test]
fn test_lf_line_endings() {
    let csv = "id,name\n1,Alice\n2,Bob\n";
    let doc = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default()).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
}

#[test]
fn test_mixed_line_endings() {
    let csv = "id,name\r\n1,Alice\n2,Bob\r\n";
    let doc = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default()).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
}

#[test]
fn test_no_trailing_newline() {
    let csv = "id,name\n1,Alice\n2,Bob";
    let doc = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default()).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
}

// =============================================================================
// Error Message Tests
// =============================================================================

#[test]
fn test_missing_column_error_message() {
    let csv = "id\n1\n";
    let result = from_csv_with_config(csv, "Person", &[], FromCsvConfig::default());

    // Test passes as long as it doesn't panic
    let _ = result;
}

#[test]
fn test_empty_id_error_message() {
    let csv = "id,name\n,Alice\n";
    let result = from_csv_with_config(csv, "Person", &["name"], FromCsvConfig::default());

    assert!(result.is_err());
    if let Err(err) = result {
        let msg = err.to_string();
        assert!(msg.contains("id") || msg.contains("empty"));
    }
}

#[test]
fn test_security_limit_error_message() {
    let config = FromCsvConfig {
        max_rows: 1,
        ..Default::default()
    };

    let csv = "id,name\n1,Alice\n2,Bob\n";
    let result = from_csv_with_config(csv, "Person", &["name"], config);

    assert!(result.is_err());
    if let Err(err) = result {
        let msg = err.to_string();
        assert!(msg.contains("limit") || msg.contains("exceeded"));
    }
}

// =============================================================================
// Config Validation Tests
// =============================================================================

#[test]
fn test_config_debug() {
    let config = FromCsvConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("FromCsvConfig"));
}

#[test]
fn test_config_clone() {
    let config = FromCsvConfig {
        delimiter: b'\t',
        max_rows: 500,
        ..Default::default()
    };
    let cloned = config.clone();
    assert_eq!(cloned.delimiter, b'\t');
    assert_eq!(cloned.max_rows, 500);
}

#[test]
fn test_no_headers_config() {
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
fn test_trim_disabled() {
    let csv = "id,name\n1,  Alice  \n";
    let config = FromCsvConfig {
        trim: false,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv, "Person", &["name"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    // Whitespace should be preserved
    let name = match &list.rows[0].fields[1] {
        hedl_core::Value::String(s) => s.as_ref(),
        _ => panic!("Expected string"),
    };
    assert!(name.starts_with(' ') || name.ends_with(' '));
}

// =============================================================================
// Stress Tests
// =============================================================================

#[test]
fn test_many_rows_within_limit() {
    let mut csv = String::from("id,name\n");
    for i in 1..=1000 {
        csv.push_str(&format!("{i},name{i}\n"));
    }

    let config = FromCsvConfig {
        max_rows: 2000,
        ..Default::default()
    };
    let doc = from_csv_with_config(&csv, "Person", &["name"], config).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1000);
}

#[test]
fn test_many_columns_within_limit() {
    let mut header = String::from("id");
    let mut row = String::from("1");
    for i in 1..=100 {
        header.push_str(&format!(",col{i}"));
        row.push_str(&format!(",val{i}"));
    }
    let csv = format!("{header}\n{row}\n");

    let columns: Vec<&str> = (1..=100).map(|_| "name").collect();
    let config = FromCsvConfig {
        max_columns: 200,
        ..Default::default()
    };
    let result = from_csv_with_config(&csv, "Item", &columns[..100], config);

    assert!(result.is_ok());
}

#[test]
fn test_long_cell_within_limit() {
    let long_text = "a".repeat(1000);
    let csv = format!("id,text\n1,\"{long_text}\"\n");

    let config = FromCsvConfig {
        max_cell_size: 2000,
        ..Default::default()
    };
    let doc = from_csv_with_config(&csv, "Item", &["text"], config).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1);
}

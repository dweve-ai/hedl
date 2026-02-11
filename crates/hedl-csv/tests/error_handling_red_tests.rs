// Error handling verification tests
//
// These tests verify that CSV conversion functions return proper errors
// instead of panicking when given invalid inputs or exceeded limits.

use hedl_csv::{from_csv, from_csv_reader, from_csv_with_config, to_csv_list, FromCsvConfig};

#[test]
fn test_from_csv_exceeds_max_rows_limit() {
    let config = FromCsvConfig {
        max_rows: 2, // Very small limit
        ..Default::default()
    };

    let csv_data = "id,name\n1,Alice\n2,Bob\n3,Charlie"; // 3 rows, exceeds limit

    let result = from_csv_with_config(csv_data, "Person", &["name"], config);
    assert!(
        result.is_err(),
        "Expected error when exceeding max_rows limit"
    );
}

#[test]
fn test_from_csv_exceeds_max_columns_limit() {
    let config = FromCsvConfig {
        max_columns: 2, // Very small limit (including id column)
        ..Default::default()
    };

    let csv_data = "id,name,age,email\n1,Alice,30,alice@test.com";

    let result = from_csv_with_config(csv_data, "Person", &["name", "age", "email"], config);
    assert!(
        result.is_err(),
        "Expected error when exceeding max_columns limit"
    );
}

#[test]
fn test_from_csv_exceeds_max_cell_size_limit() {
    let config = FromCsvConfig {
        max_cell_size: 10, // Very small limit
        ..Default::default()
    };

    let long_value = "a".repeat(100);
    let csv_data = format!("id,description\n1,{}", long_value);

    let result = from_csv_with_config(&csv_data, "Item", &["description"], config);
    assert!(
        result.is_err(),
        "Expected error when exceeding max_cell_size limit"
    );
}

#[test]
fn test_from_csv_invalid_reference_format() {
    let csv_data = "id,ref\n1,@\n2,@:"; // Empty reference IDs

    // Malformed references should return error
    let result = from_csv(csv_data, "Item", &["ref"]);
    assert!(
        result.is_err(),
        "Expected error for malformed reference format"
    );
}

#[test]
fn test_from_csv_mismatched_column_count() {
    let csv_data = "id,name,age\n1,Alice,30\n2,Bob"; // Second row missing age column

    let result = from_csv(csv_data, "Person", &["name", "age"]);
    assert!(
        result.is_err(),
        "Expected error for mismatched column count"
    );
}

#[test]
fn test_to_csv_list_nonexistent_list_key() {
    use hedl_core::{Document, Item, MatrixList, Node, Value};

    let mut doc = Document::new((2, 0));
    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    list.add_row(Node::new(
        "User",
        "alice",
        vec![Value::String("Alice".to_string().into())],
    ));

    doc.root.insert("users".to_string(), Item::List(list));

    // Exporting non-existent list should return error
    let result = to_csv_list(&doc, "nonexistent");
    assert!(result.is_err(), "Expected error for non-existent list key");
}

#[test]
fn test_to_csv_list_not_a_list_item() {
    use hedl_core::{Document, Item, Value};

    let mut doc = Document::new((2, 0));

    // Insert a Scalar, not a List
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Int(42)));

    // Exporting non-list item as CSV should return error
    let result = to_csv_list(&doc, "value");
    assert!(result.is_err(), "Expected error for non-list item");
}

#[test]
fn test_from_csv_reader_io_error() {
    use std::io::{self, Read};

    // Create a reader that always returns an error
    struct ErrorReader;
    impl Read for ErrorReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::Other, "Simulated I/O error"))
        }
    }

    // Should return error, not panic
    let result = from_csv_reader(ErrorReader, "Person", &["name"]);
    assert!(result.is_err(), "Expected error for I/O failure");
}

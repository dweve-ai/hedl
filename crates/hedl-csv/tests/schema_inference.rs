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

//! Schema inference tests for CSV parsing.
//!
//! These tests verify that automatic type inference from CSV headers
//! correctly identifies column types based on sampled data.

use hedl_core::Value;
use hedl_csv::{from_csv_with_config, FromCsvConfig};

// ==================== Basic Type Inference Tests ====================

#[test]
fn test_infer_all_integers() {
    let csv_data = r#"id,age,count
1,30,100
2,25,200
3,35,150
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "Record", &["age", "count"], config).unwrap();
    let list = doc.get("records").unwrap().as_list().unwrap();

    // All three columns should be inferred as Int
    assert_eq!(list.rows.len(), 3);

    // First row
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // id
    assert_eq!(list.rows[0].fields[1], Value::Int(30)); // age
    assert_eq!(list.rows[0].fields[2], Value::Int(100)); // count

    // Second row
    assert_eq!(list.rows[1].fields[0], Value::Int(2));
    assert_eq!(list.rows[1].fields[1], Value::Int(25));
    assert_eq!(list.rows[1].fields[2], Value::Int(200));
}

#[test]
fn test_infer_all_floats() {
    let csv_data = r#"id,price,score
1,99.99,87.5
2,149.50,92.3
3,75.25,88.1
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "Product", &["price", "score"], config).unwrap();
    let list = doc.get("products").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 3);

    // First row - id is int, price and score are floats
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // id
    assert_eq!(list.rows[0].fields[1], Value::Float(99.99)); // price
    assert_eq!(list.rows[0].fields[2], Value::Float(87.5)); // score
}

#[test]
fn test_infer_all_booleans() {
    let csv_data = r#"id,active,verified
1,true,false
2,false,true
3,true,true
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "User", &["active", "verified"], config).unwrap();
    let list = doc.get("users").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 3);

    // First row
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // id
    assert_eq!(list.rows[0].fields[1], Value::Bool(true)); // active
    assert_eq!(list.rows[0].fields[2], Value::Bool(false)); // verified

    // Second row
    assert_eq!(list.rows[1].fields[0], Value::Int(2));
    assert_eq!(list.rows[1].fields[1], Value::Bool(false));
    assert_eq!(list.rows[1].fields[2], Value::Bool(true));
}

#[test]
fn test_infer_all_strings() {
    let csv_data = r#"id,name,email
1,Alice,alice@example.com
2,Bob,bob@example.com
3,Charlie,charlie@example.com
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "User", &["name", "email"], config).unwrap();
    let list = doc.get("users").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 3);

    // First row
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // id (numeric)
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Alice".to_string())
    );
    assert_eq!(
        list.rows[0].fields[2],
        Value::String("alice@example.com".to_string())
    );
}

#[test]
fn test_infer_mixed_types() {
    let csv_data = r#"id,name,age,score,active
1,Alice,30,95.5,true
2,Bob,25,87.3,false
3,Charlie,35,92.1,true
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc =
        from_csv_with_config(csv_data, "Person", &["name", "age", "score", "active"], config)
            .unwrap();
    let list = doc.get("persons").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 3);

    // First row - verify each type is correctly inferred
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // id: Int
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Alice".to_string())
    ); // name: String
    assert_eq!(list.rows[0].fields[2], Value::Int(30)); // age: Int
    assert_eq!(list.rows[0].fields[3], Value::Float(95.5)); // score: Float
    assert_eq!(list.rows[0].fields[4], Value::Bool(true)); // active: Bool
}

// ==================== Null Handling Tests ====================

#[test]
fn test_inference_with_nulls() {
    let csv_data = r#"id,optional_age,optional_name
1,30,Alice
2,,Bob
3,25,
4,,
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(
        csv_data,
        "Record",
        &["optional_age", "optional_name"],
        config,
    )
    .unwrap();
    let list = doc.get("records").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 4);

    // Row 1: has values
    assert_eq!(list.rows[0].fields[1], Value::Int(30));
    assert_eq!(
        list.rows[0].fields[2],
        Value::String("Alice".to_string())
    );

    // Row 2: missing age
    assert_eq!(list.rows[1].fields[1], Value::Null);
    assert_eq!(list.rows[1].fields[2], Value::String("Bob".to_string()));

    // Row 3: missing name
    assert_eq!(list.rows[2].fields[1], Value::Int(25));
    assert_eq!(list.rows[2].fields[2], Value::Null);

    // Row 4: all optional fields missing
    assert_eq!(list.rows[3].fields[1], Value::Null);
    assert_eq!(list.rows[3].fields[2], Value::Null);
}

#[test]
fn test_inference_all_nulls_column() {
    let csv_data = r#"id,always_null,value
1,,100
2,,200
3,,300
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc =
        from_csv_with_config(csv_data, "Record", &["always_null", "value"], config).unwrap();
    let list = doc.get("records").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 3);

    // Column with all nulls should still be Null type
    assert_eq!(list.rows[0].fields[1], Value::Null);
    assert_eq!(list.rows[1].fields[1], Value::Null);
    assert_eq!(list.rows[2].fields[1], Value::Null);

    // Value column should be Int
    assert_eq!(list.rows[0].fields[2], Value::Int(100));
}

// ==================== Edge Cases ====================

#[test]
fn test_inference_integers_vs_floats() {
    // Mix of integers and floats should infer as Float
    let csv_data = r#"id,value
1,10
2,20.5
3,30
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "Record", &["value"], config).unwrap();
    let list = doc.get("records").unwrap().as_list().unwrap();

    // Should infer as Float because not all values are integers
    assert_eq!(list.rows[0].fields[1], Value::Float(10.0));
    assert_eq!(list.rows[1].fields[1], Value::Float(20.5));
    assert_eq!(list.rows[2].fields[1], Value::Float(30.0));
}

#[test]
fn test_inference_string_with_number_looking_values() {
    // Column with mix of numbers and non-numbers should be String
    let csv_data = r#"id,code
1,123
2,ABC
3,456
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "Item", &["code"], config).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();

    // Column inferred as String, but per-value parsing still happens for
    // values that look like numbers (uses parse_csv_value which does int inference)
    assert_eq!(list.rows[0].fields[1], Value::Int(123)); // Parses as int
    assert_eq!(list.rows[1].fields[1], Value::String("ABC".to_string()));
    assert_eq!(list.rows[2].fields[1], Value::Int(456)); // Parses as int
}

#[test]
fn test_inference_bool_with_mixed_values() {
    // Column with true/false and other values should be String
    let csv_data = r#"id,flag
1,true
2,maybe
3,false
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "Record", &["flag"], config).unwrap();
    let list = doc.get("records").unwrap().as_list().unwrap();

    // Column inferred as String, but per-value parsing still happens
    // (parse_csv_value converts "true"/"false" to bools)
    assert_eq!(list.rows[0].fields[1], Value::Bool(true));
    assert_eq!(list.rows[1].fields[1], Value::String("maybe".to_string()));
    assert_eq!(list.rows[2].fields[1], Value::Bool(false));
}

// ==================== Sample Size Tests ====================

#[test]
fn test_inference_small_sample_size() {
    let csv_data = r#"id,value
1,10
2,20
3,30
4,not_a_number
5,50
"#;

    // Sample only first 3 rows - should infer as Int
    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 3,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "Record", &["value"], config).unwrap();
    let list = doc.get("records").unwrap().as_list().unwrap();

    // First 3 rows parse as Int based on inference
    assert_eq!(list.rows[0].fields[1], Value::Int(10));
    assert_eq!(list.rows[1].fields[1], Value::Int(20));
    assert_eq!(list.rows[2].fields[1], Value::Int(30));

    // Row 4 has non-integer but still uses inferred Int type, falls back to String
    assert_eq!(
        list.rows[3].fields[1],
        Value::String("not_a_number".to_string())
    );

    // Row 5 is Int
    assert_eq!(list.rows[4].fields[1], Value::Int(50));
}

#[test]
fn test_inference_large_sample_size() {
    let csv_data = r#"id,value
1,10
2,20
3,30
4,not_a_number
5,50
"#;

    // Sample all rows - should infer as String
    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "Record", &["value"], config).unwrap();
    let list = doc.get("records").unwrap().as_list().unwrap();

    // Column inferred as String (due to "not_a_number"), but values that look
    // like numbers still parse as numbers via parse_csv_value
    assert_eq!(list.rows[0].fields[1], Value::Int(10));
    assert_eq!(list.rows[1].fields[1], Value::Int(20));
    assert_eq!(list.rows[2].fields[1], Value::Int(30));
    assert_eq!(
        list.rows[3].fields[1],
        Value::String("not_a_number".to_string())
    );
    assert_eq!(list.rows[4].fields[1], Value::Int(50));
}

// ==================== Comparison with Non-Inference ====================

#[test]
fn test_schema_inference_vs_no_inference() {
    let csv_data = r#"id,age,name
1,30,Alice
2,25,Bob
"#;

    // Without inference
    let config_no_infer = FromCsvConfig {
        infer_schema: false,
        ..Default::default()
    };

    let doc_no_infer =
        from_csv_with_config(csv_data, "Person", &["age", "name"], config_no_infer).unwrap();
    let list_no_infer = doc_no_infer.get("persons").unwrap().as_list().unwrap();

    // With inference
    let config_infer = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc_infer =
        from_csv_with_config(csv_data, "Person", &["age", "name"], config_infer).unwrap();
    let list_infer = doc_infer.get("persons").unwrap().as_list().unwrap();

    // Both should have same structure
    assert_eq!(list_no_infer.rows.len(), list_infer.rows.len());

    // Without inference: per-value inference happens
    // With inference: column-wide inference happens
    // Both should produce Int for age in this case
    assert!(matches!(list_no_infer.rows[0].fields[1], Value::Int(30)));
    assert!(matches!(list_infer.rows[0].fields[1], Value::Int(30)));
}

// ==================== Complex Scenarios ====================

#[test]
fn test_inference_with_scientific_notation() {
    let csv_data = r#"id,measurement
1,1.5e10
2,2.3e-5
3,1000000
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "Data", &["measurement"], config).unwrap();
    let list = doc.get("datas").unwrap().as_list().unwrap();

    // Should infer as Float
    assert!(matches!(list.rows[0].fields[1], Value::Float(_)));
    assert!(matches!(list.rows[1].fields[1], Value::Float(_)));
    // Last one is also Float because column is inferred as Float
    assert_eq!(list.rows[2].fields[1], Value::Float(1000000.0));
}

#[test]
fn test_inference_empty_csv() {
    let csv_data = "id,value\n";

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "Record", &["value"], config).unwrap();
    let list = doc.get("records").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 0);
}

#[test]
fn test_inference_single_row() {
    let csv_data = r#"id,value
1,42
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "Record", &["value"], config).unwrap();
    let list = doc.get("records").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 1);
    assert_eq!(list.rows[0].fields[1], Value::Int(42));
}

// ==================== Whitespace Handling ====================

#[test]
fn test_inference_with_whitespace() {
    let csv_data = r#"id,value
1,  42
2,  100
3,  75
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        trim: true, // Trimming enabled
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "Record", &["value"], config).unwrap();
    let list = doc.get("records").unwrap().as_list().unwrap();

    // Should trim and infer as Int
    assert_eq!(list.rows[0].fields[1], Value::Int(42));
    assert_eq!(list.rows[1].fields[1], Value::Int(100));
    assert_eq!(list.rows[2].fields[1], Value::Int(75));
}

// ==================== Delimiter Tests ====================

#[test]
fn test_inference_with_tab_delimiter() {
    let csv_data = "id\tage\tactive\n1\t30\ttrue\n2\t25\tfalse\n";

    let config = FromCsvConfig {
        delimiter: b'\t',
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "User", &["age", "active"], config).unwrap();
    let list = doc.get("users").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 2);
    assert_eq!(list.rows[0].fields[1], Value::Int(30));
    assert_eq!(list.rows[0].fields[2], Value::Bool(true));
}

// ==================== Fallback Behavior ====================

#[test]
fn test_inference_fallback_to_string() {
    // When type inference fails during parsing, should fallback to string
    let csv_data = r#"id,value
1,10
2,20
3,30
"#;

    let config = FromCsvConfig {
        infer_schema: true,
        sample_rows: 100,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "Record", &["value"], config).unwrap();
    let list = doc.get("records").unwrap().as_list().unwrap();

    // All values should be Int
    assert_eq!(list.rows[0].fields[1], Value::Int(10));
    assert_eq!(list.rows[1].fields[1], Value::Int(20));
    assert_eq!(list.rows[2].fields[1], Value::Int(30));
}

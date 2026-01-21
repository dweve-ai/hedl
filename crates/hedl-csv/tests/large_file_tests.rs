// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Large file handling and performance tests.
//!
//! Tests CSV parsing and generation with large datasets.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_csv::{from_csv_with_config, to_csv_with_config, FromCsvConfig, ToCsvConfig};

// =============================================================================
// Large Row Count Tests
// =============================================================================

#[test]
fn test_1000_rows() {
    let mut csv = String::from("id,name,value\n");
    for i in 1..=1000 {
        csv.push_str(&format!("{},name{},{}\n", i, i, i * 10));
    }

    let doc =
        from_csv_with_config(&csv, "Item", &["name", "value"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1000);
    assert_eq!(list.rows[999].fields[0], Value::Int(1000)); // ID
    assert_eq!(list.rows[999].fields[2], Value::Int(10000)); // value
}

#[test]
fn test_10000_rows() {
    let mut csv = String::from("id,value\n");
    for i in 1..=10000 {
        csv.push_str(&format!("{i},{i}\n"));
    }

    let config = FromCsvConfig {
        max_rows: 15000,
        ..Default::default()
    };
    let doc = from_csv_with_config(&csv, "Item", &["value"], config).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 10000);
}

#[test]
fn test_large_roundtrip() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    for i in 1..=1000 {
        let id_str = i.to_string();
        list.add_row(Node::new(
            "Item",
            &id_str,
            vec![Value::String(id_str.clone().into()), Value::Int(i * 10)],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv_with_config(&doc, ToCsvConfig::default()).unwrap();
    let doc2 = from_csv_with_config(&csv, "Item", &["value"], FromCsvConfig::default()).unwrap();

    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(list2.rows.len(), 1000);
}

// =============================================================================
// Wide Tables (Many Columns)
// =============================================================================

#[test]
fn test_100_columns() {
    let mut header = String::from("id");
    let mut row = String::from("1");
    let mut schema = Vec::new();

    for i in 1..=100 {
        header.push_str(&format!(",col{i}"));
        row.push_str(&format!(",{i}"));
        schema.push(format!("col{i}"));
    }

    let csv = format!("{header}\n{row}\n");
    let schema_refs: Vec<&str> = schema.iter().map(std::string::String::as_str).collect();

    let config = FromCsvConfig {
        max_columns: 150,
        ..Default::default()
    };
    let doc = from_csv_with_config(&csv, "Item", &schema_refs, config).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1);
    assert_eq!(list.schema.len(), 101); // id + 100 columns
}

#[test]
fn test_50_columns_with_types() {
    let mut header = String::from("id");
    let mut row = String::from("1");
    let mut schema = Vec::new();

    for i in 1..=50 {
        header.push_str(&format!(",col{i}"));
        // Alternate between int, float, string, bool
        let value = match i % 4 {
            0 => i.to_string(),
            1 => format!("{i}.5"),
            2 => format!("text{i}"),
            _ => if i % 2 == 0 { "true" } else { "false" }.to_string(),
        };
        row.push_str(&format!(",{value}"));
        schema.push(format!("col{i}"));
    }

    let csv = format!("{header}\n{row}\n");
    let schema_refs: Vec<&str> = schema.iter().map(std::string::String::as_str).collect();

    let doc = from_csv_with_config(&csv, "Item", &schema_refs, FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1);
}

// =============================================================================
// Large Cell Content
// =============================================================================

#[test]
fn test_1kb_cell() {
    let large_text = "x".repeat(1024);
    let csv = format!("id,text\n1,\"{large_text}\"\n");

    let config = FromCsvConfig {
        max_cell_size: 2048,
        ..Default::default()
    };
    let doc = from_csv_with_config(&csv, "Item", &["text"], config).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    let text = match &list.rows[0].fields[1] {
        Value::String(s) => s.as_ref(),
        _ => panic!("Expected string"),
    };
    assert_eq!(text.len(), 1024);
}

#[test]
fn test_10kb_cell() {
    let large_text = "y".repeat(10240);
    let csv = format!("id,text\n1,\"{large_text}\"\n");

    let config = FromCsvConfig {
        max_cell_size: 20480,
        ..Default::default()
    };
    let doc = from_csv_with_config(&csv, "Item", &["text"], config).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    let text = match &list.rows[0].fields[1] {
        Value::String(s) => s.as_ref(),
        _ => panic!("Expected string"),
    };
    assert_eq!(text.len(), 10240);
}

#[test]
fn test_many_large_cells() {
    let mut csv = String::from("id,text\n");
    let large_text = "z".repeat(1000);

    for i in 1..=100 {
        csv.push_str(&format!("{i},\"{large_text}\"\n"));
    }

    let config = FromCsvConfig {
        max_cell_size: 2000,
        max_total_size: 200_000,
        ..Default::default()
    };
    let doc = from_csv_with_config(&csv, "Item", &["text"], config).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 100);
}

// =============================================================================
// Combined Stress Tests
// =============================================================================

#[test]
fn test_1000_rows_10_columns() {
    let mut header = String::from("id");
    let mut schema = Vec::new();

    for i in 1..=10 {
        header.push_str(&format!(",col{i}"));
        schema.push(format!("col{i}"));
    }

    let mut csv = header.clone();
    csv.push('\n');

    for row_num in 1..=1000 {
        let mut row = row_num.to_string();
        for col_num in 1..=10 {
            row.push_str(&format!(",{}", row_num * col_num));
        }
        csv.push_str(&row);
        csv.push('\n');
    }

    let schema_refs: Vec<&str> = schema.iter().map(std::string::String::as_str).collect();
    let doc = from_csv_with_config(&csv, "Item", &schema_refs, FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1000);
    assert_eq!(list.schema.len(), 11); // id + 10 columns
}

#[test]
fn test_mixed_types_large_dataset() {
    let mut csv = String::from("id,int_val,float_val,bool_val,str_val\n");

    for i in 1..=500 {
        let text_val = format!("text_{i}");
        csv.push_str(&format!(
            "{},{},{:.2},{},{}\n",
            i,
            i * 100,
            f64::from(i) * 1.5,
            if i % 2 == 0 { "true" } else { "false" },
            text_val
        ));
    }

    let doc = from_csv_with_config(
        &csv,
        "Data",
        &["int_val", "float_val", "bool_val", "str_val"],
        FromCsvConfig::default(),
    )
    .unwrap();

    let list = doc.get("datas").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 500);

    // Verify types are preserved
    assert!(matches!(list.rows[0].fields[1], Value::Int(_)));
    assert!(matches!(list.rows[0].fields[2], Value::Float(_)));
    assert!(matches!(list.rows[0].fields[3], Value::Bool(_)));
    assert!(matches!(list.rows[0].fields[4], Value::String(_)));
}

// =============================================================================
// Memory Efficiency Tests
// =============================================================================

#[test]
fn test_empty_cells_dont_waste_memory() {
    let mut csv = String::from("id,a,b,c,d,e\n");

    for i in 1..=1000 {
        // Most cells are empty
        csv.push_str(&format!("{i},,,,,\n"));
    }

    let doc = from_csv_with_config(
        &csv,
        "Item",
        &["a", "b", "c", "d", "e"],
        FromCsvConfig::default(),
    )
    .unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1000);

    // All empty cells should be null
    for row in &list.rows {
        for field in &row.fields[1..] {
            assert_eq!(*field, Value::Null);
        }
    }
}

#[test]
fn test_repeated_values() {
    let mut csv = String::from("id,status\n");

    for i in 1..=1000 {
        csv.push_str(&format!("{i},active\n"));
    }

    let doc = from_csv_with_config(&csv, "Item", &["status"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1000);

    // All should have same value
    for row in &list.rows {
        assert_eq!(row.fields[1], Value::String("active".to_string().into()));
    }
}

// =============================================================================
// Generation Performance Tests
// =============================================================================

#[test]
fn test_generate_1000_rows() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec!["id".to_string(), "value".to_string(), "name".to_string()],
    );

    for i in 1..=1000 {
        let id_str = i.to_string();
        let item_name = format!("item_{i}");
        list.add_row(Node::new(
            "Item",
            &id_str,
            vec![
                Value::String(id_str.clone().into()),
                Value::Int(i * 10),
                Value::String(item_name.into()),
            ],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv_with_config(&doc, ToCsvConfig::default()).unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 1001); // header + 1000 rows
}

#[test]
fn test_generate_wide_table() {
    let mut doc = Document::new((1, 0));
    let mut schema = vec!["id".to_string()];

    for i in 1..=50 {
        schema.push(format!("col{i}"));
    }

    let mut list = MatrixList::new("Item", schema.clone());

    let mut fields = vec![Value::String("1".into())];
    for i in 1..=50 {
        fields.push(Value::Int(i));
    }

    list.add_row(Node::new("Item", "1", fields));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv_with_config(&doc, ToCsvConfig::default()).unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 2); // header + 1 row

    // Verify all columns present
    let header_cols = lines[0].split(',').count();
    assert_eq!(header_cols, 51); // id + 50 columns
}

// =============================================================================
// Edge Cases with Large Data
// =============================================================================

#[test]
fn test_alternating_null_values_large() {
    let mut csv = String::from("id,a,b\n");

    for i in 1..=500 {
        if i % 2 == 0 {
            csv.push_str(&format!("{i},value,\n"));
        } else {
            csv.push_str(&format!("{i},null,value\n"));
        }
    }

    let doc = from_csv_with_config(&csv, "Item", &["a", "b"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 500);
}

#[test]
fn test_sequential_ids_large() {
    let mut csv = String::from("id,name\n");

    for i in 1..=1000 {
        csv.push_str(&format!("{i},name{i}\n"));
    }

    let doc = from_csv_with_config(&csv, "Person", &["name"], FromCsvConfig::default()).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();

    // Verify IDs are sequential
    for (idx, row) in list.rows.iter().enumerate() {
        assert_eq!(&*row.id, &(idx + 1).to_string());
    }
}

#[test]
fn test_non_sequential_ids_large() {
    let mut csv = String::from("id,name\n");

    for i in (1..=1000).rev() {
        csv.push_str(&format!("{},name{}\n", i * 100, i));
    }

    let doc = from_csv_with_config(&csv, "Person", &["name"], FromCsvConfig::default()).unwrap();

    let list = doc.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1000);
}

// =============================================================================
// Unicode and Special Characters at Scale
// =============================================================================

#[test]
fn test_unicode_heavy_dataset() {
    let mut csv = String::from("id,text\n");

    for i in 1..=100 {
        csv.push_str(&format!("{i},\u{1F600}{i}\u{2764}\u{FE0F}\n"));
    }

    let doc = from_csv_with_config(&csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 100);
}

#[test]
fn test_mixed_languages() {
    let mut csv = String::from("id,text\n");

    let texts = [
        "Hello",
        "你好",
        "こんにちは",
        "Здравствуй",
        "مرحبا",
        "Γειά σου",
    ];

    for (i, text) in texts.iter().cycle().take(100).enumerate() {
        csv.push_str(&format!("{},{}\n", i + 1, text));
    }

    let doc = from_csv_with_config(&csv, "Item", &["text"], FromCsvConfig::default()).unwrap();

    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 100);
}

// =============================================================================
// Realistic Dataset Tests
// =============================================================================

#[test]
fn test_customer_dataset() {
    let mut csv = String::from("id,name,email,age,active\n");

    for i in 1..=500 {
        csv.push_str(&format!(
            "{},Customer{},customer{}@example.com,{},{}\n",
            i,
            i,
            i,
            20 + (i % 60),
            if i % 3 == 0 { "true" } else { "false" }
        ));
    }

    let doc = from_csv_with_config(
        &csv,
        "Customer",
        &["name", "email", "age", "active"],
        FromCsvConfig::default(),
    )
    .unwrap();

    let list = doc.get("customers").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 500);
}

#[test]
fn test_product_catalog() {
    let mut csv = String::from("id,sku,name,price,in_stock\n");

    for i in 1..=300 {
        let product_name = format!("Product {i}");
        csv.push_str(&format!(
            "{},SKU{:05},{},{:.2},{}\n",
            i,
            i,
            product_name,
            f64::from(i) * 9.99,
            if i % 5 == 0 { "false" } else { "true" }
        ));
    }

    let doc = from_csv_with_config(
        &csv,
        "Product",
        &["sku", "name", "price", "in_stock"],
        FromCsvConfig::default(),
    )
    .unwrap();

    let list = doc.get("products").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 300);
}

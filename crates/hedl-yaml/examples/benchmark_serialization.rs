// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Simple benchmark to verify YAML serialization optimizations
//!
//! This example demonstrates the performance characteristics of the optimized
//! YAML serialization implementation.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_yaml::{to_yaml, ToYamlConfig};
use std::collections::BTreeMap;
use std::time::Instant;

fn main() {
    println!("YAML Serialization Performance Test");
    println!("====================================\n");

    // Test 1: Small document (10 objects)
    println!("Test 1: Small Document (10 objects)");
    let small_doc = create_test_document(10);
    benchmark("Small", &small_doc);

    // Test 2: Medium document (100 objects)
    println!("\nTest 2: Medium Document (100 objects)");
    let medium_doc = create_test_document(100);
    benchmark("Medium", &medium_doc);

    // Test 3: Large document (1000 objects)
    println!("\nTest 3: Large Document (1000 objects)");
    let large_doc = create_test_document(1000);
    benchmark("Large", &large_doc);

    // Test 4: Matrix list (1000 rows × 5 columns)
    println!("\nTest 4: Matrix List (1000 rows × 5 columns)");
    let matrix_doc = create_matrix_document(1000, 5);
    benchmark("Matrix", &matrix_doc);

    println!("\nAll tests completed successfully!");
    println!("\nOptimizations applied:");
    println!("  - String constant caching (TYPE_KEY, SCHEMA_KEY, etc.)");
    println!("  - Field name caching (field_0 through field_99)");
    println!("  - Pre-allocation with capacity hints");
    println!("  - Schema pre-conversion to YamlValue");
    println!("  - Optimized expression formatting");
}

fn benchmark(name: &str, doc: &Document) {
    let config = ToYamlConfig::default();
    let iterations = if name == "Large" || name == "Matrix" {
        100
    } else {
        1000
    };

    // Warmup
    for _ in 0..10 {
        let _ = to_yaml(doc, &config).unwrap();
    }

    // Actual benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = to_yaml(doc, &config).unwrap();
    }
    let elapsed = start.elapsed();

    let avg_time_us = elapsed.as_micros() as f64 / f64::from(iterations);
    let yaml = to_yaml(doc, &config).unwrap();
    let yaml_size = yaml.len();

    println!("  Average time: {avg_time_us:.2} μs");
    println!("  Output size:  {yaml_size} bytes");
    println!(
        "  Throughput:   {:.2} MB/s",
        (yaml_size as f64 * f64::from(iterations)) / (elapsed.as_secs_f64() * 1_000_000.0)
    );
}

fn create_test_document(count: usize) -> Document {
    let mut doc = Document {
        version: (1, 0),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
    };

    for i in 0..count {
        let mut obj = BTreeMap::new();
        obj.insert("id".to_string(), Item::Scalar(Value::Int(i as i64)));
        obj.insert(
            "name".to_string(),
            Item::Scalar(Value::String(format!("Item_{i}").into())),
        );
        obj.insert(
            "value".to_string(),
            Item::Scalar(Value::Float(i as f64 * 1.5)),
        );
        obj.insert("active".to_string(), Item::Scalar(Value::Bool(i % 2 == 0)));

        doc.root.insert(format!("item_{i}"), Item::Object(obj));
    }

    doc
}

fn create_matrix_document(rows: usize, cols: usize) -> Document {
    let mut doc = Document {
        version: (1, 0),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
    };

    let mut schema = vec!["id".to_string()];
    for i in 0..cols {
        schema.push(format!("col_{i}"));
    }

    let mut list = MatrixList::new("Record".to_string(), schema.clone());

    for i in 0..rows {
        let mut fields = vec![Value::String(format!("row_{i}").into())];
        for j in 0..cols {
            fields.push(Value::Int((i * cols + j) as i64));
        }
        list.add_row(Node::new("Record", format!("row_{i}"), fields));
    }

    doc.root.insert("records".to_string(), Item::List(list));
    doc
}

// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Large document handling tests for hedl-json
//!
//! Tests performance and correctness with large JSON documents

use hedl_json::*;
use serde_json::json;

// ==================== Large Array Tests ====================

#[test]
fn test_large_array_parsing() {
    let config = FromJsonConfig::builder().max_array_size(100_000).build();

    // Create large array
    let items: Vec<_> = (0..10_000)
        .map(|i| json!({"id": i, "value": i * 2}))
        .collect();
    let json = json!({"items": items});

    let result = from_json_value(&json, &config);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert!(doc.root.contains_key("items"));
}

#[test]
fn test_large_array_conversion_to_json() {
    use hedl_core::{Item, MatrixList, Node, Value};

    // Create a large matrix list
    let mut rows = Vec::new();
    for i in 0..5_000 {
        rows.push(Node {
            type_name: "Item".to_string(),
            id: format!("i{i}"),
            fields: vec![Value::String(format!("i{i}").into()), Value::Int(i)].into(),
            children: None,
            child_count: 0,
        });
    }

    let list = MatrixList {
        type_name: "Item".to_string(),
        schema: vec!["id".to_string(), "value".to_string()],
        rows,
        count_hint: None,
    };

    let mut root = std::collections::BTreeMap::new();
    root.insert("items".to_string(), Item::List(list));

    let doc = hedl_core::Document {
        version: (1, 0),
        aliases: std::collections::BTreeMap::new(),
        structs: std::collections::BTreeMap::new(),
        nests: std::collections::BTreeMap::new(),
        root,
        schema_versions: std::collections::BTreeMap::new(),
    };

    let json_str = hedl_to_json(&doc).unwrap();
    assert!(json_str.len() > 100_000); // Should be large
}

#[test]
fn test_deeply_nested_object() {
    let config = FromJsonConfig::builder().max_depth(1000).build();

    // Create deep nesting
    let mut json = json!({"value": 42});
    for i in 0..100 {
        json = json!({"level": i, "nested": json});
    }

    let result = from_json_value(&json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_wide_object() {
    let config = FromJsonConfig::builder().max_object_size(10_000).build();

    // Create object with many fields
    let mut obj = serde_json::Map::new();
    for i in 0..1_000 {
        obj.insert(format!("field_{i}"), json!(i));
    }

    let json = serde_json::Value::Object(obj);
    let result = from_json_value(&json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_large_string_values() {
    let config = FromJsonConfig::builder()
        .max_string_length(10_000_000) // 10 MB
        .build();

    // Create large string
    let large_string = "x".repeat(1_000_000); // 1 MB string
    let json = json!({"data": large_string});

    let result = from_json_value(&json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_many_small_objects() {
    let config = FromJsonConfig::default();

    // Create array with many small objects
    let items: Vec<_> = (0..10_000)
        .map(|i| json!({"id": i, "name": format!("item_{}", i)}))
        .collect();
    let json = json!({"items": items});

    let result = from_json_value(&json, &config);
    assert!(result.is_ok());
}

// ==================== Streaming Large Files ====================

#[test]
fn test_stream_large_json_array() {
    use hedl_json::streaming::{JsonArrayStreamer, StreamConfig};
    use std::io::Cursor;

    // Create large JSON array as string
    let mut json_str = String::from("[");
    for i in 0..1_000 {
        if i > 0 {
            json_str.push(',');
        }
        json_str.push_str(&format!(r#"{{"id": {}, "value": {}}}"#, i, i * 2));
    }
    json_str.push(']');

    let reader = Cursor::new(json_str.as_bytes());
    let config = StreamConfig::default();

    let streamer = JsonArrayStreamer::new(reader, config).unwrap();

    let mut count = 0;
    for result in streamer {
        assert!(result.is_ok());
        count += 1;
    }

    assert_eq!(count, 1_000);
}

#[test]
fn test_stream_large_jsonl() {
    use hedl_json::streaming::{JsonLinesStreamer, StreamConfig};
    use std::io::Cursor;

    // Create large JSONL data
    let mut jsonl = String::new();
    for i in 0..1_000 {
        jsonl.push_str(&format!(r#"{{"id": {}, "value": {}}}"#, i, i * 2));
        jsonl.push('\n');
    }

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::default();

    let streamer = JsonLinesStreamer::new(reader, config);

    let mut count = 0;
    for result in streamer {
        assert!(result.is_ok());
        count += 1;
    }

    assert_eq!(count, 1_000);
}

#[test]
fn test_stream_with_large_objects() {
    use hedl_json::streaming::{JsonLinesStreamer, StreamConfig};
    use std::io::Cursor;

    let config = StreamConfig::builder()
        .max_object_bytes(100 * 1024 * 1024) // 100 MB limit
        .build();

    // Create JSONL with moderately large objects
    let mut jsonl = String::new();
    for i in 0..100 {
        let large_field = "x".repeat(10_000);
        jsonl.push_str(&format!(r#"{{"id": {i}, "data": "{large_field}"}}"#));
        jsonl.push('\n');
    }

    let reader = Cursor::new(jsonl.as_bytes());
    let streamer = JsonLinesStreamer::new(reader, config);

    let mut count = 0;
    for result in streamer {
        assert!(result.is_ok());
        count += 1;
    }

    assert_eq!(count, 100);
}

#[test]
fn test_memory_efficient_streaming() {
    use hedl_json::streaming::StreamConfig;

    // Verify streaming config supports large files
    let config = StreamConfig::large_file();

    assert_eq!(config.buffer_size, 256 * 1024);
    assert_eq!(config.max_object_bytes, Some(50 * 1024 * 1024));
    assert!(config.true_streaming);
}

#[test]
fn test_low_memory_streaming() {
    use hedl_json::streaming::StreamConfig;

    let config = StreamConfig::low_memory();

    assert_eq!(config.buffer_size, 8 * 1024);
    assert_eq!(config.max_object_bytes, Some(1024 * 1024));
}

// ==================== Schema Generation with Large Documents ====================

#[test]
fn test_schema_generation_large_struct() {
    use hedl_core::parse;
    use hedl_json::schema_gen::{generate_schema, SchemaConfig};

    // Create HEDL with many struct fields (max 100 columns total)
    let mut hedl = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:Record:[id");
    // Add 99 more fields (id + 99 = 100 columns, max allowed)
    for i in 0..99 {
        hedl.push_str(&format!(", field_{i}"));
    }
    // Matrix list rows need |prefix (1-space indent in v2.0)
    hedl.push_str("]\n---\ndata:@Record\n |r1");
    for i in 0..99 {
        hedl.push_str(&format!(", value_{i}"));
    }

    let doc = parse(hedl.as_bytes()).unwrap();
    let config = SchemaConfig::default();

    let schema = generate_schema(&doc, &config);
    assert!(schema.is_ok());
}

#[test]
fn test_schema_generation_many_types() {
    use hedl_core::parse;
    use hedl_json::schema_gen::{generate_schema, SchemaConfig};

    // Create HEDL with many type definitions
    let mut hedl = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n");
    for i in 0..50 {
        hedl.push_str(&format!("%S:Type{i}:[id,value]\n"));
    }
    hedl.push_str("---\n");
    for i in 0..50 {
        // Matrix list rows need |prefix (1-space indent in v2.0)
        hedl.push_str(&format!("type{i}:@Type{i}\n |t{i}, val{i}\n"));
    }

    let doc = parse(hedl.as_bytes()).unwrap();
    let config = SchemaConfig::default();

    let schema = generate_schema(&doc, &config);
    assert!(schema.is_ok());
}

// ==================== JSONPath with Large Documents ====================

#[test]
fn test_jsonpath_large_array_filter() {
    use hedl_core::parse;
    use hedl_json::jsonpath::{query, QueryConfig};

    // Create document with large array
    let mut hedl =
        String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:Item:[id,value]\n---\nitems:@Item\n");
    for i in 0..1_000 {
        // Matrix list rows need |prefix (1-space indent in v2.0)
        hedl.push_str(&format!(" |i{i}, {i}\n"));
    }

    let doc = parse(hedl.as_bytes()).unwrap();
    let config = QueryConfig::default();

    let results = query(&doc, "$.items", &config).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_jsonpath_with_max_results() {
    use hedl_core::parse;
    use hedl_json::jsonpath::{query, QueryConfigBuilder};

    let mut hedl = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n");
    for i in 0..1_000 {
        hedl.push_str(&format!("field_{i}: {i}\n"));
    }

    let doc = parse(hedl.as_bytes()).unwrap();
    let config = QueryConfigBuilder::new().max_results(10).build();

    let results = query(&doc, "$.*", &config).unwrap();
    assert_eq!(results.len(), 10);
}

// ==================== String Cache Performance ====================

#[test]
fn test_string_cache_with_repeated_fields() {
    use hedl_json::string_cache::{clear_string_cache, intern_string, string_cache_stats};

    clear_string_cache();

    let field_names = vec!["id", "name", "email", "created_at", "updated_at"];

    // Simulate parsing 10,000 objects with same field names
    for _ in 0..10_000 {
        for field in &field_names {
            intern_string(field);
        }
    }

    let stats = string_cache_stats();

    // Should have very high hit rate
    assert!(stats.hit_rate() > 0.99);
    assert_eq!(stats.entries, 5); // Only 5 unique strings
}

#[test]
fn test_string_cache_eviction() {
    use hedl_json::string_cache::{clear_string_cache, intern_string, string_cache_stats};

    clear_string_cache();

    // Add strings beyond cache capacity
    for i in 0..15_000 {
        intern_string(&format!("field_{i}"));
    }

    let stats = string_cache_stats();

    // Cache should have been cleared at least once
    assert!(stats.entries <= 10_000);
}

// ==================== Schema Cache Performance ====================

#[test]
fn test_schema_cache_with_repeated_structures() {
    use hedl_json::schema_cache::{SchemaCache, SchemaCacheKey};

    let cache = SchemaCache::new(100);

    // Simulate repeated schema lookups
    let key = SchemaCacheKey::new(vec![
        "id".to_string(),
        "name".to_string(),
        "email".to_string(),
    ]);
    let schema = vec!["id".to_string(), "name".to_string(), "email".to_string()];

    // Insert once
    cache.insert(key.clone(), schema.clone());

    // Look up many times
    for _ in 0..10_000 {
        let result = cache.get(&key);
        assert_eq!(result, Some(schema.clone()));
    }

    let stats = cache.statistics();

    // Should have very high hit rate
    assert!(stats.hit_rate() > 0.999);
}

// ==================== Performance Stress Tests ====================

#[test]
fn test_conversion_performance_baseline() {
    use std::time::Instant;

    let config = FromJsonConfig::default();

    // Create moderately sized document
    let items: Vec<_> = (0..1_000)
        .map(|i| {
            json!({
                "id": i,
                "name": format!("item_{}", i),
                "value": i * 2,
                "active": i % 2 == 0
            })
        })
        .collect();

    let json = json!({"items": items});

    let start = Instant::now();
    let result = from_json_value(&json, &config);
    let duration = start.elapsed();

    assert!(result.is_ok());

    // Should complete reasonably quickly (< 1 second for 1k items)
    assert!(duration.as_secs() < 5);
}

#[test]
fn test_to_json_performance_baseline() {
    use hedl_core::{Item, MatrixList, Node, Value};
    use std::time::Instant;

    // Create document with 1,000 items
    let mut rows = Vec::new();
    for i in 0..1_000 {
        rows.push(Node {
            type_name: "Item".to_string(),
            id: format!("i{i}"),
            fields: vec![Value::String(format!("i{i}").into()), Value::Int(i)].into(),
            children: None,
            child_count: 0,
        });
    }

    let list = MatrixList {
        type_name: "Item".to_string(),
        schema: vec!["id".to_string(), "value".to_string()],
        rows,
        count_hint: None,
    };

    let mut root = std::collections::BTreeMap::new();
    root.insert("items".to_string(), Item::List(list));

    let doc = hedl_core::Document {
        version: (1, 0),
        aliases: std::collections::BTreeMap::new(),
        structs: std::collections::BTreeMap::new(),
        nests: std::collections::BTreeMap::new(),
        root,
        schema_versions: std::collections::BTreeMap::new(),
    };

    let config = ToJsonConfig::default();

    let start = Instant::now();
    let result = to_json(&doc, &config);
    let duration = start.elapsed();

    assert!(result.is_ok());

    // Should complete reasonably quickly
    assert!(duration.as_secs() < 5);
}

#[test]
fn test_deeply_nested_performance() {
    use std::time::Instant;

    let config = FromJsonConfig::builder().max_depth(500).build();

    // Create deep nesting
    let mut json = json!({"value": 1});
    for i in 0..200 {
        json = json!({"level": i, "data": json});
    }

    let start = Instant::now();
    let result = from_json_value(&json, &config);
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(duration.as_secs() < 5);
}

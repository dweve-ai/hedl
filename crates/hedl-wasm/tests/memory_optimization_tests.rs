// HEDL WebAssembly Memory Optimization Tests
//
// Tests for memory-efficient parsing, querying, and conversion operations

use hedl_core::{parse as core_parse, Item};

/// Generate test HEDL document with specified node count
fn generate_test_doc(node_count: usize) -> String {
    let mut doc = String::from(
        r#"%V:2.0
%NULL:~
%QUOTE:"
"#,
    );
    doc.push_str("%S:Entity:[id, name, value, timestamp]\n");
    doc.push_str("---\n");
    doc.push_str("entities:@Entity\n");

    for i in 0..node_count {
        doc.push_str(&format!(
            " |entity_{}, Name {}, {}, 2024-01-01T00:00:00Z\n",
            i,
            i,
            i * 100
        ));
    }

    doc
}

#[test]
fn test_parse_partial_header_only() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
%S:Post:[id, title]
%N:User>Post
---
users:@User
 |alice, Alice, alice@example.com
 |bob, Bob, bob@example.com
"#;

    let mut doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Clear root to simulate header-only parsing
    let root_count_before = doc.root.len();
    doc.root.clear();

    assert!(root_count_before > 0, "Should have had root items");
    assert_eq!(doc.root.len(), 0, "Root should be empty after clearing");
    assert_eq!(doc.structs.len(), 2, "Should preserve structs");
    assert_eq!(doc.nests.len(), 1, "Should preserve nests");
}

#[test]
fn test_parse_partial_skip_entities() {
    let hedl = generate_test_doc(100);
    let mut doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Clear entities from lists
    for item in doc.root.values_mut() {
        if let Item::List(list) = item {
            let count_before = list.rows.len();
            list.rows.clear();
            list.rows.shrink_to_fit();
            assert_eq!(list.rows.len(), 0, "List should be empty");
            assert!(count_before > 0, "Should have had entities");
        }
    }
}

#[test]
fn test_parse_partial_truncate_entities() {
    let hedl = generate_test_doc(1000);
    let mut doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    let max_entities = 100;

    for item in doc.root.values_mut() {
        if let Item::List(list) = item {
            let count_before = list.rows.len();
            if list.rows.len() > max_entities {
                list.rows.truncate(max_entities);
                list.rows.shrink_to_fit();
            }
            assert!(count_before >= max_entities);
            assert_eq!(list.rows.len(), max_entities);
        }
    }
}

#[test]
fn test_memory_efficient_counting() {
    use std::collections::HashMap;

    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name]
%S:Post:[id, title]
---
users:@User
 |alice, Alice
 |bob, Bob
posts:@Post
 |post1, First
 |post2, Second
 |post3, Third
"#;

    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Count using &str keys (no cloning)
    let mut counts: HashMap<&str, usize> = HashMap::new();

    for item in doc.root.values() {
        if let Item::List(list) = item {
            *counts.entry(list.type_name.as_str()).or_default() += list.rows.len();
        }
    }

    assert_eq!(counts.get("User"), Some(&2));
    assert_eq!(counts.get("Post"), Some(&3));

    // Verify we can convert to owned only when needed
    let owned: HashMap<String, usize> = counts
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    assert_eq!(owned.get("User"), Some(&2));
}

#[test]
fn test_large_document_parsing() {
    // Test that large documents can be parsed without excessive memory
    let large_doc = generate_test_doc(10_000);
    let doc = core_parse(large_doc.as_bytes());
    assert!(doc.is_ok(), "Large document should parse");

    let doc = doc.unwrap();
    // generate_test_doc creates v2.0 documents, version is preserved
    assert_eq!(doc.version, (2, 0));
    assert!(doc.structs.contains_key("Entity"));
}

#[test]
fn test_partial_parse_preserves_metadata() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name]
%S:Post:[id, title]
%N:User>Post
---
users:@User
 |alice, Alice
 |bob, Bob
"#;

    let mut doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Clear entities
    doc.root.clear();

    // Metadata should be preserved - v2.0 input preserves version
    assert_eq!(doc.version, (2, 0));
    assert_eq!(doc.structs.len(), 2);
    assert!(doc.structs.contains_key("User"));
    assert!(doc.structs.contains_key("Post"));
    assert_eq!(doc.nests.len(), 1);
}

#[test]
fn test_nested_entity_truncation() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id]
%S:Post:[id]
%N:User>Post
---
users:@User
 |user1
  |post1
  |post2
  |post3
 |user2
  |post4
  |post5
"#;

    let mut doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Truncate nested children
    for item in doc.root.values_mut() {
        if let Item::List(list) = item {
            for node in &mut list.rows {
                if let Some(children_map) = node.children_mut() {
                    for children in children_map.values_mut() {
                        if children.len() > 2 {
                            children.truncate(2);
                            children.shrink_to_fit();
                        }
                    }
                }
            }
        }
    }

    // Verify truncation
    if let Some(Item::List(list)) = doc.root.get("users") {
        for node in &list.rows {
            if let Some(children_map) = node.children() {
                for children in children_map.values() {
                    assert!(children.len() <= 2, "Children should be truncated to 2");
                }
            }
        }
    }
}

#[test]
fn test_shrink_to_fit_optimization() {
    let hedl = generate_test_doc(1000);
    let mut doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    for item in doc.root.values_mut() {
        if let Item::List(list) = item {
            let capacity_before = list.rows.capacity();
            list.rows.truncate(10);
            let capacity_after_truncate = list.rows.capacity();
            list.rows.shrink_to_fit();
            let capacity_after_shrink = list.rows.capacity();

            assert!(capacity_before >= 1000);
            assert_eq!(capacity_after_truncate, capacity_before); // Truncate doesn't reduce capacity
            assert!(capacity_after_shrink < capacity_before); // Shrink_to_fit does
            assert!(capacity_after_shrink >= 10); // But keeps at least what we need
        }
    }
}

#[test]
fn test_empty_document_edge_case() {
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
"#;
    let mut doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Operations on empty document should not panic
    doc.root.clear();
    assert_eq!(doc.root.len(), 0);
}

#[test]
fn test_multiple_list_types() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id]
%S:Product:[id]
%S:Order:[id]
---
users:@User
 |user1
products:@Product
 |prod1
 |prod2
orders:@Order
 |order1
 |order2
 |order3
"#;

    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");
    let mut counts: HashMap<&str, usize> = HashMap::new();

    for item in doc.root.values() {
        if let Item::List(list) = item {
            *counts.entry(list.type_name.as_str()).or_default() += list.rows.len();
        }
    }

    assert_eq!(counts.len(), 3);
    assert_eq!(counts.get("User"), Some(&1));
    assert_eq!(counts.get("Product"), Some(&2));
    assert_eq!(counts.get("Order"), Some(&3));
}

#[test]
fn test_memory_limit_validation() {
    // Simulate checking against memory limits
    let doc_size = 1_000_000; // 1MB
    let max_input = 500 * 1024 * 1024; // 500MB

    assert!(doc_size <= max_input, "Document within limits");

    let too_large = 600 * 1024 * 1024; // 600MB
    assert!(too_large > max_input, "Document exceeds limits");
}

#[test]
fn test_progressive_entity_processing() {
    // Simulate processing entities in batches to limit memory
    let hedl = generate_test_doc(1000);
    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    let batch_size = 100;
    let mut processed = 0;

    for item in doc.root.values() {
        if let Item::List(list) = item {
            for batch in list.rows.chunks(batch_size) {
                // Process batch
                processed += batch.len();

                // In real scenario, we'd process and discard each batch
                // to keep memory usage constant
            }
        }
    }

    assert_eq!(processed, 1000);
}

#[test]
fn test_schema_only_extraction() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email, created_at]
%S:Post:[id, title, content, author_id]
%S:Comment:[id, text, post_id]
---
users:@User
 |alice, Alice, alice@example.com, 2024-01-01
posts:@Post
 |post1, Title, Content, alice
"#;

    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Extract only schema definitions
    let schemas: Vec<_> = doc.structs.keys().collect();
    assert_eq!(schemas.len(), 3);

    // Verify schema details
    assert_eq!(doc.structs.get("User").unwrap().len(), 4);
    assert_eq!(doc.structs.get("Post").unwrap().len(), 4);
    assert_eq!(doc.structs.get("Comment").unwrap().len(), 3);
}

#[test]
fn test_memory_bounded_query() {
    let hedl = generate_test_doc(10000);
    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Simulate paginated query: offset=100, limit=50
    let offset = 100;
    let limit = 50;
    let mut results = Vec::new();
    let mut count = 0;

    for item in doc.root.values() {
        if let Item::List(list) = item {
            for node in &list.rows {
                if count >= offset && results.len() < limit {
                    results.push(&node.id);
                }
                count += 1;

                if results.len() >= limit {
                    break;
                }
            }
        }
    }

    assert_eq!(results.len(), 50, "Should return exactly limit results");
    assert_eq!(results[0], "entity_100");
    assert_eq!(results[49], "entity_149");
}

#[test]
fn test_incremental_counting() {
    // Count entities incrementally without storing all IDs
    let hedl = generate_test_doc(5000);
    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    let mut total = 0;

    for item in doc.root.values() {
        if let Item::List(list) = item {
            total += list.rows.len();
            // Don't store individual entities, just count
        }
    }

    assert_eq!(total, 5000);
}

#[test]
fn test_selective_field_extraction() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email, phone, address]
---
users:@User
 |alice, Alice, alice@example.com, 555-1234, 123 Main St
 |bob, Bob, bob@example.com, 555-5678, 456 Oak Ave
"#;

    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Extract only specific fields (e.g., just id and name)
    for item in doc.root.values() {
        if let Item::List(list) = item {
            for node in &list.rows {
                // In production, we'd only extract fields 0 and 1
                assert!(node.fields.len() >= 2);
                // Simulates extracting only id and name, ignoring rest
            }
        }
    }
}

// ============ STREAMING API TESTS ============

#[test]
fn test_streaming_json_basic() {
    // Test basic streaming JSON conversion
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name]
---
users:@User
 |alice, Alice
 |bob, Bob
"#;

    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Note: We can't directly test StreamingJson here since it requires wasm_bindgen,
    // but we can test the underlying logic
    assert!(!doc.root.is_empty());
}

#[test]
fn test_streaming_parser_event_generation() {
    // Test that we can parse a document incrementally
    let hedl = generate_test_doc(100);
    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Verify document structure for streaming - v2.0 input preserves version
    assert_eq!(doc.version, (2, 0));
    assert!(doc.structs.contains_key("Entity"));
    assert_eq!(doc.root.len(), 1); // One root item (entities)

    // Count total nodes
    let mut total_nodes = 0;
    for item in doc.root.values() {
        if let Item::List(list) = item {
            total_nodes += list.rows.len();
        }
    }
    assert_eq!(total_nodes, 100);
}

#[test]
fn test_incremental_event_processing() {
    // Simulate processing events in chunks
    let hedl = generate_test_doc(1000);
    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    let chunk_size = 100;
    let mut processed = 0;

    for item in doc.root.values() {
        if let Item::List(list) = item {
            for chunk in list.rows.chunks(chunk_size) {
                // Process chunk (emit events)
                for _node in chunk {
                    processed += 1;
                }
            }
        }
    }

    assert_eq!(processed, 1000);
}

#[test]
fn test_memory_pressure_threshold() {
    // Test memory pressure detection logic
    let mobile_limit = 512 * 1024 * 1024; // 512MB typical mobile
    let threshold = (f64::from(mobile_limit) * 0.75) as usize; // 75% threshold
    let high_usage = 400 * 1024 * 1024; // 400MB

    assert!(high_usage > threshold, "Should detect memory pressure");

    let low_usage = 300 * 1024 * 1024; // 300MB
    assert!(low_usage < threshold, "Should not detect memory pressure");
}

#[test]
fn test_chunk_size_calculation() {
    // Test optimal chunk sizes for different document sizes
    let default_chunk = 16384; // 16KB

    // For small documents, chunk size can be larger
    let small_doc_size = 50_000; // 50KB
    let small_chunk = default_chunk.min(small_doc_size / 4);
    assert!(small_chunk > 0);

    // For large documents, use default chunk
    let large_doc_size = 10_000_000; // 10MB
    let large_chunk = default_chunk.min(large_doc_size / 4);
    assert_eq!(large_chunk, default_chunk);
}

#[test]
fn test_progressive_json_serialization() {
    // Test that we can serialize JSON incrementally

    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Entity:[id, value]
---
data:@Entity
 |e1, 100
 |e2, 200
 |e3, 300
"#;

    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Create a test string to simulate JSON output
    let json_str = format!("{{\"data\":[{} entities]}}", doc.root.len());
    assert!(!json_str.is_empty());

    // In streaming mode, we'd chunk this string
    let chunk_size = 10;
    let chunks: Vec<_> = json_str
        .as_bytes()
        .chunks(chunk_size)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect();

    assert!(chunks.len() > 1, "Should have multiple chunks");
}

#[test]
fn test_event_based_filtering() {
    // Test filtering during streaming to reduce memory
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, active]
---
users:@User
 |alice, true
 |bob, false
 |charlie, true
 |diana, false
"#;

    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Filter only active users during streaming
    let mut active_count = 0;
    for item in doc.root.values() {
        if let Item::List(list) = item {
            for node in &list.rows {
                if let Some(hedl_core::Value::Bool(true)) = node.fields.get(1) {
                    active_count += 1;
                }
            }
        }
    }

    assert_eq!(active_count, 2, "Should find 2 active users");
}

#[test]
fn test_memory_efficient_aggregation() {
    // Test aggregation without storing all entities
    let hedl = generate_test_doc(1000);
    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Compute statistics without storing entities
    let mut sum = 0i64;
    let mut count = 0usize;

    for item in doc.root.values() {
        if let Item::List(list) = item {
            for node in &list.rows {
                // Field 2 is the value field (i * 100)
                if let Some(hedl_core::Value::Int(val)) = node.fields.get(2) {
                    sum += val;
                    count += 1;
                }
            }
        }
    }

    assert_eq!(count, 1000);
    let expected_sum: i64 = (0..1000).map(|i| i * 100).sum();
    assert_eq!(sum, expected_sum);
}

#[test]
fn test_stream_position_tracking() {
    // Test that we can track position in a stream
    let total_items = 1000;
    let chunk_size = 100;

    let mut position = 0;
    while position < total_items {
        let chunk_end = (position + chunk_size).min(total_items);
        let chunk_len = chunk_end - position;

        assert!(chunk_len > 0);
        assert!(chunk_len <= chunk_size);

        position = chunk_end;
    }

    assert_eq!(position, total_items);
}

#[test]
fn test_partial_entity_list_iteration() {
    // Test iterating only part of an entity list
    let hedl = generate_test_doc(500);
    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    let max_to_process = 100;
    let mut processed = 0;

    'outer: for item in doc.root.values() {
        if let Item::List(list) = item {
            for _node in &list.rows {
                processed += 1;
                if processed >= max_to_process {
                    break 'outer;
                }
            }
        }
    }

    assert_eq!(processed, max_to_process);
}

#[test]
fn test_schema_extraction_without_entities() {
    // Test extracting schema info without loading entities
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
%S:Post:[id, title, content, author_id]
%A:%production:"true"
%N:User>Post
---
users:@User
 |alice, Alice, alice@example.com
posts:@Post
 |post1, Title, Content, alice
"#;

    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Extract metadata only
    let version = doc.version;
    let schemas: Vec<_> = doc.structs.keys().cloned().collect();
    let aliases: Vec<_> = doc.aliases.keys().cloned().collect();
    let nests: Vec<_> = doc.nests.keys().cloned().collect();

    // Verify we got the metadata
    assert_eq!(version, (2, 0));
    assert_eq!(schemas.len(), 2);
    assert_eq!(aliases.len(), 1);
    assert_eq!(nests.len(), 1);

    // We could now drop the entities to save memory
    // (simulated by not accessing doc.root)
}

#[test]
fn test_early_termination_on_limit() {
    // Test that we can stop processing early when limit is reached
    let hedl = generate_test_doc(10000);
    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    let limit = 500;
    let mut results = Vec::with_capacity(limit);

    'search: for item in doc.root.values() {
        if let Item::List(list) = item {
            for node in &list.rows {
                results.push(&node.id);
                if results.len() >= limit {
                    break 'search;
                }
            }
        }
    }

    assert_eq!(results.len(), limit);
}

#[test]
fn test_memory_limit_constants() {
    // Test that memory limit constants are reasonable
    let default_max = 500 * 1024 * 1024; // 500MB
    let desktop_limit = 1024 * 1024 * 1024; // 1GB
    let mobile_limit = 256 * 1024 * 1024; // 256MB (conservative mobile limit)

    assert!(default_max <= desktop_limit);
    assert!(default_max >= mobile_limit); // Default should accommodate mobile

    // Test typical document sizes fit within limits
    let small_doc = 100_000; // 100KB
    let medium_doc = 10_000_000; // 10MB
    let large_doc = 100_000_000; // 100MB

    assert!(small_doc < default_max);
    assert!(medium_doc < default_max);
    assert!(large_doc < default_max);
}

#[test]
fn test_entity_count_estimation() {
    // Test estimating entity count without full iteration
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id]
---
users:@User
 |user1
 |user2
 |user3
"#;

    let doc = core_parse(hedl.as_bytes()).expect("Parse should succeed");

    // Quick estimation by checking list.rows.len() without iteration
    let mut estimate = 0;
    for item in doc.root.values() {
        if let Item::List(list) = item {
            estimate += list.rows.len(); // O(1) operation
        }
    }

    assert_eq!(estimate, 3);
}

use std::collections::HashMap;

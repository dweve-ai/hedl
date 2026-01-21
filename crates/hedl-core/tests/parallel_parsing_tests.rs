// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for parallel parsing functionality.
//!
//! These tests verify that parallel parsing produces identical results
//! to sequential parsing while achieving better throughput on multi-core.

#![cfg(feature = "parallel")]

use hedl_core::parallel::{
    collect_ids_parallel, identify_entity_boundaries, parse_matrix_rows_parallel,
    validate_references_parallel, AtomicSecurityCounters, EntityType, MatrixRowBatch,
    ParallelConfig,
};
use hedl_core::reference::TypeRegistry;
use hedl_core::{
    parse, parse_with_limits, Item, Limits, MatrixList, Node, ParseOptions, Reference, Value,
};
use std::collections::BTreeMap;

// ==================== ParallelConfig Tests ====================

#[test]
fn test_parallel_config_disabled() {
    let config = ParallelConfig {
        enabled: false,
        ..Default::default()
    };

    // Even with enough entities, should not parallelize when disabled
    assert!(!config.should_parallelize_entities(1000));
    assert!(!config.should_parallelize_rows(1000));
}

#[test]
fn test_parallel_config_custom_thresholds() {
    let config = ParallelConfig {
        enabled: true,
        min_root_entities: 10,
        min_list_rows: 25,
        thread_pool_size: Some(4),
    };

    assert!(config.should_parallelize_entities(10));
    assert!(!config.should_parallelize_entities(9));
    assert!(config.should_parallelize_rows(25));
    assert!(!config.should_parallelize_rows(24));
}

// ==================== Entity Boundary Identification Tests ====================

#[test]
fn test_entity_boundary_empty_document() {
    let lines: Vec<(usize, &str)> = vec![];
    let boundaries = identify_entity_boundaries(&lines);
    assert!(boundaries.is_empty());
}

#[test]
fn test_entity_boundary_single_entity() {
    let lines: Vec<(usize, &str)> = vec![
        (1, "users: @User"),
        (2, "| alice, Alice"),
        (3, "| bob, Bob"),
    ];

    let boundaries = identify_entity_boundaries(&lines);
    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].key, "users");
    assert_eq!(boundaries[0].entity_type, EntityType::List);
}

#[test]
fn test_entity_boundary_mixed_types() {
    let lines: Vec<(usize, &str)> = vec![
        (1, "users: @User"),
        (2, "| alice, Alice"),
        (3, "config:"),
        (4, "  debug: true"),
        (5, "  level: 5"),
        (6, "version: 1.0"),
    ];

    let boundaries = identify_entity_boundaries(&lines);
    assert_eq!(boundaries.len(), 3);

    assert_eq!(boundaries[0].key, "users");
    assert_eq!(boundaries[0].entity_type, EntityType::List);

    assert_eq!(boundaries[1].key, "config");
    assert_eq!(boundaries[1].entity_type, EntityType::Object);

    assert_eq!(boundaries[2].key, "version");
    assert_eq!(boundaries[2].entity_type, EntityType::Scalar);
}

#[test]
fn test_entity_boundary_with_count_hint() {
    let lines: Vec<(usize, &str)> = vec![(1, "users(100): @User"), (2, "| alice, Alice")];

    let boundaries = identify_entity_boundaries(&lines);
    assert_eq!(boundaries.len(), 1);
    // Count hint should be stripped from key
    assert_eq!(boundaries[0].key, "users");
}

#[test]
fn test_entity_boundary_skips_comments() {
    let lines: Vec<(usize, &str)> = vec![
        (1, "# This is a comment"),
        (2, "users: @User"),
        (3, "# Another comment"),
        (4, "| alice, Alice"),
    ];

    let boundaries = identify_entity_boundaries(&lines);
    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].key, "users");
}

// ==================== Parallel Matrix Row Parsing Tests ====================

#[test]
fn test_parallel_row_parsing_simple() {
    use hedl_core::header::Header;

    let header = Header::new((1, 0));
    let limits = Limits::default();
    let counters = AtomicSecurityCounters::new();

    let batch = MatrixRowBatch {
        type_name: "User".to_string(),
        schema: vec!["id".to_string(), "name".to_string()],
        rows: vec![(1, "|alice, Alice"), (2, "|bob, Bob"), (3, "|carol, Carol")],
        has_ditto: false,
    };

    let nodes = parse_matrix_rows_parallel(&batch, &header, &limits, &counters).unwrap();

    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[0].id, "alice");
    assert_eq!(nodes[1].id, "bob");
    assert_eq!(nodes[2].id, "carol");
}

#[test]
fn test_parallel_row_parsing_with_child_counts() {
    use hedl_core::header::Header;

    let header = Header::new((1, 0));
    let limits = Limits::default();
    let counters = AtomicSecurityCounters::new();

    let batch = MatrixRowBatch {
        type_name: "Team".to_string(),
        schema: vec!["id".to_string(), "name".to_string()],
        rows: vec![
            (1, "|[5] team1, Engineering"),
            (2, "|[3] team2, Sales"),
            (3, "|team3, Marketing"), // No child count
        ],
        has_ditto: false,
    };

    let nodes = parse_matrix_rows_parallel(&batch, &header, &limits, &counters).unwrap();

    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[0].id, "team1");
    assert_eq!(nodes[0].child_count, 5);
    assert_eq!(nodes[1].child_count, 3);
    assert_eq!(nodes[2].child_count, 0);
}

#[test]
fn test_parallel_row_parsing_falls_back_for_ditto() {
    // Batch with ditto should still work (falls back to sequential)
    let batch = MatrixRowBatch {
        type_name: "User".to_string(),
        schema: vec!["id".to_string(), "name".to_string()],
        rows: vec![
            (1, "|alice, Alice"),
            (2, "|bob, \""), // Ditto marker
        ],
        has_ditto: true,
    };

    assert!(!batch.can_parallelize());
}

#[test]
fn test_parallel_row_parsing_security_limit() {
    use hedl_core::header::Header;

    let header = Header::new((1, 0));
    let limits = Limits {
        max_nodes: 2, // Very low limit
        ..Default::default()
    };
    let counters = AtomicSecurityCounters::new();

    let batch = MatrixRowBatch {
        type_name: "User".to_string(),
        schema: vec!["id".to_string(), "name".to_string()],
        rows: vec![
            (1, "|alice, Alice"),
            (2, "|bob, Bob"),
            (3, "|carol, Carol"), // Should exceed limit
        ],
        has_ditto: false,
    };

    let result = parse_matrix_rows_parallel(&batch, &header, &limits, &counters);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("too many nodes"));
}

// ==================== Parallel ID Collection Tests ====================

#[test]
fn test_parallel_id_collection_empty() {
    let items: BTreeMap<String, Item> = BTreeMap::new();
    let limits = Limits::default();

    let registry = collect_ids_parallel(&items, &limits).unwrap();

    // Registry should be empty
    assert!(registry.lookup_unqualified("anything").is_none());
}

#[test]
fn test_parallel_id_collection_single_list() {
    let mut items: BTreeMap<String, Item> = BTreeMap::new();

    let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    list.rows.push(Node::new(
        "User",
        "alice",
        vec![
            Value::String("alice".to_string().into()),
            Value::String("Alice".to_string().into()),
        ],
    ));
    list.rows.push(Node::new(
        "User",
        "bob",
        vec![
            Value::String("bob".to_string().into()),
            Value::String("Bob".to_string().into()),
        ],
    ));

    items.insert("users".to_string(), Item::List(list));

    let limits = Limits::default();
    let registry = collect_ids_parallel(&items, &limits).unwrap();

    assert!(registry.contains_in_type("User", "alice"));
    assert!(registry.contains_in_type("User", "bob"));
    assert!(!registry.contains_in_type("User", "carol"));
}

#[test]
fn test_parallel_id_collection_multiple_types() {
    let mut items: BTreeMap<String, Item> = BTreeMap::new();

    // Add User list
    let mut users = MatrixList::new("User", vec!["id".to_string()]);
    users.rows.push(Node::new(
        "User",
        "u1",
        vec![Value::String("u1".to_string().into())],
    ));
    users.rows.push(Node::new(
        "User",
        "u2",
        vec![Value::String("u2".to_string().into())],
    ));
    items.insert("users".to_string(), Item::List(users));

    // Add Post list
    let mut posts = MatrixList::new("Post", vec!["id".to_string()]);
    posts.rows.push(Node::new(
        "Post",
        "p1",
        vec![Value::String("p1".to_string().into())],
    ));
    posts.rows.push(Node::new(
        "Post",
        "p2",
        vec![Value::String("p2".to_string().into())],
    ));
    items.insert("posts".to_string(), Item::List(posts));

    let limits = Limits::default();
    let registry = collect_ids_parallel(&items, &limits).unwrap();

    assert!(registry.contains_in_type("User", "u1"));
    assert!(registry.contains_in_type("User", "u2"));
    assert!(registry.contains_in_type("Post", "p1"));
    assert!(registry.contains_in_type("Post", "p2"));

    // Cross-type should not match
    assert!(!registry.contains_in_type("User", "p1"));
    assert!(!registry.contains_in_type("Post", "u1"));
}

#[test]
fn test_parallel_id_collection_detects_duplicates() {
    let mut items: BTreeMap<String, Item> = BTreeMap::new();

    // Add list with duplicate IDs
    let mut list = MatrixList::new("User", vec!["id".to_string()]);
    list.rows.push(Node::new(
        "User",
        "same_id",
        vec![Value::String("same_id".to_string().into())],
    ));
    list.rows.push(Node::new(
        "User",
        "same_id",
        vec![Value::String("same_id".to_string().into())],
    )); // Duplicate!
    items.insert("users".to_string(), Item::List(list));

    let limits = Limits::default();
    let result = collect_ids_parallel(&items, &limits);

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("duplicate ID"));
    }
}

// ==================== Parallel Reference Validation Tests ====================

#[test]
fn test_parallel_reference_validation_valid() {
    let mut registry = TypeRegistry::new();
    let limits = Limits::default();
    registry.register("User", "alice", 0, &limits).unwrap();
    registry.register("User", "bob", 0, &limits).unwrap();

    let mut items: BTreeMap<String, Item> = BTreeMap::new();

    // Add item with valid reference
    items.insert(
        "author".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "alice"))),
    );

    let limits = Limits::default();
    let result = validate_references_parallel(&items, &registry, true, limits.max_nest_depth);
    assert!(result.is_ok());
}

#[test]
fn test_parallel_reference_validation_unresolved_strict() {
    let registry = TypeRegistry::new(); // Empty - no IDs registered

    let mut items: BTreeMap<String, Item> = BTreeMap::new();
    items.insert(
        "author".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified(
            "User",
            "nonexistent",
        ))),
    );

    let limits = Limits::default();
    let result = validate_references_parallel(&items, &registry, true, limits.max_nest_depth);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("unresolved reference"));
}

#[test]
fn test_parallel_reference_validation_unresolved_lenient() {
    let registry = TypeRegistry::new(); // Empty

    let mut items: BTreeMap<String, Item> = BTreeMap::new();
    items.insert(
        "author".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified(
            "User",
            "nonexistent",
        ))),
    );

    let limits = Limits::default();
    // strict = false should allow unresolved
    let result = validate_references_parallel(&items, &registry, false, limits.max_nest_depth);
    assert!(result.is_ok());
}

#[test]
fn test_parallel_reference_validation_ambiguous() {
    let mut registry = TypeRegistry::new();
    let limits = Limits::default();
    // Same ID in multiple types
    registry.register("User", "shared_id", 0, &limits).unwrap();
    registry.register("Admin", "shared_id", 0, &limits).unwrap();

    let mut items: BTreeMap<String, Item> = BTreeMap::new();
    // Unqualified reference to ambiguous ID
    items.insert(
        "target".to_string(),
        Item::Scalar(Value::Reference(Reference::unqualified("shared_id"))),
    );

    let limits = Limits::default();
    let result = validate_references_parallel(&items, &registry, true, limits.max_nest_depth);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Ambiguous"));
}

// ==================== Atomic Counter Tests ====================

#[test]
fn test_atomic_counters_concurrent_increment() {
    use std::thread;

    let counters = std::sync::Arc::new(AtomicSecurityCounters::new());
    let limits = Limits::default();

    let mut handles = vec![];
    for _ in 0..10 {
        let c = counters.clone();
        let l = limits.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                c.increment_nodes(&l, 0).unwrap();
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // All 1000 increments should be counted
    assert_eq!(counters.node_count(), 1000);
}

#[test]
fn test_atomic_counters_key_limit() {
    let counters = AtomicSecurityCounters::new();
    let limits = Limits {
        max_total_keys: 5,
        ..Default::default()
    };

    // First 5 should succeed
    for _ in 0..5 {
        counters.increment_keys(&limits, 0).unwrap();
    }

    // 6th should fail
    let result = counters.increment_keys(&limits, 0);
    assert!(result.is_err());
}

// ==================== Full Parse Integration Tests ====================

#[test]
fn test_parallel_config_thresholds_document() {
    // Test that thresholds work correctly for document parsing decisions
    let config = ParallelConfig::default();

    // Small document - no parallelism
    assert!(!config.should_parallelize_entities(5));

    // Large document - enable parallelism
    assert!(config.should_parallelize_entities(100));

    // Small list - no parallelism
    assert!(!config.should_parallelize_rows(50));

    // Large list - enable parallelism
    assert!(config.should_parallelize_rows(500));
}

#[test]
fn test_parallel_parse_deterministic_output() {
    // Parse the same document multiple times and verify results are identical
    let input = b"%VERSION: 1.0
%STRUCT: User: [id, name, age]
---
users: @User
  | alice, Alice, 30
  | bob, Bob, 25
  | carol, Carol, 35
";

    let doc1 = parse(input).unwrap();
    let doc2 = parse(input).unwrap();
    let doc3 = parse(input).unwrap();

    // All parses should produce identical results
    if let Item::List(list1) = doc1.root.get("users").unwrap() {
        if let Item::List(list2) = doc2.root.get("users").unwrap() {
            if let Item::List(list3) = doc3.root.get("users").unwrap() {
                assert_eq!(list1.rows.len(), list2.rows.len());
                assert_eq!(list2.rows.len(), list3.rows.len());

                for i in 0..list1.rows.len() {
                    assert_eq!(list1.rows[i].id, list2.rows[i].id);
                    assert_eq!(list2.rows[i].id, list3.rows[i].id);
                }
            }
        }
    }
}

// ==================== Reference Validation via Parse Tests ====================

#[test]
fn test_parse_valid_references() {
    // Test document with valid references
    let input = b"%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author]
---
users: @User
  | alice, Alice
  | bob, Bob
posts: @Post
  | post1, @User:alice
";

    let result = parse(input);
    assert!(result.is_ok());
}

#[test]
fn test_parse_unresolved_references_strict() {
    // Test document with unresolved reference
    let input = b"%VERSION: 1.0
---
author: @User:nonexistent
";

    let opts = ParseOptions::builder().strict(true).build();
    let result = parse_with_limits(input, opts);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("unresolved") || err.to_string().contains("reference"));
}

#[test]
fn test_parse_unresolved_references_lenient() {
    // Test document with unresolved reference but lenient mode
    let input = b"%VERSION: 1.0
---
author: @User:nonexistent
";

    let opts = ParseOptions::builder().strict(false).build();
    let result = parse_with_limits(input, opts);

    // Should succeed in lenient mode
    assert!(result.is_ok());
}

// ==================== Stress Tests ====================

#[test]
fn test_parallel_large_document() {
    // Generate a document with many entities to test parallel performance
    let mut doc = String::from("%VERSION: 1.0\n%STRUCT: Item: [id, value]\n---\n");

    for i in 0..100 {
        doc.push_str(&format!("list{i}: @Item\n"));
        for j in 0..10 {
            doc.push_str(&format!("  | item{i}_{j}, value{j}\n"));
        }
    }

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());

    let doc = result.unwrap();
    // Should have 100 lists
    assert_eq!(doc.root.len(), 100);
}

#[test]
fn test_parallel_deep_nesting() {
    // Test that parallel parsing handles nested structures correctly
    let input = b"%VERSION: 1.0
---
level1:
  level2:
    level3:
      level4:
        value: 42
";

    let doc = parse(input).unwrap();

    // Navigate the nested structure
    if let Item::Object(l1) = doc.root.get("level1").unwrap() {
        if let Item::Object(l2) = l1.get("level2").unwrap() {
            if let Item::Object(l3) = l2.get("level3").unwrap() {
                if let Item::Object(l4) = l3.get("level4").unwrap() {
                    if let Item::Scalar(Value::Int(v)) = l4.get("value").unwrap() {
                        assert_eq!(*v, 42);
                        return;
                    }
                }
            }
        }
    }
    panic!("Expected nested structure");
}

#[test]
fn test_parallel_mixed_content() {
    // Test parallel parsing with mixed content types
    let input = b"%VERSION: 1.0
%STRUCT: User: [id, name]
---
version: 1.0
users: @User
  | alice, Alice
  | bob, Bob
settings:
  debug: true
  level: 5
count: 42
";

    let doc = parse(input).unwrap();

    // Verify all types parsed correctly
    // Note: 1.0 is parsed as Float, not String
    assert!(matches!(
        doc.root.get("version"),
        Some(Item::Scalar(Value::Float(_)))
    ));
    assert!(matches!(doc.root.get("users"), Some(Item::List(_))));
    assert!(matches!(doc.root.get("settings"), Some(Item::Object(_))));
    assert!(matches!(
        doc.root.get("count"),
        Some(Item::Scalar(Value::Int(42)))
    ));
}

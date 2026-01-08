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

//! Stress tests for hedl-core parsing and traversal.
//!
//! These tests verify behavior under extreme conditions:
//! - Large documents (10K+ rows)
//! - Deeply nested structures
//! - Maximum key counts
//! - Concurrent parsing
//! - Memory pressure scenarios
//! - Various limits configurations

use hedl_core::{
    parse, parse_with_limits, traverse, HedlErrorKind, Limits, ParseOptions, StatsCollector,
};
use std::sync::Arc;
use std::thread;

// =============================================================================
// Large Document Tests
// =============================================================================

#[test]
fn test_parse_10k_rows() {
    let mut doc = String::from("%VERSION: 1.0\n%STRUCT: Record: [id, value]\n---\ndata: @Record\n");

    // Generate 10,000 rows
    for i in 0..10_000 {
        doc.push_str(&format!("  | record-{}, value-{}\n", i, i));
    }

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());

    let parsed = result.unwrap();
    let list = parsed.get("data").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 10_000);
}

#[test]
fn test_parse_50k_rows() {
    let mut doc = String::from("%VERSION: 1.0\n%STRUCT: Record: [id, value, count]\n---\ndata: @Record\n");

    // Generate 50,000 rows
    for i in 0..50_000 {
        doc.push_str(&format!("  | row-{}, data-{}, {}\n", i, i, i % 100));
    }

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());

    let parsed = result.unwrap();
    let list = parsed.get("data").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 50_000);
}

#[test]
fn test_parse_100k_scalars() {
    // Create document with 100K scalar key-value pairs
    let mut doc = String::from("%VERSION: 1.0\n---\n");

    for i in 0..100_000 {
        doc.push_str(&format!("key{}: value{}\n", i, i));
    }

    // Use unlimited limits for stress testing
    let result = parse_with_limits(
        doc.as_bytes(),
        ParseOptions {
            limits: Limits::unlimited(),
            strict_refs: false,
        },
    );
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert_eq!(parsed.root.len(), 100_000);
}

#[test]
fn test_parse_wide_table() {
    // Table with 100 columns and 1000 rows
    let mut schema = vec!["id".to_string()];
    for i in 1..100 {
        schema.push(format!("col{}", i));
    }

    let mut doc = format!(
        "%VERSION: 1.0\n%STRUCT: Wide: [{}]\n---\ndata: @Wide\n",
        schema.join(", ")
    );

    for row in 0..1_000 {
        doc.push_str("  | ");
        doc.push_str(&format!("row-{}", row));
        for col in 1..100 {
            doc.push_str(&format!(", val-{}-{}", row, col));
        }
        doc.push('\n');
    }

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());

    let parsed = result.unwrap();
    let list = parsed.get("data").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 1_000);
    assert_eq!(list.schema.len(), 100);
}

// =============================================================================
// Deeply Nested Structure Tests
// =============================================================================

#[test]
fn test_deeply_nested_objects() {
    // Create nested objects up to depth 50 (the default limit)
    let mut doc = String::from("%VERSION: 1.0\n---\n");

    for i in 0..50 {
        doc.push_str(&" ".repeat(i * 2));
        doc.push_str(&format!("level{}:\n", i));
    }

    // Add a scalar at the deepest level
    doc.push_str(&" ".repeat(50 * 2));
    doc.push_str("value: 42\n");

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

#[test]
fn test_nested_objects_at_limit() {
    // Test at exactly the depth limit
    let limits = Limits {
        max_indent_depth: 10,
        ..Limits::default()
    };

    let mut doc = String::from("%VERSION: 1.0\n---\n");

    for i in 0..10 {
        doc.push_str(&" ".repeat(i * 2));
        doc.push_str(&format!("level{}:\n", i));
    }

    doc.push_str(&" ".repeat(10 * 2));
    doc.push_str("value: 42\n");

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });
    assert!(result.is_ok());
}

#[test]
fn test_nested_objects_exceeds_limit() {
    // Test exceeding the depth limit
    let limits = Limits {
        max_indent_depth: 5,
        ..Limits::default()
    };

    let mut doc = String::from("%VERSION: 1.0\n---\n");

    for i in 0..7 {
        doc.push_str(&" ".repeat(i * 2));
        doc.push_str(&format!("level{}:\n", i));
    }

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err().kind, HedlErrorKind::Security));
}

#[test]
fn test_deeply_nested_hierarchy() {
    // Create a deep NEST hierarchy
    let depth = 10;

    let mut doc = String::from("%VERSION: 1.0\n");

    // Define schemas for each level
    for i in 0..depth {
        doc.push_str(&format!("%STRUCT: Level{}: [id, name]\n", i));
    }

    // Define NEST relationships
    for i in 0..depth - 1 {
        doc.push_str(&format!("%NEST: Level{} > Level{}\n", i, i + 1));
    }

    doc.push_str("---\ndata: @Level0\n");

    // Create nested data
    for level in 0..depth {
        let indent = "  ".repeat(level + 1);
        doc.push_str(&format!("{}| node-{}, name-{}\n", indent, level, level));
    }

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());

    let parsed = result.unwrap();
    let mut stats = StatsCollector::default();
    traverse(&parsed, &mut stats).unwrap();

    assert_eq!(stats.node_count, depth);
}

#[test]
fn test_nested_hierarchy_at_limit() {
    let limits = Limits {
        max_nest_depth: 50,
        ..Limits::default()
    };

    let depth = 50;
    let mut doc = String::from("%VERSION: 1.0\n");

    for i in 0..depth {
        doc.push_str(&format!("%STRUCT: L{}: [id]\n", i));
    }

    for i in 0..depth - 1 {
        doc.push_str(&format!("%NEST: L{} > L{}\n", i, i + 1));
    }

    doc.push_str("---\ndata: @L0\n");

    for level in 0..depth {
        let indent = "  ".repeat(level + 1);
        doc.push_str(&format!("{}| node-{}\n", indent, level));
    }

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });
    assert!(result.is_ok());
}

/// Test that NEST depth limit is enforced during parsing to prevent DoS attacks.
///
/// This test validates the security fix for the NEST depth limit vulnerability.
/// Without this check, an attacker could craft a HEDL document with excessive
/// nesting depth, causing stack overflow or memory exhaustion.
///
/// # Security Context
///
/// The parser maintains a stack of frames during parsing. Each nested NEST level
/// adds a new List frame to the stack. By limiting the depth, we prevent:
/// - Stack overflow from recursive parsing
/// - Excessive memory consumption from deep hierarchies
/// - Potential infinite loops in malformed documents
///
/// # Test Strategy
///
/// 1. Set a low max_nest_depth limit (5 levels)
/// 2. Create a NEST hierarchy deeper than the limit (10 levels)
/// 3. Verify parsing fails with a Security error
/// 4. Verify the error message contains depth information
#[test]
fn test_nest_depth_limit_enforced() {
    // Set a restrictive depth limit for testing
    let limits = Limits {
        max_nest_depth: 5,
        ..Limits::default()
    };

    // Create a deeper hierarchy than the limit allows
    let depth = 10;
    let mut doc = String::from("%VERSION: 1.0\n");

    // Define schemas for each level
    for i in 0..depth {
        doc.push_str(&format!("%STRUCT: Level{}: [id]\n", i));
    }

    // Define NEST relationships
    for i in 0..depth - 1 {
        doc.push_str(&format!("%NEST: Level{} > Level{}\n", i, i + 1));
    }

    doc.push_str("---\ndata: @Level0\n");

    // Create nested data - this should fail at depth 6 (exceeding limit of 5)
    for level in 0..depth {
        let indent = "  ".repeat(level + 1);
        doc.push_str(&format!("{}| node-{}\n", indent, level));
    }

    // Parse should fail with Security error
    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });

    assert!(result.is_err(), "Expected parsing to fail due to depth limit");

    let err = result.unwrap_err();
    assert!(
        matches!(err.kind, HedlErrorKind::Security),
        "Expected Security error, got: {:?}",
        err.kind
    );

    // Verify error message mentions depth
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("depth") || err_msg.contains("NEST"),
        "Error message should mention depth or NEST, got: {}",
        err_msg
    );
}

/// Test that NEST depth limit allows valid nested structures within the limit.
///
/// This is a complementary test to ensure the depth check doesn't incorrectly
/// reject valid documents that are within the configured limits.
#[test]
fn test_nest_depth_within_limit_succeeds() {
    // Set a depth limit
    let limits = Limits {
        max_nest_depth: 10,
        ..Limits::default()
    };

    // Create a hierarchy exactly at the limit
    let depth = 10;
    let mut doc = String::from("%VERSION: 1.0\n");

    for i in 0..depth {
        doc.push_str(&format!("%STRUCT: Level{}: [id]\n", i));
    }

    for i in 0..depth - 1 {
        doc.push_str(&format!("%NEST: Level{} > Level{}\n", i, i + 1));
    }

    doc.push_str("---\ndata: @Level0\n");

    // Create nested data at exactly the limit
    for level in 0..depth {
        let indent = "  ".repeat(level + 1);
        doc.push_str(&format!("{}| node-{}\n", indent, level));
    }

    // Parse should succeed
    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });

    assert!(result.is_ok(), "Expected parsing to succeed within depth limit");

    let parsed = result.unwrap();
    let mut stats = StatsCollector::default();
    traverse(&parsed, &mut stats).unwrap();

    assert_eq!(stats.node_count, depth);
}

/// Test edge case: depth limit of 1 (minimal nesting allowed).
///
/// Verifies that the depth check works correctly even with very restrictive limits.
#[test]
fn test_nest_depth_limit_minimal() {
    // Allow only 1 level of nesting
    let limits = Limits {
        max_nest_depth: 1,
        ..Limits::default()
    };

    let mut doc = String::from("%VERSION: 1.0\n");
    doc.push_str("%STRUCT: Parent: [id]\n");
    doc.push_str("%STRUCT: Child: [id]\n");
    doc.push_str("%NEST: Parent > Child\n");
    doc.push_str("---\ndata: @Parent\n");
    doc.push_str("  | parent-1\n");
    doc.push_str("    | child-1\n"); // Depth 2 - should fail

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });

    assert!(result.is_err(), "Expected parsing to fail with depth limit of 1");
    assert!(matches!(result.unwrap_err().kind, HedlErrorKind::Security));
}

// =============================================================================
// Maximum Key Count Tests
// =============================================================================

#[test]
fn test_max_object_keys() {
    // Create object with many keys
    let mut doc = String::from("%VERSION: 1.0\n---\nconfig:\n");

    for i in 0..1_000 {
        doc.push_str(&format!("  setting{}: value{}\n", i, i));
    }

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());

    let parsed = result.unwrap();
    let obj = parsed.get("config").unwrap().as_object().unwrap();
    assert_eq!(obj.len(), 1_000);
}

/// Test that max_object_keys limit is enforced per object.
///
/// This test validates that a single object cannot exceed the max_object_keys limit,
/// even if the total keys across all objects is within limits.
#[test]
fn test_max_object_keys_limit_enforced() {
    let limits = Limits {
        max_object_keys: 100,
        max_total_keys: 10_000, // High enough to not interfere
        ..Limits::default()
    };

    let mut doc = String::from("%VERSION: 1.0\n---\nconfig:\n");

    // Try to create 150 keys in one object (exceeds limit of 100)
    for i in 0..150 {
        doc.push_str(&format!("  setting{}: value{}\n", i, i));
    }

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });

    assert!(result.is_err(), "Expected parsing to fail due to max_object_keys limit");
    let err = result.unwrap_err();
    assert!(
        matches!(err.kind, HedlErrorKind::Security),
        "Expected Security error, got: {:?}",
        err.kind
    );

    let err_msg = err.to_string();
    assert!(
        err_msg.contains("too many keys") || err_msg.contains("object"),
        "Error message should mention keys or object, got: {}",
        err_msg
    );
}

/// Test that max_total_keys limit prevents DoS via many small objects.
///
/// This test validates the critical security feature that prevents attackers from
/// creating many small objects, each under max_object_keys, but collectively
/// consuming excessive memory.
///
/// # Attack Scenario
///
/// Without max_total_keys, an attacker could:
/// 1. Create 10,000 objects with 10 keys each (under max_object_keys limit)
/// 2. Total: 100,000 keys consuming significant memory
/// 3. Each object is "valid" but total memory usage is excessive
///
/// # Defense
///
/// The max_total_keys limit provides defense-in-depth by tracking cumulative
/// key count across all objects and rejecting documents that exceed the limit.
#[test]
fn test_max_total_keys_limit_enforced() {
    // Set limits: 10 keys per object, but only 50 total keys allowed
    let limits = Limits {
        max_object_keys: 10,
        max_total_keys: 50,
        ..Limits::default()
    };

    let mut doc = String::from("%VERSION: 1.0\n---\n");

    // Create 10 objects with 10 keys each = 100 total keys
    // This should fail when we hit the 51st key
    for obj_idx in 0..10 {
        doc.push_str(&format!("object{}:\n", obj_idx));
        for key_idx in 0..10 {
            doc.push_str(&format!("  key{}: value{}\n", key_idx, key_idx));
        }
    }

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });

    assert!(result.is_err(), "Expected parsing to fail due to max_total_keys limit");

    let err = result.unwrap_err();
    assert!(
        matches!(err.kind, HedlErrorKind::Security),
        "Expected Security error, got: {:?}",
        err.kind
    );

    let err_msg = err.to_string();
    assert!(
        err_msg.contains("total keys") || err_msg.contains("exceeds limit"),
        "Error message should mention total keys or limit, got: {}",
        err_msg
    );
}

/// Test that max_total_keys allows valid documents within the limit.
///
/// Ensures the limit check doesn't incorrectly reject valid documents.
#[test]
fn test_max_total_keys_within_limit_succeeds() {
    let limits = Limits {
        max_object_keys: 10,
        max_total_keys: 100,
        ..Limits::default()
    };

    let mut doc = String::from("%VERSION: 1.0\n---\n");

    // Create 9 objects with 10 keys each = 90 total keys (within limit)
    for obj_idx in 0..9 {
        doc.push_str(&format!("object{}:\n", obj_idx));
        for key_idx in 0..10 {
            doc.push_str(&format!("  key{}: value{}\n", key_idx, key_idx));
        }
    }

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });

    assert!(result.is_ok(), "Expected parsing to succeed within limits");

    let parsed = result.unwrap();
    assert_eq!(parsed.root.len(), 9, "Should have 9 top-level objects");
}

/// Test max_total_keys with nested objects.
///
/// Validates that the total key counter tracks keys across all nesting levels.
#[test]
fn test_max_total_keys_nested_objects() {
    let limits = Limits {
        max_object_keys: 20,
        max_total_keys: 40,
        ..Limits::default()
    };

    let mut doc = String::from("%VERSION: 1.0\n---\n");

    // Root level: 5 keys
    for i in 0..5 {
        doc.push_str(&format!("root_key{}: value{}\n", i, i));
    }

    // Nested object 1: 15 keys (total: 20)
    doc.push_str("nested1:\n");
    for i in 0..15 {
        doc.push_str(&format!("  nested1_key{}: value{}\n", i, i));
    }

    // Nested object 2: 15 keys (total: 35)
    doc.push_str("nested2:\n");
    for i in 0..15 {
        doc.push_str(&format!("  nested2_key{}: value{}\n", i, i));
    }

    // Try to add one more object with 10 keys (would exceed limit at 6th key)
    doc.push_str("nested3:\n");
    for i in 0..10 {
        doc.push_str(&format!("  nested3_key{}: value{}\n", i, i));
    }

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });

    assert!(result.is_err(), "Expected parsing to fail due to total keys limit across nesting");
    assert!(matches!(result.unwrap_err().kind, HedlErrorKind::Security));
}

/// Test max_total_keys at exactly the limit boundary.
///
/// Edge case testing to ensure off-by-one errors don't exist.
/// Note: Object keys themselves (object0, object1, etc.) also count toward the total!
#[test]
fn test_max_total_keys_at_exact_limit() {
    let limits = Limits {
        max_object_keys: 11,
        max_total_keys: 110,  // 10 objects + 100 nested keys
        ..Limits::default()
    };

    let mut doc = String::from("%VERSION: 1.0\n---\n");

    // Create exactly 110 total keys: 10 object keys + 100 nested keys
    for obj_idx in 0..10 {
        doc.push_str(&format!("object{}:\n", obj_idx));  // This counts as 1 key
        for key_idx in 0..10 {
            doc.push_str(&format!("  key{}: value{}\n", key_idx, key_idx));  // Each counts as 1 key
        }
    }

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });

    assert!(result.is_ok(), "Expected parsing to succeed at exact limit");
}

/// Test max_total_keys with overflow protection.
///
/// Validates that adding keys doesn't cause integer overflow.
#[test]
fn test_max_total_keys_overflow_protection() {
    let limits = Limits {
        max_object_keys: 1000,
        max_total_keys: usize::MAX, // Set to max to test overflow detection
        ..Limits::default()
    };

    // This test validates that the checked arithmetic in check_duplicate_key
    // prevents overflow. With normal operation, we can't trigger overflow,
    // but the code uses checked_add to be safe.
    let mut doc = String::from("%VERSION: 1.0\n---\n");

    // Create a reasonable number of keys (can't actually overflow in test)
    for i in 0..100 {
        doc.push_str(&format!("key{}: value{}\n", i, i));
    }

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });

    assert!(result.is_ok(), "Expected parsing to succeed with overflow protection");
}

/// Test that max_total_keys doesn't count keys in matrix list schemas.
///
/// Matrix list column names are schema metadata, not data keys,
/// so they shouldn't count against max_total_keys.
/// Note: The key "data" itself does count!
#[test]
fn test_max_total_keys_excludes_matrix_schemas() {
    let limits = Limits {
        max_object_keys: 20,
        max_total_keys: 20,  // 15 scalar keys + 1 list key "data" = 16 total
        ..Limits::default()
    };

    let mut doc = String::from("%VERSION: 1.0\n");
    doc.push_str("%STRUCT: Record: [id, name, age, city, country]\n");
    doc.push_str("---\n");

    // Add some object keys (count toward limit)
    for i in 0..15 {
        doc.push_str(&format!("key{}: value{}\n", i, i));
    }

    // Add a matrix list - the key "data" counts, but schema columns don't
    doc.push_str("data: @Record\n");  // "data" counts as 1 key (total: 16)
    doc.push_str("  | rec1, Alice, 30, NYC, USA\n");
    doc.push_str("  | rec2, Bob, 25, LA, USA\n");

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });

    // Should succeed because we only have 16 object keys (within limit of 20)
    assert!(result.is_ok(), "Matrix schema columns should not count toward max_total_keys");
}

/// Test max_total_keys with block strings.
///
/// Block string keys should count toward the limit.
#[test]
fn test_max_total_keys_includes_block_strings() {
    let limits = Limits {
        max_object_keys: 10,
        max_total_keys: 5,
        max_block_string_size: 1000,
        ..Limits::default()
    };

    let mut doc = String::from("%VERSION: 1.0\n---\n");

    // Add regular keys
    doc.push_str("key1: value1\n");
    doc.push_str("key2: value2\n");
    doc.push_str("key3: value3\n");
    doc.push_str("key4: value4\n");

    // Try to add a block string key (would be 5th key, at limit)
    doc.push_str("description: \"\"\"\n");
    doc.push_str("This is a block string\n");
    doc.push_str("with multiple lines\n");
    doc.push_str("\"\"\"\n");

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits: limits.clone(), strict_refs: true });

    assert!(result.is_ok(), "Block string keys should count toward total, but 5 is at limit");

    // Now try to add one more key (should fail)
    doc.push_str("extra: value\n");

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });
    assert!(result.is_err(), "Should fail when exceeding limit with extra key");
}

#[test]
fn test_max_aliases() {
    // Create document with many aliases (alias keys must start with %)
    let mut doc = String::from("%VERSION: 1.0\n");

    for i in 0..1_000 {
        doc.push_str(&format!("%ALIAS: %alias{}: \"value{}\"\n", i, i));
    }

    doc.push_str("---\ndata: 1\n");

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert_eq!(parsed.aliases.len(), 1_000);
}

#[test]
fn test_aliases_at_limit() {
    let limits = Limits {
        max_aliases: 100,
        ..Limits::default()
    };

    let mut doc = String::from("%VERSION: 1.0\n");

    for i in 0..100 {
        doc.push_str(&format!("%ALIAS: %alias{}: \"value{}\"\n", i, i));
    }

    doc.push_str("---\ndata: 1\n");

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });
    assert!(result.is_ok());
}

#[test]
fn test_aliases_exceeds_limit() {
    let limits = Limits {
        max_aliases: 50,
        ..Limits::default()
    };

    let mut doc = String::from("%VERSION: 1.0\n");

    for i in 0..100 {
        doc.push_str(&format!("%ALIAS: %alias{}: \"value{}\"\n", i, i));
    }

    doc.push_str("---\ndata: 1\n");

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err().kind, HedlErrorKind::Security));
}

// =============================================================================
// Concurrent Parsing Tests
// =============================================================================

#[test]
fn test_concurrent_parsing_small_docs() {
    let doc = Arc::new("%VERSION: 1.0\n%STRUCT: Item: [id, value]\n---\ndata: @Item\n  | item1, value1\n  | item2, value2\n");

    let mut handles = vec![];

    for _ in 0..10 {
        let doc_clone = Arc::clone(&doc);
        let handle = thread::spawn(move || {
            let result = parse(doc_clone.as_bytes());
            assert!(result.is_ok());
            result.unwrap()
        });
        handles.push(handle);
    }

    for handle in handles {
        let parsed = handle.join().unwrap();
        assert_eq!(parsed.version, (1, 0));
    }
}

#[test]
fn test_concurrent_parsing_large_docs() {
    // Generate a larger document
    let mut doc = String::from("%VERSION: 1.0\n%STRUCT: Record: [id, value, count]\n---\ndata: @Record\n");

    for i in 0..5_000 {
        doc.push_str(&format!("  | record-{}, value-{}, {}\n", i, i, i % 100));
    }

    let doc = Arc::new(doc);
    let mut handles = vec![];

    for _ in 0..4 {
        let doc_clone = Arc::clone(&doc);
        let handle = thread::spawn(move || {
            let result = parse(doc_clone.as_bytes());
            assert!(result.is_ok());
            let parsed = result.unwrap();
            let list = parsed.get("data").unwrap().as_list().unwrap();
            assert_eq!(list.rows.len(), 5_000);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_parsing_with_different_limits() {
    let doc = Arc::new("%VERSION: 1.0\n---\ndata: 42\n");

    let mut handles = vec![];

    // Thread with default limits
    let doc1 = Arc::clone(&doc);
    handles.push(thread::spawn(move || {
        let result = parse(doc1.as_bytes());
        assert!(result.is_ok());
    }));

    // Thread with custom limits
    let doc2 = Arc::clone(&doc);
    handles.push(thread::spawn(move || {
        let limits = Limits {
            max_nodes: 100,
            ..Limits::default()
        };
        let result = parse_with_limits(doc2.as_bytes(), ParseOptions { limits, strict_refs: true });
        assert!(result.is_ok());
    }));

    // Thread with unlimited
    let doc3 = Arc::clone(&doc);
    handles.push(thread::spawn(move || {
        let result = parse_with_limits(doc3.as_bytes(), ParseOptions {
            limits: Limits::unlimited(),
            strict_refs: true,
        });
        assert!(result.is_ok());
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}

// =============================================================================
// Memory Pressure Tests
// =============================================================================

#[test]
fn test_large_string_values() {
    // Create document with large string values
    let large_str = "x".repeat(10_000);
    let mut doc = String::from("%VERSION: 1.0\n---\n");

    for i in 0..100 {
        doc.push_str(&format!("key{}: \"{}\"\n", i, large_str));
    }

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

#[test]
fn test_block_string_at_limit() {
    let limits = Limits {
        max_block_string_size: 1200,
        ..Limits::default()
    };

    // Block string content: each line + newline contributes to size
    let content = "x".repeat(400);
    let doc = format!(
        "%VERSION: 1.0\n---\ndata: \"\"\"\n{}\n{}\n\"\"\"\n",
        content, content
    );

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });
    assert!(result.is_ok());
}

#[test]
fn test_block_string_exceeds_limit() {
    let limits = Limits {
        max_block_string_size: 100,
        ..Limits::default()
    };

    let content = "x".repeat(200);
    let doc = format!(
        "%VERSION: 1.0\n---\ndata: \"\"\"\n{}\n\"\"\"\n",
        content
    );

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err().kind, HedlErrorKind::Security));
}

#[test]
fn test_many_small_allocations() {
    // Test with many small values to stress allocator
    // Note: First column must be a valid ID (cannot start with digit)
    let mut doc = String::from("%VERSION: 1.0\n%STRUCT: Tiny: [id, b, c, d, e]\n---\ndata: @Tiny\n");

    for i in 0..10_000 {
        doc.push_str(&format!("  | id-{}, {}, {}, {}, {}\n", i, i+1, i+2, i+3, i+4));
    }

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

// =============================================================================
// Limits Configuration Tests
// =============================================================================

#[test]
fn test_node_limit_enforcement() {
    let limits = Limits {
        max_nodes: 100,
        ..Limits::default()
    };

    let mut doc = String::from("%VERSION: 1.0\n%STRUCT: Record: [id]\n---\ndata: @Record\n");

    // Try to create 200 nodes
    for i in 0..200 {
        doc.push_str(&format!("  | node-{}\n", i));
    }

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err().kind, HedlErrorKind::Security));
}

#[test]
fn test_column_limit_enforcement() {
    let limits = Limits {
        max_columns: 10,
        ..Limits::default()
    };

    let mut schema = vec!["id".to_string()];
    for i in 1..20 {
        schema.push(format!("col{}", i));
    }

    let doc = format!(
        "%VERSION: 1.0\n%STRUCT: Wide: [{}]\n---\ndata: @Wide\n",
        schema.join(", ")
    );

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err().kind, HedlErrorKind::Security));
}

#[test]
fn test_unlimited_limits() {
    let limits = Limits::unlimited();

    let mut doc = String::from("%VERSION: 1.0\n%STRUCT: Record: [id, value]\n---\ndata: @Record\n");

    for i in 0..1_000 {
        doc.push_str(&format!("  | record-{}, value-{}\n", i, i));
    }

    let result = parse_with_limits(doc.as_bytes(), ParseOptions { limits, strict_refs: true });
    assert!(result.is_ok());
}

// =============================================================================
// Traversal Stress Tests
// =============================================================================

#[test]
fn test_traverse_large_document() {
    let mut doc = String::from("%VERSION: 1.0\n%STRUCT: Record: [id, value]\n---\ndata: @Record\n");

    for i in 0..10_000 {
        doc.push_str(&format!("  | record-{}, value-{}\n", i, i));
    }

    let parsed = parse(doc.as_bytes()).unwrap();

    let mut stats = StatsCollector::default();
    let result = traverse(&parsed, &mut stats);
    assert!(result.is_ok());

    assert_eq!(stats.list_count, 1);
    assert_eq!(stats.node_count, 10_000);
}

#[test]
fn test_traverse_deep_nesting() {
    let depth = 20;

    let mut doc = String::from("%VERSION: 1.0\n");

    for i in 0..depth {
        doc.push_str(&format!("%STRUCT: Level{}: [id]\n", i));
    }

    for i in 0..depth - 1 {
        doc.push_str(&format!("%NEST: Level{} > Level{}\n", i, i + 1));
    }

    doc.push_str("---\ndata: @Level0\n");

    for level in 0..depth {
        let indent = "  ".repeat(level + 1);
        doc.push_str(&format!("{}| node-{}\n", indent, level));
    }

    let parsed = parse(doc.as_bytes()).unwrap();

    let mut stats = StatsCollector::default();
    let result = traverse(&parsed, &mut stats);
    assert!(result.is_ok());

    assert_eq!(stats.node_count, depth);
}

#[test]
fn test_traverse_wide_tree() {
    // Create a tree with one root and many children
    let mut doc = String::from("%VERSION: 1.0\n%STRUCT: Parent: [id]\n%STRUCT: Child: [id]\n%NEST: Parent > Child\n---\ndata: @Parent\n");

    doc.push_str("  | root\n");

    for i in 0..1_000 {
        doc.push_str(&format!("    | child-{}\n", i));
    }

    let parsed = parse(doc.as_bytes()).unwrap();

    let mut stats = StatsCollector::default();
    let result = traverse(&parsed, &mut stats);
    assert!(result.is_ok());

    assert_eq!(stats.node_count, 1_001); // 1 parent + 1000 children
}

#[test]
fn test_traverse_complex_mixed_structure() {
    let doc = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, title]
%NEST: User > Post
---
config:
  setting1: value1
  setting2: value2
  nested:
    deep1: 1
    deep2: 2
users: @User
  | user1, Alice
    | post1, First Post
    | post2, Second Post
  | user2, Bob
    | post3, Bob's Post
data:
  key1: value1
  key2: value2
"#;

    let parsed = parse(doc.as_bytes()).unwrap();

    let mut stats = StatsCollector::default();
    let result = traverse(&parsed, &mut stats);
    assert!(result.is_ok());

    assert_eq!(stats.object_count, 3); // config, nested, data
    assert_eq!(stats.list_count, 1);   // users
    assert_eq!(stats.node_count, 5);   // 2 users + 3 posts
    assert!(stats.scalar_count > 0);
}

// =============================================================================
// Round-Trip Consistency Tests
// =============================================================================

#[test]
fn test_round_trip_simple() {
    let doc = "%VERSION: 1.0\n---\nname: test\ncount: 42\nactive: true\n";

    let parsed1 = parse(doc.as_bytes()).unwrap();
    let parsed2 = parse(doc.as_bytes()).unwrap();

    assert_eq!(parsed1, parsed2);
}

#[test]
fn test_round_trip_complex() {
    let doc = r#"%VERSION: 1.0
%STRUCT: Record: [id, value, count]
%ALIAS: %active: "true"
---
config:
  setting: value
data: @Record
  | rec1, value1, 10
  | rec2, value2, 20
"#;

    let parsed1 = parse(doc.as_bytes()).unwrap();
    let parsed2 = parse(doc.as_bytes()).unwrap();

    assert_eq!(parsed1.version, parsed2.version);
    assert_eq!(parsed1.aliases, parsed2.aliases);
    assert_eq!(parsed1.structs, parsed2.structs);
    assert_eq!(parsed1.root, parsed2.root);
}

#[test]
fn test_round_trip_with_references() {
    let doc = r#"%VERSION: 1.0
%STRUCT: User: [id, manager]
---
users: @User
  | alice, ~
  | bob, @alice
  | charlie, @bob
"#;

    let parsed1 = parse(doc.as_bytes()).unwrap();
    let parsed2 = parse(doc.as_bytes()).unwrap();

    assert_eq!(parsed1, parsed2);

    let list1 = parsed1.get("users").unwrap().as_list().unwrap();
    let list2 = parsed2.get("users").unwrap().as_list().unwrap();

    assert_eq!(list1.rows.len(), list2.rows.len());

    // Check references are preserved
    let bob1 = &list1.rows[1];
    let bob2 = &list2.rows[1];

    assert!(bob1.fields[1].is_reference());
    assert!(bob2.fields[1].is_reference());
}

// =============================================================================
// Edge Case Stress Tests
// =============================================================================

#[test]
fn test_empty_lists() {
    let doc = "%VERSION: 1.0\n%STRUCT: Empty: [id]\n---\ndata: @Empty\n";

    let parsed = parse(doc.as_bytes()).unwrap();
    let list = parsed.get("data").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 0);
}

#[test]
fn test_single_column_many_rows() {
    let mut doc = String::from("%VERSION: 1.0\n%STRUCT: Single: [id]\n---\ndata: @Single\n");

    for i in 0..5_000 {
        doc.push_str(&format!("  | id-{}\n", i));
    }

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());

    let parsed = result.unwrap();
    let list = parsed.get("data").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 5_000);
    assert_eq!(list.schema.len(), 1);
}

#[test]
fn test_alternating_structure() {
    // Alternate between objects and lists
    let mut doc = String::from("%VERSION: 1.0\n%STRUCT: Item: [id]\n---\n");

    for i in 0..100 {
        doc.push_str(&format!("obj{}:\n  value: {}\n", i, i));
        doc.push_str(&format!("list{}: @Item\n  | item-{}\n", i, i));
    }

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert_eq!(parsed.root.len(), 200); // 100 objects + 100 lists
}

#[test]
fn test_unicode_stress() {
    // Test with various unicode characters
    let mut doc = String::from("%VERSION: 1.0\n%STRUCT: Unicode: [id, name, emoji]\n---\ndata: @Unicode\n");

    let emojis = ["🎉", "🚀", "🌟", "💎", "🔥", "⚡", "🌈", "🎨", "🎯", "🎪"];

    for i in 0..100 {
        doc.push_str(&format!(
            "  | user-{}, 名前-{}, {}\n",
            i, i, emojis[i % emojis.len()]
        ));
    }

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

#[test]
fn test_all_value_types() {
    let doc = r#"%VERSION: 1.0
%STRUCT: User: [id, manager, score, ratio, active, note]
---
users: @User
  | alice, ~, 100, 1.5, true, test note
  | bob, @alice, 200, 2.5, false, another note
"#;

    let parsed = parse(doc.as_bytes()).unwrap();
    let list = parsed.get("users").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 2);

    // Verify different value types
    let alice = &list.rows[0];
    assert!(alice.fields[0].as_str().is_some());  // String
    assert!(alice.fields[1].is_null());           // Null
    assert!(alice.fields[2].as_int().is_some());  // Int
    assert!(alice.fields[3].as_float().is_some()); // Float
    assert!(alice.fields[4].as_bool().is_some()); // Bool

    let bob = &list.rows[1];
    assert!(bob.fields[1].is_reference()); // Reference
}

#[test]
fn test_stress_reference_resolution() {
    // Create many cross-references
    let mut doc = String::from("%VERSION: 1.0\n%STRUCT: Node: [id, ref1, ref2, ref3]\n---\ndata: @Node\n");

    for i in 0..1_000 {
        let ref1 = if i > 0 { format!("@node-{}", i - 1) } else { "~".to_string() };
        let ref2 = if i > 100 { format!("@node-{}", i - 100) } else { "~".to_string() };
        let ref3 = if i > 500 { format!("@node-{}", i - 500) } else { "~".to_string() };

        doc.push_str(&format!("  | node-{}, {}, {}, {}\n", i, ref1, ref2, ref3));
    }

    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

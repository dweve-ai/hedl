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

//! Integration tests for MCP operation caching.

use hedl_mcp::{McpServer, McpServerConfig, OperationCache};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn test_cache_validate_operation() {
    let cache = OperationCache::new(100);

    let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";
    let result = json!({"valid": true, "version": "1.0"});

    // Insert into cache
    cache.insert("validate", hedl, result.clone());

    // Retrieve from cache
    let cached = cache.get("validate", hedl);
    assert!(cached.is_some());
    assert_eq!(cached.unwrap(), result);

    // Cache statistics
    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.size, 1);
}

#[test]
fn test_cache_different_operations_same_input() {
    let cache = OperationCache::new(100);

    let hedl = "%VERSION: 1.0\n---";

    // Cache different operations on same input
    cache.insert("validate", hedl, json!({"valid": true}));
    cache.insert("lint", hedl, json!({"diagnostics": []}));
    cache.insert("stats", hedl, json!({"tokens": 10}));

    // Each operation should be cached separately
    assert!(cache.get("validate", hedl).is_some());
    assert!(cache.get("lint", hedl).is_some());
    assert!(cache.get("stats", hedl).is_some());

    let stats = cache.stats();
    assert_eq!(stats.size, 3);
    assert_eq!(stats.hits, 3);
}

#[test]
fn test_cache_lru_eviction() {
    let cache = OperationCache::new(3);

    // Fill cache to capacity
    cache.insert("op", "input1", json!(1));
    cache.insert("op", "input2", json!(2));
    cache.insert("op", "input3", json!(3));

    assert_eq!(cache.stats().size, 3);

    // Insert one more (should evict input1)
    cache.insert("op", "input4", json!(4));

    assert_eq!(cache.stats().size, 3);
    assert_eq!(cache.stats().evictions, 1);

    // input1 should be evicted
    assert!(cache.get("op", "input1").is_none());

    // Others should still be present
    assert!(cache.get("op", "input2").is_some());
    assert!(cache.get("op", "input3").is_some());
    assert!(cache.get("op", "input4").is_some());
}

#[test]
fn test_cache_hit_rate_calculation() {
    let cache = OperationCache::new(100);

    cache.insert("op", "input1", json!(1));

    // 3 hits, 2 misses
    cache.get("op", "input1"); // Hit
    cache.get("op", "input2"); // Miss
    cache.get("op", "input1"); // Hit
    cache.get("op", "input3"); // Miss
    cache.get("op", "input1"); // Hit

    let stats = cache.stats();
    assert_eq!(stats.hits, 3);
    assert_eq!(stats.misses, 2);
    assert_eq!(stats.hit_rate(), 0.6);
    assert_eq!(stats.hit_rate_percent(), 60.0);
}

#[test]
fn test_cache_clear() {
    let cache = OperationCache::new(100);

    cache.insert("op", "input1", json!(1));
    cache.insert("op", "input2", json!(2));

    assert_eq!(cache.stats().size, 2);

    cache.clear();

    assert_eq!(cache.stats().size, 0);
    assert!(cache.get("op", "input1").is_none());
    assert!(cache.get("op", "input2").is_none());
}

#[test]
fn test_cache_reset_stats() {
    let cache = OperationCache::new(100);

    cache.insert("op", "input1", json!(1));
    cache.get("op", "input1"); // Hit
    cache.get("op", "input2"); // Miss

    assert_eq!(cache.stats().hits, 1);
    assert_eq!(cache.stats().misses, 1);

    cache.reset_stats();

    assert_eq!(cache.stats().hits, 0);
    assert_eq!(cache.stats().misses, 0);
    assert_eq!(cache.stats().size, 1); // Cache not cleared
}

#[test]
fn test_cache_disabled() {
    let temp_dir = TempDir::new().unwrap();
    let config = McpServerConfig {
        root_path: temp_dir.path().to_path_buf(),
        cache_size: 0, // Disable caching
        ..Default::default()
    };

    let server = McpServer::new(config);

    assert!(server.cache().is_none());
    assert!(server.cache_stats().is_none());
}

#[test]
fn test_cache_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let config = McpServerConfig {
        root_path: temp_dir.path().to_path_buf(),
        cache_size: 1000,
        ..Default::default()
    };

    let server = McpServer::new(config);

    assert!(server.cache().is_some());
    let stats = server.cache_stats().unwrap();
    assert_eq!(stats.max_size, 1000);
    assert_eq!(stats.size, 0);
}

#[test]
fn test_cache_correctness_validate() {
    let cache = Arc::new(OperationCache::new(100));

    let valid_hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";
    let invalid_hedl = "invalid hedl content";

    // Execute and cache valid HEDL
    let args = json!({ "hedl": valid_hedl, "strict": true, "lint": true });
    let result = hedl_mcp::tools::execute_hedl_validate(Some(args.clone())).unwrap();
    let result_json = serde_json::to_value(&result).unwrap();

    let cache_key = format!("{}:{}:{}", valid_hedl, true, true);
    cache.insert("validate", &cache_key, result_json.clone());

    // Verify cached result matches fresh execution
    let cached = cache.get("validate", &cache_key).unwrap();
    assert_eq!(cached, result_json);

    // Invalid HEDL should not be in cache
    let invalid_key = format!("{}:{}:{}", invalid_hedl, true, true);
    assert!(cache.get("validate", &invalid_key).is_none());
}

#[test]
fn test_cache_correctness_query() {
    let cache = Arc::new(OperationCache::new(100));

    let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n  | bob, Bob\n";

    // Query all users
    let args_all = json!({ "hedl": hedl });
    let result_all = hedl_mcp::tools::execute_hedl_query(Some(args_all.clone())).unwrap();
    let result_all_json = serde_json::to_value(&result_all).unwrap();

    let cache_key_all = format!("{}:::true", hedl);
    cache.insert("query", &cache_key_all, result_all_json.clone());

    // Query specific user
    let args_alice = json!({ "hedl": hedl, "id": "alice" });
    let result_alice = hedl_mcp::tools::execute_hedl_query(Some(args_alice.clone())).unwrap();
    let result_alice_json = serde_json::to_value(&result_alice).unwrap();

    let cache_key_alice = format!("{}::alice:true", hedl);
    cache.insert("query", &cache_key_alice, result_alice_json.clone());

    // Verify different queries are cached separately
    let cached_all = cache.get("query", &cache_key_all).unwrap();
    let cached_alice = cache.get("query", &cache_key_alice).unwrap();

    assert_eq!(cached_all, result_all_json);
    assert_eq!(cached_alice, result_alice_json);
    assert_ne!(cached_all, cached_alice);
}

#[test]
fn test_cache_correctness_stats() {
    let cache = Arc::new(OperationCache::new(100));

    let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";

    // Stats with simple tokenizer
    let args_simple = json!({ "hedl": hedl, "tokenizer": "simple" });
    let result_simple = hedl_mcp::tools::execute_hedl_stats(Some(args_simple.clone())).unwrap();
    let result_simple_json = serde_json::to_value(&result_simple).unwrap();

    let cache_key_simple = format!("{}:simple", hedl);
    cache.insert("stats", &cache_key_simple, result_simple_json.clone());

    // Verify cached result
    let cached = cache.get("stats", &cache_key_simple).unwrap();
    assert_eq!(cached, result_simple_json);
}

#[test]
fn test_cache_invalidation_on_content_change() {
    let cache = Arc::new(OperationCache::new(100));

    let hedl_v1 = "%VERSION: 1.0\n---";
    let hedl_v2 = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---";

    // Cache v1
    cache.insert("validate", hedl_v1, json!({"valid": true, "version": "1.0"}));

    // v2 should not hit cache (different content)
    assert!(cache.get("validate", hedl_v2).is_none());

    // v1 should still be cached
    assert!(cache.get("validate", hedl_v1).is_some());
}

#[test]
fn test_cache_parameter_sensitivity() {
    let cache = Arc::new(OperationCache::new(100));

    let hedl = "%VERSION: 1.0\n---";

    // Different parameter combinations should be cached separately
    let key_strict_true = format!("{}:{}:{}", hedl, true, true);
    let key_strict_false = format!("{}:{}:{}", hedl, false, true);
    let key_no_lint = format!("{}:{}:{}", hedl, true, false);

    cache.insert("validate", &key_strict_true, json!({"strict": true, "lint": true}));
    cache.insert("validate", &key_strict_false, json!({"strict": false, "lint": true}));
    cache.insert("validate", &key_no_lint, json!({"strict": true, "lint": false}));

    // Each parameter combination should be cached independently
    assert!(cache.get("validate", &key_strict_true).is_some());
    assert!(cache.get("validate", &key_strict_false).is_some());
    assert!(cache.get("validate", &key_no_lint).is_some());

    assert_eq!(cache.stats().size, 3);
}

#[test]
fn test_cache_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let cache = Arc::new(OperationCache::new(1000));
    let mut handles = vec![];

    // Spawn multiple threads performing cache operations
    for i in 0..10 {
        let cache_clone = cache.clone();
        let handle = thread::spawn(move || {
            for j in 0..100 {
                let key = format!("input_{}", j % 10);
                cache_clone.insert("op", &key, json!(i * 100 + j));
                cache_clone.get("op", &key);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Cache should be consistent
    let stats = cache.stats();
    assert!(stats.size > 0);
    assert!(stats.size <= 1000);
    assert!(stats.hits > 0);
}

#[test]
fn test_cache_memory_bounds() {
    let cache = OperationCache::new(10);

    // Insert more than capacity
    for i in 0..100 {
        cache.insert("op", &format!("input{}", i), json!(i));
    }

    // Cache size should not exceed max_size
    let stats = cache.stats();
    assert!(stats.size <= 10);
    assert!(stats.evictions > 0);
}

#[test]
fn test_cache_hash_determinism() {
    let cache = Arc::new(OperationCache::new(100));

    let hedl = "%VERSION: 1.0\n---";

    // Insert same content multiple times
    cache.insert("validate", hedl, json!(1));
    cache.insert("validate", hedl, json!(2)); // Overwrites

    // Should only have one entry
    let stats = cache.stats();
    assert_eq!(stats.size, 1);

    // Last insertion should win
    let result = cache.get("validate", hedl).unwrap();
    assert_eq!(result, json!(2));
}

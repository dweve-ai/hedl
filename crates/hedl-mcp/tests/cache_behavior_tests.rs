// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Cache behavior tests for hedl-mcp.
//!
//! Tests caching strategies, eviction policies, concurrent access,
//! and cache consistency.

use hedl_mcp::cache::OperationCache;
use serde_json::json;
use std::sync::Arc;
use std::thread;

// ============================================================================
// Basic Cache Operations
// ============================================================================

#[test]
fn test_cache_insert_and_get() {
    let cache = OperationCache::new(10);

    cache.insert("tool", "key1", json!({"result": "value1"}));

    let result = cache.get("tool", "key1");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), json!({"result": "value1"}));
}

#[test]
fn test_cache_miss() {
    let cache = OperationCache::new(10);

    let result = cache.get("tool", "nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_cache_different_tools() {
    let cache = OperationCache::new(10);

    cache.insert("tool1", "key", json!({"result": "from_tool1"}));
    cache.insert("tool2", "key", json!({"result": "from_tool2"}));

    assert_eq!(
        cache.get("tool1", "key").unwrap(),
        json!({"result": "from_tool1"})
    );
    assert_eq!(
        cache.get("tool2", "key").unwrap(),
        json!({"result": "from_tool2"})
    );
}

#[test]
fn test_cache_overwrite() {
    let cache = OperationCache::new(10);

    cache.insert("tool", "key", json!({"version": 1}));
    cache.insert("tool", "key", json!({"version": 2}));

    let result = cache.get("tool", "key");
    assert_eq!(result.unwrap(), json!({"version": 2}));
}

#[test]
fn test_cache_clear() {
    let cache = OperationCache::new(10);

    cache.insert("tool", "key1", json!({"data": "value1"}));
    cache.insert("tool", "key2", json!({"data": "value2"}));

    cache.clear();

    assert!(cache.get("tool", "key1").is_none());
    assert!(cache.get("tool", "key2").is_none());
}

// ============================================================================
// Cache Eviction Tests
// ============================================================================

#[test]
fn test_cache_lru_eviction() {
    let cache = OperationCache::new(3);

    cache.insert("tool", "key1", json!(1));
    cache.insert("tool", "key2", json!(2));
    cache.insert("tool", "key3", json!(3));

    // Access key1 to make it recently used
    cache.get("tool", "key1");

    // Insert key4, should evict key2 (least recently used)
    cache.insert("tool", "key4", json!(4));

    assert!(cache.get("tool", "key1").is_some());
    assert!(cache.get("tool", "key2").is_none()); // Evicted
    assert!(cache.get("tool", "key3").is_some());
    assert!(cache.get("tool", "key4").is_some());
}

#[test]
fn test_cache_fill_to_capacity() {
    let cache = OperationCache::new(5);

    for i in 0..5 {
        cache.insert("tool", &format!("key{i}"), json!(i));
    }

    let stats = cache.stats();
    assert_eq!(stats.size, 5);
}

#[test]
fn test_cache_beyond_capacity() {
    let cache = OperationCache::new(3);

    for i in 0..10 {
        cache.insert("tool", &format!("key{i}"), json!(i));
    }

    let stats = cache.stats();
    assert!(stats.size <= 3);
}

// ============================================================================
// Cache Statistics Tests
// ============================================================================

#[test]
fn test_cache_stats_initial() {
    let cache = OperationCache::new(10);

    let stats = cache.stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.size, 0);
    assert_eq!(stats.max_size, 10);
}

#[test]
fn test_cache_stats_hits() {
    let cache = OperationCache::new(10);

    cache.insert("tool", "key", json!("value"));

    cache.get("tool", "key"); // Hit
    cache.get("tool", "key"); // Hit
    cache.get("tool", "key"); // Hit

    let stats = cache.stats();
    assert_eq!(stats.hits, 3);
    assert_eq!(stats.misses, 0);
}

#[test]
fn test_cache_stats_misses() {
    let cache = OperationCache::new(10);

    cache.get("tool", "key1"); // Miss
    cache.get("tool", "key2"); // Miss
    cache.get("tool", "key3"); // Miss

    let stats = cache.stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 3);
}

#[test]
fn test_cache_stats_mixed() {
    let cache = OperationCache::new(10);

    cache.insert("tool", "key", json!("value"));

    cache.get("tool", "key"); // Hit
    cache.get("tool", "missing"); // Miss
    cache.get("tool", "key"); // Hit

    let stats = cache.stats();
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 1);
}

#[test]
fn test_cache_stats_hit_rate() {
    let cache = OperationCache::new(10);

    cache.insert("tool", "key", json!("value"));

    for _ in 0..7 {
        cache.get("tool", "key"); // Hits
    }

    for i in 0..3 {
        cache.get("tool", &format!("missing{i}")); // Misses
    }

    let stats = cache.stats();
    assert_eq!(stats.hits, 7);
    assert_eq!(stats.misses, 3);
    assert_eq!(stats.hit_rate(), 0.7);
}

#[test]
fn test_cache_stats_hit_rate_zero_requests() {
    let cache = OperationCache::new(10);
    let stats = cache.stats();
    assert_eq!(stats.hit_rate(), 0.0);
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[test]
fn test_cache_concurrent_reads() {
    let cache = Arc::new(OperationCache::new(100));

    cache.insert("tool", "shared_key", json!({"value": 42}));

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let cache_clone = Arc::clone(&cache);
            thread::spawn(move || {
                for _ in 0..100 {
                    let result = cache_clone.get("tool", "shared_key");
                    assert!(result.is_some());
                    assert_eq!(result.unwrap(), json!({"value": 42}));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = cache.stats();
    assert_eq!(stats.hits, 1000);
}

#[test]
fn test_cache_concurrent_writes() {
    let cache = Arc::new(OperationCache::new(1000));

    let handles: Vec<_> = (0..10)
        .map(|thread_id| {
            let cache_clone = Arc::clone(&cache);
            thread::spawn(move || {
                for i in 0..100 {
                    let key = format!("key_{thread_id}_{i}");
                    cache_clone.insert("tool", &key, json!({"thread": thread_id, "value": i}));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = cache.stats();
    assert!(stats.size > 0);
}

#[test]
fn test_cache_concurrent_read_write() {
    let cache = Arc::new(OperationCache::new(100));

    // Writers
    let writers: Vec<_> = (0..5)
        .map(|thread_id| {
            let cache_clone = Arc::clone(&cache);
            thread::spawn(move || {
                for i in 0..50 {
                    let key = format!("key_{i}");
                    cache_clone.insert("tool", &key, json!(thread_id));
                    thread::sleep(std::time::Duration::from_micros(10));
                }
            })
        })
        .collect();

    // Readers
    let readers: Vec<_> = (0..5)
        .map(|_| {
            let cache_clone = Arc::clone(&cache);
            thread::spawn(move || {
                for i in 0..50 {
                    let key = format!("key_{i}");
                    cache_clone.get("tool", &key);
                    thread::sleep(std::time::Duration::from_micros(10));
                }
            })
        })
        .collect();

    for handle in writers.into_iter().chain(readers) {
        handle.join().unwrap();
    }

    // No panics means success
}

#[test]
fn test_cache_concurrent_clear() {
    let cache = Arc::new(OperationCache::new(100));

    // Insert initial data
    for i in 0..50 {
        cache.insert("tool", &format!("key{i}"), json!(i));
    }

    let cache1 = Arc::clone(&cache);
    let h1 = thread::spawn(move || {
        cache1.clear();
    });

    let cache2 = Arc::clone(&cache);
    let h2 = thread::spawn(move || {
        for i in 50..100 {
            cache2.insert("tool", &format!("key{i}"), json!(i));
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();

    // Cache should be in a consistent state (no panics)
}

// ============================================================================
// Cache Key Generation Tests
// ============================================================================

#[test]
fn test_cache_key_uniqueness() {
    let cache = OperationCache::new(100);

    cache.insert("tool", "key1", json!(1));
    cache.insert("tool", "key2", json!(2));

    assert_eq!(cache.get("tool", "key1").unwrap(), json!(1));
    assert_eq!(cache.get("tool", "key2").unwrap(), json!(2));
}

#[test]
fn test_cache_key_special_characters() {
    let cache = OperationCache::new(100);

    cache.insert("tool", "key:with:colons", json!(1));
    cache.insert("tool", "key/with/slashes", json!(2));
    cache.insert("tool", "key with spaces", json!(3));

    assert!(cache.get("tool", "key:with:colons").is_some());
    assert!(cache.get("tool", "key/with/slashes").is_some());
    assert!(cache.get("tool", "key with spaces").is_some());
}

#[test]
fn test_cache_key_unicode() {
    let cache = OperationCache::new(100);

    cache.insert("tool", "key_🚀", json!("rocket"));
    cache.insert("tool", "key_世界", json!("world"));

    assert_eq!(cache.get("tool", "key_🚀").unwrap(), json!("rocket"));
    assert_eq!(cache.get("tool", "key_世界").unwrap(), json!("world"));
}

#[test]
fn test_cache_empty_key() {
    let cache = OperationCache::new(100);

    cache.insert("tool", "", json!("empty_key"));

    assert_eq!(cache.get("tool", "").unwrap(), json!("empty_key"));
}

#[test]
fn test_cache_long_key() {
    let cache = OperationCache::new(100);

    let long_key = "a".repeat(10000);
    cache.insert("tool", &long_key, json!("long_key_value"));

    assert_eq!(
        cache.get("tool", &long_key).unwrap(),
        json!("long_key_value")
    );
}

// ============================================================================
// Cache Value Tests
// ============================================================================

#[test]
fn test_cache_null_value() {
    let cache = OperationCache::new(10);

    cache.insert("tool", "null_key", serde_json::Value::Null);

    let result = cache.get("tool", "null_key");
    assert!(result.is_some());
    assert!(result.unwrap().is_null());
}

#[test]
fn test_cache_complex_value() {
    let cache = OperationCache::new(10);

    let complex_value = json!({
        "nested": {
            "array": [1, 2, 3],
            "object": {"key": "value"},
            "string": "text",
            "number": 42,
            "bool": true,
            "null": null
        }
    });

    cache.insert("tool", "complex", complex_value.clone());

    assert_eq!(cache.get("tool", "complex").unwrap(), complex_value);
}

#[test]
fn test_cache_large_value() {
    let cache = OperationCache::new(10);

    let large_array: Vec<i32> = (0..10000).collect();
    let large_value = json!(large_array);

    cache.insert("tool", "large", large_value.clone());

    assert_eq!(cache.get("tool", "large").unwrap(), large_value);
}

// ============================================================================
// Cache Capacity Tests
// ============================================================================

#[test]
fn test_cache_zero_capacity() {
    let cache = OperationCache::new(0);

    cache.insert("tool", "key", json!("value"));

    // With zero capacity, nothing should be cached
    assert!(cache.get("tool", "key").is_none());
}

#[test]
fn test_cache_one_capacity() {
    let cache = OperationCache::new(1);

    cache.insert("tool", "key1", json!(1));
    assert!(cache.get("tool", "key1").is_some());

    cache.insert("tool", "key2", json!(2));
    assert!(cache.get("tool", "key2").is_some());
    assert!(cache.get("tool", "key1").is_none()); // Evicted
}

#[test]
fn test_cache_large_capacity() {
    let cache = OperationCache::new(10000);

    for i in 0..5000 {
        cache.insert("tool", &format!("key{i}"), json!(i));
    }

    let stats = cache.stats();
    assert_eq!(stats.size, 5000);
    assert_eq!(stats.max_size, 10000);
}

// ============================================================================
// Cache Consistency Tests
// ============================================================================

#[test]
fn test_cache_get_does_not_modify() {
    let cache = OperationCache::new(10);

    cache.insert("tool", "key", json!({"mutable": false}));

    let result1 = cache.get("tool", "key").unwrap();
    let result2 = cache.get("tool", "key").unwrap();

    assert_eq!(result1, result2);
}

#[test]
fn test_cache_multiple_tools_isolation() {
    let cache = OperationCache::new(100);

    cache.insert("tool1", "shared_key", json!("from_tool1"));
    cache.insert("tool2", "shared_key", json!("from_tool2"));
    cache.insert("tool3", "shared_key", json!("from_tool3"));

    assert_eq!(
        cache.get("tool1", "shared_key").unwrap(),
        json!("from_tool1")
    );
    assert_eq!(
        cache.get("tool2", "shared_key").unwrap(),
        json!("from_tool2")
    );
    assert_eq!(
        cache.get("tool3", "shared_key").unwrap(),
        json!("from_tool3")
    );
}

#[test]
fn test_cache_stats_after_clear() {
    let cache = OperationCache::new(10);

    cache.insert("tool", "key", json!("value"));
    cache.get("tool", "key");

    cache.clear();

    let stats = cache.stats();
    assert_eq!(stats.size, 0);
    // Note: hits and misses are NOT reset by clear()
    assert_eq!(stats.hits, 1);
}

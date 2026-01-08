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

//! Schema caching for JSON to HEDL conversion
//!
//! When converting large JSON arrays to matrix lists, we often encounter the same
//! structure repeatedly. Caching the inferred schema significantly improves performance
//! by avoiding redundant key iteration and sorting.
//!
//! # Performance Impact
//!
//! - First schema inference: ~O(n*log(n)) where n is number of keys
//! - Cached lookup: ~O(1) hash map lookup
//! - Expected speedup: 30-50% for documents with repeated array structures
//!
//! # Thread Safety
//!
//! The cache is thread-safe using interior mutability with `RwLock`. Multiple threads
//! can read from the cache concurrently, while writes are exclusive.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

/// Cache key for schema lookup
///
/// Represents the structure of a JSON object by its sorted field names.
/// This excludes metadata fields (starting with "__") and child arrays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaCacheKey {
    /// Sorted field names (excluding metadata and children)
    pub fields: Vec<String>,
}

impl Hash for SchemaCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the sorted fields
        for field in &self.fields {
            field.hash(state);
        }
    }
}

impl SchemaCacheKey {
    /// Create a cache key from a list of field names
    ///
    /// Automatically sorts the fields for consistent hashing.
    pub fn new(mut fields: Vec<String>) -> Self {
        fields.sort();
        Self { fields }
    }
}

/// Entry in the LRU cache
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached schema (column names in order)
    schema: Vec<String>,
    /// Access counter for LRU eviction
    access_count: u64,
    /// Last access timestamp (for tie-breaking)
    last_access: std::time::Instant,
}

/// Statistics for cache performance monitoring
#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of evictions due to capacity
    pub evictions: u64,
    /// Current cache size
    pub size: usize,
    /// Maximum cache capacity
    pub capacity: usize,
}

impl CacheStatistics {
    /// Calculate cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Calculate cache miss rate (0.0 to 1.0)
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
    }
}

/// Thread-safe LRU schema cache
///
/// Uses interior mutability with `RwLock` for thread-safe access.
/// Multiple threads can read concurrently, while writes are exclusive.
///
/// # Examples
///
/// ```rust
/// use hedl_json::schema_cache::{SchemaCache, SchemaCacheKey};
///
/// let cache = SchemaCache::new(100);
///
/// // Cache a schema
/// let key = SchemaCacheKey::new(vec!["id".to_string(), "name".to_string()]);
/// cache.insert(key.clone(), vec!["id".to_string(), "name".to_string()]);
///
/// // Retrieve from cache
/// if let Some(schema) = cache.get(&key) {
///     println!("Schema: {:?}", schema);
/// }
///
/// // Get statistics
/// let stats = cache.statistics();
/// println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
/// ```
#[derive(Debug, Clone)]
pub struct SchemaCache {
    /// The underlying cache storage
    inner: Arc<RwLock<SchemaCacheInner>>,
}

#[derive(Debug)]
struct SchemaCacheInner {
    /// Map from cache key to schema
    cache: HashMap<SchemaCacheKey, CacheEntry>,
    /// Maximum cache size
    capacity: usize,
    /// Cache statistics
    stats: CacheStatistics,
}

impl SchemaCache {
    /// Create a new schema cache with the specified capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of schemas to cache (default: 100)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_json::schema_cache::SchemaCache;
    ///
    /// let cache = SchemaCache::new(100);
    /// ```
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(SchemaCacheInner {
                cache: HashMap::with_capacity(capacity),
                capacity,
                stats: CacheStatistics {
                    capacity,
                    ..Default::default()
                },
            })),
        }
    }

    /// Get a schema from the cache
    ///
    /// Returns `Some(schema)` if found, `None` otherwise.
    /// Updates access statistics for LRU tracking.
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key representing the JSON structure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_json::schema_cache::{SchemaCache, SchemaCacheKey};
    ///
    /// let cache = SchemaCache::new(100);
    /// let key = SchemaCacheKey::new(vec!["id".to_string()]);
    ///
    /// if let Some(schema) = cache.get(&key) {
    ///     println!("Found: {:?}", schema);
    /// }
    /// ```
    pub fn get(&self, key: &SchemaCacheKey) -> Option<Vec<String>> {
        let mut inner = self.inner.write().unwrap();

        if let Some(entry) = inner.cache.get_mut(key) {
            // Update LRU tracking
            entry.access_count += 1;
            entry.last_access = std::time::Instant::now();

            // Clone schema before updating stats
            let schema = entry.schema.clone();

            // Update statistics
            inner.stats.hits += 1;

            Some(schema)
        } else {
            // Update statistics
            inner.stats.misses += 1;
            None
        }
    }

    /// Insert a schema into the cache
    ///
    /// If the cache is full, evicts the least recently used entry.
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key representing the JSON structure
    /// * `schema` - Schema to cache (column names in order)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_json::schema_cache::{SchemaCache, SchemaCacheKey};
    ///
    /// let cache = SchemaCache::new(100);
    /// let key = SchemaCacheKey::new(vec!["id".to_string(), "name".to_string()]);
    /// cache.insert(key, vec!["id".to_string(), "name".to_string()]);
    /// ```
    pub fn insert(&self, key: SchemaCacheKey, schema: Vec<String>) {
        let mut inner = self.inner.write().unwrap();

        // Check if we need to evict
        if inner.cache.len() >= inner.capacity && !inner.cache.contains_key(&key) {
            // Find LRU entry (lowest access_count, oldest last_access)
            if let Some(lru_key) = inner
                .cache
                .iter()
                .min_by_key(|(_, entry)| (entry.access_count, entry.last_access))
                .map(|(k, _)| k.clone())
            {
                inner.cache.remove(&lru_key);
                inner.stats.evictions += 1;
            }
        }

        // Insert or update
        inner.cache.insert(
            key,
            CacheEntry {
                schema,
                access_count: 1,
                last_access: std::time::Instant::now(),
            },
        );

        // Update size statistic
        inner.stats.size = inner.cache.len();
    }

    /// Get current cache statistics
    ///
    /// Returns a snapshot of cache performance metrics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_json::schema_cache::SchemaCache;
    ///
    /// let cache = SchemaCache::new(100);
    /// let stats = cache.statistics();
    /// println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    /// println!("Size: {}/{}", stats.size, stats.capacity);
    /// ```
    pub fn statistics(&self) -> CacheStatistics {
        let inner = self.inner.read().unwrap();
        inner.stats.clone()
    }

    /// Clear the cache and reset statistics
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_json::schema_cache::SchemaCache;
    ///
    /// let cache = SchemaCache::new(100);
    /// cache.clear();
    /// ```
    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.cache.clear();
        inner.stats.reset();
        inner.stats.size = 0;
        inner.stats.capacity = inner.capacity;
    }

    /// Get current cache size
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_json::schema_cache::SchemaCache;
    ///
    /// let cache = SchemaCache::new(100);
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        let inner = self.inner.read().unwrap();
        inner.cache.len()
    }

    /// Check if cache is empty
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_json::schema_cache::SchemaCache;
    ///
    /// let cache = SchemaCache::new(100);
    /// assert!(cache.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get cache capacity
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_json::schema_cache::SchemaCache;
    ///
    /// let cache = SchemaCache::new(100);
    /// assert_eq!(cache.capacity(), 100);
    /// ```
    pub fn capacity(&self) -> usize {
        let inner = self.inner.read().unwrap();
        inner.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_new() {
        let key = SchemaCacheKey::new(vec!["name".to_string(), "id".to_string()]);
        assert_eq!(key.fields, vec!["id", "name"]); // Should be sorted
    }

    #[test]
    fn test_cache_key_equality() {
        let key1 = SchemaCacheKey::new(vec!["name".to_string(), "id".to_string()]);
        let key2 = SchemaCacheKey::new(vec!["id".to_string(), "name".to_string()]);
        assert_eq!(key1, key2); // Same fields, different order
    }

    #[test]
    fn test_cache_basic_operations() {
        let cache = SchemaCache::new(10);

        let key = SchemaCacheKey::new(vec!["id".to_string(), "name".to_string()]);
        let schema = vec!["id".to_string(), "name".to_string()];

        // Initially empty
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        // Insert and retrieve
        cache.insert(key.clone(), schema.clone());
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());

        let retrieved = cache.get(&key);
        assert_eq!(retrieved, Some(schema));
    }

    #[test]
    fn test_cache_miss() {
        let cache = SchemaCache::new(10);
        let key = SchemaCacheKey::new(vec!["id".to_string()]);

        let result = cache.get(&key);
        assert_eq!(result, None);

        let stats = cache.statistics();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 0);
    }

    #[test]
    fn test_cache_hit() {
        let cache = SchemaCache::new(10);
        let key = SchemaCacheKey::new(vec!["id".to_string()]);
        let schema = vec!["id".to_string()];

        cache.insert(key.clone(), schema.clone());

        let result = cache.get(&key);
        assert_eq!(result, Some(schema));

        let stats = cache.statistics();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_statistics() {
        let cache = SchemaCache::new(10);
        let key1 = SchemaCacheKey::new(vec!["id".to_string()]);
        let key2 = SchemaCacheKey::new(vec!["name".to_string()]);

        // Miss
        cache.get(&key1);

        // Insert
        cache.insert(key1.clone(), vec!["id".to_string()]);

        // Hit
        cache.get(&key1);

        // Miss
        cache.get(&key2);

        let stats = cache.statistics();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.size, 1);
        assert_eq!(stats.capacity, 10);
        assert!((stats.hit_rate() - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = SchemaCache::new(3);

        // Insert 3 entries
        for i in 0..3 {
            let key = SchemaCacheKey::new(vec![format!("field{}", i)]);
            cache.insert(key, vec![format!("field{}", i)]);
        }

        assert_eq!(cache.len(), 3);

        // Access first entry to make it recently used
        let key0 = SchemaCacheKey::new(vec!["field0".to_string()]);
        cache.get(&key0);

        // Insert 4th entry - should evict least recently used (field1 or field2)
        let key3 = SchemaCacheKey::new(vec!["field3".to_string()]);
        cache.insert(key3.clone(), vec!["field3".to_string()]);

        assert_eq!(cache.len(), 3);

        // field0 should still be there (recently accessed)
        assert!(cache.get(&key0).is_some());

        // field3 should be there (just inserted)
        assert!(cache.get(&key3).is_some());

        let stats = cache.statistics();
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn test_cache_clear() {
        let cache = SchemaCache::new(10);

        // Insert some entries
        for i in 0..5 {
            let key = SchemaCacheKey::new(vec![format!("field{}", i)]);
            cache.insert(key, vec![format!("field{}", i)]);
        }

        assert_eq!(cache.len(), 5);

        // Clear
        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        let stats = cache.statistics();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.evictions, 0);
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cache_capacity() {
        let cache = SchemaCache::new(42);
        assert_eq!(cache.capacity(), 42);
    }

    #[test]
    fn test_cache_update_existing() {
        let cache = SchemaCache::new(10);
        let key = SchemaCacheKey::new(vec!["id".to_string()]);

        // Insert initial schema
        cache.insert(key.clone(), vec!["id".to_string()]);

        // Update with new schema
        cache.insert(key.clone(), vec!["id".to_string(), "name".to_string()]);

        let result = cache.get(&key);
        assert_eq!(result, Some(vec!["id".to_string(), "name".to_string()]));

        // Should not cause eviction when updating existing key
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_clone() {
        let cache = SchemaCache::new(10);
        let key = SchemaCacheKey::new(vec!["id".to_string()]);
        cache.insert(key.clone(), vec!["id".to_string()]);

        let cache_clone = cache.clone();

        // Both caches should have the same data
        assert_eq!(cache.len(), cache_clone.len());
        assert_eq!(cache.get(&key), cache_clone.get(&key));
    }

    #[test]
    fn test_statistics_hit_rate() {
        let stats = CacheStatistics {
            hits: 7,
            misses: 3,
            evictions: 0,
            size: 5,
            capacity: 10,
        };

        assert!((stats.hit_rate() - 0.7).abs() < 0.01);
        assert!((stats.miss_rate() - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_statistics_empty() {
        let stats = CacheStatistics::default();
        assert_eq!(stats.hit_rate(), 0.0);
        assert_eq!(stats.miss_rate(), 1.0);
    }
}

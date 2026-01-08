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

//! Fixture caching for performance.
//!
//! Provides in-memory caching of loaded fixtures to avoid repeated
//! file I/O during benchmark execution.

use crate::Result;
use std::collections::HashMap;

/// In-memory cache for fixture data.
///
/// Stores loaded fixtures to avoid repeated file I/O during benchmarks.
pub struct FixtureCache {
    cache: HashMap<String, String>,
}

impl FixtureCache {
    /// Creates a new empty fixture cache.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Creates a cache preloaded with common fixtures.
    pub fn preloaded() -> Self {
        let mut cache = Self::new();
        for name in &["small", "medium", "large"] {
            if let Ok(content) = super::loader::load_fixture(name) {
                cache.cache.insert(name.to_string(), content);
            }
        }
        cache
    }

    /// Gets a fixture from cache or loads it.
    ///
    /// # Arguments
    ///
    /// * `name` - Fixture name
    ///
    /// # Returns
    ///
    /// Reference to cached fixture content.
    pub fn get_or_load(&mut self, name: &str) -> Result<&String> {
        if !self.cache.contains_key(name) {
            let content = super::loader::load_fixture(name)?;
            self.cache.insert(name.to_string(), content);
        }
        Ok(self.cache.get(name).unwrap())
    }

    /// Gets a fixture from cache if present.
    ///
    /// # Arguments
    ///
    /// * `name` - Fixture name
    ///
    /// # Returns
    ///
    /// Optional reference to cached content.
    pub fn get(&self, name: &str) -> Option<&String> {
        self.cache.get(name)
    }

    /// Inserts a fixture into the cache.
    ///
    /// # Arguments
    ///
    /// * `name` - Fixture name
    /// * `content` - Fixture content
    pub fn insert(&mut self, name: String, content: String) {
        self.cache.insert(name, content);
    }

    /// Clears the cache.
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Returns the number of cached fixtures.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns true if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Returns total size of cached data in bytes.
    pub fn total_size(&self) -> usize {
        self.cache.values().map(|s| s.len()).sum()
    }

    /// Preloads multiple fixtures into cache.
    ///
    /// # Arguments
    ///
    /// * `names` - Slice of fixture names to preload
    pub fn preload(&mut self, names: &[&str]) -> Result<()> {
        for &name in names {
            let _ = self.get_or_load(name)?;
        }
        Ok(())
    }
}

impl Default for FixtureCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache() {
        let cache = FixtureCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_get_or_load() {
        let mut cache = FixtureCache::new();

        // First load
        {
            let content = cache.get_or_load("small").unwrap();
            assert!(content.contains("%VERSION: 1.0"));
        }

        // Second call should use cache
        {
            let content2 = cache.get_or_load("small").unwrap();
            assert!(content2.contains("%VERSION: 1.0"));
        }

        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_insert() {
        let mut cache = FixtureCache::new();
        cache.insert("test".to_string(), "test content".to_string());
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get("test").unwrap(), "test content");
    }

    #[test]
    fn test_clear() {
        let mut cache = FixtureCache::new();
        cache.insert("test".to_string(), "content".to_string());
        assert_eq!(cache.len(), 1);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_total_size() {
        let mut cache = FixtureCache::new();
        cache.insert("a".to_string(), "hello".to_string());
        cache.insert("b".to_string(), "world".to_string());
        assert_eq!(cache.total_size(), 10);
    }

    #[test]
    fn test_preloaded() {
        let cache = FixtureCache::preloaded();
        assert!(!cache.is_empty());
        assert!(cache.get("small").is_some());
    }

    #[test]
    fn test_preload() {
        let mut cache = FixtureCache::new();
        cache.preload(&["small", "medium"]).unwrap();
        assert_eq!(cache.len(), 2);
    }
}

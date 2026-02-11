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

//! Memory-aware cache with usage tracking.

use super::error::ResourceLimitError;
use dashmap::DashMap;
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::debug;

/// Memory-aware cache that tracks actual memory usage.
///
/// Unlike the basic cache which only tracks entry count, this estimates
/// and enforces memory limits to prevent unbounded growth.
#[derive(Debug)]
pub struct MemoryAwareCache {
    /// Entry size tracking (key -> size in bytes).
    entry_sizes: DashMap<String, usize>,

    /// Total memory usage in bytes.
    total_size: AtomicUsize,

    /// Maximum memory budget in bytes.
    max_size: usize,
}

impl MemoryAwareCache {
    /// Create a new memory-aware cache.
    ///
    /// # Arguments
    ///
    /// * `max_size` - Maximum memory budget in bytes
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            entry_sizes: DashMap::new(),
            total_size: AtomicUsize::new(0),
            max_size,
        }
    }

    /// Insert a value with memory tracking.
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `value` - JSON value to cache
    ///
    /// # Returns
    ///
    /// `Ok(())` if inserted, `Err` if would exceed memory limit.
    pub fn insert(&self, key: String, value: Value) -> Result<(), ResourceLimitError> {
        let value_size = estimate_json_size(&value);

        // Check memory limit
        let current = self.total_size.load(Ordering::Relaxed);
        let new_total = current.saturating_add(value_size);

        if new_total > self.max_size {
            return Err(ResourceLimitError::CacheMemoryExceeded {
                current,
                limit: self.max_size,
                needed: value_size,
            });
        }

        // Track size
        self.entry_sizes.insert(key.clone(), value_size);
        self.total_size.fetch_add(value_size, Ordering::Relaxed);

        debug!(
            "Cache insert: key={}, size={}, total={}",
            key, value_size, new_total
        );

        Ok(())
    }

    /// Remove an entry and update memory tracking.
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key to remove
    pub fn remove(&self, key: &str) {
        if let Some((_, size)) = self.entry_sizes.remove(key) {
            self.total_size.fetch_sub(size, Ordering::Relaxed);
            debug!("Cache remove: key={}, size={}", key, size);
        }
    }

    /// Get current memory usage in bytes.
    pub fn current_usage(&self) -> usize {
        self.total_size.load(Ordering::Relaxed)
    }

    /// Get maximum memory budget in bytes.
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Get the number of cached entries.
    pub fn entry_count(&self) -> usize {
        self.entry_sizes.len()
    }

    /// Clear all entries and reset memory tracking.
    pub fn clear(&self) {
        self.entry_sizes.clear();
        self.total_size.store(0, Ordering::Relaxed);
    }
}

/// Estimate the memory size of a JSON value.
///
/// Provides a rough estimate of memory usage for cache entries.
/// This is an approximation, not an exact measurement.
pub(crate) fn estimate_json_size(value: &Value) -> usize {
    match value {
        Value::Null => 8,
        Value::Bool(_) => 1,
        Value::Number(_) => 8,
        Value::String(s) => s.len() + 24, // String overhead
        Value::Array(arr) => {
            24 + arr.iter().map(estimate_json_size).sum::<usize>() // Array overhead
        }
        Value::Object(obj) => {
            24 + obj
                .iter()
                .map(|(k, v)| k.len() + estimate_json_size(v))
                .sum::<usize>() // Object overhead
        }
    }
}

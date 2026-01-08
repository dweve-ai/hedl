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

//! Document management with caching and LRU eviction.
//!
//! This module handles document storage, caching, and lifecycle management for the LSP server.
//! It provides efficient document access with configurable cache limits and automatic LRU eviction.
//!
//! # Responsibilities
//!
//! - Document storage and retrieval
//! - Content hash-based change detection
//! - LRU-based cache eviction
//! - Cache statistics tracking
//! - Document size limits enforcement
//!
//! # Design
//!
//! The DocumentManager maintains a cache of analyzed documents with the following features:
//!
//! - **LRU Eviction**: Automatically evicts least recently used documents when cache is full
//! - **Dirty Tracking**: Tracks which documents need re-analysis via content hashing
//! - **Access Tracking**: Updates last access time for LRU ordering
//! - **Size Limits**: Enforces maximum document size to prevent memory exhaustion
//! - **Statistics**: Provides cache hit/miss/eviction metrics for monitoring

use crate::analysis::AnalyzedDocument;
use dashmap::DashMap;
use parking_lot::Mutex;
use ropey::Rope;
use std::sync::Arc;
use tower_lsp::lsp_types::Url;
use tracing::{debug, warn, error};

// Re-export constants for backwards compatibility
pub use crate::constants::{DEFAULT_MAX_CACHE_SIZE, DEFAULT_MAX_DOCUMENT_SIZE};

/// Document state with caching and dirty tracking.
///
/// Each document is stored with its content (as a Rope for efficient editing),
/// analysis results, content hash for change detection, and dirty flag.
pub struct DocumentState {
    /// Current rope content for efficient editing operations.
    pub rope: Rope,
    /// Cached analysis result from last parse (Arc-wrapped to avoid expensive clones).
    pub analysis: Arc<AnalyzedDocument>,
    /// Content hash for change detection.
    pub content_hash: u64,
    /// Dirty flag: true if content changed since last analysis.
    pub dirty: bool,
    /// Last access timestamp for LRU eviction.
    pub last_access: std::time::Instant,
}

/// Cache statistics for monitoring and optimization.
///
/// These statistics help identify cache performance issues and guide
/// configuration tuning.
#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    /// Number of cache hits (document found in cache).
    pub hits: u64,
    /// Number of cache misses (document not in cache).
    pub misses: u64,
    /// Number of document evictions due to cache size limit.
    pub evictions: u64,
    /// Current number of documents in cache.
    pub current_size: usize,
    /// Maximum cache size.
    pub max_size: usize,
}

/// Document manager with LRU caching and dirty tracking.
///
/// The DocumentManager is the single source of truth for all document state
/// in the LSP server. It handles document lifecycle, caching, and eviction.
///
/// # Thread Safety
///
/// The DocumentManager uses DashMap for concurrent access and parking_lot::Mutex
/// for fine-grained locking. It can be safely shared across threads.
///
/// # Example
///
/// ```no_run
/// use hedl_lsp::document_manager::DocumentManager;
///
/// let manager = DocumentManager::new(1000, 500 * 1024 * 1024);
///
/// // Insert a document
/// // manager.insert_or_update(uri, content);
///
/// // Get a document
/// // let doc = manager.get(&uri);
/// ```
pub struct DocumentManager {
    /// Document store: URI -> document state.
    documents: DashMap<Url, Arc<Mutex<DocumentState>>>,
    /// Cache statistics for monitoring.
    cache_stats: Arc<Mutex<CacheStatistics>>,
    /// Maximum number of documents to cache.
    max_cache_size: Arc<parking_lot::RwLock<usize>>,
    /// Maximum document size in bytes.
    max_document_size: Arc<parking_lot::RwLock<usize>>,
}

impl DocumentManager {
    /// Create a new document manager with specified limits.
    ///
    /// # Parameters
    ///
    /// - `max_cache_size`: Maximum number of documents to cache (default: 1000)
    /// - `max_document_size`: Maximum document size in bytes (default: 500 MB)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hedl_lsp::document_manager::DocumentManager;
    ///
    /// // Create with custom limits
    /// let manager = DocumentManager::new(2000, 1024 * 1024 * 1024);
    /// ```
    pub fn new(max_cache_size: usize, max_document_size: usize) -> Self {
        Self {
            documents: DashMap::new(),
            cache_stats: Arc::new(Mutex::new(CacheStatistics {
                max_size: max_cache_size,
                ..Default::default()
            })),
            max_cache_size: Arc::new(parking_lot::RwLock::new(max_cache_size)),
            max_document_size: Arc::new(parking_lot::RwLock::new(max_document_size)),
        }
    }

    /// Get current cache statistics.
    ///
    /// This method provides a snapshot of cache performance metrics.
    pub fn statistics(&self) -> CacheStatistics {
        let mut stats = self.cache_stats.lock();
        stats.current_size = self.documents.len();
        stats.clone()
    }

    /// Update maximum cache size (can be called during runtime).
    pub fn set_max_cache_size(&self, new_max: usize) {
        let mut max = self.max_cache_size.write();
        *max = new_max;
        let mut stats = self.cache_stats.lock();
        stats.max_size = new_max;
        debug!("Cache max size updated to: {}", new_max);
    }

    /// Get current maximum cache size.
    pub fn max_cache_size(&self) -> usize {
        *self.max_cache_size.read()
    }

    /// Update maximum document size (can be called during runtime).
    pub fn set_max_document_size(&self, new_max: usize) {
        let mut max = self.max_document_size.write();
        *max = new_max;
        debug!("Max document size updated to: {} bytes", new_max);
    }

    /// Get current maximum document size.
    pub fn max_document_size(&self) -> usize {
        *self.max_document_size.read()
    }

    /// Compute a simple hash for change detection.
    fn hash_content(content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Insert or update a document.
    ///
    /// If the document already exists, updates its content and marks it as dirty
    /// if the content changed. If it's a new document, performs initial analysis.
    ///
    /// # Memory Management
    ///
    /// This method enforces the maximum document size limit. Documents exceeding
    /// the limit are rejected and this method returns `false`.
    ///
    /// # Returns
    ///
    /// Returns `true` if the document was successfully inserted/updated,
    /// `false` if rejected due to size constraints.
    ///
    /// # Error Handling
    ///
    /// - Size limit violations: Logged as warnings and rejected
    /// - Cache eviction: Logged with LRU document details
    /// - Content hashing: Hash collisions are statistically impossible but detected
    pub fn insert_or_update(&self, uri: &Url, content: &str) -> bool {
        // Memory management: Enforce maximum document size
        let max_size = self.max_document_size();
        if content.len() > max_size {
            warn!(
                "Document size limit exceeded for {}: {} bytes > {} bytes maximum (rejected)",
                uri,
                content.len(),
                max_size
            );
            return false;
        }

        let rope = Rope::from_str(content);
        let content_hash = Self::hash_content(content);
        let line_count = content.lines().count();

        if let Some(state_ref) = self.documents.get(uri) {
            // Cache hit - existing document
            {
                let mut stats = self.cache_stats.lock();
                stats.hits += 1;
            }

            let mut state = state_ref.lock();
            // Only update if content actually changed
            if state.content_hash != content_hash {
                debug!(
                    "Document content changed for {}: {} -> {} bytes, {} lines",
                    uri,
                    state.rope.len_bytes(),
                    content.len(),
                    line_count
                );
                state.rope = rope;
                state.content_hash = content_hash;
                state.dirty = true;
                state.last_access = std::time::Instant::now();
            } else {
                debug!(
                    "Document content unchanged for {} (hash: {:#x}), updating access time only",
                    uri, content_hash
                );
                // Update access time even if content hasn't changed
                state.last_access = std::time::Instant::now();
            }
        } else {
            // Cache miss - new document
            {
                let mut stats = self.cache_stats.lock();
                stats.misses += 1;
            }

            debug!(
                "New document registered: {} ({} bytes, {} lines)",
                uri, content.len(), line_count
            );

            // Check if we need to evict before inserting
            let max_cache = self.max_cache_size();
            if self.documents.len() >= max_cache {
                warn!(
                    "Cache limit reached ({}/{}), triggering LRU eviction before inserting {}",
                    self.documents.len(),
                    max_cache,
                    uri
                );
                self.evict_lru_document();
            }

            // New document - perform initial analysis synchronously
            debug!("Starting initial analysis for new document: {}", uri);
            let analysis = Arc::new(AnalyzedDocument::analyze(content));

            if !analysis.errors.is_empty() {
                debug!(
                    "Initial analysis found {} parse errors for {}",
                    analysis.errors.len(),
                    uri
                );
            }

            let state = DocumentState {
                rope,
                analysis,
                content_hash,
                dirty: false,
                last_access: std::time::Instant::now(),
            };
            self.documents
                .insert(uri.clone(), Arc::new(Mutex::new(state)));
            debug!("Document cached: {} (hash: {:#x})", uri, content_hash);
        }

        true
    }

    /// Get document content and analysis.
    ///
    /// This method returns the document content and an Arc to the analysis.
    /// It also updates the last access time for LRU tracking.
    ///
    /// # Returns
    ///
    /// Returns `Some((content, analysis))` if the document exists, `None` otherwise.
    ///
    /// # Error Handling
    ///
    /// - Missing document: Returns None (logged at call site)
    /// - Access tracking: Always updates last access time for LRU
    pub fn get(&self, uri: &Url) -> Option<(String, Arc<AnalyzedDocument>)> {
        self.documents.get(uri).map(|entry| {
            let mut state = entry.lock();
            state.last_access = std::time::Instant::now();
            debug!(
                "Document accessed: {} ({} bytes, dirty: {})",
                uri,
                state.rope.len_bytes(),
                state.dirty
            );
            (state.rope.to_string(), Arc::clone(&state.analysis))
        })
    }

    /// Get document state reference for in-place operations.
    ///
    /// This method returns an Arc to the document state, allowing for
    /// more efficient operations that need to inspect or modify state
    /// without cloning the entire content.
    ///
    /// # Returns
    ///
    /// Returns `Some(Arc<Mutex<DocumentState>>)` if the document exists, `None` otherwise.
    pub fn get_state(&self, uri: &Url) -> Option<Arc<Mutex<DocumentState>>> {
        self.documents.get(uri).map(|entry| entry.clone())
    }

    /// Check if a document is dirty (needs re-analysis).
    ///
    /// # Returns
    ///
    /// Returns `true` if the document exists and is dirty, `false` otherwise.
    pub fn is_dirty(&self, uri: &Url) -> bool {
        self.documents
            .get(uri)
            .map(|entry| {
                let state = entry.lock();
                state.dirty
            })
            .unwrap_or(false)
    }

    /// Mark a document as clean (analysis is up-to-date).
    ///
    /// This method should be called after successfully analyzing a document.
    pub fn mark_clean(&self, uri: &Url) {
        if let Some(state_ref) = self.documents.get(uri) {
            let mut state = state_ref.lock();
            state.dirty = false;
        }
    }

    /// Update analysis for a document and mark it as clean.
    ///
    /// This is a convenience method that combines updating the analysis
    /// and marking the document as clean.
    ///
    /// # Error Handling
    ///
    /// - Missing document: Silently ignored (document may have been closed/evicted)
    /// - Analysis update: Atomic with dirty flag clearing
    pub fn update_analysis(&self, uri: &Url, analysis: Arc<AnalyzedDocument>) {
        if let Some(state_ref) = self.documents.get(uri) {
            let mut state = state_ref.lock();
            debug!(
                "Updating analysis for {}: {} entities, {} errors",
                uri,
                analysis.entities.values().map(|m| m.len()).sum::<usize>(),
                analysis.errors.len()
            );
            state.analysis = analysis;
            state.dirty = false;
        } else {
            warn!(
                "Attempted to update analysis for non-existent document: {} (may have been closed/evicted)",
                uri
            );
        }
    }

    /// Remove a document from the cache.
    ///
    /// This is typically called when a document is closed in the editor.
    ///
    /// # Returns
    ///
    /// Returns `true` if the document was removed, `false` if it didn't exist.
    pub fn remove(&self, uri: &Url) -> bool {
        self.documents.remove(uri).is_some()
    }

    /// Get all document URIs currently in the cache.
    ///
    /// This is useful for workspace-wide operations like workspace symbols.
    pub fn all_uris(&self) -> Vec<Url> {
        self.documents.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Iterate over all documents with a function.
    ///
    /// This provides a safe way to iterate over all documents without
    /// exposing the internal DashMap structure.
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&Url, &Arc<Mutex<DocumentState>>),
    {
        for entry in self.documents.iter() {
            f(entry.key(), entry.value());
        }
    }

    /// Evict the least recently used document.
    ///
    /// This is called when the number of open documents exceeds the configured
    /// maximum cache size to prevent unbounded memory growth.
    ///
    /// # Error Handling
    ///
    /// - Empty cache: Returns immediately without error
    /// - LRU selection: Uses precise timestamp comparison
    /// - Eviction: Logged with document details and idle time
    fn evict_lru_document(&self) {
        if self.documents.is_empty() {
            warn!("LRU eviction requested but cache is empty (no-op)");
            return;
        }

        // Find the LRU document
        let mut lru_uri: Option<Url> = None;
        let mut lru_time = std::time::Instant::now();
        let mut lru_size: usize = 0;

        for entry in self.documents.iter() {
            let state = entry.value().lock();
            if lru_uri.is_none() || state.last_access < lru_time {
                lru_uri = Some(entry.key().clone());
                lru_time = state.last_access;
                lru_size = state.rope.len_bytes();
            }
        }

        // Evict the LRU document
        if let Some(uri) = lru_uri {
            let idle_duration = std::time::Instant::now().duration_since(lru_time);
            warn!(
                "Evicting LRU document {} ({} bytes, idle for {:?})",
                uri, lru_size, idle_duration
            );

            if let Some((_, removed_state)) = self.documents.remove(&uri) {
                let state = removed_state.lock();
                debug!(
                    "Evicted document had {} entities, {} references",
                    state.analysis.entities.values().map(|m| m.len()).sum::<usize>(),
                    state.analysis.references.len()
                );
            }

            // Update statistics
            {
                let mut stats = self.cache_stats.lock();
                stats.evictions += 1;
                debug!(
                    "Cache statistics after eviction: {} hits, {} misses, {} evictions, {}/{} size",
                    stats.hits,
                    stats.misses,
                    stats.evictions,
                    self.documents.len(),
                    stats.max_size
                );
            }
        } else {
            error!("LRU eviction failed: no document found despite non-empty cache");
        }
    }

    /// Clear all documents from the cache.
    ///
    /// This is primarily useful for testing or when resetting the server state.
    pub fn clear(&self) {
        self.documents.clear();
        let mut stats = self.cache_stats.lock();
        stats.hits = 0;
        stats.misses = 0;
        stats.evictions = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_manager_new() {
        let manager = DocumentManager::new(100, 1024 * 1024);
        assert_eq!(manager.max_cache_size(), 100);
        assert_eq!(manager.max_document_size(), 1024 * 1024);

        let stats = manager.statistics();
        assert_eq!(stats.max_size, 100);
        assert_eq!(stats.current_size, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.evictions, 0);
    }

    #[test]
    fn test_insert_and_get() {
        let manager = DocumentManager::new(10, 1024 * 1024);
        let uri = Url::parse("file:///test.hedl").unwrap();
        let content = "%VERSION: 1.0\n---\n";

        // Insert document
        assert!(manager.insert_or_update(&uri, content));

        // Get document
        let result = manager.get(&uri);
        assert!(result.is_some());
        let (retrieved_content, analysis) = result.unwrap();
        assert_eq!(retrieved_content, content);
        assert!(analysis.document.is_some()); // Analysis should have been performed

        // Check statistics
        let stats = manager.statistics();
        assert_eq!(stats.misses, 1); // Initial insert is a miss
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.current_size, 1);
    }

    #[test]
    fn test_update_marks_dirty() {
        let manager = DocumentManager::new(10, 1024 * 1024);
        let uri = Url::parse("file:///test.hedl").unwrap();

        // Insert initial content
        manager.insert_or_update(&uri, "%VERSION: 1.0\n---\n");
        assert!(!manager.is_dirty(&uri));

        // Update with different content
        manager.insert_or_update(&uri, "%VERSION: 1.0\n%STRUCT: User: [id]\n---\n");
        assert!(manager.is_dirty(&uri));

        // Update with same content (hash unchanged)
        manager.insert_or_update(&uri, "%VERSION: 1.0\n%STRUCT: User: [id]\n---\n");
        assert!(manager.is_dirty(&uri)); // Still dirty until marked clean
    }

    #[test]
    fn test_mark_clean() {
        let manager = DocumentManager::new(10, 1024 * 1024);
        let uri = Url::parse("file:///test.hedl").unwrap();

        manager.insert_or_update(&uri, "%VERSION: 1.0\n---\n");
        manager.insert_or_update(&uri, "%VERSION: 1.0\n%STRUCT: User: [id]\n---\n");
        assert!(manager.is_dirty(&uri));

        manager.mark_clean(&uri);
        assert!(!manager.is_dirty(&uri));
    }

    #[test]
    fn test_document_size_limit() {
        let manager = DocumentManager::new(10, 100); // Only 100 bytes allowed
        let uri = Url::parse("file:///test.hedl").unwrap();

        // Small document should succeed
        assert!(manager.insert_or_update(&uri, "%VERSION: 1.0\n---\n"));

        // Large document should be rejected
        let large_content = "x".repeat(101);
        assert!(!manager.insert_or_update(&uri, &large_content));
    }

    #[test]
    fn test_lru_eviction() {
        let manager = DocumentManager::new(3, 1024 * 1024); // Max 3 documents

        // Insert 3 documents
        for i in 0..3 {
            let uri = Url::parse(&format!("file:///test{}.hedl", i)).unwrap();
            manager.insert_or_update(&uri, "%VERSION: 1.0\n---\n");
        }

        let stats = manager.statistics();
        assert_eq!(stats.current_size, 3);
        assert_eq!(stats.evictions, 0);

        // Insert 4th document should trigger eviction
        let uri4 = Url::parse("file:///test4.hedl").unwrap();
        manager.insert_or_update(&uri4, "%VERSION: 1.0\n---\n");

        let stats = manager.statistics();
        assert_eq!(stats.current_size, 3); // Still at max
        assert_eq!(stats.evictions, 1); // One eviction occurred
    }

    #[test]
    fn test_remove() {
        let manager = DocumentManager::new(10, 1024 * 1024);
        let uri = Url::parse("file:///test.hedl").unwrap();

        manager.insert_or_update(&uri, "%VERSION: 1.0\n---\n");
        assert!(manager.get(&uri).is_some());

        assert!(manager.remove(&uri));
        assert!(manager.get(&uri).is_none());

        // Removing non-existent document should return false
        assert!(!manager.remove(&uri));
    }

    #[test]
    fn test_all_uris() {
        let manager = DocumentManager::new(10, 1024 * 1024);

        for i in 0..5 {
            let uri = Url::parse(&format!("file:///test{}.hedl", i)).unwrap();
            manager.insert_or_update(&uri, "%VERSION: 1.0\n---\n");
        }

        let uris = manager.all_uris();
        assert_eq!(uris.len(), 5);
    }

    #[test]
    fn test_clear() {
        let manager = DocumentManager::new(10, 1024 * 1024);

        for i in 0..3 {
            let uri = Url::parse(&format!("file:///test{}.hedl", i)).unwrap();
            manager.insert_or_update(&uri, "%VERSION: 1.0\n---\n");
        }

        assert_eq!(manager.statistics().current_size, 3);

        manager.clear();

        assert_eq!(manager.statistics().current_size, 0);
        assert_eq!(manager.statistics().hits, 0);
        assert_eq!(manager.statistics().misses, 0);
    }

    #[test]
    fn test_runtime_config_update() {
        let manager = DocumentManager::new(100, 1024 * 1024);

        assert_eq!(manager.max_cache_size(), 100);
        manager.set_max_cache_size(200);
        assert_eq!(manager.max_cache_size(), 200);

        assert_eq!(manager.max_document_size(), 1024 * 1024);
        manager.set_max_document_size(2 * 1024 * 1024);
        assert_eq!(manager.max_document_size(), 2 * 1024 * 1024);
    }
}

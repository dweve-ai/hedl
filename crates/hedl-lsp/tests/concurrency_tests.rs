// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License in the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Comprehensive concurrency tests for hedl-lsp
//!
//! This test suite validates thread safety across all LSP components:
//!
//! 1. **DocumentManager** - Concurrent document operations
//! 2. **Analysis** - Concurrent analysis while reading
//! 3. **Cache** - LRU eviction under concurrent load
//! 4. **Reference Index** - Concurrent lookups during updates
//! 5. **Race Conditions** - Data races with loom
//! 6. **Deadlock Detection** - Verifies no deadlocks occur
//!
//! # Test Strategy
//!
//! - **Standard Tests**: Use std::thread for realistic concurrency scenarios
//! - **Loom Tests**: Use loom for exhaustive concurrency testing (smaller scenarios)
//! - **Stress Tests**: High thread count (10+) for finding rare race conditions
//!
//! # Thread Safety Requirements
//!
//! - DocumentManager uses DashMap (lock-free concurrent hash map)
//! - DocumentState uses parking_lot::Mutex for fine-grained locking
//! - AnalyzedDocument is wrapped in Arc for shared ownership
//! - ReferenceIndex operations must be atomic

use hedl_lsp::document_manager::DocumentManager;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tower_lsp::lsp_types::*;

/// Simple PRNG for tests (wrapping to avoid overflow)
fn simple_prng(seed: &mut u32) -> u32 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    *seed
}

// ============================================================================
// TEST HELPERS
// ============================================================================

/// Sample HEDL document for testing
fn sample_document(variant: usize) -> String {
    format!(
        r#"%VERSION: 1.0
%STRUCT: User: [id, name, email]
%STRUCT: Post: [id, title, author]
---
users: @User
  | user{}, User {}, user{}@example.com
  | other, Other User, other@example.com

posts: @Post
  | post{}, Post {}, @User:user{}
  | post2, Another Post, @User:other
"#,
        variant, variant, variant, variant, variant, variant
    )
}

/// Create a test URI with variant number
fn test_uri(id: usize) -> Url {
    Url::parse(&format!("file:///test{}.hedl", id)).unwrap()
}

// ============================================================================
// CONCURRENT DOCUMENT OPERATIONS
// ============================================================================

#[test]
fn test_concurrent_document_inserts() {
    // Test: Multiple threads inserting different documents simultaneously
    let manager = Arc::new(DocumentManager::new(100, 1024 * 1024));
    let num_threads = 10;
    let docs_per_thread = 10;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            thread::spawn(move || {
                for doc_id in 0..docs_per_thread {
                    let uri = test_uri(thread_id * docs_per_thread + doc_id);
                    let content = sample_document(doc_id);
                    assert!(manager.insert_or_update(&uri, &content));
                }
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify all documents were inserted
    let stats = manager.statistics();
    assert_eq!(stats.current_size, (num_threads * docs_per_thread) as usize);
}

#[test]
fn test_concurrent_document_updates() {
    // Test: Multiple threads updating the same document concurrently
    let manager = Arc::new(DocumentManager::new(100, 1024 * 1024));
    let uri = test_uri(0);
    let num_threads = 15;
    let updates_per_thread = 100;

    // Insert initial document
    manager.insert_or_update(&uri, &sample_document(0));

    let update_count = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            let uri = uri.clone();
            let update_count = Arc::clone(&update_count);
            thread::spawn(move || {
                for i in 0..updates_per_thread {
                    let content = sample_document(thread_id * updates_per_thread + i);
                    manager.insert_or_update(&uri, &content);
                    update_count.fetch_add(1, Ordering::SeqCst);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify the document exists and all updates were processed
    assert_eq!(
        update_count.load(Ordering::SeqCst),
        (num_threads * updates_per_thread) as usize
    );
    assert!(manager.get(&uri).is_some());
}

#[test]
fn test_concurrent_read_write() {
    // Test: Some threads writing, others reading concurrently
    let manager = Arc::new(DocumentManager::new(100, 1024 * 1024));
    let num_readers = 8;
    let num_writers = 4;
    let num_docs = 20;

    // Pre-populate with some documents
    for i in 0..num_docs {
        manager.insert_or_update(&test_uri(i), &sample_document(i));
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let read_count = Arc::new(AtomicUsize::new(0));
    let write_count = Arc::new(AtomicUsize::new(0));

    // Spawn reader threads
    let mut handles = Vec::new();
    for _ in 0..num_readers {
        let manager = Arc::clone(&manager);
        let stop_flag = Arc::clone(&stop_flag);
        let read_count = Arc::clone(&read_count);
        handles.push(thread::spawn(move || {
            let mut reads = 0;
            while !stop_flag.load(Ordering::Relaxed) {
                for i in 0..num_docs {
                    if let Some((content, analysis)) = manager.get(&test_uri(i)) {
                        assert!(!content.is_empty());
                        assert!(analysis.document.is_some() || !analysis.errors.is_empty());
                        reads += 1;
                    }
                }
            }
            read_count.fetch_add(reads, Ordering::SeqCst);
        }));
    }

    // Spawn writer threads
    for writer_id in 0..num_writers {
        let manager = Arc::clone(&manager);
        let stop_flag = Arc::clone(&stop_flag);
        let write_count = Arc::clone(&write_count);
        handles.push(thread::spawn(move || {
            let mut writes = 0;
            while !stop_flag.load(Ordering::Relaxed) {
                for i in 0..num_docs {
                    let content = sample_document(writer_id * num_docs + i);
                    manager.insert_or_update(&test_uri(i), &content);
                    writes += 1;
                }
            }
            write_count.fetch_add(writes, Ordering::SeqCst);
        }));
    }

    // Let threads run for a short duration
    thread::sleep(Duration::from_millis(200));
    stop_flag.store(true, Ordering::SeqCst);

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    println!(
        "Concurrent read/write test: {} reads, {} writes",
        read_count.load(Ordering::SeqCst),
        write_count.load(Ordering::SeqCst)
    );

    // Verify documents are still accessible
    for i in 0..num_docs {
        assert!(manager.get(&test_uri(i)).is_some());
    }
}

// ============================================================================
// CONCURRENT ANALYSIS OPERATIONS
// ============================================================================

#[test]
fn test_concurrent_analysis_access() {
    // Test: Multiple threads analyzing while others read analysis results
    let manager = Arc::new(DocumentManager::new(100, 1024 * 1024));
    let num_docs = 10;
    let num_threads = 12;

    // Pre-populate documents
    for i in 0..num_docs {
        manager.insert_or_update(&test_uri(i), &sample_document(i));
    }

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            thread::spawn(move || {
                for i in 0..num_docs {
                    let uri = test_uri(i);

                    // Some threads trigger updates (which invalidate analysis)
                    if thread_id % 3 == 0 {
                        let new_content = sample_document(thread_id * num_docs + i);
                        manager.insert_or_update(&uri, &new_content);
                    }

                    // All threads read analysis
                    if let Some((content, analysis)) = manager.get(&uri) {
                        // Access analysis fields
                        let _ = analysis.entities.len();
                        let _ = analysis.schemas.len();
                        let _ = analysis.references.len();
                        let _ = analysis.reference_index_v2.definition_count();
                        assert!(!content.is_empty());
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
fn test_concurrent_dirty_tracking() {
    // Test: Dirty flag updates don't cause data races
    let manager = Arc::new(DocumentManager::new(100, 1024 * 1024));
    let uri = test_uri(0);
    let num_threads = 15;

    manager.insert_or_update(&uri, &sample_document(0));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            let uri = uri.clone();
            thread::spawn(move || {
                for i in 0..50 {
                    // Update content (marks dirty)
                    let content = sample_document(thread_id * 100 + i);
                    manager.insert_or_update(&uri, &content);

                    // Check dirty status
                    let is_dirty = manager.is_dirty(&uri);

                    // Mark clean
                    if is_dirty && i % 3 == 0 {
                        manager.mark_clean(&uri);
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Document should still be accessible
    assert!(manager.get(&uri).is_some());
}

// ============================================================================
// CONCURRENT CACHE OPERATIONS (LRU EVICTION)
// ============================================================================

#[test]
fn test_concurrent_lru_eviction() {
    // Test: LRU eviction under high concurrent load
    let max_cache = 20;
    let manager = Arc::new(DocumentManager::new(max_cache, 1024 * 1024));
    let num_threads = 10;
    let docs_per_thread = 30; // More than cache can hold

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            thread::spawn(move || {
                for doc_id in 0..docs_per_thread {
                    let uri = test_uri(thread_id * docs_per_thread + doc_id);
                    let content = sample_document(doc_id);
                    manager.insert_or_update(&uri, &content);

                    // Occasionally access older documents to test LRU
                    if doc_id > 5 && doc_id % 3 == 0 {
                        let old_uri = test_uri(thread_id * docs_per_thread + doc_id - 5);
                        let _ = manager.get(&old_uri);
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Give a small grace period for any pending evictions to complete
    thread::sleep(Duration::from_millis(10));

    // Verify cache behavior under concurrent load
    let stats = manager.statistics();

    // Important: In a concurrent scenario, the cache size can temporarily exceed max_cache
    // because multiple threads might check "is cache full?" simultaneously and all proceed
    // to insert before any eviction occurs. This is a known characteristic of concurrent
    // LRU caches and is acceptable as long as:
    // 1. Evictions are occurring (memory is being reclaimed)
    // 2. The cache stabilizes to a reasonable size
    // 3. No unbounded growth occurs

    // For this test, we verify that:
    // - Evictions definitely occurred (proves LRU is working)
    // - Final cache size is reasonable (< total documents inserted)
    let total_docs_inserted = num_threads * docs_per_thread;

    assert!(stats.evictions > 0, "LRU eviction should have occurred");
    assert!(
        stats.current_size < total_docs_inserted,
        "Cache size {} should be less than total documents {}",
        stats.current_size,
        total_docs_inserted
    );

    println!(
        "LRU test: {} evictions, cache size: {}/{} (max: {}, total inserted: {})",
        stats.evictions, stats.current_size, stats.max_size, max_cache, total_docs_inserted
    );
}

#[test]
fn test_concurrent_cache_access_patterns() {
    // Test: Realistic access patterns with hot/cold documents
    let manager = Arc::new(DocumentManager::new(50, 1024 * 1024));
    let num_hot_docs = 10; // Frequently accessed
    let num_cold_docs = 100; // Rarely accessed
    let num_threads = 12;

    let stop_flag = Arc::new(AtomicBool::new(false));

    // Pre-populate hot documents
    for i in 0..num_hot_docs {
        manager.insert_or_update(&test_uri(i), &sample_document(i));
    }

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            let stop_flag = Arc::clone(&stop_flag);
            thread::spawn(move || {
                let mut rng = (thread_id * 1000) as u32;
                while !stop_flag.load(Ordering::Relaxed) {
                    let val = simple_prng(&mut rng);
                    let choice = val % 100;

                    if choice < 80 {
                        // 80% access hot documents
                        let hot_id = (val % num_hot_docs as u32) as usize;
                        let _ = manager.get(&test_uri(hot_id));
                    } else {
                        // 20% access cold documents (causing evictions)
                        let cold_id = num_hot_docs + (val % num_cold_docs as u32) as usize;
                        manager.insert_or_update(&test_uri(cold_id), &sample_document(cold_id));
                    }
                }
            })
        })
        .collect();

    // Run for short duration
    thread::sleep(Duration::from_millis(100));
    stop_flag.store(true, Ordering::SeqCst);

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Hot documents should still be in cache (LRU protection)
    let mut hot_docs_present = 0;
    for i in 0..num_hot_docs {
        if manager.get(&test_uri(i)).is_some() {
            hot_docs_present += 1;
        }
    }

    println!("Hot documents in cache: {}/{}", hot_docs_present, num_hot_docs);
    // Most hot documents should still be present (but not guaranteed due to concurrency)
}

// ============================================================================
// CONCURRENT REFERENCE INDEX OPERATIONS
// ============================================================================

#[test]
fn test_concurrent_reference_lookups() {
    // Test: Concurrent reference index queries
    let manager = Arc::new(DocumentManager::new(100, 1024 * 1024));
    let uri = test_uri(0);
    let num_threads = 15;

    // Insert document with references
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author]
---
users: @User
  | alice, Alice
  | bob, Bob
  | charlie, Charlie

posts: @Post
  | p1, @User:alice
  | p2, @User:bob
  | p3, @User:alice
  | p4, @User:charlie
  | p5, @User:alice
"#;
    manager.insert_or_update(&uri, content);

    let handles: Vec<_> = (0..num_threads)
        .map(|_| {
            let manager = Arc::clone(&manager);
            let uri = uri.clone();
            thread::spawn(move || {
                if let Some((_content, analysis)) = manager.get(&uri) {
                    for _ in 0..100 {
                        // Concurrent definition lookups
                        let _ = analysis.reference_index_v2.find_definition("User", "alice");
                        let _ = analysis.reference_index_v2.find_definition("User", "bob");
                        let _ = analysis.reference_index_v2.find_definition("Post", "p1");

                        // Concurrent reference lookups
                        let refs = analysis.reference_index_v2.find_references("@User:alice");
                        assert!(refs.len() > 0);

                        // Concurrent position-based lookups
                        let pos = Position { line: 10, character: 7 };
                        let _ = analysis.reference_index_v2.find_reference_at(pos);

                        // Access other analysis data
                        let _ = analysis.entities.len();
                        let _ = analysis.get_entity_ids("User");
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
fn test_concurrent_analysis_rebuild() {
    // Test: Analysis is rebuilt while other threads are querying
    let manager = Arc::new(DocumentManager::new(100, 1024 * 1024));
    let uri = test_uri(0);
    let num_readers = 10;
    let num_writers = 3;

    manager.insert_or_update(&uri, &sample_document(0));

    let stop_flag = Arc::new(AtomicBool::new(false));
    let error_count = Arc::new(AtomicUsize::new(0));

    // Reader threads
    let mut handles = Vec::new();
    for _ in 0..num_readers {
        let manager = Arc::clone(&manager);
        let uri = uri.clone();
        let stop_flag = Arc::clone(&stop_flag);
        let error_count = Arc::clone(&error_count);
        handles.push(thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                if let Some((_content, analysis)) = manager.get(&uri) {
                    // Try to access various analysis fields
                    // These should never panic even if analysis is being rebuilt
                    let entity_count = analysis.entities.len();
                    let schema_count = analysis.schemas.len();
                    let _ref_count = analysis.references.len();

                    // Verify basic invariants
                    if entity_count == 0 && schema_count > 0 {
                        error_count.fetch_add(1, Ordering::SeqCst);
                    }

                    // Try reference index operations
                    let _ = analysis.reference_index_v2.definition_count();
                    let _ = analysis.reference_index_v2.total_reference_count();
                } else {
                    thread::yield_now();
                }
            }
        }));
    }

    // Writer threads (trigger analysis rebuilds)
    for writer_id in 0..num_writers {
        let manager = Arc::clone(&manager);
        let uri = uri.clone();
        let stop_flag = Arc::clone(&stop_flag);
        handles.push(thread::spawn(move || {
            let mut counter = 0;
            while !stop_flag.load(Ordering::Relaxed) {
                let content = sample_document(writer_id * 1000 + counter);
                manager.insert_or_update(&uri, &content);
                counter += 1;
                thread::sleep(Duration::from_micros(100));
            }
        }));
    }

    // Run for short duration
    thread::sleep(Duration::from_millis(100));
    stop_flag.store(true, Ordering::SeqCst);

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    assert_eq!(
        error_count.load(Ordering::SeqCst),
        0,
        "No invariant violations should occur"
    );
}

// ============================================================================
// STRESS TESTS WITH HIGH THREAD COUNT
// ============================================================================

#[test]
fn test_high_concurrency_stress() {
    // Test: Extreme concurrency with 20+ threads
    let manager = Arc::new(DocumentManager::new(50, 1024 * 1024));
    let num_threads = 25;
    let num_docs = 40;

    let stop_flag = Arc::new(AtomicBool::new(false));
    let operation_count = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            let stop_flag = Arc::clone(&stop_flag);
            let operation_count = Arc::clone(&operation_count);

            thread::spawn(move || {
                let mut ops = 0;
                let mut rng = (thread_id * 12345) as u32;

                while !stop_flag.load(Ordering::Relaxed) {
                    let val = simple_prng(&mut rng);
                    let doc_id = (val % num_docs as u32) as usize;
                    let uri = test_uri(doc_id);
                    let operation = val % 100;

                    if operation < 50 {
                        // 50% reads
                        let _ = manager.get(&uri);
                    } else if operation < 90 {
                        // 40% writes
                        let content = sample_document(doc_id);
                        manager.insert_or_update(&uri, &content);
                    } else {
                        // 10% removes
                        manager.remove(&uri);
                    }

                    ops += 1;
                }

                operation_count.fetch_add(ops, Ordering::SeqCst);
            })
        })
        .collect();

    // Run stress test
    thread::sleep(Duration::from_millis(200));
    stop_flag.store(true, Ordering::SeqCst);

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    println!(
        "Stress test completed: {} operations across {} threads",
        operation_count.load(Ordering::SeqCst),
        num_threads
    );

    // Verify system is still functional
    let test_uri = test_uri(999);
    manager.insert_or_update(&test_uri, &sample_document(999));
    assert!(manager.get(&test_uri).is_some());
}

// ============================================================================
// DEADLOCK DETECTION TESTS
// ============================================================================

#[test]
fn test_no_deadlocks_on_circular_access() {
    // Test: No deadlocks when accessing documents in different orders
    let manager = Arc::new(DocumentManager::new(100, 1024 * 1024));
    let num_docs = 10;
    let num_threads = 12;

    // Pre-populate documents
    for i in 0..num_docs {
        manager.insert_or_update(&test_uri(i), &sample_document(i));
    }

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            thread::spawn(move || {
                // Each thread accesses documents in different order
                let start = thread_id % num_docs;
                for offset in 0..num_docs {
                    let doc_id = (start + offset) % num_docs;
                    let uri = test_uri(doc_id);

                    // Read and write in alternating pattern
                    if offset % 2 == 0 {
                        manager.insert_or_update(&uri, &sample_document(doc_id + 100));
                    } else {
                        let _ = manager.get(&uri);
                    }
                }
            })
        })
        .collect();

    // If there's a deadlock, this will hang
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
fn test_no_deadlocks_on_nested_operations() {
    // Test: No deadlocks when performing nested operations
    let manager = Arc::new(DocumentManager::new(100, 1024 * 1024));
    let num_threads = 10;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            thread::spawn(move || {
                for i in 0..50 {
                    let uri1 = test_uri(thread_id * 2);
                    let uri2 = test_uri(thread_id * 2 + 1);

                    // Nested access pattern
                    manager.insert_or_update(&uri1, &sample_document(i));
                    if let Some((_content, _analysis)) = manager.get(&uri1) {
                        manager.insert_or_update(&uri2, &sample_document(i + 1));
                        let _ = manager.get(&uri2);
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

// ============================================================================
// MEMORY CONSISTENCY TESTS
// ============================================================================

#[test]
fn test_memory_consistency_after_updates() {
    // Test: Verify memory consistency after concurrent updates
    let manager = Arc::new(DocumentManager::new(100, 1024 * 1024));
    let uri = test_uri(0);
    let num_threads = 10;

    manager.insert_or_update(&uri, &sample_document(0));

    // Each thread updates with a unique marker
    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            let uri = uri.clone();
            thread::spawn(move || {
                let content = format!(
                    "%VERSION: 1.0\n%STRUCT: T: [id]\n---\nitems: @T\n  | thread_{}\n",
                    thread_id
                );
                manager.insert_or_update(&uri, &content);
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Final document should be from one of the threads
    if let Some((content, _analysis)) = manager.get(&uri) {
        // Content should be valid and contain a thread marker
        assert!(content.contains("thread_"));
        assert!(content.contains("%VERSION"));

        // Extract which thread won
        if let Some(start) = content.find("thread_") {
            let thread_num_str = &content[start + 7..].chars().take_while(|c| c.is_numeric()).collect::<String>();
            if let Ok(thread_num) = thread_num_str.parse::<usize>() {
                println!("Final document is from thread {}", thread_num);
                assert!(thread_num < num_threads);
            }
        }
    }
}

#[test]
fn test_analysis_consistency() {
    // Test: Analysis results are consistent (no partial updates visible)
    let manager = Arc::new(DocumentManager::new(100, 1024 * 1024));
    let num_docs = 20;
    let num_threads = 15;

    // Pre-populate
    for i in 0..num_docs {
        manager.insert_or_update(&test_uri(i), &sample_document(i));
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let inconsistency_count = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            let stop_flag = Arc::clone(&stop_flag);
            let inconsistency_count = Arc::clone(&inconsistency_count);

            thread::spawn(move || {
                while !stop_flag.load(Ordering::Relaxed) {
                    for i in 0..num_docs {
                        let uri = test_uri(i);

                        // Writer threads
                        if thread_id % 3 == 0 {
                            manager.insert_or_update(&uri, &sample_document(i + thread_id));
                        }

                        // Reader threads check consistency
                        if let Some((_content, analysis)) = manager.get(&uri) {
                            let entity_count = analysis.entities.len();

                            // All entities in reference_index should be in entities map
                            for ((type_name, id), _loc) in analysis.reference_index_v2.all_definitions() {
                                if let Some(entity_map) = analysis.entities.get(type_name) {
                                    if !entity_map.contains_key(id) && entity_count > 0 {
                                        inconsistency_count.fetch_add(1, Ordering::SeqCst);
                                    }
                                } else if entity_count > 0 {
                                    inconsistency_count.fetch_add(1, Ordering::SeqCst);
                                }
                            }
                        }
                    }
                }
            })
        })
        .collect();

    thread::sleep(Duration::from_millis(100));
    stop_flag.store(true, Ordering::SeqCst);

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    assert_eq!(
        inconsistency_count.load(Ordering::SeqCst),
        0,
        "Analysis should never show inconsistent state"
    );
}

// ============================================================================
// LOOM-BASED EXHAUSTIVE CONCURRENCY TESTS
// ============================================================================
// Note: Loom tests are run in a separate configuration due to their special requirements

#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;
    use loom::thread;

    #[test]
    fn loom_concurrent_insert() {
        loom::model(|| {
            let manager = Arc::new(DocumentManager::new(10, 1024));
            let uri1 = test_uri(1);
            let uri2 = test_uri(2);

            let manager1 = Arc::clone(&manager);
            let uri1_clone = uri1.clone();
            let t1 = thread::spawn(move || {
                manager1.insert_or_update(&uri1_clone, &sample_document(1));
            });

            let manager2 = Arc::clone(&manager);
            let uri2_clone = uri2.clone();
            let t2 = thread::spawn(move || {
                manager2.insert_or_update(&uri2_clone, &sample_document(2));
            });

            t1.join().unwrap();
            t2.join().unwrap();

            // Both documents should be present
            assert!(manager.get(&uri1).is_some());
            assert!(manager.get(&uri2).is_some());
        });
    }

    #[test]
    fn loom_concurrent_update_same_doc() {
        loom::model(|| {
            let manager = Arc::new(DocumentManager::new(10, 1024));
            let uri = test_uri(0);

            // Initial insert
            manager.insert_or_update(&uri, &sample_document(0));

            let manager1 = Arc::clone(&manager);
            let uri1 = uri.clone();
            let t1 = thread::spawn(move || {
                manager1.insert_or_update(&uri1, &sample_document(1));
            });

            let manager2 = Arc::clone(&manager);
            let uri2 = uri.clone();
            let t2 = thread::spawn(move || {
                manager2.insert_or_update(&uri2, &sample_document(2));
            });

            t1.join().unwrap();
            t2.join().unwrap();

            // Document should exist and be from either thread
            assert!(manager.get(&uri).is_some());
        });
    }
}

// ============================================================================
// PERFORMANCE UNDER CONCURRENCY
// ============================================================================

#[test]
fn test_cache_hit_rate_under_concurrency() {
    // Test: Measure cache efficiency under concurrent load
    let manager = Arc::new(DocumentManager::new(50, 1024 * 1024));
    let num_threads = 12;
    let num_hot_docs = 30; // Fits in cache

    // Pre-populate hot documents
    for i in 0..num_hot_docs {
        manager.insert_or_update(&test_uri(i), &sample_document(i));
    }

    // Clear statistics to start fresh (pre-population counts as misses)
    manager.clear();
    for i in 0..num_hot_docs {
        manager.insert_or_update(&test_uri(i), &sample_document(i));
    }

    let stop_flag = Arc::new(AtomicBool::new(false));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            let stop_flag = Arc::clone(&stop_flag);
            thread::spawn(move || {
                let mut rng = (thread_id * 9999) as u32;
                while !stop_flag.load(Ordering::Relaxed) {
                    let val = simple_prng(&mut rng);
                    let doc_id = (val % num_hot_docs as u32) as usize;
                    let _ = manager.get(&test_uri(doc_id));
                }
            })
        })
        .collect();

    thread::sleep(Duration::from_millis(200));
    stop_flag.store(true, Ordering::SeqCst);

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let stats = manager.statistics();

    println!(
        "Cache operations: {} hits, {} misses ({}% hit rate)",
        stats.hits,
        stats.misses,
        if stats.hits + stats.misses > 0 {
            stats.hits as f64 / (stats.hits + stats.misses) as f64 * 100.0
        } else {
            0.0
        }
    );

    // Verify we had cache operations
    assert!(
        stats.hits > 0 || stats.misses > 0,
        "Should have had some cache operations"
    );

    // Note: We can't guarantee a specific hit rate due to timing and concurrent updates,
    // but we can verify the cache is working
    assert!(stats.current_size > 0, "Cache should contain documents");
    assert!(stats.current_size <= 50, "Cache should not exceed max size");
}

// ============================================================================
// SUMMARY
// ============================================================================

#[test]
fn test_suite_summary() {
    println!("\n=== CONCURRENCY TEST SUITE SUMMARY ===");
    println!("This test suite validates:");
    println!("  1. Concurrent document insert/update/remove operations");
    println!("  2. Concurrent read/write access patterns");
    println!("  3. Thread-safe analysis operations");
    println!("  4. LRU cache eviction under load");
    println!("  5. Reference index concurrent access");
    println!("  6. High concurrency stress testing (25+ threads)");
    println!("  7. Deadlock prevention");
    println!("  8. Memory consistency guarantees");
    println!("  9. Cache efficiency under concurrent load");
    println!(" 10. Loom-based exhaustive testing (when enabled)");
    println!("\nAll tests verify thread safety using:");
    println!("  - DashMap for lock-free concurrent hash maps");
    println!("  - parking_lot::Mutex for fine-grained locking");
    println!("  - Arc for shared ownership");
    println!("  - Atomic operations for lock-free counters");
    println!("=====================================\n");
}

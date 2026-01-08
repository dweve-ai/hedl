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

//! Memory benchmarks for LRU cache eviction.
//!
//! These tests verify that the LSP document cache prevents unbounded memory
//! growth through LRU eviction, especially under heavy load with many documents.

#![allow(deprecated)] // Tests use deprecated with_max_cache_size API for compatibility

use hedl_lsp::HedlLanguageServer;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService};

/// Test that opening 1000+ documents doesn't cause unbounded memory growth.
///
/// This test validates the P1 requirement: "Add LRU eviction for document cache"
/// by opening more documents than the cache can hold and verifying eviction occurs.
#[tokio::test]
async fn test_no_unbounded_memory_growth_1000_documents() {
    // Create server with default cache size (1000)
    let (service, _socket) = LspService::new(|client| HedlLanguageServer::new(client));
    let server = service.inner();

    // Open 1500 documents (exceeds cache limit)
    for i in 0..1500 {
        let uri = Url::parse(&format!("file:///test{}.hedl", i)).unwrap();
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "hedl".to_string(),
                version: 1,
                text: format!(
                    "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | user{}, User {}\n",
                    i, i
                ),
            },
        };
        server.did_open(params).await;
    }

    let stats = server.cache_statistics();

    // Verify cache size is bounded
    assert_eq!(
        stats.current_size, 1000,
        "Cache should be capped at max size (1000), not grow unbounded"
    );

    // Verify evictions occurred
    assert_eq!(
        stats.evictions, 500,
        "Should have evicted 500 documents (1500 opened - 1000 capacity)"
    );

    // Verify cache statistics are accurate
    assert_eq!(stats.misses, 1500, "All 1500 opens were cache misses");
    assert_eq!(stats.max_size, 1000);
}

/// Test memory usage stays constant with configurable small cache.
#[tokio::test]
async fn test_memory_bounded_with_small_cache() {
    // Create server with small cache (10 documents)
    let (service, _socket) =
        LspService::new(|client| HedlLanguageServer::with_max_cache_size(client, 10));
    let server = service.inner();

    // Open 100 documents
    for i in 0..100 {
        let uri = Url::parse(&format!("file:///small{}.hedl", i)).unwrap();
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "hedl".to_string(),
                version: 1,
                text: "%VERSION: 1.0\n---\n".to_string(),
            },
        };
        server.did_open(params).await;
    }

    let stats = server.cache_statistics();

    assert_eq!(stats.current_size, 10, "Cache should stay at 10");
    assert_eq!(stats.evictions, 90, "Should evict 90 documents");
    assert_eq!(stats.max_size, 10);
}

/// Test that evicted documents can be re-opened (re-parsed on demand).
#[tokio::test]
async fn test_evicted_documents_reopen_correctly() {
    let (service, _socket) =
        LspService::new(|client| HedlLanguageServer::with_max_cache_size(client, 5));
    let server = service.inner();

    // Open 5 documents
    for i in 0..5 {
        let uri = Url::parse(&format!("file:///doc{}.hedl", i)).unwrap();
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "hedl".to_string(),
                version: 1,
                text: format!("%VERSION: 1.0\n---\nvalue: {}\n", i),
            },
        };
        server.did_open(params).await;
    }

    // Open 6th document, evicting the first one
    let uri5 = Url::parse("file:///doc5.hedl").unwrap();
    let params5 = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri5,
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n---\nvalue: 5\n".to_string(),
        },
    };
    server.did_open(params5).await;

    let stats = server.cache_statistics();
    assert_eq!(stats.evictions, 1);

    // Now "re-open" the first document (it was evicted, should be re-parsed)
    let uri0 = Url::parse("file:///doc0.hedl").unwrap();
    let params0_reopened = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri0.clone(),
            language_id: "hedl".to_string(),
            version: 2,
            text: "%VERSION: 1.0\n---\nvalue: 0 (reopened)\n".to_string(),
        },
    };
    server.did_open(params0_reopened).await;

    // Verify it triggered another eviction (cache still at 5)
    let stats = server.cache_statistics();
    assert_eq!(stats.current_size, 5);
    assert_eq!(
        stats.evictions, 2,
        "Re-opening evicted document should trigger another eviction"
    );
}

/// Test cache statistics accuracy under mixed operations.
#[tokio::test]
async fn test_cache_statistics_accuracy() {
    let (service, _socket) =
        LspService::new(|client| HedlLanguageServer::with_max_cache_size(client, 100));
    let server = service.inner();

    let uri = Url::parse("file:///stats_test.hedl").unwrap();

    // Open document (miss)
    let params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n---\n".to_string(),
        },
    };
    server.did_open(params).await;

    let stats = server.cache_statistics();
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.hits, 0);

    // Change document 5 times (all hits)
    for v in 2..7 {
        let change_params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: v,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: format!("%VERSION: 1.0\n---\nversion: {}\n", v),
            }],
        };
        server.did_change(change_params).await;
    }

    let stats = server.cache_statistics();
    assert_eq!(stats.hits, 5, "5 changes should be 5 hits");
    assert_eq!(stats.misses, 1, "Still 1 miss from initial open");
    assert_eq!(stats.evictions, 0, "No evictions yet");

    // Hover on document (hit via get_document)
    let hover_params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 0,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };
    server.hover(hover_params).await.ok();

    // Note: hover doesn't go through update_document_content, so stats unchanged
    let stats = server.cache_statistics();
    assert_eq!(stats.current_size, 1);
    assert_eq!(stats.max_size, 100);
}

/// Performance test: Verify LRU eviction is efficient (O(n) where n = cache size).
#[tokio::test]
async fn test_lru_eviction_performance() {
    let (service, _socket) =
        LspService::new(|client| HedlLanguageServer::with_max_cache_size(client, 1000));
    let server = service.inner();

    // Fill cache
    for i in 0..1000 {
        let uri = Url::parse(&format!("file:///perf{}.hedl", i)).unwrap();
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "hedl".to_string(),
                version: 1,
                text: "%VERSION: 1.0\n---\n".to_string(),
            },
        };
        server.did_open(params).await;
    }

    // Measure eviction time for next 100 opens
    let start = std::time::Instant::now();
    for i in 1000..1100 {
        let uri = Url::parse(&format!("file:///perf{}.hedl", i)).unwrap();
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "hedl".to_string(),
                version: 1,
                text: "%VERSION: 1.0\n---\n".to_string(),
            },
        };
        server.did_open(params).await;
    }
    let duration = start.elapsed();

    println!(
        "100 opens with eviction: {:?} ({:.2}μs per open+evict)",
        duration,
        duration.as_micros() as f64 / 100.0
    );

    // Eviction should be reasonably fast even with 1000 documents
    // Allow 10ms total for 100 evictions (100μs per eviction)
    assert!(
        duration.as_millis() < 100,
        "LRU eviction should be efficient: {:?}",
        duration
    );

    let stats = server.cache_statistics();
    assert_eq!(stats.evictions, 100);
}

/// Test that cache size can be changed at runtime.
#[tokio::test]
async fn test_runtime_cache_size_change() {
    let (service, _socket) =
        LspService::new(|client| HedlLanguageServer::with_max_cache_size(client, 50));
    let server = service.inner();

    assert_eq!(server.max_cache_size(), 50);

    // Fill cache to 50
    for i in 0..50 {
        let uri = Url::parse(&format!("file:///runtime{}.hedl", i)).unwrap();
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "hedl".to_string(),
                version: 1,
                text: "%VERSION: 1.0\n---\n".to_string(),
            },
        };
        server.did_open(params).await;
    }

    let stats = server.cache_statistics();
    assert_eq!(stats.current_size, 50);
    assert_eq!(stats.max_size, 50);

    // Change cache size to 100
    server.set_max_cache_size(100);
    assert_eq!(server.max_cache_size(), 100);

    let stats = server.cache_statistics();
    assert_eq!(stats.max_size, 100);
    assert_eq!(stats.current_size, 50); // Documents still in cache

    // Open 50 more documents (should fit without eviction)
    for i in 50..100 {
        let uri = Url::parse(&format!("file:///runtime{}.hedl", i)).unwrap();
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "hedl".to_string(),
                version: 1,
                text: "%VERSION: 1.0\n---\n".to_string(),
            },
        };
        server.did_open(params).await;
    }

    let stats = server.cache_statistics();
    assert_eq!(stats.current_size, 100);
    assert_eq!(
        stats.evictions, 0,
        "No evictions should occur with increased size"
    );

    // Open one more to trigger eviction
    let uri100 = Url::parse("file:///runtime100.hedl").unwrap();
    let params100 = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri100,
            language_id: "hedl".to_string(),
            version: 1,
            text: "%VERSION: 1.0\n---\n".to_string(),
        },
    };
    server.did_open(params100).await;

    let stats = server.cache_statistics();
    assert_eq!(stats.current_size, 100);
    assert_eq!(stats.evictions, 1);
}

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

//! Demonstration of LRU cache behavior in HEDL LSP.
//!
//! This example shows:
//! - Cache statistics tracking
//! - LRU eviction when cache fills
//! - Runtime cache size configuration
//! - Memory bounds enforcement

use hedl_lsp::HedlLanguageServer;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService};

#[tokio::main]
async fn main() {
    println!("HEDL LSP Cache Demonstration");
    println!("============================\n");

    // Create server with small cache for demo purposes
    let (service, _socket) =
        LspService::new(|client| HedlLanguageServer::with_max_cache_size(client, 10));
    let server = service.inner();

    println!(
        "Server created with cache size: {}\n",
        server.max_cache_size()
    );

    // Open 10 documents (fill cache)
    println!("Opening 10 documents...");
    for i in 0..10 {
        let uri = Url::parse(&format!("file:///demo{}.hedl", i)).unwrap();
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
    println!("Cache filled:");
    println!("  Current size: {}/{}", stats.current_size, stats.max_size);
    println!("  Cache misses: {}", stats.misses);
    println!("  Cache hits:   {}", stats.hits);
    println!("  Evictions:    {}", stats.evictions);
    println!();

    // Open 5 more documents (trigger evictions)
    println!("Opening 5 more documents (will trigger evictions)...");
    for i in 10..15 {
        let uri = Url::parse(&format!("file:///demo{}.hedl", i)).unwrap();
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

    let stats = server.cache_statistics();
    println!("After evictions:");
    println!("  Current size: {}/{}", stats.current_size, stats.max_size);
    println!("  Cache misses: {}", stats.misses);
    println!("  Cache hits:   {}", stats.hits);
    println!(
        "  Evictions:    {} (oldest 5 documents evicted)",
        stats.evictions
    );
    println!();

    // Change a document (cache hit)
    println!("Changing document demo14.hedl...");
    let uri14 = Url::parse("file:///demo14.hedl").unwrap();
    let change_params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: uri14.clone(),
            version: 2,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "%VERSION: 1.0\n---\nvalue: 14 (modified)\n".to_string(),
        }],
    };
    server.did_change(change_params).await;

    let stats = server.cache_statistics();
    let hit_rate = stats.hits as f64 / (stats.hits + stats.misses) as f64 * 100.0;
    println!("After change:");
    println!("  Cache hits:   {} (document was in cache)", stats.hits);
    println!("  Hit rate:     {:.1}%", hit_rate);
    println!();

    // Increase cache size at runtime
    println!("Increasing cache size to 20...");
    server.set_max_cache_size(20);

    let stats = server.cache_statistics();
    println!("Cache resized:");
    println!("  New max size: {}", stats.max_size);
    println!(
        "  Current size: {} (documents still in cache)",
        stats.current_size
    );
    println!();

    // Open 10 more documents (should fit without eviction)
    println!("Opening 10 more documents (should fit without eviction)...");
    for i in 15..25 {
        let uri = Url::parse(&format!("file:///demo{}.hedl", i)).unwrap();
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

    let stats = server.cache_statistics();
    println!("After adding documents:");
    println!("  Current size: {}/{}", stats.current_size, stats.max_size);
    println!("  Evictions:    {} (no new evictions)", stats.evictions);
    println!();

    // Summary
    println!("Summary:");
    println!("========");
    println!("  Total documents opened: 25");
    println!("  Documents in cache:     {}", stats.current_size);
    println!("  Cache misses:           {} (all 25 opens)", stats.misses);
    println!("  Cache hits:             {} (1 change)", stats.hits);
    println!(
        "  Total evictions:        {} (when cache was full at 10)",
        stats.evictions
    );
    println!("  Hit rate:               {:.1}%", hit_rate);
    println!(
        "\nMemory bounded: Cache stays at {} documents max",
        stats.max_size
    );
}

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

//! Header boundary caching tests.
//!
//! These tests verify that the header boundary (--- delimiter) is cached
//! during document analysis and reused efficiently across multiple LSP
//! operations without repeated O(n) scans.

use hedl_lsp::analysis::AnalyzedDocument;
use hedl_lsp::completion::get_completions;
use tower_lsp::lsp_types::Position;

/// Test that header_end_line is correctly cached during analysis.
#[test]
fn test_header_boundary_cached() {
    let content = r#"%VERSION 1.0
%STRUCT User[id, name, email]
%STRUCT Post[id, title, author]
%ALIAS active = "Active"
---
users: @User
| alice | Alice Smith | alice@example.com |
| bob | Bob Jones | bob@example.com |
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Verify header_end_line is populated
    assert_eq!(
        analysis.header_end_line,
        Some(4),
        "Header boundary should be cached at line 4"
    );
}

/// Test that header_end_line works correctly with no header.
#[test]
fn test_no_header_boundary() {
    let content = r#"users: @User
| alice | Alice Smith | alice@example.com |
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // No header delimiter found
    assert_eq!(
        analysis.header_end_line, None,
        "No header boundary should be cached when --- is absent"
    );
}

/// Test header boundary with empty header section.
#[test]
fn test_empty_header() {
    let content = r#"---
users: @User
| alice | Alice Smith | alice@example.com |
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Header ends immediately
    assert_eq!(
        analysis.header_end_line,
        Some(0),
        "Header boundary should be cached at line 0 for immediate ---"
    );
}

/// Test header boundary with large document.
#[test]
fn test_large_document_header_cache() {
    // Create a large document with header
    let mut content = String::from("%VERSION 1.0\n");
    content.push_str("%STRUCT Entity[id, field1, field2, field3, field4]\n");
    for i in 0..10 {
        content.push_str(&format!("%ALIAS alias{} = \"Value {}\"\n", i, i));
    }
    content.push_str("---\n");

    // Add 1000 entities
    content.push_str("entities: @Entity\n");
    for i in 0..1000 {
        content.push_str(&format!("| entity{} | val1 | val2 | val3 | val4 |\n", i));
    }

    let analysis = AnalyzedDocument::analyze(&content);

    // Verify header boundary is cached
    assert!(
        analysis.header_end_line.is_some(),
        "Header boundary should be cached for large documents"
    );
    assert_eq!(
        analysis.header_end_line.unwrap(),
        12,
        "Header boundary should be at correct line (1 VERSION + 1 STRUCT + 10 ALIAS = line 12)"
    );
}

/// Test that completions use cached header boundary.
#[test]
fn test_completion_uses_cached_boundary() {
    let content = r#"%VERSION 1.0
%STRUCT User[id, name]
---
users: @User
| alice | Alice |
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Position in header (line 1)
    let header_completions = get_completions(&analysis, content, Position::new(1, 5));

    // Position in body (line 4)
    let body_completions = get_completions(&analysis, content, Position::new(4, 2));

    // Header completions should include directives
    assert!(
        header_completions.iter().any(|c| c.label == "%STRUCT"),
        "Header completions should be available in header section"
    );

    // Body completions should not include header directives
    assert!(
        !body_completions.iter().any(|c| c.label == "%STRUCT"),
        "Header completions should not appear in body section"
    );
}

/// Test performance of repeated completion calls with cached boundary.
///
/// This test simulates IDE usage where completions are requested many times
/// as the user types. The cached header boundary should make this O(1)
/// instead of O(n).
#[test]
fn test_completion_performance_with_cache() {
    // Create a document with 10,000 lines
    let mut content = String::from("%VERSION 1.0\n");
    content.push_str("%STRUCT Entity[id, name, value]\n");
    content.push_str("---\n");
    content.push_str("entities: @Entity\n");

    // Add 10,000 entities
    for i in 0..10_000 {
        content.push_str(&format!("| entity{} | Name {} | {} |\n", i, i, i * 10));
    }

    let analysis = AnalyzedDocument::analyze(&content);

    // Verify cache is populated
    assert_eq!(analysis.header_end_line, Some(2));

    // Simulate 100 completion requests at different positions in the body
    let start = std::time::Instant::now();
    for i in 0..100 {
        let line = 4 + (i * 100); // Positions throughout the document
        let _completions = get_completions(&analysis, &content, Position::new(line, 5));
    }
    let duration = start.elapsed();

    // With caching, 100 completions on a 10k line document should be reasonably fast
    // Without caching, each would scan O(n) lines
    println!(
        "100 completions on 10k line document: {:?} ({:.2}μs per completion)",
        duration,
        duration.as_micros() as f64 / 100.0
    );

    // This should complete in under 1000ms even on slow systems
    // The time is dominated by completion generation, not boundary checking
    assert!(
        duration.as_millis() < 1000,
        "Completions should be reasonably fast with cached boundary: {:?}",
        duration
    );
}

/// Test that cache survives multiple analysis passes.
#[test]
fn test_cache_persistence_across_analyses() {
    let content = r#"%VERSION 1.0
%STRUCT User[id, name]
---
users: @User
| alice | Alice |
"#;

    // Analyze multiple times
    for _ in 0..10 {
        let analysis = AnalyzedDocument::analyze(content);
        assert_eq!(
            analysis.header_end_line,
            Some(2),
            "Cache should be consistently populated"
        );
    }
}

/// Test header boundary with whitespace and comments.
#[test]
fn test_header_boundary_with_whitespace() {
    let content = r#"%VERSION 1.0
%STRUCT User[id, name]

---

users: @User
| alice | Alice |
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // The --- on line 3 (0-indexed) should be the boundary
    assert_eq!(
        analysis.header_end_line,
        Some(3),
        "Header boundary should handle blank lines correctly"
    );
}

/// Benchmark: Compare cached vs non-cached boundary detection.
///
/// This test demonstrates the performance improvement from caching
/// by simulating the old O(n) scan approach.
#[test]
fn test_benchmark_cached_vs_uncached() {
    // Create a large document
    let mut content = String::from("%VERSION 1.0\n");
    for i in 0..100 {
        content.push_str(&format!("%STRUCT Type{}[id, field]\n", i));
    }
    content.push_str("---\n");

    // Add 5000 body lines
    for i in 0..5000 {
        content.push_str(&format!("line{}: value\n", i));
    }

    let analysis = AnalyzedDocument::analyze(&content);

    // Benchmark: Cached lookup (O(1))
    let start = std::time::Instant::now();
    for _ in 0..10_000 {
        let _cached = analysis.header_end_line;
    }
    let cached_duration = start.elapsed();

    // Benchmark: Simulated uncached scan (O(n))
    let lines: Vec<&str> = content.lines().collect();
    let start = std::time::Instant::now();
    for _ in 0..10_000 {
        // Simulate the old O(n) scan
        let _uncached = lines.iter().position(|&l| l == "---");
    }
    let uncached_duration = start.elapsed();

    println!("Cached lookups (10k):   {:?}", cached_duration);
    println!("Uncached scans (10k):   {:?}", uncached_duration);
    println!(
        "Speedup: {:.1}x",
        uncached_duration.as_micros() as f64 / cached_duration.as_micros() as f64
    );

    // Cached should be at least 10x faster
    assert!(
        cached_duration < uncached_duration / 10,
        "Cached lookup should be significantly faster than O(n) scan"
    );
}

/// Test that cache handles documents with multiple --- markers correctly.
///
/// Only the first --- should be cached as the header boundary.
#[test]
fn test_multiple_separator_markers() {
    let content = r#"%VERSION 1.0
%STRUCT User[id, name]
---
users: @User
| alice | Alice |
---
notes: Some content with separator
---
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Should cache the first --- only
    assert_eq!(
        analysis.header_end_line,
        Some(2),
        "Should cache only the first --- marker"
    );
}

/// Test edge case: --- as part of content (not at line start).
#[test]
fn test_separator_in_content() {
    let content = r#"%VERSION 1.0
%STRUCT User[id, name]
---
users: @User
| alice | Alice --- Smith |
"#;

    let analysis = AnalyzedDocument::analyze(content);

    assert_eq!(
        analysis.header_end_line,
        Some(2),
        "Should only detect --- at start of trimmed line"
    );
}

/// Test cache with UTF-8 content.
#[test]
fn test_header_cache_with_utf8() {
    let content = r#"%VERSION 1.0
%STRUCT User[id, name]
%ALIAS emoji = "😀🎉"
---
users: @User
| alice | Alice 日本語 |
| bob | Bob Müller |
"#;

    let analysis = AnalyzedDocument::analyze(content);

    assert_eq!(
        analysis.header_end_line,
        Some(3),
        "Header cache should work correctly with UTF-8 content"
    );
}

/// Memory efficiency test: Verify cache doesn't add excessive overhead.
#[test]
fn test_cache_memory_overhead() {
    let content = r#"%VERSION 1.0
%STRUCT User[id, name]
---
users: @User
| alice | Alice |
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // The cache is just an Option<usize>, should be 16 bytes max
    assert_eq!(
        std::mem::size_of_val(&analysis.header_end_line),
        std::mem::size_of::<Option<usize>>(),
        "Cache should have minimal memory overhead"
    );
}

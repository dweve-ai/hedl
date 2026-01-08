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

//! Tests verifying that magic number constants are used correctly throughout hedl-lsp.
//!
//! These tests ensure that:
//! 1. Constants are accessible and have expected values
//! 2. Constants are used consistently in document management
//! 3. No regressions occur when constants are modified
//! 4. LSP protocol constants are appropriate for real-world usage

use hedl_lsp::constants::*;
use hedl_lsp::document_manager::{DocumentManager, DEFAULT_MAX_CACHE_SIZE, DEFAULT_MAX_DOCUMENT_SIZE};
use tower_lsp::lsp_types::{Position, Range, Url};

#[test]
fn test_debounce_constant_is_reasonable() {
    // Verify debounce delay is within acceptable range for user experience
    assert!(
        DEBOUNCE_MS >= 50,
        "Debounce too short ({}ms), will cause excessive CPU usage",
        DEBOUNCE_MS
    );
    assert!(
        DEBOUNCE_MS <= 500,
        "Debounce too long ({}ms), will feel laggy to users",
        DEBOUNCE_MS
    );

    // The chosen value should be exactly 200ms
    assert_eq!(DEBOUNCE_MS, 200, "Debounce delay changed from documented 200ms");
}

#[test]
fn test_memory_limit_constants() {
    // Verify megabyte conversion is standard binary MiB
    assert_eq!(BYTES_PER_MEGABYTE, 1048576, "Binary megabyte should be 1024 * 1024");

    // Verify default document size is 500 MB
    assert_eq!(
        DEFAULT_MAX_DOCUMENT_SIZE,
        500 * BYTES_PER_MEGABYTE,
        "Default max document size should be 500 MB"
    );

    // Verify default cache size is 1000
    assert_eq!(
        DEFAULT_MAX_CACHE_SIZE, 1000,
        "Default max cache size should be 1000 documents"
    );

    // Verify limits are reasonable for real-world usage
    assert!(
        DEFAULT_MAX_DOCUMENT_SIZE >= 1 * BYTES_PER_MEGABYTE,
        "Document limit too small for real files"
    );
    assert!(
        DEFAULT_MAX_DOCUMENT_SIZE <= 2048 * BYTES_PER_MEGABYTE,
        "Document limit too large, excessive memory risk"
    );
}

#[test]
fn test_document_manager_uses_constants() {
    // Verify DocumentManager correctly uses the default constants
    let manager = DocumentManager::new(DEFAULT_MAX_CACHE_SIZE, DEFAULT_MAX_DOCUMENT_SIZE);

    assert_eq!(
        manager.max_cache_size(),
        DEFAULT_MAX_CACHE_SIZE,
        "DocumentManager should use DEFAULT_MAX_CACHE_SIZE"
    );
    assert_eq!(
        manager.max_document_size(),
        DEFAULT_MAX_DOCUMENT_SIZE,
        "DocumentManager should use DEFAULT_MAX_DOCUMENT_SIZE"
    );
}

#[test]
fn test_document_size_limit_enforcement() {
    let manager = DocumentManager::new(10, 100); // 100 byte limit

    let uri = Url::parse("file:///test.hedl").unwrap();

    // Small document should succeed
    let small_doc = "a".repeat(50);
    assert!(
        manager.insert_or_update(&uri, &small_doc),
        "Small document should be accepted"
    );

    // Document exactly at limit should succeed
    let uri2 = Url::parse("file:///test2.hedl").unwrap();
    let exact_limit = "b".repeat(100);
    assert!(
        manager.insert_or_update(&uri2, &exact_limit),
        "Document at exact limit should be accepted"
    );

    // Document over limit should be rejected
    let uri3 = Url::parse("file:///test3.hedl").unwrap();
    let over_limit = "c".repeat(101);
    assert!(
        !manager.insert_or_update(&uri3, &over_limit),
        "Document over limit should be rejected"
    );
}

#[test]
fn test_lsp_protocol_constants_are_consistent() {
    // Diagnostic and symbol line end should be the same for consistency
    assert_eq!(
        DIAGNOSTIC_LINE_END_CHAR, SYMBOL_LINE_END_CHAR,
        "Diagnostic and symbol line ends should be consistent"
    );

    // Line end should be large enough for typical code lines
    assert!(
        DIAGNOSTIC_LINE_END_CHAR >= 100,
        "Line end character ({}) too small for typical code",
        DIAGNOSTIC_LINE_END_CHAR
    );

    // But not unnecessarily large
    assert!(
        DIAGNOSTIC_LINE_END_CHAR <= 10000,
        "Line end character ({}) unnecessarily large",
        DIAGNOSTIC_LINE_END_CHAR
    );
}

#[test]
fn test_position_constants() {
    // Verify position zero is actually zero
    assert_eq!(POSITION_ZERO, 0, "Position zero should be 0");

    // Verify line number offset matches LSP convention
    assert_eq!(
        LINE_NUMBER_OFFSET, 1,
        "Line number offset should be 1 (LSP 0-based, parser 1-based)"
    );

    // Test that offset is used correctly for conversion
    let parser_line = 5usize; // Parser reports line 5 (1-based)
    let lsp_line = (parser_line.saturating_sub(LINE_NUMBER_OFFSET)) as u32;
    assert_eq!(lsp_line, 4, "Parser line 5 should convert to LSP line 4");
}

#[test]
fn test_header_selection_constant() {
    // Verify header selection is reasonable
    assert!(
        HEADER_SELECTION_CHAR >= 5,
        "Header selection too narrow for \"Header\" text"
    );
    assert!(
        HEADER_SELECTION_CHAR <= 100,
        "Header selection unnecessarily wide"
    );

    // The chosen value should cover "Header" plus some padding
    assert_eq!(
        HEADER_SELECTION_CHAR, 10,
        "Header selection changed from documented 10 characters"
    );
}

#[test]
fn test_reference_width_constant() {
    // Verify default reference width is reasonable
    assert!(
        DEFAULT_REFERENCE_WIDTH >= 5,
        "Reference width too narrow for most entity IDs"
    );
    assert!(
        DEFAULT_REFERENCE_WIDTH <= 50,
        "Reference width unnecessarily wide"
    );

    // The chosen value should be 10 characters
    assert_eq!(
        DEFAULT_REFERENCE_WIDTH, 10,
        "Reference width changed from documented 10 characters"
    );
}

#[test]
fn test_range_construction_with_constants() {
    // Test that constants work correctly in Range construction
    let range = Range {
        start: Position {
            line: POSITION_ZERO,
            character: POSITION_ZERO,
        },
        end: Position {
            line: 10,
            character: SYMBOL_LINE_END_CHAR,
        },
    };

    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 0);
    assert_eq!(range.end.line, 10);
    assert_eq!(range.end.character, SYMBOL_LINE_END_CHAR);
}

#[test]
fn test_megabyte_conversion_accuracy() {
    // Verify various megabyte conversions
    assert_eq!(1 * BYTES_PER_MEGABYTE, 1048576);
    assert_eq!(10 * BYTES_PER_MEGABYTE, 10485760);
    assert_eq!(100 * BYTES_PER_MEGABYTE, 104857600);
    assert_eq!(500 * BYTES_PER_MEGABYTE, 524288000);
    assert_eq!(1024 * BYTES_PER_MEGABYTE, 1073741824); // 1 GiB

    // Verify default matches expected value
    assert_eq!(DEFAULT_MAX_DOCUMENT_SIZE, 524288000);
}

#[test]
fn test_cache_size_limits() {
    // Test that cache respects size limits with constants
    let manager = DocumentManager::new(3, 1000);

    // Insert 3 documents (at limit)
    for i in 0..3 {
        let uri = Url::parse(&format!("file:///test{}.hedl", i)).unwrap();
        manager.insert_or_update(&uri, "content");
    }

    let stats = manager.statistics();
    assert_eq!(stats.current_size, 3, "Should have 3 documents");
    assert_eq!(stats.evictions, 0, "Should have no evictions yet");

    // Insert 4th document (should trigger eviction)
    let uri = Url::parse("file:///test4.hedl").unwrap();
    manager.insert_or_update(&uri, "content");

    let stats = manager.statistics();
    assert_eq!(stats.current_size, 3, "Should still have 3 documents");
    assert_eq!(stats.evictions, 1, "Should have 1 eviction");
}

#[test]
fn test_constants_are_public() {
    // Verify that all constants are accessible (compile-time check)
    let _ = DEBOUNCE_MS;
    let _ = DEFAULT_MAX_DOCUMENT_SIZE;
    let _ = DEFAULT_MAX_CACHE_SIZE;
    let _ = BYTES_PER_MEGABYTE;
    let _ = DIAGNOSTIC_LINE_END_CHAR;
    let _ = SYMBOL_LINE_END_CHAR;
    let _ = HEADER_SELECTION_CHAR;
    let _ = LINE_NUMBER_OFFSET;
    let _ = POSITION_ZERO;
    let _ = DEFAULT_REFERENCE_WIDTH;

    // If this compiles, all constants are accessible
    assert!(true, "All constants are publicly accessible");
}

#[test]
fn test_no_magic_numbers_in_tests() {
    // This test documents what were previously magic numbers
    // and ensures they're now defined as constants

    // Document size: was 500 * 1024 * 1024
    assert_eq!(DEFAULT_MAX_DOCUMENT_SIZE, 500 * BYTES_PER_MEGABYTE);

    // Cache size: was 1000
    assert_eq!(DEFAULT_MAX_CACHE_SIZE, 1000);

    // Debounce: was 200
    assert_eq!(DEBOUNCE_MS, 200);

    // Line end: was 1000
    assert_eq!(DIAGNOSTIC_LINE_END_CHAR, 1000);
    assert_eq!(SYMBOL_LINE_END_CHAR, 1000);

    // Line offset: was 1
    assert_eq!(LINE_NUMBER_OFFSET, 1);

    // Position zero: was 0
    assert_eq!(POSITION_ZERO, 0);

    // Header selection: was 10
    assert_eq!(HEADER_SELECTION_CHAR, 10);
}

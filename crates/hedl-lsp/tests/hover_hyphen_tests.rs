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

//! Tests for hover word detection and definition indexing with hyphens.
//!
//! Validates fixes for MEDIUM priority LSP issues (Batch 3):
//! - Issue 1: Hover word detection with hyphens in identifiers
//! - Issue 2: Definition indexing with row prefixes

use hedl_lsp::analysis::AnalyzedDocument;
use hedl_lsp::hover::get_hover;
use tower_lsp::lsp_types::*;

#[test]
fn test_hover_on_hyphenated_entity_id() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Product:[id, name]
---
products:@Product
 |my-product, Test Product
 |other-item, Another Product
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Test hovering over the hyphenated ID "my-product"
    // Line 0: %V:2.0, Line 1: %NULL:~, Line 2: %QUOTE:", Line 3: %S:..., Line 4: ---, Line 5: products:, Line 6: |my-product
    // Format: " |my-product" - char 0: space, 1: |, 2: m, 3: y, 4: -, 5: p...
    let position = Position {
        line: 6,
        character: 3, // Point to 'y' in 'my-product'
    };

    let hover = get_hover(&analysis, content, position);
    assert!(
        hover.is_some(),
        "Should detect hyphenated word and provide hover"
    );

    let hover = hover.unwrap();
    if let HoverContents::Markup(markup) = hover.contents {
        let value = markup.value;
        assert!(
            value.contains("my-product") || value.contains("Product"),
            "Hover should reference the entity or its type, got: {value}"
        );
    }
}

#[test]
fn test_hover_on_hyphenated_reference() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name]
---
users:@User
 |user-1, Alice
 |user-2, Bob

data:@Arbitrary
 |d1, @User:user-1
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Find the reference "@User:user-1" and hover over it
    let line = " |d1, @User:user-1";
    let ref_pos = line.find("@User:user-1").unwrap();

    // Line 10: |d1, @User:user-1
    let position = Position {
        line: 10,
        character: (ref_pos + 8) as u32, // In the "user-1" part
    };

    let hover = get_hover(&analysis, content, position);
    assert!(
        hover.is_some(),
        "Should detect reference with hyphenated ID"
    );

    let hover = hover.unwrap();
    if let HoverContents::Markup(markup) = hover.contents {
        let value = markup.value;
        assert!(
            value.contains("user-1") || value.contains("User"),
            "Should reference full hyphenated ID or type, got: {value}"
        );
    }
}

#[test]
fn test_definition_indexing_with_bracket_row_prefix() {
    let content = r#"%V:1.2
%S:Task:[id, priority]
---
tasks:@Task
 |[3] task-alpha, high
 |[7] task-beta, medium
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Verify entities are found despite row prefixes
    assert!(
        analysis.entity_exists(Some("Task"), "task-alpha"),
        "Should find task-alpha despite |[3] prefix"
    );
    assert!(
        analysis.entity_exists(Some("Task"), "task-beta"),
        "Should find task-beta despite |[7] prefix"
    );

    // Check reference index v2 has correct character positions
    let ref_index = &analysis.reference_index_v2;

    let def_opt = ref_index.find_definition("Task", "task-alpha");
    assert!(
        def_opt.is_some(),
        "Should have indexed definition for task-alpha"
    );

    if let Some(def) = def_opt {
        assert_eq!(
            def.line, 4,
            "Definition should be on line 4 (0-indexed), got line {}",
            def.line
        );

        // Line is: " |[3] task-alpha, high"
        // The start char should point to "task-alpha", not the row prefix
        let line = " |[3] task-alpha, high";
        let expected_start = line.find("task-alpha").unwrap() as u32;

        assert_eq!(
            def.start_char, expected_start,
            "Start char should be at position {} (where 'task-alpha' starts), got {}",
            expected_start, def.start_char
        );
    }
}

#[test]
fn test_hover_range_includes_full_hyphenated_word() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Widget:[id, name]
---
widgets:@Widget
 |cool-widget-name, Test
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Hover somewhere in the middle of "cool-widget-name"
    let line = " |cool-widget-name, Test";
    let id_start = line.find("cool-widget-name").unwrap();

    let position = Position {
        line: 6,
        character: (id_start + 7) as u32, // In the "widget" part
    };

    let hover = get_hover(&analysis, content, position);
    assert!(hover.is_some(), "Should get hover for hyphenated ID");

    let hover = hover.unwrap();

    // Check that the hover range encompasses the entire hyphenated word
    if let Some(range) = hover.range {
        assert_eq!(range.start.line, 6, "Hover range should be on line 6");

        // The range should cover the entire "cool-widget-name"
        let expected_start = id_start as u32;
        let expected_end = (id_start + "cool-widget-name".len()) as u32;

        assert_eq!(
            range.start.character, expected_start,
            "Hover range should start at {}, got {}",
            expected_start, range.start.character
        );
        assert_eq!(
            range.end.character, expected_end,
            "Hover range should end at {}, got {}",
            expected_end, range.end.character
        );
    }
}

#[test]
fn test_multiple_hyphens_in_identifier() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Component:[id, description]
---
components:@Component
 |very-long-hyphenated-identifier-here, Description
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Test that we can hover anywhere in the multi-hyphenated ID
    let line = " |very-long-hyphenated-identifier-here, Description";
    let id_start = line.find("very-long-hyphenated-identifier-here").unwrap();

    // Test at different positions within the ID
    for offset in [0, 5, 10, 20, 30, 35] {
        let position = Position {
            line: 6,
            character: (id_start + offset) as u32,
        };

        let hover = get_hover(&analysis, content, position);
        assert!(
            hover.is_some(),
            "Should get hover at offset {offset} in multi-hyphenated ID"
        );

        if let Some(h) = hover {
            if let Some(range) = h.range {
                // Range should always cover the full identifier
                let expected_start = id_start as u32;
                assert_eq!(
                    range.start.character, expected_start,
                    "At offset {offset}, range should start at {expected_start}"
                );
            }
        }
    }
}

#[test]
fn test_underscore_vs_hyphen_word_detection() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Mixed:[id, name]
---
mixed:@Mixed
 |under_score_id, Name1
 |hyphen-id, Name2
 |mixed_hyphen-id, Name3
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // All three ID styles should be detected
    assert!(analysis.entity_exists(Some("Mixed"), "under_score_id"));
    assert!(analysis.entity_exists(Some("Mixed"), "hyphen-id"));
    assert!(analysis.entity_exists(Some("Mixed"), "mixed_hyphen-id"));

    // Test hover detection for each style
    let test_cases = vec![
        (6, "under_score_id"),
        (7, "hyphen-id"),
        (8, "mixed_hyphen-id"),
    ];

    for (line_num, expected_id) in test_cases {
        let line = format!(" |{expected_id}, Name");
        let id_start = line.find(expected_id).unwrap();

        let position = Position {
            line: line_num,
            character: (id_start + 3) as u32, // Middle of ID
        };

        let hover = get_hover(&analysis, content, position);
        assert!(
            hover.is_some(),
            "Should get hover for ID style: {expected_id}"
        );
    }
}

#[test]
fn test_definition_indexing_with_quoted_hyphenated_id() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Record:[id, value]
---
records:@Record
 |"my-hyphenated-id", test-value
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // The ID should be stored without quotes
    assert!(
        analysis.entity_exists(Some("Record"), "my-hyphenated-id"),
        "Should find quoted hyphenated ID"
    );

    // Check reference index
    let ref_index = &analysis.reference_index_v2;
    let def_opt = ref_index.find_definition("Record", "my-hyphenated-id");

    assert!(
        def_opt.is_some(),
        "Should have indexed quoted hyphenated ID"
    );

    if let Some(def) = def_opt {
        // Line is: " |"my-hyphenated-id", test-value"
        // The start should point inside the quotes to the actual ID
        let line = r#" |"my-hyphenated-id", test-value"#;
        let quote_pos = line.find('"').unwrap();
        let expected_start = (quote_pos + 1) as u32; // After opening quote

        assert_eq!(
            def.start_char, expected_start,
            "Start char should be at position {} (after opening quote), got {}",
            expected_start, def.start_char
        );
    }
}

#[test]
fn test_comprehensive_hyphenated_workflow() {
    // Complete workflow test: parsing, hover, references, and indexing
    // Note: v2.0 removed |[N] inline count hints, so we use standard row syntax
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Product:[id, name, category]
%S:Order:[id, product_ref, quantity]
---
products:@Product
 |gaming-laptop, Gaming Laptop Pro, electronics
 |office-chair, Ergonomic Office Chair, furniture
 |wireless-mouse, Wireless Mouse, electronics

orders:@Order
 |order-1, @Product:gaming-laptop, 2
 |order-2, @Product:wireless-mouse, 5
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Verify all entities are found
    assert!(analysis.entity_exists(Some("Product"), "gaming-laptop"));
    assert!(analysis.entity_exists(Some("Product"), "office-chair"));
    assert!(analysis.entity_exists(Some("Product"), "wireless-mouse"));
    assert!(analysis.entity_exists(Some("Order"), "order-1"));
    assert!(analysis.entity_exists(Some("Order"), "order-2"));

    // Test hover on hyphenated entity ID in definition
    let line = " |gaming-laptop, Gaming Laptop Pro, electronics";
    let id_start = line.find("gaming-laptop").unwrap();
    let position = Position {
        line: 7,
        character: (id_start + 5) as u32,
    };

    let hover = get_hover(&analysis, content, position);
    assert!(hover.is_some(), "Should get hover on entity definition");
    if let Some(h) = hover {
        if let HoverContents::Markup(markup) = h.contents {
            assert!(
                markup.value.contains("gaming-laptop") || markup.value.contains("Product"),
                "Hover should mention the entity ID or type"
            );
        }
    }

    // Test hover on hyphenated reference
    let ref_line = " |order-1, @Product:gaming-laptop, 2";
    let ref_start = ref_line.find("@Product:gaming-laptop").unwrap();
    let ref_position = Position {
        line: 12,
        character: (ref_start + 15) as u32,
    };

    let ref_hover = get_hover(&analysis, content, ref_position);
    assert!(ref_hover.is_some(), "Should get hover on reference");

    // Test definition indexing for the wireless-mouse entry
    let ref_index = &analysis.reference_index_v2;
    let def_opt = ref_index.find_definition("Product", "wireless-mouse");
    assert!(
        def_opt.is_some(),
        "Should find definition for wireless-mouse"
    );

    if let Some(def) = def_opt {
        assert_eq!(def.line, 9, "Definition should be on line 9");
        let standard_line = " |wireless-mouse, Wireless Mouse, electronics";
        let expected_start = standard_line.find("wireless-mouse").unwrap() as u32;
        assert_eq!(
            def.start_char, expected_start,
            "Start char should point to the ID"
        );
    }
}

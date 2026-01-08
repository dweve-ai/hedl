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

//! Integration tests for the enhanced reference index.
//!
//! These tests verify that the O(1) reference index correctly:
//! - Builds precise location information for all references
//! - Provides fast lookups for definitions
//! - Provides fast lookups for all references to an entity
//! - Correctly handles both qualified and unqualified references

use hedl_lsp::analysis::AnalyzedDocument;
use tower_lsp::lsp_types::Position;

#[test]
fn test_reference_index_v2_basic() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, title, author]
---
users: @User
  | alice, Alice Smith
  | bob, Bob Jones

posts: @Post
  | post1, First Post, @User:alice
  | post2, Second Post, @User:bob
  | post3, Third Post, @User:alice
"#;

    eprintln!("Content:\n{}", content);
    eprintln!("Content length: {}", content.len());

    let analysis = AnalyzedDocument::analyze(content);

    // Note: Parser may report reference validation errors, but this doesn't prevent
    // entity extraction and index building
    if !analysis.errors.is_empty() {
        eprintln!("Parse errors (expected for unqualified references): {:?}", analysis.errors);
    }

    // Test definition lookup
    eprintln!("Definition count: {}", analysis.reference_index_v2.definition_count());
    eprintln!("Entities: {:?}", analysis.entities);
    for ((t, id), loc) in analysis.reference_index_v2.all_definitions() {
        eprintln!("Found definition: {}:{} at line {}", t, id, loc.line);
    }

    let alice_def = analysis.reference_index_v2.find_definition("User", "alice");
    assert!(alice_def.is_some(), "Should find alice definition");
    assert_eq!(alice_def.unwrap().line, 5);

    let bob_def = analysis.reference_index_v2.find_definition("User", "bob");
    assert!(bob_def.is_some(), "Should find bob definition");
    assert_eq!(bob_def.unwrap().line, 6);

    // Test reference lookup
    let alice_refs = analysis.reference_index_v2.find_references("@User:alice");
    assert!(!alice_refs.is_empty(), "Should find alice references");

    // Test find reference at position
    let pos = Position {
        line: 9,
        character: 28, // Position on @User:alice
    };
    let ref_at = analysis.reference_index_v2.find_reference_at(pos);
    assert!(ref_at.is_some(), "Should find reference at position");
    let (ref_str, _) = ref_at.unwrap();
    assert_eq!(ref_str, "@User:alice");
}

#[test]
fn test_reference_index_v2_unqualified() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice

ref: @alice
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Test unqualified reference lookup
    let refs = analysis.reference_index_v2.find_references("@alice");
    assert!(!refs.is_empty(), "Should find unqualified references");
}

#[test]
fn test_reference_index_v2_multiple_references() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author]
---
users: @User
  | alice, Alice

posts: @Post
  | p1, @User:alice
  | p2, @User:alice
  | p3, @User:alice
  | p4, @alice
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Should find all qualified references
    let qualified_refs = analysis.reference_index_v2.find_references("@User:alice");
    assert_eq!(
        qualified_refs.len(),
        3,
        "Should find 3 qualified references"
    );

    // Should also find unqualified reference
    let all_refs = analysis.reference_index_v2.find_references("@alice");
    assert!(
        all_refs.len() >= 4,
        "Should find at least 4 references total (qualified + unqualified)"
    );
}

#[test]
fn test_reference_index_v2_character_positions() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice

ref: @User:alice
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Test that character positions are precise
    let refs = analysis.reference_index_v2.find_references("@User:alice");
    assert!(!refs.is_empty(), "Should find reference");

    let ref_loc = &refs[0];
    assert_eq!(ref_loc.line, 6);
    assert!(
        ref_loc.start_char < ref_loc.end_char,
        "Should have valid character range"
    );
    assert_eq!(
        ref_loc.end_char - ref_loc.start_char,
        "@User:alice".len() as u32,
        "Character range should match reference length"
    );
}

#[test]
fn test_reference_index_v2_find_at_position() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice

ref1: @User:alice
ref2: @alice
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Test finding qualified reference
    let pos1 = Position {
        line: 6,
        character: 7, // Inside @User:alice
    };
    let found1 = analysis.reference_index_v2.find_reference_at(pos1);
    assert!(found1.is_some(), "Should find qualified reference");
    assert_eq!(found1.unwrap().0, "@User:alice");

    // Test finding unqualified reference
    let pos2 = Position {
        line: 7,
        character: 7, // Inside @alice
    };
    let found2 = analysis.reference_index_v2.find_reference_at(pos2);
    assert!(found2.is_some(), "Should find unqualified reference");
    assert_eq!(found2.unwrap().0, "@alice");

    // Test position outside any reference
    let pos3 = Position {
        line: 0,
        character: 0,
    };
    let found3 = analysis.reference_index_v2.find_reference_at(pos3);
    assert!(
        found3.is_none(),
        "Should not find reference at non-reference position"
    );
}

#[test]
fn test_reference_index_v2_statistics() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author]
---
users: @User
  | alice, Alice
  | bob, Bob

posts: @Post
  | p1, @User:alice
  | p2, @User:bob
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Test statistics
    let def_count = analysis.reference_index_v2.definition_count();
    assert!(
        def_count >= 4,
        "Should have at least 4 definitions (2 User + 2 Post)"
    );

    let ref_count = analysis.reference_index_v2.total_reference_count();
    assert!(ref_count > 0, "Should have references");
}

#[test]
fn test_reference_index_v2_performance() {
    // Generate a large document with many references
    let mut content = String::from("%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n");

    for i in 0..1000 {
        content.push_str(&format!("  | user{}, User {}\n", i, i));
    }

    let analysis = AnalyzedDocument::analyze(&content);

    // All lookups should be O(1) regardless of document size
    let start = std::time::Instant::now();
    for i in 0..100 {
        let _ = analysis
            .reference_index_v2
            .find_definition("User", &format!("user{}", i));
    }
    let duration = start.elapsed();

    // 100 lookups should be fast (< 1ms for O(1) operations)
    assert!(
        duration.as_millis() < 10,
        "100 definition lookups should complete in < 10ms (was {}ms)",
        duration.as_millis()
    );
}

#[test]
fn test_reference_index_v2_with_special_characters() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice-smith, Alice Smith
  | bob_jones, Bob Jones

ref1: @User:alice-smith
ref2: @User:bob_jones
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Test that hyphens and underscores are handled correctly
    let alice_refs = analysis
        .reference_index_v2
        .find_references("@User:alice-smith");
    assert!(
        !alice_refs.is_empty(),
        "Should find references with hyphens"
    );

    let bob_refs = analysis
        .reference_index_v2
        .find_references("@User:bob_jones");
    assert!(
        !bob_refs.is_empty(),
        "Should find references with underscores"
    );
}

#[test]
fn test_reference_index_v2_nonexistent() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Test looking up non-existent entities
    let def = analysis
        .reference_index_v2
        .find_definition("User", "nonexistent");
    assert!(def.is_none(), "Should not find non-existent definition");

    let refs = analysis
        .reference_index_v2
        .find_references("@User:nonexistent");
    assert!(refs.is_empty(), "Should not find non-existent references");

    // Test non-existent type
    let def2 = analysis
        .reference_index_v2
        .find_definition("NonExistentType", "alice");
    assert!(
        def2.is_none(),
        "Should not find definition for non-existent type"
    );
}

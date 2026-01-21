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

//! Tests for LSP bug fixes.
//!
//! These tests verify that the following bugs are fixed:
//! 1. Reference index flags @ in comments/strings
//! 2. Definition scan fails for quoted IDs
//! 3. Reference hit-testing off by one

use hedl_lsp::analysis::AnalyzedDocument;
use tower_lsp::lsp_types::Position;

#[test]
fn test_bug_1_reference_in_comment_not_indexed() {
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
---
# This is a comment with @User:alice
users: @User
  | alice, Alice Smith
";

    let analysis = AnalyzedDocument::analyze(content);

    // The @ in the comment should NOT be indexed as a reference
    let refs = analysis.reference_index_v2.find_references("@User:alice");

    // Should be 0 references (the comment @ should be skipped)
    assert_eq!(
        refs.len(),
        0,
        "Should not index @ in comments, but found {} references",
        refs.len()
    );
}

#[test]
fn test_bug_1_reference_in_string_not_indexed() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, "email@example.com"
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // The @ in the email string should NOT be indexed as a reference
    let refs = analysis.reference_index_v2.find_references("@example");

    assert_eq!(
        refs.len(),
        0,
        "Should not index @ in quoted strings, but found {} references",
        refs.len()
    );

    // Also check email@example.com isn't indexed
    let email_refs = analysis.reference_index_v2.find_references("@email");
    assert_eq!(email_refs.len(), 0, "Should not index @ in quoted strings");
}

#[test]
fn test_bug_2_quoted_id_definition_found() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | "my-id", My User
  | "another-id", Another User
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Should find definitions for quoted IDs
    let my_id_def = analysis.reference_index_v2.find_definition("User", "my-id");
    assert!(
        my_id_def.is_some(),
        "Should find definition for quoted ID 'my-id'"
    );

    let another_def = analysis
        .reference_index_v2
        .find_definition("User", "another-id");
    assert!(
        another_def.is_some(),
        "Should find definition for quoted ID 'another-id'"
    );
}

#[test]
fn test_bug_2_quoted_id_reference_found() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author]
---
users: @User
  | "my-id", My User

posts: @Post
  | post1, @User:"my-id"
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Should find references with quoted IDs
    let refs = analysis.reference_index_v2.find_references("@User:my-id");
    assert!(!refs.is_empty(), "Should find references to quoted ID");
}

#[test]
fn test_bug_3_reference_hit_testing_at_start() {
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice

ref: @User:alice
";

    let analysis = AnalyzedDocument::analyze(content);

    // Find the reference location
    let refs = analysis.reference_index_v2.find_references("@User:alice");
    assert!(!refs.is_empty(), "Should find reference");

    let ref_loc = &refs[0];

    // Test cursor at the very start of the reference (on the @)
    let pos_at_start = Position {
        line: ref_loc.line,
        character: ref_loc.start_char,
    };

    let found = analysis.reference_index_v2.find_reference_at(pos_at_start);
    assert!(
        found.is_some(),
        "Should find reference when cursor is at start position (character {})",
        ref_loc.start_char
    );
}

#[test]
fn test_bug_3_reference_hit_testing_at_end() {
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice

ref: @User:alice
";

    let analysis = AnalyzedDocument::analyze(content);

    // Find the reference location
    let refs = analysis.reference_index_v2.find_references("@User:alice");
    assert!(!refs.is_empty(), "Should find reference");

    let ref_loc = &refs[0];

    // Test cursor at the end of the reference (one past the last character)
    // This should NOT match (ranges are end-exclusive)
    let pos_at_end = Position {
        line: ref_loc.line,
        character: ref_loc.end_char,
    };

    let found = analysis.reference_index_v2.find_reference_at(pos_at_end);
    assert!(
        found.is_none(),
        "Should NOT find reference when cursor is at end position (character {})",
        ref_loc.end_char
    );
}

#[test]
fn test_bug_3_reference_hit_testing_just_before_end() {
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice

ref: @User:alice
";

    let analysis = AnalyzedDocument::analyze(content);

    // Find the reference location
    let refs = analysis.reference_index_v2.find_references("@User:alice");
    assert!(!refs.is_empty(), "Should find reference");

    let ref_loc = &refs[0];

    // Test cursor one character before the end (on the last character)
    let pos_before_end = Position {
        line: ref_loc.line,
        character: ref_loc.end_char - 1,
    };

    let found = analysis
        .reference_index_v2
        .find_reference_at(pos_before_end);
    assert!(
        found.is_some(),
        "Should find reference when cursor is on last character (character {})",
        ref_loc.end_char - 1
    );
}

#[test]
fn test_comment_and_string_complex_scenario() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name, email, bio]
---
# Comment with @fake:ref and email@test.com
users: @User
  | alice, "Alice", "alice@example.com", "Bio with @mention"
  | bob, "Bob", "bob@test.org", "Not a comment but bio"

# Another comment with @User:bob
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Should NOT find any references for these:
    assert_eq!(
        analysis.reference_index_v2.find_references("@fake").len(),
        0,
        "Should not index @fake in comment"
    );
    assert_eq!(
        analysis.reference_index_v2.find_references("@test").len(),
        0,
        "Should not index @test in email"
    );
    assert_eq!(
        analysis
            .reference_index_v2
            .find_references("@mention")
            .len(),
        0,
        "Should not index @mention in quoted string"
    );
    assert_eq!(
        analysis.reference_index_v2.find_references("@alice").len(),
        0,
        "Should not index @alice in email"
    );
}

#[test]
fn test_quoted_id_with_special_chars() {
    let content = r#"%VERSION: 1.0
%STRUCT: Product: [id, name]
---
products: @Product
  | "product-123", "Product 123"
  | "item_456", "Item 456"
  | "test:special", "Test Special"
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // All quoted IDs should be found
    assert!(
        analysis
            .reference_index_v2
            .find_definition("Product", "product-123")
            .is_some(),
        "Should find quoted ID with hyphen"
    );
    assert!(
        analysis
            .reference_index_v2
            .find_definition("Product", "item_456")
            .is_some(),
        "Should find quoted ID with underscore"
    );
    assert!(
        analysis
            .reference_index_v2
            .find_definition("Product", "test:special")
            .is_some(),
        "Should find quoted ID with colon"
    );
}

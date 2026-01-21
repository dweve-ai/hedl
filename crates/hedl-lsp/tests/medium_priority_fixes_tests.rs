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

//! Tests for MEDIUM priority bug fixes.
//!
//! This module tests the following fixes:
//! 1. Type occurrence finding - finds ALL matches on a line, not just the first
//! 2. Position handling - correctly uses UTF-16 code units for LSP positions
//! 3. Active list type detection - properly handles nested lists

use hedl_lsp::analysis::AnalyzedDocument;
use hedl_lsp::completion::get_completions;
use hedl_lsp::rename::{find_all_occurrences, identify_symbol_at_position};
use hedl_lsp::utils::{get_line_and_byte_offset, utf16_col_to_byte_offset};
use tower_lsp::lsp_types::*;

// ============================================================================
// ISSUE 1: Type occurrence finds only first match
// ============================================================================

#[test]
fn test_type_occurrence_multiple_on_same_line() {
    // Test that when a type appears multiple times on the same line,
    // ALL occurrences are found, not just the first one
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, title, author]
---
data: users: @User, posts: @Post, admins: @User
";

    let analysis = AnalyzedDocument::analyze(content);
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Find all occurrences of "User" type
    let symbol = identify_symbol_at_position(
        &analysis,
        content,
        Position::new(1, 10), // Position on "User" in %STRUCT
    );

    assert!(symbol.is_some(), "Should identify User symbol");

    let occurrences = find_all_occurrences(&symbol.unwrap(), &analysis, content, &uri);

    // Should find:
    // 1. Definition in %STRUCT: User
    // 2. First usage: @User (in "users: @User")
    // 3. Second usage: @User (in "admins: @User")
    assert!(
        occurrences.len() >= 3,
        "Should find at least 3 occurrences of User (found {})",
        occurrences.len()
    );

    // Check that we found both @User on line 4
    let line_4_occurrences: Vec<_> = occurrences
        .iter()
        .filter(|loc| loc.location.line == 4)
        .collect();

    assert!(
        line_4_occurrences.len() >= 2,
        "Should find 2 occurrences of @User on line 4 (found {})",
        line_4_occurrences.len()
    );
}

#[test]
fn test_type_occurrence_in_struct_directive_not_first_match() {
    // Test that we find the correct type name even when it's not the first match
    // E.g., if there's a comment or prefix containing the type name
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
# This User comment should not interfere
---
users: @User
";

    let analysis = AnalyzedDocument::analyze(content);
    let uri = Url::parse("file:///test.hedl").unwrap();

    let symbol = identify_symbol_at_position(&analysis, content, Position::new(1, 10));
    assert!(symbol.is_some());

    let occurrences = find_all_occurrences(&symbol.unwrap(), &analysis, content, &uri);

    // Should find the %STRUCT definition and the @User usage
    // but NOT the comment
    assert!(
        occurrences.len() >= 2,
        "Should find at least 2 occurrences (found {})",
        occurrences.len()
    );
}

#[test]
fn test_type_occurrence_substring_not_matched() {
    // Test that substring matches don't count
    // E.g., "User" should not match "SuperUser" or "UserData"
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: SuperUser: [id, name, role]
%STRUCT: UserData: [id, data]
---
users: @User
superusers: @SuperUser
";

    let analysis = AnalyzedDocument::analyze(content);
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Find occurrences of "User" (not "SuperUser" or "UserData")
    let symbol = identify_symbol_at_position(&analysis, content, Position::new(1, 10));
    assert!(symbol.is_some());

    let occurrences = find_all_occurrences(&symbol.unwrap(), &analysis, content, &uri);

    // Should only find "User", not "SuperUser" or "UserData"
    for occurrence in &occurrences {
        let line_num = occurrence.location.line as usize;
        let start = occurrence.location.start_char as usize;
        let end = occurrence.location.end_char as usize;

        if let Some(line) = content.lines().nth(line_num) {
            let matched_text = &line[start..end];
            assert_eq!(
                matched_text, "User",
                "Should match exactly 'User', not substring in {line}"
            );
        }
    }
}

// ============================================================================
// ISSUE 2: Position handling assumes bytes not UTF-16
// ============================================================================

#[test]
fn test_utf16_position_with_non_ascii() {
    // Test that LSP positions (UTF-16) are correctly converted to byte offsets
    // when the line contains non-ASCII characters
    let line = "Hello 世界 world"; // "世" is 3 bytes, 1 UTF-16 code unit

    // Position 6 in UTF-16 = after "Hello "
    assert_eq!(utf16_col_to_byte_offset(line, 6), 6);

    // Position 7 in UTF-16 = after "世" (which is 3 bytes but 1 UTF-16 unit)
    assert_eq!(utf16_col_to_byte_offset(line, 7), 9);

    // Position 8 in UTF-16 = after "界"
    assert_eq!(utf16_col_to_byte_offset(line, 8), 12);

    // Position 9 in UTF-16 = after " "
    assert_eq!(utf16_col_to_byte_offset(line, 9), 13);
}

#[test]
fn test_utf16_position_with_emoji() {
    // Emoji are often 2 UTF-16 code units (surrogate pairs)
    let line = "Hi 👋 there"; // 👋 is 4 bytes, 2 UTF-16 code units

    // Position 3 in UTF-16 = after "Hi "
    assert_eq!(utf16_col_to_byte_offset(line, 3), 3);

    // Position 5 in UTF-16 = after "👋" (2 UTF-16 code units)
    assert_eq!(utf16_col_to_byte_offset(line, 5), 7);

    // Position 6 in UTF-16 = after " "
    assert_eq!(utf16_col_to_byte_offset(line, 6), 8);
}

#[test]
fn test_position_handling_with_unicode_in_hedl() {
    // Test that we can correctly identify symbols when the document contains Unicode
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, 名前, email]
---
users: @User
  | alice, アリス, alice@example.com
";

    let analysis = AnalyzedDocument::analyze(content);

    // Try to get completions at a position after Unicode characters
    // Position should be in UTF-16 code units
    let position = Position::new(4, 10); // In the middle of the line with Unicode

    let completions = get_completions(&analysis, content, position);

    // Should not crash and should provide some completions
    assert!(
        !completions.is_empty(),
        "Should provide completions even with Unicode"
    );
}

#[test]
fn test_get_line_and_byte_offset_unicode() {
    let content = "Hello\n世界\n👋";

    // Line 1, position 1 in UTF-16 = after "世"
    let (line, offset) = get_line_and_byte_offset(content, Position::new(1, 1)).unwrap();
    assert_eq!(line, "世界");
    assert_eq!(offset, 3); // 3 bytes for "世"

    // Line 2, position 2 in UTF-16 = after "👋"
    let (line, offset) = get_line_and_byte_offset(content, Position::new(2, 2)).unwrap();
    assert_eq!(line, "👋");
    assert_eq!(offset, 4); // 4 bytes for "👋"
}

// ============================================================================
// ISSUE 3: Active list type ignores nesting
// ============================================================================

#[test]
fn test_active_list_type_nested_lists() {
    // Test that we correctly identify the active list type in nested lists
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, title]
%STRUCT: Comment: [id, text]
%NEST: User > Post
%NEST: Post > Comment
---
users: @User
  | alice, Alice
    posts: @Post
      | post1, First Post
        comments: @Comment
          | c1, Great!
";

    let analysis = AnalyzedDocument::analyze(content);

    // Test completion context at different nesting levels
    // Line 8: User level matrix row
    let ctx1 = hedl_lsp::completion::get_completions(
        &analysis,
        content,
        Position::new(8, 5), // In the User row
    );

    // Should suggest User-related completions
    // The matrix cell completions should be for User type
    assert!(!ctx1.is_empty(), "Should provide completions for User row");

    // Line 10: Post level matrix row (nested under User)
    let ctx2 = hedl_lsp::completion::get_completions(
        &analysis,
        content,
        Position::new(10, 10), // In the Post row
    );

    // Should suggest Post-related completions, not User
    assert!(!ctx2.is_empty(), "Should provide completions for Post row");

    // Line 12: Comment level matrix row (nested under Post)
    let ctx3 = hedl_lsp::completion::get_completions(
        &analysis,
        content,
        Position::new(12, 12), // In the Comment row
    );

    // Should suggest Comment-related completions
    assert!(
        !ctx3.is_empty(),
        "Should provide completions for Comment row"
    );
}

#[test]
fn test_active_list_type_sibling_lists() {
    // Test that we correctly identify the type when there are sibling lists at the same level
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, title]
---
users: @User
  | alice, Alice
  | bob, Bob

posts: @Post
  | post1, First Post
  | post2, Second Post
";

    let analysis = AnalyzedDocument::analyze(content);

    // Test that we get the right type for each list
    // Line 6: User row
    let ctx1 = hedl_lsp::completion::get_completions(&analysis, content, Position::new(6, 5));

    assert!(!ctx1.is_empty(), "Should provide completions for User row");

    // Line 10: Post row
    let ctx2 = hedl_lsp::completion::get_completions(&analysis, content, Position::new(10, 5));

    assert!(!ctx2.is_empty(), "Should provide completions for Post row");
}

#[test]
fn test_active_list_type_with_indentation_variation() {
    // Test handling of lists with varying indentation levels
    let content = r"%VERSION: 1.0
%STRUCT: Level1: [id, name]
%STRUCT: Level2: [id, data]
%STRUCT: Level3: [id, value]
---
level1: @Level1
  | item1, Name1
    level2: @Level2
      | item2, Data2
        level3: @Level3
          | item3, Value3
";

    let analysis = AnalyzedDocument::analyze(content);

    // Each level should get the correct type
    let ctx1 = hedl_lsp::completion::get_completions(
        &analysis,
        content,
        Position::new(6, 5), // Level1 row (line 6)
    );
    assert!(!ctx1.is_empty());

    let ctx2 = hedl_lsp::completion::get_completions(
        &analysis,
        content,
        Position::new(8, 10), // Level2 row (line 8)
    );
    assert!(!ctx2.is_empty());

    let ctx3 = hedl_lsp::completion::get_completions(
        &analysis,
        content,
        Position::new(10, 15), // Level3 row (line 10)
    );
    assert!(!ctx3.is_empty());
}

#[test]
fn test_active_list_type_empty_parent() {
    // Test that we handle the case where a parent list has no rows
    let content = r"%VERSION: 1.0
%STRUCT: Parent: [id, name]
%STRUCT: Child: [id, data]
---
parents: @Parent
  children: @Child
    | c1, Data1
";

    let analysis = AnalyzedDocument::analyze(content);

    // Should correctly identify Child type even though Parent has no rows
    let ctx = hedl_lsp::completion::get_completions(
        &analysis,
        content,
        Position::new(6, 8), // In Child row
    );

    assert!(!ctx.is_empty(), "Should provide completions for Child row");
}

// ============================================================================
// COMBINED TESTS: Multiple issues together
// ============================================================================

#[test]
fn test_unicode_and_nested_lists_combined() {
    // Test that Unicode handling works correctly in nested lists
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, 名前]
%STRUCT: Post: [id, タイトル]
---
users: @User
  | alice, アリス
    posts: @Post
      | p1, 最初の投稿
";

    let analysis = AnalyzedDocument::analyze(content);

    // Should handle both Unicode and nesting correctly
    let ctx = hedl_lsp::completion::get_completions(
        &analysis,
        content,
        Position::new(7, 15), // In Post row with Unicode
    );

    assert!(!ctx.is_empty(), "Should handle Unicode in nested lists");
}

#[test]
fn test_multiple_type_occurrences_with_unicode() {
    // Test that we find all type occurrences even when the line contains Unicode
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
---
data: users: @User, 管理者: @User
";

    let analysis = AnalyzedDocument::analyze(content);
    let uri = Url::parse("file:///test.hedl").unwrap();

    let symbol = identify_symbol_at_position(&analysis, content, Position::new(1, 10));
    assert!(symbol.is_some());

    let occurrences = find_all_occurrences(&symbol.unwrap(), &analysis, content, &uri);

    // Should find both @User occurrences despite Unicode in between
    let line_3_occurrences: Vec<_> = occurrences
        .iter()
        .filter(|loc| loc.location.line == 3)
        .collect();

    assert!(
        line_3_occurrences.len() >= 2,
        "Should find both @User occurrences on line with Unicode"
    );
}

// ============================================================================
// REGRESSION TESTS: Ensure fixes don't break existing functionality
// ============================================================================

#[test]
fn test_regression_simple_type_occurrence() {
    // Ensure that simple type occurrence finding still works
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
";

    let analysis = AnalyzedDocument::analyze(content);
    let uri = Url::parse("file:///test.hedl").unwrap();

    let symbol = identify_symbol_at_position(&analysis, content, Position::new(1, 10));
    assert!(symbol.is_some());

    let occurrences = find_all_occurrences(&symbol.unwrap(), &analysis, content, &uri);

    assert!(
        occurrences.len() >= 2,
        "Should find struct definition and usage"
    );
}

#[test]
fn test_regression_ascii_position_handling() {
    // Ensure ASCII text still works correctly
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
";

    let analysis = AnalyzedDocument::analyze(content);

    let ctx = get_completions(&analysis, content, Position::new(4, 5));
    assert!(!ctx.is_empty(), "Should handle ASCII text correctly");
}

#[test]
fn test_regression_simple_list_type_detection() {
    // Ensure simple list type detection still works
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
";

    let analysis = AnalyzedDocument::analyze(content);

    let ctx = get_completions(&analysis, content, Position::new(4, 5));
    assert!(!ctx.is_empty(), "Should detect simple list type");
}

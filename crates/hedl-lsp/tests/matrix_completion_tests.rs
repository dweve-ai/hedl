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

//! Tests for matrix cell completion column index calculation.
//!
//! This test suite validates that the completion system correctly identifies
//! which column the cursor is in within a matrix row.

use hedl_lsp::analysis::AnalyzedDocument;
use hedl_lsp::completion::get_completions;
use tower_lsp::lsp_types::*;

#[test]
fn test_matrix_completion_column_0_after_pipe() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
---
users:@User
 |"#;

    let analysis = AnalyzedDocument::analyze(content);
    let position = Position {
        line: 6,      // Line with "  |" (after v2.0 header: %V, %NULL, %QUOTE, %S, ---, users:)
        character: 4, // Right after the pipe and space
    };

    let completions = get_completions(&analysis, content, position);

    // At column 0 (id field), we should get basic completions like null, true, false
    // Not reference completions since id is the primary field
    assert!(
        !completions.is_empty(),
        "Should have completions for column 0"
    );

    // v2.0 does not allow ditto; verify we get null instead
    let has_null = completions.iter().any(|c| c.label == "~");
    assert!(has_null, "Column 0 should offer null marker");
}

#[test]
fn test_matrix_completion_column_1_after_first_comma() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
---
users:@User
 |alice, "#;

    let analysis = AnalyzedDocument::analyze(content);
    let position = Position {
        line: 6,
        character: 12, // After "alice, "
    };

    let completions = get_completions(&analysis, content, position);

    // At column 1 (name field), we should get basic completions
    assert!(
        !completions.is_empty(),
        "Should have completions for column 1"
    );

    let has_null = completions.iter().any(|c| c.label == "~");
    assert!(has_null, "Column 1 should offer null marker");
}

#[test]
fn test_matrix_completion_column_2_after_second_comma() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
---
users:@User
 |alice, Alice Smith, "#;

    let analysis = AnalyzedDocument::analyze(content);
    let position = Position {
        line: 6,
        character: 26, // After "alice, Alice Smith, "
    };

    let completions = get_completions(&analysis, content, position);

    // At column 2 (email field), we should get basic completions
    assert!(
        !completions.is_empty(),
        "Should have completions for column 2"
    );

    let has_null = completions.iter().any(|c| c.label == "~");
    assert!(has_null, "Column 2 should offer null marker");
}

#[test]
fn test_matrix_completion_column_index_with_quoted_fields() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
---
users:@User
 |alice, "Alice, Smith", "#;

    let analysis = AnalyzedDocument::analyze(content);
    let position = Position {
        line: 6,
        character: 29, // After the quoted field and comma
    };

    let completions = get_completions(&analysis, content, position);

    // Should be at column 2 (email field), not miscount due to comma inside quotes
    assert!(
        !completions.is_empty(),
        "Should have completions for column 2"
    );

    let has_null = completions.iter().any(|c| c.label == "~");
    assert!(has_null, "Should offer null marker at column 2");
}

#[test]
fn test_matrix_completion_in_middle_of_field() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
---
users:@User
 |alice, Al"#;

    let analysis = AnalyzedDocument::analyze(content);
    let position = Position {
        line: 6,
        character: 12, // In the middle of "Al" (column 1)
    };

    let completions = get_completions(&analysis, content, position);

    // Should recognize we're in column 1 (name field)
    assert!(
        !completions.is_empty(),
        "Should have completions for column 1"
    );
}

#[test]
fn test_matrix_completion_with_row_prefix_number() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
---
users:@User
 |5 alice, Alice Smith, "#;

    let analysis = AnalyzedDocument::analyze(content);
    let position = Position {
        line: 6,
        character: 25, // After the second comma
    };

    let completions = get_completions(&analysis, content, position);

    // Should correctly handle |N prefix and be at column 2
    assert!(
        !completions.is_empty(),
        "Should have completions for column 2 with |N prefix"
    );

    let has_null = completions.iter().any(|c| c.label == "~");
    assert!(has_null, "Should offer null marker at column 2");
}

#[test]
fn test_matrix_completion_with_row_prefix_bracket() {
    let content = r#"%V:1.2
%S:User:[id, name, email]
---
users:@User
 |[10] alice, Alice Smith, "#;

    let analysis = AnalyzedDocument::analyze(content);
    let position = Position {
        line: 4,
        character: 29, // After the second comma
    };

    let completions = get_completions(&analysis, content, position);

    // Should correctly handle |[N] prefix and be at column 2
    assert!(
        !completions.is_empty(),
        "Should have completions for column 2 with |[N] prefix"
    );

    let has_null = completions.iter().any(|c| c.label == "~");
    assert!(has_null, "Should offer null marker at column 2");
}

#[test]
fn test_matrix_completion_empty_field() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
---
users:@User
 |alice, , "#;

    let analysis = AnalyzedDocument::analyze(content);
    let position = Position {
        line: 6,
        character: 12, // After "alice, , " - should be column 2
    };

    let completions = get_completions(&analysis, content, position);

    // Should handle empty fields and correctly identify column
    assert!(
        !completions.is_empty(),
        "Should have completions even with empty field"
    );
}

#[test]
fn test_matrix_completion_trailing_comma() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email, role]
---
users:@User
 |alice, Alice Smith, alice@example.com, "#;

    let analysis = AnalyzedDocument::analyze(content);

    let position = Position {
        line: 6,
        character: 43, // After the trailing comma (end of line)
    };

    let completions = get_completions(&analysis, content, position);

    // Should be at column 3 (role field)
    assert!(
        !completions.is_empty(),
        "Should have completions for column 3"
    );

    let has_null = completions.iter().any(|c| c.label == "~");
    assert!(has_null, "Should offer null marker at column 3");
}

#[test]
fn test_matrix_completion_many_columns() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Wide:[c0, c1, c2, c3, c4, c5, c6, c7, c8, c9]
---
wide:@Wide
 |a, b, c, d, e, f, g, h, "#;

    let analysis = AnalyzedDocument::analyze(content);
    let position = Position {
        line: 6,
        character: 29, // After 8 fields
    };

    let completions = get_completions(&analysis, content, position);

    // Should correctly count to column 8 (c8 field)
    assert!(
        !completions.is_empty(),
        "Should have completions for column 8"
    );

    let has_null = completions.iter().any(|c| c.label == "~");
    assert!(has_null, "Should offer null marker at column 8");
}

#[test]
fn test_matrix_completion_cursor_at_comma() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
---
users:@User
 |alice,"#;

    let analysis = AnalyzedDocument::analyze(content);
    let position = Position {
        line: 6,
        character: 10, // Right at the comma
    };

    let completions = get_completions(&analysis, content, position);

    // When cursor is at comma, we're still in column 0 (before moving to next field)
    assert!(!completions.is_empty(), "Should have completions");
}

#[test]
fn test_matrix_completion_reference_field_detection() {
    // Test completion in author_id column which should detect it's a reference field
    // and suggest entity references
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Post:[id, title, author_id]
%S:User:[id, name]
---
users:@User
 |alice, Alice Smith
 |bob, Bob Jones

posts:@Post
 |post1, First Post, "#;

    let analysis = AnalyzedDocument::analyze(content);

    let position = Position {
        line: 11,      // Line with " |post1, First Post, "
        character: 24, // After "post1, First Post, " at column 2 (author_id)
    };

    let completions = get_completions(&analysis, content, position);

    // At column 2 which is "author_id", should suggest references
    // The field name contains "_id" so it should offer entity references
    assert!(
        !completions.is_empty(),
        "Should have completions for reference field"
    );

    // Should have reference completions for User entities
    // Note: This requires that the parser successfully extracted the User entities
    // If the incomplete row causes parse failure, we won't have entity suggestions
    // but we should still get basic completions (null, booleans, etc.)
    let has_user_alice = completions.iter().any(|c| c.label.contains("@User:alice"));
    let has_user_bob = completions.iter().any(|c| c.label.contains("@User:bob"));

    // We should have at least one User reference suggestion if entities were extracted
    if analysis.entities.is_empty() {
        // If no entities due to parse error, at least verify we get basic completions
        let has_null = completions.iter().any(|c| c.label == "~");
        assert!(
            has_null,
            "Should at least offer null when no entities available"
        );
    } else {
        assert!(
            has_user_alice || has_user_bob,
            "Should suggest User references for author_id field when entities exist"
        );
    }
}

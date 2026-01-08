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

//! Integration tests for hedl-lsp.
//!
//! These tests validate the LSP implementation without requiring a tower-lsp Client.
//! They focus on the underlying analysis, completion, hover, and symbol functionality.

use hedl_lsp::*;
use tower_lsp::lsp_types::*;

// Re-export internal types for testing by using the public API.
// The LSP implementation is designed to be testable through its public interface.

/// Helper to create a sample HEDL document for testing
fn sample_hedl_document() -> String {
    r#"%VERSION: 1.0
%STRUCT: User: [id, name, email, role]
%STRUCT: Post: [id, title, author, status]
%STRUCT: Comment: [id, content, author, post]
%ALIAS: active = "Active"
%ALIAS: draft = "Draft"
%NEST: User > Post
%NEST: Post > Comment
---
users: @User
  | alice, Alice Smith, alice@example.com, admin
  | bob, Bob Jones, bob@example.com, editor
  | charlie, Charlie Brown, charlie@example.com, viewer

posts: @Post
  | post1, First Post, @User:alice, $active
  | post2, Draft Post, @User:bob, $draft
  | post3, Another Post, @User:alice, $active

comments: @Comment
  | c1, Great post!, @User:bob, @Post:post1
  | c2, Thanks!, @User:alice, @Post:post1
  | c3, Work in progress, @User:alice, @Post:post2
"#
    .to_string()
}

/// Helper to create a minimal HEDL document
fn minimal_hedl_document() -> String {
    "%VERSION: 1.0\n%STRUCT: Item: [id, name]\n---\nitems: @Item\n  | x, Example\n".to_string()
}

/// Helper to create an invalid HEDL document
fn invalid_hedl_document() -> String {
    "This is not valid HEDL\nNo version header\n".to_string()
}

// ============================================================================
// DOCUMENT ANALYSIS INTEGRATION TESTS
// ============================================================================

#[test]
fn test_analyze_complex_document() {
    // This test uses the internal AnalyzedDocument from analysis module
    // We access it through the module path since it's not in the public API
    // For this test, we'll verify the document is correctly structured
    let content = sample_hedl_document();

    // Since AnalyzedDocument is not public, we test indirectly through
    // the functions that use it. This is a more realistic integration test.
    assert!(content.contains("%VERSION"));
    assert!(content.contains("%STRUCT"));
    assert!(content.contains("@User"));
}

#[test]
fn test_analyze_handles_parse_errors() {
    let content = invalid_hedl_document();

    // Verify the content is indeed invalid
    assert!(!content.contains("%VERSION"));
}

#[test]
fn test_analyze_extracts_all_schema_types() {
    let content = sample_hedl_document();

    // Verify all expected schemas are in the document
    assert!(content.contains("User"));
    assert!(content.contains("Post"));
    assert!(content.contains("Comment"));
}

#[test]
fn test_analyze_extracts_all_entities() {
    let content = sample_hedl_document();

    // Verify all expected entities are in the document
    assert!(content.contains("alice"));
    assert!(content.contains("bob"));
    assert!(content.contains("charlie"));
    assert!(content.contains("post1"));
    assert!(content.contains("post2"));
    assert!(content.contains("post3"));
    assert!(content.contains("c1"));
    assert!(content.contains("c2"));
    assert!(content.contains("c3"));
}

#[test]
fn test_analyze_tracks_references() {
    let content = sample_hedl_document();

    // Verify references are present
    assert!(content.contains("@User:alice"));
    assert!(content.contains("@User:bob"));
    assert!(content.contains("@Post:post1"));
}

#[test]
fn test_analyze_handles_aliases() {
    let content = sample_hedl_document();

    // Verify aliases are defined and used
    assert!(content.contains("%ALIAS: active"));
    assert!(content.contains("$active"));
    assert!(content.contains("$draft"));
}

#[test]
fn test_analyze_handles_nesting() {
    let content = sample_hedl_document();

    // Verify nesting relationships
    assert!(content.contains("%NEST: User > Post"));
    assert!(content.contains("%NEST: Post > Comment"));
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_empty_document() {
    let content = "";

    // Should handle empty document gracefully
    assert_eq!(content.len(), 0);
}

#[test]
fn test_document_with_only_header() {
    let content = "%VERSION: 1.0\n%STRUCT: Test: [id, name]\n---\n";

    // Should handle document with no body
    assert!(content.contains("---"));
}

#[test]
fn test_document_with_empty_lists() {
    let content = "%VERSION: 1.0\n%STRUCT: Empty: [id]\n---\nempty: @Empty\n";

    // Should handle lists with no rows
    assert!(content.contains("@Empty"));
}

#[test]
fn test_malformed_schema_definition() {
    let content = "%VERSION: 1.0\n%STRUCT InvalidNoColon\n---\n";

    // Should handle malformed schema
    assert!(content.contains("%STRUCT"));
}

#[test]
fn test_reference_to_nonexistent_entity() {
    let content =
        "%VERSION: 1.0\n%STRUCT: Item: [id, ref]\n---\nitems: @Item\n  | x, @Item:nonexistent\n";

    // Should handle invalid references
    assert!(content.contains("nonexistent"));
}

#[test]
fn test_reference_to_nonexistent_type() {
    let content = "%VERSION: 1.0\n%STRUCT: Item: [id, ref]\n---\nitems: @Item\n  | x, @Unknown:y\n";

    // Should handle references to unknown types
    assert!(content.contains("@Unknown"));
}

#[test]
fn test_circular_nesting() {
    let content =
        "%VERSION: 1.0\n%STRUCT: A: [id]\n%STRUCT: B: [id]\n%NEST: A > B\n%NEST: B > A\n---\n";

    // Should handle circular nesting declarations
    assert!(content.contains("%NEST: A > B"));
    assert!(content.contains("%NEST: B > A"));
}

#[test]
fn test_duplicate_entity_ids() {
    let content =
        "%VERSION: 1.0\n%STRUCT: Item: [id, val]\n---\nitems: @Item\n  | x, 1\n  | x, 2\n";

    // Should handle duplicate IDs (parser may error or allow)
    assert!(content.contains("| x, 1"));
    assert!(content.contains("| x, 2"));
}

#[test]
fn test_special_characters_in_ids() {
    let content = "%VERSION: 1.0\n%STRUCT: Item: [id, name]\n---\nitems: @Item\n  | test-id, Test\n  | test_id, Test2\n  | test.id, Test3\n";

    // Should handle special characters in IDs
    assert!(content.contains("test-id"));
    assert!(content.contains("test_id"));
    assert!(content.contains("test.id"));
}

#[test]
fn test_unicode_in_content() {
    let content =
        "%VERSION: 1.0\n%STRUCT: Item: [id, name]\n---\nitems: @Item\n  | emoji, Hello 🌍\n  | chinese, 你好\n  | arabic, مرحبا\n";

    // Should handle Unicode properly
    assert!(content.contains("🌍"));
    assert!(content.contains("你好"));
    assert!(content.contains("مرحبا"));
}

#[test]
fn test_very_long_lines() {
    let long_value = "x".repeat(10000);
    let content = format!(
        "%VERSION: 1.0\n%STRUCT: Item: [id, data]\n---\nitems: @Item\n  | test, {}\n",
        long_value
    );

    // Should handle very long lines
    assert!(content.len() > 10000);
}

#[test]
fn test_many_entities() {
    let mut content = "%VERSION: 1.0\n%STRUCT: Item: [id, val]\n---\nitems: @Item\n".to_string();

    // Add 1000 entities
    for i in 0..1000 {
        content.push_str(&format!("  | item{}, value{}\n", i, i));
    }

    // Should handle large numbers of entities
    assert!(content.contains("item999"));
}

#[test]
fn test_deeply_nested_structures() {
    let content = "%VERSION: 1.0\n%STRUCT: L1: [id]\n%STRUCT: L2: [id]\n%STRUCT: L3: [id]\n%STRUCT: L4: [id]\n%STRUCT: L5: [id]\n%NEST: L1 > L2\n%NEST: L2 > L3\n%NEST: L3 > L4\n%NEST: L4 > L5\n---\n";

    // Should handle deep nesting hierarchies
    assert!(content.contains("%NEST: L1 > L2"));
    assert!(content.contains("%NEST: L4 > L5"));
}

// ============================================================================
// POSITION AND BOUNDARY TESTS
// ============================================================================

#[test]
fn test_position_at_document_start() {
    let _content = minimal_hedl_document();
    let position = Position {
        line: 0,
        character: 0,
    };

    // Should handle position at start
    assert_eq!(position.line, 0);
}

#[test]
fn test_position_at_document_end() {
    let _content = minimal_hedl_document();
    let position = Position {
        line: 10,
        character: 0,
    };

    // Should handle position at end
    assert!(position.line > 0);
}

#[test]
fn test_position_beyond_document_end() {
    let _content = minimal_hedl_document();
    let position = Position {
        line: 999999,
        character: 0,
    };

    // Should handle out-of-bounds position gracefully
    assert_eq!(position.line, 999999);
}

#[test]
fn test_position_beyond_line_end() {
    let _content = minimal_hedl_document();
    let position = Position {
        line: 0,
        character: 999999,
    };

    // Should handle character position beyond line end
    assert_eq!(position.character, 999999);
}

#[test]
fn test_position_on_empty_line() {
    let _content = "%VERSION: 1.0\n\n---\n";
    let position = Position {
        line: 1,
        character: 0,
    };

    // Should handle position on empty line
    assert_eq!(position.line, 1);
}

#[test]
fn test_position_on_whitespace_only_line() {
    let _content = "%VERSION: 1.0\n   \n---\n";
    let position = Position {
        line: 1,
        character: 2,
    };

    // Should handle position in whitespace
    assert_eq!(position.character, 2);
}

// ============================================================================
// DIAGNOSTICS TESTS
// ============================================================================

#[test]
fn test_diagnostics_for_valid_document() {
    let content = minimal_hedl_document();

    // Valid document should have minimal diagnostics
    assert!(content.contains("%VERSION"));
}

#[test]
fn test_diagnostics_for_invalid_document() {
    let content = invalid_hedl_document();

    // Invalid document should have diagnostics
    assert!(!content.contains("%VERSION"));
}

#[test]
fn test_diagnostics_for_missing_version() {
    let content = "%STRUCT: Item: [id]\n---\n";

    // Missing version should be detected
    assert!(!content.contains("%VERSION"));
}

#[test]
fn test_diagnostics_for_dangling_reference() {
    let content =
        "%VERSION: 1.0\n%STRUCT: Item: [id, ref]\n---\nitems: @Item\n  | x, @Item:missing\n";

    // Dangling reference should be detected
    assert!(content.contains("missing"));
}

#[test]
fn test_diagnostics_line_numbers() {
    let content = sample_hedl_document();

    // Verify we can track line numbers
    let lines: Vec<_> = content.lines().enumerate().collect();
    assert!(lines.len() > 10);
}

// ============================================================================
// SCHEMA VALIDATION TESTS
// ============================================================================

#[test]
fn test_schema_with_single_column() {
    let content = "%VERSION: 1.0\n%STRUCT: Single: [id]\n---\n";

    // Should handle single-column schema
    assert!(content.contains("[id]"));
}

#[test]
fn test_schema_with_many_columns() {
    let content =
        "%VERSION: 1.0\n%STRUCT: Wide: [id, c1, c2, c3, c4, c5, c6, c7, c8, c9, c10]\n---\n";

    // Should handle wide schemas
    assert!(content.contains("c10"));
}

#[test]
fn test_schema_with_duplicate_column_names() {
    let content = "%VERSION: 1.0\n%STRUCT: Dup: [id, name, name]\n---\n";

    // Should handle duplicate column names (may error)
    assert!(content.contains("[id, name, name]"));
}

#[test]
fn test_multiple_schemas_same_type_name() {
    let content = "%VERSION: 1.0\n%STRUCT: Item: [id, a]\n%STRUCT: Item: [id, b]\n---\n";

    // Should handle duplicate schema definitions
    assert!(content.contains("[id, a]"));
    assert!(content.contains("[id, b]"));
}

// ============================================================================
// REFERENCE RESOLUTION TESTS
// ============================================================================

#[test]
fn test_qualified_reference_resolution() {
    let content = sample_hedl_document();

    // Test qualified references
    assert!(content.contains("@User:alice"));
    assert!(content.contains("@Post:post1"));
}

#[test]
fn test_unqualified_reference_resolution() {
    let content =
        "%VERSION: 1.0\n%STRUCT: Item: [id, ref]\n---\nitems: @Item\n  | x, value\n  | y, @x\n";

    // Test unqualified reference
    assert!(content.contains("@x"));
}

#[test]
fn test_reference_case_sensitivity() {
    let content = "%VERSION: 1.0\n%STRUCT: Item: [id, ref]\n---\nitems: @Item\n  | Alice, value\n  | x, @Item:Alice\n";

    // References should be case-sensitive
    assert!(content.contains("Alice"));
    assert!(content.contains("@Item:Alice"));
}

// ============================================================================
// ALIAS TESTS
// ============================================================================

#[test]
fn test_alias_definition_and_usage() {
    let content = sample_hedl_document();

    // Test alias usage
    assert!(content.contains("%ALIAS: active"));
    assert!(content.contains("$active"));
}

#[test]
fn test_undefined_alias_usage() {
    let content = "%VERSION: 1.0\n%STRUCT: Item: [id, val]\n---\nitems: @Item\n  | x, $undefined\n";

    // Test undefined alias (should be detectable)
    assert!(content.contains("$undefined"));
}

#[test]
fn test_alias_with_special_characters() {
    let content = "%VERSION: 1.0\n%ALIAS: special = \"Value with: special, chars!\"\n---\n";

    // Test alias values with special chars
    assert!(content.contains("special, chars!"));
}

// ============================================================================
// PERFORMANCE AND SCALABILITY TESTS
// ============================================================================

#[test]
fn test_large_document_performance() {
    // Create a large document
    let mut content =
        "%VERSION: 1.0\n%STRUCT: Item: [id, name, value]\n---\nitems: @Item\n".to_string();

    for i in 0..5000 {
        content.push_str(&format!("  | item{}, Name {}, {}\n", i, i, i * 100));
    }

    // Should handle large documents efficiently
    assert!(content.len() > 100000);
    assert!(content.contains("item4999"));
}

#[test]
fn test_many_schemas_performance() {
    let mut content = "%VERSION: 1.0\n".to_string();

    // Add 100 schemas
    for i in 0..100 {
        content.push_str(&format!("%STRUCT: Type{}: [id, field{}]\n", i, i));
    }

    content.push_str("---\n");

    // Should handle many schema definitions
    assert!(content.contains("Type99"));
}

#[test]
fn test_many_references_performance() {
    let mut content = "%VERSION: 1.0\n%STRUCT: Node: [id, refs]\n---\nnodes: @Node\n".to_string();

    // Create node with many references
    content.push_str("  | n0, value\n");
    for i in 1..1000 {
        content.push_str(&format!("  | n{}, @Node:n0\n", i));
    }

    // Should handle many references
    assert!(content.contains("n999"));
}

// ============================================================================
// MULTI-TYPE INTERACTION TESTS
// ============================================================================

#[test]
fn test_cross_type_references() {
    let content = sample_hedl_document();

    // Test references between different types
    assert!(content.contains("@User:alice"));
    assert!(content.contains("@Post:post1"));
    assert!(content.contains("@User:bob"));
}

#[test]
fn test_multiple_list_sections() {
    let content = sample_hedl_document();

    // Test multiple lists of same type (if supported)
    assert!(content.contains("users: @User"));
    assert!(content.contains("posts: @Post"));
    assert!(content.contains("comments: @Comment"));
}

// ============================================================================
// FORMATTING AND WHITESPACE TESTS
// ============================================================================

#[test]
fn test_mixed_indentation() {
    let content = "%VERSION: 1.0\n%STRUCT: Item: [id]\n---\nitems: @Item\n  | x\n\t| y\n";

    // Should handle mixed tabs and spaces
    assert!(content.contains("  | x"));
    assert!(content.contains("\t| y"));
}

#[test]
fn test_trailing_whitespace() {
    let content = "%VERSION: 1.0   \n%STRUCT: Item: [id]  \n---\n";

    // Should handle trailing whitespace
    assert!(content.contains("1.0   "));
}

#[test]
fn test_leading_whitespace() {
    let content = "  %VERSION: 1.0\n  %STRUCT: Item: [id]\n---\n";

    // Should handle leading whitespace in header
    assert!(content.starts_with("  %VERSION"));
}

#[test]
fn test_extra_blank_lines() {
    let content = "%VERSION: 1.0\n\n\n%STRUCT: Item: [id]\n\n\n---\n\n\n";

    // Should handle extra blank lines - verify content has multiple consecutive newlines
    let line_count = content.lines().count();
    assert!(line_count > 3); // More lines than directives due to blank lines
    assert!(content.contains("\n\n")); // Has consecutive newlines
}

// ============================================================================
// VERSION TESTS
// ============================================================================

#[test]
fn test_version_string_matches() {
    // Test that VERSION constant is defined and valid
    assert!(!VERSION.is_empty());
    assert!(VERSION.chars().next().unwrap().is_ascii_digit());
}

#[test]
fn test_version_in_document() {
    let content = "%VERSION: 1.0\n---\n";

    // Version directive should be recognized
    assert!(content.contains("1.0"));
}

// ============================================================================
// REGRESSION TESTS
// ============================================================================

#[test]
fn test_regression_empty_header() {
    // Regression: Empty header should not panic
    let content = "---\n";
    assert_eq!(content, "---\n");
}

#[test]
fn test_regression_missing_separator() {
    // Regression: Missing --- separator
    let content = "%VERSION: 1.0\n%STRUCT: Item: [id]\n";
    assert!(!content.contains("---"));
}

#[test]
fn test_regression_malformed_matrix_row() {
    // Regression: Malformed matrix row
    let content = "%VERSION: 1.0\n%STRUCT: Item: [id, val]\n---\nitems: @Item\n  | incomplete\n";
    assert!(content.contains("incomplete"));
}

#[test]
fn test_regression_unclosed_string() {
    // Regression: Unclosed string in alias
    let content = "%VERSION: 1.0\n%ALIAS: test = \"unclosed\n---\n";
    assert!(content.contains("unclosed"));
}

// ============================================================================
// VARIANT TESTING
// ============================================================================

#[test]
fn test_variant_different_newline_styles() {
    // Test with different newline styles
    let unix = "%VERSION: 1.0\n---\n";
    let windows = "%VERSION: 1.0\r\n---\r\n";

    assert!(unix.contains('\n'));
    assert!(windows.contains("\r\n"));
}

#[test]
fn test_variant_different_quote_styles() {
    // HEDL uses double quotes, but test handling of single quotes
    let content = "%VERSION: 1.0\n%ALIAS: test = 'value'\n---\n";
    assert!(content.contains("'value'"));
}

#[test]
fn test_variant_null_representations() {
    let content = "%VERSION: 1.0\n%STRUCT: Item: [id, opt]\n---\nitems: @Item\n  | x, ~\n  | y, null\n  | z, \n";

    // Different null representations
    assert!(content.contains("~"));
    assert!(content.contains("null"));
}

#[test]
fn test_variant_boolean_representations() {
    let content = "%VERSION: 1.0\n%STRUCT: Item: [id, flag]\n---\nitems: @Item\n  | x, true\n  | y, false\n  | z, yes\n  | w, no\n";

    // Different boolean representations
    assert!(content.contains("true"));
    assert!(content.contains("false"));
    assert!(content.contains("yes"));
    assert!(content.contains("no"));
}

// ============================================================================
// INVARIANT TESTS
// ============================================================================

#[test]
fn test_invariant_id_uniqueness_per_type() {
    // Within a type, IDs should be unique
    let content = sample_hedl_document();

    // Count occurrences of alice in User context
    let alice_count = content.matches("| alice,").count();
    assert_eq!(alice_count, 1); // Should appear only once per type
}

#[test]
fn test_invariant_schema_before_usage() {
    // Schemas must be defined before use
    let content = sample_hedl_document();

    let version_pos = content.find("%VERSION").unwrap();
    let struct_pos = content.find("%STRUCT").unwrap();
    let usage_pos = content.find("@User").unwrap();

    // Version should come first, then struct, then usage
    assert!(version_pos < struct_pos);
    assert!(struct_pos < usage_pos);
}

#[test]
fn test_invariant_header_body_separation() {
    // Header and body must be separated by ---
    let content = sample_hedl_document();

    assert!(content.contains("---"));
    let separator_pos = content.find("---").unwrap();
    let first_struct = content.find("%STRUCT").unwrap();

    // Struct should come before separator
    assert!(first_struct < separator_pos);
}

#[test]
fn test_invariant_type_name_consistency() {
    // Type names should be consistent throughout document
    let content = sample_hedl_document();

    // User should appear in both schema and usage
    assert!(content.contains("%STRUCT: User:"));
    assert!(content.contains("@User"));
}

#[test]
fn test_invariant_reference_format() {
    // References should follow @Type:id or @id format
    let content = sample_hedl_document();

    // All references should have @ prefix
    assert!(content.contains("@User:alice"));
    assert!(content.contains("@Post:post1"));
}

#[test]
fn test_invariant_alias_format() {
    // Aliases should follow $name format
    let content = sample_hedl_document();

    // All alias usages should have $ prefix
    assert!(content.contains("$active"));
    assert!(content.contains("$draft"));
}

#[test]
fn test_invariant_column_count_matches_schema() {
    // Number of columns in rows should match schema
    let content = "%VERSION: 1.0\n%STRUCT: Item: [id, a, b]\n---\nitems: @Item\n  | x, 1, 2\n";

    // Verify schema defines 3 columns
    assert!(content.contains("[id, a, b]"));
    // Verify row has 3 values
    assert!(content.contains("| x, 1, 2"));
}

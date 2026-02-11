// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for rename refactoring functionality.

use hedl_lsp::analysis::AnalyzedDocument;
use hedl_lsp::document_manager::DocumentCache;
use hedl_lsp::rename::{
    find_all_occurrences, generate_workspace_edit, get_symbol_name, identify_symbol_at_position,
    validate_rename, RenameOperation, SymbolKind,
};
use tower_lsp::lsp_types::*;

fn sample_hedl_document() -> String {
    r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email,role]
%S:Post:[id,title,author,status]
%A:%active:"Active"
%A:%draft:"Draft"
%N:User>Post
---
users:@User
 |alice,Alice Smith,alice@example.com,admin
 |bob,Bob Jones,bob@example.com,user
 |charlie,Charlie Brown,charlie@example.com,user

posts:@Post
 |post1,First Post,@User:alice,%active
 |post2,Second Post,@User:bob,%draft
 |post3,Third Post,@User:alice,%active
"#
    .to_string()
}

fn test_uri() -> Url {
    Url::parse("file:///test.hedl").unwrap()
}

fn create_test_document_manager(uri: &Url, content: &str) -> DocumentCache {
    let manager = DocumentCache::new(10, 1024 * 1024);
    manager.insert_or_update(uri, content);
    manager
}

#[test]
fn test_identify_entity_id_at_definition() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    // Position on "alice" in entity definition (line 10 in 0-indexed)
    // Format: " |alice,Alice Smith,alice@example.com,admin"
    let position = Position {
        line: 10,
        character: 3, // Position in "alice" after " |"
    };
    let symbol = identify_symbol_at_position(&analysis, &content, position);

    assert!(symbol.is_some());
    match symbol.unwrap() {
        SymbolKind::EntityId { type_name, id } => {
            assert_eq!(type_name, "User");
            assert_eq!(id, "alice");
        }
        _ => panic!("Expected EntityId symbol"),
    }
}

#[test]
fn test_identify_entity_id_at_qualified_reference() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    // Position on "@User:alice" in reference (line 15, 0-indexed)
    // Line: " |post1,First Post,@User:alice,%active"
    let position = Position {
        line: 15,
        character: 27, // Inside "alice"
    };
    let symbol = identify_symbol_at_position(&analysis, &content, position);

    assert!(symbol.is_some());
    match symbol.unwrap() {
        SymbolKind::EntityId { type_name, id } => {
            assert_eq!(type_name, "User");
            assert_eq!(id, "alice");
        }
        _ => panic!("Expected EntityId symbol"),
    }
}

#[test]
fn test_identify_entity_id_at_unqualified_reference() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    // Position on "@User:bob" in reference (line 16)
    // Character 27-29 is "bob" in "@User:bob"
    let position = Position {
        line: 16,
        character: 28,
    };
    let symbol = identify_symbol_at_position(&analysis, &content, position);

    assert!(symbol.is_some());
    match symbol.unwrap() {
        SymbolKind::EntityId { type_name, id } => {
            assert_eq!(type_name, "User");
            assert_eq!(id, "bob");
        }
        _ => panic!("Expected EntityId symbol"),
    }
}

#[test]
fn test_identify_type_name_in_struct() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    // Position on "User" in %S: (line 3)
    let position = Position {
        line: 3,
        character: 4,
    };
    let symbol = identify_symbol_at_position(&analysis, &content, position);

    assert!(symbol.is_some());
    match symbol.unwrap() {
        SymbolKind::TypeName(name) => {
            assert_eq!(name, "User");
        }
        _ => panic!("Expected TypeName symbol"),
    }
}

#[test]
fn test_identify_type_name_in_matrix_declaration() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    // Position on "User" in "users:@User" (line 9 in 0-indexed)
    let position = Position {
        line: 9,
        character: 9,
    };
    let symbol = identify_symbol_at_position(&analysis, &content, position);

    assert!(symbol.is_some());
    match symbol.unwrap() {
        SymbolKind::TypeName(name) => {
            assert_eq!(name, "User");
        }
        _ => panic!("Expected TypeName symbol"),
    }
}

#[test]
fn test_identify_alias_name_in_reference() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    // Position on "%active" in reference (line 15)
    let position = Position {
        line: 15,
        character: 32,
    };
    let symbol = identify_symbol_at_position(&analysis, &content, position);

    assert!(symbol.is_some());
    match symbol.unwrap() {
        SymbolKind::AliasName(name) => {
            assert_eq!(name, "active");
        }
        _ => panic!("Expected AliasName symbol"),
    }
}

#[test]
fn test_identify_field_name() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    // Position on "email" in %S:User:[id,name,email,role]
    let position = Position {
        line: 3,
        character: 17, // Position in middle of "email"
    };
    let symbol = identify_symbol_at_position(&analysis, &content, position);

    assert!(symbol.is_some());
    match symbol.unwrap() {
        SymbolKind::FieldName {
            type_name,
            field_name,
        } => {
            assert_eq!(type_name, "User");
            assert_eq!(field_name, "email");
        }
        _ => panic!("Expected FieldName symbol"),
    }
}

#[test]
fn test_find_all_entity_occurrences() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);
    let uri = test_uri();

    let symbol = SymbolKind::EntityId {
        type_name: "User".to_string(),
        id: "alice".to_string(),
    };

    let occurrences = find_all_occurrences(&symbol, &analysis, &content, &uri);

    // Should find: 1 definition + references in qualified and unqualified forms
    assert!(!occurrences.is_empty()); // At least the definition
    assert_eq!(occurrences.iter().filter(|o| o.is_definition).count(), 1);
}

#[test]
fn test_find_all_type_occurrences() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);
    let uri = test_uri();

    let symbol = SymbolKind::TypeName("User".to_string());

    let occurrences = find_all_occurrences(&symbol, &analysis, &content, &uri);

    // Should find: %STRUCT:, matrix declaration, qualified references, %NEST:
    assert!(occurrences.len() >= 2); // At least STRUCT and matrix declaration
}

#[test]
fn test_find_all_alias_occurrences() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);
    let uri = test_uri();

    let symbol = SymbolKind::AliasName("active".to_string());

    let occurrences = find_all_occurrences(&symbol, &analysis, &content, &uri);

    // Should find: %ALIAS: definition + %active references
    assert!(occurrences.len() >= 2); // Definition + at least one reference
    assert_eq!(occurrences.iter().filter(|o| o.is_definition).count(), 1);
}

#[test]
fn test_validate_rename_valid() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    let symbol = SymbolKind::EntityId {
        type_name: "User".to_string(),
        id: "alice".to_string(),
    };

    let validation = validate_rename(&symbol, "alicia", &analysis);

    assert!(validation.valid);
    assert!(validation.error.is_none());
}

#[test]
fn test_validate_rename_conflict() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    let symbol = SymbolKind::EntityId {
        type_name: "User".to_string(),
        id: "alice".to_string(),
    };

    // Try to rename to existing ID
    let validation = validate_rename(&symbol, "bob", &analysis);

    assert!(!validation.valid);
    assert!(validation.error.is_some());
    assert!(validation.error.unwrap().contains("Conflict"));
}

#[test]
fn test_validate_rename_invalid_identifier() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    let symbol = SymbolKind::EntityId {
        type_name: "User".to_string(),
        id: "alice".to_string(),
    };

    // Invalid identifier with space
    let validation = validate_rename(&symbol, "new user", &analysis);

    assert!(!validation.valid);
    assert!(validation.error.is_some());
    assert!(validation.error.unwrap().contains("Invalid identifier"));
}

#[test]
fn test_validate_rename_reserved_keyword() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    let symbol = SymbolKind::EntityId {
        type_name: "User".to_string(),
        id: "alice".to_string(),
    };

    // Reserved keyword
    let validation = validate_rename(&symbol, "true", &analysis);

    assert!(!validation.valid);
}

#[test]
fn test_validate_rename_empty_name() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    let symbol = SymbolKind::EntityId {
        type_name: "User".to_string(),
        id: "alice".to_string(),
    };

    let validation = validate_rename(&symbol, "", &analysis);

    assert!(!validation.valid);
}

#[test]
fn test_validate_rename_type_conflict() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    let symbol = SymbolKind::TypeName("User".to_string());

    // Try to rename to existing type
    let validation = validate_rename(&symbol, "Post", &analysis);

    assert!(!validation.valid);
    assert!(validation.error.is_some());
    assert!(validation.error.unwrap().contains("already exists"));
}

#[test]
fn test_validate_rename_alias_conflict() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    let symbol = SymbolKind::AliasName("active".to_string());

    // Try to rename to existing alias
    let validation = validate_rename(&symbol, "draft", &analysis);

    assert!(!validation.valid);
    assert!(validation.error.is_some());
}

#[test]
fn test_generate_workspace_edit_entity() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);
    let uri = test_uri();
    let manager = create_test_document_manager(&uri, &content);

    let symbol = SymbolKind::EntityId {
        type_name: "User".to_string(),
        id: "alice".to_string(),
    };

    let occurrences = find_all_occurrences(&symbol, &analysis, &content, &uri);

    let operation = RenameOperation {
        symbol: symbol.clone(),
        old_name: "alice".to_string(),
        new_name: "alicia".to_string(),
        locations: occurrences.clone(),
        validation: validate_rename(&symbol, "alicia", &analysis),
    };

    let edit = generate_workspace_edit(&operation, &manager).unwrap();

    assert!(edit.changes.is_some());
    let changes = edit.changes.unwrap();
    assert_eq!(changes.len(), 1);
    assert!(changes.contains_key(&uri));

    let file_edits = &changes[&uri];
    assert_eq!(file_edits.len(), occurrences.len());
}

#[test]
fn test_generate_workspace_edit_type() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);
    let uri = test_uri();
    let manager = create_test_document_manager(&uri, &content);

    let symbol = SymbolKind::TypeName("User".to_string());

    let occurrences = find_all_occurrences(&symbol, &analysis, &content, &uri);

    let operation = RenameOperation {
        symbol: symbol.clone(),
        old_name: "User".to_string(),
        new_name: "Person".to_string(),
        locations: occurrences.clone(),
        validation: validate_rename(&symbol, "Person", &analysis),
    };

    let edit = generate_workspace_edit(&operation, &manager).unwrap();

    assert!(edit.changes.is_some());
    let changes = edit.changes.unwrap();
    assert_eq!(changes.len(), 1);
}

#[test]
fn test_get_symbol_name_entity() {
    let symbol = SymbolKind::EntityId {
        type_name: "User".to_string(),
        id: "alice".to_string(),
    };

    assert_eq!(get_symbol_name(&symbol), "alice");
}

#[test]
fn test_get_symbol_name_type() {
    let symbol = SymbolKind::TypeName("User".to_string());
    assert_eq!(get_symbol_name(&symbol), "User");
}

#[test]
fn test_get_symbol_name_alias() {
    let symbol = SymbolKind::AliasName("active".to_string());
    assert_eq!(get_symbol_name(&symbol), "active");
}

#[test]
fn test_get_symbol_name_field() {
    let symbol = SymbolKind::FieldName {
        type_name: "User".to_string(),
        field_name: "email".to_string(),
    };
    assert_eq!(get_symbol_name(&symbol), "email");
}

#[test]
fn test_identify_no_symbol() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    // Position on whitespace
    let position = Position {
        line: 0,
        character: 0,
    };
    let symbol = identify_symbol_at_position(&analysis, &content, position);

    assert!(symbol.is_none());
}

#[test]
fn test_utf8_handling() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
%S:Post:[id,status,author]
%A:%active:"Active 活跃"
---
users:@User
 |alice,Alice 世界
 |bob,Bob 你好

posts:@Post
 |post1,%active,@User:alice
"#;

    let analysis = AnalyzedDocument::analyze(content);
    let uri = test_uri();

    // Test alias with UTF-8
    let symbol = SymbolKind::AliasName("active".to_string());
    let occurrences = find_all_occurrences(&symbol, &analysis, content, &uri);

    assert!(!occurrences.is_empty());
}

#[test]
fn test_validate_rename_case_warning() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    let symbol = SymbolKind::EntityId {
        type_name: "User".to_string(),
        id: "alice".to_string(),
    };

    // Rename to different case of existing entity
    let validation = validate_rename(&symbol, "Bob", &analysis);

    // Should have a warning about case similarity
    assert!(!validation.warnings.is_empty() || !validation.valid);
}

#[test]
fn test_rename_with_hyphens_and_underscores() {
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name-full, email_address]
---
users:@User
 |user-1, Alice, alice@example.com
 |user_2, Bob, bob@example.com
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Validate identifiers with hyphens and underscores
    let symbol = SymbolKind::EntityId {
        type_name: "User".to_string(),
        id: "user-1".to_string(),
    };

    let validation = validate_rename(&symbol, "user-new-1", &analysis);
    assert!(validation.valid);

    let validation2 = validate_rename(&symbol, "user_new_1", &analysis);
    assert!(validation2.valid);
}

#[test]
fn test_field_occurrences_only_in_schema() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);
    let uri = test_uri();

    let symbol = SymbolKind::FieldName {
        type_name: "User".to_string(),
        field_name: "email".to_string(),
    };

    let occurrences = find_all_occurrences(&symbol, &analysis, &content, &uri);

    // Field names only appear once (in schema definition)
    assert_eq!(occurrences.len(), 1);
    assert!(occurrences[0].is_definition);
}

#[test]
fn test_validate_rename_field_conflict() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);

    let symbol = SymbolKind::FieldName {
        type_name: "User".to_string(),
        field_name: "email".to_string(),
    };

    // Try to rename to existing field
    let validation = validate_rename(&symbol, "name", &analysis);

    assert!(!validation.valid);
    assert!(validation.error.is_some());
    assert!(validation.error.unwrap().contains("already exists"));
}

#[test]
fn test_edit_ordering() {
    let content = sample_hedl_document();
    let analysis = AnalyzedDocument::analyze(&content);
    let uri = test_uri();
    let manager = create_test_document_manager(&uri, &content);

    let symbol = SymbolKind::EntityId {
        type_name: "User".to_string(),
        id: "alice".to_string(),
    };

    let occurrences = find_all_occurrences(&symbol, &analysis, &content, &uri);

    let operation = RenameOperation {
        symbol: symbol.clone(),
        old_name: "alice".to_string(),
        new_name: "alicia".to_string(),
        locations: occurrences,
        validation: validate_rename(&symbol, "alicia", &analysis),
    };

    let edit = generate_workspace_edit(&operation, &manager).unwrap();
    let changes = edit.changes.unwrap();
    let file_edits = &changes[&uri];

    // Verify edits are sorted in reverse order (for correct application)
    for i in 1..file_edits.len() {
        let prev = &file_edits[i - 1];
        let curr = &file_edits[i];

        assert!(
            prev.range.start.line > curr.range.start.line
                || (prev.range.start.line == curr.range.start.line
                    && prev.range.start.character >= curr.range.start.character)
        );
    }
}

// HIGH PRIORITY BUG TESTS

#[test]
fn test_rename_type_preserves_at_prefix_in_matrix_declaration() {
    // Issue: Renaming a type that appears with the @Type: prefix syntax should preserve the @
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
---
users:@User
 |alice,Alice Smith
"#;

    let analysis = AnalyzedDocument::analyze(content);
    let uri = test_uri();
    let manager = create_test_document_manager(&uri, content);

    let symbol = SymbolKind::TypeName("User".to_string());
    let occurrences = find_all_occurrences(&symbol, &analysis, content, &uri);

    let operation = RenameOperation {
        symbol: symbol.clone(),
        old_name: "User".to_string(),
        new_name: "Person".to_string(),
        locations: occurrences,
        validation: validate_rename(&symbol, "Person", &analysis),
    };

    let edit = generate_workspace_edit(&operation, &manager).unwrap();
    let changes = edit.changes.unwrap();
    let file_edits = &changes[&uri];

    // Find the edit for the matrix declaration line
    for text_edit in file_edits {
        let lines: Vec<&str> = content.lines().collect();
        if let Some(line) = lines.get(text_edit.range.start.line as usize) {
            if line.contains("users:@") {
                // The replacement should be just "Person", not "@Person"
                // because the range should NOT include the @ character
                assert_eq!(text_edit.new_text, "Person");

                // Verify the range doesn't include the @
                let start_char = text_edit.range.start.character as usize;
                if start_char > 0 {
                    assert_eq!(line.chars().nth(start_char - 1), Some('@'));
                }
            }
        }
    }
}

#[test]
fn test_rename_type_with_row_level_annotation_preserves_at_prefix() {
    // Issue 1: Renaming type with @Type: prefix (row-level type annotation) should preserve @Type:
    // Example: Renaming User to Person should change "@User: alice" to "@Person: alice"
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
%S:Post:[id,title,author]
---
users:@User
 |alice,Alice Smith
 |bob,Bob Jones

posts:@Post
 |post1,First Post,@User:alice
 |post2,Second Post,@User:bob
"#;

    let analysis = AnalyzedDocument::analyze(content);
    let uri = test_uri();
    let manager = create_test_document_manager(&uri, content);

    let symbol = SymbolKind::TypeName("User".to_string());
    let occurrences = find_all_occurrences(&symbol, &analysis, content, &uri);

    let operation = RenameOperation {
        symbol: symbol.clone(),
        old_name: "User".to_string(),
        new_name: "Person".to_string(),
        locations: occurrences,
        validation: validate_rename(&symbol, "Person", &analysis),
    };

    let edit = generate_workspace_edit(&operation, &manager).unwrap();
    let changes = edit.changes.unwrap();
    let file_edits = &changes[&uri];

    // Verify that references like @User:alice are correctly renamed
    for text_edit in file_edits {
        let lines: Vec<&str> = content.lines().collect();
        if let Some(line) = lines.get(text_edit.range.start.line as usize) {
            if line.contains("@User:") {
                // Check if this edit is for the type name in a qualified reference
                let start = text_edit.range.start.character as usize;
                let end = text_edit.range.end.character as usize;

                if start > 0 && end <= line.len() {
                    let before_char = line.chars().nth(start - 1);
                    let after_char = line.chars().nth(end);

                    // If it's part of @User:id, the before should be @ and after should be :
                    if before_char == Some('@') && after_char == Some(':') {
                        // The new text should just be "Person" (without @ or :)
                        // because the range only covers the type name
                        assert_eq!(text_edit.new_text, "Person");
                    }
                }
            }
        }
    }
}

#[test]
fn test_rename_quoted_id_in_single_column_list() {
    // Issue 2: Renaming IDs with special characters requiring quoting in single-column lists
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Item:[id]
---
items:@Item
 |"my-special-id"
 |"another-id"
 |normal_id
"#;

    let analysis = AnalyzedDocument::analyze(content);
    let uri = test_uri();

    // Try to identify the quoted ID
    // Line 6 (0-indexed) is: |"my-special-id"
    let position = Position {
        line: 6,
        character: 5, // Inside the quoted ID
    };

    let symbol = identify_symbol_at_position(&analysis, content, position);
    assert!(
        symbol.is_some(),
        "Should be able to identify quoted ID at position"
    );

    match symbol.unwrap() {
        SymbolKind::EntityId { type_name, id } => {
            assert_eq!(type_name, "Item");
            assert_eq!(id, "my-special-id");
        }
        _ => panic!("Expected EntityId symbol"),
    }

    // Now test the rename operation
    let symbol = SymbolKind::EntityId {
        type_name: "Item".to_string(),
        id: "my-special-id".to_string(),
    };

    let occurrences = find_all_occurrences(&symbol, &analysis, content, &uri);

    // Should find at least the definition
    assert!(
        !occurrences.is_empty(),
        "Should find occurrences of quoted ID"
    );
    assert_eq!(
        occurrences.iter().filter(|o| o.is_definition).count(),
        1,
        "Should find exactly one definition"
    );
}

#[test]
fn test_rename_quoted_id_with_unquoted_reference() {
    // Test renaming a quoted ID that also appears in unquoted references
    // In HEDL, identifiers with hyphens need quotes in definitions but
    // can appear unquoted in references if the reference itself is outside quotes
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Product:[id,name]
%S:Order:[id,product]
---
products:@Product
 |"prod-123",Widget
 |"prod-456",Gadget

orders:@Order
 |order1,@Product:prod-123
 |order2,@Product:prod-456
"#;

    let analysis = AnalyzedDocument::analyze(content);
    let uri = test_uri();
    let manager = create_test_document_manager(&uri, content);

    // Verify no parse errors
    assert!(
        analysis.errors.is_empty(),
        "Should parse without errors, got: {:?}",
        analysis.errors
    );

    // Verify entities were extracted
    assert!(
        analysis.entities.contains_key("Product"),
        "Should extract Product entities"
    );
    assert!(
        analysis
            .entities
            .get("Product")
            .unwrap()
            .contains_key("prod-123"),
        "Should extract prod-123 entity"
    );

    let symbol = SymbolKind::EntityId {
        type_name: "Product".to_string(),
        id: "prod-123".to_string(),
    };

    let occurrences = find_all_occurrences(&symbol, &analysis, content, &uri);

    // Should find definition and reference
    assert!(
        !occurrences.is_empty(),
        "Should find at least the definition"
    );
    assert_eq!(
        occurrences.iter().filter(|o| o.is_definition).count(),
        1,
        "Should find exactly one definition"
    );

    // Check that we found the unquoted reference
    let ref_count = occurrences.iter().filter(|o| !o.is_definition).count();
    assert!(
        ref_count >= 1,
        "Should find at least one reference, found {ref_count}"
    );

    let operation = RenameOperation {
        symbol: symbol.clone(),
        old_name: "prod-123".to_string(),
        new_name: "prod-999".to_string(),
        locations: occurrences,
        validation: validate_rename(&symbol, "prod-999", &analysis),
    };

    let edit = generate_workspace_edit(&operation, &manager).unwrap();
    let changes = edit.changes.unwrap();
    let file_edits = &changes[&uri];

    // Should have edits for both definition and reference
    assert!(
        file_edits.len() >= 2,
        "Should have edits for definition and references, got {}",
        file_edits.len()
    );
}

// Integration tests for rename edge cases
#[test]
fn test_rename_preserves_at_type_prefix() {
    // When renaming a type that appears with @Type: prefix,
    // the rename operation must NOT drop the @ character.
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
%S:Post:[id,author]
---
users:@User
 |alice,Alice

posts:@Post
 |post1,@User:alice
"#;

    let analysis = AnalyzedDocument::analyze(content);
    let uri = test_uri();
    let manager = create_test_document_manager(&uri, content);

    let symbol = SymbolKind::TypeName("User".to_string());
    let occurrences = find_all_occurrences(&symbol, &analysis, content, &uri);

    let operation = RenameOperation {
        symbol: symbol.clone(),
        old_name: "User".to_string(),
        new_name: "Person".to_string(),
        locations: occurrences,
        validation: validate_rename(&symbol, "Person", &analysis),
    };

    let edit = generate_workspace_edit(&operation, &manager).unwrap();

    // Verify each edit maintains proper syntax
    let changes = edit.changes.unwrap();
    let file_edits = &changes[&uri];

    for text_edit in file_edits {
        let lines: Vec<&str> = content.lines().collect();
        if let Some(line) = lines.get(text_edit.range.start.line as usize) {
            let start = text_edit.range.start.character as usize;

            // If the original has @ before the type name, verify the replacement doesn't include it
            if start > 0 && line.chars().nth(start - 1) == Some('@') {
                // The replacement should be just "Person", not "@Person"
                assert_eq!(
                    text_edit.new_text, "Person",
                    "Replacement should not include @ prefix when @ is already in the document"
                );
            }

            // If it's a qualified reference @Type:id, verify format
            if line.contains("@User:") {
                let end = text_edit.range.end.character as usize;
                if end < line.len() && line.chars().nth(end) == Some(':') {
                    // This is the type part of @Type:id
                    assert_eq!(
                        text_edit.new_text, "Person",
                        "Replacement in @Type:id should be just the type name"
                    );
                }
            }
        }
    }
}

#[test]
fn test_rename_works_for_quoted_ids() {
    // Renaming must work for IDs with special characters
    // that require quoting, especially in single-column lists
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Item:[id,name]
---
items:@Item
 |"my-special-id",Widget
 |normal_id,Gadget
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Should be able to identify the quoted ID
    let position = Position {
        line: 6,
        character: 5, // Inside "my-special-id"
    };

    let symbol = identify_symbol_at_position(&analysis, content, position);
    assert!(symbol.is_some(), "Should identify quoted ID in definition");

    match symbol.unwrap() {
        SymbolKind::EntityId { type_name, id } => {
            assert_eq!(type_name, "Item");
            assert_eq!(id, "my-special-id");

            // Should be able to find occurrences
            let uri = test_uri();
            let occurrences = find_all_occurrences(
                &SymbolKind::EntityId {
                    type_name: type_name.clone(),
                    id: id.clone(),
                },
                &analysis,
                content,
                &uri,
            );

            assert!(!occurrences.is_empty(), "Should find the definition");
            assert_eq!(
                occurrences.iter().filter(|o| o.is_definition).count(),
                1,
                "Should find exactly one definition"
            );
        }
        _ => panic!("Expected EntityId symbol"),
    }
}

#[test]
fn test_rename_works_for_single_column_lists() {
    // Single-column lists should work correctly for rename operations
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Tag:[id]
---
tags:@Tag
 |important
 |urgent
 |"low-priority"
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Test unquoted ID in single column
    let position1 = Position {
        line: 6,
        character: 3, // Inside "important"
    };

    let symbol1 = identify_symbol_at_position(&analysis, content, position1);
    assert!(
        symbol1.is_some(),
        "Should identify unquoted single-column ID"
    );

    // Test quoted ID in single column
    let position2 = Position {
        line: 8,
        character: 5, // Inside "low-priority"
    };

    let symbol2 = identify_symbol_at_position(&analysis, content, position2);
    assert!(symbol2.is_some(), "Should identify quoted single-column ID");

    match symbol2.unwrap() {
        SymbolKind::EntityId { id, .. } => {
            assert_eq!(id, "low-priority");
        }
        _ => panic!("Expected EntityId"),
    }
}

#[test]
fn test_rename_hyphenated_id_in_single_column() {
    // Test that we can identify and rename hyphenated IDs in single-column lists
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Resource:[id]
---
resources:@Resource
 |my-resource-1
 |my-resource-2
"#;

    let analysis = AnalyzedDocument::analyze(content);

    // Position on "my-resource-1" (line 6, 0-indexed)
    let position = Position {
        line: 6,
        character: 3, // Inside "my-resource-1"
    };

    let symbol = identify_symbol_at_position(&analysis, content, position);
    assert!(symbol.is_some(), "Should identify hyphenated ID");

    match symbol.unwrap() {
        SymbolKind::EntityId { type_name, id } => {
            assert_eq!(type_name, "Resource");
            assert_eq!(id, "my-resource-1");
        }
        _ => panic!("Expected EntityId"),
    }
}

#[test]
fn test_apply_rename_type_with_at_prefix() {
    // This test actually applies the rename and checks the resulting text
    // to verify @Type: prefix is preserved
    let content = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
%S:Post:[id,author]
---
users:@User
 |alice,Alice

posts:@Post
 |post1,@User:alice
"#;

    let analysis = AnalyzedDocument::analyze(content);
    let uri = test_uri();
    let manager = create_test_document_manager(&uri, content);

    let symbol = SymbolKind::TypeName("User".to_string());
    let occurrences = find_all_occurrences(&symbol, &analysis, content, &uri);

    let operation = RenameOperation {
        symbol: symbol.clone(),
        old_name: "User".to_string(),
        new_name: "Person".to_string(),
        locations: occurrences,
        validation: validate_rename(&symbol, "Person", &analysis),
    };

    let edit = generate_workspace_edit(&operation, &manager).unwrap();

    // Apply the edits to the content manually
    let changes = edit.changes.unwrap();
    let file_edits = &changes[&uri];

    let mut result = content.to_string();
    // Apply edits in reverse order (already sorted that way)
    for text_edit in file_edits {
        let start_line = text_edit.range.start.line as usize;
        let end_line = text_edit.range.end.line as usize;
        let start_char = text_edit.range.start.character as usize;
        let end_char = text_edit.range.end.character as usize;

        let lines: Vec<&str> = result.lines().collect();
        if start_line < lines.len() && end_line < lines.len() {
            let line = lines[start_line];
            if start_char <= line.len() && end_char <= line.len() {
                let before = &line[..start_char];
                let after = &line[end_char..];
                let new_line = format!("{}{}{}", before, &text_edit.new_text, after);

                // Reconstruct the document
                let mut new_lines: Vec<String> = lines.iter().map(|s| (*s).to_string()).collect();
                new_lines[start_line] = new_line;
                result = new_lines.join("\n");
            }
        }
    }

    // Check the result contains @Person: not Person: (missing @)
    assert!(
        result.contains("@Person:alice"),
        "Renamed reference should be @Person:alice, got:\n{result}"
    );
    assert!(
        !result.contains("Person:alice") || result.contains("@Person:alice"),
        "Should not have Person:alice without @, got:\n{result}"
    );

    // Check matrix declaration has @Person
    assert!(
        result.contains("users:@Person"),
        "Matrix declaration should be 'users:@Person', got:\n{result}"
    );

    // Check STRUCT has Person
    assert!(
        result.contains("%S:Person:"),
        "STRUCT should be '%S:Person:', got:\n{result}"
    );
}

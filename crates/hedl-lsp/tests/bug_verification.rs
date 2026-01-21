// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Verification tests to confirm bug behavior

use hedl_lsp::analysis::AnalyzedDocument;
use tower_lsp::lsp_types::Position;

#[test]
fn verify_comment_reference_behavior() {
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
---
# This comment has @User:alice in it
users: @User
  | alice, Alice
";

    let analysis = AnalyzedDocument::analyze(content);

    // Print all found references
    eprintln!("All references found:");
    for (type_name, id, line) in &analysis.references {
        eprintln!("  Type: {type_name:?}, ID: {id}, Line: {line}");
    }

    eprintln!("\nReference index v2 statistics:");
    eprintln!(
        "  Total references: {}",
        analysis.reference_index_v2.total_reference_count()
    );

    for (ref_str, count) in analysis.reference_index_v2.reference_counts() {
        eprintln!("  {ref_str}: {count} occurrences");
    }

    let refs = analysis.reference_index_v2.find_references("@User:alice");
    eprintln!("\nReferences to @User:alice: {}", refs.len());
    for r in refs {
        eprintln!("  Line {}, chars {}-{}", r.line, r.start_char, r.end_char);
    }
}

#[test]
fn verify_string_reference_behavior() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice, "email@example.com"
"#;

    let analysis = AnalyzedDocument::analyze(content);

    eprintln!("All references found:");
    for (type_name, id, line) in &analysis.references {
        eprintln!("  Type: {type_name:?}, ID: {id}, Line: {line}");
    }

    eprintln!("\nReference index v2 statistics:");
    eprintln!(
        "  Total references: {}",
        analysis.reference_index_v2.total_reference_count()
    );

    for (ref_str, count) in analysis.reference_index_v2.reference_counts() {
        eprintln!("  {ref_str}: {count} occurrences");
    }

    // Check if email@ was mistakenly indexed
    let email_refs = analysis.reference_index_v2.find_references("@example");
    eprintln!("\nReferences to @example: {}", email_refs.len());
    for r in email_refs {
        eprintln!("  Line {}, chars {}-{}", r.line, r.start_char, r.end_char);
    }
}

#[test]
fn verify_quoted_id_behavior() {
    let content = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | "my-quoted-id", My User
  | unquoted-id, Another User
"#;

    let analysis = AnalyzedDocument::analyze(content);

    eprintln!("All entities found:");
    for (type_name, entities) in &analysis.entities {
        eprintln!("  Type: {type_name}");
        for (id, line) in entities {
            eprintln!("    ID: '{id}', Line: {line}");
        }
    }

    eprintln!("\nDefinitions in reference index v2:");
    for ((t, id), loc) in analysis.reference_index_v2.all_definitions() {
        eprintln!(
            "  {}:{} at line {}, chars {}-{}",
            t, id, loc.line, loc.start_char, loc.end_char
        );
    }

    let quoted_def = analysis
        .reference_index_v2
        .find_definition("User", "my-quoted-id");
    eprintln!("\nDefinition for 'my-quoted-id': {quoted_def:?}");

    let unquoted_def = analysis
        .reference_index_v2
        .find_definition("User", "unquoted-id");
    eprintln!("Definition for 'unquoted-id': {unquoted_def:?}");
}

#[test]
fn verify_hit_testing_boundaries() {
    let content = r"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice

ref: @User:alice
";

    let analysis = AnalyzedDocument::analyze(content);

    let refs = analysis.reference_index_v2.find_references("@User:alice");
    assert!(!refs.is_empty(), "Should find at least one reference");

    let ref_loc = &refs[0];
    eprintln!("\nReference @User:alice found at:");
    eprintln!("  Line: {}", ref_loc.line);
    eprintln!("  Start char: {}", ref_loc.start_char);
    eprintln!("  End char: {}", ref_loc.end_char);
    eprintln!("  Length: {}", ref_loc.end_char - ref_loc.start_char);

    // Test at different positions
    let test_positions = [
        (ref_loc.start_char.saturating_sub(1), "before start"),
        (ref_loc.start_char, "at start"),
        (ref_loc.start_char + 1, "after start"),
        (ref_loc.end_char - 1, "before end"),
        (ref_loc.end_char, "at end"),
        (ref_loc.end_char + 1, "after end"),
    ];

    for (char_pos, desc) in test_positions {
        let pos = Position {
            line: ref_loc.line,
            character: char_pos,
        };
        let found = analysis.reference_index_v2.find_reference_at(pos);
        eprintln!(
            "  Position {} (char {}): {}",
            desc,
            char_pos,
            if found.is_some() {
                "FOUND"
            } else {
                "NOT FOUND"
            }
        );
    }
}

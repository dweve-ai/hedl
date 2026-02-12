// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for inline child list LSP features.

use hedl_lsp::analysis::AnalyzedDocument;
use hedl_lsp::code_actions::get_inline_child_code_actions;
use hedl_lsp::completion::{determine_context_optimized, get_completions, CompletionContext};
use hedl_lsp::diagnostics::check_inline_child_lists;
use hedl_lsp::hover::get_hover;
use tower_lsp::lsp_types::{Position, Url};

const SAMPLE_DOC: &str = r#"%V:1.2
%S:Product:[id,name,price]
%S:Review:[id,rating,comment]
%N:Product>Review
---
products:@Product
|p01,Laptop,999.99
  @Review#3:|r01,5,Great|r02,4,Good|r03,3,Ok
|p02,Phone,599.99
  @Review#12:|r04,5,A|r05,4,B|r06,3,C|r07,2,D|r08,1,E|r09,5,F|r10,4,G|r11,3,H|r12,2,I|r13,1,J|r14,5,K|r15,4,L
|p03,Tablet,449.99
  @Review#2:| r10,4,Space after pipe|r11,3,Another
"#;

#[test]
fn test_completion_inline_child_type() {
    let analysis = AnalyzedDocument::analyze(SAMPLE_DOC);

    // Debug: print nests
    eprintln!("Nests: {:?}", analysis.nests);
    eprintln!("Schemas: {:?}", analysis.schemas.keys().collect::<Vec<_>>());

    // Test completion after typing "@" on an indented line (after a product row)
    // Cursor at position after "  @" (line 7, after the @ character)
    let position = Position {
        line: 7,
        character: 3,
    };

    let context = determine_context_optimized(&analysis, SAMPLE_DOC, position);

    eprintln!("Context: {:?}", context);

    // Should detect inline child type context
    match &context {
        CompletionContext::InlineChildType {
            parent_type,
            partial_type,
        } => {
            eprintln!("Parent type: {:?}", parent_type);
            eprintln!("Partial type: {:?}", partial_type);
            assert_eq!(parent_type, &Some("Product".to_string()));
            assert!(partial_type.is_none() || partial_type == &Some(String::new()));
        }
        _ => panic!("Expected InlineChildType context, got {:?}", context),
    }

    // Get completions - should suggest Review (the child type)
    let completions = get_completions(&analysis, SAMPLE_DOC, position);
    eprintln!("Completions: {}", completions.len());
    for c in &completions {
        eprintln!("  - {}: {:?}", c.label, c.insert_text);
    }
    assert!(!completions.is_empty(), "Should have completions");

    // Should have Review completion with snippet
    let review_completion = completions
        .iter()
        .find(|c| c.label.contains("Review"))
        .expect("Should have Review completion");

    assert!(review_completion
        .insert_text
        .as_ref()
        .unwrap()
        .contains("#"));
    assert!(review_completion
        .insert_text
        .as_ref()
        .unwrap()
        .contains(":|"));
}

#[test]
fn test_hover_inline_child_valid() {
    let analysis = AnalyzedDocument::analyze(SAMPLE_DOC);

    // Hover over the @Review#3: part
    let position = Position {
        line: 7,
        character: 5, // Over "Review" in "@Review#3:|..."
    };

    let hover = get_hover(&analysis, SAMPLE_DOC, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    let content = match hover.contents {
        tower_lsp::lsp_types::HoverContents::Markup(markup) => markup.value,
        _ => panic!("Expected markup content"),
    };

    // Should mention inline child list
    assert!(content.contains("Inline Child List"));
    assert!(content.contains("Review"));
    assert!(content.contains("3")); // count
}

#[test]
fn test_hover_inline_child_count_mismatch() {
    let doc_with_mismatch = r#"%V:1.2
%S:Product:[id,name]
%S:Review:[id,rating]
%N:Product>Review
---
products:@Product
|p01,Laptop
  @Review#2:|r01,5
"#;

    let analysis = AnalyzedDocument::analyze(doc_with_mismatch);
    let position = Position {
        line: 7,
        character: 5,
    };

    let hover = get_hover(&analysis, doc_with_mismatch, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    let content = match hover.contents {
        tower_lsp::lsp_types::HoverContents::Markup(markup) => markup.value,
        _ => panic!("Expected markup content"),
    };

    // Should warn about count mismatch
    assert!(content.contains("Warning"));
    assert!(content.contains("Count mismatch") || content.contains("mismatch"));
}

#[test]
fn test_hover_inline_child_exceeds_max() {
    let analysis = AnalyzedDocument::analyze(SAMPLE_DOC);

    // Hover over line with 12 children (exceeds style guideline of 10)
    let position = Position {
        line: 9,
        character: 5, // Over "@Review#12:|..."
    };

    let hover = get_hover(&analysis, SAMPLE_DOC, position);
    assert!(hover.is_some());

    let hover = hover.unwrap();
    let content = match hover.contents {
        tower_lsp::lsp_types::HoverContents::Markup(markup) => markup.value,
        _ => panic!("Expected markup content"),
    };

    // Should show style guideline about exceeding 10
    assert!(content.contains("Style"));
    assert!(content.contains("> 10") || content.contains("12 declared"));
    assert!(content.contains("expanded"));
}

#[test]
fn test_diagnostics_count_mismatch() {
    let doc_with_mismatch = r#"%V:1.2
%S:Product:[id,name]
%S:Review:[id,rating]
%N:Product>Review
---
products:@Product
|p01,Laptop
  @Review#3:|r01,5|r02,4
"#;

    let analysis = AnalyzedDocument::analyze(doc_with_mismatch);
    let diagnostics = check_inline_child_lists(doc_with_mismatch, &analysis);

    // Should have error for count mismatch (declared 3, found 2)
    let count_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| {
            d.code.as_ref().and_then(|c| match c {
                tower_lsp::lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                _ => None,
            }) == Some("inline-child-count-mismatch")
        })
        .collect();

    assert!(!count_errors.is_empty(), "Should have count mismatch error");
}

#[test]
fn test_diagnostics_exceeds_max() {
    let analysis = AnalyzedDocument::analyze(SAMPLE_DOC);
    let diagnostics = check_inline_child_lists(SAMPLE_DOC, &analysis);

    // Should have warning for line with 12 children (exceeds recommended max of 10)
    let max_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| {
            d.code.as_ref().and_then(|c| match c {
                tower_lsp::lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                _ => None,
            }) == Some("inline-child-exceeds-max")
        })
        .collect();

    assert!(!max_errors.is_empty(), "Should have exceeds-max error");
}

#[test]
fn test_diagnostics_space_after_pipe() {
    let analysis = AnalyzedDocument::analyze(SAMPLE_DOC);
    let diagnostics = check_inline_child_lists(SAMPLE_DOC, &analysis);

    // Should have warning for space after pipe
    let space_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| {
            d.code.as_ref().and_then(|c| match c {
                tower_lsp::lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                _ => None,
            }) == Some("inline-child-space-after-pipe")
        })
        .collect();

    assert!(
        !space_warnings.is_empty(),
        "Should have space-after-pipe warning"
    );
    assert_eq!(
        space_warnings[0].severity,
        Some(tower_lsp::lsp_types::DiagnosticSeverity::WARNING)
    );
}

#[test]
fn test_code_action_convert_to_expanded() {
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Create range covering the line with 6 children
    let range = tower_lsp::lsp_types::Range {
        start: Position {
            line: 9,
            character: 0,
        },
        end: Position {
            line: 9,
            character: 100,
        },
    };

    let actions = get_inline_child_code_actions(&uri, SAMPLE_DOC, range);

    // Should have action to convert to expanded format
    let convert_action = actions
        .iter()
        .find(|a| match a {
            tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(action) => {
                action.title.contains("expanded")
            }
            _ => false,
        })
        .expect("Should have convert to expanded action");

    // Verify the action contains proper edit
    match convert_action {
        tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(action) => {
            assert!(action.edit.is_some());
            let edit = action.edit.as_ref().unwrap();
            assert!(edit.changes.is_some());

            let changes = edit.changes.as_ref().unwrap();
            let text_edits = changes.get(&uri).expect("Should have edits for URI");
            assert!(!text_edits.is_empty());

            // Check that expanded format includes @Review: and multiple |
            let new_text = &text_edits[0].new_text;
            assert!(new_text.contains("@Review:"));
            assert_eq!(new_text.matches('|').count(), 12); // 12 child rows (v2.0 limit is 10)
        }
        _ => panic!("Expected CodeAction"),
    }
}

#[test]
fn test_code_action_remove_space() {
    let uri = Url::parse("file:///test.hedl").unwrap();

    // Create range covering line with space after pipe
    let range = tower_lsp::lsp_types::Range {
        start: Position {
            line: 11,
            character: 0,
        },
        end: Position {
            line: 11,
            character: 100,
        },
    };

    let actions = get_inline_child_code_actions(&uri, SAMPLE_DOC, range);

    // Should have action to remove space
    let remove_space_action = actions
        .iter()
        .find(|a| match a {
            tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(action) => {
                action.title.contains("Remove space")
            }
            _ => false,
        })
        .expect("Should have remove space action");

    // Verify the action removes the space
    match remove_space_action {
        tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(action) => {
            assert!(action.edit.is_some());
            let edit = action.edit.as_ref().unwrap();
            let changes = edit.changes.as_ref().unwrap();
            let text_edits = changes.get(&uri).unwrap();

            let new_text = &text_edits[0].new_text;
            // Should not have space after :|
            assert!(new_text.contains(":|r10,"));
            assert!(!new_text.contains(":| r10,"));
        }
        _ => panic!("Expected CodeAction"),
    }
}

#[test]
fn test_multiple_child_types() {
    let doc = r#"%V:1.2
%S:Product:[id,name]
%S:Review:[id,rating]
%S:Inventory:[id,warehouse,qty]
%N:Product>Review
%N:Product>Inventory
---
products:@Product
|p01,Laptop
  @
"#;

    let analysis = AnalyzedDocument::analyze(doc);

    // Debug: check nests
    eprintln!("Nests: {:?}", analysis.nests);

    // Test completion shows both Review and Inventory
    // Position after "  @" on a new line
    let position = Position {
        line: 9,
        character: 3, // After "  @"
    };

    let completions = get_completions(&analysis, doc, position);
    eprintln!(
        "Completions for multiple child types: {}",
        completions.len()
    );
    for c in &completions {
        eprintln!("  - {}", c.label);
    }

    // Should suggest both Review and Inventory
    let has_review = completions.iter().any(|c| c.label.contains("Review"));
    let has_inventory = completions.iter().any(|c| c.label.contains("Inventory"));

    assert!(has_review, "Should suggest Review");
    assert!(has_inventory, "Should suggest Inventory");
}

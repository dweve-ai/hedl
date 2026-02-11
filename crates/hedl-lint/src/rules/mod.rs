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

//! Lint rules

// Submodules
mod common;
mod inline_rules;
mod naming_rules;
mod reference_rules;
mod schema_rules;
mod v20_rules;

// Re-export common types
pub use common::{LintRule, RuleConfig};

// Re-export all rule structs
pub use inline_rules::{InlineChildExceedsMaxRule, InlineCountMismatchRule, MissingCountHintRule};
pub use naming_rules::IdNamingRule;
pub use reference_rules::UnqualifiedKvReferenceRule;
pub use schema_rules::{EmptyListRule, UnusedSchemaRule};
pub use v20_rules::{ForbidDittoRule, IndentationRule, RequiredHeadersRule, SpaceAfterPipeRule};

/// Get all default rules (v2.0).
pub fn default_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(IdNamingRule),
        Box::new(UnusedSchemaRule),
        Box::new(EmptyListRule),
        Box::new(UnqualifiedKvReferenceRule),
        Box::new(InlineChildExceedsMaxRule),
        Box::new(InlineCountMismatchRule),
        Box::new(MissingCountHintRule),
        Box::new(ForbidDittoRule),
        Box::new(RequiredHeadersRule),
        Box::new(SpaceAfterPipeRule),
        Box::new(IndentationRule),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{DiagnosticKind, Severity};
    use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
    use std::collections::BTreeMap;

    // ==================== RuleConfig tests ====================

    // ==================== Inline child rules tests ====================

    #[test]
    fn test_inline_child_exceeds_max_rule_id() {
        let rule = InlineChildExceedsMaxRule;
        assert_eq!(rule.id(), "inline-child-exceeds-max");
    }

    #[test]
    fn test_inline_count_mismatch_rule_id() {
        let rule = InlineCountMismatchRule;
        assert_eq!(rule.id(), "inline-count-mismatch");
    }

    #[test]
    fn test_missing_count_hint_rule_id() {
        let rule = MissingCountHintRule;
        assert_eq!(rule.id(), "missing-count-hint");
    }

    #[test]
    fn test_rule_config_default() {
        let config = RuleConfig::default();
        assert!(config.enabled);
        assert!(!config.error);
    }

    #[test]
    fn test_rule_config_clone() {
        let config = RuleConfig {
            enabled: false,
            error: true,
        };
        let cloned = config.clone();
        assert!(!cloned.enabled);
        assert!(cloned.error);
    }

    #[test]
    fn test_rule_config_debug() {
        let config = RuleConfig::default();
        let debug = format!("{config:?}");
        assert!(debug.contains("RuleConfig"));
        assert!(debug.contains("enabled"));
    }

    // ==================== default_rules tests ====================

    #[test]
    fn test_default_rules_count() {
        // v2.0 has 11 default rules
        let rules = default_rules();
        assert_eq!(rules.len(), 11);
    }

    #[test]
    fn test_default_rules_ids() {
        let rules = default_rules();
        let ids: Vec<&str> = rules.iter().map(|r| r.id()).collect();

        // v2.0 default rules
        assert!(ids.contains(&"id-naming"));
        assert!(ids.contains(&"unused-schema"));
        assert!(ids.contains(&"empty-list"));
        assert!(ids.contains(&"unqualified-kv-ref"));
        assert!(ids.contains(&"inline-child-exceeds-max"));
        assert!(ids.contains(&"inline-count-mismatch"));
        assert!(ids.contains(&"missing-count-hint"));
        assert!(ids.contains(&"forbid-ditto"));
        assert!(ids.contains(&"required-headers"));
        assert!(ids.contains(&"space-after-pipe"));
        assert!(ids.contains(&"indentation"));
        assert_eq!(ids.len(), 11);
    }

    #[test]
    fn test_default_rules_have_descriptions() {
        let rules = default_rules();
        for rule in rules {
            assert!(!rule.description().is_empty());
        }
    }

    // ==================== IdNamingRule tests ====================

    #[test]
    fn test_id_naming_rule_id() {
        let rule = IdNamingRule;
        assert_eq!(rule.id(), "id-naming");
    }

    #[test]
    fn test_id_naming_rule_description() {
        let rule = IdNamingRule;
        assert!(!rule.description().is_empty());
        assert!(rule.description().contains("ID"));
    }

    #[test]
    fn test_id_naming_empty_doc() {
        let rule = IdNamingRule;
        let doc = Document::new((2, 0));
        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_id_naming_short_id() {
        let rule = IdNamingRule;
        let mut doc = Document::new((2, 0));

        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "a", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(diagnostics[0].kind(), DiagnosticKind::IdNaming));
        assert!(diagnostics[0].message().contains("short"));
    }

    #[test]
    fn test_id_naming_numeric_id() {
        let rule = IdNamingRule;
        let mut doc = Document::new((2, 0));

        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "123", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        assert!(!diagnostics.is_empty());
        assert!(diagnostics.iter().any(|d| d.message().contains("numbers")));
    }

    #[test]
    fn test_id_naming_descriptive_id_passes() {
        let rule = IdNamingRule;
        let mut doc = Document::new((2, 0));

        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "user_alice", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_id_naming_mixed_alphanumeric_passes() {
        let rule = IdNamingRule;
        let mut doc = Document::new((2, 0));

        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "user123", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        // user123 has letters and numbers - not numeric only, so passes
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_id_naming_underscore_only_passes() {
        let rule = IdNamingRule;
        let mut doc = Document::new((2, 0));

        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "___", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        // Underscores only but no digits, so doesn't trigger numeric check
        // But 3 chars, so doesn't trigger short check
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_id_naming_numeric_with_underscores() {
        let rule = IdNamingRule;
        let mut doc = Document::new((2, 0));

        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "1_2_3", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        // 1_2_3 has digits and only underscores/digits
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_id_naming_nested_objects() {
        let rule = IdNamingRule;
        let mut doc = Document::new((2, 0));

        let mut nested = BTreeMap::new();
        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "x", vec![])); // Short ID
        nested.insert("nested_list".to_string(), Item::List(list));

        doc.root
            .insert("container".to_string(), Item::Object(nested));

        let diagnostics = rule.check(&doc);
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_id_naming_multiple_violations() {
        let rule = IdNamingRule;
        let mut doc = Document::new((2, 0));

        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "a", vec![]));
        list.add_row(Node::new("Test", "b", vec![]));
        list.add_row(Node::new("Test", "123", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        assert_eq!(diagnostics.len(), 3); // 2 short + 1 numeric
    }

    // ==================== UnusedSchemaRule tests ====================

    #[test]
    fn test_unused_schema_rule_id() {
        let rule = UnusedSchemaRule;
        assert_eq!(rule.id(), "unused-schema");
    }

    #[test]
    fn test_unused_schema_empty_doc() {
        let rule = UnusedSchemaRule;
        let doc = Document::new((2, 0));
        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_unused_schema_all_used() {
        let rule = UnusedSchemaRule;
        let mut doc = Document::new((2, 0));

        doc.structs
            .insert("User".to_string(), vec!["id".to_string()]);

        let mut list = MatrixList::new("User", vec!["id".to_string()]);
        list.add_row(Node::new("User", "u1", vec![]));
        doc.root.insert("users".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_unused_schema_one_unused() {
        let rule = UnusedSchemaRule;
        let mut doc = Document::new((2, 0));

        doc.structs
            .insert("User".to_string(), vec!["id".to_string()]);
        doc.structs
            .insert("Unused".to_string(), vec!["id".to_string()]);

        let mut list = MatrixList::new("User", vec!["id".to_string()]);
        list.add_row(Node::new("User", "u1", vec![]));
        doc.root.insert("users".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message().contains("Unused"));
    }

    #[test]
    fn test_unused_schema_multiple_unused() {
        let rule = UnusedSchemaRule;
        let mut doc = Document::new((2, 0));

        doc.structs.insert("A".to_string(), vec!["id".to_string()]);
        doc.structs.insert("B".to_string(), vec!["id".to_string()]);
        doc.structs.insert("C".to_string(), vec!["id".to_string()]);

        let diagnostics = rule.check(&doc);
        assert_eq!(diagnostics.len(), 3);
    }

    #[test]
    fn test_unused_schema_deep_nested_types() {
        let rule = UnusedSchemaRule;
        let mut doc = Document::new((2, 0));

        // Define schemas
        doc.structs
            .insert("User".to_string(), vec!["id".to_string()]);
        doc.structs
            .insert("Post".to_string(), vec!["id".to_string()]);
        doc.structs
            .insert("Comment".to_string(), vec!["id".to_string()]);
        doc.structs
            .insert("UnusedType".to_string(), vec!["id".to_string()]);

        // Create hierarchy: User > Post > Comment
        // Comment is only used in a deep nest, not as a direct child
        let mut user_list = MatrixList::new("User", vec!["id".to_string()]);
        let mut user = Node::new("User", "alice", vec![]);

        // Add a Post child to User
        let mut post = Node::new("Post", "post1", vec![]);

        // Add a Comment child to Post (deep nesting)
        let comment = Node::new("Comment", "comment1", vec![]);
        post.add_child("Comment", comment);

        user.add_child("Post", post);
        user_list.add_row(user);

        doc.root.insert("users".to_string(), Item::List(user_list));

        let diagnostics = rule.check(&doc);

        // Only UnusedType should be reported as unused
        // User, Post, and Comment are all used (even Comment which is deeply nested)
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message().contains("UnusedType"));
    }

    #[test]
    fn test_unused_schema_multiple_branches_deep_nesting() {
        let rule = UnusedSchemaRule;
        let mut doc = Document::new((2, 0));

        // Define schemas with multiple branches
        doc.structs
            .insert("User".to_string(), vec!["id".to_string()]);
        doc.structs
            .insert("Post".to_string(), vec!["id".to_string()]);
        doc.structs
            .insert("Comment".to_string(), vec!["id".to_string()]);
        doc.structs
            .insert("Like".to_string(), vec!["id".to_string()]);
        doc.structs
            .insert("Tag".to_string(), vec!["id".to_string()]);

        // Create hierarchy with multiple branches:
        // User > Post > Comment
        //             > Like
        //      > Tag (directly under User)
        let mut user_list = MatrixList::new("User", vec!["id".to_string()]);
        let mut user = Node::new("User", "alice", vec![]);

        // Add Post with Comment and Like children
        let mut post = Node::new("Post", "post1", vec![]);
        post.add_child("Comment", Node::new("Comment", "comment1", vec![]));
        post.add_child("Like", Node::new("Like", "like1", vec![]));
        user.add_child("Post", post);

        // Add Tag directly to User
        user.add_child("Tag", Node::new("Tag", "tag1", vec![]));

        user_list.add_row(user);
        doc.root.insert("users".to_string(), Item::List(user_list));

        let diagnostics = rule.check(&doc);

        // All types should be marked as used
        assert!(diagnostics.is_empty());
    }

    // ==================== EmptyListRule tests ====================

    #[test]
    fn test_empty_list_rule_id() {
        let rule = EmptyListRule;
        assert_eq!(rule.id(), "empty-list");
    }

    #[test]
    fn test_empty_list_no_lists() {
        let rule = EmptyListRule;
        let doc = Document::new((2, 0));
        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_empty_list_non_empty_list() {
        let rule = EmptyListRule;
        let mut doc = Document::new((2, 0));

        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "t1", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_empty_list_detected() {
        let rule = EmptyListRule;
        let mut doc = Document::new((2, 0));

        let list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty_items".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(diagnostics[0].kind(), DiagnosticKind::EmptyList));
    }

    #[test]
    fn test_empty_list_nested() {
        let rule = EmptyListRule;
        let mut doc = Document::new((2, 0));

        let mut nested = BTreeMap::new();
        let list = MatrixList::new("Empty", vec!["id".to_string()]);
        nested.insert("nested".to_string(), Item::List(list));

        doc.root
            .insert("container".to_string(), Item::Object(nested));

        let diagnostics = rule.check(&doc);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn test_empty_list_multiple() {
        let rule = EmptyListRule;
        let mut doc = Document::new((2, 0));

        doc.root.insert(
            "a".to_string(),
            Item::List(MatrixList::new("A", vec!["id".to_string()])),
        );
        doc.root.insert(
            "b".to_string(),
            Item::List(MatrixList::new("B", vec!["id".to_string()])),
        );

        let diagnostics = rule.check(&doc);
        assert_eq!(diagnostics.len(), 2);
    }

    // ==================== UnqualifiedKvReferenceRule tests ====================

    #[test]
    fn test_unqualified_kv_ref_rule_id() {
        let rule = UnqualifiedKvReferenceRule;
        assert_eq!(rule.id(), "unqualified-kv-ref");
    }

    #[test]
    fn test_unqualified_kv_ref_empty_doc() {
        let rule = UnqualifiedKvReferenceRule;
        let doc = Document::new((2, 0));
        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_unqualified_kv_ref_qualified_passes() {
        let rule = UnqualifiedKvReferenceRule;
        let mut doc = Document::new((2, 0));

        let ref_val = Value::Reference(Reference::qualified("User", "alice"));
        doc.root.insert("owner".to_string(), Item::Scalar(ref_val));

        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_unqualified_kv_ref_detected() {
        let rule = UnqualifiedKvReferenceRule;
        let mut doc = Document::new((2, 0));

        let ref_val = Value::Reference(Reference::local("some_id"));
        doc.root.insert("owner".to_string(), Item::Scalar(ref_val));

        let diagnostics = rule.check(&doc);
        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].kind(),
            DiagnosticKind::UnqualifiedKvReference
        ));
        assert!(diagnostics[0].suggestion().is_some());
    }

    #[test]
    fn test_unqualified_kv_ref_nested() {
        let rule = UnqualifiedKvReferenceRule;
        let mut doc = Document::new((2, 0));

        let mut nested = BTreeMap::new();
        let ref_val = Value::Reference(Reference::local("nested_id"));
        nested.insert("ref".to_string(), Item::Scalar(ref_val));

        doc.root
            .insert("container".to_string(), Item::Object(nested));

        let diagnostics = rule.check(&doc);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn test_unqualified_kv_ref_multiple() {
        let rule = UnqualifiedKvReferenceRule;
        let mut doc = Document::new((2, 0));

        doc.root.insert(
            "ref1".to_string(),
            Item::Scalar(Value::Reference(Reference::local("a"))),
        );
        doc.root.insert(
            "ref2".to_string(),
            Item::Scalar(Value::Reference(Reference::local("b"))),
        );

        let diagnostics = rule.check(&doc);
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn test_unqualified_kv_ref_non_ref_scalar() {
        let rule = UnqualifiedKvReferenceRule;
        let mut doc = Document::new((2, 0));

        doc.root.insert(
            "name".to_string(),
            Item::Scalar(Value::String("test".to_string().into())),
        );
        doc.root
            .insert("count".to_string(), Item::Scalar(Value::Int(42)));

        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    // ==================== LintRule trait tests ====================

    #[test]
    fn test_lint_rule_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<IdNamingRule>();
        assert_send_sync::<UnusedSchemaRule>();
        assert_send_sync::<EmptyListRule>();
        assert_send_sync::<UnqualifiedKvReferenceRule>();
        assert_send_sync::<InlineChildExceedsMaxRule>();
        assert_send_sync::<InlineCountMismatchRule>();
        assert_send_sync::<MissingCountHintRule>();
        assert_send_sync::<ForbidDittoRule>();
        assert_send_sync::<RequiredHeadersRule>();
    }

    #[test]
    fn test_boxed_rules() {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(IdNamingRule), Box::new(EmptyListRule)];

        for rule in &rules {
            let doc = Document::new((2, 0));
            let _ = rule.check(&doc);
        }
    }

    // ==================== SpaceAfterPipeRule tests ====================

    #[test]
    fn test_space_after_pipe_rule_id() {
        let rule = SpaceAfterPipeRule;
        assert_eq!(rule.id(), "space-after-pipe");
    }

    #[test]
    fn test_space_after_pipe_rule_description() {
        let rule = SpaceAfterPipeRule;
        assert!(!rule.description().is_empty());
        assert!(rule.description().contains("space"));
    }

    #[test]
    fn test_space_after_pipe_no_context() {
        let rule = SpaceAfterPipeRule;
        let doc = Document::new((2, 0));
        // Without context, no diagnostics
        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_space_after_pipe_correct_format() {
        let rule = SpaceAfterPipeRule;
        let doc = Document::new((2, 0));
        let source = "@Post#2:|p1,Hello|p2,World";
        let ctx = crate::runner::LintContext::new(None, 0, source.to_string());
        let diagnostics = rule.check_with_context(&doc, &ctx);
        // No space after pipe - should be clean
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_space_after_pipe_detects_space() {
        let rule = SpaceAfterPipeRule;
        let doc = Document::new((2, 0));
        // v2.0 violation: space after pipe in inline child declaration
        let source = "@Post#2:| p1,Hello| p2,World";
        let ctx = crate::runner::LintContext::new(None, 0, source.to_string());
        let diagnostics = rule.check_with_context(&doc, &ctx);
        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].kind(),
            DiagnosticKind::SpaceAfterPipe
        ));
    }

    #[test]
    fn test_space_after_pipe_multiple_lines() {
        let rule = SpaceAfterPipeRule;
        let doc = Document::new((2, 0));
        // First line has space after pipe (violation), second line is correct
        let source = "@Post#1:| bad\n@Comment#2:|good|fine";
        let ctx = crate::runner::LintContext::new(None, 0, source.to_string());
        let diagnostics = rule.check_with_context(&doc, &ctx);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line(), Some(1));
    }

    #[test]
    fn test_space_after_pipe_non_inline_skipped() {
        let rule = SpaceAfterPipeRule;
        let doc = Document::new((2, 0));
        // Regular line without @ prefix should be ignored
        let source = "title: |some value with pipe";
        let ctx = crate::runner::LintContext::new(None, 0, source.to_string());
        let diagnostics = rule.check_with_context(&doc, &ctx);
        assert!(diagnostics.is_empty());
    }

    // ==================== IndentationRule tests ====================

    #[test]
    fn test_indentation_rule_id() {
        let rule = IndentationRule;
        assert_eq!(rule.id(), "indentation");
    }

    #[test]
    fn test_indentation_rule_description() {
        let rule = IndentationRule;
        assert!(!rule.description().is_empty());
        assert!(rule.description().contains("indentation"));
    }

    #[test]
    fn test_indentation_pre_v20_skipped() {
        let rule = IndentationRule;
        let doc = Document::new((1, 2)); // Pre-v2.0 - skips indentation checks
        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_indentation_v20_without_context() {
        let rule = IndentationRule;
        let doc = Document::new((2, 0)); // v2.0 document
        let diagnostics = rule.check(&doc);
        // Without context, no diagnostics are generated
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_indentation_correct() {
        use crate::runner::LintContext;

        let rule = IndentationRule;
        let mut doc = Document::new((2, 0));

        let source = "%V:2.0
%STRUCT: Test: [id]
---
company: Acme
items:@Test
 |item1
 |item2
metadata:
 version: 1.0
";

        doc.structs
            .insert("Test".to_string(), vec!["id".to_string()]);

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_indentation_tab_detected() {
        use crate::runner::LintContext;

        let rule = IndentationRule;
        let doc = Document::new((2, 0));

        let source = "%V:2.0
---
company: Acme
\titems:@Test
"; // Tab instead of space

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].kind(),
            DiagnosticKind::InvalidIndentation
        ));
        assert!(diagnostics[0].message().contains("Tabs"));
    }

    #[test]
    fn test_indentation_wrong_nesting() {
        use crate::runner::LintContext;

        let rule = IndentationRule;
        let doc = Document::new((2, 0));

        let source = "%V:2.0
---
metadata:
  version: 1.0
"; // 2 spaces instead of 1

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics
            .iter()
            .any(|d| matches!(d.kind(), DiagnosticKind::InvalidIndentation)));
    }

    #[test]
    fn test_indentation_matrix_row() {
        use crate::runner::LintContext;

        let rule = IndentationRule;
        let doc = Document::new((2, 0));

        let source = "%V:2.0
---
items:@Test
  |item1
"; // 2 spaces instead of 1

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics
            .iter()
            .any(|d| matches!(d.kind(), DiagnosticKind::InvalidIndentation)
                && d.message().contains("Matrix row")));
    }

    // ==================== ForbidDittoRule tests ====================

    #[test]
    fn test_forbid_ditto_rule_id() {
        let rule = ForbidDittoRule;
        assert_eq!(rule.id(), "forbid-ditto");
    }

    #[test]
    fn test_forbid_ditto_pre_v20_allows() {
        use crate::runner::LintContext;

        let rule = ForbidDittoRule;
        let doc = Document::new((1, 2)); // Pre-v2.0 allows ditto

        let source = "%VERSION: 1.2
---
items:@Test
 |id1,value1
 |id2,^
";

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        // Pre-v2.0 allows ditto, so no diagnostics
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_forbid_ditto_v20_detects() {
        use crate::runner::LintContext;

        let rule = ForbidDittoRule;
        let doc = Document::new((2, 0)); // v2.0 forbids ditto

        let source = "%V:2.0
%NULL:~
%QUOTE:\"
---
items:@Test
 |id1,value1
 |id2,^
";

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(diagnostics[0].kind(), DiagnosticKind::ForbidDitto));
        assert_eq!(diagnostics[0].severity(), Severity::Error);
        assert!(diagnostics[0].message().contains("not allowed"));
    }

    #[test]
    fn test_forbid_ditto_no_context() {
        let rule = ForbidDittoRule;
        let doc = Document::new((2, 0));

        // Without context, no diagnostics
        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_forbid_ditto_multiple_violations() {
        use crate::runner::LintContext;

        let rule = ForbidDittoRule;
        let doc = Document::new((2, 0));

        let source = "%V:2.0
%NULL:~
%QUOTE:\"
---
items:@Test
 |id1,value1
 |id2,^
 |id3,^
";

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics
            .iter()
            .all(|d| matches!(d.kind(), DiagnosticKind::ForbidDitto)));
    }

    // ==================== RequiredHeadersRule tests ====================

    #[test]
    fn test_required_headers_rule_id() {
        let rule = RequiredHeadersRule;
        assert_eq!(rule.id(), "required-headers");
    }

    #[test]
    fn test_required_headers_pre_v20_not_checked() {
        use crate::runner::LintContext;

        let rule = RequiredHeadersRule;
        let doc = Document::new((1, 2)); // Pre-v2.0 doesn't require these

        let source = "%VERSION: 1.2
---
key: value
";

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        // Pre-v2.0 doesn't require NULL/QUOTE, so no diagnostics
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_required_headers_v20_all_present() {
        use crate::runner::LintContext;

        let rule = RequiredHeadersRule;
        let doc = Document::new((2, 0));

        let source = "%V:2.0
%NULL:~
%QUOTE:\"
---
key: value
";

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        // All required headers present
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_required_headers_missing_null() {
        use crate::runner::LintContext;

        let rule = RequiredHeadersRule;
        let doc = Document::new((2, 0));

        let source = "%V:2.0
%QUOTE:\"
---
key: value
";

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].kind(),
            DiagnosticKind::RequiredHeaders
        ));
        assert!(diagnostics[0].message().contains("%NULL"));
        assert_eq!(diagnostics[0].severity(), Severity::Error);
    }

    #[test]
    fn test_required_headers_missing_quote() {
        use crate::runner::LintContext;

        let rule = RequiredHeadersRule;
        let doc = Document::new((2, 0));

        let source = "%V:2.0
%NULL:~
---
key: value
";

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].kind(),
            DiagnosticKind::RequiredHeaders
        ));
        assert!(diagnostics[0].message().contains("%QUOTE"));
        assert_eq!(diagnostics[0].severity(), Severity::Error);
    }

    #[test]
    fn test_required_headers_missing_both() {
        use crate::runner::LintContext;

        let rule = RequiredHeadersRule;
        let doc = Document::new((2, 0));

        let source = "%V:2.0
---
key: value
";

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics
            .iter()
            .all(|d| matches!(d.kind(), DiagnosticKind::RequiredHeaders)));
        assert!(diagnostics.iter().any(|d| d.message().contains("%NULL")));
        assert!(diagnostics.iter().any(|d| d.message().contains("%QUOTE")));
    }

    #[test]
    fn test_required_headers_no_context() {
        let rule = RequiredHeadersRule;
        let doc = Document::new((2, 0));

        // Without context, no diagnostics (parser provides defaults)
        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_required_headers_custom_values() {
        use crate::runner::LintContext;

        let rule = RequiredHeadersRule;
        let doc = Document::new((2, 0));

        let source = "%V:2.0
%NULL:?
%QUOTE:'
---
key: value
";

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        // Custom values are acceptable
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_required_headers_with_whitespace() {
        use crate::runner::LintContext;

        let rule = RequiredHeadersRule;
        let doc = Document::new((2, 0));

        let source = "%V:2.0
%NULL :~
%QUOTE :\"
---
key: value
";

        let context = LintContext::from_text(source);
        let diagnostics = rule.check_with_context(&doc, &context);

        // Whitespace should be accepted
        assert!(diagnostics.is_empty());
    }
}

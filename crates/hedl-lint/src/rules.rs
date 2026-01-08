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

use crate::diagnostic::Diagnostic;
use hedl_core::{Document, Item, Node};
use std::any::Any;
use std::collections::BTreeMap;

/// Configuration for a single rule
#[derive(Debug, Clone)]
pub struct RuleConfig {
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Whether to treat warnings as errors
    pub error: bool,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            error: false,
        }
    }
}

/// Trait for lint rules
pub trait LintRule: Send + Sync {
    /// Rule identifier
    fn id(&self) -> &str;

    /// Rule description
    fn description(&self) -> &str;

    /// Run the rule on a document
    fn check(&self, doc: &Document) -> Vec<Diagnostic>;

    /// Run the rule on a document with context information
    ///
    /// The default implementation calls `check()`, ignoring the context.
    /// Rules that need context (file path, line numbers) should override this method.
    ///
    /// The context is passed as `&dyn Any` to avoid circular imports.
    /// Cast it to `&crate::runner::LintContext` to access context information.
    fn check_with_context(&self, doc: &Document, _context: &dyn Any) -> Vec<Diagnostic> {
        self.check(doc)
    }
}

/// Rule: ID naming conventions
pub struct IdNamingRule;

impl LintRule for IdNamingRule {
    fn id(&self) -> &str {
        "id-naming"
    }
    fn description(&self) -> &str {
        "Check ID naming conventions (lowercase, descriptive)"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        check_item_ids(&doc.root, &mut diagnostics);
        diagnostics
    }
}

/// Maximum recursion depth for document traversal.
///
/// This limit prevents stack overflow attacks from deeply nested document structures.
/// A malicious document with 1000+ levels of nesting could cause stack exhaustion,
/// leading to process crashes or potential security vulnerabilities.
///
/// Security Rationale:
/// - Stack frames typically consume 100-200 bytes each
/// - At 1000 depth, this represents ~100-200KB of stack usage
/// - Most legitimate HEDL documents have <10 levels of nesting
/// - This limit provides defense-in-depth against DoS attacks
const MAX_RECURSION_DEPTH: usize = 1000;

/// Check all item IDs in a document tree with depth protection.
///
/// # Security
///
/// This function implements recursive depth limiting to prevent stack overflow
/// from maliciously crafted deeply nested documents. If the depth limit is
/// exceeded, further traversal is halted and a warning diagnostic is generated.
fn check_item_ids(items: &BTreeMap<String, Item>, diagnostics: &mut Vec<Diagnostic>) {
    check_item_ids_bounded(items, diagnostics, 0);
}

fn check_item_ids_bounded(
    items: &BTreeMap<String, Item>,
    diagnostics: &mut Vec<Diagnostic>,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        diagnostics.push(Diagnostic::warning(
            crate::diagnostic::DiagnosticKind::Custom(
                "max-depth-exceeded".to_string()
            ),
            format!(
                "Maximum nesting depth of {} exceeded during ID checking. \
                 Further nested items will not be checked.",
                MAX_RECURSION_DEPTH
            ),
            "id-naming",
        ));
        return;
    }

    for item in items.values() {
        match item {
            Item::List(list) => {
                for row in &list.rows {
                    check_node_id(&row.id, diagnostics);
                    check_node_children_bounded(&row.children, diagnostics, depth + 1);
                }
            }
            Item::Object(child) => {
                check_item_ids_bounded(child, diagnostics, depth + 1);
            }
            _ => {}
        }
    }
}

fn check_node_id(id: &str, diagnostics: &mut Vec<Diagnostic>) {
    use crate::diagnostic::DiagnosticKind;

    // Check for non-descriptive IDs
    if id.len() == 1 {
        diagnostics.push(Diagnostic::hint(
            DiagnosticKind::IdNaming,
            format!(
                "ID '{}' is very short, consider a more descriptive name",
                id
            ),
            "id-naming",
        ));
    }
    // Check for numeric-only IDs (must have at least one digit, not just underscores)
    let has_digit = id.chars().any(|c| c.is_ascii_digit());
    let all_numeric_or_underscore = id.chars().all(|c| c.is_ascii_digit() || c == '_');
    if has_digit && all_numeric_or_underscore {
        diagnostics.push(Diagnostic::hint(
            DiagnosticKind::IdNaming,
            format!(
                "ID '{}' contains only numbers, consider adding descriptive prefix",
                id
            ),
            "id-naming",
        ));
    }
}

/// Check node children with recursion depth protection.
///
/// # Security
///
/// This function enforces a maximum recursion depth to prevent stack overflow
/// vulnerabilities. Deeply nested structures are common attack vectors for
/// causing denial-of-service through stack exhaustion.
fn check_node_children_bounded(
    children: &BTreeMap<String, Vec<Node>>,
    diagnostics: &mut Vec<Diagnostic>,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        diagnostics.push(Diagnostic::warning(
            crate::diagnostic::DiagnosticKind::Custom(
                "max-depth-exceeded".to_string()
            ),
            format!(
                "Maximum nesting depth of {} exceeded. \
                 Further nested nodes will not be checked.",
                MAX_RECURSION_DEPTH
            ),
            "id-naming",
        ));
        return;
    }

    for nodes in children.values() {
        for node in nodes {
            check_node_id(&node.id, diagnostics);
            check_node_children_bounded(&node.children, diagnostics, depth + 1);
        }
    }
}

/// Rule: Unused schemas
pub struct UnusedSchemaRule;

impl LintRule for UnusedSchemaRule {
    fn id(&self) -> &str {
        "unused-schema"
    }
    fn description(&self) -> &str {
        "Check for unused %STRUCT definitions"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        use crate::diagnostic::DiagnosticKind;
        use std::collections::HashSet;

        let mut used_types = HashSet::new();
        collect_used_types(&doc.root, &mut used_types);

        let mut diagnostics = Vec::new();
        for type_name in doc.structs.keys() {
            if !used_types.contains(type_name.as_str()) {
                diagnostics.push(Diagnostic::warning(
                    DiagnosticKind::UnusedSchema,
                    format!("Schema '{}' is defined but never used", type_name),
                    "unused-schema",
                ));
            }
        }

        diagnostics
    }
}

/// Collect used types with depth protection.
///
/// # Security
///
/// Implements recursion depth limiting to prevent stack overflow from
/// deeply nested document structures during type collection.
fn collect_used_types<'a>(
    items: &'a BTreeMap<String, Item>,
    used: &mut std::collections::HashSet<&'a str>,
) {
    collect_used_types_bounded(items, used, 0);
}

fn collect_used_types_bounded<'a>(
    items: &'a BTreeMap<String, Item>,
    used: &mut std::collections::HashSet<&'a str>,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        // Silently stop traversal at max depth for type collection
        // This prevents stack overflow while still collecting types from
        // non-malicious portions of the document
        return;
    }

    for item in items.values() {
        match item {
            Item::List(list) => {
                used.insert(&list.type_name);
                for row in &list.rows {
                    for child_type in row.children.keys() {
                        used.insert(child_type);
                    }
                }
            }
            Item::Object(child) => {
                collect_used_types_bounded(child, used, depth + 1);
            }
            _ => {}
        }
    }
}

/// Rule: Unused aliases
///
/// This rule is available but not enabled by default.
/// It can be enabled via LintConfig for stricter linting.
#[allow(dead_code)]
pub struct UnusedAliasRule;

impl LintRule for UnusedAliasRule {
    fn id(&self) -> &str {
        "unused-alias"
    }
    fn description(&self) -> &str {
        "Check for unused %ALIAS definitions"
    }

    fn check(&self, _doc: &Document) -> Vec<Diagnostic> {
        // Alias usage tracking requires parser-level integration to track references
        // Returns empty for now as Document doesn't expose alias reference counts
        Vec::new()
    }
}

/// Rule: Empty matrix lists
pub struct EmptyListRule;

impl LintRule for EmptyListRule {
    fn id(&self) -> &str {
        "empty-list"
    }
    fn description(&self) -> &str {
        "Warn about empty matrix lists"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        check_empty_lists(&doc.root, &mut diagnostics);
        diagnostics
    }
}

/// Check for empty lists with depth protection.
///
/// # Security
///
/// Implements recursion depth limiting to prevent stack overflow from
/// deeply nested document structures during empty list detection.
fn check_empty_lists(items: &BTreeMap<String, Item>, diagnostics: &mut Vec<Diagnostic>) {
    check_empty_lists_bounded(items, diagnostics, 0);
}

fn check_empty_lists_bounded(
    items: &BTreeMap<String, Item>,
    diagnostics: &mut Vec<Diagnostic>,
    depth: usize,
) {
    use crate::diagnostic::DiagnosticKind;

    if depth > MAX_RECURSION_DEPTH {
        diagnostics.push(Diagnostic::warning(
            DiagnosticKind::Custom("max-depth-exceeded".to_string()),
            format!(
                "Maximum nesting depth of {} exceeded during empty list checking. \
                 Further nested items will not be checked.",
                MAX_RECURSION_DEPTH
            ),
            "empty-list",
        ));
        return;
    }

    for (key, item) in items {
        match item {
            Item::List(list) => {
                if list.rows.is_empty() {
                    diagnostics.push(Diagnostic::hint(
                        DiagnosticKind::EmptyList,
                        format!("Matrix list '{}' is empty", key),
                        "empty-list",
                    ));
                }
            }
            Item::Object(child) => {
                check_empty_lists_bounded(child, diagnostics, depth + 1);
            }
            _ => {}
        }
    }
}

/// Rule: Unqualified references in Key-Value context
pub struct UnqualifiedKvReferenceRule;

impl LintRule for UnqualifiedKvReferenceRule {
    fn id(&self) -> &str {
        "unqualified-kv-ref"
    }
    fn description(&self) -> &str {
        "Warn about unqualified references in Key-Value context"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        check_kv_references(&doc.root, &mut diagnostics);
        diagnostics
    }
}

/// Check for unqualified references in Key-Value context with depth protection.
///
/// # Security
///
/// Implements recursion depth limiting to prevent stack overflow from
/// deeply nested document structures during reference checking.
fn check_kv_references(items: &BTreeMap<String, Item>, diagnostics: &mut Vec<Diagnostic>) {
    check_kv_references_bounded(items, diagnostics, 0);
}

fn check_kv_references_bounded(
    items: &BTreeMap<String, Item>,
    diagnostics: &mut Vec<Diagnostic>,
    depth: usize,
) {
    use crate::diagnostic::DiagnosticKind;
    use hedl_core::Value;

    if depth > MAX_RECURSION_DEPTH {
        diagnostics.push(Diagnostic::warning(
            DiagnosticKind::Custom("max-depth-exceeded".to_string()),
            format!(
                "Maximum nesting depth of {} exceeded during reference checking. \
                 Further nested items will not be checked.",
                MAX_RECURSION_DEPTH
            ),
            "unqualified-kv-ref",
        ));
        return;
    }

    for item in items.values() {
        match item {
            Item::Scalar(Value::Reference(r)) => {
                if r.type_name.is_none() {
                    diagnostics.push(
                        Diagnostic::warning(
                            DiagnosticKind::UnqualifiedKvReference,
                            format!("Unqualified reference '@{}' in Key-Value context, consider using qualified form '@Type:{}'", r.id, r.id),
                            "unqualified-kv-ref"
                        ).with_suggestion(format!("Use @Type:{}", r.id))
                    );
                }
            }
            Item::Object(child) => {
                check_kv_references_bounded(child, diagnostics, depth + 1);
            }
            _ => {}
        }
    }
}

/// Get all default rules
pub fn default_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(IdNamingRule),
        Box::new(UnusedSchemaRule),
        // NOTE: UnusedAliasRule is available but not enabled by default
        // as it requires parser-level integration to track alias usage.
        // Enable it manually via LintRunner::add_rule() if needed.
        Box::new(EmptyListRule),
        Box::new(UnqualifiedKvReferenceRule),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::DiagnosticKind;
    use hedl_core::{MatrixList, Node, Reference, Value};

    // ==================== RuleConfig tests ====================

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
        let debug = format!("{:?}", config);
        assert!(debug.contains("RuleConfig"));
        assert!(debug.contains("enabled"));
    }

    // ==================== default_rules tests ====================

    #[test]
    fn test_default_rules_count() {
        let rules = default_rules();
        assert_eq!(rules.len(), 4); // UnusedAliasRule is available but not in defaults
    }

    #[test]
    fn test_default_rules_ids() {
        let rules = default_rules();
        let ids: Vec<&str> = rules.iter().map(|r| r.id()).collect();

        assert!(ids.contains(&"id-naming"));
        assert!(ids.contains(&"unused-schema"));
        assert!(!ids.contains(&"unused-alias")); // Not in defaults
        assert!(ids.contains(&"empty-list"));
        assert!(ids.contains(&"unqualified-kv-ref"));
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
        let doc = Document::new((1, 0));
        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_id_naming_short_id() {
        let rule = IdNamingRule;
        let mut doc = Document::new((1, 0));

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
        let mut doc = Document::new((1, 0));

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
        let mut doc = Document::new((1, 0));

        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "user_alice", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_id_naming_mixed_alphanumeric_passes() {
        let rule = IdNamingRule;
        let mut doc = Document::new((1, 0));

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
        let mut doc = Document::new((1, 0));

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
        let mut doc = Document::new((1, 0));

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
        let mut doc = Document::new((1, 0));

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
        let mut doc = Document::new((1, 0));

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
        let doc = Document::new((1, 0));
        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_unused_schema_all_used() {
        let rule = UnusedSchemaRule;
        let mut doc = Document::new((1, 0));

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
        let mut doc = Document::new((1, 0));

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
        let mut doc = Document::new((1, 0));

        doc.structs.insert("A".to_string(), vec!["id".to_string()]);
        doc.structs.insert("B".to_string(), vec!["id".to_string()]);
        doc.structs.insert("C".to_string(), vec!["id".to_string()]);

        let diagnostics = rule.check(&doc);
        assert_eq!(diagnostics.len(), 3);
    }

    // ==================== UnusedAliasRule tests ====================

    #[test]
    fn test_unused_alias_rule_id() {
        let rule = UnusedAliasRule;
        assert_eq!(rule.id(), "unused-alias");
    }

    #[test]
    fn test_unused_alias_empty_doc() {
        let rule = UnusedAliasRule;
        let doc = Document::new((1, 0));
        let diagnostics = rule.check(&doc);
        // Empty document has no aliases to check
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
        let doc = Document::new((1, 0));
        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_empty_list_non_empty_list() {
        let rule = EmptyListRule;
        let mut doc = Document::new((1, 0));

        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "t1", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_empty_list_detected() {
        let rule = EmptyListRule;
        let mut doc = Document::new((1, 0));

        let list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty_items".to_string(), Item::List(list));

        let diagnostics = rule.check(&doc);
        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(diagnostics[0].kind(), DiagnosticKind::EmptyList));
    }

    #[test]
    fn test_empty_list_nested() {
        let rule = EmptyListRule;
        let mut doc = Document::new((1, 0));

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
        let mut doc = Document::new((1, 0));

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
        let doc = Document::new((1, 0));
        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_unqualified_kv_ref_qualified_passes() {
        let rule = UnqualifiedKvReferenceRule;
        let mut doc = Document::new((1, 0));

        let ref_val = Value::Reference(Reference::qualified("User", "alice"));
        doc.root.insert("owner".to_string(), Item::Scalar(ref_val));

        let diagnostics = rule.check(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_unqualified_kv_ref_detected() {
        let rule = UnqualifiedKvReferenceRule;
        let mut doc = Document::new((1, 0));

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
        let mut doc = Document::new((1, 0));

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
        let mut doc = Document::new((1, 0));

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
        let mut doc = Document::new((1, 0));

        doc.root.insert(
            "name".to_string(),
            Item::Scalar(Value::String("test".to_string())),
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
        assert_send_sync::<UnusedAliasRule>();
        assert_send_sync::<EmptyListRule>();
        assert_send_sync::<UnqualifiedKvReferenceRule>();
    }

    #[test]
    fn test_boxed_rules() {
        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(IdNamingRule), Box::new(EmptyListRule)];

        for rule in &rules {
            let doc = Document::new((1, 0));
            let _ = rule.check(&doc);
        }
    }
}

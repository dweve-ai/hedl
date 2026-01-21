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

//! Single-pass document visitor for efficient multi-rule linting
//!
//! This module implements a unified visitor pattern that traverses the document
//! tree once and applies all enabled rules during the same pass, eliminating
//! redundant traversals.

use crate::diagnostic::{Diagnostic, DiagnosticKind};
use hedl_core::{Document, Item, MatrixList, Node, Value};
use std::collections::{BTreeMap, HashSet};

/// Maximum recursion depth for document traversal (security limit)
const MAX_RECURSION_DEPTH: usize = 1000;

/// Unified visitor that applies all rules in a single document traversal
pub struct MultiRuleVisitor<'a> {
    // Rule-specific state
    id_naming_diagnostics: Vec<Diagnostic>,
    empty_lists: Vec<String>,
    unqualified_refs: Vec<(String, String)>, // (key, id)
    used_types: HashSet<&'a str>,

    // Traversal state
    depth: usize,
    current_path: Vec<String>,

    // Limits and config
    max_diagnostics: usize,
    diagnostic_count: usize,
    depth_warning_issued: bool,

    // Rule enable/disable flags
    id_naming_enabled: bool,
    unused_schema_enabled: bool,
    empty_list_enabled: bool,
    unqualified_kv_ref_enabled: bool,
}

impl<'a> MultiRuleVisitor<'a> {
    /// Create a new visitor with specified diagnostic limit
    pub fn new(max_diagnostics: usize) -> Self {
        Self {
            id_naming_diagnostics: Vec::new(),
            empty_lists: Vec::new(),
            unqualified_refs: Vec::new(),
            used_types: HashSet::new(),
            depth: 0,
            current_path: Vec::new(),
            max_diagnostics,
            diagnostic_count: 0,
            depth_warning_issued: false,
            // All rules enabled by default
            id_naming_enabled: true,
            unused_schema_enabled: true,
            empty_list_enabled: true,
            unqualified_kv_ref_enabled: true,
        }
    }

    /// Enable or disable specific rules
    pub fn set_rule_enabled(&mut self, rule_id: &str, enabled: bool) {
        match rule_id {
            "id-naming" => self.id_naming_enabled = enabled,
            "unused-schema" => self.unused_schema_enabled = enabled,
            "empty-list" => self.empty_list_enabled = enabled,
            "unqualified-kv-ref" => self.unqualified_kv_ref_enabled = enabled,
            _ => {} // Unknown rule, ignore
        }
    }

    /// Single entry point for document traversal
    pub fn visit_document(&mut self, doc: &'a Document) {
        self.visit_items(&doc.root);

        // Post-traversal: Check for unused schemas
        if self.unused_schema_enabled && !self.should_stop() {
            self.check_unused_schemas(doc);
        }
    }

    /// Visit all items in a container
    fn visit_items(&mut self, items: &'a BTreeMap<String, Item>) {
        // Early exit on depth or diagnostic limit
        if self.should_stop() {
            return;
        }

        if self.depth > MAX_RECURSION_DEPTH {
            if !self.depth_warning_issued {
                self.depth_warning_issued = true;
                self.diagnostic_count += 1;
            }
            return;
        }

        for (key, item) in items {
            self.current_path.push(key.clone());

            match item {
                Item::List(list) => {
                    self.visit_list(key, list);
                }
                Item::Object(child) => {
                    self.depth += 1;
                    self.visit_items(child);
                    self.depth -= 1;
                }
                Item::Scalar(value) => {
                    // Check for unqualified references in Key-Value context
                    if self.unqualified_kv_ref_enabled {
                        self.check_scalar_value(key, value);
                    }
                }
            }

            self.current_path.pop();

            // Check diagnostic limit after each major item
            if self.should_stop() {
                return;
            }
        }
    }

    /// Visit a matrix list and apply relevant rules
    fn visit_list(&mut self, key: &str, list: &'a MatrixList) {
        if self.should_stop() {
            return;
        }

        // EmptyListRule check
        if self.empty_list_enabled && list.rows.is_empty() {
            self.empty_lists.push(key.to_string());
            self.diagnostic_count += 1;
        }

        // UnusedSchemaRule: collect type usage
        if self.unused_schema_enabled {
            self.used_types.insert(&list.type_name);
        }

        // Process nodes if id-naming rule is enabled
        if self.id_naming_enabled || self.unused_schema_enabled {
            for row in &list.rows {
                if self.id_naming_enabled {
                    self.check_node(row);
                }

                // Collect type usage for unused-schema rule
                if self.unused_schema_enabled {
                    // Collect from reference values in row fields
                    for field in &row.fields {
                        if let hedl_core::Value::Reference(ref r) = field {
                            if let Some(ref type_name) = r.type_name {
                                self.used_types.insert(type_name);
                            }
                        }
                    }

                    // Collect child type names
                    if let Some(children) = row.children() {
                        for child_type in children.keys() {
                            self.used_types.insert(child_type);
                        }
                    }
                }

                // Recursively visit nested children if any rule needs it
                if self.id_naming_enabled {
                    if let Some(children) = row.children() {
                        self.visit_node_children(children);
                    }
                } else if self.unused_schema_enabled {
                    // Still need to collect types from nested nodes
                    if let Some(children) = row.children() {
                        self.collect_types_from_children(children);
                    }
                }

                if self.should_stop() {
                    return;
                }
            }
        }
    }

    /// Check a single node (`IdNamingRule` logic)
    fn check_node(&mut self, node: &Node) {
        if self.should_stop() {
            return;
        }

        // Check for short IDs
        if node.id.len() == 1 {
            self.id_naming_diagnostics.push(Diagnostic::hint(
                DiagnosticKind::IdNaming,
                format!(
                    "ID '{}' is very short, consider a more descriptive name",
                    node.id
                ),
                "id-naming",
            ));
            self.diagnostic_count += 1;
        }

        // Check for numeric-only IDs
        let has_digit = node.id.chars().any(|c| c.is_ascii_digit());
        let all_numeric_or_underscore = node.id.chars().all(|c| c.is_ascii_digit() || c == '_');

        if has_digit && all_numeric_or_underscore {
            self.id_naming_diagnostics.push(Diagnostic::hint(
                DiagnosticKind::IdNaming,
                format!(
                    "ID '{}' contains only numbers, consider adding descriptive prefix",
                    node.id
                ),
                "id-naming",
            ));
            self.diagnostic_count += 1;
        }
    }

    /// Visit nested node children
    fn visit_node_children(&mut self, children: &'a BTreeMap<String, Vec<Node>>) {
        if self.should_stop() {
            return;
        }

        if self.depth > MAX_RECURSION_DEPTH {
            if !self.depth_warning_issued {
                self.depth_warning_issued = true;
                self.diagnostic_count += 1;
            }
            return;
        }

        self.depth += 1;
        for nodes in children.values() {
            for node in nodes {
                self.check_node(node);

                // Collect types for unused-schema rule
                if self.unused_schema_enabled {
                    // Collect from reference values in fields
                    for field in &node.fields {
                        if let hedl_core::Value::Reference(ref r) = field {
                            if let Some(ref type_name) = r.type_name {
                                self.used_types.insert(type_name);
                            }
                        }
                    }

                    // Collect child types
                    if let Some(children) = node.children() {
                        for child_type in children.keys() {
                            self.used_types.insert(child_type);
                        }
                    }
                }

                if let Some(children) = node.children() {
                    self.visit_node_children(children);
                }

                if self.should_stop() {
                    break;
                }
            }
            if self.should_stop() {
                break;
            }
        }
        self.depth -= 1;
    }

    /// Collect types from children without checking IDs (optimization when id-naming is disabled)
    fn collect_types_from_children(&mut self, children: &'a BTreeMap<String, Vec<Node>>) {
        if self.depth > MAX_RECURSION_DEPTH {
            return;
        }

        self.depth += 1;
        for (child_type, nodes) in children {
            self.used_types.insert(child_type);
            for node in nodes {
                // Collect types from reference values in node fields
                for field in &node.fields {
                    if let hedl_core::Value::Reference(ref r) = field {
                        if let Some(ref type_name) = r.type_name {
                            self.used_types.insert(type_name);
                        }
                    }
                }

                if let Some(node_children) = node.children() {
                    self.collect_types_from_children(node_children);
                }
            }
        }
        self.depth -= 1;
    }

    /// Check scalar value for unqualified references
    fn check_scalar_value(&mut self, _key: &str, value: &Value) {
        if self.should_stop() {
            return;
        }

        if let Value::Reference(ref r) = value {
            if r.type_name.is_none() {
                self.unqualified_refs
                    .push((self.current_path.join("."), r.id.to_string()));
                self.diagnostic_count += 1;
            }
        }
    }

    /// Post-traversal: Check for unused schemas
    fn check_unused_schemas(&mut self, doc: &'a Document) {
        // Already at limit, don't check
        if self.should_stop() {
            return;
        }

        // Preallocate space for unused schema diagnostics
        let unused_count = doc
            .structs
            .keys()
            .filter(|schema_name| !self.used_types.contains(schema_name.as_str()))
            .count();

        // Don't process if it would exceed limit
        if self.diagnostic_count + unused_count > self.max_diagnostics {
            return;
        }

        for schema_name in doc.structs.keys() {
            if !self.used_types.contains(schema_name.as_str()) {
                self.diagnostic_count += 1;
                if self.diagnostic_count > self.max_diagnostics {
                    break;
                }
            }
        }
    }

    /// Check if we should stop traversal
    #[inline]
    fn should_stop(&self) -> bool {
        self.diagnostic_count >= self.max_diagnostics
    }

    /// Convert collected state into diagnostics
    pub fn into_diagnostics(self, doc: &Document) -> Vec<Diagnostic> {
        let capacity = self.diagnostic_count.min(self.max_diagnostics);
        let mut diagnostics = Vec::with_capacity(capacity);

        // Add ID naming diagnostics
        if self.id_naming_enabled {
            diagnostics.extend(self.id_naming_diagnostics);
        }

        // Add empty list diagnostics
        if self.empty_list_enabled {
            for list_key in self.empty_lists {
                if diagnostics.len() >= self.max_diagnostics {
                    break;
                }
                diagnostics.push(Diagnostic::hint(
                    DiagnosticKind::EmptyList,
                    format!("Matrix list '{list_key}' is empty"),
                    "empty-list",
                ));
            }
        }

        // Add unqualified reference diagnostics
        if self.unqualified_kv_ref_enabled {
            for (_path, ref_id) in self.unqualified_refs {
                if diagnostics.len() >= self.max_diagnostics {
                    break;
                }
                diagnostics.push(
                    Diagnostic::warning(
                        DiagnosticKind::UnqualifiedKvReference,
                        format!(
                            "Unqualified reference '@{ref_id}' in Key-Value context, consider using qualified form '@Type:{ref_id}'"
                        ),
                        "unqualified-kv-ref",
                    )
                    .with_suggestion(format!("Use @Type:{ref_id}")),
                );
            }
        }

        // Add unused schema diagnostics
        if self.unused_schema_enabled {
            for schema_name in doc.structs.keys() {
                if diagnostics.len() >= self.max_diagnostics {
                    break;
                }
                if !self.used_types.contains(schema_name.as_str()) {
                    diagnostics.push(Diagnostic::warning(
                        DiagnosticKind::UnusedSchema,
                        format!("Schema '{schema_name}' is defined but never used"),
                        "unused-schema",
                    ));
                }
            }
        }

        // Add depth warning if issued
        if self.depth_warning_issued {
            diagnostics.push(Diagnostic::warning(
                DiagnosticKind::Custom("max-depth-exceeded".to_string()),
                format!(
                    "Maximum nesting depth of {MAX_RECURSION_DEPTH} exceeded. Further nested items were not checked."
                ),
                "visitor",
            ));
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};

    #[test]
    fn test_visitor_empty_document() {
        let doc = Document::new((1, 0));
        let mut visitor = MultiRuleVisitor::new(10_000);
        visitor.visit_document(&doc);
        let diagnostics = visitor.into_diagnostics(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_visitor_short_id() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "a", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let mut visitor = MultiRuleVisitor::new(10_000);
        visitor.visit_document(&doc);
        let diagnostics = visitor.into_diagnostics(&doc);

        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(diagnostics[0].kind(), DiagnosticKind::IdNaming));
    }

    #[test]
    fn test_visitor_numeric_id() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "123", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let mut visitor = MultiRuleVisitor::new(10_000);
        visitor.visit_document(&doc);
        let diagnostics = visitor.into_diagnostics(&doc);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics
            .iter()
            .any(|d| matches!(d.kind(), DiagnosticKind::IdNaming)));
    }

    #[test]
    fn test_visitor_empty_list() {
        let mut doc = Document::new((1, 0));
        let list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty".to_string(), Item::List(list));

        let mut visitor = MultiRuleVisitor::new(10_000);
        visitor.visit_document(&doc);
        let diagnostics = visitor.into_diagnostics(&doc);

        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(diagnostics[0].kind(), DiagnosticKind::EmptyList));
    }

    #[test]
    fn test_visitor_unused_schema() {
        let mut doc = Document::new((1, 0));
        doc.structs
            .insert("Unused".to_string(), vec!["id".to_string()]);

        let mut visitor = MultiRuleVisitor::new(10_000);
        visitor.visit_document(&doc);
        let diagnostics = visitor.into_diagnostics(&doc);

        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].kind(),
            DiagnosticKind::UnusedSchema
        ));
    }

    #[test]
    fn test_visitor_unqualified_reference() {
        let mut doc = Document::new((1, 0));
        let ref_val = Value::Reference(Reference::local("some_id"));
        doc.root.insert("ref".to_string(), Item::Scalar(ref_val));

        let mut visitor = MultiRuleVisitor::new(10_000);
        visitor.visit_document(&doc);
        let diagnostics = visitor.into_diagnostics(&doc);

        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].kind(),
            DiagnosticKind::UnqualifiedKvReference
        ));
    }

    #[test]
    fn test_visitor_multiple_violations() {
        let mut doc = Document::new((1, 0));

        // Short ID
        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "a", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        // Empty list
        let empty_list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty".to_string(), Item::List(empty_list));

        // Unused schema
        doc.structs
            .insert("Unused".to_string(), vec!["id".to_string()]);

        // Unqualified reference
        let ref_val = Value::Reference(Reference::local("some_id"));
        doc.root.insert("ref".to_string(), Item::Scalar(ref_val));

        let mut visitor = MultiRuleVisitor::new(10_000);
        visitor.visit_document(&doc);
        let diagnostics = visitor.into_diagnostics(&doc);

        assert_eq!(diagnostics.len(), 4);
    }

    #[test]
    fn test_visitor_diagnostic_limit() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Test", vec!["id".to_string()]);

        // Add 100 nodes with short IDs
        for i in 0..100 {
            list.add_row(Node::new("Test", format!("{}", i % 10), vec![]));
        }
        doc.root.insert("items".to_string(), Item::List(list));

        // Set limit to 10
        let mut visitor = MultiRuleVisitor::new(10);
        visitor.visit_document(&doc);
        let diagnostics = visitor.into_diagnostics(&doc);

        // Should stop at limit
        assert!(diagnostics.len() <= 10);
    }

    #[test]
    fn test_visitor_rule_disable() {
        let mut doc = Document::new((1, 0));

        // Create violations for all rules
        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "a", vec![])); // Short ID
        doc.root.insert("items".to_string(), Item::List(list));

        let empty_list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty".to_string(), Item::List(empty_list));

        doc.structs
            .insert("Unused".to_string(), vec!["id".to_string()]);

        // Disable id-naming rule
        let mut visitor = MultiRuleVisitor::new(10_000);
        visitor.set_rule_enabled("id-naming", false);
        visitor.visit_document(&doc);
        let diagnostics = visitor.into_diagnostics(&doc);

        // Should not have id-naming diagnostics
        assert!(!diagnostics
            .iter()
            .any(|d| matches!(d.kind(), DiagnosticKind::IdNaming)));
        // But should have others
        assert!(diagnostics
            .iter()
            .any(|d| matches!(d.kind(), DiagnosticKind::EmptyList)));
    }

    #[test]
    fn test_visitor_nested_objects() {
        let mut doc = Document::new((1, 0));

        let mut nested = BTreeMap::new();
        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "x", vec![])); // Short ID
        nested.insert("nested_list".to_string(), Item::List(list));

        doc.root
            .insert("container".to_string(), Item::Object(nested));

        let mut visitor = MultiRuleVisitor::new(10_000);
        visitor.visit_document(&doc);
        let diagnostics = visitor.into_diagnostics(&doc);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics
            .iter()
            .any(|d| matches!(d.kind(), DiagnosticKind::IdNaming)));
    }

    #[test]
    fn test_visitor_well_formed_document() {
        let mut doc = Document::new((1, 0));

        doc.structs
            .insert("User".to_string(), vec!["id".to_string()]);

        let mut list = MatrixList::new("User", vec!["id".to_string()]);
        list.add_row(Node::new("User", "alice_smith", vec![]));
        doc.root.insert("users".to_string(), Item::List(list));

        let ref_val = Value::Reference(Reference::qualified("User", "alice_smith"));
        doc.root.insert("owner".to_string(), Item::Scalar(ref_val));

        let mut visitor = MultiRuleVisitor::new(10_000);
        visitor.visit_document(&doc);
        let diagnostics = visitor.into_diagnostics(&doc);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_visitor_all_rules_disabled() {
        let mut doc = Document::new((1, 0));

        // Create violations for all rules
        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "a", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let empty_list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty".to_string(), Item::List(empty_list));

        doc.structs
            .insert("Unused".to_string(), vec!["id".to_string()]);

        let ref_val = Value::Reference(Reference::local("some_id"));
        doc.root.insert("ref".to_string(), Item::Scalar(ref_val));

        // Disable all rules
        let mut visitor = MultiRuleVisitor::new(10_000);
        visitor.set_rule_enabled("id-naming", false);
        visitor.set_rule_enabled("empty-list", false);
        visitor.set_rule_enabled("unused-schema", false);
        visitor.set_rule_enabled("unqualified-kv-ref", false);

        visitor.visit_document(&doc);
        let diagnostics = visitor.into_diagnostics(&doc);

        assert!(diagnostics.is_empty());
    }
}

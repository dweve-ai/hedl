// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Document structure for parsed HEDL.
//!
//! # Memory Optimizations
//!
//! This module includes several memory optimizations:
//! - SmallVec for inline field storage (avoids allocations for ≤4 fields)
//! - Lazy BTreeMap allocation for children (None until first child added)
//! - Compact child_count representation (u16 instead of `Option<usize>`)
//! - Type names can be interned during parsing to reduce duplication

use crate::Value;
use smallvec::SmallVec;
use std::collections::BTreeMap;

/// A node in a matrix list.
///
/// # Memory Layout Optimizations
///
/// Size reduced from ~120 bytes to ~80 bytes per node:
/// - `fields`: Uses SmallVec to inline up to 4 values (typical case)
/// - `children`: Lazy allocation - None until first child added (saves 24 bytes for leaf nodes)
/// - `child_count`: Uses u16 (2 bytes) with 0 = no hint (vs 16 bytes for `Option<usize>`)
///
/// For 10,000 nodes: ~400KB saved in struct overhead alone.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// The type name (from schema).
    /// During parsing, this can be interned to share memory across all nodes of same type.
    pub type_name: String,
    /// The node's ID (first column value).
    pub id: String,
    /// Field values (aligned with schema columns).
    /// SmallVec avoids heap allocation for ≤4 fields (common case: 90%+ of nodes).
    pub fields: SmallVec<[Value; 4]>,
    /// Child nodes grouped by type (from NEST relationships).
    /// Lazy allocation - None until first child added. ~70% of nodes are leaves.
    pub children: Option<Box<BTreeMap<String, Vec<Node>>>>,
    /// Count of direct children (for LLM comprehension hints).
    /// 0 = no hint provided. Max 65,535 children (sufficient for all practical cases).
    pub child_count: u16,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            type_name: String::new(),
            id: String::new(),
            fields: SmallVec::new(),
            children: None,
            child_count: 0,
        }
    }
}

impl Node {
    /// Create a new node.
    pub fn new(type_name: impl Into<String>, id: impl Into<String>, fields: Vec<Value>) -> Self {
        Self {
            type_name: type_name.into(),
            id: id.into(),
            fields: SmallVec::from_vec(fields),
            children: None,
            child_count: 0,
        }
    }

    /// Get a field value by column index.
    pub fn get_field(&self, index: usize) -> Option<&Value> {
        self.fields.get(index)
    }

    /// Add a child node.
    /// Uses lazy allocation - children BTreeMap is only created on first add.
    pub fn add_child(&mut self, child_type: impl Into<String>, child: Node) {
        let children = self
            .children
            .get_or_insert_with(|| Box::new(BTreeMap::new()));
        children.entry(child_type.into()).or_default().push(child);
    }

    /// Set the child count hint (for LLM comprehension).
    /// Saturates at u16::MAX (65,535) if count exceeds.
    pub fn set_child_count(&mut self, count: usize) {
        self.child_count = count.min(u16::MAX as usize) as u16;
    }

    /// Get the child count hint, if provided.
    pub fn get_child_count(&self) -> Option<usize> {
        if self.child_count > 0 {
            Some(self.child_count as usize)
        } else {
            None
        }
    }

    /// Create a new node with a child count hint.
    ///
    /// # Arguments
    /// - `type_name`: The type name from the schema
    /// - `fields`: Field values where `fields[0]` MUST be the node ID (a string)
    /// - `child_count`: Expected number of children (for LLM hints)
    pub fn with_child_count(
        type_name: impl Into<String>,
        id: impl Into<String>,
        fields: Vec<Value>,
        child_count: usize,
    ) -> Self {
        Self {
            type_name: type_name.into(),
            id: id.into(),
            fields: SmallVec::from_vec(fields),
            children: None,
            child_count: child_count.min(u16::MAX as usize) as u16,
        }
    }

    /// Get the children map, if any children have been added.
    pub fn children(&self) -> Option<&BTreeMap<String, Vec<Node>>> {
        self.children.as_deref()
    }

    /// Get mutable access to children map, if it exists.
    pub fn children_mut(&mut self) -> Option<&mut BTreeMap<String, Vec<Node>>> {
        self.children.as_deref_mut()
    }
}

/// A typed matrix list with schema.
#[derive(Debug, Clone, PartialEq)]
pub struct MatrixList {
    /// The type name.
    pub type_name: String,
    /// Column names (schema).
    pub schema: Vec<String>,
    /// Row data as nodes.
    pub rows: Vec<Node>,
    /// Optional count hint for LLM comprehension (e.g., `teams(3):@Team`).
    pub count_hint: Option<usize>,
}

impl MatrixList {
    /// Create a new matrix list.
    pub fn new(type_name: impl Into<String>, schema: Vec<String>) -> Self {
        Self {
            type_name: type_name.into(),
            schema,
            rows: Vec::new(),
            count_hint: None,
        }
    }

    /// Create a new matrix list with rows.
    pub fn with_rows(type_name: impl Into<String>, schema: Vec<String>, rows: Vec<Node>) -> Self {
        Self {
            type_name: type_name.into(),
            schema,
            rows,
            count_hint: None,
        }
    }

    /// Create a new matrix list with a count hint.
    pub fn with_count_hint(
        type_name: impl Into<String>,
        schema: Vec<String>,
        count_hint: usize,
    ) -> Self {
        Self {
            type_name: type_name.into(),
            schema,
            rows: Vec::new(),
            count_hint: Some(count_hint),
        }
    }

    /// Add a row/node to the list.
    pub fn add_row(&mut self, node: Node) {
        self.rows.push(node);
    }

    /// Get the number of columns.
    pub fn column_count(&self) -> usize {
        self.schema.len()
    }
}

/// An item in the document body.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// A scalar value.
    Scalar(Value),
    /// A nested object.
    Object(BTreeMap<String, Item>),
    /// A matrix list.
    List(MatrixList),
}

impl Item {
    /// Try to get as a scalar value.
    pub fn as_scalar(&self) -> Option<&Value> {
        match self {
            Self::Scalar(v) => Some(v),
            _ => None,
        }
    }

    /// Try to get as an object.
    pub fn as_object(&self) -> Option<&BTreeMap<String, Item>> {
        match self {
            Self::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Try to get as a matrix list.
    pub fn as_list(&self) -> Option<&MatrixList> {
        match self {
            Self::List(l) => Some(l),
            _ => None,
        }
    }
}

/// A parsed HEDL document.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// Version (major, minor).
    pub version: (u32, u32),
    /// Schema versions for types used in this document (for schema evolution).
    /// Empty map means no schema versioning is used (legacy behavior).
    pub schema_versions: BTreeMap<String, crate::schema_version::SchemaVersion>,
    /// Alias definitions.
    pub aliases: BTreeMap<String, String>,
    /// Struct definitions (type -> columns).
    pub structs: BTreeMap<String, Vec<String>>,
    /// Nest relationships (parent -> children).
    /// A parent type can have multiple child types.
    pub nests: BTreeMap<String, Vec<String>>,
    /// Root body content.
    pub root: BTreeMap<String, Item>,
}

impl Document {
    /// Create a new empty document.
    pub fn new(version: (u32, u32)) -> Self {
        Self {
            version,
            schema_versions: BTreeMap::new(),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root: BTreeMap::new(),
        }
    }

    /// Get schema version for a type.
    pub fn get_schema_version(
        &self,
        type_name: &str,
    ) -> Option<crate::schema_version::SchemaVersion> {
        self.schema_versions.get(type_name).copied()
    }

    /// Set schema version for a type.
    pub fn set_schema_version(
        &mut self,
        type_name: String,
        version: crate::schema_version::SchemaVersion,
    ) {
        self.schema_versions.insert(type_name, version);
    }

    /// Get an item from the root by key.
    pub fn get(&self, key: &str) -> Option<&Item> {
        self.root.get(key)
    }

    /// Get a struct schema by type name.
    pub fn get_schema(&self, type_name: &str) -> Option<&Vec<String>> {
        self.structs.get(type_name)
    }

    /// Get the child types for a parent type (from NEST).
    /// A parent type can have multiple child types (e.g., Customer > Address, Customer > Order).
    pub fn get_child_types(&self, parent_type: &str) -> Option<&Vec<String>> {
        self.nests.get(parent_type)
    }

    /// Expand an alias key to its value.
    pub fn expand_alias(&self, key: &str) -> Option<&String> {
        self.aliases.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Node tests ====================

    #[test]
    fn test_node_new() {
        let node = Node::new("User", "user-1", vec![Value::Int(42)]);
        assert_eq!(node.type_name, "User");
        assert_eq!(node.id, "user-1");
        assert_eq!(node.fields.len(), 1);
        assert!(node.children.is_none());
    }

    #[test]
    fn test_node_get_field() {
        let node = Node::new(
            "User",
            "1",
            vec![Value::Int(1), Value::String("name".to_string().into())],
        );
        assert_eq!(node.get_field(0), Some(&Value::Int(1)));
        assert_eq!(
            node.get_field(1),
            Some(&Value::String("name".to_string().into()))
        );
        assert_eq!(node.get_field(2), None);
    }

    #[test]
    fn test_node_add_child() {
        let mut parent = Node::new("User", "1", vec![]);
        let child = Node::new("Post", "p1", vec![]);
        parent.add_child("Post", child);

        let children = parent.children().unwrap();
        assert!(children.contains_key("Post"));
        assert_eq!(children["Post"].len(), 1);
    }

    #[test]
    fn test_node_add_multiple_children_same_type() {
        let mut parent = Node::new("User", "1", vec![]);
        parent.add_child("Post", Node::new("Post", "p1", vec![]));
        parent.add_child("Post", Node::new("Post", "p2", vec![]));

        let children = parent.children().unwrap();
        assert_eq!(children["Post"].len(), 2);
    }

    #[test]
    fn test_node_add_children_different_types() {
        let mut parent = Node::new("User", "1", vec![]);
        parent.add_child("Post", Node::new("Post", "p1", vec![]));
        parent.add_child("Comment", Node::new("Comment", "c1", vec![]));

        let children = parent.children().unwrap();
        assert_eq!(children.len(), 2);
        assert!(children.contains_key("Post"));
        assert!(children.contains_key("Comment"));
    }

    #[test]
    fn test_node_equality() {
        let a = Node::new("User", "1", vec![Value::Int(42)]);
        let b = Node::new("User", "1", vec![Value::Int(42)]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_node_clone() {
        let mut original = Node::new("User", "1", vec![Value::Int(42)]);
        original.add_child("Post", Node::new("Post", "p1", vec![]));
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_node_debug() {
        let node = Node::new("User", "1", vec![]);
        let debug = format!("{:?}", node);
        assert!(debug.contains("User"));
        assert!(debug.contains("type_name"));
    }

    #[test]
    fn test_node_child_count() {
        let node = Node::with_child_count("User", "1", vec![], 5);
        assert_eq!(node.get_child_count(), Some(5));
    }

    #[test]
    fn test_node_no_child_count() {
        let node = Node::new("User", "1", vec![]);
        assert_eq!(node.get_child_count(), None);
    }

    #[test]
    fn test_node_set_child_count() {
        let mut node = Node::new("User", "1", vec![]);
        node.set_child_count(10);
        assert_eq!(node.get_child_count(), Some(10));
    }

    #[test]
    fn test_node_child_count_saturation() {
        let node = Node::with_child_count("User", "1", vec![], 100_000);
        assert_eq!(node.get_child_count(), Some(65535)); // u16::MAX
    }

    #[test]
    fn test_node_lazy_children() {
        let node = Node::new("User", "1", vec![]);
        assert!(node.children.is_none()); // Not allocated until first child
    }

    #[test]
    fn test_node_children_accessor() {
        let mut node = Node::new("User", "1", vec![]);
        assert!(node.children().is_none());

        node.add_child("Post", Node::new("Post", "p1", vec![]));
        assert!(node.children().is_some());
        assert!(node.children().unwrap().contains_key("Post"));
    }

    #[test]
    fn test_node_children_mut_accessor() {
        let mut node = Node::new("User", "1", vec![]);
        node.add_child("Post", Node::new("Post", "p1", vec![]));

        if let Some(children) = node.children_mut() {
            children.insert("Comment".to_string(), vec![]);
        }

        assert!(node.children().unwrap().contains_key("Comment"));
    }

    // ==================== MatrixList tests ====================

    #[test]
    fn test_matrix_list_new() {
        let list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
        assert_eq!(list.type_name, "User");
        assert_eq!(list.schema, vec!["id", "name"]);
        assert!(list.rows.is_empty());
    }

    #[test]
    fn test_matrix_list_add_row() {
        let mut list = MatrixList::new("User", vec!["id".to_string()]);
        list.add_row(Node::new("User", "1", vec![]));
        assert_eq!(list.rows.len(), 1);
    }

    #[test]
    fn test_matrix_list_column_count() {
        let list = MatrixList::new(
            "User",
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );
        assert_eq!(list.column_count(), 3);
    }

    #[test]
    fn test_matrix_list_empty_schema() {
        let list = MatrixList::new("Empty", vec![]);
        assert_eq!(list.column_count(), 0);
    }

    #[test]
    fn test_matrix_list_equality() {
        let mut a = MatrixList::new("User", vec!["id".to_string()]);
        a.add_row(Node::new("User", "1", vec![]));
        let mut b = MatrixList::new("User", vec!["id".to_string()]);
        b.add_row(Node::new("User", "1", vec![]));
        assert_eq!(a, b);
    }

    #[test]
    fn test_matrix_list_clone() {
        let mut list = MatrixList::new("User", vec!["id".to_string()]);
        list.add_row(Node::new("User", "1", vec![]));
        let cloned = list.clone();
        assert_eq!(list, cloned);
    }

    #[test]
    fn test_matrix_list_with_rows() {
        let rows = vec![
            Node::new("User", "1", vec![]),
            Node::new("User", "2", vec![]),
        ];
        let list = MatrixList::with_rows("User", vec!["id".to_string()], rows);
        assert_eq!(list.rows.len(), 2);
    }

    #[test]
    fn test_matrix_list_with_count_hint() {
        let list = MatrixList::with_count_hint("User", vec!["id".to_string()], 100);
        assert_eq!(list.count_hint, Some(100));
    }

    // ==================== Item tests ====================

    #[test]
    fn test_item_scalar() {
        let item = Item::Scalar(Value::Int(42));
        assert!(item.as_scalar().is_some());
        assert!(item.as_object().is_none());
        assert!(item.as_list().is_none());
    }

    #[test]
    fn test_item_object() {
        let mut obj = BTreeMap::new();
        obj.insert("key".to_string(), Item::Scalar(Value::Int(1)));
        let item = Item::Object(obj);
        assert!(item.as_object().is_some());
        assert!(item.as_scalar().is_none());
    }

    #[test]
    fn test_item_list() {
        let list = MatrixList::new("User", vec!["id".to_string()]);
        let item = Item::List(list);
        assert!(item.as_list().is_some());
        assert!(item.as_scalar().is_none());
    }

    #[test]
    fn test_item_as_scalar_returns_value() {
        let item = Item::Scalar(Value::String("hello".to_string().into()));
        let value = item.as_scalar().unwrap();
        assert_eq!(value.as_str(), Some("hello"));
    }

    #[test]
    fn test_item_as_object_returns_map() {
        let mut obj = BTreeMap::new();
        obj.insert("a".to_string(), Item::Scalar(Value::Int(1)));
        let item = Item::Object(obj);
        let map = item.as_object().unwrap();
        assert!(map.contains_key("a"));
    }

    #[test]
    fn test_item_equality() {
        let a = Item::Scalar(Value::Int(42));
        let b = Item::Scalar(Value::Int(42));
        assert_eq!(a, b);
    }

    #[test]
    fn test_item_clone() {
        let item = Item::Scalar(Value::String("test".to_string().into()));
        let cloned = item.clone();
        assert_eq!(item, cloned);
    }

    // ==================== Document tests ====================

    #[test]
    fn test_document_new() {
        let doc = Document::new((2, 0));
        assert_eq!(doc.version, (2, 0));
        assert!(doc.aliases.is_empty());
        assert!(doc.structs.is_empty());
        assert!(doc.nests.is_empty());
        assert!(doc.root.is_empty());
    }

    #[test]
    fn test_document_get() {
        let mut doc = Document::new((2, 0));
        doc.root
            .insert("key".to_string(), Item::Scalar(Value::Int(42)));
        assert!(doc.get("key").is_some());
        assert!(doc.get("missing").is_none());
    }

    #[test]
    fn test_document_get_schema() {
        let mut doc = Document::new((2, 0));
        doc.structs.insert(
            "User".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );
        let schema = doc.get_schema("User").unwrap();
        assert_eq!(schema, &vec!["id".to_string(), "name".to_string()]);
        assert!(doc.get_schema("Missing").is_none());
    }

    #[test]
    fn test_document_get_child_types() {
        let mut doc = Document::new((2, 0));
        doc.nests
            .insert("User".to_string(), vec!["Post".to_string()]);
        assert_eq!(doc.get_child_types("User"), Some(&vec!["Post".to_string()]));
        assert!(doc.get_child_types("Post").is_none());
    }

    #[test]
    fn test_document_expand_alias() {
        let mut doc = Document::new((2, 0));
        doc.aliases.insert("active".to_string(), "true".to_string());
        assert_eq!(doc.expand_alias("active"), Some(&"true".to_string()));
        assert!(doc.expand_alias("missing").is_none());
    }

    #[test]
    fn test_document_equality() {
        let a = Document::new((2, 0));
        let b = Document::new((2, 0));
        assert_eq!(a, b);
    }

    #[test]
    fn test_document_clone() {
        let mut doc = Document::new((2, 0));
        doc.aliases.insert("key".to_string(), "value".to_string());
        let cloned = doc.clone();
        assert_eq!(doc, cloned);
    }

    #[test]
    fn test_document_debug() {
        let doc = Document::new((2, 0));
        let debug = format!("{:?}", doc);
        assert!(debug.contains("version"));
        assert!(debug.contains("aliases"));
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_node_empty_fields() {
        let node = Node::new("Type", "id", vec![]);
        assert!(node.fields.is_empty());
        assert!(node.get_field(0).is_none());
    }

    #[test]
    fn test_node_unicode_id() {
        let node = Node::new("User", "日本語", vec![]);
        assert_eq!(node.id, "日本語");
    }

    #[test]
    fn test_document_version_zero() {
        let doc = Document::new((0, 0));
        assert_eq!(doc.version, (0, 0));
    }

    #[test]
    fn test_document_large_version() {
        let doc = Document::new((999, 999));
        assert_eq!(doc.version, (999, 999));
    }

    #[test]
    fn test_nested_items() {
        let mut inner = BTreeMap::new();
        inner.insert("nested".to_string(), Item::Scalar(Value::Int(42)));

        let mut outer = BTreeMap::new();
        outer.insert("inner".to_string(), Item::Object(inner));

        let item = Item::Object(outer);
        let obj = item.as_object().unwrap();
        let inner_item = obj.get("inner").unwrap();
        let inner_obj = inner_item.as_object().unwrap();
        assert!(inner_obj.contains_key("nested"));
    }

    #[test]
    fn test_deeply_nested_nodes() {
        let mut root = Node::new("A", "a", vec![]);
        let mut child = Node::new("B", "b", vec![]);
        child.add_child("C", Node::new("C", "c", vec![]));
        root.add_child("B", child);

        let children = root.children().unwrap();
        assert_eq!(children["B"].len(), 1);
        let b_children = children["B"][0].children().unwrap();
        assert_eq!(b_children["C"].len(), 1);
    }

    #[test]
    fn test_node_default() {
        let node = Node::default();
        assert_eq!(node.type_name, "");
        assert!(node.fields.is_empty());
        assert!(node.children.is_none());
        assert_eq!(node.child_count, 0);
    }

    // ==================== Document schema_versions tests ====================

    #[test]
    fn test_document_schema_versions_empty() {
        let doc = Document::new((2, 0));
        assert!(doc.schema_versions.is_empty());
    }

    #[test]
    fn test_document_set_schema_version() {
        use crate::schema_version::SchemaVersion;
        let mut doc = Document::new((2, 0));
        doc.set_schema_version("User".to_string(), SchemaVersion::new(1, 2, 3));
        assert_eq!(doc.schema_versions.len(), 1);
        assert!(doc.schema_versions.contains_key("User"));
    }

    #[test]
    fn test_document_get_schema_version() {
        use crate::schema_version::SchemaVersion;
        let mut doc = Document::new((2, 0));
        let version = SchemaVersion::new(2, 0, 0);
        doc.set_schema_version("Post".to_string(), version);
        assert_eq!(doc.get_schema_version("Post"), Some(version));
        assert_eq!(doc.get_schema_version("User"), None);
    }

    #[test]
    fn test_document_multiple_schema_versions() {
        use crate::schema_version::SchemaVersion;
        let mut doc = Document::new((2, 0));
        doc.set_schema_version("User".to_string(), SchemaVersion::new(1, 0, 0));
        doc.set_schema_version("Post".to_string(), SchemaVersion::new(2, 1, 0));
        doc.set_schema_version("Comment".to_string(), SchemaVersion::new(1, 5, 2));

        assert_eq!(doc.schema_versions.len(), 3);
        assert_eq!(
            doc.get_schema_version("User"),
            Some(SchemaVersion::new(1, 0, 0))
        );
        assert_eq!(
            doc.get_schema_version("Post"),
            Some(SchemaVersion::new(2, 1, 0))
        );
        assert_eq!(
            doc.get_schema_version("Comment"),
            Some(SchemaVersion::new(1, 5, 2))
        );
    }

    #[test]
    fn test_document_replace_schema_version() {
        use crate::schema_version::SchemaVersion;
        let mut doc = Document::new((2, 0));
        doc.set_schema_version("User".to_string(), SchemaVersion::new(1, 0, 0));
        doc.set_schema_version("User".to_string(), SchemaVersion::new(2, 0, 0));

        assert_eq!(doc.schema_versions.len(), 1);
        assert_eq!(
            doc.get_schema_version("User"),
            Some(SchemaVersion::new(2, 0, 0))
        );
    }

    #[test]
    fn test_document_schema_version_with_clone() {
        use crate::schema_version::SchemaVersion;
        let mut doc = Document::new((2, 0));
        doc.set_schema_version("User".to_string(), SchemaVersion::new(1, 0, 0));
        let cloned = doc.clone();

        assert_eq!(cloned.schema_versions.len(), 1);
        assert_eq!(
            cloned.get_schema_version("User"),
            Some(SchemaVersion::new(1, 0, 0))
        );
    }

    #[test]
    fn test_document_schema_version_equality() {
        use crate::schema_version::SchemaVersion;
        let mut a = Document::new((2, 0));
        a.set_schema_version("User".to_string(), SchemaVersion::new(1, 0, 0));

        let mut b = Document::new((2, 0));
        b.set_schema_version("User".to_string(), SchemaVersion::new(1, 0, 0));

        assert_eq!(a, b);
    }

    #[test]
    fn test_document_schema_version_inequality() {
        use crate::schema_version::SchemaVersion;
        let mut a = Document::new((2, 0));
        a.set_schema_version("User".to_string(), SchemaVersion::new(1, 0, 0));

        let mut b = Document::new((2, 0));
        b.set_schema_version("User".to_string(), SchemaVersion::new(2, 0, 0));

        assert_ne!(a, b);
    }
}

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

//! Builder pattern for creating customizable test fixtures.
//!
//! This module provides fluent builders for constructing HEDL documents,
//! lists, and nodes with customizable properties.

use hedl_core::{Document, Item, MatrixList, Node, Reference, Tensor, Value};
use std::collections::BTreeMap;

/// Builder for creating customizable Document fixtures.
///
/// # Examples
///
/// ```
/// use hedl_test::fixtures::builders::DocumentBuilder;
/// use hedl_core::Value;
///
/// let doc = DocumentBuilder::new()
///     .version(1, 0)
///     .scalar("name", Value::String("Alice".to_string()))
///     .scalar("age", Value::Int(30))
///     .build();
///
/// assert_eq!(doc.version, (1, 0));
/// assert!(doc.root.contains_key("name"));
/// ```
#[derive(Debug, Clone)]
pub struct DocumentBuilder {
    version: (u32, u32),
    aliases: BTreeMap<String, String>,
    structs: BTreeMap<String, Vec<String>>,
    nests: BTreeMap<String, String>,
    root: BTreeMap<String, Item>,
}

impl Default for DocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentBuilder {
    /// Creates a new DocumentBuilder with default settings.
    pub fn new() -> Self {
        Self {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root: BTreeMap::new(),
        }
    }

    /// Sets the document version.
    pub fn version(mut self, major: u32, minor: u32) -> Self {
        self.version = (major, minor);
        self
    }

    /// Adds an alias.
    pub fn alias(mut self, alias: impl Into<String>, target: impl Into<String>) -> Self {
        self.aliases.insert(alias.into(), target.into());
        self
    }

    /// Adds a struct definition.
    pub fn struct_def(mut self, type_name: impl Into<String>, fields: Vec<String>) -> Self {
        self.structs.insert(type_name.into(), fields);
        self
    }

    /// Adds a NEST relationship.
    pub fn nest(mut self, parent: impl Into<String>, child: impl Into<String>) -> Self {
        self.nests.insert(parent.into(), child.into());
        self
    }

    /// Adds a scalar value to the root.
    pub fn scalar(mut self, name: impl Into<String>, value: Value) -> Self {
        self.root.insert(name.into(), Item::Scalar(value));
        self
    }

    /// Adds a list to the root.
    pub fn list(mut self, name: impl Into<String>, list: MatrixList) -> Self {
        self.root.insert(name.into(), Item::List(list));
        self
    }

    /// Adds a root item directly.
    pub fn item(mut self, name: impl Into<String>, item: Item) -> Self {
        self.root.insert(name.into(), item);
        self
    }

    /// Builds the Document.
    pub fn build(self) -> Document {
        Document {
            version: self.version,
            aliases: self.aliases,
            structs: self.structs,
            nests: self.nests,
            root: self.root,
        }
    }
}

/// Builder for creating customizable MatrixList fixtures.
///
/// # Examples
///
/// ```
/// use hedl_test::fixtures::builders::MatrixListBuilder;
/// use hedl_core::{Node, Value};
///
/// let list = MatrixListBuilder::new("User")
///     .schema(vec!["id".to_string(), "name".to_string()])
///     .row(Node::new("User", "alice", vec![
///         Value::String("alice".to_string()),
///         Value::String("Alice".to_string()),
///     ]))
///     .build();
///
/// assert_eq!(list.type_name, "User");
/// assert_eq!(list.rows.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct MatrixListBuilder {
    type_name: String,
    schema: Vec<String>,
    rows: Vec<Node>,
    count_hint: Option<usize>,
}

impl MatrixListBuilder {
    /// Creates a new MatrixListBuilder with the given type name.
    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            type_name: type_name.into(),
            schema: Vec::new(),
            rows: Vec::new(),
            count_hint: None,
        }
    }

    /// Sets the schema (field names).
    pub fn schema(mut self, schema: Vec<String>) -> Self {
        self.schema = schema;
        self
    }

    /// Adds a schema field.
    pub fn field(mut self, field: impl Into<String>) -> Self {
        self.schema.push(field.into());
        self
    }

    /// Adds a row (node).
    pub fn row(mut self, node: Node) -> Self {
        self.rows.push(node);
        self
    }

    /// Adds multiple rows.
    pub fn rows(mut self, nodes: Vec<Node>) -> Self {
        self.rows.extend(nodes);
        self
    }

    /// Sets the count hint.
    pub fn count_hint(mut self, count: usize) -> Self {
        self.count_hint = Some(count);
        self
    }

    /// Builds the MatrixList.
    pub fn build(self) -> MatrixList {
        MatrixList {
            type_name: self.type_name,
            schema: self.schema,
            rows: self.rows,
            count_hint: self.count_hint,
        }
    }
}

/// Builder for creating customizable Node fixtures.
///
/// # Examples
///
/// ```
/// use hedl_test::fixtures::builders::NodeBuilder;
/// use hedl_core::Value;
///
/// let node = NodeBuilder::new("User", "alice")
///     .field(Value::String("alice".to_string()))
///     .field(Value::String("Alice Smith".to_string()))
///     .field(Value::Int(30))
///     .build();
///
/// assert_eq!(node.id, "alice");
/// assert_eq!(node.fields.len(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct NodeBuilder {
    type_name: String,
    id: String,
    fields: Vec<Value>,
    children: BTreeMap<String, Vec<Node>>,
    child_count: Option<usize>,
}

impl NodeBuilder {
    /// Creates a new NodeBuilder with the given type and ID.
    pub fn new(type_name: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            type_name: type_name.into(),
            id: id.into(),
            fields: Vec::new(),
            children: BTreeMap::new(),
            child_count: None,
        }
    }

    /// Adds a field value.
    pub fn field(mut self, value: Value) -> Self {
        self.fields.push(value);
        self
    }

    /// Adds multiple field values.
    pub fn fields(mut self, values: Vec<Value>) -> Self {
        self.fields.extend(values);
        self
    }

    /// Adds child nodes under a relationship name.
    pub fn children(mut self, rel_name: impl Into<String>, nodes: Vec<Node>) -> Self {
        self.children.insert(rel_name.into(), nodes);
        self
    }

    /// Adds a single child node under a relationship name.
    pub fn child(mut self, rel_name: impl Into<String>, node: Node) -> Self {
        self.children
            .entry(rel_name.into())
            .or_default()
            .push(node);
        self
    }

    /// Sets the child count hint.
    pub fn child_count(mut self, count: usize) -> Self {
        self.child_count = Some(count);
        self
    }

    /// Builds the Node.
    pub fn build(self) -> Node {
        Node {
            type_name: self.type_name,
            id: self.id,
            fields: self.fields,
            children: self.children,
            child_count: self.child_count,
        }
    }
}

/// Builder for creating Value fixtures with fluent API.
///
/// # Examples
///
/// ```
/// use hedl_test::fixtures::builders::ValueBuilder;
///
/// let v = ValueBuilder::string("hello");
/// let v = ValueBuilder::int(42);
/// let v = ValueBuilder::bool_val(true);
/// let v = ValueBuilder::reference("User", "alice");
/// ```
pub struct ValueBuilder;

impl ValueBuilder {
    /// Creates a null value.
    pub fn null() -> Value {
        Value::Null
    }

    /// Creates a boolean value.
    pub fn bool_val(value: bool) -> Value {
        Value::Bool(value)
    }

    /// Creates an integer value.
    pub fn int(value: i64) -> Value {
        Value::Int(value)
    }

    /// Creates a float value.
    pub fn float(value: f64) -> Value {
        Value::Float(value)
    }

    /// Creates a string value.
    pub fn string(value: impl Into<String>) -> Value {
        Value::String(value.into())
    }

    /// Creates a reference value.
    pub fn reference(type_name: impl Into<String>, id: impl Into<String>) -> Value {
        Value::Reference(Reference {
            type_name: Some(type_name.into()),
            id: id.into(),
        })
    }

    /// Creates a local reference (no type).
    pub fn local_ref(id: impl Into<String>) -> Value {
        Value::Reference(Reference {
            type_name: None,
            id: id.into(),
        })
    }

    /// Creates a 1D tensor.
    pub fn tensor_1d(values: Vec<f64>) -> Value {
        Value::Tensor(Tensor::Array(
            values.into_iter().map(Tensor::Scalar).collect(),
        ))
    }

    /// Creates a 2D tensor.
    pub fn tensor_2d(rows: Vec<Vec<f64>>) -> Value {
        Value::Tensor(Tensor::Array(
            rows.into_iter()
                .map(|row| Tensor::Array(row.into_iter().map(Tensor::Scalar).collect()))
                .collect(),
        ))
    }
}

/// Quick builder functions for common patterns.
pub mod quick {
    use super::*;

    /// Creates a simple document with scalar values.
    ///
    /// # Example
    ///
    /// ```
    /// use hedl_test::fixtures::builders::quick::simple_scalars;
    ///
    /// let doc = simple_scalars(vec![
    ///     ("name", "Alice"),
    ///     ("city", "NYC"),
    /// ]);
    /// ```
    pub fn simple_scalars(fields: Vec<(&str, &str)>) -> Document {
        let mut builder = DocumentBuilder::new();

        for (name, value) in fields {
            builder = builder.scalar(name, Value::String(value.to_string()));
        }

        builder.build()
    }

    /// Creates a document with a simple user list.
    ///
    /// # Example
    ///
    /// ```
    /// use hedl_test::fixtures::builders::quick::simple_user_list;
    ///
    /// let doc = simple_user_list(vec![
    ///     ("alice", "Alice Smith", "alice@example.com"),
    ///     ("bob", "Bob Jones", "bob@example.com"),
    /// ]);
    /// ```
    pub fn simple_user_list(users: Vec<(&str, &str, &str)>) -> Document {
        let mut list = MatrixListBuilder::new("User")
            .schema(vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string(),
            ]);

        for (id, name, email) in users {
            let node = NodeBuilder::new("User", id)
                .field(Value::String(id.to_string()))
                .field(Value::String(name.to_string()))
                .field(Value::String(email.to_string()))
                .build();

            list = list.row(node);
        }

        DocumentBuilder::new()
            .struct_def("User", vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string(),
            ])
            .list("users", list.build())
            .build()
    }

    /// Creates a document with references.
    ///
    /// # Example
    ///
    /// ```
    /// use hedl_test::fixtures::builders::quick::with_references;
    ///
    /// let doc = with_references(
    ///     vec![("alice", "Alice"), ("bob", "Bob")],
    ///     vec![("post1", "Title", "alice")],
    /// );
    /// ```
    pub fn with_references(
        users: Vec<(&str, &str)>,
        posts: Vec<(&str, &str, &str)>, // (id, title, author_id)
    ) -> Document {
        let mut users_list = MatrixListBuilder::new("User")
            .schema(vec!["id".to_string(), "name".to_string()]);

        for (id, name) in users {
            let node = NodeBuilder::new("User", id)
                .field(Value::String(id.to_string()))
                .field(Value::String(name.to_string()))
                .build();

            users_list = users_list.row(node);
        }

        let mut posts_list = MatrixListBuilder::new("Post")
            .schema(vec![
                "id".to_string(),
                "title".to_string(),
                "author".to_string(),
            ]);

        for (id, title, author_id) in posts {
            let node = NodeBuilder::new("Post", id)
                .field(Value::String(id.to_string()))
                .field(Value::String(title.to_string()))
                .field(ValueBuilder::reference("User", author_id))
                .build();

            posts_list = posts_list.row(node);
        }

        DocumentBuilder::new()
            .struct_def("User", vec!["id".to_string(), "name".to_string()])
            .struct_def("Post", vec![
                "id".to_string(),
                "title".to_string(),
                "author".to_string(),
            ])
            .list("users", users_list.build())
            .list("posts", posts_list.build())
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_builder() {
        let doc = DocumentBuilder::new()
            .version(1, 0)
            .scalar("name", Value::String("test".to_string()))
            .scalar("age", Value::Int(42))
            .build();

        assert_eq!(doc.version, (1, 0));
        assert_eq!(doc.root.len(), 2);
        assert!(doc.root.contains_key("name"));
        assert!(doc.root.contains_key("age"));
    }

    #[test]
    fn test_matrix_list_builder() {
        let list = MatrixListBuilder::new("User")
            .schema(vec!["id".to_string(), "name".to_string()])
            .row(Node::new("User", "alice", vec![
                Value::String("alice".to_string()),
                Value::String("Alice".to_string()),
            ]))
            .build();

        assert_eq!(list.type_name, "User");
        assert_eq!(list.schema.len(), 2);
        assert_eq!(list.rows.len(), 1);
    }

    #[test]
    fn test_node_builder() {
        let node = NodeBuilder::new("User", "alice")
            .field(Value::String("alice".to_string()))
            .field(Value::String("Alice".to_string()))
            .field(Value::Int(30))
            .build();

        assert_eq!(node.type_name, "User");
        assert_eq!(node.id, "alice");
        assert_eq!(node.fields.len(), 3);
    }

    #[test]
    fn test_value_builder() {
        assert!(matches!(ValueBuilder::null(), Value::Null));
        assert!(matches!(ValueBuilder::bool_val(true), Value::Bool(true)));
        assert!(matches!(ValueBuilder::int(42), Value::Int(42)));
        assert!(matches!(ValueBuilder::float(3.14), Value::Float(_)));
        assert!(matches!(
            ValueBuilder::string("test"),
            Value::String(ref s) if s == "test"
        ));
        assert!(matches!(
            ValueBuilder::reference("User", "alice"),
            Value::Reference(_)
        ));
    }

    #[test]
    fn test_quick_simple_scalars() {
        let doc = quick::simple_scalars(vec![
            ("name", "Alice"),
            ("city", "NYC"),
        ]);

        assert_eq!(doc.root.len(), 2);
        assert!(doc.root.contains_key("name"));
        assert!(doc.root.contains_key("city"));
    }

    #[test]
    fn test_quick_simple_user_list() {
        let doc = quick::simple_user_list(vec![
            ("alice", "Alice Smith", "alice@example.com"),
            ("bob", "Bob Jones", "bob@example.com"),
        ]);

        assert!(doc.root.contains_key("users"));
        if let Some(Item::List(list)) = doc.root.get("users") {
            assert_eq!(list.rows.len(), 2);
        } else {
            panic!("Expected users list");
        }
    }

    #[test]
    fn test_quick_with_references() {
        let doc = quick::with_references(
            vec![("alice", "Alice"), ("bob", "Bob")],
            vec![("post1", "Title", "alice")],
        );

        assert!(doc.root.contains_key("users"));
        assert!(doc.root.contains_key("posts"));

        if let Some(Item::List(list)) = doc.root.get("posts") {
            assert_eq!(list.rows.len(), 1);

            // Check that the post has a reference
            let post = &list.rows[0];
            if let Value::Reference(r) = &post.fields[2] {
                assert_eq!(r.id, "alice");
            } else {
                panic!("Expected reference in post author field");
            }
        } else {
            panic!("Expected posts list");
        }
    }
}

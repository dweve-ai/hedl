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

//! Event types for streaming parser.
//!
//! This module defines the events yielded by [`StreamingParser`](crate::StreamingParser)
//! and associated metadata structures.
//!
//! # Event Flow
//!
//! A typical HEDL document produces events in this order:
//!
//! 1. `ListStart` or `ObjectStart` - Container begins
//! 2. `Node` or `Scalar` - Data items
//! 3. `ListEnd` or `ObjectEnd` - Container ends
//! 4. (Repeat for additional containers)
//!
//! # Example Event Sequence
//!
//! For this HEDL document:
//!
//! ```text
//! %VERSION: 1.0
//! %STRUCT: User: [id, name]
//! ---
//! users: @User
//!   | alice, Alice
//!   | bob, Bob
//! ```
//!
//! The parser yields:
//!
//! ```text
//! ListStart { key: "users", type_name: "User", ... }
//! Node(NodeInfo { id: "alice", ... })
//! Node(NodeInfo { id: "bob", ... })
//! ListEnd { key: "users", count: 2, ... }
//! ```

use hedl_core::Value;
use std::collections::BTreeMap;

/// Header information parsed from the HEDL document.
///
/// Contains metadata extracted from header directives:
/// - `%VERSION`: Document format version
/// - `%STRUCT`: Schema definitions mapping type names to column lists
/// - `%ALIAS`: Variable substitutions
/// - `%NEST`: Parent-child relationship rules
///
/// # Examples
///
/// ```rust
/// use hedl_stream::{StreamingParser, HeaderInfo};
/// use std::io::Cursor;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let input = r#"
/// %VERSION: 1.0
/// %STRUCT: User: [id, name, email]
/// %ALIAS: admin = "Administrator"
/// %NEST: User > Order
/// ---
/// "#;
///
/// let parser = StreamingParser::new(Cursor::new(input))?;
/// let header = parser.header().unwrap();
///
/// // Access version
/// let (major, minor) = header.version;
/// println!("HEDL version {}.{}", major, minor);
///
/// // Look up schema
/// if let Some(schema) = header.get_schema("User") {
///     println!("User has {} fields", schema.len());
/// }
///
/// // Check for alias
/// if let Some(value) = header.aliases.get("admin") {
///     println!("Alias 'admin' expands to '{}'", value);
/// }
///
/// // Check nesting rule
/// if let Some(child) = header.get_child_type("User") {
///     println!("User can contain {}", child);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct HeaderInfo {
    /// HEDL version (major, minor).
    pub version: (u32, u32),
    /// Schema definitions: type -> columns.
    pub structs: BTreeMap<String, Vec<String>>,
    /// Alias definitions.
    pub aliases: BTreeMap<String, String>,
    /// Nest relationships: parent -> child.
    pub nests: BTreeMap<String, String>,
}

impl HeaderInfo {
    /// Create a new empty header.
    pub fn new() -> Self {
        Self {
            version: (1, 0),
            structs: BTreeMap::new(),
            aliases: BTreeMap::new(),
            nests: BTreeMap::new(),
        }
    }

    /// Get schema columns for a type.
    #[inline]
    pub fn get_schema(&self, type_name: &str) -> Option<&Vec<String>> {
        self.structs.get(type_name)
    }

    /// Get child type for a parent type (from NEST).
    #[inline]
    pub fn get_child_type(&self, parent_type: &str) -> Option<&String> {
        self.nests.get(parent_type)
    }
}

impl Default for HeaderInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a parsed node (entity/row).
///
/// Represents a single entity parsed from a HEDL matrix row. Contains the
/// entity's type, ID, field values, and parent relationship information.
///
/// # Field Access
///
/// Fields can be accessed by index using [`get_field()`](Self::get_field).
/// The first field (index 0) is always the ID.
///
/// # Examples
///
/// ## Accessing Fields
///
/// ```rust
/// use hedl_stream::{StreamingParser, NodeEvent};
/// use hedl_core::Value;
/// use std::io::Cursor;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let input = r#"
/// %VERSION: 1.0
/// %STRUCT: User: [id, name, email, active]
/// ---
/// users: @User
///   | alice, Alice Smith, alice@example.com, true
/// "#;
///
/// let parser = StreamingParser::new(Cursor::new(input))?;
///
/// for event in parser {
///     if let Ok(NodeEvent::Node(node)) = event {
///         // Access by index
///         assert_eq!(node.get_field(0), Some(&Value::String("alice".to_string())));
///         assert_eq!(node.get_field(1), Some(&Value::String("Alice Smith".to_string())));
///         assert_eq!(node.get_field(3), Some(&Value::Bool(true)));
///
///         // Or use the id field directly
///         assert_eq!(node.id, "alice");
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Checking Parent Relationships
///
/// ```rust
/// use hedl_stream::{StreamingParser, NodeEvent};
/// use std::io::Cursor;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let input = r#"
/// %VERSION: 1.0
/// %STRUCT: User: [id, name]
/// %STRUCT: Order: [id, amount]
/// %NEST: User > Order
/// ---
/// users: @User
///   | alice, Alice
///     | order1, 100.00
/// "#;
///
/// let parser = StreamingParser::new(Cursor::new(input))?;
///
/// for event in parser.filter_map(|e| e.ok()) {
///     if let NodeEvent::Node(node) = event {
///         if node.is_nested() {
///             println!("{} is a child of {:?}",
///                 node.id, node.parent_id.as_ref().unwrap());
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// The entity type name.
    pub type_name: String,
    /// The entity ID.
    pub id: String,
    /// Field values aligned with schema.
    pub fields: Vec<Value>,
    /// Nesting depth (0 = top-level).
    pub depth: usize,
    /// Parent node ID (if nested).
    pub parent_id: Option<String>,
    /// Parent type name (if nested).
    pub parent_type: Option<String>,
    /// Line number in source.
    pub line: usize,
}

impl NodeInfo {
    /// Create a new node info.
    pub fn new(
        type_name: String,
        id: String,
        fields: Vec<Value>,
        depth: usize,
        line: usize,
    ) -> Self {
        Self {
            type_name,
            id,
            fields,
            depth,
            parent_id: None,
            parent_type: None,
            line,
        }
    }

    /// Set parent information.
    pub fn with_parent(mut self, parent_type: String, parent_id: String) -> Self {
        self.parent_type = Some(parent_type);
        self.parent_id = Some(parent_id);
        self
    }

    /// Get a field value by column index.
    #[inline]
    pub fn get_field(&self, index: usize) -> Option<&Value> {
        self.fields.get(index)
    }

    /// Check if this is a nested (child) node.
    #[inline]
    pub fn is_nested(&self) -> bool {
        self.depth > 0 || self.parent_id.is_some()
    }
}

/// Event emitted by the streaming parser.
#[derive(Debug, Clone)]
pub enum NodeEvent {
    /// Header has been parsed.
    Header(HeaderInfo),

    /// Start of a new list.
    ListStart {
        /// Key name for the list.
        key: String,
        /// Type name.
        type_name: String,
        /// Schema columns.
        schema: Vec<String>,
        /// Line number.
        line: usize,
    },

    /// A node/row has been parsed.
    Node(NodeInfo),

    /// End of a list.
    ListEnd {
        /// Key name for the list.
        key: String,
        /// Type name.
        type_name: String,
        /// Number of nodes in the list.
        count: usize,
    },

    /// A scalar key-value pair.
    Scalar {
        /// Key name.
        key: String,
        /// Value.
        value: Value,
        /// Line number.
        line: usize,
    },

    /// Start of an object.
    ObjectStart {
        /// Key name.
        key: String,
        /// Line number.
        line: usize,
    },

    /// End of an object.
    ObjectEnd {
        /// Key name.
        key: String,
    },

    /// End of document.
    EndOfDocument,
}

impl NodeEvent {
    /// Check if this is a node event.
    #[inline]
    pub fn is_node(&self) -> bool {
        matches!(self, Self::Node(_))
    }

    /// Get the node info if this is a node event.
    #[inline]
    pub fn as_node(&self) -> Option<&NodeInfo> {
        match self {
            Self::Node(info) => Some(info),
            _ => None,
        }
    }

    /// Get the line number for this event.
    #[inline]
    pub fn line(&self) -> Option<usize> {
        match self {
            Self::Header(_) => Some(1),
            Self::ListStart { line, .. } => Some(*line),
            Self::Node(info) => Some(info.line),
            Self::Scalar { line, .. } => Some(*line),
            Self::ObjectStart { line, .. } => Some(*line),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== HeaderInfo tests ====================

    #[test]
    fn test_header_info_new() {
        let header = HeaderInfo::new();
        assert_eq!(header.version, (1, 0));
        assert!(header.structs.is_empty());
        assert!(header.aliases.is_empty());
        assert!(header.nests.is_empty());
    }

    #[test]
    fn test_header_info_default() {
        let header = HeaderInfo::default();
        assert_eq!(header.version, (1, 0));
        assert!(header.structs.is_empty());
    }

    #[test]
    fn test_header_info_get_schema() {
        let mut header = HeaderInfo::new();
        header.structs.insert(
            "User".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );

        assert_eq!(
            header.get_schema("User"),
            Some(&vec!["id".to_string(), "name".to_string()])
        );
        assert_eq!(header.get_schema("Unknown"), None);
    }

    #[test]
    fn test_header_info_get_child_type() {
        let mut header = HeaderInfo::new();
        header.nests.insert("User".to_string(), "Order".to_string());

        assert_eq!(header.get_child_type("User"), Some(&"Order".to_string()));
        assert_eq!(header.get_child_type("Product"), None);
    }

    #[test]
    fn test_header_info_multiple_structs() {
        let mut header = HeaderInfo::new();
        header
            .structs
            .insert("User".to_string(), vec!["id".to_string()]);
        header.structs.insert(
            "Product".to_string(),
            vec!["id".to_string(), "price".to_string()],
        );
        header.structs.insert(
            "Order".to_string(),
            vec!["id".to_string(), "user".to_string(), "product".to_string()],
        );

        assert_eq!(header.structs.len(), 3);
        assert_eq!(header.get_schema("User").unwrap().len(), 1);
        assert_eq!(header.get_schema("Product").unwrap().len(), 2);
        assert_eq!(header.get_schema("Order").unwrap().len(), 3);
    }

    #[test]
    fn test_header_info_multiple_aliases() {
        let mut header = HeaderInfo::new();
        header
            .aliases
            .insert("active".to_string(), "Active".to_string());
        header
            .aliases
            .insert("inactive".to_string(), "Inactive".to_string());

        assert_eq!(header.aliases.len(), 2);
        assert_eq!(header.aliases.get("active"), Some(&"Active".to_string()));
    }

    #[test]
    fn test_header_info_multiple_nests() {
        let mut header = HeaderInfo::new();
        header.nests.insert("User".to_string(), "Order".to_string());
        header
            .nests
            .insert("Order".to_string(), "LineItem".to_string());

        assert_eq!(header.nests.len(), 2);
        assert_eq!(header.get_child_type("User"), Some(&"Order".to_string()));
        assert_eq!(
            header.get_child_type("Order"),
            Some(&"LineItem".to_string())
        );
    }

    #[test]
    fn test_header_info_clone() {
        let mut header = HeaderInfo::new();
        header.version = (2, 1);
        header
            .structs
            .insert("Test".to_string(), vec!["col".to_string()]);

        let cloned = header.clone();
        assert_eq!(cloned.version, (2, 1));
        assert_eq!(cloned.structs.get("Test"), Some(&vec!["col".to_string()]));
    }

    #[test]
    fn test_header_info_debug() {
        let header = HeaderInfo::new();
        let debug = format!("{:?}", header);
        assert!(debug.contains("HeaderInfo"));
        assert!(debug.contains("version"));
    }

    // ==================== NodeInfo tests ====================

    #[test]
    fn test_node_info_new() {
        let node = NodeInfo::new(
            "User".to_string(),
            "alice".to_string(),
            vec![
                Value::String("alice".to_string()),
                Value::String("Alice".to_string()),
            ],
            0,
            10,
        );

        assert_eq!(node.type_name, "User");
        assert_eq!(node.id, "alice");
        assert_eq!(node.fields.len(), 2);
        assert_eq!(node.depth, 0);
        assert_eq!(node.line, 10);
        assert_eq!(node.parent_id, None);
        assert_eq!(node.parent_type, None);
    }

    #[test]
    fn test_node_info_with_parent() {
        let node = NodeInfo::new(
            "Order".to_string(),
            "order1".to_string(),
            vec![Value::String("order1".to_string())],
            1,
            15,
        )
        .with_parent("User".to_string(), "alice".to_string());

        assert_eq!(node.parent_type, Some("User".to_string()));
        assert_eq!(node.parent_id, Some("alice".to_string()));
    }

    #[test]
    fn test_node_info_get_field() {
        let node = NodeInfo::new(
            "Data".to_string(),
            "row1".to_string(),
            vec![
                Value::String("row1".to_string()),
                Value::Int(42),
                Value::Bool(true),
            ],
            0,
            5,
        );

        assert_eq!(node.get_field(0), Some(&Value::String("row1".to_string())));
        assert_eq!(node.get_field(1), Some(&Value::Int(42)));
        assert_eq!(node.get_field(2), Some(&Value::Bool(true)));
        assert_eq!(node.get_field(3), None);
        assert_eq!(node.get_field(100), None);
    }

    #[test]
    fn test_node_info_is_nested_by_depth() {
        let nested = NodeInfo::new("Child".to_string(), "c1".to_string(), vec![], 1, 10);
        assert!(nested.is_nested());

        let top_level = NodeInfo::new("Parent".to_string(), "p1".to_string(), vec![], 0, 5);
        assert!(!top_level.is_nested());
    }

    #[test]
    fn test_node_info_is_nested_by_parent() {
        let with_parent = NodeInfo::new("Child".to_string(), "c1".to_string(), vec![], 0, 10)
            .with_parent("Parent".to_string(), "p1".to_string());
        assert!(with_parent.is_nested());
    }

    #[test]
    fn test_node_info_clone() {
        let node = NodeInfo::new(
            "User".to_string(),
            "alice".to_string(),
            vec![Value::String("alice".to_string())],
            0,
            1,
        );
        let cloned = node.clone();

        assert_eq!(cloned.type_name, "User");
        assert_eq!(cloned.id, "alice");
    }

    #[test]
    fn test_node_info_debug() {
        let node = NodeInfo::new("User".to_string(), "alice".to_string(), vec![], 0, 1);
        let debug = format!("{:?}", node);
        assert!(debug.contains("NodeInfo"));
        assert!(debug.contains("User"));
        assert!(debug.contains("alice"));
    }

    #[test]
    fn test_node_info_empty_fields() {
        let node = NodeInfo::new("Empty".to_string(), "e1".to_string(), vec![], 0, 1);
        assert!(node.fields.is_empty());
        assert_eq!(node.get_field(0), None);
    }

    #[test]
    fn test_node_info_all_value_types() {
        let node = NodeInfo::new(
            "AllTypes".to_string(),
            "test".to_string(),
            vec![
                Value::Null,
                Value::Bool(true),
                Value::Int(-42),
                Value::Float(3.5),
                Value::String("hello".to_string()),
            ],
            0,
            1,
        );

        assert_eq!(node.get_field(0), Some(&Value::Null));
        assert_eq!(node.get_field(1), Some(&Value::Bool(true)));
        assert_eq!(node.get_field(2), Some(&Value::Int(-42)));
        assert_eq!(node.get_field(3), Some(&Value::Float(3.5)));
        assert_eq!(node.get_field(4), Some(&Value::String("hello".to_string())));
    }

    // ==================== NodeEvent tests ====================

    #[test]
    fn test_node_event_is_node() {
        let node_info = NodeInfo::new("User".to_string(), "a".to_string(), vec![], 0, 1);
        let node_event = NodeEvent::Node(node_info);
        assert!(node_event.is_node());

        let header_event = NodeEvent::Header(HeaderInfo::new());
        assert!(!header_event.is_node());

        let list_start = NodeEvent::ListStart {
            key: "users".to_string(),
            type_name: "User".to_string(),
            schema: vec![],
            line: 1,
        };
        assert!(!list_start.is_node());
    }

    #[test]
    fn test_node_event_as_node() {
        let node_info = NodeInfo::new("User".to_string(), "alice".to_string(), vec![], 0, 5);
        let node_event = NodeEvent::Node(node_info);

        let extracted = node_event.as_node().unwrap();
        assert_eq!(extracted.id, "alice");

        let header_event = NodeEvent::Header(HeaderInfo::new());
        assert!(header_event.as_node().is_none());
    }

    #[test]
    fn test_node_event_line_header() {
        let event = NodeEvent::Header(HeaderInfo::new());
        assert_eq!(event.line(), Some(1));
    }

    #[test]
    fn test_node_event_line_list_start() {
        let event = NodeEvent::ListStart {
            key: "users".to_string(),
            type_name: "User".to_string(),
            schema: vec![],
            line: 42,
        };
        assert_eq!(event.line(), Some(42));
    }

    #[test]
    fn test_node_event_line_node() {
        let node = NodeInfo::new("User".to_string(), "a".to_string(), vec![], 0, 100);
        let event = NodeEvent::Node(node);
        assert_eq!(event.line(), Some(100));
    }

    #[test]
    fn test_node_event_line_scalar() {
        let event = NodeEvent::Scalar {
            key: "name".to_string(),
            value: Value::String("test".to_string()),
            line: 25,
        };
        assert_eq!(event.line(), Some(25));
    }

    #[test]
    fn test_node_event_line_object_start() {
        let event = NodeEvent::ObjectStart {
            key: "config".to_string(),
            line: 50,
        };
        assert_eq!(event.line(), Some(50));
    }

    #[test]
    fn test_node_event_line_list_end() {
        let event = NodeEvent::ListEnd {
            key: "users".to_string(),
            type_name: "User".to_string(),
            count: 10,
        };
        assert_eq!(event.line(), None);
    }

    #[test]
    fn test_node_event_line_object_end() {
        let event = NodeEvent::ObjectEnd {
            key: "config".to_string(),
        };
        assert_eq!(event.line(), None);
    }

    #[test]
    fn test_node_event_line_end_of_document() {
        let event = NodeEvent::EndOfDocument;
        assert_eq!(event.line(), None);
    }

    #[test]
    fn test_node_event_clone() {
        let event = NodeEvent::Scalar {
            key: "key".to_string(),
            value: Value::Int(42),
            line: 10,
        };
        let cloned = event.clone();

        if let NodeEvent::Scalar { key, value, line } = cloned {
            assert_eq!(key, "key");
            assert_eq!(value, Value::Int(42));
            assert_eq!(line, 10);
        } else {
            panic!("Expected Scalar");
        }
    }

    #[test]
    fn test_node_event_debug() {
        let event = NodeEvent::EndOfDocument;
        let debug = format!("{:?}", event);
        assert!(debug.contains("EndOfDocument"));
    }

    #[test]
    fn test_node_event_list_start_fields() {
        let event = NodeEvent::ListStart {
            key: "users".to_string(),
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            line: 5,
        };

        if let NodeEvent::ListStart {
            key,
            type_name,
            schema,
            line,
        } = event
        {
            assert_eq!(key, "users");
            assert_eq!(type_name, "User");
            assert_eq!(schema, vec!["id".to_string(), "name".to_string()]);
            assert_eq!(line, 5);
        }
    }

    #[test]
    fn test_node_event_list_end_fields() {
        let event = NodeEvent::ListEnd {
            key: "products".to_string(),
            type_name: "Product".to_string(),
            count: 100,
        };

        if let NodeEvent::ListEnd {
            key,
            type_name,
            count,
        } = event
        {
            assert_eq!(key, "products");
            assert_eq!(type_name, "Product");
            assert_eq!(count, 100);
        }
    }

    #[test]
    fn test_node_event_scalar_all_value_types() {
        let events = [
            NodeEvent::Scalar {
                key: "null".to_string(),
                value: Value::Null,
                line: 1,
            },
            NodeEvent::Scalar {
                key: "bool".to_string(),
                value: Value::Bool(true),
                line: 2,
            },
            NodeEvent::Scalar {
                key: "int".to_string(),
                value: Value::Int(-100),
                line: 3,
            },
            NodeEvent::Scalar {
                key: "float".to_string(),
                value: Value::Float(2.75),
                line: 4,
            },
            NodeEvent::Scalar {
                key: "string".to_string(),
                value: Value::String("text".to_string()),
                line: 5,
            },
        ];

        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.line(), Some(i + 1));
            assert!(!event.is_node());
        }
    }
}

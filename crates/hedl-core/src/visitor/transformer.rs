// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Transformer visitor that consumes and rebuilds trees.

use crate::visitor::VisitorContext;
use crate::{Document, Item, Node, Value};

/// Transformer visitor that consumes and rebuilds trees.
///
/// Useful for structural transformations that need to create new nodes
/// or dramatically restructure the tree. Unlike `VisitorMut` which modifies
/// in-place, transformers consume the original tree and produce a new one.
///
/// # Ownership Model
///
/// - Takes ownership of values during transformation
/// - Returns `None` to filter out elements
/// - Returns modified values to transform them
/// - Can create entirely new structures
///
/// # Example: Filter Nodes by Type
///
/// ```
/// use hedl_core::visitor::{Transformer, VisitorContext};
/// use hedl_core::Node;
///
/// struct UserFilter;
///
/// impl Transformer for UserFilter {
///     fn transform_node(
///         &mut self,
///         node: Node,
///         _ctx: &VisitorContext<'_>,
///     ) -> Option<Node> {
///         if node.type_name == "User" {
///             Some(node)
///         } else {
///             None  // Filter out non-User nodes
///         }
///     }
/// }
/// ```
///
/// # Example: Rename Fields
///
/// ```
/// use hedl_core::visitor::{Transformer, VisitorContext};
/// use hedl_core::{Item, Value};
///
/// struct FieldRenamer {
///     old_name: String,
///     new_name: String,
/// }
///
/// impl Transformer for FieldRenamer {
///     fn transform_item(
///         &mut self,
///         key: String,
///         item: Item,
///         _ctx: &VisitorContext<'_>,
///     ) -> Option<(String, Item)> {
///         let new_key = if key == self.old_name {
///             self.new_name.clone()
///         } else {
///             key
///         };
///         Some((new_key, item))
///     }
/// }
/// ```
pub trait Transformer {
    /// Transform the entire document.
    ///
    /// Default implementation returns the document unchanged.
    /// Override to perform document-level transformations.
    fn transform_document(&mut self, doc: Document, _ctx: &VisitorContext<'_>) -> Document {
        doc
    }

    /// Transform a single item.
    ///
    /// # Arguments
    ///
    /// - `key`: The key for this item
    /// - `item`: The item to transform
    /// - `ctx`: Visitor context
    ///
    /// # Returns
    ///
    /// - `Some((new_key, new_item))` to keep the item (possibly modified)
    /// - `None` to filter out the item
    fn transform_item(
        &mut self,
        key: String,
        item: Item,
        _ctx: &VisitorContext<'_>,
    ) -> Option<(String, Item)> {
        Some((key, item))
    }

    /// Transform a node.
    ///
    /// # Arguments
    ///
    /// - `node`: The node to transform
    /// - `ctx`: Visitor context
    ///
    /// # Returns
    ///
    /// - `Some(new_node)` to keep the node (possibly modified)
    /// - `None` to filter out the node
    fn transform_node(&mut self, node: Node, _ctx: &VisitorContext<'_>) -> Option<Node> {
        Some(node)
    }

    /// Transform a value.
    ///
    /// # Arguments
    ///
    /// - `value`: The value to transform
    /// - `ctx`: Visitor context
    ///
    /// # Returns
    ///
    /// Transformed value (cannot filter scalars, use Item transformation instead).
    fn transform_value(&mut self, value: Value, _ctx: &VisitorContext<'_>) -> Value {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoOpTransformer;
    impl Transformer for NoOpTransformer {}

    #[test]
    fn test_default_implementations_preserve_values() {
        let mut transformer = NoOpTransformer;
        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        // Test document transformation
        let doc2 = Document::new((1, 0));
        let transformed = transformer.transform_document(doc2, &ctx);
        assert_eq!(transformed.version, (1, 0));

        // Test item transformation
        let item = Item::Scalar(Value::Int(42));
        let result = transformer.transform_item("key".to_string(), item.clone(), &ctx);
        assert_eq!(result, Some(("key".to_string(), item)));

        // Test node transformation
        let node = Node::new("User", "1", vec![]);
        let result = transformer.transform_node(node.clone(), &ctx);
        assert_eq!(result, Some(node));

        // Test value transformation
        let value = Value::Int(42);
        let result = transformer.transform_value(value.clone(), &ctx);
        assert_eq!(result, value);
    }

    struct ValueDoubler;

    impl Transformer for ValueDoubler {
        fn transform_value(&mut self, value: Value, _ctx: &VisitorContext<'_>) -> Value {
            match value {
                Value::Int(n) => Value::Int(n * 2),
                other => other,
            }
        }
    }

    #[test]
    fn test_transformer_can_modify_values() {
        let mut transformer = ValueDoubler;
        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        let value = Value::Int(21);
        let result = transformer.transform_value(value, &ctx);
        assert_eq!(result, Value::Int(42));

        let value = Value::String("test".into());
        let result = transformer.transform_value(value.clone(), &ctx);
        assert_eq!(result, value);
    }

    struct NodeFilter {
        allowed_type: String,
    }

    impl Transformer for NodeFilter {
        fn transform_node(&mut self, node: Node, _ctx: &VisitorContext<'_>) -> Option<Node> {
            if node.type_name == self.allowed_type {
                Some(node)
            } else {
                None
            }
        }
    }

    #[test]
    fn test_transformer_can_filter_nodes() {
        let mut transformer = NodeFilter {
            allowed_type: "User".to_string(),
        };
        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        let user_node = Node::new("User", "1", vec![]);
        let result = transformer.transform_node(user_node.clone(), &ctx);
        assert_eq!(result, Some(user_node));

        let post_node = Node::new("Post", "1", vec![]);
        let result = transformer.transform_node(post_node, &ctx);
        assert_eq!(result, None);
    }

    struct KeyRenamer {
        old_key: String,
        new_key: String,
    }

    impl Transformer for KeyRenamer {
        fn transform_item(
            &mut self,
            key: String,
            item: Item,
            _ctx: &VisitorContext<'_>,
        ) -> Option<(String, Item)> {
            let new_key = if key == self.old_key {
                self.new_key.clone()
            } else {
                key
            };
            Some((new_key, item))
        }
    }

    #[test]
    fn test_transformer_can_rename_keys() {
        let mut transformer = KeyRenamer {
            old_key: "old_name".to_string(),
            new_key: "new_name".to_string(),
        };
        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        let item = Item::Scalar(Value::Int(42));

        let result = transformer.transform_item("old_name".to_string(), item.clone(), &ctx);
        assert_eq!(result, Some(("new_name".to_string(), item.clone())));

        let result = transformer.transform_item("other".to_string(), item.clone(), &ctx);
        assert_eq!(result, Some(("other".to_string(), item)));
    }

    struct ItemFilter;

    impl Transformer for ItemFilter {
        fn transform_item(
            &mut self,
            key: String,
            item: Item,
            _ctx: &VisitorContext<'_>,
        ) -> Option<(String, Item)> {
            match &item {
                Item::Scalar(Value::Null) => None, // Filter out null scalars
                _ => Some((key, item)),
            }
        }
    }

    #[test]
    fn test_transformer_can_filter_items() {
        let mut transformer = ItemFilter;
        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        let null_item = Item::Scalar(Value::Null);
        let result = transformer.transform_item("key".to_string(), null_item, &ctx);
        assert_eq!(result, None);

        let int_item = Item::Scalar(Value::Int(42));
        let result = transformer.transform_item("key".to_string(), int_item.clone(), &ctx);
        assert_eq!(result, Some(("key".to_string(), int_item)));
    }

    struct NodeIdPrefixer {
        prefix: String,
    }

    impl Transformer for NodeIdPrefixer {
        fn transform_node(&mut self, mut node: Node, _ctx: &VisitorContext<'_>) -> Option<Node> {
            node.id = format!("{}_{}", self.prefix, node.id);
            Some(node)
        }
    }

    #[test]
    fn test_transformer_can_modify_node_structure() {
        let mut transformer = NodeIdPrefixer {
            prefix: "tenant1".to_string(),
        };
        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        let node = Node::new("User", "alice", vec![]);
        let result = transformer.transform_node(node, &ctx).unwrap();
        assert_eq!(result.id, "tenant1_alice");
    }

    struct DocumentVersionUpdater {
        new_version: (u32, u32),
    }

    impl Transformer for DocumentVersionUpdater {
        fn transform_document(&mut self, mut doc: Document, _ctx: &VisitorContext<'_>) -> Document {
            doc.version = self.new_version;
            doc
        }
    }

    #[test]
    fn test_transformer_can_modify_document() {
        let mut transformer = DocumentVersionUpdater {
            new_version: (2, 0),
        };
        let doc = Document::new((1, 0));
        let temp_doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&temp_doc);

        let result = transformer.transform_document(doc, &ctx);
        assert_eq!(result.version, (2, 0));
    }
}

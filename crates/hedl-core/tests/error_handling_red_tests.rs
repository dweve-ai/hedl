// Error handling verification tests
//
// These tests verify that library functions return proper errors (Result::Err
// or Option::None) instead of panicking. Each test confirms that invalid
// inputs or exceeded limits produce graceful error responses.

use hedl_core::{
    reference::{register_node, resolve_references, ReferenceMode, TypeRegistry},
    Document, Item, Limits, MatrixList, Node, Value,
};

#[test]
fn test_type_registry_exceeds_max_total_ids_limit() {
    let mut registry = TypeRegistry::new();
    let limits = Limits {
        max_total_ids: 2, // Very small limit
        ..Default::default()
    };

    // First two should succeed
    registry.register("User", "id1", 1, &limits).unwrap();
    registry.register("User", "id2", 2, &limits).unwrap();

    // Third should return error
    let result = registry.register("User", "id3", 3, &limits);
    assert!(
        result.is_err(),
        "Expected error when exceeding max_total_ids limit"
    );
}

#[test]
fn test_register_node_duplicate_id_in_same_type() {
    let mut registry = TypeRegistry::new();
    let limits = Limits::default();

    register_node(&mut registry, "User", "alice", 1, &limits).unwrap();

    // Duplicate ID should return error
    let result = register_node(&mut registry, "User", "alice", 5, &limits);
    assert!(
        result.is_err(),
        "Expected error for duplicate ID registration"
    );
}

#[test]
fn test_resolve_references_strict_mode_unresolved_reference() {
    let mut doc = Document::new((2, 0));

    // Create a schema
    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    // Create a list with a reference to non-existent ID
    let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    list.add_row(Node::new(
        "User",
        "alice",
        vec![Value::String("Alice".to_string().into())],
    ));
    list.add_row(Node::new(
        "User",
        "bob",
        vec![Value::Reference(hedl_core::Reference::local("charlie"))], // Unresolved reference
    ));

    doc.root.insert("users".to_string(), Item::List(list));

    // Strict mode should return error for unresolved reference
    let result = resolve_references(&doc, ReferenceMode::Strict);
    assert!(
        result.is_err(),
        "Expected error for unresolved reference in strict mode"
    );
}

#[test]
fn test_resolve_references_ambiguous_reference() {
    let mut doc = Document::new((2, 0));

    // Create two types with the same ID
    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    doc.structs.insert(
        "Admin".to_string(),
        vec!["id".to_string(), "role".to_string()],
    );

    // Create first list with "alice" ID
    let mut users = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    users.add_row(Node::new(
        "User",
        "alice",
        vec![Value::String("Alice User".to_string().into())],
    ));
    doc.root.insert("users".to_string(), Item::List(users));

    // Create second list with duplicate "alice" ID
    let mut admins = MatrixList::new("Admin", vec!["id".to_string(), "role".to_string()]);
    admins.add_row(Node::new(
        "Admin",
        "alice",
        vec![Value::String("admin".to_string().into())],
    ));
    doc.root.insert("admins".to_string(), Item::List(admins));

    // Create reference to ambiguous ID
    let mut refs = MatrixList::new("Ref", vec!["id".to_string(), "ref".to_string()]);
    refs.add_row(Node::new(
        "Ref",
        "ref1",
        vec![Value::Reference(hedl_core::Reference::local("alice"))], // Ambiguous
    ));
    doc.root.insert("refs".to_string(), Item::List(refs));

    // Should return error for ambiguous reference
    let result = resolve_references(&doc, ReferenceMode::Strict);
    assert!(
        result.is_err(),
        "Expected error for ambiguous reference in strict mode"
    );
}

#[test]
fn test_document_get_nonexistent_key_returns_none() {
    let doc = Document::new((2, 0));

    // Getting non-existent key should return None
    assert!(
        doc.get("nonexistent").is_none(),
        "Expected None for non-existent key"
    );
}

#[test]
fn test_item_as_list_on_non_list_returns_none() {
    let item = Item::Scalar(Value::Int(42));

    // Converting non-list item to list should return None
    assert!(item.as_list().is_none(), "Expected None for non-list item");
}

#[test]
fn test_item_as_object_on_non_object_returns_none() {
    let item = Item::Scalar(Value::String("hello".to_string().into()));

    // Converting non-object item to object should return None
    assert!(
        item.as_object().is_none(),
        "Expected None for non-object item"
    );
}

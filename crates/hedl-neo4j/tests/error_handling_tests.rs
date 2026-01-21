// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive error handling tests.

use hedl_neo4j::error::Neo4jError;

#[test]
fn test_error_missing_schema() {
    let err = Neo4jError::MissingSchema("User".to_string());
    let msg = err.to_string();
    assert!(msg.contains("User"));
    assert!(msg.contains("schema"));
}

#[test]
fn test_error_invalid_reference() {
    let err = Neo4jError::InvalidReference("@Invalid:Ref".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Invalid:Ref"));
    assert!(msg.contains("reference"));
}

#[test]
fn test_error_unresolved_reference_with_type() {
    let err = Neo4jError::UnresolvedReference {
        type_name: Some("User".to_string()),
        id: "alice".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("User"));
    assert!(msg.contains("alice"));
    assert!(msg.contains("unresolved"));
}

#[test]
fn test_error_unresolved_reference_without_type() {
    let err = Neo4jError::UnresolvedReference {
        type_name: None,
        id: "orphan_id".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("orphan_id"));
    assert!(msg.contains("unresolved"));
}

#[test]
fn test_error_invalid_node_id() {
    let err = Neo4jError::InvalidNodeId("expected string, got null".to_string());
    let msg = err.to_string();
    assert!(msg.contains("expected string"));
    assert!(msg.contains("got null"));
}

#[test]
fn test_error_empty_matrix_list() {
    let err = Neo4jError::EmptyMatrixList("User".to_string());
    let msg = err.to_string();
    assert!(msg.contains("User"));
    assert!(msg.contains("empty"));
}

#[test]
fn test_error_inconsistent_data() {
    let err = Neo4jError::InconsistentData("row count mismatch".to_string());
    let msg = err.to_string();
    assert!(msg.contains("row count mismatch"));
    assert!(msg.contains("inconsistent"));
}

#[test]
fn test_error_invalid_identifier() {
    let err = Neo4jError::InvalidIdentifier("123invalid".to_string());
    let msg = err.to_string();
    assert!(msg.contains("123invalid"));
    assert!(msg.contains("identifier"));
}

#[test]
fn test_error_record_parse_error() {
    let err = Neo4jError::RecordParseError("malformed JSON".to_string());
    let msg = err.to_string();
    assert!(msg.contains("malformed JSON"));
    assert!(msg.contains("parse"));
}

#[test]
fn test_error_missing_property() {
    let err = Neo4jError::MissingProperty {
        label: "User".to_string(),
        property: "email".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("User"));
    assert!(msg.contains("email"));
    assert!(msg.contains("property"));
}

#[test]
fn test_error_type_conversion() {
    let err = Neo4jError::TypeConversion("cannot convert string to int".to_string());
    let msg = err.to_string();
    assert!(msg.contains("convert"));
    assert!(msg.contains("string to int"));
}

#[test]
fn test_error_circular_reference() {
    let err = Neo4jError::CircularReference("A -> B -> C -> A".to_string());
    let msg = err.to_string();
    assert!(msg.contains("circular"));
    assert!(msg.contains("A -> B -> C -> A"));
}

#[test]
fn test_error_recursion_limit_exceeded() {
    let err = Neo4jError::RecursionLimitExceeded {
        depth: 150,
        max_depth: 100,
    };
    let msg = err.to_string();
    assert!(msg.contains("150"));
    assert!(msg.contains("100"));
    assert!(msg.contains("depth"));
    assert!(msg.contains("exceeds"));
}

#[test]
fn test_error_string_length_exceeded() {
    let err = Neo4jError::StringLengthExceeded {
        length: 2_000_000,
        max_length: 1_000_000,
        property: "description".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("2000000") || msg.contains("2,000,000"));
    assert!(msg.contains("1000000") || msg.contains("1,000,000"));
    assert!(msg.contains("description"));
    assert!(msg.contains("exceeds"));
}

#[test]
fn test_error_node_count_exceeded() {
    let err = Neo4jError::NodeCountExceeded {
        count: 150_000,
        max_count: 100_000,
    };
    let msg = err.to_string();
    assert!(msg.contains("150000") || msg.contains("150,000"));
    assert!(msg.contains("100000") || msg.contains("100,000"));
    assert!(msg.contains("exceeds"));
}

#[test]
fn test_error_integer_overflow() {
    let err = Neo4jError::IntegerOverflow {
        context: "batch size calculation".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("overflow"));
    assert!(msg.contains("batch size calculation"));
}

#[test]
fn test_error_json_error_conversion() {
    let json_err: serde_json::Error = serde_json::from_str::<i32>("\"not a number\"").unwrap_err();
    let neo4j_err: Neo4jError = json_err.into();

    assert!(matches!(neo4j_err, Neo4jError::JsonError(_)));
    let msg = neo4j_err.to_string();
    assert!(msg.contains("JSON") || msg.contains("json"));
}

#[test]
fn test_error_hedl_error() {
    let err = Neo4jError::HedlError("parsing failed".to_string());
    let msg = err.to_string();
    assert!(msg.contains("HEDL"));
    assert!(msg.contains("parsing failed"));
}

#[test]
fn test_error_display_formats() {
    // Test that all error variants have reasonable display formats
    let errors = vec![
        Neo4jError::MissingSchema("Test".to_string()),
        Neo4jError::InvalidReference("test".to_string()),
        Neo4jError::UnresolvedReference {
            type_name: Some("Test".to_string()),
            id: "id".to_string(),
        },
        Neo4jError::InvalidNodeId("test".to_string()),
        Neo4jError::EmptyMatrixList("Test".to_string()),
        Neo4jError::InconsistentData("test".to_string()),
        Neo4jError::InvalidIdentifier("test".to_string()),
        Neo4jError::RecordParseError("test".to_string()),
        Neo4jError::MissingProperty {
            label: "Test".to_string(),
            property: "prop".to_string(),
        },
        Neo4jError::TypeConversion("test".to_string()),
        Neo4jError::CircularReference("test".to_string()),
        Neo4jError::RecursionLimitExceeded {
            depth: 10,
            max_depth: 5,
        },
        Neo4jError::StringLengthExceeded {
            length: 100,
            max_length: 50,
            property: "test".to_string(),
        },
        Neo4jError::NodeCountExceeded {
            count: 100,
            max_count: 50,
        },
        Neo4jError::IntegerOverflow {
            context: "test".to_string(),
        },
        Neo4jError::HedlError("test".to_string()),
    ];

    for error in errors {
        let msg = error.to_string();
        assert!(!msg.is_empty(), "Error message should not be empty");
        assert!(msg.len() > 5, "Error message should be meaningful");
    }
}

#[test]
fn test_result_type_usage() {
    // Test that Result type works correctly
    fn returns_ok() -> hedl_neo4j::Result<i32> {
        Ok(42)
    }

    fn returns_err() -> hedl_neo4j::Result<i32> {
        Err(Neo4jError::InvalidIdentifier("test".to_string()))
    }

    assert_eq!(returns_ok().unwrap(), 42);
    assert!(returns_err().is_err());
}

#[test]
fn test_error_type_is_error_trait() {
    // Verify Neo4jError implements std::error::Error
    let err = Neo4jError::InvalidIdentifier("test".to_string());
    let _err_ref: &dyn std::error::Error = &err;
}

#[test]
fn test_error_debug_format() {
    let err = Neo4jError::MissingSchema("User".to_string());
    let debug = format!("{err:?}");
    assert!(debug.contains("MissingSchema"));
    assert!(debug.contains("User"));
}

#[test]
fn test_error_messages_are_user_friendly() {
    // Ensure error messages are clear and actionable

    let err = Neo4jError::StringLengthExceeded {
        length: 1_500_000,
        max_length: 1_000_000,
        property: "biography".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("biography"));
    assert!(msg.contains("1500000") || msg.contains("1,500,000"));
    assert!(msg.contains("1000000") || msg.contains("1,000,000"));

    let err2 = Neo4jError::RecursionLimitExceeded {
        depth: 120,
        max_depth: 100,
    };
    let msg2 = err2.to_string();
    assert!(msg2.contains("120"));
    assert!(msg2.contains("100"));
}

#[test]
fn test_error_equality() {
    // Test Debug derive works for equality checks
    let err1 = Neo4jError::MissingSchema("User".to_string());
    let err2 = Neo4jError::MissingSchema("User".to_string());
    let err3 = Neo4jError::MissingSchema("Post".to_string());

    assert_eq!(format!("{err1:?}"), format!("{:?}", err2));
    assert_ne!(format!("{err1:?}"), format!("{:?}", err3));
}

#[test]
fn test_complex_error_scenarios() {
    // Test error scenarios with complex context

    // Deeply nested reference chain
    let err = Neo4jError::CircularReference(
        "User:alice -> Post:1 -> Comment:5 -> User:alice".to_string(),
    );
    let msg = err.to_string();
    assert!(msg.contains("circular"));
    assert!(msg.contains("User:alice"));

    // Large string with context
    let err = Neo4jError::StringLengthExceeded {
        length: 100_000_000,
        max_length: 50_000_000,
        property: "article_content".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("article_content"));

    // Recursion with context
    let err = Neo4jError::RecursionLimitExceeded {
        depth: 1000,
        max_depth: 100,
    };
    let msg = err.to_string();
    assert!(msg.contains("1000"));
    assert!(msg.contains("100"));
}

#[test]
fn test_error_context_preservation() {
    // Ensure error context is preserved through type conversions

    let json_error = serde_json::from_str::<i32>("\"invalid\"").unwrap_err();
    let neo4j_error: Neo4jError = json_error.into();

    match neo4j_error {
        Neo4jError::JsonError(inner) => {
            let msg = inner.to_string();
            assert!(!msg.is_empty());
        }
        _ => panic!("Expected JsonError variant"),
    }
}

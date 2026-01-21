//! Integration test for cypher module structure after SRP refactoring.
//!
//! Verifies that:
//! 1. All modules export correct functions
//! 2. Backward compatibility maintained
//! 3. Modules work together correctly

use hedl_neo4j::cypher;

#[test]
fn test_escape_module_exports() {
    // Escaping functions available
    assert_eq!(cypher::escape_string("it's").as_ref(), "it\\'s");
    assert_eq!(cypher::escape_identifier("name"), "name");
    assert_eq!(cypher::escape_label("User"), ":User");
    assert_eq!(cypher::escape_relationship_type("KNOWS"), ":KNOWS");
    assert_eq!(cypher::quote_string("hello"), "'hello'");
    assert_eq!(cypher::to_identifier("my-name"), "my_name");
    assert_eq!(cypher::to_relationship_type("hasPost"), "HAS_POST");
}

#[test]
fn test_validate_module_exports() {
    // Validation functions available
    assert!(cypher::is_valid_identifier("name"));
    assert!(!cypher::is_valid_identifier("123name"));
    assert!(cypher::validate_identifier("valid").is_ok());
    assert!(cypher::validate_identifier("123").is_err());

    // String length validation
    let config = hedl_neo4j::ToCypherConfig::default().with_max_string_length(10);
    assert!(cypher::validate_string_length("short", "test", &config).is_ok());
    assert!(cypher::validate_string_length("toolongstring", "test", &config).is_err());
}

#[test]
fn test_unicode_module_exports() {
    // Unicode security functions available
    assert_eq!(cypher::normalize_unicode("café"), "café");
    assert_eq!(cypher::sanitize_identifier("name\u{200B}test"), "nametest");
}

#[test]
fn test_modules_work_together() {
    // Real-world workflow: sanitize, validate, escape
    let input = "user\u{200B}name"; // Contains zero-width space

    // Step 1: Sanitize (unicode module)
    let sanitized = cypher::sanitize_identifier(input);
    assert_eq!(sanitized, "username");

    // Step 2: Validate (validate module)
    assert!(cypher::is_valid_identifier(&sanitized));

    // Step 3: Escape (escape module)
    let escaped = cypher::escape_identifier(&sanitized);
    assert_eq!(escaped, "username");
}

#[test]
fn test_backward_compatibility_imports() {
    // All old imports still work
    use hedl_neo4j::cypher::{
        escape_identifier, escape_label, escape_relationship_type, escape_string,
        is_valid_identifier, normalize_unicode, sanitize_identifier, validate_identifier,
        validate_string_length,
    };

    // Functions work as expected
    assert_eq!(escape_string("test").as_ref(), "test");
    assert_eq!(escape_identifier("name"), "name");
    assert_eq!(escape_label("User"), ":User");
    assert_eq!(escape_relationship_type("KNOWS"), ":KNOWS");
    assert!(is_valid_identifier("name"));
    assert!(validate_identifier("name").is_ok());
    assert_eq!(normalize_unicode("test"), "test");
    assert_eq!(sanitize_identifier("test"), "test");

    let config = hedl_neo4j::ToCypherConfig::default().with_max_string_length(100);
    assert!(validate_string_length("short", "test", &config).is_ok());
}

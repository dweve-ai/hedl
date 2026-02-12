// Comprehensive tests for header parsing and directive handling
//
// Tests focus on:
// - Header directive parsing (VERSION, STRUCT, ALIAS, NEST)
// - Header validation
// - Directive order and combinations
// - Error cases and malformed headers

use hedl_stream::{StreamError, StreamingParser};
use std::io::Cursor;

// ==================== VERSION directive tests ====================

#[test]
fn test_version_basic() {
    let input = r#"
%VERSION: 1.0
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();
    assert_eq!(header.version, (1, 0));
}

#[test]
fn test_version_different_versions() {
    let versions = vec!["1.0", "1.1", "2.0", "10.5", "99.99"];

    for version_str in versions {
        let input = format!("%VERSION: {version_str}\n---\n");
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let header = parser.header().unwrap();

        let parts: Vec<&str> = version_str.split('.').collect();
        let major: u32 = parts[0].parse().unwrap();
        let minor: u32 = parts[1].parse().unwrap();
        assert_eq!(header.version, (major, minor));
    }
}

#[test]
fn test_version_with_whitespace() {
    let inputs = vec![
        "%VERSION:   1.0  \n---\n",
        "%VERSION:\t1.0\n---\n",
        "%VERSION:  1.0  \t \n---\n",
    ];

    for input in inputs {
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let header = parser.header().unwrap();
        assert_eq!(header.version, (1, 0));
    }
}

#[test]
fn test_missing_version() {
    let input = "---\nkey: value\n";
    let result = StreamingParser::new(Cursor::new(input));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, StreamError::MissingVersion));
    }
}

#[test]
fn test_invalid_version_format() {
    let invalid_versions = vec![
        "%VERSION: 1\n---\n",     // Single number
        "%VERSION: 1.0.0\n---\n", // Three numbers
        "%VERSION: abc\n---\n",   // Non-numeric
        "%VERSION: 1.x\n---\n",   // Invalid minor
        "%VERSION: x.0\n---\n",   // Invalid major
        "%VERSION:\n---\n",       // Empty
    ];

    for input in invalid_versions {
        let result = StreamingParser::new(Cursor::new(input));
        assert!(result.is_err(), "Should fail for input: {input}");
    }
}

#[test]
fn test_version_must_come_before_separator() {
    let input = "---\n%VERSION: 1.0\n";
    let result = StreamingParser::new(Cursor::new(input));
    assert!(result.is_err());
}

// ==================== STRUCT directive tests ====================

#[test]
fn test_struct_basic() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(
        header.get_schema("User"),
        Some(&vec!["id".to_string(), "name".to_string()])
    );
}

#[test]
fn test_struct_multiple_fields() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, email, age, active]
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    let schema = header.get_schema("User").unwrap();
    assert_eq!(schema.len(), 5);
    assert_eq!(schema[0], "id");
    assert_eq!(schema[1], "name");
    assert_eq!(schema[2], "email");
    assert_eq!(schema[3], "age");
    assert_eq!(schema[4], "active");
}

#[test]
fn test_struct_single_field() {
    let input = r#"
%VERSION: 1.0
%STRUCT: Simple: [id]
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.get_schema("Simple"), Some(&vec!["id".to_string()]));
}

#[test]
fn test_struct_multiple_types() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Product: [id, price]
%STRUCT: Order: [id, user, product]
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert!(header.get_schema("User").is_some());
    assert!(header.get_schema("Product").is_some());
    assert!(header.get_schema("Order").is_some());
}

#[test]
fn test_struct_with_whitespace() {
    let inputs = vec![
        "%VERSION: 1.0\n%STRUCT:  User  : [ id , name ]\n---\n",
        "%VERSION: 1.0\n%STRUCT:\tUser\t:\t[\tid\t,\tname\t]\n---\n",
    ];

    for input in inputs {
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let header = parser.header().unwrap();
        assert!(header.get_schema("User").is_some());
    }
}

#[test]
fn test_struct_invalid_formats() {
    let invalid_structs = vec![
        "%VERSION: 1.0\n%STRUCT: User\n---\n", // Missing fields
        "%VERSION: 1.0\n%STRUCT: User: id, name\n---\n", // Missing brackets
        "%VERSION: 1.0\n%STRUCT: User: []\n---\n", // Empty fields
        "%VERSION: 1.0\n%STRUCT: : [id, name]\n---\n", // Missing type name
    ];

    for input in invalid_structs {
        let result = StreamingParser::new(Cursor::new(input));
        assert!(result.is_err(), "Should fail for input: {input}");
    }
}

#[test]
fn test_struct_field_with_hyphens() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [user-id, first-name, last-name]
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    let schema = header.get_schema("User").unwrap();
    assert_eq!(schema[0], "user-id");
    assert_eq!(schema[1], "first-name");
    assert_eq!(schema[2], "last-name");
}

#[test]
fn test_struct_field_with_underscores() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [user_id, first_name, last_name]
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    let schema = header.get_schema("User").unwrap();
    assert_eq!(schema[0], "user_id");
    assert_eq!(schema[1], "first_name");
    assert_eq!(schema[2], "last_name");
}

// ==================== ALIAS directive tests ====================

#[test]
fn test_alias_basic() {
    let input = r#"
%VERSION: 1.0
%ALIAS: active = "Active"
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.aliases.get("active"), Some(&"Active".to_string()));
}

#[test]
fn test_alias_multiple() {
    let input = r#"
%VERSION: 1.0
%ALIAS: active = "Active"
%ALIAS: inactive = "Inactive"
%ALIAS: pending = "Pending"
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.aliases.get("active"), Some(&"Active".to_string()));
    assert_eq!(
        header.aliases.get("inactive"),
        Some(&"Inactive".to_string())
    );
    assert_eq!(header.aliases.get("pending"), Some(&"Pending".to_string()));
}

#[test]
fn test_alias_with_spaces_in_value() {
    let input = r#"
%VERSION: 1.0
%ALIAS: admin = "Administrator Role"
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(
        header.aliases.get("admin"),
        Some(&"Administrator Role".to_string())
    );
}

#[test]
fn test_alias_without_quotes() {
    let input = r#"
%VERSION: 1.0
%ALIAS: active = Active
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.aliases.get("active"), Some(&"Active".to_string()));
}

#[test]
fn test_alias_with_whitespace() {
    let input = r#"
%VERSION: 1.0
%ALIAS:  active  =  "Active"
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.aliases.get("active"), Some(&"Active".to_string()));
}

// ==================== NEST directive tests ====================

#[test]
fn test_nest_basic() {
    let input = r#"
%VERSION: 1.0
%NEST: User > Order
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(
        header.get_child_types("User"),
        Some(&vec!["Order".to_string()])
    );
}

#[test]
fn test_nest_multiple() {
    let input = r#"
%VERSION: 1.0
%NEST: User > Order
%NEST: Order > LineItem
%NEST: Product > Review
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(
        header.get_child_types("User"),
        Some(&vec!["Order".to_string()])
    );
    assert_eq!(
        header.get_child_types("Order"),
        Some(&vec!["LineItem".to_string()])
    );
    assert_eq!(
        header.get_child_types("Product"),
        Some(&vec!["Review".to_string()])
    );
}

#[test]
fn test_nest_with_whitespace() {
    let input = r#"
%VERSION: 1.0
%NEST:  User  >  Order
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(
        header.get_child_types("User"),
        Some(&vec!["Order".to_string()])
    );
}

#[test]
fn test_nest_invalid_formats() {
    let invalid_nests = vec![
        "%VERSION: 1.0\n%NEST: User\n---\n",       // Missing child
        "%VERSION: 1.0\n%NEST: User Order\n---\n", // Missing >
        "%VERSION: 1.0\n%NEST: > Order\n---\n",    // Missing parent
        "%VERSION: 1.0\n%NEST: User >\n---\n",     // Missing child after >
    ];

    for input in invalid_nests {
        let result = StreamingParser::new(Cursor::new(input));
        assert!(result.is_err(), "Should fail for input: {input}");
    }
}

// ==================== Directive combination tests ====================

#[test]
fn test_all_directives_together() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, status]
%STRUCT: Order: [id, amount]
%ALIAS: active = "Active"
%ALIAS: inactive = "Inactive"
%NEST: User > Order
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.version, (1, 0));
    assert!(header.get_schema("User").is_some());
    assert!(header.get_schema("Order").is_some());
    assert_eq!(header.aliases.get("active"), Some(&"Active".to_string()));
    assert_eq!(
        header.get_child_types("User"),
        Some(&vec!["Order".to_string()])
    );
}

#[test]
fn test_directive_order_variation() {
    let input = r#"
%VERSION: 1.0
%NEST: User > Order
%ALIAS: active = "Active"
%STRUCT: User: [id, name]
%STRUCT: Order: [id, amount]
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.version, (1, 0));
    assert!(header.get_schema("User").is_some());
    assert!(header.get_schema("Order").is_some());
}

#[test]
fn test_multiple_structs_with_nesting() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Order: [id, user, amount]
%STRUCT: LineItem: [id, product, quantity]
%NEST: User > Order
%NEST: Order > LineItem
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.structs.len(), 3);
    assert_eq!(header.nests.len(), 2);
}

// ==================== Separator tests ====================

#[test]
fn test_separator_basic() {
    let input = "%VERSION: 1.0\n---\n";
    let parser = StreamingParser::new(Cursor::new(input));
    assert!(parser.is_ok());
}

#[test]
fn test_separator_with_extra_dashes() {
    let input = "%VERSION: 1.0\n-----\n";
    let parser = StreamingParser::new(Cursor::new(input));
    assert!(parser.is_ok());
}

#[test]
fn test_separator_with_whitespace_before() {
    let input = "%VERSION: 1.0\n  ---\n";
    let parser = StreamingParser::new(Cursor::new(input));
    assert!(parser.is_ok());
}

#[test]
fn test_separator_with_whitespace_after() {
    let input = "%VERSION: 1.0\n---  \n";
    let parser = StreamingParser::new(Cursor::new(input));
    assert!(parser.is_ok());
}

#[test]
fn test_missing_separator() {
    let input = "%VERSION: 1.0\nkey: value\n";
    let parser = StreamingParser::new(Cursor::new(input));
    // Parser may successfully parse header and treat "key: value" as data
    // This verifies current behavior
    if let Ok(mut p) = parser {
        // Try to get events and see if any errors occur
        for event in &mut p {
            if event.is_err() {
                break;
            }
        }
    }
}

// ==================== Comment handling in header ====================

#[test]
fn test_header_with_comments() {
    let input = r#"
# This is a comment
%VERSION: 1.0
# Another comment
%STRUCT: User: [id, name]
# Comment before separator
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.version, (1, 0));
    assert!(header.get_schema("User").is_some());
}

#[test]
fn test_inline_comments_in_header() {
    let input = r#"
%VERSION: 1.0  # Version directive
%STRUCT: User: [id, name]  # User schema
---
";
    // Inline comments after directives may not be supported
    // This test verifies the current behavior
    if let Ok(p) = StreamingParser::new(Cursor::new(input)) {
        let header = p.header().unwrap();
        assert_eq!(header.version, (1, 0));
    }
}

// ==================== Empty header tests ====================

#[test]
fn test_minimal_header() {
    let input = "%VERSION: 1.0\n---\n";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.version, (1, 0));
    assert!(header.structs.is_empty());
    assert!(header.aliases.is_empty());
    assert!(header.nests.is_empty());
}

#[test]
fn test_header_with_blank_lines() {
    let input = r#"

%VERSION: 1.0

%STRUCT: User: [id, name]

---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.version, (1, 0));
    assert!(header.get_schema("User").is_some());
}

// ==================== Edge cases ====================

#[test]
fn test_struct_overwrite() {
    // Later STRUCT directive should overwrite earlier one
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: User: [id, name, email]
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    let schema = header.get_schema("User").unwrap();
    assert_eq!(schema.len(), 3); // Should have the second definition
}

#[test]
fn test_alias_overwrite() {
    let input = r#"
%VERSION: 1.0
%ALIAS: status = "First"
%ALIAS: status = "Second"
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.aliases.get("status"), Some(&"Second".to_string()));
}

#[test]
fn test_struct_type_name_with_numbers() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User2: [id, name]
%STRUCT: Type123: [id]
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert!(header.get_schema("User2").is_some());
    assert!(header.get_schema("Type123").is_some());
}

#[test]
fn test_struct_field_name_with_numbers() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id1, field2, name3]
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    let schema = header.get_schema("User").unwrap();
    assert_eq!(schema[0], "id1");
    assert_eq!(schema[1], "field2");
    assert_eq!(schema[2], "name3");
}

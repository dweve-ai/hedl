// Tests for v2.0 compact directive syntax
//
// Tests focus on:
// - %V:2.0 (compact version)
// - %NULL:~ (null literal character)
// - %QUOTE:" (quote character)
// - %S:Type:[cols] (compact schema)
// - %N:Parent>Child (compact nesting)
// - %C:Type.total=N (count hints)
// - %C:Type.field:val=N (field count hints)

use hedl_stream::StreamingParser;
use std::io::Cursor;

// ==================== %V: directive tests ====================

#[test]
fn test_v_directive_basic() {
    let input = r#"
%V:2.0
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();
    assert_eq!(header.version, (2, 0));
}

#[test]
fn test_v_directive_different_versions() {
    let versions = vec!["1.0", "1.1", "1.2", "1.3", "2.0"];

    for version_str in versions {
        let input = format!("%V:{version_str}\n---\n");
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let header = parser.header().unwrap();

        let parts: Vec<&str> = version_str.split('.').collect();
        let major: u32 = parts[0].parse().unwrap();
        let minor: u32 = parts[1].parse().unwrap();
        assert_eq!(header.version, (major, minor));
    }
}

#[test]
fn test_v_directive_with_whitespace() {
    let input = "%V:  2.0  \n---\n";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();
    assert_eq!(header.version, (2, 0));
}

// ==================== %NULL: directive tests ====================

#[test]
fn test_null_directive_basic() {
    let input = r#"
%V:2.0
%NULL:~
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();
    assert_eq!(header.null_char, '~');
}

#[test]
fn test_null_directive_custom_char() {
    let chars = vec!['~', 'N', '∅', '_'];

    for null_char in chars {
        let input = format!("%V:2.0\n%NULL:{null_char}\n---\n");
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let header = parser.header().unwrap();
        assert_eq!(header.null_char, null_char);
    }
}

#[test]
fn test_null_directive_with_whitespace() {
    let input = "%V:2.0\n%NULL:  ~  \n---\n";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();
    assert_eq!(header.null_char, '~');
}

// ==================== %QUOTE: directive tests ====================

#[test]
fn test_quote_directive_basic() {
    let input = r#"
%V:2.0
%QUOTE:"
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();
    assert_eq!(header.quote_char, '\"');
}

#[test]
fn test_quote_directive_custom_char() {
    let chars = vec!['\"', '\'', '`'];

    for quote_char in chars {
        let input = format!("%V:2.0\n%QUOTE:{quote_char}\n---\n");
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let header = parser.header().unwrap();
        assert_eq!(header.quote_char, quote_char);
    }
}

#[test]
fn test_quote_directive_with_whitespace() {
    let input = "%V:2.0\n%QUOTE:  \"  \n---\n";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();
    assert_eq!(header.quote_char, '\"');
}

// ==================== %S: directive tests ====================

#[test]
fn test_s_directive_basic() {
    let input = r#"
%V:2.0
%S:User:[id,name]
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
fn test_s_directive_multiple_fields() {
    let input = r#"
%V:2.0
%S:User:[id,name,email,age,active]
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
fn test_s_directive_multiple_types() {
    let input = r#"
%V:2.0
%S:User:[id,name]
%S:Product:[id,price]
%S:Order:[id,user,product]
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert!(header.get_schema("User").is_some());
    assert!(header.get_schema("Product").is_some());
    assert!(header.get_schema("Order").is_some());
}

#[test]
fn test_s_directive_with_spaces() {
    let input = "%V:2.0\n%S: User : [ id , name ]\n---\n";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();
    assert!(header.get_schema("User").is_some());
}

// ==================== %N: directive tests ====================

#[test]
fn test_n_directive_basic() {
    let input = r#"
%V:2.0
%N:User>Order
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
fn test_n_directive_multiple() {
    let input = r#"
%V:2.0
%N:User>Order
%N:Order>LineItem
%N:Product>Review
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
fn test_n_directive_with_spaces() {
    let input = "%V:2.0\n%N: User > Order \n---\n";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(
        header.get_child_types("User"),
        Some(&vec!["Order".to_string()])
    );
}

// ==================== %C: directive tests ====================

#[test]
fn test_c_directive_total() {
    let input = r#"
%V:2.0
%C:User.total=100
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.count_totals.get("User"), Some(&100));
}

#[test]
fn test_c_directive_multiple_totals() {
    let input = r#"
%V:2.0
%C:User.total=100
%C:Product.total=250
%C:Order.total=500
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.count_totals.get("User"), Some(&100));
    assert_eq!(header.count_totals.get("Product"), Some(&250));
    assert_eq!(header.count_totals.get("Order"), Some(&500));
}

#[test]
fn test_c_directive_field_single_value() {
    let input = r#"
%V:2.0
%C:User.status:active=42
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    let field_key = "User.status";
    let counts = header.count_fields.get(field_key).unwrap();
    assert_eq!(counts.get("active"), Some(&42));
}

#[test]
fn test_c_directive_field_multiple_values() {
    let input = r#"
%V:2.0
%C:User.status:active=42,inactive=18,pending=5
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    let field_key = "User.status";
    let counts = header.count_fields.get(field_key).unwrap();
    assert_eq!(counts.get("active"), Some(&42));
    assert_eq!(counts.get("inactive"), Some(&18));
    assert_eq!(counts.get("pending"), Some(&5));
}

#[test]
fn test_c_directive_field_with_spaces() {
    let input = r#"
%V:2.0
%C:User.status: active = 42 , inactive = 18
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    let field_key = "User.status";
    let counts = header.count_fields.get(field_key).unwrap();
    assert_eq!(counts.get("active"), Some(&42));
    assert_eq!(counts.get("inactive"), Some(&18));
}

// ==================== Combined v2.0 directives ====================

#[test]
fn test_all_v20_directives() {
    let input = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,status]
%S:Order:[id,amount]
%N:User>Order
%C:User.total=100
%C:User.status:active=65,inactive=35
%C:Order.total=250
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    // Version
    assert_eq!(header.version, (2, 0));

    // Null and quote chars
    assert_eq!(header.null_char, '~');
    assert_eq!(header.quote_char, '\"');

    // Schemas
    assert!(header.get_schema("User").is_some());
    assert!(header.get_schema("Order").is_some());

    // Nesting
    assert_eq!(
        header.get_child_types("User"),
        Some(&vec!["Order".to_string()])
    );

    // Count totals
    assert_eq!(header.count_totals.get("User"), Some(&100));
    assert_eq!(header.count_totals.get("Order"), Some(&250));

    // Field counts
    let field_key = format!("User.{}", "status");
    let user_status_counts = header.count_fields.get(&field_key).unwrap();
    assert_eq!(user_status_counts.get("active"), Some(&65));
    assert_eq!(user_status_counts.get("inactive"), Some(&35));
}

// ==================== Backward compatibility tests ====================

#[test]
fn test_mixed_v10_and_v20_syntax() {
    // v2.0 version with v1.0 style directives should still work
    let input = r#"
%V:2.0
%STRUCT: User: [id, name]
%NEST: User > Order
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.version, (2, 0));
    assert!(header.get_schema("User").is_some());
    assert_eq!(
        header.get_child_types("User"),
        Some(&vec!["Order".to_string()])
    );
}

#[test]
fn test_v10_syntax_still_works() {
    // Old v1.0 syntax should continue to work
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
%NEST: User > Order
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.version, (1, 0));
    assert!(header.get_schema("User").is_some());
    assert_eq!(
        header.get_child_types("User"),
        Some(&vec!["Order".to_string()])
    );
}

#[test]
fn test_compact_and_verbose_schemas_can_mix() {
    let input = r#"
%V:2.0
%S:User:[id,name]
%STRUCT: Product: [id, price]
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert!(header.get_schema("User").is_some());
    assert!(header.get_schema("Product").is_some());
}

// ==================== Error handling ====================

#[test]
fn test_null_directive_missing_char() {
    let input = r#"
%V:2.0
%NULL:
---
";
    let result = StreamingParser::new(Cursor::new(input));
    assert!(result.is_err());
}

#[test]
fn test_quote_directive_missing_char() {
    let input = r#"
%V:2.0
%QUOTE:
---
";
    let result = StreamingParser::new(Cursor::new(input));
    assert!(result.is_err());
}

#[test]
fn test_s_directive_missing_bracket() {
    let input = r#"
%V:2.0
%S:User:id,name
---
";
    let result = StreamingParser::new(Cursor::new(input));
    assert!(result.is_err());
}

#[test]
fn test_n_directive_missing_arrow() {
    let input = r#"
%V:2.0
%N:User Order
---
";
    let result = StreamingParser::new(Cursor::new(input));
    assert!(result.is_err());
}

#[test]
fn test_c_directive_missing_dot() {
    let input = r#"
%V:2.0
%C:Usertotal=100
---
";
    let result = StreamingParser::new(Cursor::new(input));
    assert!(result.is_err());
}

#[test]
fn test_c_directive_invalid_count() {
    let input = r#"
%V:2.0
%C:User.total=abc
---
"#;
    let result = StreamingParser::new(Cursor::new(input));
    assert!(result.is_err());
}

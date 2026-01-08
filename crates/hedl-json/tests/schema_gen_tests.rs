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

//! Integration tests for JSON Schema generation

use hedl_core::parse;
use hedl_json::schema_gen::{generate_schema, generate_schema_value, validate_schema, SchemaConfig};
use serde_json::{json, Value as JsonValue};

/// Helper to parse HEDL from string
fn parse_hedl(input: &str) -> hedl_core::Document {
    // Prepend HEDL header if not present, or separate header from body if needed
    let hedl = if input.contains("%VERSION") || input.starts_with("%HEDL") {
        input.to_string()
    } else if input.contains("%STRUCT") || input.contains("%NEST") {
        // Has directives but no VERSION - add VERSION and ensure separator
        let (header, body) = if input.contains("---") {
            let parts: Vec<&str> = input.splitn(2, "---").collect();
            (parts[0].trim().to_string(), parts.get(1).map(|s| s.trim().to_string()).unwrap_or_default())
        } else {
            // Extract directives to header
            let mut header_lines = Vec::new();
            let mut body_lines = Vec::new();
            for line in input.lines() {
                if line.trim().starts_with('%') {
                    header_lines.push(line.to_string());
                } else {
                    body_lines.push(line.to_string());
                }
            }
            (header_lines.join("\n"), body_lines.join("\n"))
        };
        format!("%VERSION: 1.0\n{}\n---\n{}", header, body)
    } else {
        format!("%VERSION: 1.0\n---\n{}", input)
    };
    parse(hedl.as_bytes()).unwrap()
}

// ==================== Basic Functionality ====================

#[test]
fn test_generate_basic_schema() {
    let hedl = r#"
name: Alice
age: 30
active: true
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema(&doc, &SchemaConfig::default()).unwrap();

    // Should be valid JSON
    let parsed: JsonValue = serde_json::from_str(&schema).unwrap();
    assert!(parsed.is_object());

    // Should have required fields
    assert!(parsed.get("$schema").is_some());
    assert!(parsed.get("type").is_some());
    assert!(parsed.get("properties").is_some());
}

#[test]
fn test_schema_value_generation() {
    let hedl = "value: 42";
    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    assert_eq!(
        schema.get("$schema").and_then(|v| v.as_str()),
        Some("http://json-schema.org/draft-07/schema#")
    );
}

#[test]
fn test_empty_document_schema() {
    let hedl = "";
    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let props = schema.get("properties").unwrap().as_object().unwrap();
    assert!(props.is_empty());
}

// ==================== Type Coverage ====================

#[test]
fn test_all_scalar_types() {
    let hedl = r#"
null_val: ~
bool_val: true
int_val: 42
float_val: 3.14
string_val: "hello"
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let props = schema.get("properties").unwrap().as_object().unwrap();

    assert_eq!(
        props
            .get("null_val")
            .unwrap()
            .get("type")
            .unwrap()
            .as_str(),
        Some("null")
    );
    assert_eq!(
        props
            .get("bool_val")
            .unwrap()
            .get("type")
            .unwrap()
            .as_str(),
        Some("boolean")
    );
    assert_eq!(
        props
            .get("int_val")
            .unwrap()
            .get("type")
            .unwrap()
            .as_str(),
        Some("integer")
    );
    assert_eq!(
        props
            .get("float_val")
            .unwrap()
            .get("type")
            .unwrap()
            .as_str(),
        Some("number")
    );
    assert_eq!(
        props
            .get("string_val")
            .unwrap()
            .get("type")
            .unwrap()
            .as_str(),
        Some("string")
    );
}

#[test]
fn test_reference_type() {
    let hedl = r#"
%STRUCT: User: [id, name]
users: @User
  |u123, Alice
owner: @User:u123
"#;
    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let props = schema.get("properties").unwrap().as_object().unwrap();
    let owner = props.get("owner").unwrap().as_object().unwrap();

    assert_eq!(owner.get("type").unwrap().as_str(), Some("string"));
    assert!(owner.contains_key("pattern"));
    assert!(owner.contains_key("description"));
}

#[test]
fn test_expression_type() {
    let hedl = r#"computed: $(now())"#;
    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let props = schema.get("properties").unwrap().as_object().unwrap();
    let computed = props.get("computed").unwrap().as_object().unwrap();

    assert_eq!(computed.get("type").unwrap().as_str(), Some("string"));
    assert!(computed.contains_key("pattern"));
}

#[test]
fn test_tensor_types() {
    let hedl = r#"
scalar: [42.0]
array: [[1.0, 2.0], [3.0, 4.0]]
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let props = schema.get("properties").unwrap().as_object().unwrap();

    // Scalar tensor
    let scalar = props.get("scalar").unwrap().as_object().unwrap();
    assert!(scalar.contains_key("type"));

    // Array tensor
    let array = props.get("array").unwrap().as_object().unwrap();
    assert_eq!(array.get("type").unwrap().as_str(), Some("array"));
}

// ==================== %STRUCT:Definitions ====================

#[test]
fn test_struct_definition() {
    let hedl = r#"
%STRUCT: User: [id, name, email]
users: @User
  |u1, Alice, alice@example.com
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    assert!(defs.contains_key("User"));

    let user = defs.get("User").unwrap().as_object().unwrap();
    assert_eq!(user.get("type").unwrap().as_str(), Some("object"));

    let props = user.get("properties").unwrap().as_object().unwrap();
    assert!(props.contains_key("id"));
    assert!(props.contains_key("name"));
    assert!(props.contains_key("email"));
}

#[test]
fn test_struct_required_fields() {
    let hedl = r#"
%STRUCT: User: [id, name]
users: @User
  |u1, Alice
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    let user = defs.get("User").unwrap().as_object().unwrap();

    let required = user.get("required").unwrap().as_array().unwrap();
    assert_eq!(required.len(), 1);
    assert_eq!(required[0].as_str(), Some("id"));
}

#[test]
fn test_multiple_structs() {
    let hedl = r#"
%STRUCT: User: [id, name]
%STRUCT: Post: [id, title]
%STRUCT: Comment: [id, text]

users: @User
  |u1, Alice
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    assert_eq!(defs.len(), 3);
    assert!(defs.contains_key("User"));
    assert!(defs.contains_key("Post"));
    assert!(defs.contains_key("Comment"));
}

#[test]
fn test_matrix_list_references_definition() {
    let hedl = r#"
%STRUCT: Product: [id, name, price]
products: @Product
  |p1, Widget, 9.99
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let props = schema.get("properties").unwrap().as_object().unwrap();
    let products = props.get("products").unwrap().as_object().unwrap();

    assert_eq!(products.get("type").unwrap().as_str(), Some("array"));

    let items = products.get("items").unwrap().as_object().unwrap();
    assert_eq!(
        items.get("$ref").unwrap().as_str(),
        Some("#/definitions/Product")
    );
}

// ==================== Nested Types (NEST) ====================

#[test]
fn test_nest_relationship() {
    let hedl = r#"
%STRUCT: Team: [id, name]
%STRUCT: Member: [id, name]
%NEST: Team > Member

teams: @Team
  |t1, Engineering
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    let team = defs.get("Team").unwrap().as_object().unwrap();
    let props = team.get("properties").unwrap().as_object().unwrap();

    // Should have Members property (pluralized)
    assert!(props.contains_key("Members"));

    let members = props.get("Members").unwrap().as_object().unwrap();
    assert_eq!(members.get("type").unwrap().as_str(), Some("array"));

    let items = members.get("items").unwrap().as_object().unwrap();
    assert_eq!(
        items.get("$ref").unwrap().as_str(),
        Some("#/definitions/Member")
    );
}

#[test]
fn test_nested_children_array() {
    let hedl = r#"
%STRUCT: Department: [id, name]
%STRUCT: Employee: [id, name]
%NEST: Department > Employee

departments: @Department
  |d1, Engineering
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    let dept = defs.get("Department").unwrap().as_object().unwrap();
    let props = dept.get("properties").unwrap().as_object().unwrap();

    assert!(props.contains_key("Employees"));
}

// ==================== Configuration Options ====================

#[test]
fn test_config_title() {
    let hedl = "name: Alice";
    let doc = parse_hedl(hedl);
    let config = SchemaConfig::builder()
        .title("User Schema")
        .build();

    let schema = generate_schema_value(&doc, &config).unwrap();
    assert_eq!(
        schema.get("title").and_then(|v| v.as_str()),
        Some("User Schema")
    );
}

#[test]
fn test_config_description() {
    let hedl = "name: Alice";
    let doc = parse_hedl(hedl);
    let config = SchemaConfig::builder()
        .description("Schema for user data")
        .build();

    let schema = generate_schema_value(&doc, &config).unwrap();
    assert_eq!(
        schema.get("description").and_then(|v| v.as_str()),
        Some("Schema for user data")
    );
}

#[test]
fn test_config_schema_id() {
    let hedl = "name: Alice";
    let doc = parse_hedl(hedl);
    let config = SchemaConfig::builder()
        .schema_id("https://example.com/schema.json")
        .build();

    let schema = generate_schema_value(&doc, &config).unwrap();
    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://example.com/schema.json")
    );
}

#[test]
fn test_config_strict_mode() {
    let hedl = "name: Alice";
    let doc = parse_hedl(hedl);
    let config = SchemaConfig::builder().strict(true).build();

    let schema = generate_schema_value(&doc, &config).unwrap();
    assert_eq!(
        schema.get("additionalProperties").and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[test]
fn test_config_strict_mode_in_definitions() {
    let hedl = r#"
%STRUCT: User: [id, name]
users: @User
  |u1, Alice
"#;

    let doc = parse_hedl(hedl);
    let config = SchemaConfig::builder().strict(true).build();

    let schema = generate_schema_value(&doc, &config).unwrap();
    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    let user = defs.get("User").unwrap().as_object().unwrap();

    assert_eq!(
        user.get("additionalProperties").and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[test]
fn test_config_include_examples() {
    let hedl = "age: 30";
    let doc = parse_hedl(hedl);
    let config = SchemaConfig::builder().include_examples(true).build();

    let schema = generate_schema_value(&doc, &config).unwrap();
    let props = schema.get("properties").unwrap().as_object().unwrap();
    let age = props.get("age").unwrap().as_object().unwrap();

    assert!(age.contains_key("examples"));
    let examples = age.get("examples").unwrap().as_array().unwrap();
    assert_eq!(examples[0].as_i64(), Some(30));
}

#[test]
fn test_config_metadata_disabled() {
    let hedl = "name: Alice";
    let doc = parse_hedl(hedl);
    let config = SchemaConfig::builder()
        .include_metadata(false)
        .title("Should not appear")
        .description("Should not appear")
        .schema_id("https://example.com")
        .build();

    let schema = generate_schema_value(&doc, &config).unwrap();
    assert!(!schema.as_object().unwrap().contains_key("title"));
    assert!(!schema.as_object().unwrap().contains_key("description"));
    assert!(!schema.as_object().unwrap().contains_key("$id"));
}

#[test]
fn test_config_builder_chaining() {
    let hedl = "name: Alice";
    let doc = parse_hedl(hedl);
    let config = SchemaConfig::builder()
        .title("Test Schema")
        .description("Test description")
        .schema_id("https://test.com")
        .strict(true)
        .include_examples(true)
        .build();

    let schema = generate_schema_value(&doc, &config).unwrap();

    assert!(schema.get("title").is_some());
    assert!(schema.get("description").is_some());
    assert!(schema.get("$id").is_some());
    assert_eq!(
        schema.get("additionalProperties").and_then(|v| v.as_bool()),
        Some(false)
    );
}

// ==================== Schema Validation ====================

#[test]
fn test_validate_valid_schema() {
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {}
    });

    assert!(validate_schema(&schema).is_ok());
}

#[test]
fn test_validate_missing_schema_field() {
    let schema = json!({
        "type": "object"
    });

    assert!(validate_schema(&schema).is_err());
}

#[test]
fn test_validate_missing_type() {
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#"
    });

    assert!(validate_schema(&schema).is_err());
}

#[test]
fn test_validate_invalid_type() {
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "invalid_type"
    });

    assert!(validate_schema(&schema).is_err());
}

#[test]
fn test_validate_generated_schema() {
    let hedl = r#"
%STRUCT: User: [id, name, email]
users: @User
  |u1, Alice, alice@example.com
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    // Generated schema should be valid
    let validation_result = validate_schema(&schema);
    if let Err(e) = &validation_result {
        eprintln!("Validation error: {:?}", e);
        eprintln!("Generated schema: {}", serde_json::to_string_pretty(&schema).unwrap());
    }
    assert!(validation_result.is_ok());
}

// ==================== Type Inference ====================

#[test]
fn test_infer_email_field() {
    let hedl = r#"
%STRUCT: User: [id, email]
users: @User
  |u1, alice@example.com
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    let user = defs.get("User").unwrap().as_object().unwrap();
    let props = user.get("properties").unwrap().as_object().unwrap();
    let email = props.get("email").unwrap().as_object().unwrap();

    assert_eq!(email.get("format").and_then(|v| v.as_str()), Some("email"));
}

#[test]
fn test_infer_url_field() {
    let hedl = r#"
%STRUCT: Site: [id, url]
sites: @Site
  |s1, https://example.com
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    let site = defs.get("Site").unwrap().as_object().unwrap();
    let props = site.get("properties").unwrap().as_object().unwrap();
    let url = props.get("url").unwrap().as_object().unwrap();

    assert_eq!(url.get("format").and_then(|v| v.as_str()), Some("uri"));
}

#[test]
fn test_infer_date_field() {
    let hedl = r#"
%STRUCT: Event: [id, created_at]
events: @Event
  |e1, 2024-01-01T00:00:00Z
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    let event = defs.get("Event").unwrap().as_object().unwrap();
    let props = event.get("properties").unwrap().as_object().unwrap();
    let created_at = props.get("created_at").unwrap().as_object().unwrap();

    assert_eq!(
        created_at.get("format").and_then(|v| v.as_str()),
        Some("date-time")
    );
}

#[test]
fn test_infer_boolean_field() {
    let hedl = r#"
%STRUCT: User: [id, is_active]
users: @User
  |u1, true
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    let user = defs.get("User").unwrap().as_object().unwrap();
    let props = user.get("properties").unwrap().as_object().unwrap();
    let is_active = props.get("is_active").unwrap().as_object().unwrap();

    assert_eq!(
        is_active.get("type").and_then(|v| v.as_str()),
        Some("boolean")
    );
}

// ==================== Complex Structures ====================

#[test]
fn test_nested_objects() {
    let hedl = r#"
user:
  name: Alice
  address:
    city: Seattle
    zip: 98101
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let props = schema.get("properties").unwrap().as_object().unwrap();
    let user = props.get("user").unwrap().as_object().unwrap();
    assert_eq!(user.get("type").unwrap().as_str(), Some("object"));

    let user_props = user.get("properties").unwrap().as_object().unwrap();
    assert!(user_props.contains_key("address"));

    let address = user_props.get("address").unwrap().as_object().unwrap();
    assert_eq!(address.get("type").unwrap().as_str(), Some("object"));
}

#[test]
fn test_deep_nesting() {
    let hedl = r#"
root:
  level1:
    level2:
      level3:
        value: deep
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    // Should handle deep nesting
    let props = schema.get("properties").unwrap().as_object().unwrap();
    assert!(props.contains_key("root"));
}

// ==================== Edge Cases ====================

#[test]
fn test_empty_struct() {
    let mut doc = hedl_core::Document::new((1, 0));
    doc.structs.insert("Empty".to_string(), vec![]);

    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();
    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    assert!(defs.contains_key("Empty"));
}

#[test]
fn test_special_characters_in_field_names() {
    let hedl = r#"
%STRUCT: User: [id, user_name, email_address]
users: @User
  |u1, Alice, alice@example.com
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    let user = defs.get("User").unwrap().as_object().unwrap();
    let props = user.get("properties").unwrap().as_object().unwrap();

    // Should preserve field names with underscores
    assert!(props.contains_key("user_name") && props.contains_key("email_address"));
}

#[test]
fn test_unicode_in_values() {
    let hedl = "name: 日本語";
    let doc = parse_hedl(hedl);
    let schema = generate_schema_value(&doc, &SchemaConfig::default()).unwrap();

    // Should handle unicode
    let props = schema.get("properties").unwrap().as_object().unwrap();
    assert!(props.contains_key("name"));
}

// ==================== Full Integration Tests ====================

#[test]
fn test_complete_api_schema() {
    let hedl = r#"
%STRUCT: User: [id, name, email, created_at]
%STRUCT: Post: [id, title, content, author_id]
%STRUCT: Comment: [id, text, post_id, user_id]

%NEST: User > Post
%NEST: Post > Comment

users: @User
  |u1, Alice, alice@example.com, 2024-01-01T00:00:00Z
  |u2, Bob, bob@example.com, 2024-01-02T00:00:00Z

posts: @Post
  |p1, "Hello World", "First post", u1
  |p2, "Second Post", "More content", u2

comments: @Comment
  |c1, "Nice post!", p1, u2
  |c2, "Thanks!", p1, u1
"#;

    let doc = parse_hedl(hedl);
    let config = SchemaConfig::builder()
        .title("Blog API Schema")
        .description("Schema for a simple blog API")
        .schema_id("https://api.example.com/schema")
        .strict(true)
        .include_examples(false)
        .build();

    let schema = generate_schema_value(&doc, &config).unwrap();

    // Validate schema structure
    assert!(validate_schema(&schema).is_ok());

    // Check metadata
    assert_eq!(
        schema.get("title").and_then(|v| v.as_str()),
        Some("Blog API Schema")
    );

    // Check definitions
    let defs = schema.get("definitions").unwrap().as_object().unwrap();
    assert_eq!(defs.len(), 3);
    assert!(defs.contains_key("User"));
    assert!(defs.contains_key("Post"));
    assert!(defs.contains_key("Comment"));

    // Check User has Posts
    let user = defs.get("User").unwrap().as_object().unwrap();
    let user_props = user.get("properties").unwrap().as_object().unwrap();
    assert!(user_props.contains_key("Posts"));

    // Check Post has Comments
    let post = defs.get("Post").unwrap().as_object().unwrap();
    let post_props = post.get("properties").unwrap().as_object().unwrap();
    assert!(post_props.contains_key("Comments"));

    // Check root properties
    let props = schema.get("properties").unwrap().as_object().unwrap();
    assert!(props.contains_key("users"));
    assert!(props.contains_key("posts"));
    assert!(props.contains_key("comments"));
}

#[test]
fn test_schema_serialization() {
    let hedl = r#"
%STRUCT: Product: [id, name, price]
products: @Product
  |p1, Widget, 9.99
"#;

    let doc = parse_hedl(hedl);
    let schema_str = generate_schema(&doc, &SchemaConfig::default()).unwrap();

    // Should be valid JSON
    let parsed: JsonValue = serde_json::from_str(&schema_str).unwrap();
    assert!(parsed.is_object());

    // Should be pretty-printed (contains newlines)
    assert!(schema_str.contains('\n'));
}

#[test]
fn test_roundtrip_schema_generation() {
    let hedl = r#"
%STRUCT: User: [id, name, age]
users: @User
  |u1, Alice, 30
"#;

    let doc = parse_hedl(hedl);

    // Generate schema
    let schema_str = generate_schema(&doc, &SchemaConfig::default()).unwrap();

    // Parse back to JSON
    let schema_json: JsonValue = serde_json::from_str(&schema_str).unwrap();

    // Validate
    assert!(validate_schema(&schema_json).is_ok());

    // Re-serialize should produce valid JSON
    let reserialize = serde_json::to_string_pretty(&schema_json).unwrap();
    let reparsed: JsonValue = serde_json::from_str(&reserialize).unwrap();
    assert_eq!(schema_json, reparsed);
}

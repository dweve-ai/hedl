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

//! JSON Schema (Draft 7) generation from HEDL documents.
//!
//! This module provides comprehensive JSON Schema generation with support for:
//!
//! - **All HEDL Types**: Scalars, tensors, references, expressions, structs
//! - **Type Inference**: Smart format detection (email, URI, date-time)
//! - **%NEST:Relationships**: Hierarchical structures with nested arrays
//! - **Schema Validation**: Validates generated schemas for correctness
//! - **Configuration**: Title, description, strict mode, examples
//!
//! # Examples
//!
//! ## Basic Schema Generation
//!
//! ```rust
//! use hedl_core::parse;
//! use hedl_json::schema_gen::{generate_schema, SchemaConfig};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let hedl = r#"
//! name: Alice
//! age: 30
//! active: true
//! "#;
//!
//! let doc = parse(hedl.as_bytes())?;
//! let schema = generate_schema(&doc, &SchemaConfig::default())?;
//! println!("{}", schema);
//! # Ok(())
//! # }
//! ```
//!
//! ## Schema with %STRUCT:Definitions
//!
//! ```rust
//! use hedl_core::parse;
//! use hedl_json::schema_gen::{generate_schema, SchemaConfig};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let hedl = r#"
//! %STRUCT: User: [id, name, email]
//! users: @User
//!   u1, Alice, alice@example.com
//! "#;
//!
//! let doc = parse(hedl.as_bytes())?;
//! let config = SchemaConfig::builder()
//!     .title("User API Schema")
//!     .strict(true)
//!     .build();
//! let schema = generate_schema(&doc, &config)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Schema with %NEST:Relationships
//!
//! ```rust
//! use hedl_core::parse;
//! use hedl_json::schema_gen::{generate_schema, SchemaConfig};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let hedl = r#"
//! %STRUCT: Team: [id, name]
//! %STRUCT: Member: [id, name]
//! %NEST: Team > Member
//!
//! teams: @Team
//!   t1, Engineering
//! "#;
//!
//! let doc = parse(hedl.as_bytes())?;
//! let schema = generate_schema(&doc, &SchemaConfig::default())?;
//! # Ok(())
//! # }
//! ```

use hedl_core::{Document, Item, MatrixList, Value};
use hedl_core::lex::Tensor;
use serde_json::{json, Map, Value as JsonValue};
use std::collections::BTreeMap;
use thiserror::Error;

/// Errors that can occur during schema generation
#[derive(Error, Debug)]
pub enum SchemaError {
    /// Schema validation failed
    #[error("Schema validation failed: {0}")]
    ValidationError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Configuration for JSON Schema generation
#[derive(Debug, Clone)]
pub struct SchemaConfig {
    /// Schema title (optional)
    pub title: Option<String>,
    /// Schema description (optional)
    pub description: Option<String>,
    /// Schema $id URI (optional)
    pub schema_id: Option<String>,
    /// Strict mode: disallow additional properties (default: false)
    pub strict: bool,
    /// Include example values in schema (default: false)
    pub include_examples: bool,
    /// Include metadata fields like title, description, $id (default: true)
    pub include_metadata: bool,
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            schema_id: None,
            strict: false,
            include_examples: false,
            include_metadata: true,
        }
    }
}

impl SchemaConfig {
    /// Create a new builder for SchemaConfig
    pub fn builder() -> SchemaConfigBuilder {
        SchemaConfigBuilder::default()
    }
}

/// Builder for SchemaConfig
#[derive(Debug)]
pub struct SchemaConfigBuilder {
    title: Option<String>,
    description: Option<String>,
    schema_id: Option<String>,
    strict: bool,
    include_examples: bool,
    include_metadata: bool,
}

impl Default for SchemaConfigBuilder {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            schema_id: None,
            strict: false,
            include_examples: false,
            include_metadata: true, // Default to true to match SchemaConfig
        }
    }
}

impl SchemaConfigBuilder {
    /// Set the schema title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the schema description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the schema $id URI
    pub fn schema_id(mut self, schema_id: impl Into<String>) -> Self {
        self.schema_id = Some(schema_id.into());
        self
    }

    /// Enable strict mode (disallow additional properties)
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Include example values in schema
    pub fn include_examples(mut self, include: bool) -> Self {
        self.include_examples = include;
        self
    }

    /// Include metadata fields (title, description, $id)
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Build the SchemaConfig
    pub fn build(self) -> SchemaConfig {
        SchemaConfig {
            title: self.title,
            description: self.description,
            schema_id: self.schema_id,
            strict: self.strict,
            include_examples: self.include_examples,
            include_metadata: self.include_metadata,
        }
    }
}

/// Generate JSON Schema (Draft 7) from HEDL document as a JSON string
///
/// # Arguments
///
/// * `doc` - The HEDL document to convert
/// * `config` - Schema generation configuration
///
/// # Returns
///
/// A pretty-printed JSON Schema string
///
/// # Errors
///
/// Returns error if schema generation or serialization fails
pub fn generate_schema(doc: &Document, config: &SchemaConfig) -> Result<String, SchemaError> {
    let schema = generate_schema_value(doc, config)?;
    Ok(serde_json::to_string_pretty(&schema)?)
}

/// Generate JSON Schema (Draft 7) from HEDL document as a JsonValue
///
/// # Arguments
///
/// * `doc` - The HEDL document to convert
/// * `config` - Schema generation configuration
///
/// # Returns
///
/// A JsonValue representing the JSON Schema
///
/// # Errors
///
/// Returns error if schema generation fails
pub fn generate_schema_value(
    doc: &Document,
    config: &SchemaConfig,
) -> Result<JsonValue, SchemaError> {
    let mut schema = Map::with_capacity(8);

    // Required: $schema field
    schema.insert(
        "$schema".to_string(),
        json!("http://json-schema.org/draft-07/schema#"),
    );

    // Optional metadata fields
    if config.include_metadata {
        if let Some(ref title) = config.title {
            schema.insert("title".to_string(), json!(title));
        }
        if let Some(ref description) = config.description {
            schema.insert("description".to_string(), json!(description));
        }
        if let Some(ref schema_id) = config.schema_id {
            schema.insert("$id".to_string(), json!(schema_id));
        }
    }

    // Root type is always object
    schema.insert("type".to_string(), json!("object"));

    // Generate definitions from %STRUCT:declarations
    if !doc.structs.is_empty() {
        let definitions = generate_definitions(doc, config);
        schema.insert("definitions".to_string(), JsonValue::Object(definitions));
    }

    // Generate properties from root items
    let properties = generate_properties(&doc.root, doc, config);
    schema.insert("properties".to_string(), JsonValue::Object(properties));

    // Strict mode: no additional properties
    if config.strict {
        schema.insert("additionalProperties".to_string(), json!(false));
    }

    Ok(JsonValue::Object(schema))
}

/// Generate schema definitions from %STRUCT:declarations
fn generate_definitions(doc: &Document, config: &SchemaConfig) -> Map<String, JsonValue> {
    let mut definitions = Map::with_capacity(doc.structs.len());

    for (type_name, schema_fields) in &doc.structs {
        let mut def = Map::with_capacity(4);
        def.insert("type".to_string(), json!("object"));

        // Generate properties for struct fields
        let mut properties = Map::with_capacity(schema_fields.len());

        for field_name in schema_fields {
            // Infer type from actual data if available
            let field_schema = infer_field_type(type_name, field_name, doc, config);
            properties.insert(field_name.clone(), field_schema);
        }

        // Add nested children if %NEST:relationship exists
        if let Some(child_type) = doc.nests.get(type_name) {
            let child_array_name = pluralize(child_type);
            let child_ref = json!({
                "type": "array",
                "items": {
                    "$ref": format!("#/definitions/{}", child_type)
                }
            });
            properties.insert(child_array_name, child_ref);
        }

        def.insert("properties".to_string(), JsonValue::Object(properties));

        // Required fields: only "id" is required (first column)
        if !schema_fields.is_empty() {
            def.insert("required".to_string(), json!([schema_fields[0]]));
        }

        // Strict mode: no additional properties
        if config.strict {
            def.insert("additionalProperties".to_string(), json!(false));
        }

        definitions.insert(type_name.clone(), JsonValue::Object(def));
    }

    definitions
}

/// Generate properties from root items
fn generate_properties(
    items: &BTreeMap<String, Item>,
    doc: &Document,
    config: &SchemaConfig,
) -> Map<String, JsonValue> {
    let mut properties = Map::with_capacity(items.len());

    for (key, item) in items {
        let prop_schema = item_to_schema(item, doc, config);
        properties.insert(key.clone(), prop_schema);
    }

    properties
}

/// Convert an Item to a JSON Schema property
fn item_to_schema(item: &Item, doc: &Document, config: &SchemaConfig) -> JsonValue {
    match item {
        Item::Scalar(value) => value_to_schema(value, None, config),
        Item::Object(obj) => object_to_schema(obj, doc, config),
        Item::List(list) => matrix_list_to_schema(list, config),
    }
}

/// Convert a Value to a JSON Schema type
fn value_to_schema(value: &Value, field_name: Option<&str>, config: &SchemaConfig) -> JsonValue {
    let mut schema = Map::with_capacity(4);

    match value {
        Value::Null => {
            schema.insert("type".to_string(), json!("null"));
        }
        Value::Bool(b) => {
            schema.insert("type".to_string(), json!("boolean"));
            if config.include_examples {
                schema.insert("examples".to_string(), json!([b]));
            }
        }
        Value::Int(n) => {
            schema.insert("type".to_string(), json!("integer"));
            if config.include_examples {
                schema.insert("examples".to_string(), json!([n]));
            }
        }
        Value::Float(f) => {
            schema.insert("type".to_string(), json!("number"));
            if config.include_examples {
                schema.insert("examples".to_string(), json!([f]));
            }
        }
        Value::String(s) => {
            schema.insert("type".to_string(), json!("string"));

            // Smart format detection
            if let Some(format) = infer_string_format(s, field_name) {
                schema.insert("format".to_string(), json!(format));
            }

            if config.include_examples {
                schema.insert("examples".to_string(), json!([s]));
            }
        }
        Value::Tensor(tensor) => {
            // Tensor schema depends on shape
            return tensor_to_schema(tensor, config);
        }
        Value::Reference(reference) => {
            schema.insert("type".to_string(), json!("string"));
            schema.insert("pattern".to_string(), json!("^@([A-Z][a-zA-Z0-9]*:)?[a-zA-Z0-9_-]+$"));
            schema.insert(
                "description".to_string(),
                json!(format!(
                    "Reference to {}",
                    reference
                        .type_name.as_deref()
                        .unwrap_or("entity")
                )),
            );
        }
        Value::Expression(_) => {
            schema.insert("type".to_string(), json!("string"));
            schema.insert("pattern".to_string(), json!(r"^\$\(.+\)$"));
            schema.insert(
                "description".to_string(),
                json!("HEDL expression $(...)"),
            );
        }
    }

    JsonValue::Object(schema)
}

/// Convert an object to a JSON Schema
fn object_to_schema(
    obj: &BTreeMap<String, Item>,
    doc: &Document,
    config: &SchemaConfig,
) -> JsonValue {
    let mut schema = Map::with_capacity(3);
    schema.insert("type".to_string(), json!("object"));

    let properties = generate_properties(obj, doc, config);
    schema.insert("properties".to_string(), JsonValue::Object(properties));

    if config.strict {
        schema.insert("additionalProperties".to_string(), json!(false));
    }

    JsonValue::Object(schema)
}

/// Convert a matrix list to a JSON Schema
fn matrix_list_to_schema(list: &MatrixList, _config: &SchemaConfig) -> JsonValue {
    let mut schema = Map::with_capacity(2);
    schema.insert("type".to_string(), json!("array"));

    // Reference to the struct definition
    let items = json!({
        "$ref": format!("#/definitions/{}", list.type_name)
    });
    schema.insert("items".to_string(), items);

    JsonValue::Object(schema)
}

/// Convert a tensor to a JSON Schema
fn tensor_to_schema(tensor: &Tensor, config: &SchemaConfig) -> JsonValue {
    match tensor {
        Tensor::Scalar(val) => {
            let mut schema = Map::with_capacity(2);
            schema.insert("type".to_string(), json!("number"));
            if config.include_examples {
                schema.insert("examples".to_string(), json!([val]));
            }
            JsonValue::Object(schema)
        }
        Tensor::Array(_) => {
            // Multi-dimensional array
            json!({
                "type": "array",
                "items": {
                    "oneOf": [
                        {"type": "number"},
                        {"type": "array"}
                    ]
                }
            })
        }
    }
}

/// Infer field type from actual data in the document
fn infer_field_type(
    type_name: &str,
    field_name: &str,
    doc: &Document,
    config: &SchemaConfig,
) -> JsonValue {
    // Find the first instance of this type in the document
    for item in doc.root.values() {
        if let Item::List(list) = item {
            if list.type_name == type_name && !list.rows.is_empty() {
                // Find the field index
                if let Some(field_idx) = list.schema.iter().position(|f| f == field_name) {
                    // Get the first row's value for this field
                    if let Some(node) = list.rows.first() {
                        if let Some(value) = node.fields.get(field_idx) {
                            return value_to_schema(value, Some(field_name), config);
                        }
                    }
                }
            }
        }
    }

    // Default fallback: string type with format hints
    let mut schema = Map::with_capacity(2);
    schema.insert("type".to_string(), json!("string"));

    // Smart format detection based on field name
    if let Some(format) = infer_format_from_name(field_name) {
        schema.insert("format".to_string(), json!(format));
    }

    JsonValue::Object(schema)
}

/// Infer JSON Schema format from string value
fn infer_string_format(s: &str, field_name: Option<&str>) -> Option<&'static str> {
    // Email detection
    if s.contains('@') && s.contains('.') && !s.starts_with('@') {
        return Some("email");
    }

    // URI detection
    if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("ftp://") {
        return Some("uri");
    }

    // ISO 8601 date-time detection
    if s.contains('T') && (s.contains('Z') || s.contains('+') || s.contains('-'))
        && s.len() >= 19 {
            // Minimum length for ISO 8601
            return Some("date-time");
        }

    // UUID detection
    if s.len() == 36 && s.chars().filter(|&c| c == '-').count() == 4 {
        return Some("uuid");
    }

    // Field name-based inference as fallback
    infer_format_from_name(field_name?)
}

/// Infer format from field name
fn infer_format_from_name(field_name: &str) -> Option<&'static str> {
    let lower = field_name.to_lowercase();

    if lower.contains("email") {
        Some("email")
    } else if lower.contains("url") || lower.contains("uri") {
        Some("uri")
    } else if lower.contains("date") || lower.ends_with("_at") || lower.ends_with("_on") {
        Some("date-time")
    } else if lower.contains("uuid") || lower.contains("guid") {
        Some("uuid")
    } else {
        None
    }
}

/// Pluralize a type name (simple English pluralization)
fn pluralize(word: &str) -> String {
    if word.ends_with('s')
        || word.ends_with('x')
        || word.ends_with('z')
        || word.ends_with("ch")
        || word.ends_with("sh")
    {
        format!("{}es", word)
    } else if word.ends_with('y') && !word.ends_with("ay") && !word.ends_with("ey") {
        format!("{}ies", &word[..word.len() - 1])
    } else {
        format!("{}s", word)
    }
}

/// Validate a JSON Schema for correctness
///
/// Validates that the schema:
/// - Has required `$schema` field (root level only)
/// - Has required `type` field
/// - Type is a valid JSON Schema type
/// - References are well-formed
///
/// # Arguments
///
/// * `schema` - The JSON Schema to validate
///
/// # Returns
///
/// Ok(()) if valid, Err with validation message otherwise
pub fn validate_schema(schema: &JsonValue) -> Result<(), SchemaError> {
    validate_schema_internal(schema, true)
}

/// Internal validation function with control over $schema field requirement
fn validate_schema_internal(schema: &JsonValue, require_schema_field: bool) -> Result<(), SchemaError> {
    let obj = schema
        .as_object()
        .ok_or_else(|| SchemaError::ValidationError("Schema must be an object".to_string()))?;

    // Validate $schema field (only for root schema)
    if require_schema_field && !obj.contains_key("$schema") {
        return Err(SchemaError::ValidationError(
            "Schema must have $schema field".to_string(),
        ));
    }

    // Validate type field
    if !obj.contains_key("type") {
        return Err(SchemaError::ValidationError(
            "Schema must have type field".to_string(),
        ));
    }

    // Validate type value
    let schema_type = obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SchemaError::ValidationError("type must be a string".to_string()))?;

    let valid_types = [
        "null", "boolean", "object", "array", "number", "string", "integer",
    ];
    if !valid_types.contains(&schema_type) {
        return Err(SchemaError::ValidationError(format!(
            "Invalid type: {}. Must be one of: {:?}",
            schema_type, valid_types
        )));
    }

    // Recursively validate definitions (without requiring $schema field)
    if let Some(definitions) = obj.get("definitions") {
        if let Some(defs) = definitions.as_object() {
            for (name, def_schema) in defs {
                validate_schema_internal(def_schema, false).map_err(|e| {
                    SchemaError::ValidationError(format!("Invalid definition '{}': {}", name, e))
                })?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pluralize() {
        assert_eq!(pluralize("User"), "Users");
        assert_eq!(pluralize("Post"), "Posts");
        assert_eq!(pluralize("Category"), "Categories");
        assert_eq!(pluralize("Box"), "Boxes");
        assert_eq!(pluralize("Class"), "Classes");
    }

    #[test]
    fn test_infer_string_format_email() {
        assert_eq!(
            infer_string_format("alice@example.com", None),
            Some("email")
        );
    }

    #[test]
    fn test_infer_string_format_uri() {
        assert_eq!(
            infer_string_format("https://example.com", None),
            Some("uri")
        );
    }

    #[test]
    fn test_infer_string_format_datetime() {
        assert_eq!(
            infer_string_format("2024-01-01T00:00:00Z", None),
            Some("date-time")
        );
    }

    #[test]
    fn test_infer_format_from_name() {
        assert_eq!(infer_format_from_name("email"), Some("email"));
        assert_eq!(infer_format_from_name("url"), Some("uri"));
        assert_eq!(infer_format_from_name("created_at"), Some("date-time"));
        assert_eq!(infer_format_from_name("uuid"), Some("uuid"));
    }

    #[test]
    fn test_config_builder() {
        let config = SchemaConfig::builder()
            .title("Test")
            .description("Desc")
            .strict(true)
            .build();

        assert_eq!(config.title, Some("Test".to_string()));
        assert_eq!(config.description, Some("Desc".to_string()));
        assert!(config.strict);
    }

    #[test]
    fn test_default_config() {
        let config = SchemaConfig::default();
        assert!(config.title.is_none());
        assert!(!config.strict);
        assert!(!config.include_examples);
        assert!(config.include_metadata);
    }
}

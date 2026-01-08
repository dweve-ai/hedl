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

//! Declarative macros for JSON schema generation.
//!
//! This module provides a comprehensive macro system to reduce boilerplate
//! in schema definitions for MCP tools. The macros support common patterns:
//! - String arguments (HEDL content, file paths, formats)
//! - Boolean flags (ditto, strict, recursive, etc.)
//! - Integer parameters (limit, offset, size constraints)
//! - Optional JSON objects (format-specific options)
//! - Enumerated types (format choices, tokenizers)
//!
//! # Examples
//!
//! ```text
//! use crate::tools::schema_macros::*;
//!
//! // Simple string argument with description
//! let schema = schema_object! {
//!     hedl: schema_string!("HEDL document content to validate")
//! };
//!
//! // String with enum constraints
//! let schema = schema_object! {
//!     format: schema_enum!(["json", "yaml", "csv"], "Target output format")
//! };
//!
//! // Boolean with default
//! let schema = schema_object! {
//!     strict: schema_bool!("Enable strict validation", default: true)
//! };
//!
//! // Complete tool schema with required fields
//! let schema = tool_schema! {
//!     required: ["hedl", "format"],
//!     properties: {
//!         hedl: schema_string!("HEDL content"),
//!         format: schema_enum!(["json", "yaml"], "Output format"),
//!         ditto: schema_bool!("Use ditto optimization", default: true)
//!     }
//! };
//! ```

/// Generate a JSON schema object with type "string" and description.
///
/// # Usage
/// ```text
/// schema_string!("Description of the string field")
/// schema_string!("File path", pattern: r"\.hedl$")
/// ```
#[macro_export]
macro_rules! schema_string {
    ($description:expr) => {
        serde_json::json!({
            "type": "string",
            "description": $description
        })
    };
    ($description:expr, pattern: $pattern:expr) => {
        serde_json::json!({
            "type": "string",
            "description": $description,
            "pattern": $pattern
        })
    };
}

/// Generate a JSON schema object with type "boolean" and optional default.
///
/// # Usage
/// ```text
/// schema_bool!("Enable feature")
/// schema_bool!("Enable strict mode", default: true)
/// schema_bool!("Disable validation", default: false)
/// ```
#[macro_export]
macro_rules! schema_bool {
    ($description:expr) => {
        serde_json::json!({
            "type": "boolean",
            "description": $description
        })
    };
    ($description:expr, default: $default:expr) => {
        serde_json::json!({
            "type": "boolean",
            "description": $description,
            "default": $default
        })
    };
}

/// Generate a JSON schema object with type "integer" and optional constraints.
///
/// # Usage
/// ```text
/// schema_integer!("Number of items")
/// schema_integer!("Maximum rows", minimum: 1, maximum: 1000)
/// schema_integer!("Page size", default: 100)
/// ```
#[macro_export]
macro_rules! schema_integer {
    ($description:expr) => {
        serde_json::json!({
            "type": "integer",
            "description": $description
        })
    };
    ($description:expr, default: $default:expr) => {
        serde_json::json!({
            "type": "integer",
            "description": $description,
            "default": $default
        })
    };
    ($description:expr, minimum: $min:expr, maximum: $max:expr) => {
        serde_json::json!({
            "type": "integer",
            "description": $description,
            "minimum": $min,
            "maximum": $max
        })
    };
}

/// Generate a JSON schema object with string enum constraints.
///
/// # Usage
/// ```text
/// schema_enum!(["json", "yaml", "csv"], "Output format")
/// schema_enum!(["simple", "cl100k"], "Tokenizer algorithm", default: "simple")
/// ```
#[macro_export]
macro_rules! schema_enum {
    ([$($variant:expr),+ $(,)?], $description:expr) => {
        serde_json::json!({
            "type": "string",
            "enum": [$($variant),+],
            "description": $description
        })
    };
    ([$($variant:expr),+ $(,)?], $description:expr, default: $default:expr) => {
        serde_json::json!({
            "type": "string",
            "enum": [$($variant),+],
            "description": $description,
            "default": $default
        })
    };
}

/// Generate a JSON schema object for optional format-specific options.
///
/// # Usage
/// ```text
/// schema_options! {
///     pretty: schema_bool!("Pretty-print output (json)"),
///     delimiter: schema_string!("Field delimiter (csv)")
/// }
/// ```
#[macro_export]
macro_rules! schema_options {
    ($($field:ident: $schema:expr),+ $(,)?) => {
        serde_json::json!({
            "type": "object",
            "description": "Format-specific options",
            "properties": {
                $(stringify!($field): $schema),+
            }
        })
    };
}

/// Generate a JSON schema object with string array type.
///
/// # Usage
/// ```text
/// schema_string_array!("List of column names")
/// schema_string_array!("Field names for schema inference", items_pattern: r"^[a-z_]+$")
/// ```
#[macro_export]
macro_rules! schema_string_array {
    ($description:expr) => {
        serde_json::json!({
            "type": "array",
            "items": { "type": "string" },
            "description": $description
        })
    };
    ($description:expr, items_pattern: $pattern:expr) => {
        serde_json::json!({
            "type": "array",
            "items": {
                "type": "string",
                "pattern": $pattern
            },
            "description": $description
        })
    };
}

/// Generate a complete tool schema with properties and required fields.
///
/// This is the top-level macro for defining entire tool schemas.
///
/// # Usage
/// ```text
/// tool_schema! {
///     required: ["hedl"],
///     properties: {
///         hedl: schema_string!("HEDL document content"),
///         format: schema_enum!(["json", "yaml"], "Output format"),
///         ditto: schema_bool!("Use ditto optimization", default: true)
///     }
/// }
/// ```
#[macro_export]
macro_rules! tool_schema {
    (
        required: [$($req:expr),* $(,)?],
        properties: {
            $($field:ident: $schema:expr),+ $(,)?
        }
    ) => {
        serde_json::json!({
            "type": "object",
            "properties": {
                $(stringify!($field): $schema),+
            },
            "required": [$($req),*]
        })
    };
}

/// Generate schema for HEDL content argument (with size validation).
///
/// This is a specialized macro for the common "hedl" string argument.
///
/// # Usage
/// ```text
/// hedl_content_arg!()
/// hedl_content_arg!("Custom description")
/// ```
#[macro_export]
macro_rules! hedl_content_arg {
    () => {
        $crate::schema_string!("HEDL document content")
    };
    ($description:expr) => {
        $crate::schema_string!($description)
    };
}

/// Generate schema for file path argument.
///
/// # Usage
/// ```text
/// path_arg!()
/// path_arg!("Output file path")
/// ```
#[macro_export]
macro_rules! path_arg {
    () => {
        $crate::schema_string!("File or directory path")
    };
    ($description:expr) => {
        $crate::schema_string!($description)
    };
}

/// Generate schema for format argument with enum values.
///
/// # Usage
/// ```text
/// format_arg!(["json", "yaml", "csv"])
/// format_arg!(["json", "yaml", "csv"], "Source format to convert from")
/// ```
#[macro_export]
macro_rules! format_arg {
    ([$($variant:expr),+ $(,)?]) => {
        $crate::schema_enum!([$($variant),+], "Format type")
    };
    ([$($variant:expr),+ $(,)?], $description:expr) => {
        $crate::schema_enum!([$($variant),+], $description)
    };
}

/// Generate schema for validation arguments (strict, lint).
///
/// # Usage
/// ```text
/// validation_args!()
/// ```
#[macro_export]
macro_rules! validation_args {
    () => {
        (
            $crate::schema_bool!(
                "Enable strict validation mode: treat lint warnings as errors",
                default: true
            ),
            $crate::schema_bool!(
                "Run linting rules in addition to parsing",
                default: true
            )
        )
    };
}

/// Generate schema for pagination arguments (limit, offset).
///
/// # Usage
/// ```text
/// pagination_args!()
/// pagination_args!(default_limit: 50)
/// ```
#[macro_export]
macro_rules! pagination_args {
    () => {
        (
            $crate::schema_integer!(
                "Maximum number of entities to return",
                default: 100
            ),
            $crate::schema_integer!(
                "Number of entities to skip",
                default: 0
            )
        )
    };
    (default_limit: $limit:expr) => {
        (
            $crate::schema_integer!(
                "Maximum number of entities to return",
                default: $limit
            ),
            $crate::schema_integer!(
                "Number of entities to skip",
                default: 0
            )
        )
    };
}

/// Generate schema for file operation arguments (validate, format, backup).
///
/// # Usage
/// ```text
/// file_write_args!()
/// ```
#[macro_export]
macro_rules! file_write_args {
    () => {
        (
            $crate::schema_bool!(
                "Validate HEDL before writing",
                default: true
            ),
            $crate::schema_bool!(
                "Format/canonicalize before writing",
                default: false
            ),
            $crate::schema_bool!(
                "Create backup of existing file if it exists",
                default: true
            )
        )
    };
}

/// Generate schema for ditto optimization argument.
///
/// # Usage
/// ```text
/// ditto_arg!()
/// ditto_arg!("Apply ditto optimization for repeated values")
/// ```
#[macro_export]
macro_rules! ditto_arg {
    () => {
        $crate::schema_bool!(
            "Enable ditto optimization for repeated values",
            default: true
        )
    };
    ($description:expr) => {
        $crate::schema_bool!($description, default: true)
    };
}

/// Generate schema for conversion options (to format).
///
/// # Usage
/// ```text
/// convert_to_options!()
/// ```
#[macro_export]
macro_rules! convert_to_options {
    () => {
        $crate::schema_options! {
            pretty: $crate::schema_bool!("Pretty-print output (json)"),
            include_headers: $crate::schema_bool!("Include headers (csv)"),
            use_merge: $crate::schema_bool!("Use MERGE vs CREATE (cypher)"),
            include_constraints: $crate::schema_bool!("Include constraints (cypher)")
        }
    };
}

/// Generate schema for conversion options (from format).
///
/// # Usage
/// ```text
/// convert_from_options!()
/// ```
#[macro_export]
macro_rules! convert_from_options {
    () => {
        $crate::schema_options! {
            type_name: $crate::schema_string!("Type name for entities (csv)"),
            schema: $crate::schema_string_array!("Column names (csv)"),
            delimiter: $crate::schema_string!("Field delimiter (csv)")
        }
    };
}

#[cfg(test)]
mod tests {
    use serde_json::Value as JsonValue;

    #[test]
    fn test_schema_string() {
        let schema = schema_string!("Test description");
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["description"], "Test description");
        assert!(schema.get("pattern").is_none());
    }

    #[test]
    fn test_schema_string_with_pattern() {
        let schema = schema_string!("HEDL file", pattern: r"\.hedl$");
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["description"], "HEDL file");
        assert_eq!(schema["pattern"], r"\.hedl$");
    }

    #[test]
    fn test_schema_bool() {
        let schema = schema_bool!("Enable feature");
        assert_eq!(schema["type"], "boolean");
        assert_eq!(schema["description"], "Enable feature");
        assert!(schema.get("default").is_none());
    }

    #[test]
    fn test_schema_bool_with_default() {
        let schema = schema_bool!("Enable strict mode", default: true);
        assert_eq!(schema["type"], "boolean");
        assert_eq!(schema["description"], "Enable strict mode");
        assert_eq!(schema["default"], true);
    }

    #[test]
    fn test_schema_integer() {
        let schema = schema_integer!("Row count");
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["description"], "Row count");
    }

    #[test]
    fn test_schema_integer_with_default() {
        let schema = schema_integer!("Page size", default: 50);
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["default"], 50);
    }

    #[test]
    fn test_schema_integer_with_range() {
        let schema = schema_integer!("Port number", minimum: 1, maximum: 65535);
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["minimum"], 1);
        assert_eq!(schema["maximum"], 65535);
    }

    #[test]
    fn test_schema_enum() {
        let schema = schema_enum!(["json", "yaml", "csv"], "Output format");
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["description"], "Output format");
        let enum_vals = schema["enum"].as_array().unwrap();
        assert_eq!(enum_vals.len(), 3);
        assert!(enum_vals.contains(&JsonValue::String("json".to_string())));
    }

    #[test]
    fn test_schema_enum_with_default() {
        let schema = schema_enum!(["simple", "cl100k"], "Tokenizer", default: "simple");
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["default"], "simple");
    }

    #[test]
    fn test_schema_options() {
        let schema = schema_options! {
            pretty: schema_bool!("Pretty output"),
            indent: schema_integer!("Indent size")
        };
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["description"], "Format-specific options");
        assert!(schema["properties"].get("pretty").is_some());
        assert!(schema["properties"].get("indent").is_some());
    }

    #[test]
    fn test_schema_string_array() {
        let schema = schema_string_array!("Column names");
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "string");
        assert_eq!(schema["description"], "Column names");
    }

    #[test]
    fn test_tool_schema() {
        let schema = tool_schema! {
            required: ["hedl", "format"],
            properties: {
                hedl: schema_string!("HEDL content"),
                format: schema_enum!(["json", "yaml"], "Format"),
                ditto: schema_bool!("Use ditto", default: true)
            }
        };
        assert_eq!(schema["type"], "object");
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 2);
        assert!(schema["properties"].get("hedl").is_some());
        assert!(schema["properties"].get("format").is_some());
        assert!(schema["properties"].get("ditto").is_some());
    }

    #[test]
    fn test_hedl_content_arg() {
        let schema = hedl_content_arg!();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["description"], "HEDL document content");

        let custom = hedl_content_arg!("Custom HEDL description");
        assert_eq!(custom["description"], "Custom HEDL description");
    }

    #[test]
    fn test_path_arg() {
        let schema = path_arg!();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["description"], "File or directory path");

        let custom = path_arg!("Output directory");
        assert_eq!(custom["description"], "Output directory");
    }

    #[test]
    fn test_format_arg() {
        let schema = format_arg!(["json", "yaml", "csv"]);
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["description"], "Format type");
        let enum_vals = schema["enum"].as_array().unwrap();
        assert_eq!(enum_vals.len(), 3);
    }

    #[test]
    fn test_validation_args() {
        let (strict, lint) = validation_args!();
        assert_eq!(strict["type"], "boolean");
        assert_eq!(strict["default"], true);
        assert_eq!(lint["type"], "boolean");
        assert_eq!(lint["default"], true);
    }

    #[test]
    fn test_pagination_args() {
        let (limit, offset) = pagination_args!();
        assert_eq!(limit["type"], "integer");
        assert_eq!(limit["default"], 100);
        assert_eq!(offset["type"], "integer");
        assert_eq!(offset["default"], 0);

        let (custom_limit, _) = pagination_args!(default_limit: 50);
        assert_eq!(custom_limit["default"], 50);
    }

    #[test]
    fn test_file_write_args() {
        let (validate, format, backup) = file_write_args!();
        assert_eq!(validate["default"], true);
        assert_eq!(format["default"], false);
        assert_eq!(backup["default"], true);
    }

    #[test]
    fn test_ditto_arg() {
        let schema = ditto_arg!();
        assert_eq!(schema["type"], "boolean");
        assert_eq!(schema["default"], true);
        assert!(schema["description"].as_str().unwrap().contains("ditto"));
    }

    #[test]
    fn test_convert_to_options() {
        let schema = convert_to_options!();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].get("pretty").is_some());
        assert!(schema["properties"].get("include_headers").is_some());
        assert!(schema["properties"].get("use_merge").is_some());
        assert!(schema["properties"].get("include_constraints").is_some());
    }

    #[test]
    fn test_convert_from_options() {
        let schema = convert_from_options!();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].get("type_name").is_some());
        assert!(schema["properties"].get("schema").is_some());
        assert!(schema["properties"].get("delimiter").is_some());
    }
}

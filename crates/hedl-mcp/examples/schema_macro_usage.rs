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

//! Schema Macro Usage Examples
//!
//! This example demonstrates how to use the schema macro system to create
//! JSON schemas for MCP tools with minimal boilerplate.

#[macro_use]
extern crate hedl_mcp;

/// Example 1: Basic String Schema
fn example_basic_string() {
    let schema = schema_string!("A simple string field");

    println!("Basic String Schema:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Output:
    // {
    //   "type": "string",
    //   "description": "A simple string field"
    // }
}

/// Example 2: String with Pattern Validation
fn example_string_with_pattern() {
    let schema = schema_string!("Email address", pattern: r"^[^@]+@[^@]+\.[^@]+$");

    println!("String with Pattern:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Output:
    // {
    //   "type": "string",
    //   "description": "Email address",
    //   "pattern": "^[^@]+@[^@]+\\.[^@]+$"
    // }
}

/// Example 3: Boolean with Default
fn example_boolean_with_default() {
    let schema = schema_bool!("Enable debug mode", default: false);

    println!("Boolean with Default:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Output:
    // {
    //   "type": "boolean",
    //   "description": "Enable debug mode",
    //   "default": false
    // }
}

/// Example 4: Integer with Range Constraints
fn example_integer_with_range() {
    let schema = schema_integer!("Server port", minimum: 1024, maximum: 65535);

    println!("Integer with Range:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Output:
    // {
    //   "type": "integer",
    //   "description": "Server port",
    //   "minimum": 1024,
    //   "maximum": 65535
    // }
}

/// Example 5: Enumerated String
fn example_enum() {
    let schema = schema_enum!(["json", "yaml", "toml", "xml"], "Configuration format");

    println!("Enumerated String:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Output:
    // {
    //   "type": "string",
    //   "enum": ["json", "yaml", "toml", "xml"],
    //   "description": "Configuration format"
    // }
}

/// Example 6: String Array
fn example_string_array() {
    let schema = schema_string_array!("List of allowed origins");

    println!("String Array:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Output:
    // {
    //   "type": "array",
    //   "items": { "type": "string" },
    //   "description": "List of allowed origins"
    // }
}

/// Example 7: Options Object
fn example_options_object() {
    let schema = schema_options! {
        timeout: schema_integer!("Request timeout in seconds", default: 30),
        retry: schema_bool!("Enable automatic retries", default: true),
        max_retries: schema_integer!("Maximum retry attempts", default: 3)
    };

    println!("Options Object:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Output:
    // {
    //   "type": "object",
    //   "description": "Format-specific options",
    //   "properties": {
    //     "timeout": { ... },
    //     "retry": { ... },
    //     "max_retries": { ... }
    //   }
    // }
}

/// Example 8: Complete Tool Schema
fn example_complete_tool_schema() {
    let schema = tool_schema! {
        required: ["input", "format"],
        properties: {
            input: schema_string!("Input data to process"),
            format: schema_enum!(["json", "yaml"], "Output format"),
            verbose: schema_bool!("Enable verbose output", default: false),
            max_items: schema_integer!("Maximum items to process", default: 100)
        }
    };

    println!("Complete Tool Schema:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Output:
    // {
    //   "type": "object",
    //   "properties": { ... },
    //   "required": ["input", "format"]
    // }
}

/// Example 9: Using Domain-Specific Macros
fn example_domain_specific_macros() {
    // HEDL content argument
    let hedl_arg = hedl_content_arg!();
    println!("HEDL Content Arg:");
    println!("{}\n", serde_json::to_string_pretty(&hedl_arg).unwrap());

    // Path argument
    let path_arg = path_arg!("Output directory");
    println!("Path Arg:");
    println!("{}\n", serde_json::to_string_pretty(&path_arg).unwrap());

    // Format argument
    let format_arg = format_arg!(["json", "csv", "parquet"]);
    println!("Format Arg:");
    println!("{}\n", serde_json::to_string_pretty(&format_arg).unwrap());

    // Ditto optimization argument
    let ditto_arg = ditto_arg!();
    println!("Ditto Arg:");
    println!("{}\n", serde_json::to_string_pretty(&ditto_arg).unwrap());
}

/// Example 10: Using Multi-Field Macros
fn example_multi_field_macros() {
    // Validation arguments
    let (strict, lint) = validation_args!();
    println!("Validation Args:");
    println!("strict: {}", serde_json::to_string_pretty(&strict).unwrap());
    println!("lint: {}\n", serde_json::to_string_pretty(&lint).unwrap());

    // Pagination arguments
    let (limit, offset) = pagination_args!();
    println!("Pagination Args:");
    println!("limit: {}", serde_json::to_string_pretty(&limit).unwrap());
    println!("offset: {}\n", serde_json::to_string_pretty(&offset).unwrap());

    // File write arguments
    let (validate, format, backup) = file_write_args!();
    println!("File Write Args:");
    println!("validate: {}", serde_json::to_string_pretty(&validate).unwrap());
    println!("format: {}", serde_json::to_string_pretty(&format).unwrap());
    println!("backup: {}\n", serde_json::to_string_pretty(&backup).unwrap());
}

/// Example 11: Conversion Tool Schema (To Format)
fn example_convert_to_tool() {
    let schema = tool_schema! {
        required: ["hedl", "format"],
        properties: {
            hedl: hedl_content_arg!("HEDL document to convert"),
            format: format_arg!(["json", "yaml", "csv", "parquet", "cypher"], "Target format"),
            options: convert_to_options!()
        }
    };

    println!("Convert To Tool Schema:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());
}

/// Example 12: Conversion Tool Schema (From Format)
fn example_convert_from_tool() {
    let schema = tool_schema! {
        required: ["content", "format"],
        properties: {
            content: schema_string!("Content to convert (base64 for binary formats)"),
            format: format_arg!(["json", "yaml", "csv", "parquet"], "Source format"),
            options: convert_from_options!()
        }
    };

    println!("Convert From Tool Schema:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());
}

/// Example 13: Validation Tool Schema
fn example_validation_tool() {
    let (strict, lint) = validation_args!();

    let schema = tool_schema! {
        required: ["hedl"],
        properties: {
            hedl: hedl_content_arg!("HEDL document to validate"),
            strict: strict,
            lint: lint
        }
    };

    println!("Validation Tool Schema:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());
}

/// Example 14: Stream Parsing Tool Schema
fn example_stream_tool() {
    let (limit, offset) = pagination_args!(default_limit: 50);

    let schema = tool_schema! {
        required: ["hedl"],
        properties: {
            hedl: hedl_content_arg!("HEDL document to stream parse"),
            limit: limit,
            offset: offset,
            type_filter: schema_string!("Filter by entity type")
        }
    };

    println!("Stream Tool Schema:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());
}

/// Example 15: File Operations Tool Schema
fn example_file_ops_tool() {
    let (validate, format, backup) = file_write_args!();

    let schema = tool_schema! {
        required: ["path", "content"],
        properties: {
            path: path_arg!("File path to write"),
            content: schema_string!("Content to write"),
            validate: validate,
            format: format,
            backup: backup
        }
    };

    println!("File Operations Tool Schema:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());
}

/// Main function demonstrating all examples
fn main() {
    println!("=== Schema Macro Usage Examples ===\n");

    println!("--- Basic Types ---");
    example_basic_string();
    example_string_with_pattern();
    example_boolean_with_default();
    example_integer_with_range();
    example_enum();
    example_string_array();

    println!("\n--- Composite Types ---");
    example_options_object();
    example_complete_tool_schema();

    println!("\n--- Domain-Specific Macros ---");
    example_domain_specific_macros();

    println!("\n--- Multi-Field Macros ---");
    example_multi_field_macros();

    println!("\n--- Real-World Tool Schemas ---");
    example_convert_to_tool();
    example_convert_from_tool();
    example_validation_tool();
    example_stream_tool();
    example_file_ops_tool();

    println!("\n=== All Examples Complete ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_examples_compile() {
        // Just verify that all examples compile and run without panicking
        example_basic_string();
        example_string_with_pattern();
        example_boolean_with_default();
        example_integer_with_range();
        example_enum();
        example_string_array();
        example_options_object();
        example_complete_tool_schema();
        example_domain_specific_macros();
        example_multi_field_macros();
        example_convert_to_tool();
        example_convert_from_tool();
        example_validation_tool();
        example_stream_tool();
        example_file_ops_tool();
    }

    #[test]
    fn test_schema_structure_valid() {
        let schema = tool_schema! {
            required: ["test"],
            properties: {
                test: schema_string!("Test field")
            }
        };

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].get("test").is_some());
        assert_eq!(schema["required"][0], "test");
    }
}

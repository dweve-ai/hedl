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

//! HEDL tools for MCP server.
//!
//! Available tools:
//! - `hedl_read`: Read and parse HEDL files from a directory
//! - `hedl_query`: Query the node registry for specific entities
//! - `hedl_validate`: Validate HEDL input
//! - `hedl_optimize`: Convert JSON to optimized HEDL format
//! - `hedl_stats`: Get token usage statistics
//! - `hedl_format`: Format HEDL to canonical form
//! - `hedl_write`: Write HEDL content to a file
//! - `hedl_convert_to`: Convert HEDL to other formats
//! - `hedl_convert_from`: Convert other formats to HEDL
//! - `hedl_stream`: Stream parse a large HEDL document

mod conversion;
mod file_ops;
mod formatting;
mod helpers;
mod inspection;
mod json_utils;
#[macro_use]
mod schema_macros;
mod streaming;
mod types;
mod validation;

// Re-export public APIs
pub use conversion::{execute_hedl_convert_from, execute_hedl_convert_to};
pub use file_ops::{execute_hedl_read, execute_hedl_write};
pub use formatting::{execute_hedl_format, execute_hedl_optimize};
pub use inspection::{execute_hedl_query, execute_hedl_stats};
pub use streaming::execute_hedl_stream;
pub use validation::execute_hedl_validate;

use crate::error::{McpError, McpResult};
use crate::protocol::{CallToolResult, Tool};
use serde_json::Value as JsonValue;
use std::path::Path;

#[cfg(test)]
use serde_json::json;

/// Get all available HEDL tools.
pub fn get_tools() -> Vec<Tool> {
    let (strict, lint) = validation_args!();
    let (limit, offset) = pagination_args!();
    let (validate, format, backup) = file_write_args!();

    vec![
        Tool {
            name: "hedl_read".to_string(),
            description: "Read and parse HEDL files from a directory. Returns parsed document structure with entities and relationships.".to_string(),
            input_schema: tool_schema! {
                required: ["path"],
                properties: {
                    path: path_arg!("Path to a HEDL file or directory containing .hedl files"),
                    recursive: schema_bool!("If path is a directory, whether to search recursively", default: true),
                    include_json: schema_bool!("Include JSON representation of parsed documents", default: false)
                }
            },
        },
        Tool {
            name: "hedl_query".to_string(),
            description: "Query the node registry for entities by type and/or ID. Performs graph-aware lookups on parsed HEDL documents.".to_string(),
            input_schema: tool_schema! {
                required: ["hedl"],
                properties: {
                    hedl: hedl_content_arg!("HEDL document content to query"),
                    type_name: schema_string!("Filter by entity type name (e.g., 'User', 'Product')"),
                    id: schema_string!("Filter by entity ID"),
                    include_children: schema_bool!("Include nested children in results", default: true)
                }
            },
        },
        Tool {
            name: "hedl_validate".to_string(),
            description: "Validate HEDL input and return detailed diagnostics. Checks syntax, schema, references, and best practices.".to_string(),
            input_schema: tool_schema! {
                required: ["hedl"],
                properties: {
                    hedl: hedl_content_arg!("HEDL document content to validate"),
                    strict: strict,
                    lint: lint
                }
            },
        },
        Tool {
            name: "hedl_optimize".to_string(),
            description: "Convert JSON to optimized HEDL format. Reduces token usage by 40-60% while preserving all data and relationships.".to_string(),
            input_schema: tool_schema! {
                required: ["json"],
                properties: {
                    json: schema_string!("JSON content to convert to HEDL"),
                    ditto: ditto_arg!(),
                    compact: schema_bool!("Minimize whitespace in output", default: false)
                }
            },
        },
        Tool {
            name: "hedl_stats".to_string(),
            description: "Get token usage statistics comparing HEDL vs JSON representation. Shows exact token savings for LLM context optimization.".to_string(),
            input_schema: tool_schema! {
                required: ["hedl"],
                properties: {
                    hedl: hedl_content_arg!("HEDL document content to analyze"),
                    tokenizer: schema_enum!(["cl100k", "simple"], "Tokenizer to use (cl100k for GPT-4, simple for approximate count)", default: "simple")
                }
            },
        },
        Tool {
            name: "hedl_format".to_string(),
            description: "Format HEDL to canonical form. Normalizes whitespace, ordering, and applies ditto optimization.".to_string(),
            input_schema: tool_schema! {
                required: ["hedl"],
                properties: {
                    hedl: hedl_content_arg!("HEDL document content to format"),
                    ditto: ditto_arg!("Apply ditto optimization for repeated values")
                }
            },
        },
        Tool {
            name: "hedl_write".to_string(),
            description: "Write HEDL content to a file. Supports writing optimized data back to disk after processing.".to_string(),
            input_schema: tool_schema! {
                required: ["path", "content"],
                properties: {
                    path: path_arg!("Path to write the HEDL file (relative to root directory)"),
                    content: schema_string!("HEDL content to write"),
                    validate: validate,
                    format: format,
                    backup: backup
                }
            },
        },
        Tool {
            name: "hedl_convert_to".to_string(),
            description: "Convert HEDL to other formats: json, yaml, csv, parquet, cypher (Neo4j).".to_string(),
            input_schema: tool_schema! {
                required: ["hedl", "format"],
                properties: {
                    hedl: hedl_content_arg!("HEDL document content to convert"),
                    format: format_arg!(["json", "yaml", "csv", "parquet", "cypher"], "Target format"),
                    options: convert_to_options!()
                }
            },
        },
        Tool {
            name: "hedl_convert_from".to_string(),
            description: "Convert other formats to HEDL: json, yaml, csv, parquet.".to_string(),
            input_schema: tool_schema! {
                required: ["content", "format"],
                properties: {
                    content: schema_string!("Content to convert (base64 for parquet)"),
                    format: format_arg!(["json", "yaml", "csv", "parquet"], "Source format"),
                    options: convert_from_options!()
                }
            },
        },
        Tool {
            name: "hedl_stream".to_string(),
            description: "Stream parse a large HEDL document with pagination. Memory-efficient for large files.".to_string(),
            input_schema: tool_schema! {
                required: ["hedl"],
                properties: {
                    hedl: hedl_content_arg!("HEDL document content to stream parse"),
                    limit: limit,
                    offset: offset,
                    type_filter: schema_string!("Only return entities of this type")
                }
            },
        },
    ]
}

/// Execute a tool by name.
pub fn execute_tool(
    name: &str,
    arguments: Option<JsonValue>,
    root_path: &Path,
) -> McpResult<CallToolResult> {
    match name {
        "hedl_read" => execute_hedl_read(arguments, root_path),
        "hedl_query" => execute_hedl_query(arguments),
        "hedl_validate" => execute_hedl_validate(arguments),
        "hedl_optimize" => execute_hedl_optimize(arguments),
        "hedl_stats" => execute_hedl_stats(arguments),
        "hedl_format" => execute_hedl_format(arguments),
        "hedl_write" => execute_hedl_write(arguments, root_path),
        "hedl_convert_to" => execute_hedl_convert_to(arguments),
        "hedl_convert_from" => execute_hedl_convert_from(arguments),
        "hedl_stream" => execute_hedl_stream(arguments),
        _ => Err(McpError::ToolNotFound(name.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tools_returns_all_tools() {
        let tools = get_tools();
        assert_eq!(tools.len(), 10);

        let names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
        // Core tools
        assert!(names.contains(&"hedl_read"));
        assert!(names.contains(&"hedl_query"));
        assert!(names.contains(&"hedl_validate"));
        assert!(names.contains(&"hedl_optimize"));
        assert!(names.contains(&"hedl_stats"));
        assert!(names.contains(&"hedl_format"));
        assert!(names.contains(&"hedl_write"));
        // Unified conversion tools
        assert!(names.contains(&"hedl_convert_to"));
        assert!(names.contains(&"hedl_convert_from"));
        assert!(names.contains(&"hedl_stream"));
    }

    #[test]
    fn test_tool_descriptions_not_empty() {
        let tools = get_tools();
        for tool in &tools {
            assert!(
                !tool.description.is_empty(),
                "Tool {} has empty description",
                tool.name
            );
        }
    }

    #[test]
    fn test_tool_schemas_valid() {
        let tools = get_tools();
        for tool in &tools {
            // Each tool should have an input_schema with type "object"
            assert_eq!(
                tool.input_schema["type"], "object",
                "Tool {} missing object type",
                tool.name
            );
            // Each tool should have properties
            assert!(
                tool.input_schema.get("properties").is_some(),
                "Tool {} missing properties",
                tool.name
            );
        }
    }

    #[test]
    fn test_execute_tool_unknown() {
        let result = execute_tool("unknown_tool", None, Path::new("."));
        assert!(result.is_err());

        if let Err(McpError::ToolNotFound(name)) = result {
            assert_eq!(name, "unknown_tool");
        } else {
            panic!("Expected ToolNotFound error");
        }
    }

    #[test]
    fn test_execute_tool_validate() {
        let args = json!({ "hedl": "%VERSION 1.0\n---" });
        let result = execute_tool("hedl_validate", Some(args), Path::new(".")).unwrap();
        assert!(!result.content.is_empty());
    }
}

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

//! Format conversion tools (to/from HEDL).

use crate::error::{McpError, McpResult};
use crate::protocol::{CallToolResult, Content};
use crate::tools::helpers::{parse_args, validate_input_size};
use crate::tools::json_utils::count_entities;
use crate::tools::types::{ConvertFromArgs, ConvertToArgs, MAX_INPUT_SIZE};
use hedl_core::parse;
use hedl_json::{from_json_value, to_json_value, FromJsonConfig, ToJsonConfig};
use serde_json::{json, Value as JsonValue};

/// Execute hedl_convert_to tool.
pub fn execute_hedl_convert_to(args: Option<JsonValue>) -> McpResult<CallToolResult> {
    let args: ConvertToArgs = parse_args(args)?;
    let options = args.options.unwrap_or(json!({}));

    // Security: Validate input size to prevent memory exhaustion
    validate_input_size(&args.hedl, MAX_INPUT_SIZE)?;

    let doc = parse(args.hedl.as_bytes())?;

    let output = match args.format.as_str() {
        "json" => {
            let pretty = options
                .get("pretty")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let config = ToJsonConfig::default();
            let json_value = to_json_value(&doc, &config);
            if pretty {
                serde_json::to_string_pretty(&json_value)?
            } else {
                serde_json::to_string(&json_value)?
            }
        }
        "yaml" => hedl_yaml::hedl_to_yaml(&doc)
            .map_err(|e| McpError::InvalidArguments(format!("YAML conversion failed: {}", e)))?,
        "csv" => {
            let include_headers = options
                .get("include_headers")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let config = hedl_csv::ToCsvConfig {
                include_headers,
                ..Default::default()
            };
            hedl_csv::to_csv_with_config(&doc, config)
                .map_err(|e| McpError::InvalidArguments(format!("CSV conversion failed: {}", e)))?
        }
        "parquet" => {
            let bytes = hedl_parquet::to_parquet_bytes(&doc).map_err(|e| {
                McpError::InvalidArguments(format!("Parquet conversion failed: {}", e))
            })?;
            use base64::{engine::general_purpose::STANDARD, Engine as _};
            return Ok(CallToolResult {
                content: vec![Content::Text {
                    text: serde_json::to_string_pretty(&json!({
                        "parquet_base64": STANDARD.encode(&bytes),
                        "bytes": bytes.len()
                    }))?,
                }],
                is_error: None,
            });
        }
        "cypher" => {
            let use_merge = options
                .get("use_merge")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let include_constraints = options
                .get("include_constraints")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let mut config = hedl_neo4j::ToCypherConfig::new();
            if !use_merge {
                config = config.with_create();
            }
            if !include_constraints {
                config = config.without_constraints();
            }
            hedl_neo4j::to_cypher(&doc, &config)
                .map_err(|e| {
                    McpError::InvalidArguments(format!("Cypher conversion failed: {}", e))
                })?
                .to_string()
        }
        _ => {
            return Err(McpError::InvalidArguments(format!(
                "Unknown format: {}",
                args.format
            )))
        }
    };

    Ok(CallToolResult {
        content: vec![Content::Text { text: output }],
        is_error: None,
    })
}

/// Execute hedl_convert_from tool.
pub fn execute_hedl_convert_from(args: Option<JsonValue>) -> McpResult<CallToolResult> {
    let args: ConvertFromArgs = parse_args(args)?;
    let options = args.options.unwrap_or(json!({}));

    // Security: Validate input size to prevent memory exhaustion
    validate_input_size(&args.content, MAX_INPUT_SIZE)?;

    let doc = match args.format.as_str() {
        "json" => {
            let json_value: JsonValue = serde_json::from_str(&args.content)
                .map_err(|e| McpError::InvalidArguments(format!("Invalid JSON: {}", e)))?;
            let config = FromJsonConfig::default();
            from_json_value(&json_value, &config)
                .map_err(|e| McpError::InvalidArguments(format!("JSON conversion failed: {}", e)))?
        }
        "yaml" => hedl_yaml::yaml_to_hedl(&args.content)
            .map_err(|e| McpError::InvalidArguments(format!("YAML parse failed: {}", e)))?,
        "csv" => {
            let type_name = options
                .get("type_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Item");
            let delimiter = options
                .get("delimiter")
                .and_then(|v| v.as_str())
                .and_then(|s| s.as_bytes().first().copied())
                .unwrap_or(b',');

            // Get schema from options or infer from headers
            let schema_opt: Option<Vec<String>> =
                options.get("schema").and_then(|v| v.as_array()).map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                });

            let inferred_schema: Vec<String>;
            let schema_strs: Vec<&str> = if let Some(ref s) = schema_opt {
                s.iter().map(|x| x.as_str()).collect()
            } else {
                let mut reader = csv::ReaderBuilder::new()
                    .has_headers(true)
                    .from_reader(args.content.as_bytes());
                let headers = reader
                    .headers()
                    .map_err(|e| McpError::InvalidArguments(format!("CSV parse error: {}", e)))?;
                inferred_schema = headers
                    .iter()
                    .filter(|h| *h != "id")
                    .map(|h| h.to_string())
                    .collect();
                inferred_schema.iter().map(|s| s.as_str()).collect()
            };

            let config = hedl_csv::FromCsvConfig {
                delimiter,
                has_headers: true,
                trim: true,
                max_rows: usize::MAX,
                infer_schema: true,  // Enable auto-schema inference
                sample_rows: 100,    // Default sample size
                list_key: None,      // Use default pluralized list key
            };
            hedl_csv::from_csv_with_config(&args.content, type_name, &schema_strs, config)
                .map_err(|e| McpError::InvalidArguments(format!("CSV conversion failed: {}", e)))?
        }
        "parquet" => {
            use base64::{engine::general_purpose::STANDARD, Engine as _};
            let bytes = STANDARD
                .decode(&args.content)
                .map_err(|e| McpError::InvalidArguments(format!("Invalid base64: {}", e)))?;
            hedl_parquet::from_parquet_bytes(&bytes)
                .map_err(|e| McpError::InvalidArguments(format!("Parquet parse failed: {}", e)))?
        }
        _ => {
            return Err(McpError::InvalidArguments(format!(
                "Unknown format: {}",
                args.format
            )))
        }
    };

    // Canonicalize to HEDL output
    let c14n_config = hedl_c14n::CanonicalConfig::default();
    let hedl_output = hedl_c14n::canonicalize_with_config(&doc, &c14n_config)
        .map_err(|e| McpError::InvalidArguments(format!("Canonicalization failed: {}", e)))?;

    Ok(CallToolResult {
        content: vec![Content::Text {
            text: serde_json::to_string_pretty(&json!({
                "hedl": hedl_output,
                "entities": count_entities(&doc)
            }))?,
        }],
        is_error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ HEDL_CONVERT_TO TESTS ============

    #[test]
    fn test_hedl_convert_to_json_basic() {
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice Smith\n";
        let args = json!({ "hedl": hedl, "format": "json" });
        let result = execute_hedl_convert_to(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        // Should be valid JSON
        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_hedl_convert_to_json_compact() {
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";
        let args = json!({ "hedl": hedl, "format": "json", "options": { "pretty": false } });
        let result = execute_hedl_convert_to(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        // Compact JSON shouldn't have newlines
        assert!(
            !text.contains('\n'),
            "Expected compact JSON without newlines"
        );
    }

    #[test]
    fn test_hedl_convert_to_yaml() {
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";
        let args = json!({ "hedl": hedl, "format": "yaml" });
        let result = execute_hedl_convert_to(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        // Should contain YAML-like content
        assert!(text.contains("users"));
    }

    #[test]
    fn test_hedl_convert_to_csv() {
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";
        let args = json!({ "hedl": hedl, "format": "csv" });
        let result = execute_hedl_convert_to(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        // Should contain CSV headers
        assert!(text.contains("id") || text.contains("name"));
    }

    #[test]
    fn test_hedl_convert_to_cypher() {
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";
        let args = json!({ "hedl": hedl, "format": "cypher" });
        let result = execute_hedl_convert_to(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        // Should contain Cypher MERGE/CREATE statements
        assert!(text.contains("MERGE") || text.contains("CREATE"));
    }

    #[test]
    fn test_hedl_convert_to_parquet() {
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";
        let args = json!({ "hedl": hedl, "format": "parquet" });
        let result = execute_hedl_convert_to(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        // Should return JSON with base64 parquet data
        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert!(parsed.get("parquet_base64").is_some());
        assert!(parsed.get("bytes").is_some());
    }

    #[test]
    fn test_hedl_convert_to_invalid_hedl() {
        let hedl = "not valid hedl";
        let args = json!({ "hedl": hedl, "format": "json" });
        let result = execute_hedl_convert_to(Some(args));

        assert!(result.is_err());
    }

    #[test]
    fn test_hedl_convert_to_unknown_format() {
        let hedl = "%VERSION: 1.0\n---\n";
        let args = json!({ "hedl": hedl, "format": "unknown_format" });
        let result = execute_hedl_convert_to(Some(args));

        assert!(result.is_err());
    }

    #[test]
    fn test_convert_to_csv_no_headers() {
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";
        let args =
            json!({ "hedl": hedl, "format": "csv", "options": { "include_headers": false } });
        let result = execute_hedl_convert_to(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        assert!(!text.is_empty());
    }

    // ============ HEDL_CONVERT_FROM TESTS ============

    #[test]
    fn test_convert_from_json() {
        let json_input = r#"{"name": "test", "value": 42}"#;
        let args = json!({ "content": json_input, "format": "json" });
        let result = execute_hedl_convert_from(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert!(parsed.get("hedl").is_some());
    }

    #[test]
    fn test_convert_from_yaml() {
        let yaml = "name: test\ncount: 42\n";
        let args = json!({ "content": yaml, "format": "yaml" });
        let result = execute_hedl_convert_from(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert!(parsed.get("hedl").is_some());
    }

    #[test]
    fn test_convert_from_csv() {
        let csv = "id,name,age\n1,Alice,30\n2,Bob,25\n";
        let args = json!({ "content": csv, "format": "csv", "options": { "type_name": "User" } });
        let result = execute_hedl_convert_from(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert!(parsed.get("hedl").is_some());
    }

    #[test]
    fn test_convert_parquet_round_trip() {
        let hedl = "%VERSION: 1.0\n%STRUCT: Data: [id, value]\n---\ndata: @Data\n  | row1, 42\n";

        // Convert to parquet
        let args = json!({ "hedl": hedl, "format": "parquet" });
        let result = execute_hedl_convert_to(Some(args)).unwrap();
        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        let parquet_base64 = parsed["parquet_base64"].as_str().unwrap();

        // Convert back
        let args2 = json!({ "content": parquet_base64, "format": "parquet" });
        let result2 = execute_hedl_convert_from(Some(args2)).unwrap();
        let text2 = match &result2.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed2: JsonValue = serde_json::from_str(text2).unwrap();
        assert!(parsed2.get("hedl").is_some());
    }

    #[test]
    fn test_convert_from_invalid_json() {
        let args = json!({ "content": "not valid json", "format": "json" });
        let result = execute_hedl_convert_from(Some(args));
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_from_invalid_format() {
        let args = json!({ "content": "test", "format": "invalid" });
        let result = execute_hedl_convert_from(Some(args));
        assert!(result.is_err());
    }
}

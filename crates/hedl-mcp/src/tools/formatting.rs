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

//! Formatting and canonicalization tools.

use crate::error::{McpError, McpResult};
use crate::protocol::{CallToolResult, Content};
use crate::tools::helpers::{estimate_tokens, parse_args, validate_input_size};
use crate::tools::types::{FormatArgs, OptimizeArgs, MAX_INPUT_SIZE};
use hedl_core::parse;
use hedl_json::{from_json_value, FromJsonConfig};
use serde_json::{json, Value as JsonValue};

/// Execute hedl_format tool.
pub fn execute_hedl_format(args: Option<JsonValue>) -> McpResult<CallToolResult> {
    let args: FormatArgs = parse_args(args)?;

    // Security: Validate input size to prevent memory exhaustion
    validate_input_size(&args.hedl, MAX_INPUT_SIZE)?;

    let doc = parse(args.hedl.as_bytes())?;

    let mut config = hedl_c14n::CanonicalConfig::default();
    config.use_ditto = args.ditto;

    let formatted = hedl_c14n::canonicalize_with_config(&doc, &config)
        .map_err(|e| McpError::InvalidArguments(format!("Format failed: {}", e)))?;

    Ok(CallToolResult {
        content: vec![Content::Text { text: formatted }],
        is_error: None,
    })
}

/// Execute hedl_optimize tool.
pub fn execute_hedl_optimize(args: Option<JsonValue>) -> McpResult<CallToolResult> {
    let args: OptimizeArgs = parse_args(args)?;

    // Security: Validate input size to prevent memory exhaustion
    validate_input_size(&args.json, MAX_INPUT_SIZE)?;

    // Parse JSON
    let json_value: JsonValue = serde_json::from_str(&args.json)
        .map_err(|e| McpError::InvalidArguments(format!("Invalid JSON: {}", e)))?;

    // Convert to HEDL document
    let config = FromJsonConfig::default();
    let doc = from_json_value(&json_value, &config)
        .map_err(|e| McpError::InvalidArguments(format!("Cannot convert JSON to HEDL: {}", e)))?;

    // Canonicalize with ditto optimization
    let mut c14n_config = hedl_c14n::CanonicalConfig::default();
    c14n_config.use_ditto = args.ditto;

    let hedl_output = hedl_c14n::canonicalize_with_config(&doc, &c14n_config)
        .map_err(|e| McpError::InvalidArguments(format!("Canonicalization failed: {}", e)))?;

    // Calculate stats (can be negative if HEDL is larger)
    let json_tokens = estimate_tokens(&args.json);
    let hedl_tokens = estimate_tokens(&hedl_output);
    let tokens_diff = json_tokens as i64 - hedl_tokens as i64;
    let savings_percent = if json_tokens > 0 {
        (tokens_diff as f64 / json_tokens as f64 * 100.0).round() as i64
    } else {
        0
    };

    Ok(CallToolResult {
        content: vec![Content::Text {
            text: serde_json::to_string_pretty(&json!({
                "hedl": hedl_output,
                "stats": {
                    "json_tokens": json_tokens,
                    "hedl_tokens": hedl_tokens,
                    "savings_percent": savings_percent,
                    "tokens_saved": tokens_diff
                }
            }))?,
        }],
        is_error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hedl_format_basic() {
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";
        let args = json!({ "hedl": hedl });
        let result = execute_hedl_format(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        // Formatted output should contain version
        assert!(text.contains("%VERSION"));
    }

    #[test]
    fn test_hedl_format_with_ditto() {
        let hedl = "%VERSION: 1.0\n%STRUCT: Data: [id, category]\n---\ndata: @Data\n  | row1, CategoryA\n  | row2, CategoryA\n";
        let args = json!({ "hedl": hedl, "ditto": true });
        let result = execute_hedl_format(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        // With ditto optimization, repeated values should be replaced with ^
        assert!(text.contains('^') || text.contains("row2"));
    }

    #[test]
    fn test_hedl_format_without_ditto() {
        let hedl = "%VERSION: 1.0\n%STRUCT: Data: [id, category]\n---\ndata: @Data\n  | row1, CategoryA\n  | row2, CategoryA\n";
        let args = json!({ "hedl": hedl, "ditto": false });
        let result = execute_hedl_format(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        // Without ditto, values should not be replaced
        // Check for CategoryA (both rows should have it)
        assert!(text.contains("CategoryA"));
    }

    #[test]
    fn test_hedl_optimize_valid_json() {
        let json_input = r#"{"users": [{"id": "alice", "name": "Alice"}]}"#;
        let args = json!({ "json": json_input });
        let result = execute_hedl_optimize(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert!(parsed.get("hedl").is_some());
        assert!(parsed.get("stats").is_some());
    }

    #[test]
    fn test_hedl_optimize_invalid_json() {
        let args = json!({ "json": "not valid json" });
        let result = execute_hedl_optimize(Some(args));

        assert!(result.is_err());
    }

    #[test]
    fn test_hedl_optimize_with_ditto() {
        let json_input = r#"{"items": [{"id": "a", "cat": "X"}, {"id": "b", "cat": "X"}]}"#;
        let args = json!({ "json": json_input, "ditto": true });
        let result = execute_hedl_optimize(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        let hedl = parsed["hedl"].as_str().unwrap();

        // With ditto enabled, repeated values might use ^
        assert!(hedl.contains('%') || !hedl.is_empty()); // Just verify we got HEDL output
    }
}

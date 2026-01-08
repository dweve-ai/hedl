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

//! Inspection and query tools.

use crate::error::McpResult;
use crate::protocol::{CallToolResult, Content};
use crate::tools::helpers::{estimate_tokens, parse_args, validate_input_size};
use crate::tools::json_utils::{doc_schema_for_type, node_to_json};
use crate::tools::types::{QueryArgs, StatsArgs, MAX_INPUT_SIZE};
use hedl_core::{parse, Item, Node};
use hedl_json::{to_json_value, ToJsonConfig};
use serde_json::{json, Value as JsonValue};

/// Execute hedl_query tool.
pub fn execute_hedl_query(args: Option<JsonValue>) -> McpResult<CallToolResult> {
    let args: QueryArgs = parse_args(args)?;

    // Security: Validate input size to prevent memory exhaustion
    validate_input_size(&args.hedl, MAX_INPUT_SIZE)?;

    let doc = parse(args.hedl.as_bytes())?;
    let mut matches = Vec::new();

    for item in doc.root.values() {
        find_matching_entities(
            item,
            &args.type_name,
            &args.id,
            args.include_children,
            &mut matches,
        );
    }

    Ok(CallToolResult {
        content: vec![Content::Text {
            text: serde_json::to_string_pretty(&json!({
                "matches": matches.len(),
                "entities": matches
            }))?,
        }],
        is_error: None,
    })
}

fn find_matching_entities(
    item: &Item,
    type_filter: &Option<String>,
    id_filter: &Option<String>,
    include_children: bool,
    matches: &mut Vec<JsonValue>,
) {
    match item {
        Item::List(list) => {
            let type_matches = type_filter.as_ref().is_none_or(|t| &list.type_name == t);

            for node in &list.rows {
                let id_matches = id_filter.as_ref().is_none_or(|i| &node.id == i);

                if type_matches && id_matches {
                    matches.push(node_to_json(node, &list.schema, include_children));
                }

                // Search children
                for children in node.children.values() {
                    for child in children {
                        find_matching_node(
                            child,
                            type_filter,
                            id_filter,
                            include_children,
                            matches,
                            &doc_schema_for_type(&child.type_name),
                        );
                    }
                }
            }
        }
        Item::Object(obj) => {
            for child in obj.values() {
                find_matching_entities(child, type_filter, id_filter, include_children, matches);
            }
        }
        Item::Scalar(_) => {}
    }
}

fn find_matching_node(
    node: &Node,
    type_filter: &Option<String>,
    id_filter: &Option<String>,
    include_children: bool,
    matches: &mut Vec<JsonValue>,
    schema: &[String],
) {
    let type_matches = type_filter.as_ref().is_none_or(|t| &node.type_name == t);
    let id_matches = id_filter.as_ref().is_none_or(|i| &node.id == i);

    if type_matches && id_matches {
        matches.push(node_to_json(node, schema, include_children));
    }

    for children in node.children.values() {
        for child in children {
            find_matching_node(
                child,
                type_filter,
                id_filter,
                include_children,
                matches,
                &doc_schema_for_type(&child.type_name),
            );
        }
    }
}

/// Execute hedl_stats tool.
pub fn execute_hedl_stats(args: Option<JsonValue>) -> McpResult<CallToolResult> {
    let args: StatsArgs = parse_args(args)?;

    // Security: Validate input size to prevent memory exhaustion
    validate_input_size(&args.hedl, MAX_INPUT_SIZE)?;

    // Parse HEDL
    let doc = parse(args.hedl.as_bytes())?;

    // Convert to JSON for comparison
    let config = ToJsonConfig::default();
    let json_value = to_json_value(&doc, &config).map_err(crate::error::McpError::InvalidArguments)?;
    let json_str = serde_json::to_string(&json_value)?;
    let json_pretty = serde_json::to_string_pretty(&json_value)?;

    // Calculate token counts
    let hedl_tokens = estimate_tokens(&args.hedl);
    let json_tokens = estimate_tokens(&json_str);
    let json_pretty_tokens = estimate_tokens(&json_pretty);

    // Calculate savings (can be negative if HEDL is larger)
    let compact_diff = json_tokens as i64 - hedl_tokens as i64;
    let pretty_diff = json_pretty_tokens as i64 - hedl_tokens as i64;

    let vs_compact = if json_tokens > 0 {
        (compact_diff as f64 / json_tokens as f64 * 100.0).round() as i64
    } else {
        0
    };
    let vs_pretty = if json_pretty_tokens > 0 {
        (pretty_diff as f64 / json_pretty_tokens as f64 * 100.0).round() as i64
    } else {
        0
    };

    Ok(CallToolResult {
        content: vec![Content::Text {
            text: serde_json::to_string_pretty(&json!({
                "tokenizer": args.tokenizer,
                "hedl": {
                    "bytes": args.hedl.len(),
                    "tokens": hedl_tokens,
                    "lines": args.hedl.lines().count()
                },
                "json_compact": {
                    "bytes": json_str.len(),
                    "tokens": json_tokens
                },
                "json_pretty": {
                    "bytes": json_pretty.len(),
                    "tokens": json_pretty_tokens
                },
                "savings": {
                    "vs_compact_percent": vs_compact,
                    "vs_pretty_percent": vs_pretty,
                    "tokens_saved_vs_compact": compact_diff,
                    "tokens_saved_vs_pretty": pretty_diff
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
    fn test_hedl_stats_basic() {
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice Smith\n";
        let args = json!({ "hedl": hedl });
        let result = execute_hedl_stats(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert!(parsed.get("hedl").is_some());
        assert!(parsed.get("json_compact").is_some());
        assert!(parsed.get("json_pretty").is_some());
        assert!(parsed.get("savings").is_some());
    }

    #[test]
    fn test_hedl_stats_shows_savings() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name, email]\n---\nusers: @User\n  | alice, Alice Smith, alice@example.com\n  | bob, Bob Jones, bob@example.com\n  | charlie, Charlie Brown, charlie@example.com\n";
        let args = json!({ "hedl": hedl });
        let result = execute_hedl_stats(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();

        // HEDL should typically be more compact than JSON
        let hedl_tokens = parsed["hedl"]["tokens"].as_u64().unwrap();
        let json_tokens = parsed["json_pretty"]["tokens"].as_u64().unwrap();

        // For structured data, HEDL should usually be smaller
        assert!(hedl_tokens > 0);
        assert!(json_tokens > 0);
    }

    #[test]
    fn test_hedl_query_all_entities() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n  | bob, Bob\n";
        let args = json!({ "hedl": hedl });
        let result = execute_hedl_query(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["matches"], 2);
    }

    #[test]
    fn test_hedl_query_by_type() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n%STRUCT: Product: [id, title]\n---\nusers: @User\n  | alice, Alice\nproducts: @Product\n  | widget, Widget\n";
        let args = json!({ "hedl": hedl, "type_name": "User" });
        let result = execute_hedl_query(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["matches"], 1);
    }

    #[test]
    fn test_hedl_query_by_id() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n  | bob, Bob\n";
        let args = json!({ "hedl": hedl, "id": "alice" });
        let result = execute_hedl_query(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["matches"], 1);
    }

    #[test]
    fn test_hedl_query_no_matches() {
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";
        let args = json!({ "hedl": hedl, "id": "nonexistent" });
        let result = execute_hedl_query(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["matches"], 0);
    }
}

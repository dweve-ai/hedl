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

//! Streaming parser tool.

use crate::error::{McpError, McpResult};
use crate::protocol::{CallToolResult, Content};
use crate::tools::helpers::{parse_args, validate_input_size};
use crate::tools::json_utils::value_to_json;
use crate::tools::types::{StreamArgs, MAX_INPUT_SIZE};
use serde_json::{json, Value as JsonValue};

/// Execute hedl_stream tool.
pub fn execute_hedl_stream(args: Option<JsonValue>) -> McpResult<CallToolResult> {
    let args: StreamArgs = parse_args(args)?;

    // Security: Validate input size to prevent memory exhaustion
    validate_input_size(&args.hedl, MAX_INPUT_SIZE)?;

    let reader = std::io::Cursor::new(args.hedl.as_bytes());
    let parser = hedl_stream::StreamingParser::new(reader)
        .map_err(|e| McpError::InvalidArguments(format!("Stream parse error: {}", e)))?;

    let mut entities = Vec::new();
    let mut skipped = 0;
    let mut count = 0;

    for event in parser {
        let event =
            event.map_err(|e| McpError::InvalidArguments(format!("Stream error: {}", e)))?;

        if let hedl_stream::NodeEvent::Node(node) = event {
            // Apply type filter
            if let Some(ref filter) = args.type_filter {
                if &node.type_name != filter {
                    continue;
                }
            }

            // Apply offset
            if skipped < args.offset {
                skipped += 1;
                continue;
            }

            // Apply limit
            if count >= args.limit {
                break;
            }

            entities.push(json!({
                "type": node.type_name,
                "id": node.id,
                "fields": node.fields.iter().map(value_to_json).collect::<Vec<_>>()
            }));
            count += 1;
        }
    }

    Ok(CallToolResult {
        content: vec![Content::Text {
            text: serde_json::to_string_pretty(&json!({
                "entities": entities,
                "count": entities.len(),
                "offset": args.offset,
                "limit": args.limit
            }))?,
        }],
        is_error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Content;

    #[test]
    fn test_hedl_stream_basic() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n  | bob, Bob\n";
        let args = json!({ "hedl": hedl });
        let result = execute_hedl_stream(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert!(parsed.get("entities").is_some());
        assert!(parsed.get("count").is_some());
    }

    #[test]
    fn test_hedl_stream_with_limit() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n  | bob, Bob\n  | charlie, Charlie\n";
        let args = json!({ "hedl": hedl, "limit": 2 });
        let result = execute_hedl_stream(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        let count = parsed["count"].as_u64().unwrap();
        assert!(count <= 2);
    }

    #[test]
    fn test_hedl_stream_with_offset() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n  | bob, Bob\n  | charlie, Charlie\n";
        let args = json!({ "hedl": hedl, "offset": 1, "limit": 10 });
        let result = execute_hedl_stream(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        let offset = parsed["offset"].as_u64().unwrap();
        assert_eq!(offset, 1);
    }

    #[test]
    fn test_hedl_stream_with_type_filter() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n%STRUCT: Product: [id, title]\n---\nusers: @User\n  | alice, Alice\nproducts: @Product\n  | widget, Widget\n";
        let args = json!({ "hedl": hedl, "type_filter": "User" });
        let result = execute_hedl_stream(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        let entities = parsed["entities"].as_array().unwrap();

        for entity in entities {
            assert_eq!(entity["type"], "User");
        }
    }

    #[test]
    fn test_hedl_stream_invalid_hedl() {
        let args = json!({ "hedl": "invalid hedl content" });
        let result = execute_hedl_stream(Some(args));
        assert!(result.is_err());
    }
}

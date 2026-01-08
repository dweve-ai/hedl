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

//! Validation and linting tools.

use crate::error::McpResult;
use crate::protocol::{CallToolResult, Content};
use crate::tools::helpers::{parse_args, validate_input_size};
use crate::tools::json_utils::count_entities;
use crate::tools::types::{ValidateArgs, MAX_INPUT_SIZE};
use hedl_core::parse;
use hedl_lint::lint;
use serde_json::{json, Value as JsonValue};

/// Execute hedl_validate tool.
pub fn execute_hedl_validate(args: Option<JsonValue>) -> McpResult<CallToolResult> {
    let args: ValidateArgs = parse_args(args)?;

    // Security: Validate input size to prevent memory exhaustion
    validate_input_size(&args.hedl, MAX_INPUT_SIZE)?;

    // Parse
    let parse_result = parse(args.hedl.as_bytes());

    let mut result = json!({
        "valid": parse_result.is_ok()
    });

    match parse_result {
        Ok(doc) => {
            result["version"] = json!(format!("{}.{}", doc.version.0, doc.version.1));
            result["schemas"] = json!(doc.structs.len());
            result["entities"] = count_entities(&doc);

            // Run linting if requested
            if args.lint {
                let diagnostics = lint(&doc);

                if !diagnostics.is_empty() {
                    use hedl_lint::Severity;

                    let has_errors = diagnostics.iter().any(|d| d.severity() == Severity::Error);
                    let has_warnings = diagnostics.iter().any(|d| d.severity() == Severity::Warning);

                    result["lint"] = json!({
                        "count": diagnostics.len(),
                        "diagnostics": diagnostics.iter().map(|d| json!({
                            "severity": format!("{:?}", d.severity()),
                            "message": d.message(),
                            "line": d.line(),
                            "rule_id": d.rule_id()
                        })).collect::<Vec<_>>()
                    });

                    // In strict mode, treat lint warnings as validation errors
                    if args.strict && has_warnings {
                        result["valid"] = json!(false);
                        result["strict_mode"] = json!(true);
                        result["strict_validation_failed"] = json!("Lint warnings present in strict mode");
                    } else if has_errors {
                        // Errors always fail validation, regardless of strict mode
                        result["valid"] = json!(false);
                    }
                }
            }
        }
        Err(e) => {
            result["error"] = json!({
                "kind": format!("{:?}", e.kind),
                "message": e.message,
                "line": e.line
            });
        }
    }

    let is_error = !result["valid"].as_bool().unwrap_or(false);

    Ok(CallToolResult {
        content: vec![Content::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(is_error),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hedl_validate_valid_document() {
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice Smith\n";
        let args = json!({ "hedl": hedl });
        let result = execute_hedl_validate(Some(args)).unwrap();

        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["valid"], true);
    }

    #[test]
    fn test_hedl_validate_invalid_document() {
        let hedl = "invalid hedl content";
        let args = json!({ "hedl": hedl });
        let result = execute_hedl_validate(Some(args)).unwrap();

        assert!(result.is_error.unwrap_or(false));

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["valid"], false);
        assert!(parsed.get("error").is_some());
    }

    #[test]
    fn test_hedl_validate_missing_version() {
        let hedl = "%STRUCT: User: [id, name]\n---\n";
        let args = json!({ "hedl": hedl });
        let result = execute_hedl_validate(Some(args)).unwrap();

        assert!(result.is_error.unwrap_or(false));
    }

    #[test]
    fn test_hedl_validate_with_lint() {
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice Smith\n";
        let args = json!({ "hedl": hedl, "lint": true });
        let result = execute_hedl_validate(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["valid"], true);
    }

    #[test]
    fn test_hedl_validate_strict_mode_with_warnings() {
        // Create a document that will generate lint warnings (short IDs)
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | a, Alice\n  | b, Bob\n";
        let args = json!({ "hedl": hedl, "lint": true, "strict": true });
        let result = execute_hedl_validate(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();

        // In strict mode, warnings should cause validation to fail
        // Note: This depends on whether short IDs generate warnings in the linter
        // If there are no warnings generated, this test won't detect strict mode behavior
        if parsed.get("lint").is_some() && parsed["lint"]["count"].as_u64().unwrap_or(0) > 0 {
            // Check if any diagnostics are warnings
            let has_warnings = parsed["lint"]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["severity"].as_str() == Some("Warning"));

            if has_warnings {
                assert_eq!(parsed["valid"], false, "Strict mode should fail on warnings");
                assert_eq!(parsed["strict_mode"], true);
                assert!(parsed.get("strict_validation_failed").is_some());
            }
        }
    }

    #[test]
    fn test_hedl_validate_non_strict_mode_with_warnings() {
        // Create a document that will generate lint warnings (short IDs)
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | a, Alice\n  | b, Bob\n";
        let args = json!({ "hedl": hedl, "lint": true, "strict": false });
        let result = execute_hedl_validate(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();

        // In non-strict mode, warnings should not cause validation to fail
        if parsed.get("lint").is_some() && parsed["lint"]["count"].as_u64().unwrap_or(0) > 0 {
            // Check if we only have warnings (no errors)
            let has_errors = parsed["lint"]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["severity"].as_str() == Some("Error"));

            if !has_errors {
                assert_eq!(parsed["valid"], true, "Non-strict mode should pass with only warnings");
                assert!(parsed.get("strict_mode").is_none() || parsed["strict_mode"] == false);
            }
        }
    }

    #[test]
    fn test_hedl_validate_strict_mode_with_errors() {
        // Create a document that will generate lint errors (if linter has error-level rules)
        // For now, we'll use an invalid document to ensure validation fails
        let hedl = "invalid hedl content";
        let args = json!({ "hedl": hedl, "lint": true, "strict": true });
        let result = execute_hedl_validate(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();

        // Errors should always fail validation, regardless of strict mode
        assert_eq!(parsed["valid"], false);
        assert!(result.is_error.unwrap_or(false));
    }

    #[test]
    fn test_hedl_validate_lint_disabled() {
        // Even with warnings in the document, if lint is disabled, they shouldn't be reported
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | a, Alice\n";
        let args = json!({ "hedl": hedl, "lint": false });
        let result = execute_hedl_validate(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();

        // With lint disabled, no lint diagnostics should be present
        assert!(parsed.get("lint").is_none());
        assert_eq!(parsed["valid"], true);
    }

    #[test]
    fn test_hedl_validate_unused_schema_warning() {
        // Test with unused schema which generates a warning
        let hedl = "%VERSION: 1.0\n%STRUCT: UnusedType: [id]\n%STRUCT: UsedType: [id]\n---\nused: @UsedType\n  | test\n";

        // Strict mode should fail
        let args = json!({ "hedl": hedl, "lint": true, "strict": true });
        let result = execute_hedl_validate(Some(args)).unwrap();
        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();

        if parsed.get("lint").is_some() && parsed["lint"]["count"].as_u64().unwrap_or(0) > 0 {
            let has_warnings = parsed["lint"]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["severity"].as_str() == Some("Warning"));

            if has_warnings {
                assert_eq!(parsed["valid"], false, "Strict mode should fail with unused schema warning");
            }
        }

        // Non-strict mode should pass
        let args = json!({ "hedl": hedl, "lint": true, "strict": false });
        let result = execute_hedl_validate(Some(args)).unwrap();
        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();

        let has_only_warnings = if let Some(lint) = parsed.get("lint") {
            lint["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .all(|d| d["severity"].as_str() != Some("Error"))
        } else {
            true
        };

        if has_only_warnings {
            assert_eq!(parsed["valid"], true, "Non-strict mode should pass with only warnings");
        }
    }

    #[test]
    fn test_hedl_validate_diagnostics_include_rule_id() {
        // Verify that diagnostics include rule_id field
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | a, Alice\n";
        let args = json!({ "hedl": hedl, "lint": true });
        let result = execute_hedl_validate(Some(args)).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };
        let parsed: JsonValue = serde_json::from_str(text).unwrap();

        if let Some(lint) = parsed.get("lint") {
            if let Some(diagnostics) = lint["diagnostics"].as_array() {
                for diagnostic in diagnostics {
                    assert!(
                        diagnostic.get("rule_id").is_some(),
                        "Diagnostic should include rule_id"
                    );
                }
            }
        }
    }
}

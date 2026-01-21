// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for all MCP tools.

use hedl_mcp::tools::{execute_tool, get_tools};
use hedl_mcp::{Content, McpError};
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_hedl_read_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.hedl");

    fs::write(&file_path, "%VERSION: 1.0\n---\nname: Test\n").unwrap();

    let args = json!({
        "path": "test.hedl",
        "include_json": false
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path()).unwrap();
    assert!(!result.content.is_empty());
}

#[test]
fn test_hedl_read_directory_recursive() {
    let temp_dir = TempDir::new().unwrap();

    // Create nested structure
    let sub_dir = temp_dir.path().join("sub");
    fs::create_dir(&sub_dir).unwrap();

    fs::write(
        temp_dir.path().join("test1.hedl"),
        "%VERSION: 1.0\n---\nname: Test1\n",
    )
    .unwrap();
    fs::write(
        sub_dir.join("test2.hedl"),
        "%VERSION: 1.0\n---\nname: Test2\n",
    )
    .unwrap();

    let args = json!({
        "path": ".",
        "recursive": true,
        "include_json": false
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path()).unwrap();

    // Parse result to check files_read count
    if let Content::Text { text } = &result.content[0] {
        let json_result: serde_json::Value = serde_json::from_str(text).unwrap();
        let files_read = json_result["files_read"].as_u64().unwrap();
        assert_eq!(files_read, 2);
    } else {
        panic!("Expected text content");
    }
}

#[test]
fn test_hedl_read_directory_non_recursive() {
    let temp_dir = TempDir::new().unwrap();

    let sub_dir = temp_dir.path().join("sub");
    fs::create_dir(&sub_dir).unwrap();

    fs::write(
        temp_dir.path().join("test1.hedl"),
        "%VERSION: 1.0\n---\nname: Test1\n",
    )
    .unwrap();
    fs::write(
        sub_dir.join("test2.hedl"),
        "%VERSION: 1.0\n---\nname: Test2\n",
    )
    .unwrap();

    let args = json!({
        "path": ".",
        "recursive": false,
        "include_json": false
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path()).unwrap();

    if let Content::Text { text } = &result.content[0] {
        let json_result: serde_json::Value = serde_json::from_str(text).unwrap();
        let files_read = json_result["files_read"].as_u64().unwrap();
        // Should only read files in root, not subdirectory
        assert_eq!(files_read, 1);
    }
}

#[test]
fn test_hedl_read_with_json() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.hedl");

    fs::write(&file_path, "%VERSION: 1.0\n---\nname: Test\n").unwrap();

    let args = json!({
        "path": "test.hedl",
        "include_json": true
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path()).unwrap();

    if let Content::Text { text } = &result.content[0] {
        let json_result: serde_json::Value = serde_json::from_str(text).unwrap();
        let results = json_result["results"].as_array().unwrap();
        // API returns "data" field (not "json") when include_json is true
        assert!(results[0].get("data").is_some());
    }
}

#[test]
fn test_hedl_read_file_not_found() {
    let temp_dir = TempDir::new().unwrap();

    let args = json!({
        "path": "nonexistent.hedl"
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path());
    assert!(result.is_err());
    assert!(matches!(result, Err(McpError::FileNotFound(_))));
}

#[test]
fn test_hedl_write_basic() {
    let temp_dir = TempDir::new().unwrap();

    let args = json!({
        "path": "output.hedl",
        "content": "%VERSION: 1.0\n---\nname: Test\n",
        "validate": true,
        "format": false,
        "backup": false
    });

    let result = execute_tool("hedl_write", Some(args), temp_dir.path());
    assert!(result.is_ok());

    // Verify file was written
    let written_content = fs::read_to_string(temp_dir.path().join("output.hedl")).unwrap();
    assert!(written_content.contains("name: Test"));
}

#[test]
fn test_hedl_write_with_validation() {
    let temp_dir = TempDir::new().unwrap();

    // Invalid HEDL content
    let args = json!({
        "path": "output.hedl",
        "content": "invalid hedl",
        "validate": true
    });

    let result = execute_tool("hedl_write", Some(args), temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn test_hedl_write_with_format() {
    let temp_dir = TempDir::new().unwrap();

    // Use valid HEDL (space after colon is required)
    // Format option reformats spacing/alignment, not syntax errors
    let args = json!({
        "path": "output.hedl",
        "content": "%VERSION: 1.0\n---\nname: Test\nage: 30\n",
        "format": true,
        "validate": false
    });

    let result = execute_tool("hedl_write", Some(args), temp_dir.path());
    assert!(result.is_ok());

    let written_content = fs::read_to_string(temp_dir.path().join("output.hedl")).unwrap();
    // Should contain the formatted content
    assert!(written_content.contains("name: Test"));
    assert!(written_content.contains("age: 30") || written_content.contains("age:"));
}

#[test]
fn test_hedl_write_with_backup() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("existing.hedl");

    // Create existing file
    fs::write(&file_path, "%VERSION: 1.0\n---\nname: Original\n").unwrap();

    let args = json!({
        "path": "existing.hedl",
        "content": "%VERSION: 1.0\n---\nname: Updated\n",
        "backup": true,
        "validate": false
    });

    let result = execute_tool("hedl_write", Some(args), temp_dir.path());
    assert!(result.is_ok());

    // Verify backup was created
    let backup_exists = temp_dir.path().read_dir().unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("existing.hedl.")
    });

    assert!(backup_exists);
}

#[test]
fn test_hedl_query_by_type() {
    let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n  | bob, Bob\n";

    let args = json!({
        "hedl": hedl,
        "type_name": "User"
    });

    let result = execute_tool("hedl_query", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        let json_result: serde_json::Value = serde_json::from_str(text).unwrap();
        let entities = json_result["entities"].as_array().unwrap();
        assert_eq!(entities.len(), 2);
    }
}

#[test]
fn test_hedl_query_by_id() {
    let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n  | bob, Bob\n";

    let args = json!({
        "hedl": hedl,
        "id": "alice"
    });

    let result = execute_tool("hedl_query", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        let json_result: serde_json::Value = serde_json::from_str(text).unwrap();
        let entities = json_result["entities"].as_array().unwrap();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0]["id"], "alice");
    }
}

#[test]
fn test_hedl_validate_valid() {
    let args = json!({
        "hedl": "%VERSION: 1.0\n---\nname: Test\n"
    });

    let result = execute_tool("hedl_validate", Some(args), Path::new(".")).unwrap();
    assert!(result.is_error.is_none() || !result.is_error.unwrap());
}

#[test]
fn test_hedl_validate_invalid() {
    let args = json!({
        "hedl": "invalid hedl content"
    });

    let result = execute_tool("hedl_validate", Some(args), Path::new(".")).unwrap();
    assert!(result.is_error.unwrap_or(false));
}

#[test]
fn test_hedl_validate_with_lint() {
    let args = json!({
        "hedl": "%VERSION: 1.0\n---\nname: Test\n",
        "lint": true
    });

    let result = execute_tool("hedl_validate", Some(args), Path::new(".")).unwrap();
    // Should include lint results
    if let Content::Text { text } = &result.content[0] {
        let json_result: serde_json::Value = serde_json::from_str(text).unwrap();
        // Lint field might or might not be present depending on whether there are issues
        assert!(json_result["valid"].is_boolean());
    }
}

#[test]
fn test_hedl_optimize_json_to_hedl() {
    let json_data = r#"{"name": "Test", "age": 30}"#;

    let args = json!({
        "json": json_data,
        "ditto": false,
        "compact": false
    });

    let result = execute_tool("hedl_optimize", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        // Result is JSON containing { "hedl": "...", "stats": {...} }
        let json_result: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(json_result.get("hedl").is_some());
        assert!(json_result.get("stats").is_some());
        // The hedl output should contain the field name
        let hedl_content = json_result["hedl"].as_str().unwrap();
        assert!(hedl_content.contains("name"));
    }
}

#[test]
fn test_hedl_optimize_with_ditto() {
    // JSON root must be an object, not an array
    let json_data = r#"{"items": [{"name": "Test", "type": "A"}, {"name": "Test", "type": "B"}]}"#;

    let args = json!({
        "json": json_data,
        "ditto": true,
        "compact": false
    });

    let result = execute_tool("hedl_optimize", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        // Result is JSON containing { "hedl": "...", "stats": {...} }
        let json_result: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(json_result.get("hedl").is_some());
        // With ditto enabled, repeated values might use ^ (ditto mark)
        let hedl_content = json_result["hedl"].as_str().unwrap();
        assert!(hedl_content.contains("items") || hedl_content.contains("Test"));
    }
}

#[test]
fn test_hedl_stats() {
    let hedl = "%VERSION: 1.0\n---\nname: Test\nage: 30\n";

    let args = json!({
        "hedl": hedl,
        "tokenizer": "simple"
    });

    let result = execute_tool("hedl_stats", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        let json_result: serde_json::Value = serde_json::from_str(text).unwrap();
        // API returns nested structure: hedl.tokens, json_compact.tokens, savings
        assert!(json_result.get("hedl").is_some());
        assert!(json_result["hedl"].get("tokens").is_some());
        assert!(json_result.get("json_compact").is_some());
        assert!(json_result.get("savings").is_some());
    }
}

#[test]
fn test_hedl_format() {
    // Use valid HEDL input (space after colon is required by parser)
    // hedl_format parses first, then canonicalizes/formats
    let hedl = "%VERSION: 1.0\n---\nname: Test\nage: 30\n";

    let args = json!({
        "hedl": hedl,
        "ditto": false
    });

    let result = execute_tool("hedl_format", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        // Should contain properly formatted output with version header
        assert!(text.contains("%VERSION"));
        assert!(text.contains("name"));
        assert!(text.contains("Test"));
    }
}

#[test]
fn test_hedl_convert_to_json() {
    let hedl = "%VERSION: 1.0\n---\nname: Test\nage: 30\n";

    let args = json!({
        "hedl": hedl,
        "format": "json",
        "options": {
            "pretty": true
        }
    });

    let result = execute_tool("hedl_convert_to", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        // Should be valid parseable JSON
        let json_result: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(json_result.is_object());
        // The output structure depends on the conversion implementation
        // Just verify we got JSON output containing the expected data
        let json_str = serde_json::to_string(&json_result).unwrap();
        assert!(json_str.contains("Test") || json_str.contains("name"));
    }
}

#[test]
fn test_hedl_convert_to_yaml() {
    let hedl = "%VERSION: 1.0\n---\nname: Test\n";

    let args = json!({
        "hedl": hedl,
        "format": "yaml"
    });

    let result = execute_tool("hedl_convert_to", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        assert!(text.contains("name"));
        assert!(text.contains("Test"));
    }
}

#[test]
fn test_hedl_convert_to_csv() {
    let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n  | bob, Bob\n";

    let args = json!({
        "hedl": hedl,
        "format": "csv",
        "options": {
            "include_headers": true
        }
    });

    let result = execute_tool("hedl_convert_to", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        assert!(text.contains("id,name") || text.contains("id") && text.contains("name"));
        assert!(text.contains("alice"));
        assert!(text.contains("bob"));
    }
}

#[test]
fn test_hedl_convert_to_unknown_format() {
    let hedl = "%VERSION: 1.0\n---\nname: Test\n";

    let args = json!({
        "hedl": hedl,
        "format": "unknown"
    });

    let result = execute_tool("hedl_convert_to", Some(args), Path::new("."));
    assert!(result.is_err());
    assert!(matches!(result, Err(McpError::InvalidArguments(_))));
}

#[test]
fn test_hedl_convert_from_json() {
    let json_data = r#"{"name": "Test", "age": 30}"#;

    let args = json!({
        "content": json_data,
        "format": "json"
    });

    let result = execute_tool("hedl_convert_from", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        assert!(text.contains("name"));
        assert!(text.contains("Test"));
    }
}

#[test]
fn test_hedl_convert_from_yaml() {
    let yaml_data = "name: Test\nage: 30\n";

    let args = json!({
        "content": yaml_data,
        "format": "yaml"
    });

    let result = execute_tool("hedl_convert_from", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        assert!(text.contains("name"));
        assert!(text.contains("Test"));
    }
}

#[test]
fn test_hedl_convert_from_csv() {
    let csv_data = "id,name\nalice,Alice\nbob,Bob\n";

    let args = json!({
        "content": csv_data,
        "format": "csv",
        "options": {
            "type_name": "User"
        }
    });

    let result = execute_tool("hedl_convert_from", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        assert!(text.contains("alice"));
        assert!(text.contains("bob"));
    }
}

#[test]
fn test_hedl_convert_from_invalid_json() {
    let args = json!({
        "content": "{ invalid json",
        "format": "json"
    });

    let result = execute_tool("hedl_convert_from", Some(args), Path::new("."));
    assert!(result.is_err());
}

#[test]
fn test_hedl_stream() {
    let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n  | bob, Bob\n  | charlie, Charlie\n";

    let args = json!({
        "hedl": hedl,
        "limit": 2,
        "offset": 0
    });

    let result = execute_tool("hedl_stream", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        let json_result: serde_json::Value = serde_json::from_str(text).unwrap();
        let entities = json_result["entities"].as_array().unwrap();
        assert_eq!(entities.len(), 2);
    }
}

#[test]
fn test_hedl_stream_with_offset() {
    let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n  | bob, Bob\n  | charlie, Charlie\n";

    let args = json!({
        "hedl": hedl,
        "limit": 2,
        "offset": 1
    });

    let result = execute_tool("hedl_stream", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        let json_result: serde_json::Value = serde_json::from_str(text).unwrap();
        let entities = json_result["entities"].as_array().unwrap();
        // Should skip first entity and return next 2
        assert_eq!(entities.len(), 2);
    }
}

#[test]
fn test_hedl_stream_with_type_filter() {
    let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n%STRUCT Product: [id, name]\n---\nusers: @User\n  | alice, Alice\nproducts: @Product\n  | p1, Product1\n";

    let args = json!({
        "hedl": hedl,
        "type_filter": "User"
    });

    let result = execute_tool("hedl_stream", Some(args), Path::new(".")).unwrap();

    if let Content::Text { text } = &result.content[0] {
        let json_result: serde_json::Value = serde_json::from_str(text).unwrap();
        let entities = json_result["entities"].as_array().unwrap();
        // Should only return User entities
        assert_eq!(entities.len(), 1);
    }
}

#[test]
fn test_tool_not_found() {
    let result = execute_tool("nonexistent_tool", None, Path::new("."));
    assert!(matches!(result, Err(McpError::ToolNotFound(_))));
}

#[test]
fn test_all_tools_have_valid_schemas() {
    let tools = get_tools();

    for tool in &tools {
        // Each tool should have a valid JSON schema
        assert!(tool.input_schema.is_object());
        assert_eq!(tool.input_schema["type"], "object");
        assert!(tool.input_schema.get("properties").is_some());
    }
}

#[test]
fn test_tools_count() {
    let tools = get_tools();
    // Should have 11 tools as documented
    assert_eq!(tools.len(), 11);
}

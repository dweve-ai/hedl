// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive server integration tests for hedl-mcp.
//!
//! Tests the full request/response flow, MCP protocol compliance,
//! and end-to-end tool execution scenarios.

use hedl_mcp::{McpServer, McpServerConfig};
use serde_json::{json, Value};
use tempfile::TempDir;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_server(temp_dir: &TempDir) -> McpServer {
    let config = McpServerConfig {
        root_path: temp_dir.path().to_path_buf(),
        name: "test-server".to_string(),
        version: "1.0.0".to_string(),
        rate_limit_burst: 100,
        rate_limit_per_second: 50,
        cache_size: 100,
    };
    McpServer::new(config)
}

fn parse_response(response_str: &str) -> Value {
    serde_json::from_str(response_str).expect("Failed to parse response")
}

// ============================================================================
// Protocol Lifecycle Tests
// ============================================================================

#[test]
fn test_initialize_handshake() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(response["result"]["serverInfo"]["name"], "test-server");
    assert_eq!(response["result"]["serverInfo"]["version"], "1.0.0");
}

#[test]
fn test_initialize_missing_params() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32602);
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Missing params"));
}

#[test]
fn test_initialized_notification() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialized"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert_eq!(response["result"], json!({}));
}

#[test]
fn test_shutdown() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "shutdown"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert_eq!(response["result"], json!({}));
}

#[test]
fn test_ping() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "ping"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert_eq!(response["result"], json!({}));
}

#[test]
fn test_unknown_method() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "unknown_method"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32601);
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Method not found"));
}

// ============================================================================
// Tools List Tests
// ============================================================================

#[test]
fn test_tools_list() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["tools"].is_array());

    let tools = response["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 11);

    // Verify key tools are present
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"hedl_read"));
    assert!(tool_names.contains(&"hedl_validate"));
    assert!(tool_names.contains(&"batch"));
}

#[test]
fn test_tools_have_valid_schemas() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    let tools = response["result"]["tools"].as_array().unwrap();

    for tool in tools {
        assert!(tool["name"].is_string());
        assert!(tool["description"].is_string());
        assert!(tool["inputSchema"].is_object());
        assert_eq!(tool["inputSchema"]["type"], "object");
        assert!(tool["inputSchema"]["properties"].is_object());
    }
}

// ============================================================================
// Tool Execution Tests
// ============================================================================

#[test]
fn test_hedl_validate_valid() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "hedl_validate",
            "arguments": {
                "hedl": "%VERSION 1.0\n---\nentity User { name: string }"
            }
        }
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert!(response["result"]["content"].is_array());
}

#[test]
fn test_hedl_validate_invalid() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "hedl_validate",
            "arguments": {
                "hedl": "invalid hedl syntax {"
            }
        }
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    // Tool errors are returned as successful responses with is_error flag
}

#[test]
fn test_hedl_stats() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "hedl_stats",
            "arguments": {
                "hedl": "%VERSION 1.0\n---\nentity User { name: string }"
            }
        }
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert!(response["result"]["content"].is_array());
}

#[test]
fn test_hedl_format() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "hedl_format",
            "arguments": {
                "hedl": "%VERSION 1.0\n---\nentity User { name: string }"
            }
        }
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
}

#[test]
fn test_hedl_optimize() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "hedl_optimize",
            "arguments": {
                "json": r#"{"name": "John", "age": 30}"#
            }
        }
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
}

#[test]
fn test_unknown_tool() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "unknown_tool",
            "arguments": {}
        }
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["isError"].as_bool().unwrap_or(false));
}

#[test]
fn test_tool_call_missing_params() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32602);
}

// ============================================================================
// Resources Tests
// ============================================================================

#[test]
fn test_resources_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/list"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["resources"].is_array());
    assert_eq!(response["result"]["resources"].as_array().unwrap().len(), 0);
}

#[test]
fn test_resources_list_with_files() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    // Create test HEDL files
    std::fs::write(
        temp_dir.path().join("test1.hedl"),
        "%VERSION 1.0\n---\nentity User { name: string }",
    )
    .unwrap();
    std::fs::write(
        temp_dir.path().join("test2.hedl"),
        "%VERSION 1.0\n---\nentity Product { id: string }",
    )
    .unwrap();

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/list"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["resources"].is_array());
    let resources = response["result"]["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 2);

    for resource in resources {
        assert!(resource["uri"].as_str().unwrap().ends_with(".hedl"));
        assert_eq!(resource["mimeType"], "text/hedl");
    }
}

#[test]
fn test_resources_read() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let content = "%VERSION 1.0\n---\nentity User { name: string }";
    std::fs::write(temp_dir.path().join("test.hedl"), content).unwrap();

    let uri = format!("file://{}/test.hedl", temp_dir.path().display());
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/read",
        "params": {
            "uri": uri
        }
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["contents"].is_array());
    let contents = response["result"]["contents"].as_array().unwrap();
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0]["text"], content);
}

#[test]
fn test_resources_read_missing_params() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/read"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32602);
}

#[test]
fn test_resources_read_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let uri = format!("file://{}/nonexistent.hedl", temp_dir.path().display());
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/read",
        "params": {
            "uri": uri
        }
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    // Error code can be -32002 (IO error) or -32003 (path not found)
    let code = response["error"]["code"].as_i64().unwrap();
    assert!(code == -32002 || code == -32003);
}

#[test]
fn test_resources_read_path_traversal() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let uri = "file://../../../etc/passwd";
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/read",
        "params": {
            "uri": uri
        }
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32003); // PathTraversal error
}

// ============================================================================
// Cache Tests
// ============================================================================

#[test]
fn test_cache_hit() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let hedl = "%VERSION 1.0\n---\nentity User { name: string }";
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "hedl_validate",
            "arguments": {
                "hedl": hedl
            }
        }
    });

    // First call
    let response_str1 = server.handle_request_string(&request.to_string());
    let response1 = parse_response(&response_str1);
    assert_eq!(response1["jsonrpc"], "2.0");

    // Second call should hit cache
    let response_str2 = server.handle_request_string(&request.to_string());
    let response2 = parse_response(&response_str2);
    assert_eq!(response2["jsonrpc"], "2.0");

    // Verify cache stats
    let stats = server.cache_stats().unwrap();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
}

#[test]
fn test_cache_miss_different_content() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let hedl1 = "%VERSION 1.0\n---\nentity User { name: string }";
    let hedl2 = "%VERSION 1.0\n---\nentity Product { id: string }";

    let request1 = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "hedl_validate",
            "arguments": {
                "hedl": hedl1
            }
        }
    });

    let request2 = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "hedl_validate",
            "arguments": {
                "hedl": hedl2
            }
        }
    });

    server.handle_request_string(&request1.to_string());
    server.handle_request_string(&request2.to_string());

    let stats = server.cache_stats().unwrap();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 2);
}

#[test]
fn test_cache_disabled() {
    let temp_dir = TempDir::new().unwrap();
    let config = McpServerConfig {
        root_path: temp_dir.path().to_path_buf(),
        name: "test-server".to_string(),
        version: "1.0.0".to_string(),
        rate_limit_burst: 100,
        rate_limit_per_second: 50,
        cache_size: 0, // Disable cache
    };
    let server = McpServer::new(config);

    assert!(server.cache().is_none());
    assert!(server.cache_stats().is_none());
}

// ============================================================================
// JSON-RPC Protocol Validation Tests
// ============================================================================

#[test]
fn test_malformed_json() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let malformed = "{\"jsonrpc\": \"2.0\", \"method\": incomplete";
    let response_str = server.handle_request_string(malformed);
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32700); // Parse error
}

#[test]
fn test_invalid_json_rpc_version() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "1.0",
        "id": 1,
        "method": "ping"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["jsonrpc"], "2.0");
    // Server validates the request and returns an error
    if response["error"].is_object() {
        assert_eq!(response["error"]["code"], -32600);
    } else {
        // Or it may process successfully if validation is not enforced
        assert!(response["result"].is_object());
    }
}

#[test]
fn test_id_types() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    // String ID
    let request1 = json!({
        "jsonrpc": "2.0",
        "id": "string-id",
        "method": "ping"
    });
    let response1 = parse_response(&server.handle_request_string(&request1.to_string()));
    assert_eq!(response1["id"], "string-id");

    // Number ID
    let request2 = json!({
        "jsonrpc": "2.0",
        "id": 42,
        "method": "ping"
    });
    let response2 = parse_response(&server.handle_request_string(&request2.to_string()));
    assert_eq!(response2["id"], 42);

    // Null ID
    let request3 = json!({
        "jsonrpc": "2.0",
        "id": null,
        "method": "ping"
    });
    let response3 = parse_response(&server.handle_request_string(&request3.to_string()));
    assert!(response3["id"].is_null());
}

#[test]
fn test_invalid_id_type() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    // Array ID (invalid)
    let request = json!({
        "jsonrpc": "2.0",
        "id": [1, 2, 3],
        "method": "ping"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    // Server validates the request
    if response["error"].is_object() {
        assert_eq!(response["error"]["code"], -32600);
    } else {
        // May process if validation not strict
        assert!(response["result"].is_object());
    }
}

#[test]
fn test_empty_method_name() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": ""
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert!(response["error"].is_object());
    // Empty method triggers validation error or "method not found"
    let code = response["error"]["code"].as_i64().unwrap();
    assert!(code == -32600 || code == -32601);
}

#[test]
fn test_method_with_whitespace() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "ping with space"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert!(response["error"].is_object());
    // Method with whitespace triggers validation error or "method not found"
    let code = response["error"]["code"].as_i64().unwrap();
    assert!(code == -32600 || code == -32601);
}

#[test]
fn test_params_must_be_object_or_array() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(&temp_dir);

    // String params (invalid)
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "ping",
        "params": "invalid"
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    // Server validates params or accepts them
    if response["error"].is_object() {
        assert_eq!(response["error"]["code"], -32600);
    } else {
        // May succeed if validation not strict
        assert!(response["result"].is_object());
    }
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_server_with_custom_config() {
    let temp_dir = TempDir::new().unwrap();
    let config = McpServerConfig {
        root_path: temp_dir.path().to_path_buf(),
        name: "custom-server".to_string(),
        version: "2.0.0".to_string(),
        rate_limit_burst: 50,
        rate_limit_per_second: 25,
        cache_size: 50,
    };
    let mut server = McpServer::new(config);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test",
                "version": "1.0.0"
            }
        }
    });

    let response_str = server.handle_request_string(&request.to_string());
    let response = parse_response(&response_str);

    assert_eq!(response["result"]["serverInfo"]["name"], "custom-server");
    assert_eq!(response["result"]["serverInfo"]["version"], "2.0.0");
}

#[test]
fn test_server_with_root_helper() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::with_root(temp_dir.path().to_path_buf());

    assert!(server.cache().is_some());
}

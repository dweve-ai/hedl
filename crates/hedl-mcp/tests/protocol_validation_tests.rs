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

//! Protocol validation tests for MCP server.
//!
//! Tests JSON-RPC 2.0 compliance, MCP protocol adherence, and request/response validation.

use hedl_mcp::{
    ClientCapabilities, ClientInfo, Content, InitializeParams, InitializeResult, JsonRpcRequest,
    JsonRpcResponse, ListToolsResult, McpServer, McpServerConfig,
};
use serde_json::{json, Value};
use std::path::PathBuf;

// =============================================================================
// TEST HELPERS
// =============================================================================

fn create_test_server() -> McpServer {
    let config = McpServerConfig {
        root_path: PathBuf::from("."),
        rate_limit_burst: 0, // Disable rate limiting for tests
        rate_limit_per_second: 0,
        ..Default::default()
    };
    McpServer::new(config)
}

fn make_request(method: &str, params: Option<Value>, id: Option<Value>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id,
    }
}

fn initialize_server(server: &mut McpServer) -> JsonRpcResponse {
    let request = make_request(
        "initialize",
        Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
        Some(json!(1)),
    );
    server.handle_request(request)
}

// =============================================================================
// JSON-RPC 2.0 COMPLIANCE
// =============================================================================

#[test]
fn test_jsonrpc_version_in_response() {
    let mut server = create_test_server();
    let response = initialize_server(&mut server);

    assert_eq!(
        response.jsonrpc, "2.0",
        "Response must include jsonrpc: 2.0"
    );
}

#[test]
fn test_response_contains_id_matching_request() {
    let mut server = create_test_server();

    // Test with numeric ID
    let request = make_request(
        "initialize",
        Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" }
        })),
        Some(json!(42)),
    );
    let response = server.handle_request(request);
    assert_eq!(response.id, Some(json!(42)));

    // Test with string ID
    let request = make_request("ping", None, Some(json!("request-uuid-123")));
    let response = server.handle_request(request);
    assert_eq!(response.id, Some(json!("request-uuid-123")));

    // Test with null ID (notification-style but expecting response)
    let request = make_request("ping", None, Some(Value::Null));
    let response = server.handle_request(request);
    assert_eq!(response.id, Some(Value::Null));
}

#[test]
fn test_success_response_has_result_no_error() {
    let mut server = create_test_server();
    let response = initialize_server(&mut server);

    assert!(
        response.result.is_some(),
        "Success response must have result"
    );
    assert!(
        response.error.is_none(),
        "Success response must not have error"
    );
}

#[test]
fn test_error_response_has_error_no_result() {
    let mut server = create_test_server();

    // Unknown method should return error
    let request = make_request("unknown_method", None, Some(json!(1)));
    let response = server.handle_request(request);

    assert!(
        response.result.is_none(),
        "Error response must not have result"
    );
    assert!(response.error.is_some(), "Error response must have error");
}

#[test]
fn test_error_codes_standard_compliance() {
    let mut server = create_test_server();

    // -32601: Method not found
    let response = server.handle_request(make_request("nonexistent", None, Some(json!(1))));
    assert_eq!(
        response.error.as_ref().unwrap().code,
        -32601,
        "Unknown method should return -32601"
    );

    // -32602: Invalid params
    let response = server.handle_request(make_request(
        "initialize",
        Some(json!("not an object")),
        Some(json!(2)),
    ));
    assert_eq!(
        response.error.as_ref().unwrap().code,
        -32602,
        "Invalid params should return -32602"
    );

    // -32602: Missing params
    let response = server.handle_request(make_request("initialize", None, Some(json!(3))));
    assert_eq!(
        response.error.as_ref().unwrap().code,
        -32602,
        "Missing params should return -32602"
    );
}

#[test]
fn test_error_object_structure() {
    let mut server = create_test_server();

    let response = server.handle_request(make_request("unknown", None, Some(json!(1))));
    let error = response.error.unwrap();

    // JSON-RPC 2.0 error object requirements
    assert!(error.code != 0, "Error code must be non-zero");
    assert!(!error.message.is_empty(), "Error message must be non-empty");
    // data is optional
}

// =============================================================================
// MCP PROTOCOL COMPLIANCE
// =============================================================================

#[test]
fn test_initialize_returns_valid_result() {
    let mut server = create_test_server();
    let response = initialize_server(&mut server);

    let result: InitializeResult =
        serde_json::from_value(response.result.unwrap()).expect("Should parse InitializeResult");

    // Protocol version must be present
    assert!(
        !result.protocol_version.is_empty(),
        "Protocol version required"
    );

    // Server info required
    assert!(!result.server_info.name.is_empty(), "Server name required");
    assert!(
        !result.server_info.version.is_empty(),
        "Server version required"
    );

    // Capabilities must be present (can be empty)
    // Just verifying it exists by successful parse
}

#[test]
fn test_server_capabilities_structure() {
    let mut server = create_test_server();
    let response = initialize_server(&mut server);

    let result: InitializeResult = serde_json::from_value(response.result.unwrap()).unwrap();

    // HEDL MCP server should advertise tools and resources
    assert!(
        result.capabilities.tools.is_some(),
        "Should advertise tools capability"
    );
    assert!(
        result.capabilities.resources.is_some(),
        "Should advertise resources capability"
    );
}

#[test]
fn test_tools_list_returns_array_of_tools() {
    let mut server = create_test_server();
    initialize_server(&mut server);

    let response = server.handle_request(make_request("tools/list", None, Some(json!(2))));

    let result: ListToolsResult =
        serde_json::from_value(response.result.unwrap()).expect("Should parse ListToolsResult");

    assert!(!result.tools.is_empty(), "Should have at least one tool");

    for tool in &result.tools {
        // Each tool must have required fields
        assert!(!tool.name.is_empty(), "Tool name required");
        assert!(!tool.description.is_empty(), "Tool description required");
        assert!(
            tool.input_schema.is_object(),
            "Tool input_schema must be an object"
        );
    }
}

#[test]
fn test_tool_input_schema_is_valid_json_schema() {
    let mut server = create_test_server();
    initialize_server(&mut server);

    let response = server.handle_request(make_request("tools/list", None, Some(json!(1))));
    let result: ListToolsResult = serde_json::from_value(response.result.unwrap()).unwrap();

    for tool in &result.tools {
        let schema = &tool.input_schema;

        // JSON Schema must have "type" field
        assert!(
            schema.get("type").is_some(),
            "Tool {} input_schema must have type field",
            tool.name
        );
    }
}

#[test]
fn test_tools_call_with_valid_arguments() {
    let mut server = create_test_server();
    initialize_server(&mut server);

    // Call hedl_validate tool with valid HEDL
    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "hedl_validate",
            "arguments": {
                "hedl": "#HEDL 1.0\n"
            }
        })),
        Some(json!(3)),
    ));

    assert!(response.error.is_none(), "Valid tool call should succeed");

    let result = response.result.unwrap();
    assert!(
        result.get("content").is_some(),
        "Tool result should have content"
    );
}

#[test]
fn test_tools_call_with_unknown_tool() {
    let mut server = create_test_server();
    initialize_server(&mut server);

    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "nonexistent_tool",
            "arguments": {}
        })),
        Some(json!(1)),
    ));

    // Unknown tool should return a success with is_error: true
    // (per MCP spec, tool errors are success responses with error content)
    assert!(
        response.error.is_none(),
        "Tool errors should be success responses"
    );

    let result = response.result.unwrap();
    assert_eq!(
        result.get("isError"),
        Some(&json!(true)),
        "Unknown tool should have isError: true"
    );
}

#[test]
fn test_initialized_notification() {
    let mut server = create_test_server();
    initialize_server(&mut server);

    // Send initialized notification
    let response = server.handle_request(make_request("initialized", None, Some(json!(2))));

    // Should succeed
    assert!(
        response.error.is_none(),
        "initialized notification should succeed"
    );
}

#[test]
fn test_shutdown_method() {
    let mut server = create_test_server();
    initialize_server(&mut server);

    let response = server.handle_request(make_request("shutdown", None, Some(json!(3))));

    assert!(response.error.is_none(), "shutdown should succeed");
}

#[test]
fn test_ping_method() {
    let mut server = create_test_server();

    let response = server.handle_request(make_request("ping", None, Some(json!(1))));

    assert!(response.error.is_none(), "ping should succeed");
    assert_eq!(response.result, Some(json!({})));
}

// =============================================================================
// REQUEST VALIDATION
// =============================================================================

#[test]
fn test_initialize_params_validation() {
    let mut server = create_test_server();

    // Missing protocolVersion
    let response = server.handle_request(make_request(
        "initialize",
        Some(json!({
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" }
        })),
        Some(json!(1)),
    ));
    assert!(
        response.error.is_some(),
        "Missing protocolVersion should error"
    );

    // Missing clientInfo
    let response = server.handle_request(make_request(
        "initialize",
        Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {}
        })),
        Some(json!(2)),
    ));
    assert!(response.error.is_some(), "Missing clientInfo should error");

    // Invalid type for capabilities
    let response = server.handle_request(make_request(
        "initialize",
        Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": "not an object",
            "clientInfo": { "name": "test", "version": "1.0" }
        })),
        Some(json!(3)),
    ));
    assert!(
        response.error.is_some(),
        "Invalid capabilities type should error"
    );
}

#[test]
fn test_tools_call_params_validation() {
    let mut server = create_test_server();
    initialize_server(&mut server);

    // Missing name
    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "arguments": {}
        })),
        Some(json!(1)),
    ));
    assert!(response.error.is_some(), "Missing tool name should error");

    // Invalid params type
    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!("not an object")),
        Some(json!(2)),
    ));
    assert!(response.error.is_some(), "Invalid params type should error");
}

#[test]
fn test_resources_read_params_validation() {
    let mut server = create_test_server();
    initialize_server(&mut server);

    // Missing uri
    let response = server.handle_request(make_request(
        "resources/read",
        Some(json!({})),
        Some(json!(1)),
    ));
    assert!(response.error.is_some(), "Missing uri should error");
}

// =============================================================================
// RESPONSE STRUCTURE VALIDATION
// =============================================================================

#[test]
fn test_tool_call_result_structure() {
    let mut server = create_test_server();
    initialize_server(&mut server);

    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "hedl_validate",
            "arguments": { "hedl": "#HEDL 1.0\n" }
        })),
        Some(json!(1)),
    ));

    let result = response.result.unwrap();

    // Must have content array
    assert!(
        result.get("content").is_some(),
        "Result must have content field"
    );
    assert!(result["content"].is_array(), "content must be an array");

    // Each content item should have type
    for item in result["content"].as_array().unwrap() {
        assert!(item.get("type").is_some(), "Content item must have type");
    }
}

#[test]
fn test_resources_list_result_structure() {
    let mut server = create_test_server();
    initialize_server(&mut server);

    let response = server.handle_request(make_request("resources/list", None, Some(json!(1))));

    let result = response.result.unwrap();

    // Must have resources array
    assert!(
        result.get("resources").is_some(),
        "Result must have resources field"
    );
    assert!(result["resources"].is_array(), "resources must be an array");

    // Each resource should have required fields (if any)
    for resource in result["resources"].as_array().unwrap() {
        assert!(resource.get("uri").is_some(), "Resource must have uri");
        assert!(resource.get("name").is_some(), "Resource must have name");
    }
}

// =============================================================================
// EDGE CASES
// =============================================================================

#[test]
fn test_empty_params() {
    let mut server = create_test_server();
    initialize_server(&mut server);

    // tools/list with empty params should work
    let response =
        server.handle_request(make_request("tools/list", Some(json!({})), Some(json!(1))));
    assert!(response.error.is_none());

    // resources/list with empty params should work
    let response = server.handle_request(make_request(
        "resources/list",
        Some(json!({})),
        Some(json!(2)),
    ));
    assert!(response.error.is_none());
}

#[test]
fn test_null_id() {
    let mut server = create_test_server();

    let request = make_request("ping", None, Some(Value::Null));
    let response = server.handle_request(request);

    // null ID should be preserved
    assert_eq!(response.id, Some(Value::Null));
}

#[test]
fn test_no_id_notification_style() {
    let mut server = create_test_server();

    // Request without ID (notification-style)
    let request = make_request("ping", None, None);
    let response = server.handle_request(request);

    // Response ID should be None for notifications
    assert_eq!(response.id, None);
}

#[test]
fn test_large_id_values() {
    let mut server = create_test_server();

    // Large numeric ID
    let request = make_request("ping", None, Some(json!(9007199254740991i64)));
    let response = server.handle_request(request);
    assert_eq!(response.id, Some(json!(9007199254740991i64)));

    // Long string ID
    let long_id = "x".repeat(1000);
    let request = make_request("ping", None, Some(json!(long_id)));
    let response = server.handle_request(request);
    assert_eq!(response.id, Some(json!(long_id)));
}

#[test]
fn test_unicode_in_params() {
    let mut server = create_test_server();
    initialize_server(&mut server);

    // Unicode in tool arguments
    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "hedl_validate",
            "arguments": {
                "hedl": "#HEDL 1.0\n// Unicode: \u{4e2d}\u{6587} \u{1F600} \u{1F4BB}"
            }
        })),
        Some(json!(1)),
    ));

    // Should process without error
    assert!(response.error.is_none());
}

#[test]
fn test_extra_fields_ignored() {
    let mut server = create_test_server();

    // Request with extra fields
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "ping".to_string(),
        params: Some(json!({ "extra_field": "should be ignored" })),
        id: Some(json!(1)),
    };

    let response = server.handle_request(request);
    assert!(response.error.is_none(), "Extra fields should be ignored");
}

// =============================================================================
// SERIALIZATION/DESERIALIZATION
// =============================================================================

#[test]
fn test_request_serialization_roundtrip() {
    let original = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: Some(json!({ "name": "test", "arguments": {} })),
        id: Some(json!(42)),
    };

    let serialized = serde_json::to_string(&original).unwrap();
    let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.jsonrpc, original.jsonrpc);
    assert_eq!(deserialized.method, original.method);
    assert_eq!(deserialized.params, original.params);
    assert_eq!(deserialized.id, original.id);
}

#[test]
fn test_response_serialization_roundtrip() {
    let success = JsonRpcResponse::success(Some(json!(1)), json!({"key": "value"}));
    let serialized = serde_json::to_string(&success).unwrap();
    let deserialized: JsonRpcResponse = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.jsonrpc, "2.0");
    assert_eq!(deserialized.id, Some(json!(1)));
    assert!(deserialized.result.is_some());
    assert!(deserialized.error.is_none());

    let error = JsonRpcResponse::error(Some(json!(2)), -32601, "Not found".to_string(), None);
    let serialized = serde_json::to_string(&error).unwrap();
    let deserialized: JsonRpcResponse = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.jsonrpc, "2.0");
    assert_eq!(deserialized.id, Some(json!(2)));
    assert!(deserialized.result.is_none());
    assert!(deserialized.error.is_some());
    assert_eq!(deserialized.error.unwrap().code, -32601);
}

#[test]
fn test_init_params_serialization() {
    let params = InitializeParams {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ClientCapabilities::default(),
        client_info: ClientInfo {
            name: "test".to_string(),
            version: "1.0".to_string(),
        },
    };

    let serialized = serde_json::to_string(&params).unwrap();
    let deserialized: InitializeParams = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.protocol_version, params.protocol_version);
    assert_eq!(deserialized.client_info.name, params.client_info.name);
}

// =============================================================================
// CONTENT TYPES
// =============================================================================

#[test]
fn test_text_content_type() {
    let content = Content::Text {
        text: "Hello, world!".to_string(),
    };

    let serialized = serde_json::to_string(&content).unwrap();
    assert!(serialized.contains("\"type\":\"text\""));
    assert!(serialized.contains("\"text\":\"Hello, world!\""));

    let deserialized: Content = serde_json::from_str(&serialized).unwrap();
    match deserialized {
        Content::Text { text } => assert_eq!(text, "Hello, world!"),
        _ => panic!("Expected Text content"),
    }
}

#[test]
fn test_resource_content_type() {
    use hedl_mcp::ResourceContent;

    let content = Content::Resource {
        resource: ResourceContent {
            uri: "file:///test.hedl".to_string(),
            mime_type: Some("text/hedl".to_string()),
            text: Some("content".to_string()),
        },
    };

    let serialized = serde_json::to_string(&content).unwrap();
    assert!(serialized.contains("\"type\":\"resource\""));

    let deserialized: Content = serde_json::from_str(&serialized).unwrap();
    match deserialized {
        Content::Resource { resource } => {
            assert_eq!(resource.uri, "file:///test.hedl");
            assert_eq!(resource.mime_type, Some("text/hedl".to_string()));
        }
        _ => panic!("Expected Resource content"),
    }
}

// =============================================================================
// CAPABILITY NEGOTIATION
// =============================================================================

#[test]
fn test_client_capabilities_optional_fields() {
    let caps: ClientCapabilities = serde_json::from_value(json!({})).unwrap();
    assert!(caps.roots.is_none());
    assert!(caps.sampling.is_none());

    let caps: ClientCapabilities = serde_json::from_value(json!({
        "roots": { "listChanged": true }
    }))
    .unwrap();
    assert!(caps.roots.is_some());
    assert_eq!(caps.roots.unwrap().list_changed, Some(true));
}

#[test]
fn test_server_capabilities_fields() {
    let mut server = create_test_server();
    let response = initialize_server(&mut server);

    let result: InitializeResult = serde_json::from_value(response.result.unwrap()).unwrap();

    // Verify capability structure
    if let Some(tools) = &result.capabilities.tools {
        // listChanged is optional
        let _ = tools.list_changed;
    }

    if let Some(resources) = &result.capabilities.resources {
        // subscribe and listChanged are optional
        let _ = resources.subscribe;
        let _ = resources.list_changed;
    }
}

// =============================================================================
// PROTOCOL VERSION HANDLING
// =============================================================================

#[test]
fn test_protocol_version_response() {
    let mut server = create_test_server();
    let response = initialize_server(&mut server);

    let result: InitializeResult = serde_json::from_value(response.result.unwrap()).unwrap();

    // Server should return a valid protocol version
    assert!(
        !result.protocol_version.is_empty(),
        "Protocol version should not be empty"
    );

    // Version should follow expected format (YYYY-MM-DD)
    let parts: Vec<&str> = result.protocol_version.split('-').collect();
    assert_eq!(
        parts.len(),
        3,
        "Protocol version should be YYYY-MM-DD format"
    );
}

// =============================================================================
// METHOD ENUMERATION
// =============================================================================

#[test]
fn test_all_standard_methods() {
    let mut server = create_test_server();

    // Initialize first
    initialize_server(&mut server);

    let methods = vec![
        (
            "initialize",
            Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            })),
        ),
        ("initialized", None),
        ("shutdown", None),
        ("tools/list", None),
        ("resources/list", None),
        ("ping", None),
    ];

    // Re-initialize for clean state
    let mut server = create_test_server();
    initialize_server(&mut server);

    for (method, params) in methods {
        let response = server.handle_request(make_request(method, params, Some(json!(1))));

        // None of these should return Method Not Found
        if let Some(error) = &response.error {
            assert_ne!(error.code, -32601, "Method {method} should be implemented");
        }
    }
}

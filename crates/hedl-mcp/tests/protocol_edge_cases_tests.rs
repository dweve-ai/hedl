// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Protocol edge case tests for hedl-mcp.
//!
//! Tests handling of malformed requests, boundary conditions,
//! and protocol compliance edge cases.

use hedl_mcp::*;
use serde_json::{json, Value};

// ============================================================================
// JSON-RPC Request Validation Tests
// ============================================================================

#[test]
fn test_validate_correct_request() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test_method".to_string(),
        params: Some(json!({"key": "value"})),
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_validate_wrong_version() {
    let request = JsonRpcRequest {
        jsonrpc: "1.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: None,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid JSON-RPC version"));
}

#[test]
fn test_validate_invalid_id_bool() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(true)),
        method: "test".to_string(),
        params: None,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid request id"));
}

#[test]
fn test_validate_invalid_id_array() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!([1, 2, 3])),
        method: "test".to_string(),
        params: None,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid request id"));
}

#[test]
fn test_validate_invalid_id_object() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!({"key": "value"})),
        method: "test".to_string(),
        params: None,
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_empty_method() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: String::new(),
        params: None,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Method name cannot be empty"));
}

#[test]
fn test_validate_method_with_whitespace() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "method with space".to_string(),
        params: None,
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Method name cannot contain whitespace"));
}

#[test]
fn test_validate_method_with_tab() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "method\twith\ttab".to_string(),
        params: None,
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_method_with_newline() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "method\nwith\nnewline".to_string(),
        params: None,
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_params_string_invalid() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(json!("invalid")),
    };

    let result = request.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Params must be an object or array"));
}

#[test]
fn test_validate_params_number_invalid() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(json!(42)),
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_params_bool_invalid() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(json!(true)),
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_params_object_valid() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(json!({"key": "value"})),
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_validate_params_array_valid() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(json!([1, 2, 3])),
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_validate_params_empty_object() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(json!({})),
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_validate_params_empty_array() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(json!([])),
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_validate_params_null() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(Value::Null),
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_no_id() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: None,
        method: "test".to_string(),
        params: None,
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_validate_null_id() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(Value::Null),
        method: "test".to_string(),
        params: None,
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_validate_string_id() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!("request-123")),
        method: "test".to_string(),
        params: None,
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_validate_number_id() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(42)),
        method: "test".to_string(),
        params: None,
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_validate_float_id() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3.5)),
        method: "test".to_string(),
        params: None,
    };

    assert!(request.validate().is_ok());
}

// ============================================================================
// JSON-RPC Response Construction Tests
// ============================================================================

#[test]
fn test_success_response() {
    let response = JsonRpcResponse::success(Some(json!(1)), json!({"result": "ok"}));

    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, Some(json!(1)));
    assert_eq!(response.result, Some(json!({"result": "ok"})));
    assert!(response.error.is_none());
}

#[test]
fn test_success_response_no_id() {
    let response = JsonRpcResponse::success(None, json!({}));

    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.id.is_none());
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_error_response() {
    let response = JsonRpcResponse::error(
        Some(json!(1)),
        -32600,
        "Invalid Request".to_string(),
        Some(json!({"details": "extra info"})),
    );

    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, Some(json!(1)));
    assert!(response.result.is_none());
    assert!(response.error.is_some());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32600);
    assert_eq!(error.message, "Invalid Request");
    assert_eq!(error.data, Some(json!({"details": "extra info"})));
}

#[test]
fn test_error_response_no_data() {
    let response =
        JsonRpcResponse::error(Some(json!(1)), -32601, "Method not found".to_string(), None);

    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert!(error.data.is_none());
}

// ============================================================================
// Protocol Types Serialization Tests
// ============================================================================

#[test]
fn test_serialize_initialize_params() {
    let params = InitializeParams {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ClientCapabilities::default(),
        client_info: ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["protocolVersion"], "2024-11-05");
    assert_eq!(json["clientInfo"]["name"], "test-client");
}

#[test]
fn test_serialize_server_capabilities() {
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(false),
            list_changed: Some(true),
        }),
        prompts: None,
    };

    let json = serde_json::to_value(&capabilities).unwrap();
    assert_eq!(json["tools"]["listChanged"], true);
    assert_eq!(json["resources"]["subscribe"], false);
    assert!(json["prompts"].is_null());
}

#[test]
fn test_serialize_tool() {
    let tool = Tool {
        name: "test_tool".to_string(),
        description: "A test tool".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "param": {"type": "string"}
            }
        }),
    };

    let json = serde_json::to_value(&tool).unwrap();
    assert_eq!(json["name"], "test_tool");
    assert_eq!(json["description"], "A test tool");
    assert_eq!(json["inputSchema"]["type"], "object");
}

#[test]
fn test_serialize_call_tool_result() {
    let result = CallToolResult {
        content: vec![Content::Text {
            text: "result text".to_string(),
        }],
        is_error: Some(false),
    };

    let json = serde_json::to_value(&result).unwrap();
    assert!(json["content"].is_array());
    assert_eq!(json["content"][0]["type"], "text");
    assert_eq!(json["content"][0]["text"], "result text");
    assert_eq!(json["isError"], false);
}

#[test]
fn test_serialize_call_tool_result_no_error_flag() {
    let result = CallToolResult {
        content: vec![Content::Text {
            text: "test".to_string(),
        }],
        is_error: None,
    };

    let json = serde_json::to_value(&result).unwrap();
    assert!(json.get("isError").is_none());
}

#[test]
fn test_serialize_resource_content() {
    let content = ResourceContent {
        uri: "file:///test.hedl".to_string(),
        mime_type: Some("text/hedl".to_string()),
        text: Some("content".to_string()),
    };

    let json = serde_json::to_value(&content).unwrap();
    assert_eq!(json["uri"], "file:///test.hedl");
    assert_eq!(json["mimeType"], "text/hedl");
    assert_eq!(json["text"], "content");
}

#[test]
fn test_content_text_variant() {
    let content = Content::Text {
        text: "test content".to_string(),
    };

    let json = serde_json::to_value(&content).unwrap();
    assert_eq!(json["type"], "text");
    assert_eq!(json["text"], "test content");
}

#[test]
fn test_content_resource_variant() {
    let resource = ResourceContent {
        uri: "file:///test.hedl".to_string(),
        mime_type: Some("text/hedl".to_string()),
        text: Some("content".to_string()),
    };

    let content = Content::Resource { resource };

    let json = serde_json::to_value(&content).unwrap();
    assert_eq!(json["type"], "resource");
    assert_eq!(json["resource"]["uri"], "file:///test.hedl");
}

// ============================================================================
// Protocol Types Deserialization Tests
// ============================================================================

#[test]
fn test_deserialize_json_rpc_request() {
    let json = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "test",
        "params": {"key": "value"}
    });

    let request: JsonRpcRequest = serde_json::from_value(json).unwrap();
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.id, Some(json!(1)));
    assert_eq!(request.method, "test");
    assert!(request.params.is_some());
}

#[test]
fn test_deserialize_json_rpc_request_no_params() {
    let json = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "test"
    });

    let request: JsonRpcRequest = serde_json::from_value(json).unwrap();
    assert!(request.params.is_none());
}

#[test]
fn test_deserialize_json_rpc_response_success() {
    let json = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {"data": "value"}
    });

    let response: JsonRpcResponse = serde_json::from_value(json).unwrap();
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_deserialize_json_rpc_response_error() {
    let json = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32600,
            "message": "Invalid Request"
        }
    });

    let response: JsonRpcResponse = serde_json::from_value(json).unwrap();
    assert!(response.result.is_none());
    assert!(response.error.is_some());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32600);
    assert_eq!(error.message, "Invalid Request");
}

#[test]
fn test_deserialize_call_tool_params() {
    let json = json!({
        "name": "test_tool",
        "arguments": {"param": "value"}
    });

    let params: CallToolParams = serde_json::from_value(json).unwrap();
    assert_eq!(params.name, "test_tool");
    assert!(params.arguments.is_some());
}

#[test]
fn test_deserialize_call_tool_params_no_arguments() {
    let json = json!({
        "name": "test_tool"
    });

    let params: CallToolParams = serde_json::from_value(json).unwrap();
    assert_eq!(params.name, "test_tool");
    assert!(params.arguments.is_none());
}

#[test]
fn test_deserialize_read_resource_params() {
    let json = json!({
        "uri": "file:///test.hedl"
    });

    let params: ReadResourceParams = serde_json::from_value(json).unwrap();
    assert_eq!(params.uri, "file:///test.hedl");
}

// ============================================================================
// Boundary Condition Tests
// ============================================================================

#[test]
fn test_very_long_method_name() {
    let long_method = "a".repeat(10000);
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: long_method.clone(),
        params: None,
    };

    // Should pass validation (no max length constraint)
    assert!(request.validate().is_ok());
}

#[test]
fn test_very_long_error_message() {
    let long_message = "x".repeat(10000);
    let response = JsonRpcResponse::error(Some(json!(1)), -32600, long_message.clone(), None);

    assert_eq!(response.error.unwrap().message, long_message);
}

#[test]
fn test_deeply_nested_params() {
    let mut value = json!("leaf");
    for _ in 0..100 {
        value = json!({"nested": value});
    }

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(value),
    };

    // Should serialize/deserialize correctly
    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.method, "test");
}

#[test]
fn test_large_array_params() {
    let large_array: Vec<i32> = (0..10000).collect();
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(json!(large_array)),
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_unicode_in_method_name() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test_🚀_method".to_string(),
        params: None,
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_unicode_in_params() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(json!({"text": "Hello 世界 🌍"})),
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_zero_error_code() {
    let response = JsonRpcResponse::error(Some(json!(1)), 0, "Zero code".to_string(), None);
    assert_eq!(response.error.unwrap().code, 0);
}

#[test]
fn test_positive_error_code() {
    let response = JsonRpcResponse::error(Some(json!(1)), 100, "Positive code".to_string(), None);
    assert_eq!(response.error.unwrap().code, 100);
}

#[test]
fn test_large_negative_error_code() {
    let response = JsonRpcResponse::error(
        Some(json!(1)),
        -99999,
        "Large negative code".to_string(),
        None,
    );
    assert_eq!(response.error.unwrap().code, -99999);
}

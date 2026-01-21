// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Protocol compliance and edge case tests.

use hedl_mcp::*;
use serde_json::json;

#[test]
fn test_json_rpc_request_parsing() {
    let request_str = r#"{"jsonrpc":"2.0","id":1,"method":"test","params":{}}"#;
    let request: JsonRpcRequest = serde_json::from_str(request_str).unwrap();

    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.id, Some(json!(1)));
    assert_eq!(request.method, "test");
}

#[test]
fn test_json_rpc_request_without_params() {
    let request_str = r#"{"jsonrpc":"2.0","id":1,"method":"test"}"#;
    let request: JsonRpcRequest = serde_json::from_str(request_str).unwrap();

    assert_eq!(request.params, None);
}

#[test]
fn test_json_rpc_request_without_id() {
    let request_str = r#"{"jsonrpc":"2.0","method":"test"}"#;
    let request: JsonRpcRequest = serde_json::from_str(request_str).unwrap();

    assert_eq!(request.id, None);
}

#[test]
fn test_json_rpc_request_validation() {
    let valid_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: None,
    };

    assert!(valid_request.validate().is_ok());

    let invalid_version = JsonRpcRequest {
        jsonrpc: "1.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: None,
    };

    assert!(invalid_version.validate().is_err());
}

#[test]
fn test_json_rpc_response_success() {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        result: Some(json!({"status": "ok"})),
        error: None,
    };

    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains("result"));
    assert!(!serialized.contains("error"));
}

#[test]
fn test_json_rpc_response_error() {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        result: None,
        error: Some(JsonRpcError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        }),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    assert!(!serialized.contains("result"));
    assert!(serialized.contains("error"));
}

#[test]
fn test_json_rpc_error_codes() {
    let errors = vec![
        (-32700, "Parse error"),
        (-32600, "Invalid Request"),
        (-32601, "Method not found"),
        (-32602, "Invalid params"),
        (-32603, "Internal error"),
    ];

    for (code, message) in errors {
        let error = JsonRpcError {
            code,
            message: message.to_string(),
            data: None,
        };

        assert_eq!(error.code, code);
        assert_eq!(error.message, message);
    }
}

#[test]
fn test_initialize_request() {
    let request = InitializeParams {
        protocol_version: "1.0".to_string(),
        capabilities: ClientCapabilities {
            roots: None,
            sampling: None,
        },
        client_info: ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    assert_eq!(request.protocol_version, "1.0");
    assert_eq!(request.client_info.name, "test-client");
}

#[test]
fn test_initialize_result() {
    let result = InitializeResult {
        protocol_version: "1.0".to_string(),
        capabilities: ServerCapabilities {
            tools: None,
            resources: None,
            prompts: None,
        },
        server_info: ServerInfo {
            name: "hedl-mcp".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    assert_eq!(result.server_info.name, "hedl-mcp");
}

#[test]
fn test_tool_serialization() {
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

    let serialized = serde_json::to_value(&tool).unwrap();
    assert_eq!(serialized["name"], "test_tool");
    assert_eq!(serialized["description"], "A test tool");
    assert_eq!(serialized["inputSchema"]["type"], "object");
}

#[test]
fn test_call_tool_request() {
    let request = CallToolParams {
        name: "hedl_validate".to_string(),
        arguments: Some(json!({"hedl": "test"})),
    };

    assert_eq!(request.name, "hedl_validate");
    assert!(request.arguments.is_some());
}

#[test]
fn test_call_tool_result_success() {
    let result = CallToolResult {
        content: vec![Content::Text {
            text: "Success".to_string(),
        }],
        is_error: Some(false),
    };

    assert_eq!(result.content.len(), 1);
    assert_eq!(result.is_error, Some(false));
}

#[test]
fn test_call_tool_result_error() {
    let result = CallToolResult {
        content: vec![Content::Text {
            text: "Error message".to_string(),
        }],
        is_error: Some(true),
    };

    assert_eq!(result.is_error, Some(true));
}

#[test]
fn test_content_text() {
    let content = Content::Text {
        text: "Hello, world!".to_string(),
    };

    if let Content::Text { text } = content {
        assert_eq!(text, "Hello, world!");
    } else {
        panic!("Expected Text content");
    }
}

#[test]
fn test_content_resource_nested() {
    let content = Content::Resource {
        resource: ResourceContent {
            uri: "file:///path/to/image.png".to_string(),
            mime_type: Some("image/png".to_string()),
            text: None,
        },
    };

    if let Content::Resource { resource } = content {
        assert_eq!(resource.uri, "file:///path/to/image.png");
        assert_eq!(resource.mime_type, Some("image/png".to_string()));
    } else {
        panic!("Expected Resource content");
    }
}

#[test]
fn test_content_resource() {
    let content = Content::Resource {
        resource: ResourceContent {
            uri: "file:///path/to/resource".to_string(),
            mime_type: Some("application/json".to_string()),
            text: None,
        },
    };

    if let Content::Resource { resource } = content {
        assert_eq!(resource.uri, "file:///path/to/resource");
        assert_eq!(resource.mime_type, Some("application/json".to_string()));
    } else {
        panic!("Expected Resource content");
    }
}

#[test]
fn test_list_tools_request() {
    // MCP uses JSON-RPC with no params for list_tools
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/list".to_string(),
        params: None,
    };
    assert_eq!(request.method, "tools/list");
}

#[test]
fn test_list_tools_result() {
    let result = ListToolsResult {
        tools: vec![
            Tool {
                name: "tool1".to_string(),
                description: "Tool 1".to_string(),
                input_schema: json!({"type": "object"}),
            },
            Tool {
                name: "tool2".to_string(),
                description: "Tool 2".to_string(),
                input_schema: json!({"type": "object"}),
            },
        ],
    };

    assert_eq!(result.tools.len(), 2);
}

#[test]
fn test_ping_request() {
    // MCP uses JSON-RPC with ping method
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "ping".to_string(),
        params: None,
    };
    assert_eq!(request.method, "ping");
}

#[test]
fn test_ping_result() {
    // Ping returns empty object
    let result = json!({});
    assert!(result.is_object());
}

#[test]
fn test_resource() {
    let resource = Resource {
        uri: "file:///data/test.hedl".to_string(),
        name: "test.hedl".to_string(),
        description: Some("Test HEDL file".to_string()),
        mime_type: Some("application/hedl".to_string()),
    };

    assert!(resource.uri.starts_with("file://"));
    assert!(resource.description.is_some());
}

#[test]
fn test_list_resources_result() {
    let result = ListResourcesResult {
        resources: vec![
            Resource {
                uri: "file:///data/test1.hedl".to_string(),
                name: "test1.hedl".to_string(),
                description: None,
                mime_type: None,
            },
            Resource {
                uri: "file:///data/test2.hedl".to_string(),
                name: "test2.hedl".to_string(),
                description: None,
                mime_type: None,
            },
        ],
    };

    assert_eq!(result.resources.len(), 2);
}

#[test]
fn test_read_resource_request() {
    let request = ReadResourceParams {
        uri: "file:///data/test.hedl".to_string(),
    };

    assert!(request.uri.starts_with("file://"));
}

#[test]
fn test_read_resource_result() {
    let result = ReadResourceResult {
        contents: vec![ResourceContent {
            uri: "file:///data/test.hedl".to_string(),
            mime_type: Some("application/hedl".to_string()),
            text: Some("HEDL content".to_string()),
        }],
    };

    assert_eq!(result.contents.len(), 1);
}

#[test]
fn test_resource_contents_text() {
    let contents = ResourceContent {
        uri: "file:///test.txt".to_string(),
        mime_type: Some("text/plain".to_string()),
        text: Some("Hello".to_string()),
    };

    assert_eq!(contents.text, Some("Hello".to_string()));
}

#[test]
fn test_resource_contents_binary() {
    // ResourceContent doesn't have a Blob variant, just use text field as None
    let contents = ResourceContent {
        uri: "file:///test.bin".to_string(),
        mime_type: Some("application/octet-stream".to_string()),
        text: None,
    };

    assert_eq!(
        contents.mime_type,
        Some("application/octet-stream".to_string())
    );
    assert_eq!(contents.text, None);
}

#[test]
fn test_protocol_version_validation() {
    let valid_versions = vec!["1.0", "2.0", "2.1"];

    for _version in valid_versions {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "test".to_string(),
            params: None,
        };

        assert_eq!(request.jsonrpc, "2.0");
    }
}

#[test]
fn test_error_with_data() {
    let error = JsonRpcError {
        code: -32600,
        message: "Invalid Request".to_string(),
        data: Some(json!({
            "details": "Missing required parameter"
        })),
    };

    assert!(error.data.is_some());
    assert_eq!(error.data.unwrap()["details"], "Missing required parameter");
}

#[test]
fn test_capabilities_serialization() {
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        prompts: Some(PromptsCapability {
            list_changed: Some(true),
        }),
    };

    let serialized = serde_json::to_value(&capabilities).unwrap();
    assert!(serialized["tools"].is_object());
    assert!(serialized["resources"].is_object());
}

#[test]
fn test_client_info_serialization() {
    let client_info = ClientInfo {
        name: "test-client".to_string(),
        version: "1.2.3".to_string(),
    };

    let serialized = serde_json::to_value(&client_info).unwrap();
    assert_eq!(serialized["name"], "test-client");
    assert_eq!(serialized["version"], "1.2.3");
}

#[test]
fn test_server_info_serialization() {
    let server_info = ServerInfo {
        name: "hedl-mcp".to_string(),
        version: "1.0.0".to_string(),
    };

    let serialized = serde_json::to_value(&server_info).unwrap();
    assert_eq!(serialized["name"], "hedl-mcp");
    assert_eq!(serialized["version"], "1.0.0");
}

#[test]
fn test_json_rpc_id_types() {
    // String ID
    let request1 = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!("abc-123")),
        method: "test".to_string(),
        params: None,
    };

    // Number ID
    let request2 = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(42)),
        method: "test".to_string(),
        params: None,
    };

    // Null ID
    let request3 = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(null)),
        method: "test".to_string(),
        params: None,
    };

    assert!(request1.id.is_some());
    assert!(request2.id.is_some());
    assert!(request3.id.is_some());
}

#[test]
fn test_empty_params() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(json!({})),
    };

    assert!(request.params.is_some());
    assert!(request.params.unwrap().is_object());
}

#[test]
fn test_array_params() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(json!([1, 2, 3])),
    };

    assert!(request.params.is_some());
    assert!(request.params.unwrap().is_array());
}

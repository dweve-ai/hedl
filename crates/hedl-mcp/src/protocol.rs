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

//! MCP Protocol types and JSON-RPC message handling.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version string, must be "2.0".
    pub jsonrpc: String,
    /// Request identifier (string, number, or null).
    pub id: Option<Value>,
    /// Method name to invoke.
    pub method: String,
    /// Optional method parameters.
    #[serde(default)]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Validate JSON-RPC 2.0 protocol compliance.
    ///
    /// # Validation Rules
    ///
    /// 1. JSON-RPC version must be exactly "2.0"
    /// 2. Request ID must be string, number, or null (if present)
    /// 3. Method name must be non-empty and contain no whitespace
    /// 4. Params must be object or array (if present)
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the request is valid
    /// - `Err(String)` with a descriptive error message if validation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_mcp::JsonRpcRequest;
    /// use serde_json::json;
    ///
    /// let request = JsonRpcRequest {
    ///     jsonrpc: "2.0".to_string(),
    ///     id: Some(json!(1)),
    ///     method: "initialize".to_string(),
    ///     params: Some(json!({"key": "value"})),
    /// };
    ///
    /// assert!(request.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        // Validate jsonrpc version
        if self.jsonrpc != "2.0" {
            return Err(format!(
                "Invalid JSON-RPC version: expected '2.0', got '{}'",
                self.jsonrpc
            ));
        }

        // Validate id type (must be string, number, or null)
        if let Some(id) = &self.id {
            match id {
                Value::String(_) | Value::Number(_) | Value::Null => {}
                _ => return Err("Invalid request id: must be string, number, or null".to_string()),
            }
        }

        // Validate method name
        if self.method.is_empty() {
            return Err("Method name cannot be empty".to_string());
        }
        if self.method.contains(char::is_whitespace) {
            return Err("Method name cannot contain whitespace".to_string());
        }

        // Validate params type (must be object or array if present)
        if let Some(params) = &self.params {
            if !params.is_object() && !params.is_array() {
                return Err("Params must be an object or array".to_string());
            }
        }

        Ok(())
    }
}

/// JSON-RPC response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version string, always "2.0".
    pub jsonrpc: String,
    /// Request identifier this response corresponds to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    /// Result value on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error object on failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a success response.
    #[must_use]
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    #[must_use]
    pub fn error(id: Option<Value>, code: i32, message: String, data: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data,
            }),
        }
    }
}

/// JSON-RPC error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code indicating the error type.
    pub code: i32,
    /// Human-readable error message.
    pub message: String,
    /// Additional error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// --- MCP Protocol Types ---

/// Server capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Tools capability configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    /// Resources capability configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    /// Prompts capability configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
}

/// Tools capability configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    /// Whether to notify clients when the tools list changes.
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Resources capability configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    /// Whether clients can subscribe to resource updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    /// Whether to notify clients when the resources list changes.
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Prompts capability configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    /// Whether to notify clients when the prompts list changes.
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Initialize request params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// MCP protocol version.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Client capability declarations.
    pub capabilities: ClientCapabilities,
    /// Client identification information.
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Client capabilities.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientCapabilities {
    /// Root URI capability configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootsCapability>,
    /// Sampling capability configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<Value>,
}

/// Root URI capability configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootsCapability {
    /// Whether to notify clients when the roots list changes.
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Client info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client application name.
    pub name: String,
    /// Client application version.
    pub version: String,
}

/// Initialize result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// MCP protocol version supported by server.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capability declarations.
    pub capabilities: ServerCapabilities,
    /// Server identification information.
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// Server info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server application name.
    pub name: String,
    /// Server application version.
    pub version: String,
}

/// Tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name identifier.
    pub name: String,
    /// Human-readable tool description.
    pub description: String,
    /// JSON Schema for tool input parameters.
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Tool call request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    /// Name of the tool to call.
    pub name: String,
    /// Arguments to pass to the tool.
    #[serde(default)]
    pub arguments: Option<Value>,
}

/// Tool call result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    /// Content items returned by the tool.
    pub content: Vec<Content>,
    /// Whether the tool call resulted in an error.
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Content item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    /// Text content.
    #[serde(rename = "text")]
    Text {
        /// The text content.
        text: String,
    },
    /// Resource content.
    #[serde(rename = "resource")]
    Resource {
        /// The resource content.
        resource: ResourceContent,
    },
}

/// Resource content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    /// Resource URI identifier.
    pub uri: String,
    /// MIME type of the resource content.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Text content of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Resource definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource URI identifier.
    pub uri: String,
    /// Human-readable resource name.
    pub name: String,
    /// Optional resource description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the resource content.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// List tools result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    /// Available tools.
    pub tools: Vec<Tool>,
}

/// List resources result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResult {
    /// Available resources.
    pub resources: Vec<Resource>,
}

/// Read resource params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceParams {
    /// URI of the resource to read.
    pub uri: String,
}

/// Read resource result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult {
    /// Resource content items.
    pub contents: Vec<ResourceContent>,
}

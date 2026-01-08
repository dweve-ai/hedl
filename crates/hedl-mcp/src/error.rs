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

//! Error types for the MCP server.

use thiserror::Error;

/// MCP server error type.
#[derive(Error, Debug)]
pub enum McpError {
    /// HEDL parsing error.
    #[error("HEDL parse error: {0}")]
    Parse(#[from] hedl_core::HedlError),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid request.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Tool not found.
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Resource not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Invalid arguments.
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// Path traversal attempt.
    #[error("Path traversal not allowed: {0}")]
    PathTraversal(String),

    /// File not found.
    #[error("File not found: {0}")]
    FileNotFound(String),
}

/// Result type for MCP operations.
pub type McpResult<T> = Result<T, McpError>;

impl McpError {
    /// Get the MCP error code.
    pub fn code(&self) -> i32 {
        match self {
            Self::Parse(_) => -32001,
            Self::Json(_) => -32700,
            Self::Io(_) => -32002,
            Self::InvalidRequest(_) => -32600,
            Self::ToolNotFound(_) => -32601,
            Self::ResourceNotFound(_) => -32602,
            Self::InvalidArguments(_) => -32602,
            Self::PathTraversal(_) => -32003,
            Self::FileNotFound(_) => -32004,
        }
    }
}

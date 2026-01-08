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

//! HEDL Model Context Protocol (MCP) Server
//!
//! This crate provides an MCP server that allows AI/LLM systems to interact
//! with HEDL files and the HEDL ecosystem. Key features:
//!
//! - **Read HEDL files** from a directory with graph-aware lookups
//! - **Query entities** by type and ID via the reference registry
//! - **Validate HEDL** input with detailed error reporting
//! - **Optimize JSON to HEDL** for token-efficient context injection
//! - **Get token statistics** comparing HEDL vs JSON representations

pub mod cache;
mod error;
mod protocol;
mod rate_limiter;
mod server;
pub mod tools;

pub use cache::{CacheStats, OperationCache};
pub use error::{McpError, McpResult};
pub use protocol::*;
pub use rate_limiter::RateLimiter;
pub use server::{McpServer, McpServerConfig};
pub use tools::{execute_tool, get_tools};

/// MCP Server version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Server name for MCP protocol
pub const SERVER_NAME: &str = "hedl-mcp";

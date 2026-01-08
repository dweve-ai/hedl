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

//! HEDL MCP Server binary.
//!
//! This server implements the Model Context Protocol (MCP) for AI/LLM integration.
//!
//! # Usage
//!
//! ```bash
//! # Run with default settings (current directory as root)
//! hedl-mcp
//!
//! # Run with a specific root directory
//! hedl-mcp --root /path/to/hedl/files
//!
//! # Run with debug logging
//! RUST_LOG=debug hedl-mcp
//! ```
//!
//! # Available Tools
//!
//! - `hedl_read`: Read and parse HEDL files from a directory
//! - `hedl_query`: Query the node registry for entities by type/ID
//! - `hedl_validate`: Validate HEDL input with detailed diagnostics
//! - `hedl_optimize`: Convert JSON to optimized HEDL format
//! - `hedl_stats`: Get token usage statistics (HEDL vs JSON)
//! - `hedl_to_json`: Convert HEDL to JSON format
//! - `hedl_format`: Format HEDL to canonical form

use clap::Parser;
use hedl_mcp::{McpServer, McpServerConfig};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "hedl-mcp")]
#[command(author = "Dweve B.V.")]
#[command(version)]
#[command(about = "HEDL Model Context Protocol (MCP) Server for AI/LLM integration")]
struct Cli {
    /// Root directory for file operations
    #[arg(short, long, default_value = ".")]
    root: PathBuf,

    /// Use async runtime (recommended for production)
    #[arg(long, default_value = "true")]
    r#async: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("hedl_mcp=info".parse().expect("valid log directive")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    let config = McpServerConfig {
        root_path: cli.root.canonicalize().unwrap_or(cli.root),
        ..Default::default()
    };

    let mut server = McpServer::new(config);

    if cli.r#async {
        // Run async server
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(async { server.run_stdio_async().await })?;
    } else {
        // Run sync server
        server.run_stdio()?;
    }

    Ok(())
}

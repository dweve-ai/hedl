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

//! HEDL Language Server binary.
//!
//! Provides IDE integration for HEDL through the Language Server Protocol.
//!
//! # Usage
//!
//! ```bash
//! # Run the language server (stdio transport)
//! hedl-lsp
//!
//! # With debug logging
//! RUST_LOG=debug hedl-lsp
//! ```
//!
//! # Features
//!
//! - **Diagnostics**: Real-time error and warning reporting
//! - **Autocomplete**: Context-aware completion for IDs, types, and references
//! - **Hover**: Documentation and type information on hover
//! - **Go to Definition**: Navigate to entity and type definitions
//! - **Find References**: Find all uses of an entity
//! - **Document Symbols**: Outline view with entities and schemas
//! - **Formatting**: Canonical formatting with ditto optimization
//!
//! # Editor Integration
//!
//! ## VS Code
//!
//! Add to `settings.json`:
//! ```json
//! {
//!   "hedl.server.path": "/path/to/hedl-lsp"
//! }
//! ```
//!
//! ## Neovim (nvim-lspconfig)
//!
//! ```lua
//! require('lspconfig.configs').hedl = {
//!   default_config = {
//!     cmd = { 'hedl-lsp' },
//!     filetypes = { 'hedl' },
//!     root_dir = function() return vim.fn.getcwd() end,
//!   },
//! }
//! require('lspconfig').hedl.setup {}
//! ```

use hedl_lsp::HedlLanguageServer;
use tower_lsp::{LspService, Server};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Initialize logging to stderr
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("hedl_lsp=info".parse().expect("valid log directive"))
                .add_directive("tower_lsp=info".parse().expect("valid log directive")),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting HEDL Language Server v{}", hedl_lsp::VERSION);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(HedlLanguageServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}

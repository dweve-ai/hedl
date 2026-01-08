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

//! HEDL Language Server Protocol (LSP) Implementation
//!
//! This crate provides IDE integration for HEDL through the Language Server Protocol,
//! enabling rich editing experiences in LSP-compatible editors like VS Code, Neovim,
//! Emacs, and others.
//!
//! # Features
//!
//! - **Diagnostics**: Real-time error and warning reporting with syntax and lint checks
//! - **Autocomplete**: Context-aware completion for IDs, types, references, and directives
//! - **Hover**: Documentation and type information on hover with entity validation
//! - **Go to Definition**: Navigate to entity and type definitions across the document
//! - **Find References**: Find all usages of entities and types
//! - **Document Symbols**: Hierarchical outline view with entities and schemas
//! - **Workspace Symbols**: Search symbols across all open documents
//! - **Semantic Highlighting**: Type-aware syntax highlighting for better readability
//! - **Document Formatting**: Canonical HEDL formatting with ditto optimization
//!
//! # Performance
//!
//! The LSP implementation includes four key performance optimizations:
//!
//! 1. **Debouncing** (200ms): Batches multiple keystrokes together, reducing parse
//!    operations by ~90% during typing.
//! 2. **Dirty Tracking**: Content hash-based change detection prevents redundant
//!    parsing when document content hasn't changed.
//! 3. **Caching**: Parsed documents are cached and reused for LSP queries without
//!    blocking the UI.
//! 4. **Reference Index**: O(1) hash map lookups for definitions and references,
//!    replacing previous O(n) linear search bottleneck.
//!
//! # Memory Management
//!
//! The implementation includes multiple safeguards for memory management:
//!
//! - **Document Size Limit**: Maximum 500MB per document (configurable) to prevent memory exhaustion
//! - **Open Document Limit**: Maximum 1000 simultaneously open documents with LRU eviction
//! - **UTF-8 Safety**: All string slicing operations are UTF-8 boundary aware
//! - **Input Validation**: Comprehensive bounds checking on all LSP positions
//!
//! # Usage
//!
//! ## Running the Server
//!
//! ```bash
//! # Run the language server (stdio transport)
//! hedl-lsp
//!
//! # With debug logging
//! RUST_LOG=debug hedl-lsp
//!
//! # With trace-level logging for maximum detail
//! RUST_LOG=trace hedl-lsp
//! ```
//!
//! ## Programmatic Usage
//!
//! ### Default Configuration (500 MB documents, 1000 document cache)
//!
//! ```no_run
//! use hedl_lsp::HedlLanguageServer;
//! use tower_lsp::{LspService, Server};
//!
//! #[tokio::main]
//! async fn main() {
//!     let stdin = tokio::io::stdin();
//!     let stdout = tokio::io::stdout();
//!
//!     let (service, socket) = LspService::new(|client| {
//!         HedlLanguageServer::new(client)
//!     });
//!
//!     Server::new(stdin, stdout, socket).serve(service).await;
//! }
//! ```
//!
//! ### Custom Configuration (e.g., 1 GB documents, 2000 document cache)
//!
//! ```no_run
//! use hedl_lsp::HedlLanguageServer;
//! use tower_lsp::{LspService, Server};
//!
//! #[tokio::main]
//! async fn main() {
//!     let stdin = tokio::io::stdin();
//!     let stdout = tokio::io::stdout();
//!
//!     let (service, socket) = LspService::new(|client| {
//!         HedlLanguageServer::with_config(
//!             client,
//!             2000,                    // max documents
//!             1024 * 1024 * 1024       // 1 GB max document size
//!         )
//!     });
//!
//!     Server::new(stdin, stdout, socket).serve(service).await;
//! }
//! ```
//!
//! # Architecture
//!
//! The crate is organized into several modules:
//!
//! - `backend`: LSP server implementation with document management
//! - [`analysis`]: Document parsing and analysis with entity/reference extraction
//! - [`completion`]: Context-aware autocompletion logic
//! - [`hover`]: Hover information provider with type and entity details
//! - [`reference_index`]: O(1) reference index for fast definition and reference lookups
//! - [`symbols`]: Document and workspace symbol providers
//! - [`utils`]: Safe string handling utilities for UTF-8 safety

pub mod analysis;
mod backend;
pub mod completion;
pub mod constants;
pub mod document_manager;
pub mod hover;
pub mod reference_index;
pub mod symbols;
pub mod utils;

#[cfg(test)]
mod tests;

pub use backend::HedlLanguageServer;
pub use document_manager::{CacheStatistics, DocumentManager};

/// LSP server version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

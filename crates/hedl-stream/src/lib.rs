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

//! Streaming HEDL Parser
//!
//! This crate provides a streaming, memory-efficient parser for HEDL documents.
//! Instead of loading the entire document into memory, it yields events or nodes
//! one at a time, making it suitable for processing multi-GB files.
//!
//! # Features
//!
//! - **Memory Efficient**: Process files larger than available RAM
//! - **Iterator-based**: Standard Rust iterator interface (sync)
//! - **Async Support**: Non-blocking I/O with tokio (optional)
//! - **Event-driven**: Optional SAX-like event callbacks
//! - **Timeout Protection**: Prevent infinite loops from malicious/untrusted input
//! - **Compatible**: Works with `hedl-parquet` and `hedl-neo4j` for streaming export
//!
//! # Sync vs Async
//!
//! ## Synchronous API (default)
//!
//! Use the synchronous API for:
//! - Processing local files
//! - Single-threaded batch processing
//! - Simpler code without async complexity
//! - CPU-bound workloads with minimal I/O wait
//!
//! ```rust,no_run
//! use hedl_stream::{StreamingParser, NodeEvent};
//! use std::io::BufReader;
//! use std::fs::File;
//!
//! let file = File::open("large-dataset.hedl").unwrap();
//! let reader = BufReader::new(file);
//!
//! let parser = StreamingParser::new(reader).unwrap();
//!
//! for event in parser {
//!     match event {
//!         Ok(NodeEvent::Node(node)) => {
//!             println!("{}:{}", node.type_name, node.id);
//!         }
//!         Ok(NodeEvent::ListStart { type_name, .. }) => {
//!             println!("List started: {}", type_name);
//!         }
//!         Err(e) => {
//!             eprintln!("Error: {}", e);
//!             break;
//!         }
//!         _ => {}
//!     }
//! }
//! ```
//!
//! ## Asynchronous API (feature = "async")
//!
//! Use the asynchronous API for:
//! - Processing network streams or pipes
//! - High-concurrency scenarios (many parallel streams)
//! - Integration with async web servers or frameworks
//! - Non-blocking I/O in async runtime contexts
//!
//! ```rust,no_run
//! # #[cfg(feature = "async")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use hedl_stream::{AsyncStreamingParser, NodeEvent};
//! use tokio::fs::File;
//!
//! let file = File::open("large-dataset.hedl").await?;
//! let parser = AsyncStreamingParser::new(file).await?;
//!
//! while let Some(event) = parser.next_event().await? {
//!     match event {
//!         NodeEvent::Node(node) => {
//!             println!("{}:{}", node.type_name, node.id);
//!         }
//!         NodeEvent::ListStart { type_name, .. } => {
//!             println!("List started: {}", type_name);
//!         }
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Timeout Protection for Untrusted Input
//!
//! When parsing untrusted input, configure a timeout to prevent infinite loops:
//!
//! ```rust,no_run
//! use hedl_stream::{StreamingParser, StreamingParserConfig};
//! use std::time::Duration;
//! use std::io::Cursor;
//!
//! let config = StreamingParserConfig {
//!     timeout: Some(Duration::from_secs(10)),
//!     ..Default::default()
//! };
//!
//! let untrusted_input = "..."; // Input from untrusted source
//! let parser = StreamingParser::with_config(
//!     Cursor::new(untrusted_input),
//!     config
//! ).unwrap();
//!
//! // Parser will return StreamError::Timeout if parsing exceeds 10 seconds
//! for event in parser {
//!     // Process events...
//!     # break;
//! }
//! ```

mod error;
mod event;
mod parser;
mod reader;

#[cfg(feature = "async")]
mod async_parser;
#[cfg(feature = "async")]
mod async_reader;

pub use error::{StreamError, StreamResult};
pub use event::{HeaderInfo, NodeEvent, NodeInfo};
pub use parser::{StreamingParser, StreamingParserConfig};
pub use reader::LineReader;

#[cfg(feature = "async")]
pub use async_parser::AsyncStreamingParser;
#[cfg(feature = "async")]
pub use async_reader::AsyncLineReader;

/// Re-export core types for convenience.
pub use hedl_core::{Reference, Value};

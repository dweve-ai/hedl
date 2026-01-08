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

//! HEDL CLI library for command-line parsing and execution.
//!
//! This library provides the core functionality for the HEDL command-line interface,
//! including all command implementations for validation, formatting, linting, and
//! format conversion operations.
//!
//! # Commands
//!
//! The CLI provides the following commands:
//!
//! ## Validation & Inspection
//!
//! - **validate**: Validate HEDL file syntax and structure
//! - **inspect**: Visualize HEDL internal structure with tree view
//!
//! ## Formatting & Canonicalization
//!
//! - **format**: Format HEDL files to canonical form
//! - **lint**: Lint HEDL files for best practices and style
//!
//! ## Format Conversion
//!
//! Bidirectional conversion between HEDL and popular data formats:
//!
//! - **to-json/from-json**: JSON conversion (compact and pretty)
//! - **to-yaml/from-yaml**: YAML conversion
//! - **to-xml/from-xml**: XML conversion (compact and pretty)
//! - **to-csv/from-csv**: CSV conversion (tabular data)
//! - **to-parquet/from-parquet**: Apache Parquet conversion (columnar)
//!
//! ## Analysis & Statistics
//!
//! - **stats**: Compare size and token efficiency vs other formats
//!
//! ## Utilities
//!
//! - **completion**: Generate shell completion scripts (bash, zsh, fish, powershell, elvish)
//!
//! ## Batch Processing
//!
//! - **batch-validate**: Validate multiple files in parallel
//! - **batch-format**: Format multiple files in parallel
//! - **batch-lint**: Lint multiple files in parallel
//!
//! # Examples
//!
//! ## Validation
//!
//! ```no_run
//! use hedl_cli::commands::validate;
//!
//! # fn main() -> Result<(), String> {
//! // Validate a HEDL file
//! validate("example.hedl", false)?;
//!
//! // Strict validation (all references must resolve)
//! validate("example.hedl", true)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Format Conversion
//!
//! ```no_run
//! use hedl_cli::commands::{to_json, from_json};
//!
//! # fn main() -> Result<(), String> {
//! // Convert HEDL to pretty JSON
//! to_json("data.hedl", Some("output.json"), false, true)?;
//!
//! // Convert JSON back to HEDL
//! from_json("data.json", Some("output.hedl"))?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Batch Processing
//!
//! ```no_run
//! use hedl_cli::commands::batch_validate;
//!
//! # fn main() -> Result<(), String> {
//! // Validate multiple files in parallel
//! let files = vec!["file1.hedl".to_string(), "file2.hedl".to_string()];
//! batch_validate(files, false, true, false)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Security
//!
//! The CLI includes several security features:
//!
//! - **File size limits**: Prevents OOM attacks (configurable via `HEDL_MAX_FILE_SIZE`)
//! - **Input validation**: Type names and parameters are validated
//! - **Safe conversions**: All format conversions use safe, well-tested libraries
//!
//! # Performance
//!
//! - **Parallel processing**: Batch commands automatically use parallel processing
//! - **Optimized stats**: Format conversions run in parallel (3-5x speedup)
//! - **Memory efficiency**: Streaming where possible, size checks before reading
//!
//! # Error Handling
//!
//! All commands return `Result<(), String>` for consistent error handling.
//! Errors are descriptive and include context like file paths and line numbers
//! where applicable.

pub mod batch;
pub mod cli;
pub mod commands;
pub mod error;

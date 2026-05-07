// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! HEDL Filter - Command output filtering and compression for LLM context windows.
//!
//! Provides intelligent filtering of command-line tool outputs, reducing token
//! consumption by 60-90% while preserving semantic completeness.
//!
//! # HEDL Advantage
//!
//! Unlike other tools that simply truncate text, HEDL Filter can convert
//! structured command output into HEDL format, achieving even greater token
//! savings through schema-once encoding.
//!
//! # Two-Tier Architecture
//!
//! 1. **Native Rust Filters**: Semantic parsers for common commands with
//!    structured HEDL output generation.
//! 2. **TOML Declarative Filters**: 8-stage pipeline for extensibility.

pub mod commands;
pub mod config;
pub mod engine;
pub mod hedl;
pub mod hooks;
pub mod output;
pub mod registry;
pub mod source;
pub mod utils;

pub use engine::{run_command, FilterConfig, FilterResult, ExecutionMode};
pub use hedl::{apply_toml_filter, find_matching_filter, CompiledFilter};
pub use output::{to_hedl, StructuredData};
pub use registry::{classify_command, rewrite_command, Classification};
pub use source::{FilterLevel, Language, filter_source, smart_truncate};

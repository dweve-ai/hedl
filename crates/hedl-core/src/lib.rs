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

//! Core parser and data model for HEDL format.
//!
//! This crate provides the main parsing functionality for HEDL documents,
//! including header directives, body parsing, and reference resolution.
//!
//! # Lexical Analysis
//!
//! The [`lex`] module provides the complete lexical analysis infrastructure,
//! consolidating functionality from `hedl-lex`, `hedl-row`, and `hedl-tensor`:
//!
//! - Token validation (key tokens, type names, ID tokens, references)
//! - CSV/matrix row parsing with expression and tensor support
//! - Multi-dimensional tensor literal parsing
//! - Incremental parsing for IDE integration
//! - Source position and span tracking for error reporting
//!
//! See [`lex`] module documentation for more details and examples.

mod block_string;
pub mod convert;
mod document;
mod error;
pub mod errors;
mod header;
mod inference;
pub mod lex;
mod limits;
mod parser;
mod preprocess;
mod reference;
pub mod traverse;
mod value;

pub use document::{Document, Item, MatrixList, Node};
pub use error::{HedlError, HedlErrorKind, HedlResult};
pub use limits::Limits;
pub use parser::{parse, parse_with_limits, ParseOptions, ParseOptionsBuilder};
pub use traverse::{traverse, DocumentVisitor, StatsCollector, VisitorContext};
pub use value::{Reference, Value};

// Re-export useful types from the consolidated lex module
pub use lex::{ExprLiteral, Expression, Reference as ReferenceToken, Tensor};

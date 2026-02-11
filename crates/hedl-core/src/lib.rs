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

#![cfg_attr(not(test), warn(missing_docs))]
mod block_string;
/// Value coercion for HEDL type system.
pub mod coercion;
/// Common conversion utilities and traits for HEDL format converters.
pub mod convert;
mod document;
mod error;
/// Error handling utilities and centralized error messages.
pub mod errors;
/// Header directive parsing for HEDL.
pub mod header;
/// Value inference ladder for HEDL.
pub mod inference;

// Re-export header types for convenience
pub use header::{CountValue, ParseMode};
/// Lexical analysis subsystem for HEDL.
pub mod lex;
mod limits;
/// Parallel parsing infrastructure for hedl-core.
#[cfg(feature = "parallel")]
pub mod parallel;
mod parser;
/// Input preprocessing for HEDL parsing.
pub mod preprocess;
/// Reference resolution for HEDL.
pub mod reference;
/// Schema versioning for HEDL types.
pub mod schema_version;
/// Document traversal trait for format converters.
pub mod traverse;
/// Type system for bidirectional type checking in HEDL.
pub mod types;
/// Comprehensive validation framework for HEDL documents.
pub mod validation;
mod value;
/// Comprehensive visitor pattern API for HEDL document traversal.
pub mod visitor;

pub use coercion::{
    coerce, coerce_with_config, CoercionConfig, CoercionLevel, CoercionMode, CoercionResult,
};
pub use document::{Document, Item, MatrixList, Node};
pub use error::{HedlError, HedlErrorKind, HedlResult};
pub use inference::{
    infer_value_synthesize, infer_value_with_type, InferenceConfidence, InferenceContext,
    InferenceResult,
};
pub use limits::Limits;
pub use parser::{parse, parse_with_limits, ParseOptions, ParseOptionsBuilder};
pub use preprocess::preprocess;
pub use reference::{resolve_references, resolve_references_with_limits, ReferenceMode};
pub use schema_version::{FieldDef, Schema, SchemaVersion};
pub use traverse::{traverse, DocumentVisitor, StatsCollector, VisitorContext};

// Re-export visitor types for public API
pub use types::{value_to_expected_type, ExpectedType, TensorDtype};
pub use value::{Reference, Value};
pub use visitor::{
    transform, traverse as visitor_traverse, traverse_fallible, traverse_mut, DepthCounter,
    FallibleVisitor, NodeCollector, PathCollector, PathSegment, ReferenceCollector, Transformer,
    TraversalConfig, TraversalMode, TraversalOrder, TraversalResult, TraversalStats, VisitDecision,
    Visitor, VisitorMut,
};

// Re-export useful types from the consolidated lex module
pub use lex::{ExprLiteral, Expression, Reference as ReferenceToken, Tensor};

// Re-export parallel types when feature is enabled
#[cfg(feature = "parallel")]
pub use parallel::{
    collect_ids_parallel, identify_entity_boundaries, parse_matrix_rows_parallel,
    validate_references_parallel, AtomicSecurityCounters, EntityBoundary, EntityType,
    MatrixRowBatch, ParallelConfig,
};

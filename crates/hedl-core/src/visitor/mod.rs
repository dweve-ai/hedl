// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Comprehensive visitor pattern API for HEDL document traversal.
//!
//! This module provides a flexible, performance-oriented visitor API that supports:
//! - Multiple visitor styles (immutable, mutable, consuming, fallible)
//! - Fine-grained control flow (continue, skip children, early termination)
//! - Pre-order and post-order traversal strategies
//! - Specialized utility visitors for common patterns
//! - Visitor composition and chaining
//!
//! # Architecture
//!
//! The visitor pattern separates traversal logic from processing logic,
//! enabling reusable document analysis and transformation components.
//!
//! ## Visitor Traits
//!
//! - [`Visitor`]: Immutable visiting for read-only analysis
//! - [`VisitorMut`]: Mutable visiting for in-place modification
//! - [`Transformer`]: Consuming visitor that rebuilds trees
//! - [`FallibleVisitor`]: Result-based visiting with error propagation
//!
//! ## Control Flow
//!
//! All visitor methods return [`VisitDecision`] to control traversal:
//! - `Continue`: Visit this node and its children
//! - `SkipChildren`: Visit this node but skip children
//! - `Stop`: Terminate traversal immediately
//!
//! # Example: Simple Node Counter
//!
//! ```
//! use hedl_core::visitor::{Visitor, VisitDecision, VisitorContext};
//! use hedl_core::Node;
//!
//! struct NodeCounter {
//!     count: usize,
//! }
//!
//! impl Visitor for NodeCounter {
//!     fn visit_node(&mut self, _node: &Node, _ctx: &VisitorContext) -> VisitDecision {
//!         self.count += 1;
//!         VisitDecision::Continue
//!     }
//! }
//! ```
//!
//! # Example: Early Termination Search
//!
//! ```
//! use hedl_core::visitor::{Visitor, VisitDecision, VisitorContext};
//! use hedl_core::Node;
//!
//! struct FindNode {
//!     target_id: String,
//!     found: bool,
//! }
//!
//! impl Visitor for FindNode {
//!     fn visit_node(&mut self, node: &Node, _ctx: &VisitorContext) -> VisitDecision {
//!         if node.id == self.target_id {
//!             self.found = true;
//!             VisitDecision::Stop  // Early termination
//!         } else {
//!             VisitDecision::Continue
//!         }
//!     }
//! }
//! ```

mod config;
mod context;
mod decision;
mod fallible;
mod transformer;
mod traverse;
pub mod utils;
#[allow(clippy::module_inception)]
mod visitor;
mod visitor_mut;

pub use config::{TraversalConfig, TraversalMode, TraversalOrder};
pub use context::{PathSegment, TraversalStats, VisitorContext};
pub use decision::VisitDecision;
pub use fallible::FallibleVisitor;
pub use transformer::Transformer;
pub use traverse::{transform, traverse, traverse_fallible, traverse_mut, TraversalResult};
pub use utils::{
    DepthCounter, FindNode, NodeCollector, PathCollector, ReferenceCollector, TypeCounter,
};
pub use visitor::Visitor;
pub use visitor_mut::VisitorMut;

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

//! Canonical test fixtures covering all HEDL features.
//!
//! This module provides a comprehensive set of fixtures organized by category:
//!
//! - **values**: Scalar values, strings, references, expressions
//! - **expressions**: Expression-based fixtures
//! - **matrices**: Matrix lists with various configurations
//! - **documents**: Complex multi-entity documents
//! - **errors**: Invalid documents and error test cases
//! - **builders**: Builder pattern for customizable fixtures

pub mod builders;
mod documents;
pub mod errors;
mod expressions;
mod matrices;
mod values;

pub use documents::*;
pub use expressions::*;
pub use matrices::*;
pub use values::*;

use crate::FixtureList;

/// Returns all fixture functions for iteration.
///
/// Useful for running the same test across all fixtures.
pub fn all() -> FixtureList {
    vec![
        ("scalars", scalars),
        ("special_strings", special_strings),
        ("references", references),
        ("expressions", expressions),
        ("tensors", tensors),
        ("named_values", named_values),
        ("user_list", user_list),
        ("mixed_type_list", mixed_type_list),
        ("with_references", with_references),
        ("with_nest", with_nest),
        ("deep_nest", deep_nest),
        ("edge_cases", edge_cases),
        ("comprehensive", comprehensive),
        ("blog", blog),
        ("empty", empty),
    ]
}

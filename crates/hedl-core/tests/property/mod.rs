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

//! Property-based tests for HEDL core parsing.
//!
//! These tests use proptest to validate invariants across a wide range of inputs,
//! catching edge cases that example-based tests might miss.
//!
//! # Test Modules
//!
//! ## Core Functionality (Phase 1-6)
//! - `value_inference`: Tests for value type inference determinism
//! - `references`: Tests for reference resolution consistency
//! - `ditto`: Tests for ditto marker expansion correctness
//! - `roundtrip`: Tests for parse/serialize preservation
//! - `errors`: Tests for error handling consistency
//! - `boundaries`: Tests for boundary conditions and limits
//! - `nest`: Tests for NEST hierarchy semantics
//! - `block_strings`: Tests for block string handling
//! - `expressions`: Tests for expression and reference handling
//!
//! Each module runs 1000+ test cases per property to ensure comprehensive coverage.
//!
//! # Total Coverage
//!
//! - **100+ individual property tests** across all modules
//! - **100,000+ test cases** generated per full test run
//! - **Comprehensive invariant validation** for all core features

/// Property-based tests for block string handling.
pub mod block_strings;
/// Property-based tests for boundary conditions and limits.
pub mod boundaries;
/// Property-based tests for ditto marker expansion.
pub mod ditto;
/// Property-based tests for error handling consistency.
pub mod errors;
/// Property-based tests for expression and reference handling.
pub mod expressions;
/// Property-based tests for NEST hierarchy semantics.
pub mod nest;
/// Property-based tests for reference resolution.
pub mod references;
/// Property-based tests for parse/serialize roundtrip preservation.
pub mod roundtrip;
/// Property-based tests for value type inference.
pub mod value_inference;

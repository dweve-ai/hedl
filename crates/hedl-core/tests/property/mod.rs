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
//! - `value_inference`: Tests for value type inference determinism
//! - `references`: Tests for reference resolution consistency
//! - `roundtrip`: Tests for parse → canonicalize → parse roundtrip properties
//! - `ditto`: Tests for ditto marker expansion correctness
//!
//! Each module runs 1000+ test cases per property to ensure comprehensive coverage.

pub mod ditto;
pub mod references;
pub mod roundtrip;
pub mod value_inference;

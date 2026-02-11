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

//! Helper utilities module for HEDL benchmarks.
//!
//! Provides DRY-compliant helper functions organized by function:
//!
//! - **parsing**: HEDL parsing utilities with timing
//! - **conversion**: Format conversion helpers
//! - **streaming**: Streaming parser utilities
//! - **validation**: Document validation helpers
//! - **metrics**: Throughput and size comparison utilities

/// Format conversion helpers.
pub mod conversion;
/// Metrics and measurement utilities for benchmarks.
pub mod metrics;
/// Parsing helpers for benchmarks.
pub mod parsing;
// Note: streaming module disabled - requires refactoring to handle
// iterator lifetime issues with StreamingParser. The streaming parser
// itself works correctly in hedl-stream, but wrapping it in helper
// functions introduces complex lifetime constraints. Use StreamingParser
// directly from hedl-stream instead.
/// Validation helpers for benchmarks.
pub mod validation;

// Re-export commonly used functions
pub use conversion::{
    convert_from_json, convert_from_yaml, convert_to_canonical, convert_to_json, convert_to_xml,
    convert_to_yaml, roundtrip_test, Format,
};
pub use metrics::{compare_sizes, measure_throughput_ns, SizeComparison};
pub use parsing::{parse_batch, parse_safe, parse_unchecked, parse_with_timing};
pub use validation::{
    is_valid_hedl, validate_json_roundtrip, validate_roundtrip, validate_strict,
    validate_yaml_roundtrip,
};

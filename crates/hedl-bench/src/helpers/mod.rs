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

pub mod conversion;
pub mod parsing;
// pub mod streaming;  // TODO: Fix iterator type mismatches
pub mod validation;

// Re-export commonly used functions
pub use conversion::{
    convert_from_json, convert_from_yaml, convert_to_canonical, convert_to_json, convert_to_xml,
    convert_to_yaml, roundtrip_test, Format,
};
pub use parsing::{parse_batch, parse_safe, parse_unchecked, parse_with_timing};
// pub use streaming::{
//     create_stream_parser, count_node_events, count_all_events,
//     validate_stream_parse,
// };
pub use validation::{
    is_valid_hedl, validate_json_roundtrip, validate_roundtrip, validate_strict,
    validate_yaml_roundtrip,
};

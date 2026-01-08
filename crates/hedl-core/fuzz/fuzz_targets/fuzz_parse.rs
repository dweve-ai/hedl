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


#![no_main]

//! Fuzz target for HEDL core parser.
//!
//! This fuzzer tests the parser for crashes, panics, and memory safety issues
//! with arbitrary input data. It exercises the full parsing pipeline including:
//!
//! - Preprocessing and line splitting
//! - Header parsing (VERSION, TYPE, ALIAS, NEST)
//! - Body parsing (objects, lists, matrix rows)
//! - Reference resolution
//! - Error handling
//!
//! # Security Testing
//!
//! The fuzzer specifically targets security-critical paths:
//!
//! - Input validation and UTF-8 handling
//! - Limit enforcement (max_file_size, max_line_length)
//! - Memory allocation patterns
//! - Integer overflow/underflow in counters
//! - Recursive parsing structures
//!
//! # Running the Fuzzer
//!
//! ```bash
//! # Install cargo-fuzz if not already installed
//! cargo install cargo-fuzz
//!
//! # Run the fuzzer (from hedl-core directory)
//! cargo fuzz run fuzz_parse
//!
//! # Run with specific options
//! cargo fuzz run fuzz_parse -- -max_len=100000 -max_total_time=300
//!
//! # Run on multiple cores
//! cargo fuzz run fuzz_parse -- -jobs=8
//!
//! # Run with AddressSanitizer
//! cargo fuzz run fuzz_parse --sanitizer=address
//! ```
//!
//! # Expected Behavior
//!
//! - Parser should never panic (except documented panics in unreachable code)
//! - All errors should be Result::Err, never unwraps
//! - Memory usage should remain bounded
//! - Invalid UTF-8 should be handled gracefully
//! - Security limits should be enforced

use libfuzzer_sys::fuzz_target;
use hedl_core::{parse, parse_with_limits, ParseOptions, Limits};

fuzz_target!(|data: &[u8]| {
    // Test 1: Parse with default limits
    // This exercises the main parser path with production defaults
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = parse(text.as_bytes());
    }

    // Test 2: Parse with very restrictive limits for edge case testing
    // This helps find issues with limit enforcement
    let restrictive_limits = Limits {
        max_file_size: 1024 * 10,      // 10KB
        max_line_length: 256,
        max_indent_depth: 5,
        max_nodes: 100,
        max_aliases: 10,
        max_columns: 10,
        max_nest_depth: 5,
        max_block_string_size: 1024,
        max_object_keys: 20,
        max_total_keys: 100,
    };

    let options = ParseOptions {
        limits: restrictive_limits,
        strict_refs: true,
    };

    if let Ok(text) = std::str::from_utf8(data) {
        let _ = parse_with_limits(text.as_bytes(), options.clone());
    }

    // Test 3: Parse with non-strict reference resolution
    // This tests the reference resolution path that allows unresolved refs
    let options_non_strict = ParseOptions {
        limits: Limits::default(),
        strict_refs: false,
    };

    if let Ok(text) = std::str::from_utf8(data) {
        let _ = parse_with_limits(text.as_bytes(), options_non_strict);
    }

    // Test 4: Test raw bytes (invalid UTF-8 handling)
    // The parser should gracefully reject invalid UTF-8
    let _ = parse(data);
});

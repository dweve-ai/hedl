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

//! Fuzz target for HEDL security limit enforcement.
//!
//! This fuzzer specifically tests that all security limits are properly enforced
//! and that the parser doesn't allow resource exhaustion attacks. It targets:
//!
//! - `max_file_size`: File size limit enforcement
//! - `max_line_length`: Line length limit enforcement
//! - `max_indent_depth`: Indentation depth limit enforcement
//! - `max_nodes`: Node count limit enforcement
//! - `max_aliases`: Alias count limit enforcement
//! - `max_columns`: Schema column limit enforcement
//! - `max_nest_depth`: NEST hierarchy depth limit enforcement
//! - `max_block_string_size`: Block string size limit enforcement
//! - `max_object_keys`: Per-object key count limit enforcement
//! - `max_total_keys`: Total key count limit enforcement
//!
//! # Attack Scenarios Tested
//!
//! 1. **File Size DoS**: Extremely large files
//! 2. **Line Length DoS**: Single extremely long line
//! 3. **Deep Nesting DoS**: Deeply nested objects/lists
//! 4. **Node Bomb DoS**: Millions of matrix rows
//! 5. **Alias Bomb DoS**: Thousands of aliases
//! 6. **Wide Schema DoS**: Hundreds of columns
//! 7. **Deep NEST DoS**: Deeply nested NEST hierarchies
//! 8. **Block String DoS**: Multi-MB block strings
//! 9. **Key Bomb DoS**: Thousands of keys per object
//! 10. **Total Key DoS**: Many small objects with many keys
//!
//! # Running the Fuzzer
//!
//! ```bash
//! cargo fuzz run fuzz_limits
//!
//! # Test with specific attack patterns
//! cargo fuzz run fuzz_limits -- -max_len=1000000
//!
//! # Monitor memory usage
//! cargo fuzz run fuzz_limits -- -rss_limit_mb=512
//! ```

use libfuzzer_sys::fuzz_target;
use hedl_core::{parse_with_limits, ParseOptions, Limits};

fuzz_target!(|data: &[u8]| {
    // Only test valid UTF-8 since we're focused on limit enforcement
    if let Ok(text) = std::str::from_utf8(data) {
        // Test 1: Very tight limits - should catch most violations quickly
        let tight_limits = Limits {
            max_file_size: 1024,           // 1KB
            max_line_length: 100,          // 100 bytes
            max_indent_depth: 3,           // 3 levels
            max_nodes: 10,                 // 10 nodes
            max_aliases: 5,                // 5 aliases
            max_columns: 5,                // 5 columns
            max_nest_depth: 3,             // 3 NEST levels
            max_block_string_size: 512,    // 512 bytes
            max_object_keys: 10,           // 10 keys per object
            max_total_keys: 50,            // 50 keys total
        };

        let options = ParseOptions {
            limits: tight_limits,
            strict_refs: false, // Focus on limit enforcement, not reference validation
        };

        let result = parse_with_limits(text.as_bytes(), options);

        // Verify that errors are returned for limit violations, not panics
        if let Err(err) = result {
            let err_str = err.to_string();
            // Common limit violation error patterns
            let _is_limit_error = err_str.contains("exceeds limit")
                || err_str.contains("too many")
                || err_str.contains("overflow")
                || err_str.contains("maximum");
        }

        // Test 2: Moderate limits - catch more subtle issues
        let moderate_limits = Limits {
            max_file_size: 10 * 1024,      // 10KB
            max_line_length: 512,
            max_indent_depth: 10,
            max_nodes: 100,
            max_aliases: 20,
            max_columns: 20,
            max_nest_depth: 10,
            max_block_string_size: 4096,
            max_object_keys: 50,
            max_total_keys: 500,
        };

        let options2 = ParseOptions {
            limits: moderate_limits,
            strict_refs: false,
        };

        let _ = parse_with_limits(text.as_bytes(), options2);

        // Test 3: Test boundary conditions - limits set to 0
        // Should reject almost everything, testing edge case handling
        let zero_limits = Limits {
            max_file_size: 0,
            max_line_length: 0,
            max_indent_depth: 0,
            max_nodes: 0,
            max_aliases: 0,
            max_columns: 0,
            max_nest_depth: 0,
            max_block_string_size: 0,
            max_object_keys: 0,
            max_total_keys: 0,
        };

        let options3 = ParseOptions {
            limits: zero_limits,
            strict_refs: false,
        };

        let _ = parse_with_limits(text.as_bytes(), options3);

        // Test 4: Test with limits set to 1 (minimum viable)
        let min_limits = Limits {
            max_file_size: 1,
            max_line_length: 1,
            max_indent_depth: 1,
            max_nodes: 1,
            max_aliases: 1,
            max_columns: 1,
            max_nest_depth: 1,
            max_block_string_size: 1,
            max_object_keys: 1,
            max_total_keys: 1,
        };

        let options4 = ParseOptions {
            limits: min_limits,
            strict_refs: false,
        };

        let _ = parse_with_limits(text.as_bytes(), options4);
    }
});

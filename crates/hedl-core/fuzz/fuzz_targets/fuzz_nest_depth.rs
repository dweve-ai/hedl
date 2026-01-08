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

//! Fuzz target for HEDL NEST hierarchy depth enforcement.
//!
//! This fuzzer specifically targets the NEST hierarchy depth limit, which is
//! critical for preventing stack overflow DoS attacks. NEST allows parent-child
//! relationships between entity types, and without depth limits, an attacker
//! could create documents with thousands of nested levels.
//!
//! # Attack Scenarios
//!
//! 1. **Deep Linear NEST**: A->B->C->D->... (single child per level)
//! 2. **Deep Wide NEST**: Multiple children at each level
//! 3. **Recursive NEST**: Type contains itself (if allowed)
//! 4. **Mixed Depth**: Some branches deep, others shallow
//! 5. **Boundary Testing**: Exactly at max_nest_depth limit
//!
//! # Security Importance
//!
//! The NEST depth limit prevents:
//! - Stack overflow from recursive parsing
//! - Excessive memory allocation from deep hierarchies
//! - CPU exhaustion from deep traversal
//! - Reference resolution complexity explosion
//!
//! # NEST Depth Checking Points
//!
//! 1. During parsing (`find_list_frame` - line 861-938)
//! 2. During reference collection (`collect_node_ids`)
//! 3. During reference validation (`validate_references`)
//!
//! All three paths must enforce the limit consistently.
//!
//! # Running the Fuzzer
//!
//! ```bash
//! cargo fuzz run fuzz_nest_depth
//!
//! # Test with deep nesting patterns
//! cargo fuzz run fuzz_nest_depth -- -max_len=50000
//!
//! # Monitor stack usage
//! cargo fuzz run fuzz_nest_depth -- -rss_limit_mb=256
//! ```

use libfuzzer_sys::fuzz_target;
use hedl_core::{parse_with_limits, ParseOptions, Limits};

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        // Test 1: Very shallow NEST depth (2 levels)
        // Should catch depth violations early
        let shallow_limits = Limits {
            max_nest_depth: 2,
            max_indent_depth: 10,           // Higher indent than NEST to isolate NEST testing
            max_nodes: 100,                 // Enough nodes to test depth
            ..Limits::default()
        };

        let options1 = ParseOptions {
            limits: shallow_limits,
            strict_refs: false,
        };

        let result1 = parse_with_limits(text.as_bytes(), options1);

        if let Err(err) = result1 {
            let err_str = err.to_string();
            // Check for NEST depth errors specifically
            let _is_nest_error = err_str.contains("NEST")
                && (err_str.contains("depth") || err_str.contains("exceeds"));
        }

        // Test 2: Moderate NEST depth (5 levels)
        let moderate_limits = Limits {
            max_nest_depth: 5,
            max_indent_depth: 20,
            max_nodes: 200,
            ..Limits::default()
        };

        let options2 = ParseOptions {
            limits: moderate_limits,
            strict_refs: false,
        };

        let _ = parse_with_limits(text.as_bytes(), options2);

        // Test 3: Single NEST level (no nesting allowed)
        let no_nest_limits = Limits {
            max_nest_depth: 1,
            max_indent_depth: 10,
            max_nodes: 50,
            ..Limits::default()
        };

        let options3 = ParseOptions {
            limits: no_nest_limits,
            strict_refs: false,
        };

        let _ = parse_with_limits(text.as_bytes(), options3);

        // Test 4: Zero NEST depth (should reject all NEST structures)
        let zero_nest_limits = Limits {
            max_nest_depth: 0,
            max_indent_depth: 10,
            max_nodes: 50,
            ..Limits::default()
        };

        let options4 = ParseOptions {
            limits: zero_nest_limits,
            strict_refs: false,
        };

        let _ = parse_with_limits(text.as_bytes(), options4);

        // Test 5: Deep NEST with strict reference checking
        // Tests interaction between NEST depth and reference resolution depth
        let deep_strict_limits = Limits {
            max_nest_depth: 10,
            max_indent_depth: 30,
            max_nodes: 500,
            ..Limits::default()
        };

        let options5 = ParseOptions {
            limits: deep_strict_limits,
            strict_refs: true,              // Strict mode for reference validation
        };

        let result5 = parse_with_limits(text.as_bytes(), options5);

        if let Err(err) = result5 {
            let err_str = err.to_string();
            // Could be NEST depth error or reference error
            let _is_depth_or_ref = err_str.contains("depth")
                || err_str.contains("reference")
                || err_str.contains("NEST");
        }

        // Test 6: Tight limits on both NEST and indent depth
        // Tests interaction between different depth limits
        let tight_both_limits = Limits {
            max_nest_depth: 3,
            max_indent_depth: 3,            // Same as NEST depth
            max_nodes: 30,
            ..Limits::default()
        };

        let options6 = ParseOptions {
            limits: tight_both_limits,
            strict_refs: false,
        };

        let _ = parse_with_limits(text.as_bytes(), options6);

        // Test 7: NEST depth higher than indent depth (unusual but valid)
        let unusual_limits = Limits {
            max_nest_depth: 10,
            max_indent_depth: 5,            // Lower than NEST - should hit indent first
            max_nodes: 100,
            ..Limits::default()
        };

        let options7 = ParseOptions {
            limits: unusual_limits,
            strict_refs: false,
        };

        let _ = parse_with_limits(text.as_bytes(), options7);
    }
});

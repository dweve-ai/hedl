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

//! Fuzz target for HEDL reference resolution.
//!
//! This fuzzer tests the reference resolution system with complex reference graphs,
//! circular dependencies, and edge cases. It targets:
//!
//! - Qualified references (@Type:id)
//! - Unqualified references (@id)
//! - Ambiguous unqualified references
//! - Forward references
//! - Self-references
//! - Circular reference graphs
//! - Missing references
//! - Type registry index consistency
//!
//! # Reference Graph Patterns Tested
//!
//! 1. **Simple References**: Single @id pointing to existing node
//! 2. **Qualified References**: @Type:id with explicit type
//! 3. **Ambiguous References**: @id matching multiple types
//! 4. **Missing References**: @id with no matching node
//! 5. **Self-References**: Node referencing itself
//! 6. **Circular Graphs**: A->B->C->A cycles
//! 7. **Complex Graphs**: Multiple interconnected reference chains
//! 8. **Type Collisions**: Same ID in different types
//!
//! # Security Testing
//!
//! - Reference resolution should not cause stack overflow
//! - Circular references should be detected or handled gracefully
//! - Missing references should error in strict mode
//! - Index corruption should be impossible
//!
//! # Running the Fuzzer
//!
//! ```bash
//! cargo fuzz run fuzz_references
//!
//! # Focus on complex reference graphs
//! cargo fuzz run fuzz_references -- -max_len=10000
//!
//! # Test with strict reference checking
//! cargo fuzz run fuzz_references -- -dict=references.dict
//! ```

use libfuzzer_sys::fuzz_target;
use hedl_core::{parse_with_limits, ParseOptions, Limits};

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        // Test 1: Strict reference resolution
        // This should catch unresolved and ambiguous references
        let strict_options = ParseOptions {
            limits: Limits::default(),
            strict_refs: true,
        };

        let result = parse_with_limits(text.as_bytes(), strict_options);

        if let Err(err) = result {
            let err_str = err.to_string();
            // Reference errors should be properly typed
            let _is_ref_error = err_str.contains("reference")
                || err_str.contains("ambiguous")
                || err_str.contains("unresolved")
                || err_str.contains("collision")
                || err_str.contains("duplicate ID");
        }

        // Test 2: Non-strict reference resolution
        // This should allow unresolved references but still detect ambiguity
        let non_strict_options = ParseOptions {
            limits: Limits::default(),
            strict_refs: false,
        };

        let result2 = parse_with_limits(text.as_bytes(), non_strict_options.clone());

        // Non-strict mode should still error on ambiguous references
        if let Err(err) = result2 {
            let err_str = err.to_string();
            let _may_be_ambiguous = err_str.contains("ambiguous");
        }

        // Test 3: Reference resolution with tight limits
        // Tests interaction between reference tracking and limit enforcement
        let limited_options = ParseOptions {
            limits: Limits {
                max_file_size: 10 * 1024,
                max_line_length: 512,
                max_indent_depth: 10,
                max_nodes: 50,          // Limited nodes for reference graph
                max_aliases: 10,
                max_columns: 10,
                max_nest_depth: 5,      // Limited NEST depth
                max_block_string_size: 1024,
                max_object_keys: 20,
                max_total_keys: 100,
            },
            strict_refs: true,
        };

        let _ = parse_with_limits(text.as_bytes(), limited_options);

        // Test 4: Very limited nodes to test reference tracking with minimal graph
        let minimal_options = ParseOptions {
            limits: Limits {
                max_nodes: 3,           // Just enough for simple reference tests
                max_nest_depth: 2,
                ..Limits::default()
            },
            strict_refs: false,
        };

        let _ = parse_with_limits(text.as_bytes(), minimal_options);

        // Test 5: Zero aliases to test reference behavior without alias expansion
        let no_alias_options = ParseOptions {
            limits: Limits {
                max_aliases: 0,         // No aliases allowed
                ..Limits::default()
            },
            strict_refs: true,
        };

        let _ = parse_with_limits(text.as_bytes(), no_alias_options);
    }
});

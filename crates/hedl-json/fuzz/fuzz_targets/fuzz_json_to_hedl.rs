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

use libfuzzer_sys::fuzz_target;
use hedl_json::{json_to_hedl, hedl_to_json, from_json, FromJsonConfig};

/// Fuzz target for JSON to HEDL conversion.
///
/// This fuzzer tests the robustness of the JSON parser against malformed,
/// malicious, or edge-case inputs. It helps identify:
///
/// - Panics or crashes from unexpected input
/// - Memory safety violations
/// - Integer overflows or underflows
/// - Infinite loops or excessive resource consumption
/// - Incorrect error handling
///
/// # Security Testing
///
/// The fuzzer specifically targets security-critical paths:
///
/// - Deeply nested JSON structures (DoS protection)
/// - Large arrays and objects (memory limits)
/// - Malformed JSON (error handling)
/// - Unicode edge cases
/// - Number parsing edge cases
///
/// # Running the Fuzzer
///
/// ```bash
/// # Install cargo-fuzz if not already installed
/// cargo install cargo-fuzz
///
/// # Run the fuzzer (from hedl-json directory)
/// cargo fuzz run fuzz_json_to_hedl
///
/// # Run with specific options
/// cargo fuzz run fuzz_json_to_hedl -- -max_len=10000 -max_total_time=300
///
/// # Run on multiple cores
/// cargo fuzz run fuzz_json_to_hedl -- -jobs=8
/// ```
fuzz_target!(|data: &[u8]| {
    // Attempt to parse the input as UTF-8 (JSON is text-based)
    if let Ok(json_str) = std::str::from_utf8(data) {
        // Use restrictive limits for fuzzing to prevent resource exhaustion
        let config = FromJsonConfig {
            max_depth: 50,
            max_string_length: 10_000,
            ..Default::default()
        };

        // Try parsing JSON to HEDL
        if let Ok(doc) = from_json(json_str, &config) {
            // If parsing succeeded, try round-trip conversion
            // This should never panic
            let _ = hedl_to_json(&doc);
        }

        // Also try the simple function
        let _ = json_to_hedl(json_str);
    }
});

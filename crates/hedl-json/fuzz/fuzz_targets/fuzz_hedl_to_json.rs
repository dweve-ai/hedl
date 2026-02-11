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
use hedl_json::{to_json, ToJsonConfig};

/// Fuzz target for HEDL to JSON conversion.
///
/// This fuzzer tests that the JSON generator handles all valid HEDL
/// documents without panicking, including edge cases in:
///
/// - Special characters that need JSON escaping
/// - Deeply nested structures
/// - Large documents
/// - Unicode content
/// - Number formatting
/// - Null and boolean values
///
/// # Running the Fuzzer
///
/// ```bash
/// cargo fuzz run fuzz_hedl_to_json
/// ```
fuzz_target!(|data: &[u8]| {
    // Try to parse as HEDL first
    if let Ok(doc) = hedl_core::parse(data) {
        // Test with default config
        let config = ToJsonConfig::default();
        let _ = to_json(&doc, &config);

        // Test with metadata disabled
        let no_meta_config = ToJsonConfig {
            include_metadata: false,
            ..Default::default()
        };
        let _ = to_json(&doc, &no_meta_config);

        // Test with flattened lists
        let flat_config = ToJsonConfig {
            flatten_lists: true,
            ..Default::default()
        };
        let _ = to_json(&doc, &flat_config);

        // Test with ASCII-safe output (escape non-ASCII)
        let ascii_config = ToJsonConfig {
            ascii_safe: true,
            ..Default::default()
        };
        let _ = to_json(&doc, &ascii_config);

        // Test with children excluded
        let no_children_config = ToJsonConfig {
            include_children: false,
            ..Default::default()
        };
        let _ = to_json(&doc, &no_children_config);

        // Test all config options combined
        let full_config = ToJsonConfig {
            include_metadata: true,
            flatten_lists: true,
            include_children: true,
            ascii_safe: true,
        };
        let _ = to_json(&doc, &full_config);
    }
});

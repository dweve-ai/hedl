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
use hedl_yaml::{to_yaml, ToYamlConfig};

/// Fuzz target for HEDL to YAML conversion.
///
/// This fuzzer tests that the YAML generator handles all valid HEDL
/// documents without panicking, including edge cases in:
///
/// - Special characters that need YAML escaping
/// - Deeply nested structures
/// - Large documents
/// - Unicode content
/// - Keys that need quoting
///
/// # Running the Fuzzer
///
/// ```bash
/// cargo fuzz run fuzz_hedl_to_yaml
/// ```
fuzz_target!(|data: &[u8]| {
    // Try to parse as HEDL first
    if let Ok(doc) = hedl_core::parse(data) {
        // Test with default config
        let config = ToYamlConfig::default();
        let _ = to_yaml(&doc, &config);

        // Test with metadata disabled
        let no_meta_config = ToYamlConfig {
            include_metadata: false,
            ..Default::default()
        };
        let _ = to_yaml(&doc, &no_meta_config);

        // Test with flattened lists
        let flat_config = ToYamlConfig {
            flatten_lists: true,
            ..Default::default()
        };
        let _ = to_yaml(&doc, &flat_config);
    }
});

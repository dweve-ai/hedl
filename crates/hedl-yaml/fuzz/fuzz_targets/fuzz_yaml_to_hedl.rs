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
use hedl_yaml::{from_yaml, to_yaml, FromYamlConfig, ToYamlConfig};

/// Fuzz target for YAML to HEDL conversion.
///
/// This fuzzer tests the robustness of the YAML parser against malformed,
/// malicious, or edge-case inputs. It helps identify:
///
/// - Panics or crashes from unexpected input
/// - YAML bomb attacks (alias expansion)
/// - Memory safety violations
/// - Incorrect error handling
///
/// # Security Testing
///
/// The fuzzer specifically targets security-critical paths:
///
/// - YAML anchors and aliases (circular references)
/// - Deeply nested structures
/// - Large documents
/// - Malformed YAML (invalid indentation, unclosed quotes)
/// - Unicode edge cases
/// - Multi-document YAML streams
///
/// # Running the Fuzzer
///
/// ```bash
/// cargo fuzz run fuzz_yaml_to_hedl
/// ```
fuzz_target!(|data: &[u8]| {
    // Attempt to parse the input as UTF-8 (YAML is text-based)
    if let Ok(yaml_str) = std::str::from_utf8(data) {
        // Use restrictive limits for fuzzing to prevent resource exhaustion
        let config = FromYamlConfig {
            max_nesting_depth: 50,
            max_array_length: 1000,
            max_document_size: 100_000, // 100KB for fuzzing
            ..Default::default()
        };

        // Try parsing YAML to HEDL
        if let Ok(doc) = from_yaml(yaml_str, &config) {
            // If parsing succeeded, try round-trip conversion
            let to_config = ToYamlConfig::default();
            let _ = to_yaml(&doc, &to_config);
        }

        // Also try with default config
        let default_config = FromYamlConfig::default();
        let _ = from_yaml(yaml_str, &default_config);
    }
});

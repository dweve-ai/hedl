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
use hedl_xml::{from_xml, to_xml, FromXmlConfig, ToXmlConfig};

/// Fuzz target for XML to HEDL conversion.
///
/// This fuzzer tests the robustness of the XML parser against malformed,
/// malicious, or edge-case inputs. It helps identify:
///
/// - Panics or crashes from unexpected input
/// - XXE (XML External Entity) attack vectors
/// - Billion laughs DoS attacks
/// - Memory safety violations
/// - Incorrect error handling
///
/// # Security Testing
///
/// The fuzzer specifically targets security-critical paths:
///
/// - DOCTYPE declarations and entity definitions
/// - Deeply nested XML structures
/// - Large attribute values
/// - Malformed XML (unclosed tags, invalid characters)
/// - Unicode edge cases in element names and content
///
/// # Running the Fuzzer
///
/// ```bash
/// cargo fuzz run fuzz_xml_to_hedl
/// ```
fuzz_target!(|data: &[u8]| {
    // Attempt to parse the input as UTF-8 (XML is text-based)
    if let Ok(xml_str) = std::str::from_utf8(data) {
        // Test with strict security config (rejects DOCTYPE)
        let strict_config = FromXmlConfig::strict_security();
        let _ = from_xml(xml_str, &strict_config);

        // Test with default config
        let default_config = FromXmlConfig::default();
        if let Ok(doc) = from_xml(xml_str, &default_config) {
            // If parsing succeeded, try round-trip conversion
            let to_config = ToXmlConfig::default();
            let _ = to_xml(&doc, &to_config);
        }

        // Test with list inference disabled
        let no_infer_config = FromXmlConfig {
            infer_lists: false,
            ..Default::default()
        };
        let _ = from_xml(xml_str, &no_infer_config);
    }
});

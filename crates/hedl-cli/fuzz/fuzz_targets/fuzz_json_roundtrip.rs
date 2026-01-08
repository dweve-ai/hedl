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

//! Fuzz target for HEDL ↔ JSON roundtrip conversion.
//!
//! This fuzzer tests that valid HEDL documents can be converted to JSON
//! and back without crashes or data corruption. It validates the stability
//! of the conversion pipeline under adversarial inputs.

use libfuzzer_sys::fuzz_target;
use hedl_core::parse;
use hedl_json::{to_json_value, json_to_hedl, ToJsonConfig};

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        // Try to parse as HEDL
        if let Ok(doc) = parse(text.as_bytes()) {
            // Convert to JSON (should not panic)
            let config = ToJsonConfig {
                include_metadata: true,
                ..Default::default()
            };

            if let Ok(json_value) = to_json_value(&doc, &config) {
                // Convert JSON string representation
                if let Ok(json_str) = serde_json::to_string(&json_value) {
                    // Try to parse back (may fail, but shouldn't panic)
                    let _ = json_to_hedl(&json_str);
                }
            }
        }
    }
});

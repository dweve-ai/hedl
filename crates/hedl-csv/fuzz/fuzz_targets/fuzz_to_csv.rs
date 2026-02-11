// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_main]

use libfuzzer_sys::fuzz_target;
use hedl_csv::{to_csv, to_csv_with_config, ToCsvConfig};

/// Fuzz target for HEDL to CSV conversion.
///
/// This fuzzer tests that the CSV generator handles all valid HEDL
/// documents without panicking, including edge cases in:
///
/// - Special characters that need CSV escaping (quotes, commas, newlines)
/// - Nested structures (flattening logic)
/// - Large documents
/// - Unicode content
/// - Various delimiter options
///
/// # Running the Fuzzer
///
/// ```bash
/// cargo fuzz run fuzz_to_csv
/// ```
fuzz_target!(|data: &[u8]| {
    // Try to parse as HEDL first
    if let Ok(doc) = hedl_core::parse(data) {
        // Test with default config
        let _ = to_csv(&doc);

        // Test with custom delimiter (semicolon)
        let semicolon_config = ToCsvConfig {
            delimiter: b';',
            ..Default::default()
        };
        let _ = to_csv_with_config(&doc, semicolon_config);

        // Test with tab delimiter
        let tab_config = ToCsvConfig {
            delimiter: b'\t',
            ..Default::default()
        };
        let _ = to_csv_with_config(&doc, tab_config);

        // Test with pipe delimiter
        let pipe_config = ToCsvConfig {
            delimiter: b'|',
            ..Default::default()
        };
        let _ = to_csv_with_config(&doc, pipe_config);

        // Test with no headers
        let no_header_config = ToCsvConfig {
            include_headers: false,
            ..Default::default()
        };
        let _ = to_csv_with_config(&doc, no_header_config);
    }
});

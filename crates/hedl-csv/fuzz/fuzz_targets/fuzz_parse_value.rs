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

/// Fuzz target for individual field value parsing.
///
/// This fuzzer focuses specifically on the type inference logic that converts
/// CSV field strings into HEDL values. It tests:
///
/// - Null detection (empty strings, "~")
/// - Boolean parsing ("true", "false")
/// - Integer parsing (various formats, edge cases)
/// - Float parsing (normal, scientific, special values)
/// - Reference parsing (@id, @Type:id)
/// - Expression parsing ($(expr))
/// - Tensor parsing ([1,2,3])
/// - String fallback
///
/// # Security Focus
///
/// This fuzzer helps identify:
///
/// - Integer overflow in number parsing
/// - Float parsing edge cases (NaN, Infinity, subnormals)
/// - Reference validation bypass
/// - Expression parser crashes
/// - Tensor depth limits
/// - UTF-8 handling issues
///
/// # Running the Fuzzer
///
/// ```bash
/// cargo fuzz run fuzz_parse_value
///
/// # Focus on shorter inputs for value parsing
/// cargo fuzz run fuzz_parse_value -- -max_len=1000
/// ```
fuzz_target!(|data: &[u8]| {
    // The parse_csv_value function is not public, so we test it indirectly
    // through from_csv by creating a minimal CSV with the fuzz data as a field
    if let Ok(field_str) = std::str::from_utf8(data) {
        // Create a minimal CSV with the fuzzed data as a field value
        // We use a quoted field to allow special characters
        let csv = format!("id,value\n1,\"{}\"\n", field_str.replace("\"", "\"\""));

        let config = hedl_csv::FromCsvConfig {
            max_rows: 10,
            ..Default::default()
        };

        // Attempt to parse - we don't care about the result, just that it doesn't crash
        let _ = hedl_csv::from_csv_with_config(&csv, "Item", &["value"], config);

        // Also test without quoting for cases where CSV library handles it
        if !field_str.contains(',') && !field_str.contains('\n') && !field_str.contains('"') {
            let csv_unquoted = format!("id,value\n1,{}\n", field_str);
            let config = hedl_csv::FromCsvConfig {
                max_rows: 10,
                ..Default::default()
            };
            let _ = hedl_csv::from_csv_with_config(&csv_unquoted, "Item", &["value"], config);
        }
    }
});

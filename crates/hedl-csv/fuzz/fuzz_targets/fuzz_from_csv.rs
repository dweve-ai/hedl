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
use hedl_csv::{from_csv_with_config, FromCsvConfig};

/// Fuzz target for CSV parsing.
///
/// This fuzzer tests the robustness of the CSV parser against malformed,
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
/// - Row count limits (DoS protection)
/// - Field parsing and type inference
/// - UTF-8 validation
/// - Reference parsing
/// - Expression parsing
/// - Tensor parsing
///
/// # Running the Fuzzer
///
/// ```bash
/// # Install cargo-fuzz if not already installed
/// cargo install cargo-fuzz
///
/// # Run the fuzzer (from hedl-csv directory)
/// cargo fuzz run fuzz_from_csv
///
/// # Run with specific options
/// cargo fuzz run fuzz_from_csv -- -max_len=10000 -max_total_time=300
///
/// # Run on multiple cores
/// cargo fuzz run fuzz_from_csv -- -jobs=8
/// ```
///
/// # Corpus
///
/// The fuzzer builds a corpus of interesting inputs that trigger different
/// code paths. You can add seed inputs to `fuzz/corpus/fuzz_from_csv/` to
/// guide the fuzzer toward specific scenarios.
fuzz_target!(|data: &[u8]| {
    // Attempt to parse the input as UTF-8 (CSV is text-based)
    if let Ok(csv_str) = std::str::from_utf8(data) {
        // Use restrictive limits for fuzzing to prevent resource exhaustion
        let config = FromCsvConfig {
            // Limit rows to prevent excessive memory allocation during fuzzing
            max_rows: 1000,
            ..Default::default()
        };

        // Try parsing with a minimal schema
        // We ignore errors since fuzzing will generate invalid inputs
        let _ = from_csv_with_config(csv_str, "FuzzItem", &["field1"], config.clone());

        // Try with multiple fields
        let _ = from_csv_with_config(
            csv_str,
            "FuzzItem",
            &["field1", "field2", "field3"],
            config.clone(),
        );

        // Try with tab delimiter
        let config_tsv = FromCsvConfig {
            delimiter: b'\t',
            max_rows: 1000,
            ..Default::default()
        };
        let _ = from_csv_with_config(csv_str, "FuzzItem", &["value"], config_tsv);

        // Try without headers
        let config_no_headers = FromCsvConfig {
            has_headers: false,
            max_rows: 1000,
            ..Default::default()
        };
        let _ = from_csv_with_config(csv_str, "FuzzItem", &["col"], config_no_headers);

        // Try without trimming
        let config_no_trim = FromCsvConfig {
            trim: false,
            max_rows: 1000,
            ..Default::default()
        };
        let _ = from_csv_with_config(csv_str, "FuzzItem", &["data"], config_no_trim);
    }

    // Also try parsing as raw bytes (tests UTF-8 validation)
    let config = FromCsvConfig {
        max_rows: 1000,
        ..Default::default()
    };
    let _ = hedl_csv::from_csv_reader_with_config(
        data,
        "FuzzItem",
        &["field"],
        config,
    );
});

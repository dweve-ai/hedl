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
use hedl_json::jsonpath::{query, QueryConfig};

/// Fuzz target for JSONPath query parsing and execution.
///
/// This fuzzer tests the robustness of the JSONPath query engine against
/// malformed or malicious query strings.
///
/// # Security Testing
///
/// The fuzzer specifically targets:
///
/// - Malformed JSONPath expressions
/// - Deeply nested path selectors
/// - Filter expressions with edge cases
/// - Unicode in path selectors
/// - Very long path expressions
///
/// # Running the Fuzzer
///
/// ```bash
/// cargo fuzz run fuzz_jsonpath
/// ```
fuzz_target!(|data: &[u8]| {
    // Attempt to parse the input as UTF-8
    if let Ok(path_str) = std::str::from_utf8(data) {
        // Create a simple document to query against
        let doc_bytes = b"%V:2.0\n---\nname: test\nvalue: 42\nitems: 3\n a\n b\n c";
        if let Ok(doc) = hedl_core::parse(doc_bytes) {
            let config = QueryConfig::default();

            // Try executing the query - should never panic
            let _ = query(&doc, path_str, &config);
        }
    }
});

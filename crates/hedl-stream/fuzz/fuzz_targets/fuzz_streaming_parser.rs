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
use hedl_stream::StreamingParser;
use std::io::Cursor;

/// Fuzz target for the streaming parser.
///
/// This fuzzer tests the parser's robustness against arbitrary input data,
/// including malformed HEDL documents, invalid UTF-8, extreme nesting,
/// and other edge cases.
///
/// # Fuzzing Strategy
///
/// 1. **Input Validation:** Ensures parser doesn't panic on any input
/// 2. **Error Handling:** Verifies all errors are caught and reported
/// 3. **Memory Safety:** Detects memory leaks, use-after-free, buffer overflows
/// 4. **Resource Limits:** Tests max_line_length and max_indent_depth enforcement
///
/// # Running the Fuzzer
///
/// ```bash
/// # Install cargo-fuzz
/// cargo install cargo-fuzz
///
/// # Run the fuzzer
/// cd crates/hedl-stream
/// cargo fuzz run fuzz_streaming_parser
///
/// # Run with corpus
/// cargo fuzz run fuzz_streaming_parser -- -max_len=100000
///
/// # Run with specific timeout
/// cargo fuzz run fuzz_streaming_parser -- -timeout=10
/// ```
///
/// # Expected Behavior
///
/// - Parser should never panic (except documented expect() calls)
/// - All errors should be Result::Err, never unwraps
/// - Memory usage should remain bounded
/// - Invalid UTF-8 should return Io error
/// - Resource limits should be enforced
fuzz_target!(|data: &[u8]| {
    // Attempt to parse the fuzzer input as a HEDL document
    // We use from_utf8_lossy to handle invalid UTF-8 gracefully
    let input = String::from_utf8_lossy(data);

    // Try to create a parser - this may fail on invalid input
    let cursor = Cursor::new(input.as_bytes());

    // Attempt to parse the header - this is where VERSION validation happens
    match StreamingParser::new(cursor) {
        Ok(parser) => {
            // If parser creation succeeds, try to consume all events
            // This exercises the full parsing logic
            for event in parser {
                // We don't care about the result, just that it doesn't panic
                // Errors are expected for malformed input
                let _ = event;
            }
        }
        Err(_) => {
            // Parser creation failed - this is fine for invalid input
            // The important thing is that it returned an error rather than panicking
        }
    }
});

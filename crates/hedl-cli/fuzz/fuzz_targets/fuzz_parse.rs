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

//! Fuzz target for HEDL parser.
//!
//! This fuzzer tests the parser for crashes, panics, and memory safety issues
//! with arbitrary input data. It helps discover edge cases and security
//! vulnerabilities in the parsing logic.

use libfuzzer_sys::fuzz_target;
use hedl_core::parse;

fuzz_target!(|data: &[u8]| {
    // Ignore if input is invalid UTF-8 - that's expected to fail gracefully
    if let Ok(text) = std::str::from_utf8(data) {
        // Parse should never panic, only return error
        let _ = parse(text.as_bytes());
    }
});

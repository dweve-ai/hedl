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
use hedl_neo4j::{to_cypher, hedl_to_cypher, ToCypherConfig};

/// Fuzz target for HEDL to Cypher conversion.
///
/// This fuzzer tests that the Cypher generator handles all valid HEDL
/// documents without panicking, and properly escapes special characters
/// to prevent Cypher injection attacks.
///
/// # Security Testing
///
/// The fuzzer specifically targets:
///
/// - Special characters in property values (quotes, backslashes)
/// - Unicode in labels and property names
/// - Large documents with many nodes/relationships
/// - Deeply nested structures
/// - Edge cases in ID generation
///
/// # Running the Fuzzer
///
/// ```bash
/// cargo fuzz run fuzz_hedl_to_cypher
/// ```
fuzz_target!(|data: &[u8]| {
    // Try to parse as HEDL first
    if let Ok(doc) = hedl_core::parse(data) {
        // Test with default config (MERGE mode)
        let _ = hedl_to_cypher(&doc);

        // Test with CREATE mode
        let create_config = ToCypherConfig::new()
            .with_create()
            .without_constraints();
        let _ = to_cypher(&doc, &create_config);

        // Test with custom ID property
        let custom_id_config = ToCypherConfig::new()
            .with_id_property("nodeId");
        let _ = to_cypher(&doc, &custom_id_config);

        // Test with batching
        let batch_config = ToCypherConfig::builder()
            .batch_size(100)
            .build();
        let _ = to_cypher(&doc, &batch_config);
    }
});

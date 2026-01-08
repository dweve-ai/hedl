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

//! Fuzz target for HEDL format command.
//!
//! This fuzzer tests the format command for crashes, panics, and memory safety
//! issues with arbitrary input data. It focuses on:
//! - Malformed HEDL input
//! - Deeply nested structures
//! - Oversized inputs
//! - Edge cases in canonicalization
//!
//! The fuzzer ensures that format operations never panic and handle all errors
//! gracefully, even with adversarial inputs.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Ignore if input is invalid UTF-8 - that's expected to fail gracefully
    if let Ok(text) = std::str::from_utf8(data) {
        // Limit input size to prevent timeout (100 KB max for fuzzing)
        if text.len() > 100_000 {
            return;
        }

        // Write input to a temporary file
        if let Ok(temp_file) = tempfile::NamedTempFile::new() {
            if std::fs::write(temp_file.path(), text).is_ok() {
                // Test format command (should never panic)
                // We call the underlying functions directly since we're testing the lib
                use hedl_core::parse;
                use hedl_c14n::{canonicalize_with_config, CanonicalConfig};

                // Parse should handle all malformed input gracefully
                if let Ok(doc) = parse(text.as_bytes()) {
                    // Test canonicalization without ditto
                    let config = CanonicalConfig::default();
                    let _ = canonicalize_with_config(&doc, &config);

                    // Test canonicalization with ditto
                    let mut config_ditto = CanonicalConfig::default();
                    config_ditto.use_ditto = true;
                    let _ = canonicalize_with_config(&doc, &config_ditto);

                    // Test count hints addition (format --with-counts)
                    let mut doc_with_counts = doc.clone();
                    add_count_hints_to_doc(&mut doc_with_counts);
                    let _ = canonicalize_with_config(&doc_with_counts, &config);
                }
            }
        }
    }
});

/// Helper to test count hints functionality (mirrors format.rs logic)
fn add_count_hints_to_doc(doc: &mut hedl_core::Document) {
    for item in doc.root.values_mut() {
        add_count_hints_to_item(item);
    }
}

fn add_count_hints_to_item(item: &mut hedl_core::Item) {
    match item {
        hedl_core::Item::List(list) => {
            list.count_hint = Some(list.rows.len());
            for node in &mut list.rows {
                add_child_count_to_node(node);
            }
        }
        hedl_core::Item::Object(map) => {
            for nested_item in map.values_mut() {
                add_count_hints_to_item(nested_item);
            }
        }
        hedl_core::Item::Scalar(_) => {}
    }
}

fn add_child_count_to_node(node: &mut hedl_core::Node) {
    let total_children: usize = node.children.values().map(|v| v.len()).sum();
    if total_children > 0 {
        node.child_count = Some(total_children);
        for child_list in node.children.values_mut() {
            for child_node in child_list {
                add_child_count_to_node(child_node);
            }
        }
    }
}

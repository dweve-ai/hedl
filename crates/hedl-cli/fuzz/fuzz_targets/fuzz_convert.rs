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

//! Fuzz target for HEDL conversion commands.
//!
//! This fuzzer tests all conversion operations (to/from JSON, YAML, XML, CSV)
//! for crashes, panics, and memory safety issues. It focuses on:
//! - Malformed input in each format
//! - Roundtrip conversion stability
//! - Edge cases in type conversions
//! - Oversized and deeply nested structures
//!
//! The fuzzer ensures that conversion operations never panic and handle all
//! errors gracefully.

use libfuzzer_sys::fuzz_target;
use hedl_core::parse;
use hedl_json::{to_json_value, from_json, ToJsonConfig, FromJsonConfig};
use hedl_yaml::{to_yaml, from_yaml, ToYamlConfig, FromYamlConfig};
use hedl_xml::{to_xml, from_xml, ToXmlConfig, FromXmlConfig};
use hedl_csv::{to_csv, from_csv};

fuzz_target!(|data: &[u8]| {
    // Limit input size to prevent timeout (100 KB max for fuzzing)
    if data.len() > 100_000 {
        return;
    }

    if let Ok(text) = std::str::from_utf8(data) {
        // Test HEDL → JSON → HEDL roundtrip
        test_json_conversion(text);

        // Test HEDL → YAML → HEDL roundtrip
        test_yaml_conversion(text);

        // Test HEDL → XML → HEDL roundtrip
        test_xml_conversion(text);

        // Test HEDL → CSV conversion (one-way for now)
        test_csv_conversion(text);

        // Test JSON → HEDL conversion
        test_from_json(text);

        // Test YAML → HEDL conversion
        test_from_yaml(text);

        // Test XML → HEDL conversion
        test_from_xml(text);

        // Test CSV → HEDL conversion
        test_from_csv(text);
    }
});

fn test_json_conversion(text: &str) {
    if let Ok(doc) = parse(text.as_bytes()) {
        // Convert to JSON with metadata
        let config = ToJsonConfig {
            include_metadata: true,
            ..Default::default()
        };
        if let Ok(json_value) = to_json_value(&doc, &config) {
            // Serialize to string
            if let Ok(json_str) = serde_json::to_string(&json_value) {
                // Try to convert back
                let from_config = FromJsonConfig::default();
                let _ = from_json(&json_str, &from_config);
            }

            // Also test pretty printing
            let _ = serde_json::to_string_pretty(&json_value);
        }

        // Convert without metadata
        let config_no_meta = ToJsonConfig {
            include_metadata: false,
            ..Default::default()
        };
        let _ = to_json_value(&doc, &config_no_meta);
    }
}

fn test_yaml_conversion(text: &str) {
    if let Ok(doc) = parse(text.as_bytes()) {
        let config = ToYamlConfig::default();
        if let Ok(yaml_str) = to_yaml(&doc, &config) {
            // Try to convert back
            let from_config = FromYamlConfig::default();
            let _ = from_yaml(&yaml_str, &from_config);
        }
    }
}

fn test_xml_conversion(text: &str) {
    if let Ok(doc) = parse(text.as_bytes()) {
        // Test default XML (minified)
        if let Ok(xml_str) = to_xml(&doc, &ToXmlConfig::default()) {
            // Try to convert back
            let from_config = FromXmlConfig::default();
            let _ = from_xml(&xml_str, &from_config);
        }

        // Test pretty XML
        let config_pretty = ToXmlConfig {
            pretty: true,
            ..Default::default()
        };
        let _ = to_xml(&doc, &config_pretty);
    }
}

fn test_csv_conversion(text: &str) {
    if let Ok(doc) = parse(text.as_bytes()) {
        // CSV conversion may fail for non-tabular data, which is expected
        let _ = to_csv(&doc);
    }
}

fn test_from_json(text: &str) {
    // Try to parse as JSON and convert to HEDL
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
        if let Ok(json_str) = serde_json::to_string(&value) {
            let config = FromJsonConfig::default();
            let _ = from_json(&json_str, &config);
        }
    }
}

fn test_from_yaml(text: &str) {
    // Try to parse as YAML and convert to HEDL
    let config = FromYamlConfig::default();
    let _ = from_yaml(text, &config);
}

fn test_from_xml(text: &str) {
    // Try to parse as XML and convert to HEDL
    let config = FromXmlConfig::default();
    let _ = from_xml(text, &config);
}

fn test_from_csv(text: &str) {
    // Try to parse as CSV and convert to HEDL
    // Use a safe type name
    let type_name = "FuzzTest";

    // Extract column names from first line if possible
    if let Some(first_line) = text.lines().next() {
        let columns: Vec<&str> = first_line.split(',').skip(1).collect();

        // Only test if we have reasonable column count (prevent DoS)
        if columns.len() <= 100 {
            let _ = from_csv(text, type_name, &columns);
        }
    }
}

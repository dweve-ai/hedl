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

//! Fuzz target for HEDL stats command.
//!
//! This fuzzer tests the stats generation for crashes, panics, and memory
//! safety issues. It focuses on:
//! - Malformed HEDL input
//! - Edge cases in size calculation
//! - Token estimation with unusual inputs
//! - Conversion to all formats for comparison
//!
//! The fuzzer ensures that stats operations never panic and handle all errors
//! gracefully.

use libfuzzer_sys::fuzz_target;
use hedl_core::parse;
use hedl_json::{to_json_value, ToJsonConfig};
use hedl_yaml::{to_yaml, ToYamlConfig};
use hedl_xml::{to_xml, ToXmlConfig};

/// Token estimation constants (mirrors stats.rs)
const CHARS_PER_CONTENT_TOKEN: usize = 4;
const WHITESPACE_PER_TOKEN: usize = 3;

fuzz_target!(|data: &[u8]| {
    // Limit input size to prevent timeout (100 KB max for fuzzing)
    if data.len() > 100_000 {
        return;
    }

    if let Ok(text) = std::str::from_utf8(data) {
        // Test stats calculation
        test_stats_calculation(text);
    }
});

fn test_stats_calculation(text: &str) {
    // Parse HEDL (may fail, which is expected)
    if let Ok(doc) = parse(text.as_bytes()) {
        let hedl_bytes = text.len();

        // Test JSON conversion and size calculation
        let json_config = ToJsonConfig::default();
        if let Ok(json_value) = to_json_value(&doc, &json_config) {
            // Compact JSON
            if let Ok(json) = serde_json::to_string(&json_value) {
                let json_bytes = json.len();
                let _ = calc_savings(hedl_bytes, json_bytes);

                // Test token estimation
                let hedl_tokens = estimate_tokens(text);
                let json_tokens = estimate_tokens(&json);
                let _ = calc_token_savings(hedl_tokens, json_tokens);
            }

            // Pretty JSON
            if let Ok(json_pretty) = serde_json::to_string_pretty(&json_value) {
                let json_pretty_bytes = json_pretty.len();
                let _ = calc_savings(hedl_bytes, json_pretty_bytes);

                let json_pretty_tokens = estimate_tokens(&json_pretty);
                let _ = calc_token_savings(estimate_tokens(text), json_pretty_tokens);
            }
        }

        // Test YAML conversion and size calculation
        let yaml_config = ToYamlConfig::default();
        if let Ok(yaml) = to_yaml(&doc, &yaml_config) {
            let yaml_bytes = yaml.len();
            let _ = calc_savings(hedl_bytes, yaml_bytes);

            let yaml_tokens = estimate_tokens(&yaml);
            let _ = calc_token_savings(estimate_tokens(text), yaml_tokens);
        }

        // Test XML conversion and size calculation (default/minified)
        if let Ok(xml) = to_xml(&doc, &ToXmlConfig::default()) {
            let xml_bytes = xml.len();
            let _ = calc_savings(hedl_bytes, xml_bytes);

            let xml_tokens = estimate_tokens(&xml);
            let _ = calc_token_savings(estimate_tokens(text), xml_tokens);
        }

        // Test pretty XML
        let xml_pretty_config = ToXmlConfig {
            pretty: true,
            ..Default::default()
        };
        if let Ok(xml_pretty) = to_xml(&doc, &xml_pretty_config) {
            let xml_pretty_bytes = xml_pretty.len();
            let _ = calc_savings(hedl_bytes, xml_pretty_bytes);

            let xml_pretty_tokens = estimate_tokens(&xml_pretty);
            let _ = calc_token_savings(estimate_tokens(text), xml_pretty_tokens);
        }
    }
}

/// Calculate byte savings (mirrors stats.rs)
fn calc_savings(hedl_bytes: usize, other: usize) -> (i64, f64) {
    let diff = other as i64 - hedl_bytes as i64;
    let pct = if other > 0 {
        (diff as f64 / other as f64) * 100.0
    } else {
        0.0
    };
    (diff, pct)
}

/// Calculate token savings (mirrors stats.rs)
fn calc_token_savings(hedl_tokens: usize, other: usize) -> (i64, f64) {
    let diff = other as i64 - hedl_tokens as i64;
    let pct = if other > 0 {
        (diff as f64 / other as f64) * 100.0
    } else {
        0.0
    };
    (diff, pct)
}

/// Estimate token count (mirrors stats.rs)
fn estimate_tokens(text: &str) -> usize {
    let chars = text.len();
    let whitespace = text.chars().filter(|c| c.is_whitespace()).count();
    let non_whitespace = chars - whitespace;

    let content_tokens = non_whitespace / CHARS_PER_CONTENT_TOKEN;
    let whitespace_tokens = whitespace / WHITESPACE_PER_TOKEN;

    content_tokens + whitespace_tokens
}

/// Test formatting functions (mirrors stats.rs)
#[allow(dead_code)]
fn format_bytes(bytes: usize) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}

#[allow(dead_code)]
fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

#[allow(dead_code)]
fn format_diff(diff: i64) -> String {
    if diff > 0 {
        format!("+{}", format_number(diff as usize))
    } else if diff < 0 {
        format!("-{}", format_number((-diff) as usize))
    } else {
        "0".to_string()
    }
}

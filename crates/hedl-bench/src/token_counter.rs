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

//! Token counting utilities for comparing HEDL efficiency vs other formats.
//!
//! Uses tiktoken-rs for accurate GPT-4 (cl100k_base) tokenization.

use hedl_core::Document;
use hedl_json::{to_json, ToJsonConfig};
use hedl_toon::ToToonConfig;
use once_cell::sync::Lazy;
use tiktoken_rs::{cl100k_base, CoreBPE};

// P0 OPTIMIZATION: Cache tokenizer (40-80x speedup)
static TOKENIZER: Lazy<CoreBPE> =
    Lazy::new(|| cl100k_base().expect("Failed to load cl100k_base tokenizer"));

/// Token efficiency statistics comparing HEDL to ALL other formats.
#[derive(Debug, Clone)]
pub struct TokenStats {
    /// HEDL format size in bytes
    pub hedl_bytes: usize,
    /// HEDL format token count
    pub hedl_tokens: usize,
    /// JSON compact format size in bytes
    pub json_compact_bytes: usize,
    /// JSON compact format token count
    pub json_compact_tokens: usize,
    /// JSON pretty format size in bytes
    pub json_pretty_bytes: usize,
    /// JSON pretty format token count
    pub json_pretty_tokens: usize,
    /// YAML format size in bytes
    pub yaml_bytes: usize,
    /// YAML format token count
    pub yaml_tokens: usize,
    /// XML format size in bytes
    pub xml_bytes: usize,
    /// XML format token count
    pub xml_tokens: usize,
    /// TOON format size in bytes
    pub toon_bytes: usize,
    /// TOON format token count
    pub toon_tokens: usize,
    /// CSV format size in bytes
    pub csv_bytes: usize,
    /// CSV format token count
    pub csv_tokens: usize,
    /// Token savings vs JSON compact (percentage, 0-100)
    pub savings_vs_json: f64,
    /// Token savings vs JSON pretty (percentage, 0-100)
    pub savings_vs_json_pretty: f64,
    /// Token savings vs YAML (percentage, 0-100)
    pub savings_vs_yaml: f64,
    /// Token savings vs XML (percentage, 0-100)
    pub savings_vs_xml: f64,
    /// Token savings vs TOON (percentage, 0-100)
    pub savings_vs_toon: f64,
    /// Token savings vs CSV (percentage, 0-100)
    pub savings_vs_csv: f64,
    /// Bytes per token for HEDL
    pub hedl_bytes_per_token: f64,
    /// Bytes per token for JSON
    pub json_bytes_per_token: f64,
}

impl TokenStats {
    /// Create a summary report as a formatted string showing ALL formats.
    pub fn report(&self) -> String {
        format!(
            r#"Token Efficiency Report - ALL FORMATS
=======================================

Format          | Bytes    | Tokens   | Bytes/Token | Savings
----------------|----------|----------|-------------|--------
HEDL            | {:>8} | {:>8} | {:>10.2} | baseline
JSON (compact)  | {:>8} | {:>8} | {:>10.2} | {:>6.1}%
JSON (pretty)   | {:>8} | {:>8} | {:>10.2} | {:>6.1}%
YAML            | {:>8} | {:>8} | {:>10.2} | {:>6.1}%
XML             | {:>8} | {:>8} | {:>10.2} | {:>6.1}%
TOON            | {:>8} | {:>8} | {:>10.2} | {:>6.1}%
CSV             | {:>8} | {:>8} | {:>10.2} | {:>6.1}%

HEDL Token Savings Summary:
  vs JSON (compact): {:.1}%
  vs JSON (pretty):  {:.1}%
  vs YAML:           {:.1}%
  vs XML:            {:.1}%
  vs TOON:           {:.1}%
  vs CSV:            {:.1}%
"#,
            self.hedl_bytes,
            self.hedl_tokens,
            self.hedl_bytes_per_token,
            self.json_compact_bytes,
            self.json_compact_tokens,
            self.json_bytes_per_token,
            self.savings_vs_json,
            self.json_pretty_bytes,
            self.json_pretty_tokens,
            self.json_pretty_bytes as f64 / self.json_pretty_tokens.max(1) as f64,
            self.savings_vs_json_pretty,
            self.yaml_bytes,
            self.yaml_tokens,
            self.yaml_bytes as f64 / self.yaml_tokens.max(1) as f64,
            self.savings_vs_yaml,
            self.xml_bytes,
            self.xml_tokens,
            self.xml_bytes as f64 / self.xml_tokens.max(1) as f64,
            self.savings_vs_xml,
            self.toon_bytes,
            self.toon_tokens,
            self.toon_bytes as f64 / self.toon_tokens.max(1) as f64,
            self.savings_vs_toon,
            self.csv_bytes,
            self.csv_tokens,
            self.csv_bytes as f64 / self.csv_tokens.max(1) as f64,
            self.savings_vs_csv,
            self.savings_vs_json,
            self.savings_vs_json_pretty,
            self.savings_vs_yaml,
            self.savings_vs_xml,
            self.savings_vs_toon,
            self.savings_vs_csv,
        )
    }
}

/// Count tokens in a text string using GPT-4's cl100k_base tokenizer.
///
/// # Example
///
/// ```no_run
/// use hedl_bench::count_tokens;
///
/// let tokens = count_tokens("Hello, world!");
/// assert!(tokens > 0);
/// ```
pub fn count_tokens(text: &str) -> usize {
    // P0 OPTIMIZATION: Use cached tokenizer (40-80x speedup)
    TOKENIZER.encode_with_special_tokens(text).len()
}

/// Compare token efficiency of HEDL document across ALL formats.
///
/// Takes a parsed HEDL document and computes token counts for:
/// - Original HEDL text (reconstructed via canonicalization)
/// - JSON compact format
/// - JSON pretty format
/// - YAML format
/// - XML format
/// - TOON format
/// - CSV format
///
/// # Example
///
/// ```no_run
/// use hedl_bench::{compare_formats, generate_users};
///
/// let hedl = generate_users(10);
/// let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
/// let stats = compare_formats(&doc);
///
/// println!("{}", stats.report());
/// ```
pub fn compare_formats(doc: &Document) -> TokenStats {
    // Reconstruct HEDL from document
    let hedl = hedl_c14n::canonicalize(doc).unwrap_or_default();

    // Convert to JSON compact
    let json_compact_config = ToJsonConfig {
        include_metadata: false,
        flatten_lists: true,
        include_children: true,
    };
    let json_compact = to_json(doc, &json_compact_config).unwrap_or_default();

    // Convert to JSON pretty (manual formatting since ToJsonConfig doesn't have pretty)
    let json_value: serde_json::Value = serde_json::from_str(&json_compact).unwrap_or_default();
    let json_pretty = serde_json::to_string_pretty(&json_value).unwrap_or_default();

    // Convert to YAML
    let yaml = hedl_yaml::to_yaml(doc, &hedl_yaml::ToYamlConfig::default()).unwrap_or_default();

    // Convert to XML
    let xml = hedl_xml::to_xml(doc, &hedl_xml::ToXmlConfig::default()).unwrap_or_default();

    // Convert to TOON
    let toon = hedl_toon::to_toon(doc, &ToToonConfig::default()).unwrap_or_default();

    // Convert to CSV (may fail for non-tabular data)
    let csv = hedl_csv::to_csv(doc).unwrap_or_default();

    // Count tokens
    let hedl_tokens = count_tokens(&hedl);
    let json_compact_tokens = count_tokens(&json_compact);
    let json_pretty_tokens = count_tokens(&json_pretty);
    let yaml_tokens = count_tokens(&yaml);
    let xml_tokens = count_tokens(&xml);
    let toon_tokens = count_tokens(&toon);
    let csv_tokens = count_tokens(&csv);

    // Calculate savings (use signed arithmetic to handle cases where HEDL might be larger)
    let savings_vs_json = if json_compact_tokens > 0 {
        ((json_compact_tokens as i64 - hedl_tokens as i64) as f64 / json_compact_tokens as f64)
            * 100.0
    } else {
        0.0
    };

    let savings_vs_json_pretty = if json_pretty_tokens > 0 {
        ((json_pretty_tokens as i64 - hedl_tokens as i64) as f64 / json_pretty_tokens as f64)
            * 100.0
    } else {
        0.0
    };

    let savings_vs_yaml = if yaml_tokens > 0 {
        ((yaml_tokens as i64 - hedl_tokens as i64) as f64 / yaml_tokens as f64) * 100.0
    } else {
        0.0
    };

    let savings_vs_xml = if xml_tokens > 0 {
        ((xml_tokens as i64 - hedl_tokens as i64) as f64 / xml_tokens as f64) * 100.0
    } else {
        0.0
    };

    let savings_vs_toon = if toon_tokens > 0 {
        ((toon_tokens as i64 - hedl_tokens as i64) as f64 / toon_tokens as f64) * 100.0
    } else {
        0.0
    };

    let savings_vs_csv = if csv_tokens > 0 {
        ((csv_tokens as i64 - hedl_tokens as i64) as f64 / csv_tokens as f64) * 100.0
    } else {
        0.0
    };

    TokenStats {
        hedl_bytes: hedl.len(),
        hedl_tokens,
        json_compact_bytes: json_compact.len(),
        json_compact_tokens,
        json_pretty_bytes: json_pretty.len(),
        json_pretty_tokens,
        yaml_bytes: yaml.len(),
        yaml_tokens,
        xml_bytes: xml.len(),
        xml_tokens,
        toon_bytes: toon.len(),
        toon_tokens,
        csv_bytes: csv.len(),
        csv_tokens,
        savings_vs_json,
        savings_vs_json_pretty,
        savings_vs_yaml,
        savings_vs_xml,
        savings_vs_toon,
        savings_vs_csv,
        hedl_bytes_per_token: hedl.len() as f64 / hedl_tokens.max(1) as f64,
        json_bytes_per_token: json_compact.len() as f64 / json_compact_tokens.max(1) as f64,
    }
}

/// Compare formats for a raw HEDL string.
///
/// Convenience function that parses the HEDL first.
pub fn compare_formats_str(hedl: &str) -> Result<TokenStats, String> {
    let doc = hedl_core::parse(hedl.as_bytes()).map_err(|e| format!("Parse error: {}", e))?;
    Ok(compare_formats(&doc))
}

/// Batch comparison across multiple datasets.
///
/// Returns a summary of token efficiency across all provided documents.
pub fn compare_batch(documents: &[(&str, &Document)]) -> Vec<(String, TokenStats)> {
    documents
        .iter()
        .map(|(name, doc)| (name.to_string(), compare_formats(doc)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datasets::generate_users;

    #[test]
    fn test_count_tokens() {
        let text = "Hello, world!";
        let tokens = count_tokens(text);
        assert!(tokens > 0);
        assert!(tokens < text.len()); // Tokens should be fewer than chars
    }

    #[test]
    fn test_compare_formats() {
        let hedl = generate_users(10);
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
        let stats = compare_formats(&doc);

        // HEDL should have fewer tokens than JSON
        assert!(stats.hedl_tokens < stats.json_pretty_tokens);
        assert!(stats.savings_vs_json_pretty > 0.0);

        // Verify byte counts are reasonable
        assert!(stats.hedl_bytes > 0);
        assert!(stats.json_compact_bytes > 0);
        assert!(stats.yaml_bytes > 0);
    }

    #[test]
    fn test_hedl_more_efficient_than_json() {
        // Test with various dataset sizes
        for count in [10, 50, 100] {
            let hedl = generate_users(count);
            let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
            let stats = compare_formats(&doc);

            // HEDL should be at least 20% more efficient than pretty JSON
            assert!(
                stats.savings_vs_json_pretty > 20.0,
                "Expected >20% savings for {} users, got {:.1}%",
                count,
                stats.savings_vs_json_pretty
            );
        }
    }

    #[test]
    fn test_report_format() {
        let hedl = generate_users(10);
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
        let stats = compare_formats(&doc);
        let report = stats.report();

        assert!(report.contains("Token Efficiency Report"));
        assert!(report.contains("HEDL"));
        assert!(report.contains("JSON"));
        assert!(report.contains("YAML"));
        assert!(report.contains("Savings"));
    }
}

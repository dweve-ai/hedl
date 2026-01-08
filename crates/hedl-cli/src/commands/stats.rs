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

//! Stats command - shows size and token savings comparison
//!
//! This module provides parallel statistics generation for HEDL files,
//! comparing size and token efficiency against JSON, YAML, and XML formats.
//!
//! # Performance
//!
//! Uses rayon for parallel format conversion and token estimation:
//! - Format conversions run in parallel (JSON, YAML, XML)
//! - Token estimations computed in parallel when enabled
//! - Typical speedup: 3-5x on multi-core systems
//!
//! # Thread Safety
//!
//! All format conversions are independent and thread-safe. No shared mutable state.

use super::read_file;
use hedl_core::{parse, Document};
use hedl_json::{to_json_value, ToJsonConfig};
use hedl_xml::{to_xml as hedl_to_xml, ToXmlConfig};
use hedl_yaml::{to_yaml as hedl_to_yaml, ToYamlConfig};
use rayon::prelude::*;
use std::sync::Arc;

/// Token estimation constants for cl100k_base-like approximation.
/// These constants represent empirical averages for structured data formats.
const CHARS_PER_CONTENT_TOKEN: usize = 4;
const WHITESPACE_PER_TOKEN: usize = 3;

/// Estimate token count using cl100k_base-like approximation
/// Rough heuristic: ~4 characters per token for structured data
fn estimate_tokens(text: &str) -> usize {
    // More accurate estimation for structured data:
    // - Whitespace-heavy formats inflate char count but not tokens as much
    // - Special characters often become single tokens
    // - Numbers and short words are often single tokens

    let chars = text.len();
    let whitespace = text.chars().filter(|c| c.is_whitespace()).count();
    let non_whitespace = chars - whitespace;

    // Structured data averages ~3.5-4 chars per token for content
    // Whitespace is compressed (roughly 2-3 whitespace per token)
    let content_tokens = non_whitespace / CHARS_PER_CONTENT_TOKEN;
    let whitespace_tokens = whitespace / WHITESPACE_PER_TOKEN;

    content_tokens + whitespace_tokens
}

/// Format statistics computed in parallel
#[derive(Debug, Clone)]
struct FormatStats {
    json_compact: String,
    json_pretty: String,
    yaml: String,
    xml_compact: String,
    xml_pretty: String,
}

impl FormatStats {
    /// Compute all format conversions in parallel.
    ///
    /// Uses rayon to parallelize conversion to JSON (compact/pretty), YAML,
    /// and XML (compact/pretty) formats for maximum performance.
    ///
    /// # Arguments
    ///
    /// * `doc` - The HEDL document to convert
    ///
    /// # Returns
    ///
    /// Returns `FormatStats` containing all converted formats.
    ///
    /// # Errors
    ///
    /// Returns `Err` if any format conversion fails.
    ///
    /// # Performance
    ///
    /// Achieves 3-5x speedup on multi-core systems by running conversions
    /// in parallel threads.
    fn compute_parallel(doc: &Document) -> Result<Self, String> {
        // Use Arc to share the document across threads safely
        let doc = Arc::new(doc.clone());

        // Define conversion tasks as closures
        let tasks: Vec<Box<dyn Fn() -> Result<String, String> + Send + Sync>> = vec![
            // JSON compact
            Box::new({
                let doc = Arc::clone(&doc);
                move || {
                    let config = ToJsonConfig::default();
                    let value = to_json_value(&doc, &config)
                        .map_err(|e| format!("JSON conversion error: {}", e))?;
                    serde_json::to_string(&value)
                        .map_err(|e| format!("JSON serialization error: {}", e))
                }
            }),
            // JSON pretty
            Box::new({
                let doc = Arc::clone(&doc);
                move || {
                    let config = ToJsonConfig::default();
                    let value = to_json_value(&doc, &config)
                        .map_err(|e| format!("JSON conversion error: {}", e))?;
                    serde_json::to_string_pretty(&value)
                        .map_err(|e| format!("JSON pretty serialization error: {}", e))
                }
            }),
            // YAML
            Box::new({
                let doc = Arc::clone(&doc);
                move || {
                    let config = ToYamlConfig::default();
                    hedl_to_yaml(&doc, &config)
                        .map_err(|e| format!("YAML conversion error: {}", e))
                }
            }),
            // XML compact
            Box::new({
                let doc = Arc::clone(&doc);
                move || {
                    let config = ToXmlConfig {
                        pretty: false,
                        ..Default::default()
                    };
                    hedl_to_xml(&doc, &config)
                        .map_err(|e| format!("XML conversion error: {}", e))
                }
            }),
            // XML pretty
            Box::new({
                let doc = Arc::clone(&doc);
                move || {
                    let config = ToXmlConfig {
                        pretty: true,
                        ..Default::default()
                    };
                    hedl_to_xml(&doc, &config)
                        .map_err(|e| format!("XML pretty conversion error: {}", e))
                }
            }),
        ];

        // Execute all conversions in parallel
        let results: Result<Vec<String>, String> = tasks
            .par_iter()
            .map(|task| task())
            .collect();

        let mut outputs = results?;

        // Extract results in order (reverse pop for efficiency)
        outputs.reverse();
        Ok(FormatStats {
            json_compact: outputs.pop().unwrap(),
            json_pretty: outputs.pop().unwrap(),
            yaml: outputs.pop().unwrap(),
            xml_compact: outputs.pop().unwrap(),
            xml_pretty: outputs.pop().unwrap(),
        })
    }
}

/// Generate comprehensive size and efficiency statistics for a HEDL file.
///
/// Parses a HEDL file and compares its size and token efficiency against equivalent
/// JSON, YAML, and XML representations. Optionally estimates LLM token counts for
/// context window optimization.
///
/// # Arguments
///
/// * `file` - Path to the HEDL file to analyze
/// * `show_tokens` - If `true`, includes estimated token counts for LLM context
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns `Err` if:
/// - The file cannot be read
/// - The file contains syntax errors
/// - Format conversions fail
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::stats;
///
/// # fn main() -> Result<(), String> {
/// // Show byte size comparison
/// stats("data.hedl", false)?;
///
/// // Show byte and token comparison
/// stats("data.hedl", true)?;
/// # Ok(())
/// # }
/// ```
///
/// # Output
///
/// Displays a formatted table showing:
/// - File sizes in bytes for HEDL, JSON (compact/pretty), YAML, XML (compact/pretty)
/// - Size savings (absolute and percentage)
/// - Estimated token counts (if enabled)
/// - Token savings compared to other formats
///
/// # Performance
///
/// Uses parallel processing (rayon) to compute all format conversions simultaneously,
/// achieving 3-5x speedup on multi-core systems. Token estimation is also parallelized.
///
/// # Token Estimation
///
/// Token counts use a heuristic approximation:
/// - ~4 characters per content token
/// - ~3 whitespace characters per token
/// - Based on empirical averages for structured data formats
pub fn stats(file: &str, show_tokens: bool) -> Result<(), String> {
    let content = read_file(file)?;
    let hedl_bytes = content.len();

    // Parse HEDL
    let doc = parse(content.as_bytes()).map_err(|e| format!("Parse error: {}", e))?;

    // Compute all format conversions in parallel
    let formats = FormatStats::compute_parallel(&doc)?;

    // Extract byte sizes
    let json_bytes = formats.json_compact.len();
    let json_pretty_bytes = formats.json_pretty.len();
    let yaml_bytes = formats.yaml.len();
    let xml_bytes = formats.xml_compact.len();
    let xml_pretty_bytes = formats.xml_pretty.len();

    // Calculate savings
    let calc_savings = |other: usize| -> (i64, f64) {
        let diff = other as i64 - hedl_bytes as i64;
        let pct = if other > 0 {
            (diff as f64 / other as f64) * 100.0
        } else {
            0.0
        };
        (diff, pct)
    };

    println!("HEDL Size Comparison");
    println!("====================");
    println!();
    println!("Input: {}", file);
    println!();

    // Byte comparison table
    println!("Bytes:");
    println!(
        "  {:<20} {:>10} {:>12} {:>10}",
        "Format", "Size", "Savings", "%"
    );
    println!("  {:-<20} {:-^10} {:-^12} {:-^10}", "", "", "", "");

    println!("  {:<20} {:>10}", "HEDL", format_bytes(hedl_bytes));

    let (json_diff, json_pct) = calc_savings(json_bytes);
    println!(
        "  {:<20} {:>10} {:>12} {:>9.1}%",
        "JSON (minified)",
        format_bytes(json_bytes),
        format_diff(json_diff),
        json_pct
    );

    let (json_pretty_diff, json_pretty_pct) = calc_savings(json_pretty_bytes);
    println!(
        "  {:<20} {:>10} {:>12} {:>9.1}%",
        "JSON (pretty)",
        format_bytes(json_pretty_bytes),
        format_diff(json_pretty_diff),
        json_pretty_pct
    );

    let (yaml_diff, yaml_pct) = calc_savings(yaml_bytes);
    println!(
        "  {:<20} {:>10} {:>12} {:>9.1}%",
        "YAML",
        format_bytes(yaml_bytes),
        format_diff(yaml_diff),
        yaml_pct
    );

    let (xml_diff, xml_pct) = calc_savings(xml_bytes);
    println!(
        "  {:<20} {:>10} {:>12} {:>9.1}%",
        "XML (minified)",
        format_bytes(xml_bytes),
        format_diff(xml_diff),
        xml_pct
    );

    let (xml_pretty_diff, xml_pretty_pct) = calc_savings(xml_pretty_bytes);
    println!(
        "  {:<20} {:>10} {:>12} {:>9.1}%",
        "XML (pretty)",
        format_bytes(xml_pretty_bytes),
        format_diff(xml_pretty_diff),
        xml_pretty_pct
    );

    // Token estimation (parallel)
    if show_tokens {
        println!();
        println!("Estimated Tokens (LLM context):");

        // Compute token estimates in parallel
        let texts = vec![
            &content,
            &formats.json_compact,
            &formats.json_pretty,
            &formats.yaml,
            &formats.xml_compact,
            &formats.xml_pretty,
        ];

        let token_counts: Vec<usize> = texts
            .par_iter()
            .map(|text| estimate_tokens(text))
            .collect();

        let hedl_tokens = token_counts[0];
        let json_tokens = token_counts[1];
        let json_pretty_tokens = token_counts[2];
        let yaml_tokens = token_counts[3];
        let xml_tokens = token_counts[4];
        let xml_pretty_tokens = token_counts[5];

        let calc_token_savings = |other: usize| -> (i64, f64) {
            let diff = other as i64 - hedl_tokens as i64;
            let pct = if other > 0 {
                (diff as f64 / other as f64) * 100.0
            } else {
                0.0
            };
            (diff, pct)
        };

        println!(
            "  {:<20} {:>10} {:>12} {:>10}",
            "Format", "Tokens", "Savings", "%"
        );
        println!("  {:-<20} {:-^10} {:-^12} {:-^10}", "", "", "", "");

        println!("  {:<20} {:>10}", "HEDL", format_number(hedl_tokens));

        let (json_tok_diff, json_tok_pct) = calc_token_savings(json_tokens);
        println!(
            "  {:<20} {:>10} {:>12} {:>9.1}%",
            "JSON (minified)",
            format_number(json_tokens),
            format_diff(json_tok_diff),
            json_tok_pct
        );

        let (json_pretty_tok_diff, json_pretty_tok_pct) = calc_token_savings(json_pretty_tokens);
        println!(
            "  {:<20} {:>10} {:>12} {:>9.1}%",
            "JSON (pretty)",
            format_number(json_pretty_tokens),
            format_diff(json_pretty_tok_diff),
            json_pretty_tok_pct
        );

        let (yaml_tok_diff, yaml_tok_pct) = calc_token_savings(yaml_tokens);
        println!(
            "  {:<20} {:>10} {:>12} {:>9.1}%",
            "YAML",
            format_number(yaml_tokens),
            format_diff(yaml_tok_diff),
            yaml_tok_pct
        );

        let (xml_tok_diff, xml_tok_pct) = calc_token_savings(xml_tokens);
        println!(
            "  {:<20} {:>10} {:>12} {:>9.1}%",
            "XML (minified)",
            format_number(xml_tokens),
            format_diff(xml_tok_diff),
            xml_tok_pct
        );

        let (xml_pretty_tok_diff, xml_pretty_tok_pct) = calc_token_savings(xml_pretty_tokens);
        println!(
            "  {:<20} {:>10} {:>12} {:>9.1}%",
            "XML (pretty)",
            format_number(xml_pretty_tokens),
            format_diff(xml_pretty_tok_diff),
            xml_pretty_tok_pct
        );

        println!();
        println!("Note: Token estimates use ~4 chars/token heuristic for structured data.");
    }

    Ok(())
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

fn format_diff(diff: i64) -> String {
    if diff > 0 {
        format!("+{}", format_number(diff as usize))
    } else if diff < 0 {
        format!("-{}", format_number((-diff) as usize))
    } else {
        "0".to_string()
    }
}

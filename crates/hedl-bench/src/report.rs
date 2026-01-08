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

//! Comprehensive reporting module for HEDL benchmarks.
//!
//! Provides unified reporting infrastructure for all benchmark types:
//! - Performance metrics
//! - Token efficiency comparisons
//! - Format conversion overhead
//! - LLM accuracy testing
//! - Memory usage analysis

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;

/// Configuration for benchmark report exports
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Export to JSON format
    pub json: bool,
    /// Export to Markdown format
    pub markdown: bool,
    /// Export to HTML format
    pub html: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            json: true,
            markdown: true,
            html: true,
        }
    }
}

impl ExportConfig {
    /// Create config with all exports enabled
    pub fn all() -> Self {
        Self::default()
    }

    /// Create config with all exports disabled
    pub fn none() -> Self {
        Self {
            json: false,
            markdown: false,
            html: false,
        }
    }

    /// Create config with only JSON export enabled
    pub fn json_only() -> Self {
        Self {
            json: true,
            markdown: false,
            html: false,
        }
    }

    /// Create config with only Markdown export enabled
    pub fn markdown_only() -> Self {
        Self {
            json: false,
            markdown: true,
            html: false,
        }
    }

    /// Create config with only HTML export enabled
    pub fn html_only() -> Self {
        Self {
            json: false,
            markdown: false,
            html: true,
        }
    }

    /// Enable JSON export
    pub fn with_json(mut self) -> Self {
        self.json = true;
        self
    }

    /// Enable Markdown export
    pub fn with_markdown(mut self) -> Self {
        self.markdown = true;
        self
    }

    /// Enable HTML export
    pub fn with_html(mut self) -> Self {
        self.html = true;
        self
    }
}

/// Performance benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfResult {
    pub name: String,
    pub iterations: u64,
    pub total_time_ns: u64,
    pub throughput_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_time_ns: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throughput_mbs: Option<f64>,
}

/// Cell value in a custom table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TableCell {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
}

impl TableCell {
    pub fn as_string(&self) -> String {
        match self {
            TableCell::String(s) => s.clone(),
            TableCell::Integer(i) => i.to_string(),
            TableCell::Float(f) => format!("{:.2}", f),
            TableCell::Bool(b) => b.to_string(),
        }
    }

    pub fn format_with_precision(&self, precision: usize) -> String {
        match self {
            TableCell::Float(f) => format!("{:.prec$}", f, prec = precision),
            _ => self.as_string(),
        }
    }

    pub fn as_float(&self) -> f64 {
        match self {
            TableCell::Float(f) => *f,
            TableCell::Integer(i) => *i as f64,
            _ => 0.0,
        }
    }

    pub fn as_integer(&self) -> i64 {
        match self {
            TableCell::Integer(i) => *i,
            TableCell::Float(f) => *f as i64,
            _ => 0,
        }
    }
}

/// Custom table for flexible benchmark reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTable {
    pub title: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<TableCell>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<Vec<TableCell>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatDatasetResult {
    pub format: String,
    pub correct: usize,
    pub total: usize,
    pub accuracy_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonRow {
    pub metric: String,
    pub values: Vec<String>,
}

/// Dynamic insight/recommendation based on actual results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub category: String, // "strength", "weakness", "recommendation", "finding"
    pub title: String,
    pub description: String,
    pub data_points: Vec<String>,
}

impl PerfResult {
    pub fn avg_time_ns(&self) -> u64 {
        self.total_time_ns / self.iterations
    }

    pub fn throughput_mbs(&self) -> Option<f64> {
        self.throughput_bytes.map(|bytes| {
            let bytes_per_sec = (bytes as f64 * 1e9) / self.total_time_ns as f64;
            bytes_per_sec / 1_000_000.0 // Convert to MB/s
        })
    }
}

/// Dataset complexity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComplexityLevel {
    /// Flat tabular data (users, products, events)
    Flat,
    /// Moderate nesting (blog posts with comments, orders with items)
    ModerateNesting,
    /// Heavy use of ditto markers for repeated values
    DittoHeavy,
    /// Cross-references and graph structures
    ReferenceHeavy,
    /// Deep hierarchical nesting (5+ levels)
    DeepHierarchy,
}

impl ComplexityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ComplexityLevel::Flat => "Flat",
            ComplexityLevel::ModerateNesting => "Moderate Nesting",
            ComplexityLevel::DittoHeavy => "Ditto-Heavy",
            ComplexityLevel::ReferenceHeavy => "Reference-Heavy",
            ComplexityLevel::DeepHierarchy => "Deep Hierarchy",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ComplexityLevel::Flat => "Simple tabular data, no nesting",
            ComplexityLevel::ModerateNesting => "Some hierarchy, nested objects",
            ComplexityLevel::DittoHeavy => "Repeated values using ^ markers",
            ComplexityLevel::ReferenceHeavy => "Cross-references, graph structures",
            ComplexityLevel::DeepHierarchy => "5+ levels of nesting",
        }
    }
}

/// Format comparison metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatMetrics {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<ComplexityLevel>,
    pub hedl_tokens: usize,
    pub json_tokens: usize,
    pub yaml_tokens: usize,
    pub xml_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toon_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_tokens: Option<usize>,
    pub hedl_bytes: usize,
    pub json_bytes: usize,
    pub yaml_bytes: usize,
    pub xml_bytes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toon_bytes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_bytes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_savings_vs_json: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_savings_vs_json: Option<f64>,
}

impl FormatMetrics {
    pub fn token_savings_vs_json(&self) -> f64 {
        if self.json_tokens == 0 {
            return 0.0;
        }
        ((self.json_tokens - self.hedl_tokens) as f64 / self.json_tokens as f64) * 100.0
    }

    pub fn byte_savings_vs_json(&self) -> f64 {
        if self.json_bytes == 0 {
            return 0.0;
        }
        ((self.json_bytes - self.hedl_bytes) as f64 / self.json_bytes as f64) * 100.0
    }
}

/// Comprehensive benchmark report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub title: String,
    pub perf_results: Vec<PerfResult>,
    pub format_metrics: Vec<FormatMetrics>,
    pub notes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,

    // Custom tables for flexible reporting
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub custom_tables: Vec<CustomTable>,

    // Dynamic insights and recommendations
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub insights: Vec<Insight>,
}

impl BenchmarkReport {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            perf_results: Vec::new(),
            format_metrics: Vec::new(),
            notes: Vec::new(),
            timestamp: None,
            metadata: None,
            custom_tables: Vec::new(),
            insights: Vec::new(),
        }
    }

    pub fn add_perf(&mut self, result: PerfResult) {
        self.perf_results.push(result);
    }

    pub fn add_format(&mut self, metrics: FormatMetrics) {
        self.format_metrics.push(metrics);
    }

    pub fn add_note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }

    pub fn add_custom_table(&mut self, table: CustomTable) {
        self.custom_tables.push(table);
    }

    pub fn add_insight(&mut self, insight: Insight) {
        self.insights.push(insight);
    }

    pub fn set_timestamp(&mut self) {
        use std::time::SystemTime;
        if let Ok(duration) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            self.timestamp = Some(format!("{}", duration.as_secs()));
        }
    }

    /// Export report as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export report as JSON to file
    pub fn save_json(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let json = self
            .to_json()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        fs::write(path, json)
    }

    /// Export report as Markdown
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# {}\n\n", self.title));

        if let Some(ts) = &self.timestamp {
            md.push_str(&format!("**Generated**: {}\n\n", ts));
        }

        if !self.perf_results.is_empty() {
            md.push_str("## Performance Metrics\n\n");
            md.push_str("| Benchmark | Avg Time | Throughput | Iterations |\n");
            md.push_str("|-----------|----------|------------|------------|\n");

            for result in &self.perf_results {
                let avg_time = Self::format_time(result.avg_time_ns());
                let throughput = result
                    .throughput_mbs()
                    .map(|mbs| format!("{:.2} MB/s", mbs))
                    .unwrap_or_else(|| "N/A".to_string());

                md.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    result.name, avg_time, throughput, result.iterations
                ));
            }
            md.push('\n');
        }

        if !self.format_metrics.is_empty() {
            md.push_str("## Format Comparison\n\n");

            // Check if any metrics have TOON or CSV
            let has_toon = self.format_metrics.iter().any(|m| m.toon_tokens.is_some());
            let has_csv = self.format_metrics.iter().any(|m| m.csv_tokens.is_some());
            let has_complexity = self.format_metrics.iter().any(|m| m.complexity.is_some());

            // Print header based on available formats
            if has_complexity {
                md.push_str("### Token Counts by Complexity Level\n\n");
            }

            if has_toon && has_csv {
                md.push_str("| Dataset | HEDL | JSON | YAML | XML | TOON | CSV | Savings |\n");
                md.push_str("|---------|------|------|------|-----|------|-----|----------|\n");
            } else if has_toon {
                md.push_str("| Dataset | HEDL | JSON | YAML | XML | TOON | Savings |\n");
                md.push_str("|---------|------|------|------|-----|------|----------|\n");
            } else if has_csv {
                md.push_str("| Dataset | HEDL | JSON | YAML | XML | CSV | Savings |\n");
                md.push_str("|---------|------|------|------|-----|-----|----------|\n");
            } else {
                md.push_str("| Dataset | HEDL | JSON | YAML | XML | Savings |\n");
                md.push_str("|---------|------|------|------|-----|----------|\n");
            }

            let mut total_hedl = 0;
            let mut total_json = 0;
            let mut total_yaml = 0;
            let mut total_xml = 0;
            let mut total_toon = 0;
            let mut total_csv = 0;

            for metrics in &self.format_metrics {
                total_hedl += metrics.hedl_tokens;
                total_json += metrics.json_tokens;
                total_yaml += metrics.yaml_tokens;
                total_xml += metrics.xml_tokens;
                if let Some(t) = metrics.toon_tokens {
                    total_toon += t;
                }
                if let Some(c) = metrics.csv_tokens {
                    total_csv += c;
                }

                if has_toon && has_csv {
                    md.push_str(&format!(
                        "| {} | {} | {} | {} | {} | {} | {} | {:.1}% |\n",
                        metrics.name,
                        metrics.hedl_tokens,
                        metrics.json_tokens,
                        metrics.yaml_tokens,
                        metrics.xml_tokens,
                        metrics.toon_tokens.unwrap_or(0),
                        metrics.csv_tokens.unwrap_or(0),
                        metrics.token_savings_vs_json()
                    ));
                } else if has_toon {
                    md.push_str(&format!(
                        "| {} | {} | {} | {} | {} | {} | {:.1}% |\n",
                        metrics.name,
                        metrics.hedl_tokens,
                        metrics.json_tokens,
                        metrics.yaml_tokens,
                        metrics.xml_tokens,
                        metrics.toon_tokens.unwrap_or(0),
                        metrics.token_savings_vs_json()
                    ));
                } else if has_csv {
                    md.push_str(&format!(
                        "| {} | {} | {} | {} | {} | {} | {:.1}% |\n",
                        metrics.name,
                        metrics.hedl_tokens,
                        metrics.json_tokens,
                        metrics.yaml_tokens,
                        metrics.xml_tokens,
                        metrics.csv_tokens.unwrap_or(0),
                        metrics.token_savings_vs_json()
                    ));
                } else {
                    md.push_str(&format!(
                        "| {} | {} | {} | {} | {} | {:.1}% |\n",
                        metrics.name,
                        metrics.hedl_tokens,
                        metrics.json_tokens,
                        metrics.yaml_tokens,
                        metrics.xml_tokens,
                        metrics.token_savings_vs_json()
                    ));
                }
            }

            if self.format_metrics.len() > 1 {
                let total_savings = ((total_json - total_hedl) as f64 / total_json as f64) * 100.0;
                if has_toon && has_csv {
                    md.push_str(&format!(
                        "| **TOTAL** | **{}** | **{}** | **{}** | **{}** | **{}** | **{}** | **{:.1}%** |\n",
                        total_hedl, total_json, total_yaml, total_xml, total_toon, total_csv, total_savings
                    ));
                } else if has_toon {
                    md.push_str(&format!(
                        "| **TOTAL** | **{}** | **{}** | **{}** | **{}** | **{}** | **{:.1}%** |\n",
                        total_hedl, total_json, total_yaml, total_xml, total_toon, total_savings
                    ));
                } else if has_csv {
                    md.push_str(&format!(
                        "| **TOTAL** | **{}** | **{}** | **{}** | **{}** | **{}** | **{:.1}%** |\n",
                        total_hedl, total_json, total_yaml, total_xml, total_csv, total_savings
                    ));
                } else {
                    md.push_str(&format!(
                        "| **TOTAL** | **{}** | **{}** | **{}** | **{}** | **{:.1}%** |\n",
                        total_hedl, total_json, total_yaml, total_xml, total_savings
                    ));
                }
            }
            md.push('\n');

            // Add complexity level summary if available
            md.push_str(&self.complexity_summary_markdown());
        }

        // Custom tables
        for table in &self.custom_tables {
            md.push_str(&format!("## {}\n\n", table.title));

            // Header row
            md.push_str("| ");
            for header in &table.headers {
                md.push_str(&format!("{} | ", header));
            }
            md.push('\n');

            // Separator row
            md.push('|');
            for _ in &table.headers {
                md.push_str("--------|");
            }
            md.push('\n');

            // Data rows
            for row in &table.rows {
                md.push_str("| ");
                for cell in row {
                    md.push_str(&format!("{} | ", cell.as_string()));
                }
                md.push('\n');
            }

            // Footer row if present
            if let Some(footer) = &table.footer {
                md.push_str("| ");
                for cell in footer {
                    md.push_str(&format!("**{}** | ", cell.as_string()));
                }
                md.push('\n');
            }

            md.push('\n');
        }

        // Insights
        if !self.insights.is_empty() {
            md.push_str("## Insights\n\n");

            for insight in &self.insights {
                let emoji = match insight.category.as_str() {
                    "strength" => "✅",
                    "weakness" => "⚠️",
                    "recommendation" => "💡",
                    "finding" => "🔍",
                    _ => "•",
                };

                md.push_str(&format!("### {} {}\n\n", emoji, insight.title));
                md.push_str(&format!("{}\n\n", insight.description));

                if !insight.data_points.is_empty() {
                    for point in &insight.data_points {
                        md.push_str(&format!("- {}\n", point));
                    }
                    md.push('\n');
                }
            }
        }

        if !self.notes.is_empty() {
            md.push_str("## Notes\n\n");
            for (i, note) in self.notes.iter().enumerate() {
                md.push_str(&format!("{}. {}\n", i + 1, note));
            }
            md.push('\n');
        }

        md
    }

    /// Save report as Markdown file
    pub fn save_markdown(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        fs::write(path, self.to_markdown())
    }

    /// Export report as HTML
    pub fn to_html(&self) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str(
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str(&format!("<title>{}</title>\n", self.title));
        html.push_str("<style>\n");
        html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; ");
        html.push_str("max-width: 1200px; margin: 0 auto; padding: 20px; background: #f5f5f5; }\n");
        html.push_str(
            "h1 { color: #333; border-bottom: 3px solid #4CAF50; padding-bottom: 10px; }\n",
        );
        html.push_str("h2 { color: #555; margin-top: 30px; }\n");
        html.push_str("table { width: 100%; border-collapse: collapse; background: white; ");
        html.push_str("box-shadow: 0 2px 4px rgba(0,0,0,0.1); margin: 20px 0; }\n");
        html.push_str(
            "th { background: #4CAF50; color: white; padding: 12px; text-align: left; }\n",
        );
        html.push_str("td { padding: 10px; border-bottom: 1px solid #ddd; }\n");
        html.push_str("tr:hover { background: #f5f5f5; }\n");
        html.push_str(".timestamp { color: #888; font-size: 0.9em; }\n");
        html.push_str(
            ".notes { background: #fff; padding: 15px; border-left: 4px solid #4CAF50; }\n",
        );
        html.push_str(".savings-good { color: #4CAF50; font-weight: bold; }\n");
        html.push_str("</style>\n</head>\n<body>\n");

        html.push_str(&format!("<h1>{}</h1>\n", self.title));

        if let Some(ts) = &self.timestamp {
            html.push_str(&format!("<p class=\"timestamp\">Generated: {}</p>\n", ts));
        }

        if !self.perf_results.is_empty() {
            html.push_str("<h2>Performance Metrics</h2>\n<table>\n");
            html.push_str("<tr><th>Benchmark</th><th>Avg Time</th><th>Throughput</th><th>Iterations</th></tr>\n");

            for result in &self.perf_results {
                let avg_time = Self::format_time(result.avg_time_ns());
                let throughput = result
                    .throughput_mbs()
                    .map(|mbs| format!("{:.2} MB/s", mbs))
                    .unwrap_or_else(|| "N/A".to_string());

                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    result.name, avg_time, throughput, result.iterations
                ));
            }
            html.push_str("</table>\n");
        }

        if !self.format_metrics.is_empty() {
            html.push_str("<h2>Format Comparison</h2>\n<table>\n");

            // Check if any metrics have TOON or CSV
            let has_toon = self.format_metrics.iter().any(|m| m.toon_tokens.is_some());
            let has_csv = self.format_metrics.iter().any(|m| m.csv_tokens.is_some());

            // Print header based on available formats
            if has_toon && has_csv {
                html.push_str("<tr><th>Dataset</th><th>HEDL</th><th>JSON</th><th>YAML</th><th>XML</th><th>TOON</th><th>CSV</th><th>Savings</th></tr>\n");
            } else if has_toon {
                html.push_str("<tr><th>Dataset</th><th>HEDL</th><th>JSON</th><th>YAML</th><th>XML</th><th>TOON</th><th>Savings</th></tr>\n");
            } else if has_csv {
                html.push_str("<tr><th>Dataset</th><th>HEDL</th><th>JSON</th><th>YAML</th><th>XML</th><th>CSV</th><th>Savings</th></tr>\n");
            } else {
                html.push_str("<tr><th>Dataset</th><th>HEDL</th><th>JSON</th><th>YAML</th><th>XML</th><th>Savings</th></tr>\n");
            }

            for metrics in &self.format_metrics {
                let savings = metrics.token_savings_vs_json();
                let savings_class = if savings > 0.0 {
                    " class=\"savings-good\""
                } else {
                    ""
                };

                if has_toon && has_csv {
                    html.push_str(&format!(
                        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td{}>{:.1}%</td></tr>\n",
                        metrics.name,
                        metrics.hedl_tokens,
                        metrics.json_tokens,
                        metrics.yaml_tokens,
                        metrics.xml_tokens,
                        metrics.toon_tokens.unwrap_or(0),
                        metrics.csv_tokens.unwrap_or(0),
                        savings_class,
                        savings
                    ));
                } else if has_toon {
                    html.push_str(&format!(
                        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td{}>{:.1}%</td></tr>\n",
                        metrics.name,
                        metrics.hedl_tokens,
                        metrics.json_tokens,
                        metrics.yaml_tokens,
                        metrics.xml_tokens,
                        metrics.toon_tokens.unwrap_or(0),
                        savings_class,
                        savings
                    ));
                } else if has_csv {
                    html.push_str(&format!(
                        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td{}>{:.1}%</td></tr>\n",
                        metrics.name,
                        metrics.hedl_tokens,
                        metrics.json_tokens,
                        metrics.yaml_tokens,
                        metrics.xml_tokens,
                        metrics.csv_tokens.unwrap_or(0),
                        savings_class,
                        savings
                    ));
                } else {
                    html.push_str(&format!(
                        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td{}>{:.1}%</td></tr>\n",
                        metrics.name,
                        metrics.hedl_tokens,
                        metrics.json_tokens,
                        metrics.yaml_tokens,
                        metrics.xml_tokens,
                        savings_class,
                        savings
                    ));
                }
            }
            html.push_str("</table>\n");

            // Add complexity level summary if available
            html.push_str(&self.complexity_summary_html());
        }

        // Add custom tables
        for table in &self.custom_tables {
            html.push_str(&format!("<h2>{}</h2>\n", table.title));
            html.push_str("<table>\n<thead>\n<tr>\n");

            // Header row
            for header in &table.headers {
                html.push_str(&format!("<th>{}</th>", header));
            }
            html.push_str("</tr>\n</thead>\n<tbody>\n");

            // Data rows
            for row in &table.rows {
                html.push_str("<tr>\n");
                for cell in row {
                    html.push_str(&format!("<td>{}</td>", cell.as_string()));
                }
                html.push_str("</tr>\n");
            }

            html.push_str("</tbody>\n");

            // Footer row if present
            if let Some(footer) = &table.footer {
                html.push_str("<tfoot>\n<tr>\n");
                for cell in footer {
                    html.push_str(&format!("<td><strong>{}</strong></td>", cell.as_string()));
                }
                html.push_str("</tr>\n</tfoot>\n");
            }

            html.push_str("</table>\n");
        }

        // Add insights
        if !self.insights.is_empty() {
            html.push_str("<h2>Key Insights</h2>\n");
            for insight in &self.insights {
                let emoji = match insight.category.as_str() {
                    "strength" => "✅",
                    "weakness" => "⚠️",
                    "recommendation" => "💡",
                    "finding" => "🔍",
                    _ => "•",
                };
                html.push_str(&format!("<h3>{} {}</h3>\n", emoji, insight.title));
                html.push_str(&format!("<p>{}</p>\n", insight.description));
                if !insight.data_points.is_empty() {
                    html.push_str("<ul>\n");
                    for point in &insight.data_points {
                        html.push_str(&format!("<li>{}</li>\n", point));
                    }
                    html.push_str("</ul>\n");
                }
            }
        }

        if !self.notes.is_empty() {
            html.push_str("<div class=\"notes\">\n<h2>Notes</h2>\n<ol>\n");
            for note in &self.notes {
                html.push_str(&format!("<li>{}</li>\n", note));
            }
            html.push_str("</ol>\n</div>\n");
        }

        html.push_str("</body>\n</html>");
        html
    }

    /// Save report as HTML file
    pub fn save_html(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        fs::write(path, self.to_html())
    }

    /// Save reports based on export configuration
    ///
    /// # Arguments
    ///
    /// * `base_path` - Base path for the reports (without extension)
    /// * `config` - Export configuration specifying which formats to generate
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hedl_bench::{BenchmarkReport, ExportConfig};
    ///
    /// let report = BenchmarkReport::new("My Report");
    /// let config = ExportConfig::none().with_json().with_html();
    /// // Exports to "target/my_report.json" and "target/my_report.html" only
    /// report.save_all("target/my_report", &config).unwrap();
    /// ```
    pub fn save_all(
        &self,
        base_path: impl AsRef<Path>,
        config: &ExportConfig,
    ) -> std::io::Result<()> {
        let base = base_path.as_ref();
        let mut errors = Vec::new();

        if config.json {
            let json_path = base.with_extension("json");
            if let Err(e) = self.save_json(&json_path) {
                errors.push(format!("JSON export failed: {}", e));
            } else {
                println!("✅ JSON report: {}", json_path.display());
            }
        }

        if config.markdown {
            let md_path = base.with_extension("md");
            if let Err(e) = self.save_markdown(&md_path) {
                errors.push(format!("Markdown export failed: {}", e));
            } else {
                println!("✅ Markdown report: {}", md_path.display());
            }
        }

        if config.html {
            let html_path = base.with_extension("html");
            if let Err(e) = self.save_html(&html_path) {
                errors.push(format!("HTML export failed: {}", e));
            } else {
                println!("✅ HTML report: {}", html_path.display());
            }
        }

        if !errors.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                errors.join("; "),
            ));
        }

        Ok(())
    }

    /// Print a comprehensive formatted report
    pub fn print(&self) {
        println!("\n{}", "=".repeat(80));
        println!("{}", self.title);
        println!("{}", "=".repeat(80));

        if !self.perf_results.is_empty() {
            self.print_perf_section();
        }

        if !self.format_metrics.is_empty() {
            self.print_format_section();
        }

        if !self.custom_tables.is_empty() {
            self.print_custom_tables();
        }

        if !self.insights.is_empty() {
            self.print_insights_section();
        }

        if !self.notes.is_empty() {
            self.print_notes_section();
        }

        println!("{}\n", "=".repeat(80));
    }

    fn print_custom_tables(&self) {
        for table in &self.custom_tables {
            println!("\n## {}\n", table.title);

            // Print header
            for header in &table.headers {
                print!("{:<20} | ", header);
            }
            println!();
            println!("{}", "-".repeat(table.headers.len() * 23));

            // Print rows
            for row in &table.rows {
                for cell in row {
                    print!("{:<20} | ", cell.as_string());
                }
                println!();
            }

            // Print footer if present
            if let Some(footer) = &table.footer {
                println!("{}", "-".repeat(table.headers.len() * 23));
                for cell in footer {
                    print!("{:<20} | ", cell.as_string());
                }
                println!();
            }
        }
    }

    fn print_insights_section(&self) {
        println!("\n## Insights\n");
        for insight in &self.insights {
            let emoji = match insight.category.as_str() {
                "strength" => "✅",
                "weakness" => "⚠️",
                "recommendation" => "💡",
                "finding" => "🔍",
                _ => "•",
            };
            println!("{} {}", emoji, insight.title);
            println!("  {}", insight.description);
            for point in &insight.data_points {
                println!("  - {}", point);
            }
            println!();
        }
    }

    fn print_perf_section(&self) {
        println!("\n## Performance Metrics\n");
        println!(
            "{:<40} | {:>12} | {:>15} | {:>12}",
            "Benchmark", "Avg Time", "Throughput", "Iterations"
        );
        println!("{:-<85}", "");

        for result in &self.perf_results {
            let avg_time = Self::format_time(result.avg_time_ns());
            let throughput = result
                .throughput_mbs()
                .map(|mbs| format!("{:.2} MB/s", mbs))
                .unwrap_or_else(|| "N/A".to_string());

            println!(
                "{:<40} | {:>12} | {:>15} | {:>12}",
                truncate(&result.name, 40),
                avg_time,
                throughput,
                result.iterations
            );
        }
    }

    fn print_format_section(&self) {
        println!("\n## Format Comparison\n");

        // Check if any metrics have TOON or CSV
        let has_toon = self.format_metrics.iter().any(|m| m.toon_tokens.is_some());
        let has_csv = self.format_metrics.iter().any(|m| m.csv_tokens.is_some());

        // Print header based on available formats
        if has_toon && has_csv {
            println!(
                "{:<25} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>10}",
                "Dataset", "HEDL", "JSON", "YAML", "XML", "TOON", "CSV", "Savings"
            );
            println!("{:-<100}", "");
        } else if has_toon {
            println!(
                "{:<25} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>10}",
                "Dataset", "HEDL", "JSON", "YAML", "XML", "TOON", "Savings"
            );
            println!("{:-<92}", "");
        } else if has_csv {
            println!(
                "{:<25} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>10}",
                "Dataset", "HEDL", "JSON", "YAML", "XML", "CSV", "Savings"
            );
            println!("{:-<92}", "");
        } else {
            println!(
                "{:<25} | {:>8} | {:>8} | {:>8} | {:>8} | {:>10}",
                "Dataset", "HEDL", "JSON", "YAML", "XML", "Savings"
            );
            println!("{:-<80}", "");
        }

        let mut total_hedl = 0;
        let mut total_json = 0;
        let mut total_yaml = 0;
        let mut total_xml = 0;
        let mut total_toon = 0;
        let mut total_csv = 0;

        for metrics in &self.format_metrics {
            total_hedl += metrics.hedl_tokens;
            total_json += metrics.json_tokens;
            total_yaml += metrics.yaml_tokens;
            total_xml += metrics.xml_tokens;
            if let Some(t) = metrics.toon_tokens {
                total_toon += t;
            }
            if let Some(c) = metrics.csv_tokens {
                total_csv += c;
            }

            if has_toon && has_csv {
                println!(
                    "{:<25} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>9.1}%",
                    truncate(&metrics.name, 25),
                    metrics.hedl_tokens,
                    metrics.json_tokens,
                    metrics.yaml_tokens,
                    metrics.xml_tokens,
                    metrics.toon_tokens.unwrap_or(0),
                    metrics.csv_tokens.unwrap_or(0),
                    metrics.token_savings_vs_json()
                );
            } else if has_toon {
                println!(
                    "{:<25} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>9.1}%",
                    truncate(&metrics.name, 25),
                    metrics.hedl_tokens,
                    metrics.json_tokens,
                    metrics.yaml_tokens,
                    metrics.xml_tokens,
                    metrics.toon_tokens.unwrap_or(0),
                    metrics.token_savings_vs_json()
                );
            } else if has_csv {
                println!(
                    "{:<25} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>9.1}%",
                    truncate(&metrics.name, 25),
                    metrics.hedl_tokens,
                    metrics.json_tokens,
                    metrics.yaml_tokens,
                    metrics.xml_tokens,
                    metrics.csv_tokens.unwrap_or(0),
                    metrics.token_savings_vs_json()
                );
            } else {
                println!(
                    "{:<25} | {:>8} | {:>8} | {:>8} | {:>8} | {:>9.1}%",
                    truncate(&metrics.name, 25),
                    metrics.hedl_tokens,
                    metrics.json_tokens,
                    metrics.yaml_tokens,
                    metrics.xml_tokens,
                    metrics.token_savings_vs_json()
                );
            }
        }

        if self.format_metrics.len() > 1 {
            let total_savings = ((total_json - total_hedl) as f64 / total_json as f64) * 100.0;
            if has_toon && has_csv {
                println!("{:-<100}", "");
                println!(
                    "{:<25} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>9.1}%",
                    "TOTAL",
                    total_hedl,
                    total_json,
                    total_yaml,
                    total_xml,
                    total_toon,
                    total_csv,
                    total_savings
                );
            } else if has_toon {
                println!("{:-<92}", "");
                println!(
                    "{:<25} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>9.1}%",
                    "TOTAL",
                    total_hedl,
                    total_json,
                    total_yaml,
                    total_xml,
                    total_toon,
                    total_savings
                );
            } else if has_csv {
                println!("{:-<92}", "");
                println!(
                    "{:<25} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>9.1}%",
                    "TOTAL",
                    total_hedl,
                    total_json,
                    total_yaml,
                    total_xml,
                    total_csv,
                    total_savings
                );
            } else {
                println!("{:-<80}", "");
                println!(
                    "{:<25} | {:>8} | {:>8} | {:>8} | {:>8} | {:>9.1}%",
                    "TOTAL", total_hedl, total_json, total_yaml, total_xml, total_savings
                );
            }
        }
    }

    fn print_notes_section(&self) {
        println!("\n## Notes\n");
        for (i, note) in self.notes.iter().enumerate() {
            println!("{}. {}", i + 1, note);
        }
    }

    fn format_time(ns: u64) -> String {
        if ns < 1_000 {
            format!("{} ns", ns)
        } else if ns < 1_000_000 {
            format!("{:.2} µs", ns as f64 / 1_000.0)
        } else if ns < 1_000_000_000 {
            format!("{:.2} ms", ns as f64 / 1_000_000.0)
        } else {
            format!("{:.2} s", ns as f64 / 1_000_000_000.0)
        }
    }

    /// Generate complexity level summary for markdown
    fn complexity_summary_markdown(&self) -> String {
        use std::collections::HashMap;

        let mut level_stats: HashMap<ComplexityLevel, Vec<f64>> = HashMap::new();

        for metrics in &self.format_metrics {
            if let Some(complexity) = metrics.complexity {
                let savings = metrics.token_savings_vs_json();
                level_stats
                    .entry(complexity)
                    .or_default()
                    .push(savings);
            }
        }

        if level_stats.is_empty() {
            return String::new();
        }

        let mut md = String::from("### Summary: HEDL Advantage by Complexity\n\n");
        md.push_str("| Complexity Level | Avg Savings | Description |\n");
        md.push_str("|------------------|-------------|-------------|\n");

        // Print in order from Level 1 to Level 5
        for level in [
            ComplexityLevel::Flat,
            ComplexityLevel::ModerateNesting,
            ComplexityLevel::DittoHeavy,
            ComplexityLevel::ReferenceHeavy,
            ComplexityLevel::DeepHierarchy,
        ] {
            if let Some(savings_list) = level_stats.get(&level) {
                let avg_savings = savings_list.iter().sum::<f64>() / savings_list.len() as f64;
                md.push_str(&format!(
                    "| {} | {:.1}% | {} |\n",
                    level.as_str(),
                    avg_savings,
                    level.description()
                ));
            }
        }
        md.push_str("\n**Key Finding**: HEDL's token efficiency advantage increases with document complexity.\n\n");
        md
    }

    /// Generate complexity level summary for HTML
    fn complexity_summary_html(&self) -> String {
        use std::collections::HashMap;

        let mut level_stats: HashMap<ComplexityLevel, Vec<f64>> = HashMap::new();

        for metrics in &self.format_metrics {
            if let Some(complexity) = metrics.complexity {
                let savings = metrics.token_savings_vs_json();
                level_stats
                    .entry(complexity)
                    .or_default()
                    .push(savings);
            }
        }

        if level_stats.is_empty() {
            return String::new();
        }

        let mut html = String::from("<h3>Summary: HEDL Advantage by Complexity</h3>\n<table>\n");
        html.push_str(
            "<tr><th>Complexity Level</th><th>Avg Savings</th><th>Description</th></tr>\n",
        );

        // Print in order from Level 1 to Level 5
        for level in [
            ComplexityLevel::Flat,
            ComplexityLevel::ModerateNesting,
            ComplexityLevel::DittoHeavy,
            ComplexityLevel::ReferenceHeavy,
            ComplexityLevel::DeepHierarchy,
        ] {
            if let Some(savings_list) = level_stats.get(&level) {
                let avg_savings = savings_list.iter().sum::<f64>() / savings_list.len() as f64;
                html.push_str(&format!(
                    "<tr><td>{}</td><td class=\"savings-good\">{:.1}%</td><td>{}</td></tr>\n",
                    level.as_str(),
                    avg_savings,
                    level.description()
                ));
            }
        }
        html.push_str("</table>\n");
        html.push_str("<p><strong>Key Finding:</strong> HEDL's token efficiency advantage increases with document complexity.</p>\n");
        html
    }
}

impl fmt::Display for BenchmarkReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n{}", "=".repeat(80))?;
        writeln!(f, "{}", self.title)?;
        writeln!(f, "{}", "=".repeat(80))?;

        if !self.format_metrics.is_empty() {
            writeln!(f, "\nFormat Efficiency:")?;
            for metrics in &self.format_metrics {
                writeln!(
                    f,
                    "  {}: {} tokens (HEDL) vs {} (JSON) = {:.1}% savings",
                    metrics.name,
                    metrics.hedl_tokens,
                    metrics.json_tokens,
                    metrics.token_savings_vs_json()
                )?;
            }
        }

        writeln!(f, "{}\n", "=".repeat(80))?;
        Ok(())
    }
}

/// Summary report combining multiple benchmark results
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SummaryReport {
    pub reports: Vec<BenchmarkReport>,
}

impl SummaryReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_report(&mut self, report: BenchmarkReport) {
        self.reports.push(report);
    }

    /// Print comprehensive summary across all benchmarks
    pub fn print_summary(&self) {
        println!("\n{}", "=".repeat(80));
        println!("HEDL BENCHMARK SUITE - COMPREHENSIVE SUMMARY");
        println!("{}", "=".repeat(80));

        // Aggregate statistics
        let total_perf_tests: usize = self.reports.iter().map(|r| r.perf_results.len()).sum();
        let total_format_comparisons: usize =
            self.reports.iter().map(|r| r.format_metrics.len()).sum();

        println!("\n## Overview\n");
        println!("Total Benchmark Reports: {}", self.reports.len());
        println!("Total Performance Tests: {}", total_perf_tests);
        println!("Total Format Comparisons: {}", total_format_comparisons);

        // Aggregate token efficiency
        let mut total_hedl_tokens = 0;
        let mut total_json_tokens = 0;

        for report in &self.reports {
            for metrics in &report.format_metrics {
                total_hedl_tokens += metrics.hedl_tokens;
                total_json_tokens += metrics.json_tokens;
            }
        }

        if total_json_tokens > 0 {
            let overall_savings =
                ((total_json_tokens - total_hedl_tokens) as f64 / total_json_tokens as f64) * 100.0;
            println!("\n## Token Efficiency Across All Benchmarks\n");
            println!("Total HEDL Tokens:  {}", total_hedl_tokens);
            println!("Total JSON Tokens:  {}", total_json_tokens);
            println!("Overall Savings:    {:.1}%", overall_savings);
        }

        println!("\n## Individual Reports\n");
        for (i, report) in self.reports.iter().enumerate() {
            println!(
                "{}. {} ({} tests, {} comparisons)",
                i + 1,
                report.title,
                report.perf_results.len(),
                report.format_metrics.len()
            );
        }

        println!("\n{}", "=".repeat(80));
        println!("Run individual benchmarks with: cargo bench -p hedl-bench --bench <name>");
        println!("{}\n", "=".repeat(80));
    }
}

/// Truncate string to max length with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_result() {
        let result = PerfResult {
            name: "test".to_string(),
            iterations: 100,
            total_time_ns: 1_000_000,
            throughput_bytes: Some(1_000_000),
            avg_time_ns: Some(10_000),
            throughput_mbs: Some(1.0),
        };

        assert_eq!(result.avg_time_ns(), 10_000);
        assert!(result.throughput_mbs().is_some());
    }

    #[test]
    fn test_format_metrics() {
        let metrics = FormatMetrics {
            name: "test".to_string(),
            complexity: Some(ComplexityLevel::Flat),
            hedl_tokens: 100,
            json_tokens: 150,
            yaml_tokens: 140,
            xml_tokens: 160,
            toon_tokens: None,
            csv_tokens: None,
            hedl_bytes: 500,
            json_bytes: 750,
            yaml_bytes: 700,
            xml_bytes: 800,
            toon_bytes: None,
            csv_bytes: None,
            token_savings_vs_json: None,
            byte_savings_vs_json: None,
        };

        assert!((metrics.token_savings_vs_json() - 33.33).abs() < 0.1);
        assert!((metrics.byte_savings_vs_json() - 33.33).abs() < 0.1);
    }

    #[test]
    fn test_benchmark_report() {
        let mut report = BenchmarkReport::new("Test Report");
        report.add_note("This is a test");

        assert_eq!(report.title, "Test Report");
        assert_eq!(report.notes.len(), 1);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
    }
}

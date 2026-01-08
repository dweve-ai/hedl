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

//! HTML export for benchmark reports.

use crate::reporters::types::BenchmarkReport;
use std::fs;
use std::io;
use std::path::Path;

/// Exports benchmark report as HTML.
///
/// # Arguments
///
/// * `report` - The benchmark report to export
/// * `path` - Output file path
///
/// # Returns
///
/// Result indicating success or failure.
pub fn export_html(report: &BenchmarkReport, path: &Path) -> io::Result<()> {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str(&format!("<title>{}</title>\n", report.title));
    html.push_str("<style>\n");
    html.push_str(include_str!("styles.css"));
    html.push_str("</style>\n</head>\n<body>\n");

    html.push_str(&format!("<h1>{}</h1>\n", report.title));
    html.push_str(&format!("<p>Timestamp: {}</p>\n", report.timestamp));

    if !report.notes.is_empty() {
        html.push_str("<h2>Notes</h2>\n<ul>\n");
        for note in &report.notes {
            html.push_str(&format!("<li>{}</li>\n", note));
        }
        html.push_str("</ul>\n");
    }

    html.push_str("<h2>Results</h2>\n");
    html.push_str("<table>\n<tr><th>Benchmark</th><th>Duration</th><th>Iterations</th><th>Throughput</th></tr>\n");

    for result in &report.results {
        let throughput: String = result
            .throughput_mbs()
            .map(|t| format!("{:.2} MB/s", t))
            .unwrap_or_else(|| "N/A".to_string());

        html.push_str(&format!(
            "<tr><td>{}</td><td>{:?}</td><td>{}</td><td>{}</td></tr>\n",
            result.name,
            result.avg_duration(),
            result.iterations,
            throughput
        ));
    }
    html.push_str("</table>\n");

    if !report.analysis.bottlenecks.is_empty() {
        html.push_str("<h2>Bottlenecks</h2>\n<ul>\n");
        for bottleneck in &report.analysis.bottlenecks {
            html.push_str(&format!(
                "<li><strong>[{}]</strong> {}: {}</li>\n",
                bottleneck.severity.as_str().to_uppercase(),
                bottleneck.location,
                bottleneck.description
            ));
        }
        html.push_str("</ul>\n");
    }

    if !report.recommendations.is_empty() {
        html.push_str("<h2>Recommendations</h2>\n<ol>\n");
        for rec in &report.recommendations {
            html.push_str(&format!(
                "<li><strong>[{}]</strong> {}</li>\n",
                rec.severity.as_str().to_uppercase(),
                rec.message
            ));
        }
        html.push_str("</ol>\n");
    }

    html.push_str("</body>\n</html>");

    fs::write(path, html)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_html() {
        let report = BenchmarkReport::new("Test");
        let temp = NamedTempFile::new().unwrap();

        export_html(&report, temp.path()).unwrap();

        let content = fs::read_to_string(temp.path()).unwrap();
        assert!(content.contains("<h1>Test</h1>"));
    }
}

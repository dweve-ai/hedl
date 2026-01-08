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

//! Markdown export for benchmark reports.

use crate::reporters::types::BenchmarkReport;
use std::fs;
use std::io;
use std::path::Path;

/// Exports benchmark report as Markdown.
///
/// # Arguments
///
/// * `report` - The benchmark report to export
/// * `path` - Output file path
///
/// # Returns
///
/// Result indicating success or failure.
pub fn export_markdown(report: &BenchmarkReport, path: &Path) -> io::Result<()> {
    let mut md = String::new();

    md.push_str(&format!("# {}\n\n", report.title));
    md.push_str(&format!("**Timestamp:** {}\n\n", report.timestamp));

    if !report.notes.is_empty() {
        md.push_str("## Notes\n\n");
        for note in &report.notes {
            md.push_str(&format!("- {}\n", note));
        }
        md.push('\n');
    }

    md.push_str("## Results\n\n");
    md.push_str("| Benchmark | Duration | Iterations | Throughput |\n");
    md.push_str("|-----------|----------|------------|------------|\n");

    for result in &report.results {
        let throughput: String = result
            .throughput_mbs()
            .map(|t| format!("{:.2} MB/s", t))
            .unwrap_or_else(|| "N/A".to_string());

        md.push_str(&format!(
            "| {} | {:?} | {} | {} |\n",
            result.name,
            result.avg_duration(),
            result.iterations,
            throughput
        ));
    }

    if !report.analysis.bottlenecks.is_empty() {
        md.push_str("\n## Bottlenecks\n\n");
        for bottleneck in &report.analysis.bottlenecks {
            md.push_str(&format!(
                "- **[{}]** {}: {}\n",
                bottleneck.severity.as_str().to_uppercase(),
                bottleneck.location,
                bottleneck.description
            ));
        }
    }

    if !report.analysis.regressions.is_empty() {
        md.push_str("\n## Regressions\n\n");
        for regression in &report.analysis.regressions {
            md.push_str(&format!(
                "- **[{}]** {}: {}% slower\n",
                regression.status.severity().to_uppercase(),
                regression.name,
                regression.status.percentage()
            ));
        }
    }

    if !report.recommendations.is_empty() {
        md.push_str("\n## Recommendations\n\n");
        for (i, rec) in report.recommendations.iter().enumerate() {
            md.push_str(&format!(
                "{}. **[{}]** {}\n",
                i + 1,
                rec.severity.as_str().to_uppercase(),
                rec.message
            ));
            md.push_str(&format!(
                "   - Impact: {:.1}% improvement\n",
                rec.impact.improvement_pct
            ));
        }
    }

    fs::write(path, md)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_markdown() {
        let report = BenchmarkReport::new("Test");
        let temp = NamedTempFile::new().unwrap();

        export_markdown(&report, temp.path()).unwrap();

        let content = fs::read_to_string(temp.path()).unwrap();
        assert!(content.contains("# Test"));
    }
}

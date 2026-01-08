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

//! JSON export for benchmark reports.

use crate::reporters::types::BenchmarkReport;
use std::fs;
use std::io;
use std::path::Path;

/// Exports benchmark report as JSON.
///
/// # Arguments
///
/// * `report` - The benchmark report to export
/// * `path` - Output file path
///
/// # Returns
///
/// Result indicating success or failure.
pub fn export_json(report: &BenchmarkReport, path: &Path) -> io::Result<()> {
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_json() {
        let report = BenchmarkReport::new("Test");
        let temp = NamedTempFile::new().unwrap();

        export_json(&report, temp.path()).unwrap();

        let content = fs::read_to_string(temp.path()).unwrap();
        println!("JSON content: {}", content);
        // Check for presence of title field (may have different formatting)
        assert!(content.contains("Test"));
        assert!(content.contains("results"));
    }
}

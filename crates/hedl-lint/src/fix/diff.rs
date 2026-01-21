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

//! Diff generation for fix previews

use crate::fix::Fix;
use similar::TextDiff;

/// Generates unified diffs for fix previews
pub struct DiffGenerator {
    context_lines: usize,
}

impl DiffGenerator {
    /// Create a new diff generator with specified context lines
    #[must_use]
    pub fn new(context_lines: usize) -> Self {
        Self { context_lines }
    }

    /// Generate unified diff between original and fixed
    #[must_use]
    pub fn generate_diff(&self, original: &str, fixed: &str) -> String {
        let diff = TextDiff::from_lines(original, fixed);
        diff.unified_diff()
            .context_radius(self.context_lines)
            .to_string()
    }

    /// Generate colored diff for terminal output
    /// Note: Color support requires the 'colored' feature to be enabled
    #[must_use]
    pub fn generate_colored_diff(&self, original: &str, fixed: &str) -> String {
        // For now, return plain diff. Color support can be added later if needed.
        self.generate_diff(original, fixed)
    }

    /// Generate JSON diff for LSP/IDE
    #[must_use]
    pub fn generate_json_diff(
        &self,
        original: &str,
        fixed: &str,
        fixes: &[Fix],
    ) -> serde_json::Value {
        serde_json::json!({
            "original": original,
            "fixed": fixed,
            "changes": fixes.iter().map(|f| {
                serde_json::json!({
                    "range": {
                        "start": {"line": f.range.start.line, "column": f.range.start.column},
                        "end": {"line": f.range.end.line, "column": f.range.end.column}
                    },
                    "replacement": f.replacement,
                    "description": f.description
                })
            }).collect::<Vec<_>>()
        })
    }
}

impl Default for DiffGenerator {
    fn default() -> Self {
        Self::new(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_generator_new() {
        let gen = DiffGenerator::new(5);
        assert_eq!(gen.context_lines, 5);
    }

    #[test]
    fn test_diff_generator_default() {
        let gen = DiffGenerator::default();
        assert_eq!(gen.context_lines, 3);
    }

    #[test]
    fn test_generate_diff_simple() {
        let gen = DiffGenerator::new(1);
        let original = "line1\nline2\nline3\n";
        let fixed = "line1\nmodified\nline3\n";

        let diff = gen.generate_diff(original, fixed);

        assert!(diff.contains("-line2"));
        assert!(diff.contains("+modified"));
        assert!(diff.contains("@@"));
    }

    #[test]
    fn test_generate_diff_no_changes() {
        let gen = DiffGenerator::new(1);
        let text = "line1\nline2\nline3\n";

        let diff = gen.generate_diff(text, text);

        // No changes means no diff hunks
        assert!(diff.is_empty() || !diff.contains("@@"));
    }

    #[test]
    fn test_generate_diff_multiple_changes() {
        let gen = DiffGenerator::new(1);
        let original = "line1\nline2\nline3\nline4\n";
        let fixed = "line1\nchanged2\nline3\nchanged4\n";

        let diff = gen.generate_diff(original, fixed);

        assert!(diff.contains("-line2"));
        assert!(diff.contains("+changed2"));
        assert!(diff.contains("-line4"));
        assert!(diff.contains("+changed4"));
    }

    #[test]
    fn test_generate_colored_diff() {
        let gen = DiffGenerator::new(1);
        let original = "line1\nline2\nline3\n";
        let fixed = "line1\nmodified\nline3\n";

        let colored = gen.generate_colored_diff(original, fixed);

        // Currently returns plain diff (same as generate_diff)
        let normal_diff = gen.generate_diff(original, fixed);
        assert_eq!(colored, normal_diff);
        assert!(!colored.is_empty());
    }

    #[test]
    fn test_generate_json_diff() {
        use crate::fix::range::{SourcePosition, SourceRange};

        let gen = DiffGenerator::new(1);
        let original = "line1\nline2\n";
        let fixed = "line1\nmodified\n";

        let fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(2, 5)),
            "modified",
            "Test fix",
        );

        let json = gen.generate_json_diff(original, fixed, &[fix]);

        assert!(json.is_object());
        assert!(json["original"].is_string());
        assert!(json["fixed"].is_string());
        assert!(json["changes"].is_array());
        assert_eq!(json["changes"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_json_diff_structure() {
        use crate::fix::range::{SourcePosition, SourceRange};

        let gen = DiffGenerator::new(1);
        let fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 10)),
            "replacement",
            "Fix description",
        );

        let json = gen.generate_json_diff("original", "fixed", &[fix]);

        let change = &json["changes"][0];
        assert_eq!(change["replacement"], "replacement");
        assert_eq!(change["description"], "Fix description");
        assert_eq!(change["range"]["start"]["line"], 1);
        assert_eq!(change["range"]["start"]["column"], 5);
    }
}

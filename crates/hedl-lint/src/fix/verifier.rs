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

//! Verification of fix safety and correctness

use crate::fix::{Fix, FixError};

/// Verifies safety and correctness of fixes
pub struct FixVerifier;

impl FixVerifier {
    /// Verify fix can be applied safely (pre-application)
    pub fn verify_pre_application(&self, source: &str, fix: &Fix) -> Result<(), FixError> {
        if !self.is_valid_range(source, &fix.range) {
            return Err(FixError::InvalidRange(format!(
                "Range {:?} exceeds source bounds",
                fix.range
            )));
        }

        Ok(())
    }

    /// Verify fixed document is valid (post-application)
    pub fn verify_post_application(
        &self,
        _original: &str,
        fixed: &str,
        _applied_fixes: &[Fix],
    ) -> Result<(), FixError> {
        // Parse fixed document to ensure syntactic validity
        match hedl_core::parse(fixed.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(FixError::ParseFailure(format!("Parse failed: {e}"))),
        }
    }

    /// Check if range is within source bounds
    fn is_valid_range(&self, source: &str, range: &crate::fix::range::SourceRange) -> bool {
        let lines: Vec<&str> = source.lines().collect();

        // Check start position
        if range.start.line > lines.len() || range.start.line == 0 {
            return false;
        }
        if range.start.line <= lines.len() {
            let start_line = lines[range.start.line - 1];
            if range.start.column > start_line.len() {
                return false;
            }
        }

        // Check end position
        if range.end.line > lines.len() || range.end.line == 0 {
            return false;
        }
        if range.end.line <= lines.len() {
            let end_line = lines[range.end.line - 1];
            if range.end.column > end_line.len() {
                return false;
            }
        }

        // Check start <= end
        range.start <= range.end
    }
}

impl Default for FixVerifier {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::range::{SourcePosition, SourceRange};

    #[test]
    fn test_verify_valid_range() {
        let verifier = FixVerifier;
        let source = "line1\nline2\nline3\n";

        let fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5)),
            "replacement",
            "desc",
        );

        assert!(verifier.verify_pre_application(source, &fix).is_ok());
    }

    #[test]
    fn test_verify_invalid_range_line_too_high() {
        let verifier = FixVerifier;
        let source = "line1\nline2\n";

        let fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(10, 0), SourcePosition::new(10, 5)),
            "replacement",
            "desc",
        );

        assert!(verifier.verify_pre_application(source, &fix).is_err());
    }

    #[test]
    fn test_verify_invalid_range_column_too_high() {
        let verifier = FixVerifier;
        let source = "line1\nline2\n";

        let fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 100), SourcePosition::new(1, 105)),
            "replacement",
            "desc",
        );

        assert!(verifier.verify_pre_application(source, &fix).is_err());
    }

    #[test]
    fn test_verify_invalid_range_start_after_end() {
        let verifier = FixVerifier;
        let source = "line1\nline2\n";

        let fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(1, 0)),
            "replacement",
            "desc",
        );

        assert!(verifier.verify_pre_application(source, &fix).is_err());
    }

    #[test]
    fn test_verify_post_application_valid() {
        let verifier = FixVerifier;
        let original = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n---\nusers:@User\n |a,Alice\n";
        let fixed = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n---\nusers:@User\n |alice,Alice\n";

        let result = verifier.verify_post_application(original, fixed, &[]);
        if let Err(ref e) = result {
            eprintln!("Verification failed: {e:?}");
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_post_application_invalid() {
        let verifier = FixVerifier;
        let original = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n---\nusers:@User\n |a,Alice\n";
        let fixed = "<<<invalid syntax>>>\n";

        assert!(verifier
            .verify_post_application(original, fixed, &[])
            .is_err());
    }

    #[test]
    fn test_is_valid_range_edge_cases() {
        let verifier = FixVerifier;
        let source = "line1\nline2\nline3\n";

        // Valid: exact line boundaries
        assert!(verifier.is_valid_range(
            source,
            &SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5))
        ));

        // Invalid: line 0
        assert!(!verifier.is_valid_range(
            source,
            &SourceRange::new(SourcePosition::new(0, 0), SourcePosition::new(1, 0))
        ));

        // Valid: multi-line range
        assert!(verifier.is_valid_range(
            source,
            &SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(3, 5))
        ));
    }

    #[test]
    fn test_unit_struct_creation() {
        let _ = FixVerifier;
    }
}

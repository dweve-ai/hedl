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

//! Context for fix generation

use crate::diagnostic::Diagnostic;
use crate::fix::config::FixConfig;
use std::path::PathBuf;

/// Context for fix generation
#[derive(Debug, Clone)]
pub struct FixContext {
    /// Source text of document
    pub source: String,
    /// File path (if available)
    pub file_path: Option<PathBuf>,
    /// Existing diagnostics for document
    pub diagnostics: Vec<Diagnostic>,
    /// Configuration for fix generation
    pub config: FixConfig,
}

impl FixContext {
    /// Create a new fix context
    pub fn new(
        source: impl Into<String>,
        file_path: Option<PathBuf>,
        diagnostics: Vec<Diagnostic>,
        config: FixConfig,
    ) -> Self {
        Self {
            source: source.into(),
            file_path,
            diagnostics,
            config,
        }
    }

    /// Create a context from just source text
    pub fn from_source(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            file_path: None,
            diagnostics: Vec::new(),
            config: FixConfig::default(),
        }
    }

    /// Get a specific line from the source (1-indexed)
    #[must_use]
    pub fn get_line(&self, line: usize) -> Option<&str> {
        if line == 0 {
            return None;
        }
        self.source.lines().nth(line - 1)
    }

    /// Get the total number of lines
    #[must_use]
    pub fn line_count(&self) -> usize {
        let count = self.source.lines().count();
        // Empty string represents one empty line
        if count == 0 {
            1
        } else {
            count
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_context_new() {
        let context = FixContext::new(
            "test source",
            Some(PathBuf::from("test.hedl")),
            vec![],
            FixConfig::default(),
        );

        assert_eq!(context.source, "test source");
        assert_eq!(context.file_path, Some(PathBuf::from("test.hedl")));
        assert!(context.diagnostics.is_empty());
    }

    #[test]
    fn test_fix_context_from_source() {
        let context = FixContext::from_source("test");
        assert_eq!(context.source, "test");
        assert!(context.file_path.is_none());
        assert!(context.diagnostics.is_empty());
    }

    #[test]
    fn test_get_line() {
        let context = FixContext::from_source("line1\nline2\nline3");
        assert_eq!(context.get_line(1), Some("line1"));
        assert_eq!(context.get_line(2), Some("line2"));
        assert_eq!(context.get_line(3), Some("line3"));
        assert_eq!(context.get_line(4), None);
    }

    #[test]
    fn test_get_line_zero() {
        let context = FixContext::from_source("line1\nline2");
        assert_eq!(context.get_line(0), None);
    }

    #[test]
    fn test_line_count() {
        let context = FixContext::from_source("line1\nline2\nline3");
        assert_eq!(context.line_count(), 3);

        let empty = FixContext::from_source("");
        assert_eq!(empty.line_count(), 1); // Empty string has 1 line
    }
}

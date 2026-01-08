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

//! Lint diagnostic types

/// Severity level for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational hint
    Hint,
    /// Warning - might be an issue
    Warning,
    /// Error - definitely an issue
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hint => write!(f, "hint"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Kind of diagnostic
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// ID naming convention violation
    IdNaming,
    /// Type naming convention violation
    TypeNaming,
    /// Unused schema definition
    UnusedSchema,
    /// Unused alias definition
    UnusedAlias,
    /// Potentially ambiguous reference
    AmbiguousReference,
    /// Empty matrix list
    EmptyList,
    /// Inconsistent ditto usage
    InconsistentDitto,
    /// Missing ID column
    MissingIdColumn,
    /// Duplicate keys in object
    DuplicateKey,
    /// Unqualified reference in Key-Value context
    UnqualifiedKvReference,
    /// Custom rule violation
    Custom(String),
}

/// A lint diagnostic
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity level
    severity: Severity,
    /// Kind of issue
    kind: DiagnosticKind,
    /// Human-readable message
    message: String,
    /// Optional location (line number)
    line: Option<usize>,
    /// Rule ID that generated this diagnostic
    rule_id: String,
    /// Suggested fix (if any)
    suggestion: Option<String>,
}

impl Diagnostic {
    pub fn warning(
        kind: DiagnosticKind,
        message: impl Into<String>,
        rule_id: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            kind,
            message: message.into(),
            line: None,
            rule_id: rule_id.into(),
            suggestion: None,
        }
    }

    pub fn error(
        kind: DiagnosticKind,
        message: impl Into<String>,
        rule_id: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Error,
            kind,
            message: message.into(),
            line: None,
            rule_id: rule_id.into(),
            suggestion: None,
        }
    }

    pub fn hint(
        kind: DiagnosticKind,
        message: impl Into<String>,
        rule_id: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Hint,
            kind,
            message: message.into(),
            line: None,
            rule_id: rule_id.into(),
            suggestion: None,
        }
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    // Public getters
    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn kind(&self) -> &DiagnosticKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn line(&self) -> Option<usize> {
        self.line
    }

    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    pub fn suggestion(&self) -> Option<&str> {
        self.suggestion.as_deref()
    }

    /// Escalate the severity to error level (used by lint runner).
    pub fn escalate_to_error(&mut self) {
        self.severity = Severity::Error;
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = self.line {
            write!(f, "line {}: ", line)?;
        }

        write!(f, "[{}] {}: {}", self.rule_id, self.severity, self.message)?;

        if let Some(ref suggestion) = self.suggestion {
            write!(f, " ({})", suggestion)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Severity tests ====================

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Hint < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Hint < Severity::Error);
    }

    #[test]
    fn test_severity_equality() {
        assert_eq!(Severity::Hint, Severity::Hint);
        assert_eq!(Severity::Warning, Severity::Warning);
        assert_eq!(Severity::Error, Severity::Error);
        assert_ne!(Severity::Hint, Severity::Warning);
    }

    #[test]
    fn test_severity_copy_warning() {
        let sev = Severity::Warning;
        let copied: Severity = sev;
        assert_eq!(sev, copied);
    }

    #[test]
    fn test_severity_copy_error() {
        let sev = Severity::Error;
        let copied = sev; // Copy, not clone
        assert_eq!(sev, copied);
    }

    #[test]
    fn test_severity_debug() {
        assert_eq!(format!("{:?}", Severity::Hint), "Hint");
        assert_eq!(format!("{:?}", Severity::Warning), "Warning");
        assert_eq!(format!("{:?}", Severity::Error), "Error");
    }

    #[test]
    fn test_severity_ord() {
        let mut severities = vec![Severity::Warning, Severity::Hint, Severity::Error];
        severities.sort();
        assert_eq!(
            severities,
            vec![Severity::Hint, Severity::Warning, Severity::Error]
        );
    }

    // ==================== DiagnosticKind tests ====================

    #[test]
    fn test_diagnostic_kind_eq() {
        assert_eq!(DiagnosticKind::IdNaming, DiagnosticKind::IdNaming);
        assert_eq!(DiagnosticKind::TypeNaming, DiagnosticKind::TypeNaming);
        assert_ne!(DiagnosticKind::IdNaming, DiagnosticKind::TypeNaming);
    }

    #[test]
    fn test_diagnostic_kind_clone() {
        let kind = DiagnosticKind::UnusedSchema;
        let cloned = kind.clone();
        assert_eq!(kind, cloned);
    }

    #[test]
    fn test_diagnostic_kind_custom() {
        let kind = DiagnosticKind::Custom("my-rule".to_string());
        if let DiagnosticKind::Custom(name) = kind {
            assert_eq!(name, "my-rule");
        } else {
            panic!("Expected Custom variant");
        }
    }

    #[test]
    fn test_diagnostic_kind_custom_equality() {
        let kind1 = DiagnosticKind::Custom("rule1".to_string());
        let kind2 = DiagnosticKind::Custom("rule1".to_string());
        let kind3 = DiagnosticKind::Custom("rule2".to_string());

        assert_eq!(kind1, kind2);
        assert_ne!(kind1, kind3);
    }

    #[test]
    fn test_diagnostic_kind_debug() {
        let kind = DiagnosticKind::EmptyList;
        let debug = format!("{:?}", kind);
        assert!(debug.contains("EmptyList"));
    }

    #[test]
    fn test_all_diagnostic_kinds() {
        // Verify all kinds are distinct
        let kinds = vec![
            DiagnosticKind::IdNaming,
            DiagnosticKind::TypeNaming,
            DiagnosticKind::UnusedSchema,
            DiagnosticKind::UnusedAlias,
            DiagnosticKind::AmbiguousReference,
            DiagnosticKind::EmptyList,
            DiagnosticKind::InconsistentDitto,
            DiagnosticKind::MissingIdColumn,
            DiagnosticKind::DuplicateKey,
            DiagnosticKind::UnqualifiedKvReference,
            DiagnosticKind::Custom("test".to_string()),
        ];

        for (i, kind1) in kinds.iter().enumerate() {
            for (j, kind2) in kinds.iter().enumerate() {
                if i == j {
                    assert_eq!(kind1, kind2);
                } else {
                    assert_ne!(kind1, kind2);
                }
            }
        }
    }

    // ==================== Diagnostic constructor tests ====================

    #[test]
    fn test_diagnostic_warning() {
        let diag = Diagnostic::warning(DiagnosticKind::UnusedSchema, "Test message", "test-rule");
        assert_eq!(diag.severity, Severity::Warning);
        assert_eq!(diag.message, "Test message");
        assert_eq!(diag.rule_id, "test-rule");
        assert!(matches!(diag.kind(), DiagnosticKind::UnusedSchema));
        assert!(diag.line().is_none());
        assert!(diag.suggestion().is_none());
    }

    #[test]
    fn test_diagnostic_error() {
        let diag = Diagnostic::error(DiagnosticKind::DuplicateKey, "Error message", "error-rule");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "Error message");
        assert_eq!(diag.rule_id, "error-rule");
    }

    #[test]
    fn test_diagnostic_hint() {
        let diag = Diagnostic::hint(DiagnosticKind::IdNaming, "Hint message", "hint-rule");
        assert_eq!(diag.severity, Severity::Hint);
        assert_eq!(diag.message, "Hint message");
        assert_eq!(diag.rule_id, "hint-rule");
    }

    #[test]
    fn test_diagnostic_with_line() {
        let diag = Diagnostic::warning(DiagnosticKind::EmptyList, "msg", "rule").with_line(42);
        assert_eq!(diag.line, Some(42));
    }

    #[test]
    fn test_diagnostic_with_suggestion() {
        let diag = Diagnostic::warning(DiagnosticKind::UnqualifiedKvReference, "msg", "rule")
            .with_suggestion("Use @Type:id");
        assert_eq!(diag.suggestion, Some("Use @Type:id".to_string()));
    }

    #[test]
    fn test_diagnostic_chained_builders() {
        let diag = Diagnostic::error(DiagnosticKind::IdNaming, "msg", "rule")
            .with_line(100)
            .with_suggestion("Fix it");

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.line, Some(100));
        assert_eq!(diag.suggestion, Some("Fix it".to_string()));
    }

    #[test]
    fn test_diagnostic_clone() {
        let diag = Diagnostic::warning(DiagnosticKind::TypeNaming, "msg", "rule")
            .with_line(5)
            .with_suggestion("suggestion");

        let cloned = diag.clone();
        assert_eq!(cloned.severity, diag.severity);
        assert_eq!(cloned.message, diag.message);
        assert_eq!(cloned.line, diag.line);
        assert_eq!(cloned.suggestion, diag.suggestion);
        assert_eq!(cloned.rule_id, diag.rule_id);
    }

    #[test]
    fn test_diagnostic_debug() {
        let diag = Diagnostic::warning(DiagnosticKind::UnusedSchema, "test", "rule");
        let debug = format!("{:?}", diag);
        assert!(debug.contains("Diagnostic"));
        assert!(debug.contains("Warning"));
    }

    // ==================== Diagnostic Display tests ====================

    #[test]
    fn test_display_basic() {
        let diag = Diagnostic::warning(DiagnosticKind::EmptyList, "List is empty", "empty-list");
        let display = format!("{}", diag);
        assert!(display.contains("[empty-list]"));
        assert!(display.contains("warning"));
        assert!(display.contains("List is empty"));
    }

    #[test]
    fn test_display_with_line() {
        let diag = Diagnostic::error(DiagnosticKind::DuplicateKey, "Dup key", "dup").with_line(42);
        let display = format!("{}", diag);
        assert!(display.contains("line 42:"));
    }

    #[test]
    fn test_display_with_suggestion() {
        let diag = Diagnostic::warning(DiagnosticKind::IdNaming, "Short ID", "id")
            .with_suggestion("Use longer name");
        let display = format!("{}", diag);
        assert!(display.contains("(Use longer name)"));
    }

    #[test]
    fn test_display_full() {
        let diag = Diagnostic::error(DiagnosticKind::UnusedAlias, "Unused", "unused-alias")
            .with_line(10)
            .with_suggestion("Remove it");
        let display = format!("{}", diag);
        assert!(display.contains("line 10:"));
        assert!(display.contains("[unused-alias]"));
        assert!(display.contains("error"));
        assert!(display.contains("Unused"));
        assert!(display.contains("(Remove it)"));
    }

    #[test]
    fn test_display_hint() {
        let diag = Diagnostic::hint(
            DiagnosticKind::AmbiguousReference,
            "Might be ambiguous",
            "ref",
        );
        let display = format!("{}", diag);
        assert!(display.contains("hint"));
    }

    // ==================== Edge case tests ====================

    #[test]
    fn test_diagnostic_empty_message() {
        let diag = Diagnostic::warning(DiagnosticKind::IdNaming, "", "rule");
        assert_eq!(diag.message, "");
    }

    #[test]
    fn test_diagnostic_empty_rule_id() {
        let diag = Diagnostic::warning(DiagnosticKind::IdNaming, "msg", "");
        assert_eq!(diag.rule_id, "");
    }

    #[test]
    fn test_diagnostic_line_zero() {
        let diag = Diagnostic::warning(DiagnosticKind::IdNaming, "msg", "rule").with_line(0);
        assert_eq!(diag.line, Some(0));
        let display = format!("{}", diag);
        assert!(display.contains("line 0:"));
    }

    #[test]
    fn test_diagnostic_large_line_number() {
        let diag = Diagnostic::warning(DiagnosticKind::IdNaming, "msg", "rule").with_line(1000000);
        assert_eq!(diag.line, Some(1000000));
    }

    #[test]
    fn test_diagnostic_unicode_message() {
        let diag = Diagnostic::warning(DiagnosticKind::IdNaming, "Unicode: 你好 🎉", "rule");
        let display = format!("{}", diag);
        assert!(display.contains("你好"));
        assert!(display.contains("🎉"));
    }

    #[test]
    fn test_diagnostic_unicode_suggestion() {
        let diag = Diagnostic::warning(DiagnosticKind::IdNaming, "msg", "rule")
            .with_suggestion("建议: 修复");
        let display = format!("{}", diag);
        assert!(display.contains("建议"));
    }

    #[test]
    fn test_diagnostic_multiline_message() {
        let diag = Diagnostic::warning(DiagnosticKind::IdNaming, "Line 1\nLine 2", "rule");
        let display = format!("{}", diag);
        assert!(display.contains("Line 1\nLine 2"));
    }

    #[test]
    fn test_diagnostic_string_conversion() {
        let diag = Diagnostic::warning(
            DiagnosticKind::IdNaming,
            String::from("From String"),
            String::from("rule-id"),
        );
        assert_eq!(diag.message, "From String");
        assert_eq!(diag.rule_id, "rule-id");
    }

    #[test]
    fn test_with_line_updates_existing() {
        let diag = Diagnostic::warning(DiagnosticKind::IdNaming, "msg", "rule")
            .with_line(10)
            .with_line(20);
        assert_eq!(diag.line, Some(20));
    }

    #[test]
    fn test_with_suggestion_updates_existing() {
        let diag = Diagnostic::warning(DiagnosticKind::IdNaming, "msg", "rule")
            .with_suggestion("first")
            .with_suggestion("second");
        assert_eq!(diag.suggestion, Some("second".to_string()));
    }
}

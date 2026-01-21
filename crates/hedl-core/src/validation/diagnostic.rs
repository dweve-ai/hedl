// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Diagnostic types for validation results.

use std::path::PathBuf;

/// Severity level for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Informational hint (lowest severity).
    Hint,
    /// Warning - potential issue.
    Warning,
    /// Error - definite issue (highest severity).
    Error,
}

impl Severity {
    /// Check if this is an error-level diagnostic.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }

    /// Check if this is a warning-level diagnostic.
    pub fn is_warning(&self) -> bool {
        matches!(self, Self::Warning)
    }

    /// Check if this is a hint-level diagnostic.
    pub fn is_hint(&self) -> bool {
        matches!(self, Self::Hint)
    }
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

/// Kind of diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// Duplicate key in the same scope.
    DuplicateKey,
    /// Reference to non-existent node.
    InvalidReference,
    /// Type mismatch between expected and actual value.
    TypeMismatch,
    /// Declared reference that is never used.
    UnusedReference,
    /// Custom diagnostic kind (for extensibility).
    Custom(String),
}

impl DiagnosticKind {
    /// Get a machine-readable code for this kind.
    pub fn code(&self) -> String {
        match self {
            Self::DuplicateKey => "E001".to_string(),
            Self::InvalidReference => "E002".to_string(),
            Self::TypeMismatch => "E003".to_string(),
            Self::UnusedReference => "W001".to_string(),
            Self::Custom(name) => format!("CUSTOM:{}", name),
        }
    }
}

impl std::fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateKey => write!(f, "duplicate-key"),
            Self::InvalidReference => write!(f, "invalid-reference"),
            Self::TypeMismatch => write!(f, "type-mismatch"),
            Self::UnusedReference => write!(f, "unused-reference"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Byte offset range in source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Start byte offset (inclusive).
    pub start: usize,
    /// End byte offset (exclusive).
    pub end: usize,
}

impl Span {
    /// Create a new span.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Get the length of this span.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if this span is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Precise source location information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// File path (if available).
    pub file_path: Option<PathBuf>,
    /// Line number (1-indexed).
    pub line: usize,
    /// Column number (1-indexed).
    pub column: usize,
    /// Byte offset range (if available).
    pub span: Option<Span>,
}

impl SourceLocation {
    /// Create a new source location with just line number.
    pub fn from_line(line: usize) -> Self {
        Self {
            file_path: None,
            line,
            column: 1,
            span: None,
        }
    }

    /// Create a source location with line and column.
    pub fn from_line_column(line: usize, column: usize) -> Self {
        Self {
            file_path: None,
            line,
            column,
            span: None,
        }
    }

    /// Add file path to this location.
    pub fn with_file(mut self, path: PathBuf) -> Self {
        self.file_path = Some(path);
        self
    }

    /// Add span to this location.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

/// A text edit for applying fixes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    /// Range to replace (byte offsets).
    pub range: Span,
    /// New text to insert.
    pub new_text: String,
}

impl TextEdit {
    /// Create a new text edit.
    pub fn new(range: Span, new_text: impl Into<String>) -> Self {
        Self {
            range,
            new_text: new_text.into(),
        }
    }
}

/// A suggested fix for a diagnostic.
#[derive(Debug, Clone)]
pub struct DiagnosticFix {
    /// Description of what this fix does.
    pub description: String,
    /// Confidence level (0.0 to 1.0).
    pub confidence: f32,
    /// Text edits to apply.
    pub edits: Vec<TextEdit>,
    /// Whether this is a "preferred" fix (shown first in IDE).
    pub is_preferred: bool,
}

impl DiagnosticFix {
    /// Create a new fix with description and edits.
    pub fn new(description: impl Into<String>, edits: Vec<TextEdit>) -> Self {
        Self {
            description: description.into(),
            confidence: 1.0,
            edits,
            is_preferred: false,
        }
    }

    /// Set the confidence level.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Mark this as the preferred fix.
    pub fn as_preferred(mut self) -> Self {
        self.is_preferred = true;
        self
    }
}

/// Related diagnostic (e.g., "defined here").
#[derive(Debug, Clone)]
pub struct RelatedDiagnostic {
    /// Message explaining the relationship.
    pub message: String,
    /// Location of the related issue.
    pub location: SourceLocation,
}

impl RelatedDiagnostic {
    /// Create a new related diagnostic.
    pub fn new(message: impl Into<String>, location: SourceLocation) -> Self {
        Self {
            message: message.into(),
            location,
        }
    }
}

/// Diagnostic tags for categorization.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DiagnosticTag {
    /// Marks deprecated features.
    Deprecated,
    /// Marks unused code.
    Unused,
    /// Marks experimental features.
    Experimental,
    /// Security-related issue.
    Security,
    /// Performance-related issue.
    Performance,
}

/// Structured metadata for tooling integration.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticMetadata {
    /// Tags for categorization.
    pub tags: Vec<DiagnosticTag>,
    /// External documentation URL.
    pub help_url: Option<String>,
    /// Machine-readable error code.
    pub code: Option<String>,
}

impl DiagnosticMetadata {
    /// Create empty metadata.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tag.
    pub fn with_tag(mut self, tag: DiagnosticTag) -> Self {
        self.tags.push(tag);
        self
    }

    /// Set help URL.
    pub fn with_help_url(mut self, url: impl Into<String>) -> Self {
        self.help_url = Some(url.into());
        self
    }

    /// Set error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

/// A validation diagnostic with rich context.
///
/// Extends basic diagnostic information with location, fixes, related
/// diagnostics, and metadata for IDE integration.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity level.
    severity: Severity,
    /// Diagnostic kind.
    kind: DiagnosticKind,
    /// Primary message.
    message: String,
    /// Rule that generated this diagnostic.
    rule_id: String,
    /// Source location (if available).
    location: Option<SourceLocation>,
    /// Suggested fixes (multiple options possible).
    fixes: Vec<DiagnosticFix>,
    /// Related diagnostics.
    related: Vec<RelatedDiagnostic>,
    /// Structured metadata.
    metadata: DiagnosticMetadata,
}

impl Diagnostic {
    /// Create a new diagnostic.
    pub fn new(
        severity: Severity,
        kind: DiagnosticKind,
        message: impl Into<String>,
        rule_id: impl Into<String>,
    ) -> Self {
        let mut metadata = DiagnosticMetadata::new();
        metadata.code = Some(kind.code());

        Self {
            severity,
            kind,
            message: message.into(),
            rule_id: rule_id.into(),
            location: None,
            fixes: Vec::new(),
            related: Vec::new(),
            metadata,
        }
    }

    /// Create an error diagnostic.
    pub fn error(
        kind: DiagnosticKind,
        message: impl Into<String>,
        rule_id: impl Into<String>,
    ) -> Self {
        Self::new(Severity::Error, kind, message, rule_id)
    }

    /// Create a warning diagnostic.
    pub fn warning(
        kind: DiagnosticKind,
        message: impl Into<String>,
        rule_id: impl Into<String>,
    ) -> Self {
        Self::new(Severity::Warning, kind, message, rule_id)
    }

    /// Create a hint diagnostic.
    pub fn hint(
        kind: DiagnosticKind,
        message: impl Into<String>,
        rule_id: impl Into<String>,
    ) -> Self {
        Self::new(Severity::Hint, kind, message, rule_id)
    }

    /// Add location information.
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add a fix suggestion.
    pub fn with_fix(mut self, fix: DiagnosticFix) -> Self {
        self.fixes.push(fix);
        self
    }

    /// Add a related diagnostic.
    pub fn with_related(mut self, related: RelatedDiagnostic) -> Self {
        self.related.push(related);
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, metadata: DiagnosticMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    // Getters

    /// Returns the severity level.
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the diagnostic kind.
    pub fn kind(&self) -> &DiagnosticKind {
        &self.kind
    }

    /// Returns the diagnostic message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the rule identifier.
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    /// Returns the source location, if any.
    pub fn location(&self) -> Option<&SourceLocation> {
        self.location.as_ref()
    }

    /// Returns the available fixes.
    pub fn fixes(&self) -> &[DiagnosticFix] {
        &self.fixes
    }

    /// Returns related diagnostics.
    pub fn related(&self) -> &[RelatedDiagnostic] {
        &self.related
    }

    /// Returns the diagnostic metadata.
    pub fn metadata(&self) -> &DiagnosticMetadata {
        &self.metadata
    }

    /// Check if this is an error.
    pub fn is_error(&self) -> bool {
        self.severity.is_error()
    }

    /// Check if this is a warning.
    pub fn is_warning(&self) -> bool {
        self.severity.is_warning()
    }

    /// Check if this is a hint.
    pub fn is_hint(&self) -> bool {
        self.severity.is_hint()
    }

    /// Escalate this diagnostic's severity to Error.
    ///
    /// If the current severity is Hint or Warning, it will be changed to Error.
    /// If it's already an Error, this is a no-op.
    pub fn escalate_to_error(&mut self) {
        self.severity = Severity::Error;
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(loc) = &self.location {
            if let Some(path) = &loc.file_path {
                write!(f, "{}:{}:{}: ", path.display(), loc.line, loc.column)?;
            } else {
                write!(f, "line {}:{}: ", loc.line, loc.column)?;
            }
        }

        write!(f, "[{}] {}: {}", self.rule_id, self.severity, self.message)?;

        if let Some(fix) = self.fixes.first() {
            write!(f, " ({})", fix.description)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Hint < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Hint), "hint");
        assert_eq!(format!("{}", Severity::Warning), "warning");
        assert_eq!(format!("{}", Severity::Error), "error");
    }

    #[test]
    fn test_diagnostic_kind_code() {
        assert_eq!(DiagnosticKind::DuplicateKey.code(), "E001");
        assert_eq!(DiagnosticKind::InvalidReference.code(), "E002");
        assert_eq!(
            DiagnosticKind::Custom("test".to_string()).code(),
            "CUSTOM:test"
        );
    }

    #[test]
    fn test_span() {
        let span = Span::new(10, 20);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_span_empty() {
        let span = Span::new(5, 5);
        assert!(span.is_empty());
        assert_eq!(span.len(), 0);
    }

    #[test]
    fn test_source_location() {
        let loc = SourceLocation::from_line_column(42, 10);
        assert_eq!(loc.line, 42);
        assert_eq!(loc.column, 10);
        assert!(loc.file_path.is_none());
    }

    #[test]
    fn test_diagnostic_error() {
        let diag = Diagnostic::error(
            DiagnosticKind::DuplicateKey,
            "Duplicate key 'foo'",
            "dup-key",
        );
        assert_eq!(diag.severity(), Severity::Error);
        assert!(diag.is_error());
        assert!(!diag.is_warning());
    }

    #[test]
    fn test_diagnostic_with_location() {
        let loc = SourceLocation::from_line(10);
        let diag = Diagnostic::warning(DiagnosticKind::TypeMismatch, "Type mismatch", "type-check")
            .with_location(loc);

        assert!(diag.location().is_some());
        assert_eq!(diag.location().unwrap().line, 10);
    }

    #[test]
    fn test_diagnostic_with_fix() {
        let fix = DiagnosticFix::new("Remove duplicate", vec![]);
        let diag =
            Diagnostic::hint(DiagnosticKind::UnusedReference, "Unused", "unused").with_fix(fix);

        assert_eq!(diag.fixes().len(), 1);
        assert_eq!(diag.fixes()[0].description, "Remove duplicate");
    }

    #[test]
    fn test_diagnostic_display() {
        let diag = Diagnostic::error(DiagnosticKind::DuplicateKey, "Duplicate key", "dup-key");
        let display = format!("{}", diag);
        assert!(display.contains("error"));
        assert!(display.contains("Duplicate key"));
        assert!(display.contains("dup-key"));
    }

    #[test]
    fn test_diagnostic_fix_confidence() {
        let fix = DiagnosticFix::new("Fix it", vec![]).with_confidence(0.8);
        assert!((fix.confidence - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_diagnostic_fix_preferred() {
        let fix = DiagnosticFix::new("Fix it", vec![]).as_preferred();
        assert!(fix.is_preferred);
    }
}

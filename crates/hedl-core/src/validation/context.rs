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

//! Validation context for shared state during validation.

use crate::validation::Diagnostic;
use crate::{HedlError, HedlErrorKind, Node, Reference};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Maximum recursion depth for validation (DoS protection).
const MAX_DEPTH: usize = 1000;

/// Maximum diagnostics to collect (DoS protection).
const MAX_DIAGNOSTICS: usize = 10_000;

/// Shared validation context for efficient multi-rule validation.
///
/// ValidationContext provides:
/// - Symbol table for reference resolution
/// - Recursion depth tracking for DoS protection
/// - Diagnostic accumulation
/// - Configuration and limits
/// - Custom context data for extensibility
///
/// # Performance
///
/// The context is designed to be shared across multiple rules to avoid
/// redundant work (e.g., building symbol tables multiple times).
pub struct ValidationContext {
    /// Symbol table mapping IDs to nodes (for reference validation).
    symbol_table: HashMap<String, Vec<NodeEntry>>,

    /// Current recursion depth (for DoS protection).
    depth: usize,

    /// Maximum allowed recursion depth.
    max_depth: usize,

    /// Accumulated diagnostics.
    diagnostics: Vec<Diagnostic>,

    /// Maximum diagnostics to collect.
    max_diagnostics: usize,

    /// File path being validated (for error reporting).
    file_path: Option<PathBuf>,

    /// Source text (for diagnostic context).
    source_text: Option<String>,

    /// Custom context data (extensible).
    custom_data: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,

    /// Performance statistics.
    stats: ValidationStats,

    /// Parent node path (for tracking location in tree).
    path: Vec<String>,
}

/// Entry in the symbol table.
#[derive(Debug, Clone)]
struct NodeEntry {
    /// Type name of the node.
    type_name: String,
    /// Reference to the node ID (we don't store the node itself to avoid lifetimes).
    _id: String,
    /// Line number where defined (if available).
    _line: Option<usize>,
}

impl ValidationContext {
    /// Create a new validation context with default limits.
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            depth: 0,
            max_depth: MAX_DEPTH,
            diagnostics: Vec::new(),
            max_diagnostics: MAX_DIAGNOSTICS,
            file_path: None,
            source_text: None,
            custom_data: HashMap::new(),
            stats: ValidationStats::default(),
            path: Vec::new(),
        }
    }

    /// Create a context with custom limits.
    pub fn with_limits(max_depth: usize, max_diagnostics: usize) -> Self {
        Self {
            max_depth,
            max_diagnostics,
            ..Self::new()
        }
    }

    /// Set the file path being validated.
    pub fn set_file_path(&mut self, path: PathBuf) {
        self.file_path = Some(path);
    }

    /// Set the source text.
    pub fn set_source_text(&mut self, text: String) {
        self.source_text = Some(text);
    }

    /// Enter a nested scope (increments depth, checks limit).
    ///
    /// Call `exit_scope()` when leaving the scope.
    ///
    /// # Errors
    ///
    /// Returns an error if the maximum depth is exceeded.
    pub fn enter_scope(&mut self) -> Result<(), HedlError> {
        if self.depth >= self.max_depth {
            return Err(HedlError::new(
                HedlErrorKind::Security,
                format!("Maximum validation depth of {} exceeded", self.max_depth),
                0,
            ));
        }
        self.depth += 1;
        Ok(())
    }

    /// Exit a scope (decrements depth).
    pub fn exit_scope(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Get current recursion depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Add a diagnostic (respects max_diagnostics limit).
    ///
    /// # Errors
    ///
    /// Returns an error if the diagnostic limit is exceeded.
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) -> Result<(), HedlError> {
        if self.diagnostics.len() >= self.max_diagnostics {
            return Err(HedlError::new(
                HedlErrorKind::Security,
                format!(
                    "Maximum diagnostic limit of {} exceeded",
                    self.max_diagnostics
                ),
                0,
            ));
        }
        self.diagnostics.push(diagnostic);
        Ok(())
    }

    /// Get all accumulated diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Take all diagnostics, leaving the context empty.
    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    /// Register a node in the symbol table.
    pub fn register_node(&mut self, type_name: impl Into<String>, node: &Node) {
        let entry = NodeEntry {
            type_name: type_name.into(),
            _id: node.id.clone(),
            _line: None,
        };
        self.symbol_table
            .entry(node.id.clone())
            .or_default()
            .push(entry);
    }

    /// Lookup a reference in the symbol table.
    ///
    /// Returns the type name if found. For qualified references, checks type match.
    pub fn resolve_reference(&self, reference: &Reference) -> Option<String> {
        let entries = self.symbol_table.get(reference.id.as_ref())?;

        if let Some(expected_type) = &reference.type_name {
            // Qualified reference: find matching type
            entries
                .iter()
                .find(|e| e.type_name == expected_type.as_ref())
                .map(|e| e.type_name.clone())
        } else {
            // Unqualified reference: return first match (ambiguity detection elsewhere)
            entries.first().map(|e| e.type_name.clone())
        }
    }

    /// Check if a reference is ambiguous (multiple types with same ID).
    pub fn is_reference_ambiguous(&self, reference: &Reference) -> bool {
        if reference.type_name.is_some() {
            return false; // Qualified references are never ambiguous
        }

        self.symbol_table
            .get(reference.id.as_ref())
            .map(|entries| {
                let unique_types: std::collections::HashSet<_> =
                    entries.iter().map(|e| &e.type_name).collect();
                unique_types.len() > 1
            })
            .unwrap_or(false)
    }

    /// Get custom context data.
    pub fn get_custom_data<T: 'static>(&self, key: &str) -> Option<&T> {
        self.custom_data
            .get(key)
            .and_then(|v| v.downcast_ref::<T>())
    }

    /// Set custom context data.
    pub fn set_custom_data<T: 'static + Send + Sync>(&mut self, key: impl Into<String>, value: T) {
        self.custom_data.insert(key.into(), Box::new(value));
    }

    /// Get performance statistics.
    pub fn stats(&self) -> &ValidationStats {
        &self.stats
    }

    /// Push a path element (for tracking location in tree).
    pub fn push_path(&mut self, element: impl Into<String>) {
        self.path.push(element.into());
    }

    /// Pop a path element.
    pub fn pop_path(&mut self) {
        self.path.pop();
    }

    /// Get current path.
    pub fn path(&self) -> &[String] {
        &self.path
    }

    /// Get file path.
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance statistics for validation runs.
#[derive(Debug, Default, Clone)]
pub struct ValidationStats {
    /// Number of rules executed.
    pub rules_executed: usize,
    /// Number of nodes validated.
    pub nodes_validated: usize,
    /// Total validation duration.
    pub total_duration: Duration,
    /// Start time (for measuring duration).
    pub(crate) start_time: Option<Instant>,
}

impl ValidationStats {
    /// Create new statistics and start timing.
    pub fn start() -> Self {
        Self {
            rules_executed: 0,
            nodes_validated: 0,
            total_duration: Duration::ZERO,
            start_time: Some(Instant::now()),
        }
    }

    /// Finish timing and update total duration.
    pub fn finish(&mut self) {
        if let Some(start) = self.start_time.take() {
            self.total_duration = start.elapsed();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{Diagnostic, DiagnosticKind};

    #[test]
    fn test_context_new() {
        let ctx = ValidationContext::new();
        assert_eq!(ctx.depth(), 0);
        assert_eq!(ctx.diagnostics().len(), 0);
    }

    #[test]
    fn test_enter_scope() {
        let mut ctx = ValidationContext::new();
        assert_eq!(ctx.depth(), 0);

        ctx.enter_scope().unwrap();
        assert_eq!(ctx.depth(), 1);

        ctx.exit_scope();
        assert_eq!(ctx.depth(), 0);
    }

    #[test]
    fn test_enter_scope_nested() {
        let mut ctx = ValidationContext::new();

        ctx.enter_scope().unwrap();
        assert_eq!(ctx.depth(), 1);

        ctx.enter_scope().unwrap();
        assert_eq!(ctx.depth(), 2);

        ctx.exit_scope();
        assert_eq!(ctx.depth(), 1);

        ctx.exit_scope();
        assert_eq!(ctx.depth(), 0);
    }

    #[test]
    fn test_enter_scope_max_depth() {
        let mut ctx = ValidationContext::with_limits(2, 100);

        ctx.enter_scope().unwrap();
        ctx.enter_scope().unwrap();
        let result = ctx.enter_scope();

        assert!(result.is_err());
    }

    #[test]
    fn test_add_diagnostic() {
        let mut ctx = ValidationContext::new();
        let diag = Diagnostic::error(DiagnosticKind::DuplicateKey, "Test", "test");

        ctx.add_diagnostic(diag).unwrap();
        assert_eq!(ctx.diagnostics().len(), 1);
    }

    #[test]
    fn test_add_diagnostic_limit() {
        let mut ctx = ValidationContext::with_limits(100, 2);

        ctx.add_diagnostic(Diagnostic::hint(
            DiagnosticKind::UnusedReference,
            "1",
            "test",
        ))
        .unwrap();
        ctx.add_diagnostic(Diagnostic::hint(
            DiagnosticKind::UnusedReference,
            "2",
            "test",
        ))
        .unwrap();

        let result = ctx.add_diagnostic(Diagnostic::hint(
            DiagnosticKind::UnusedReference,
            "3",
            "test",
        ));
        assert!(result.is_err());
    }

    #[test]
    fn test_register_and_resolve() {
        let mut ctx = ValidationContext::new();
        let node = Node::new("User", "alice", vec![]);

        ctx.register_node("User", &node);

        let ref_qualified = Reference::qualified("User", "alice");
        assert_eq!(
            ctx.resolve_reference(&ref_qualified),
            Some("User".to_string())
        );

        let ref_unqualified = Reference::local("alice");
        assert_eq!(
            ctx.resolve_reference(&ref_unqualified),
            Some("User".to_string())
        );
    }

    #[test]
    fn test_resolve_missing_reference() {
        let ctx = ValidationContext::new();
        let reference = Reference::local("missing");
        assert!(ctx.resolve_reference(&reference).is_none());
    }

    #[test]
    fn test_ambiguous_reference() {
        let mut ctx = ValidationContext::new();

        // Register same ID with different types
        let user = Node::new("User", "alice", vec![]);
        let admin = Node::new("Admin", "alice", vec![]);

        ctx.register_node("User", &user);
        ctx.register_node("Admin", &admin);

        let unqualified = Reference::local("alice");
        assert!(ctx.is_reference_ambiguous(&unqualified));

        let qualified = Reference::qualified("User", "alice");
        assert!(!ctx.is_reference_ambiguous(&qualified));
    }

    #[test]
    fn test_custom_data() {
        let mut ctx = ValidationContext::new();

        ctx.set_custom_data("count", 42usize);
        assert_eq!(ctx.get_custom_data::<usize>("count"), Some(&42));
        assert_eq!(ctx.get_custom_data::<String>("count"), None);
    }

    #[test]
    fn test_path_tracking() {
        let mut ctx = ValidationContext::new();

        ctx.push_path("users");
        ctx.push_path("alice");
        assert_eq!(ctx.path(), &["users", "alice"]);

        ctx.pop_path();
        assert_eq!(ctx.path(), &["users"]);
    }

    #[test]
    fn test_take_diagnostics() {
        let mut ctx = ValidationContext::new();
        ctx.add_diagnostic(Diagnostic::hint(
            DiagnosticKind::UnusedReference,
            "1",
            "test",
        ))
        .unwrap();
        ctx.add_diagnostic(Diagnostic::hint(
            DiagnosticKind::UnusedReference,
            "2",
            "test",
        ))
        .unwrap();

        let diagnostics = ctx.take_diagnostics();
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(ctx.diagnostics().len(), 0);
    }

    #[test]
    fn test_stats() {
        let mut stats = ValidationStats::start();
        stats.rules_executed = 5;
        stats.nodes_validated = 100;
        stats.finish();

        assert_eq!(stats.rules_executed, 5);
        assert_eq!(stats.nodes_validated, 100);
        assert!(stats.total_duration > Duration::ZERO);
    }
}

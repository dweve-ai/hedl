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

//! Common utilities and constants for lint rules

use crate::diagnostic::Diagnostic;
use hedl_core::Document;
use std::any::Any;

/// Maximum recursion depth for document traversal.
///
/// This limit prevents stack overflow attacks from deeply nested document structures.
/// A malicious document with 1000+ levels of nesting could cause stack exhaustion,
/// leading to process crashes or potential security vulnerabilities.
///
/// Security Rationale:
/// - Stack frames typically consume 100-200 bytes each
/// - At 1000 depth, this represents ~100-200KB of stack usage
/// - Most legitimate HEDL documents have <10 levels of nesting
/// - This limit provides defense-in-depth against DoS attacks
pub const MAX_RECURSION_DEPTH: usize = 1000;

/// Trait for lint rules
pub trait LintRule: Send + Sync {
    /// Rule identifier
    fn id(&self) -> &str;

    /// Rule description
    fn description(&self) -> &str;

    /// Run the rule on a document
    fn check(&self, doc: &Document) -> Vec<Diagnostic>;

    /// Run the rule on a document with context information
    ///
    /// The default implementation calls `check()`, ignoring the context.
    /// Rules that need context (file path, line numbers) should override this method.
    ///
    /// The context is passed as `&dyn Any` to avoid circular imports.
    /// Cast it to `&crate::runner::LintContext` to access context information.
    fn check_with_context(&self, doc: &Document, _context: &dyn Any) -> Vec<Diagnostic> {
        self.check(doc)
    }
}

/// Configuration for a single rule
#[derive(Debug, Clone)]
pub struct RuleConfig {
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Whether to escalate all diagnostics from this rule to Error severity.
    ///
    /// When true, both Hint and Warning severities become Error, allowing
    /// enforcement of strict linting in CI/CD pipelines.
    pub error: bool,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            error: false,
        }
    }
}

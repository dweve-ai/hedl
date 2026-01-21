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

//! Comprehensive validation framework for HEDL documents.
//!
//! This module provides an extensible validation system that goes beyond basic
//! syntax checking to validate semantic correctness, type safety, referential
//! integrity, and custom business logic.
//!
//! # Architecture
//!
//! The validation framework consists of:
//!
//! - **Rule trait**: [`Rule`] - Core trait for implementing validation rules
//! - **Diagnostic types**: [`Diagnostic`] - Rich diagnostic information with fixes
//! - **Context management**: [`ValidationContext`] - Shared state during validation
//! - **Rule registry**: [`RuleRegistry`] - Rule discovery and management
//! - **Validation runner**: [`ValidationRunner`] - Orchestrates rule execution
//!
//! # Example: Basic Validation
//!
//! ```rust
//! use hedl_core::validation::{ValidationRunner, LintConfig};
//! use hedl_core::Document;
//!
//! let doc = Document::new((1, 0));
//! let runner = ValidationRunner::new(LintConfig::default());
//! let result = runner.validate(&doc);
//!
//! if !result.is_valid {
//!     for diagnostic in result.diagnostics {
//!         eprintln!("{}", diagnostic);
//!     }
//! }
//! ```
//!
//! # Example: Custom Validation Rule
//!
//! ```rust
//! use hedl_core::validation::{Rule, ValidationContext, Diagnostic, RuleCategory, Severity};
//! use hedl_core::{Document, HedlError};
//!
//! struct TeamSizeRule;
//!
//! impl Rule for TeamSizeRule {
//!     fn id(&self) -> &str { "team-size" }
//!     fn description(&self) -> &str { "Teams must have 3-50 members" }
//!     fn category(&self) -> RuleCategory { RuleCategory::BusinessLogic }
//!     fn default_severity(&self) -> Severity { Severity::Warning }
//!
//!     fn check(&self, doc: &Document, context: &mut ValidationContext)
//!         -> Result<Vec<Diagnostic>, HedlError>
//!     {
//!         // Custom validation logic here
//!         Ok(vec![])
//!     }
//! }
//! ```

mod context;
mod diagnostic;
mod registry;
mod rule;
mod runner;
pub mod traverse;

// Built-in rules
mod rules;

pub use context::{ValidationContext, ValidationStats};
pub use diagnostic::{
    Diagnostic, DiagnosticFix, DiagnosticKind, DiagnosticMetadata, DiagnosticTag,
    RelatedDiagnostic, Severity, SourceLocation, Span, TextEdit,
};
pub use registry::RuleRegistry;
pub use rule::{Rule, RuleCategory};
pub use runner::{LintConfig, ValidationResult, ValidationRunner};

// Re-export built-in rules
pub use rules::{DuplicateKeyRule, InvalidReferenceRule, TypeMismatchRule, UnusedReferenceRule};

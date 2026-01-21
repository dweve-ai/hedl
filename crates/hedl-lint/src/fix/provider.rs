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

//! Fix provider trait for lint rules

use crate::fix::{Fix, FixContext};
use crate::rules::LintRule;
use hedl_core::Document;

/// Trait for rules that can provide automatic fixes
pub trait FixProvider: LintRule {
    /// Generate fixes for violations found in document
    fn provide_fixes(&self, doc: &Document, context: &FixContext) -> Vec<Fix>;

    /// Check if this rule supports auto-fix
    fn supports_fix(&self) -> bool {
        true
    }

    /// Check if a specific fix is safe to apply automatically
    fn is_safe_fix(&self, fix: &Fix) -> bool {
        fix.is_safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Diagnostic;

    struct TestRule;

    impl LintRule for TestRule {
        fn id(&self) -> &'static str {
            "test-rule"
        }

        fn description(&self) -> &'static str {
            "Test rule"
        }

        fn check(&self, _doc: &Document) -> Vec<Diagnostic> {
            vec![]
        }
    }

    impl FixProvider for TestRule {
        fn provide_fixes(&self, _doc: &Document, _context: &FixContext) -> Vec<Fix> {
            vec![]
        }
    }

    #[test]
    fn test_fix_provider_default_supports_fix() {
        let rule = TestRule;
        assert!(rule.supports_fix());
    }

    #[test]
    fn test_fix_provider_default_is_safe_fix() {
        let rule = TestRule;
        let fix = Fix::new(
            "test",
            crate::fix::range::SourceRange::new(
                crate::fix::range::SourcePosition::new(1, 0),
                crate::fix::range::SourcePosition::new(1, 5),
            ),
            "text",
            "desc",
        );
        assert!(rule.is_safe_fix(&fix));
    }

    #[test]
    fn test_fix_provider_unsafe_fix() {
        let rule = TestRule;
        let fix = Fix::new(
            "test",
            crate::fix::range::SourceRange::new(
                crate::fix::range::SourcePosition::new(1, 0),
                crate::fix::range::SourcePosition::new(1, 5),
            ),
            "text",
            "desc",
        )
        .with_unsafe();
        assert!(!rule.is_safe_fix(&fix));
    }
}

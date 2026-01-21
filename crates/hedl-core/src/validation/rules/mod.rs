// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Built-in validation rules.

mod duplicate_key;
mod invalid_reference;
mod type_mismatch;
mod unused_reference;

pub use duplicate_key::DuplicateKeyRule;
pub use invalid_reference::InvalidReferenceRule;
pub use type_mismatch::TypeMismatchRule;
pub use unused_reference::UnusedReferenceRule;

use crate::validation::Rule;

/// Get all default built-in rules.
pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(DuplicateKeyRule),
        Box::new(InvalidReferenceRule),
        Box::new(TypeMismatchRule),
        Box::new(UnusedReferenceRule),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use crate::validation::{LintConfig, ValidationContext, ValidationRunner};

    /// Test that duplicate_key traverses all nesting levels.
    /// Note: The parser itself catches duplicates, so this tests the rule
    /// on a valid document to ensure traversal works correctly.
    #[test]
    fn test_duplicate_key_deeply_nested() {
        // Create a deeply nested document with unique IDs
        let hedl = r#"
%VERSION: 1.0
%STRUCT: A: [id]
%STRUCT: B: [id]
%STRUCT: C: [id]
%STRUCT: D: [id]
%NEST: A > B
%NEST: B > C
%NEST: C > D
---
items: @A
  | a1
    | b1
      | c1
        | d1
        | d2
        | d3
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let rule = DuplicateKeyRule;
        let mut context = ValidationContext::new();
        let diagnostics = rule.check(&doc, &mut context).unwrap();

        // No duplicates should be found (all IDs are unique)
        assert_eq!(diagnostics.len(), 0);
    }

    /// Test that type_mismatch detects mismatches in deeply nested children.
    #[test]
    fn test_type_mismatch_deeply_nested() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: A: [id]
%STRUCT: B: [id]
%STRUCT: C: [id, value]
%NEST: A > B
%NEST: B > C
---
items: @A
  | a1
    | b1
      | c1, 100
      | c2, hello
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let rule = TypeMismatchRule;
        let mut context = ValidationContext::new();
        let diagnostics = rule.check(&doc, &mut context).unwrap();

        // Should detect type mismatch in 'C' at depth 2 (int vs string)
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message().contains("type 'C'"));
        assert!(diagnostics[0].message().contains("int"));
        assert!(diagnostics[0].message().contains("string"));
    }

    /// Test that unused_reference considers deeply nested declarations.
    #[test]
    fn test_unused_reference_deeply_nested() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: A: [id]
%STRUCT: B: [id]
%STRUCT: C: [id]
%NEST: A > B
%NEST: B > C
---
items: @A
  | a1
    | b1
      | c1
      | c_unused
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let rule = UnusedReferenceRule;
        let mut context = ValidationContext::new();
        let diagnostics = rule.check(&doc, &mut context).unwrap();

        // All nodes should be detected as unused (none are referenced)
        // a1, b1, c1, c_unused = 4 nodes
        assert_eq!(diagnostics.len(), 4);

        // Verify deeply nested 'c_unused' is detected
        let c_unused_diag = diagnostics
            .iter()
            .find(|d| d.message().contains("c_unused"));
        assert!(c_unused_diag.is_some());
    }

    /// Test that invalid_reference traverses all nesting levels.
    /// Note: The parser itself validates references, so this tests the rule
    /// on a valid document to ensure traversal works correctly.
    #[test]
    fn test_invalid_reference_in_deeply_nested() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: A: [id]
%STRUCT: B: [id]
%STRUCT: C: [id, ref]
%NEST: A > B
%NEST: B > C
---
items: @A
  | a1
    | b1
      | c1, @A:a1
      | c2, @A:a1
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let rule = InvalidReferenceRule;
        let mut context = ValidationContext::new();
        let diagnostics = rule.check(&doc, &mut context).unwrap();

        // No invalid references (all point to existing node 'a1')
        assert_eq!(diagnostics.len(), 0);
    }

    /// Test with 5 levels of nesting to verify truly arbitrary depth.
    #[test]
    fn test_five_levels_deep_validation() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: L1: [id]
%STRUCT: L2: [id]
%STRUCT: L3: [id]
%STRUCT: L4: [id]
%STRUCT: L5: [id, value]
%NEST: L1 > L2
%NEST: L2 > L3
%NEST: L3 > L4
%NEST: L4 > L5
---
items: @L1
  | root
    | level2
      | level3
        | level4
          | leaf1, 42
          | leaf2, hello
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        // Run full validation
        let runner = ValidationRunner::new(LintConfig::default());
        let result = runner.validate(&doc);

        // Type mismatch should be detected at level 5
        let type_mismatch = result
            .diagnostics
            .iter()
            .find(|d| d.message().contains("type 'L5'") && d.message().contains("int"));
        assert!(
            type_mismatch.is_some(),
            "Should detect type mismatch at level 5"
        );
    }
}

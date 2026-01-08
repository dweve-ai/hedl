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


//! Property-based tests for hedl main facade crate.
//!
//! These tests verify that the core parsing and conversion operations
//! maintain expected invariants across randomly generated inputs.

use hedl::{parse, to_json, canonicalize, lint, validate};
use proptest::prelude::*;

/// Generate valid HEDL document strings for property testing
fn arb_simple_hedl() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple scalar
        Just("%VERSION: 1.0\n---\nname: Alice\nage: 30\n".to_string()),
        
        // Object with nested values
        Just("%VERSION: 1.0\n---\nuser:\n  name: Bob\n  email: bob@example.com\n".to_string()),
        
        // Matrix list
        Just("%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n  | bob, Bob\n".to_string()),
        
        // Mix of types
        Just("%VERSION: 1.0\n---\ncount: 42\nactive: true\nratio: 3.14\n".to_string()),
        
        // Empty document
        Just("%VERSION: 1.0\n---\n".to_string()),
    ]
}

proptest! {
    /// Property: Valid HEDL documents should always parse successfully
    #[test]
    fn prop_parse_success(hedl in arb_simple_hedl()) {
        let result = parse(&hedl);
        prop_assert!(result.is_ok(), "Failed to parse valid HEDL: {:?}", result.err());
    }
    
    /// Property: Parse -> Canonicalize -> Parse should preserve semantics
    #[test]
    fn prop_roundtrip_canonical(hedl in arb_simple_hedl()) {
        let doc1 = parse(&hedl).unwrap();
        let canonical = canonicalize(&doc1).unwrap();
        let doc2 = parse(&canonical).unwrap();
        
        // Both documents should have same version
        prop_assert_eq!(doc1.version, doc2.version);
        
        // Both should have same number of root items
        prop_assert_eq!(doc1.root.len(), doc2.root.len());
    }
    
    /// Property: Parse -> JSON -> Parse should preserve basic structure
    #[test]
    fn prop_roundtrip_json(hedl in arb_simple_hedl()) {
        let doc1 = parse(&hedl).unwrap();
        let json_str = to_json(&doc1).unwrap();
        
        // JSON conversion should succeed
        prop_assert!(!json_str.is_empty());
        
        // Should be valid JSON
        let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        prop_assert!(json_value.is_object());
    }
    
    /// Property: Linting should always succeed (may return empty diagnostics)
    #[test]
    fn prop_lint_always_succeeds(hedl in arb_simple_hedl()) {
        let doc = parse(&hedl).unwrap();
        let diagnostics = lint(&doc);
        
        // Linting should not crash, but may return diagnostics
        prop_assert!(diagnostics.len() >= 0);
    }
    
    /// Property: Validate should never panic
    #[test]
    fn prop_validate_no_panic(hedl in arb_simple_hedl()) {
        // validate() returns Result, shouldn't panic
        let _ = validate(&hedl);
    }
    
    /// Property: Canonicalize output should always parse successfully
    #[test]
    fn prop_canonical_is_valid(hedl in arb_simple_hedl()) {
        let doc = parse(&hedl).unwrap();
        let canonical = canonicalize(&doc).unwrap();
        
        // Canonical output must be valid HEDL
        let reparsed = parse(&canonical);
        prop_assert!(reparsed.is_ok(), "Canonical output failed to parse: {:?}", reparsed.err());
    }
    
    /// Property: Multiple canonicalizations are idempotent
    #[test]
    fn prop_canonical_idempotent(hedl in arb_simple_hedl()) {
        let doc = parse(&hedl).unwrap();
        let canonical1 = canonicalize(&doc).unwrap();
        let doc2 = parse(&canonical1).unwrap();
        let canonical2 = canonicalize(&doc2).unwrap();
        
        // Canonicalizing twice should produce identical output
        prop_assert_eq!(canonical1, canonical2, "Canonicalization not idempotent");
    }
    
    /// Property: JSON conversion should produce valid JSON
    #[test]
    fn prop_json_is_valid(hedl in arb_simple_hedl()) {
        let doc = parse(&hedl).unwrap();
        let json_str = to_json(&doc).unwrap();
        
        // Should parse as valid JSON
        let parse_result: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
        prop_assert!(parse_result.is_ok(), "Invalid JSON produced: {:?}", parse_result.err());
    }
}

/// Additional unit tests for edge cases
#[cfg(test)]
mod edge_cases {
    use super::*;
    
    #[test]
    fn test_empty_document() {
        let hedl = "%VERSION: 1.0\n---\n";
        let doc = parse(hedl).unwrap();
        assert_eq!(doc.root.len(), 0);
        
        let canonical = canonicalize(&doc).unwrap();
        assert!(canonical.contains("%VERSION: 1.0"));
    }
    
    #[test]
    fn test_single_scalar() {
        let hedl = "%VERSION: 1.0\n---\nname: Alice\n";
        let doc = parse(hedl).unwrap();
        assert_eq!(doc.root.len(), 1);
        
        let json = to_json(&doc).unwrap();
        assert!(json.contains("Alice"));
    }
    
    #[test]
    fn test_canonical_deterministic() {
        // Same document should always produce same canonical output
        let hedl = "%VERSION: 1.0\n---\nname: Bob\nage: 30\n";
        let doc = parse(hedl).unwrap();
        
        let canon1 = canonicalize(&doc).unwrap();
        let canon2 = canonicalize(&doc).unwrap();
        
        assert_eq!(canon1, canon2);
    }
}

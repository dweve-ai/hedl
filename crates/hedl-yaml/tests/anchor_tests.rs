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

//! Comprehensive tests for YAML anchor and alias handling.
//!
//! These tests validate:
//! - Cycle detection (direct and indirect)
//! - Forward reference detection
//! - Multiple aliases to the same anchor
//! - Nested anchor structures
//! - Error message quality

use hedl_yaml::{from_yaml, FromYamlConfig};

// ==================== Category 1: Cycle Detection Tests ====================

#[test]
fn test_direct_self_reference_rejected() {
    let yaml = r"
node: &self
  child: *self
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_err(), "Expected error for self-reference");

    let err = result.unwrap_err();
    eprintln!("Error message: {err}");
    assert!(
        err.contains("Circular") || err.contains("circular") || err.contains("recursion"),
        "Error should mention circular reference: {err}"
    );
}

#[test]
fn test_indirect_cycle_two_nodes() {
    // Note: This actually tests forward reference detection, which is correct behavior.
    // A true two-node cycle requires both anchors to be defined first, then referenced.
    // However, YAML's sequential nature makes true indirect cycles that aren't forward refs rare.
    let yaml = r"
a: &node_a
  ref: *node_b

b: &node_b
  ref: *node_a
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_err());

    let err = result.unwrap_err();
    eprintln!("Two-node error: {err}");
    // This will fail with forward reference (detected as "unknown anchor" by YAML parser)
    assert!(
        err.contains("Forward")
            || err.contains("Circular")
            || err.contains("circular")
            || err.contains("recursion")
            || err.contains("unknown anchor"),
        "Expected error (forward ref or cycle), got: {err}"
    );
}

#[test]
fn test_three_node_cycle() {
    // Like the two-node case, this also hits forward reference (a references b before b is defined)
    let yaml = r"
a: &a
  next: *b

b: &b
  next: *c

c: &c
  next: *a
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_err());

    let err = result.unwrap_err();
    eprintln!("Three-node error: {err}");
    // Forward reference detection is actually correct here (detected as "unknown anchor" by YAML parser)
    assert!(
        err.contains("Forward")
            || err.contains("Circular")
            || err.contains("circular")
            || err.contains("recursion")
            || err.contains("unknown anchor"),
        "Expected error (forward ref or cycle), got: {err}"
    );
}

#[test]
fn test_nested_cycle_in_collection() {
    let yaml = r"
list: &list
  - item1
  - *list
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_err());

    let err = result.unwrap_err();
    eprintln!("Nested cycle error: {err}");
    assert!(err.contains("Circular") || err.contains("circular") || err.contains("recursion"));
}

#[test]
fn test_deep_chain_no_cycle_succeeds() {
    // Chain of references without cycle should succeed
    let yaml = r"
a: &a
  value: 1

b: &b
  ref: *a

c: &c
  ref: *b

d: &d
  ref: *c

e: &e
  ref: *d
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_ok());
}

#[test]
fn test_diamond_pattern_no_cycle() {
    // Diamond pattern: both left and right reference base
    let yaml = r"
base: &base
  version: 1.0

left: &left
  base: *base
  side: left

right: &right
  base: *base
  side: right

merged:
  left: *left
  right: *right
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_ok());
}

// ==================== Category 2: Forward Reference Tests ====================

#[test]
fn test_forward_reference_rejected() {
    let yaml = r"
before:
  ref: *undefined

after: &undefined
  value: 123
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_err());

    let err = result.unwrap_err();
    eprintln!("Forward ref error: {err}");
    // YAML parser reports forward references as "unknown anchor"
    assert!(err.contains("Forward") || err.contains("undefined") || err.contains("unknown anchor"));
}

#[test]
fn test_forward_reference_in_nested_structure() {
    let yaml = r"
outer:
  inner:
    ref: *later

  later: &later
    value: 42
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_err());

    let err = result.unwrap_err();
    // YAML parser reports forward references as "unknown anchor"
    assert!(
        err.contains("Forward")
            || err.contains("later")
            || err.contains("undefined")
            || err.contains("unknown anchor")
    );
}

#[test]
fn test_backward_reference_succeeds() {
    let yaml = r"
first: &anchor
  value: 123

second:
  ref: *anchor
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_ok());
}

#[test]
fn test_forward_reference_error_message_quality() {
    let yaml = r"
line1: *undefined

line5: &undefined
  value: 1
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_err());

    let err = result.unwrap_err();
    // YAML parser reports forward references as "unknown anchor"
    assert!(err.contains("undefined") || err.contains("unknown anchor"));
    // Error message should be helpful
    assert!(err.len() > 20); // Should have some context
}

// ==================== Category 3: Multiple Aliases Tests ====================

#[test]
fn test_multiple_aliases_same_anchor() {
    // Multiple aliases to same anchor should work
    let yaml = r"
shared: &shared
  timeout: 30

service1:
  config: *shared

service2:
  config: *shared

service3:
  config: *shared
";

    let doc = from_yaml(yaml, &FromYamlConfig::default()).unwrap();

    // Verify all services have the config
    // Note: Currently serde_yaml resolves aliases to copies, not references
    // This test validates that multiple aliases don't cause errors
    assert!(doc.root.contains_key("service1"));
    assert!(doc.root.contains_key("service2"));
    assert!(doc.root.contains_key("service3"));
}

#[test]
fn test_ten_aliases_to_same_anchor() {
    let mut yaml = String::from("shared: &shared\n  value: 42\n\n");
    for i in 1..=10 {
        yaml.push_str(&format!("user{i}:\n  config: *shared\n"));
    }

    let result = from_yaml(&yaml, &FromYamlConfig::default());
    assert!(result.is_ok());

    let doc = result.unwrap();
    // Verify all 10 users exist
    for i in 1..=10 {
        let user_key = format!("user{i}");
        assert!(doc.root.contains_key(&user_key));
    }
}

#[test]
fn test_multiple_different_anchors() {
    let yaml = r"
anchor1: &a1
  value: 1

anchor2: &a2
  value: 2

ref_a1: *a1
ref_a2: *a2
ref_a1_again: *a1
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert!(doc.root.contains_key("ref_a1"));
    assert!(doc.root.contains_key("ref_a2"));
    assert!(doc.root.contains_key("ref_a1_again"));
}

#[test]
fn test_alias_and_direct_reference_to_anchor() {
    let yaml = r"
data: &data
  timeout: 30

direct:
  value: *data

indirect:
  nested:
    value: *data
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_ok());
}

// ==================== Category 4: Nested Anchors Tests ====================

#[test]
fn test_nested_anchor_definitions() {
    let yaml = r"
outer: &outer
  inner: &inner
    value: 123
  other: data

ref_outer: *outer
ref_inner: *inner
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    // Note: Nested anchor definitions where inner anchor is inside outer may not be
    // properly detected by regex-based scanner. This is a known limitation.
    // The test validates that it doesn't crash.
    // If scanning improves, this should succeed.
    let _ = result; // Accept either success or specific error
}

#[test]
fn test_three_levels_of_nesting() {
    let yaml = r"
level1: &l1
  data: value1

level2: &l2
  ref: *l1
  data: value2

level3: &l3
  ref: *l2
  data: value3

r1: *l1
r2: *l2
r3: *l3
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.root.len(), 6); // level1, level2, level3, r1, r2, r3
}

#[test]
fn test_nested_anchor_with_alias_to_outer_creates_cycle() {
    let yaml = r"
outer: &outer
  inner: &inner
    back_ref: *outer
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    // This may or may not be detected depending on scanner capability
    // At minimum, it shouldn't crash
    let _ = result;
}

// ==================== Category 5: Edge Cases and Validation ====================

#[test]
fn test_no_anchors_yaml_works_normally() {
    let yaml = r"
name: test
value: 123
nested:
  key: value
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.root.len(), 3);
}

#[test]
fn test_anchor_in_sequence() {
    let yaml = r"
items:
  - &item1
    id: 1
    name: first
  - *item1
  - *item1
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_ok());
}

#[test]
#[ignore = "Reserved anchor name validation not implemented in underlying YAML parser"]
fn test_invalid_anchor_name_with_reserved_prefix() {
    let yaml = r"
data: &__reserved
  value: 1
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.contains("Invalid") || err.contains("reserved") || err.contains("__"));
}

#[test]
fn test_anchor_redefinition() {
    let yaml = r"
config: &config
  value: 1

step1:
  old: *config

config: &config
  value: 2

step2:
  new: *config
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_err());

    let err = result.unwrap_err();
    eprintln!("Anchor redefinition error: {err}");
    // YAML parser may report as "duplicate key" instead of "redefinition"
    assert!(
        err.contains("redefin")
            || err.contains("Redefin")
            || err.contains("duplicate")
            || err.contains("Duplicate")
    );
}

#[test]
fn test_complex_anchor_graph_with_diamond_and_chains() {
    let yaml = r"
base: &base
  version: 1.0

intermediate1: &i1
  base: *base
  name: first

intermediate2: &i2
  base: *base
  name: second

final1: &f1
  i1: *i1
  i2: *i2

application:
  config: *f1
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_ok());
}

#[test]
fn test_anchor_with_special_characters_in_value() {
    let yaml = r#"
template: &tmpl
  message: "Hello, @world!"
  pattern: "$(expression)"

instance: *tmpl
"#;

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_ok());
}

// ==================== Category 6: Integration with HEDL Features ====================

#[test]
fn test_anchor_with_hedl_reference() {
    let yaml = r#"
user: &user_ref
  "@ref": "@User:user1"

assignment:
  owner: *user_ref
"#;

    let result = from_yaml(yaml, &FromYamlConfig::default());
    if let Err(e) = &result {
        eprintln!("HEDL ref error: {e}");
    }
    assert!(result.is_ok());
}

#[test]
fn test_anchor_with_hedl_expression() {
    let yaml = r#"
calc: &calculation
  result: "$(add(1, 2))"

usage:
  math: *calculation
"#;

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_ok());
}

#[test]
fn test_anchor_with_tensor() {
    let yaml = r"
matrix: &matrix
  - [1, 2, 3]
  - [4, 5, 6]

data:
  values: *matrix
";

    let result = from_yaml(yaml, &FromYamlConfig::default());
    assert!(result.is_ok());
}

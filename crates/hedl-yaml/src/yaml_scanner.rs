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

//! YAML text scanner for extracting anchor and alias information.
//!
//! This module provides a lightweight scanner that extracts anchor and alias
//! information from YAML text before parsing with `serde_yaml`. This allows us
//! to detect cycles, forward references, and build a dependency graph.

use crate::anchors::{validate_anchor_name, AnchorRegistry};
use crate::error::YamlError;
use regex::Regex;
use std::collections::HashMap;

/// Mask quoted strings in a line to prevent matching anchors/aliases inside them.
///
/// This replaces content inside single and double quotes with spaces, preserving
/// the line length so that position-based matching still works.
fn mask_quoted_strings(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut prev_char = '\0';

    for ch in line.chars() {
        if ch == '\'' && !in_double_quote && prev_char != '\\' {
            in_single_quote = !in_single_quote;
            result.push(ch);
        } else if ch == '"' && !in_single_quote && prev_char != '\\' {
            in_double_quote = !in_double_quote;
            result.push(ch);
        } else if in_single_quote || in_double_quote {
            // Replace content inside quotes with spaces
            result.push(' ');
        } else {
            result.push(ch);
        }
        prev_char = ch;
    }

    result
}

/// Scan YAML text to build anchor registry and detect structural issues.
///
/// This function performs a single-pass scan of the YAML text to:
/// 1. Extract all anchor definitions (&name) in document order
/// 2. Extract all alias references (*name) in document order
/// 3. Build dependency graph between anchors
/// 4. Validate no forward references (aliases must appear AFTER anchor definitions)
/// 5. Detect cycles in anchor dependencies
///
/// # Arguments
///
/// * `yaml_text` - The YAML text to scan
///
/// # Returns
///
/// An `AnchorRegistry` containing all anchor and alias information.
///
/// # Errors
///
/// Returns an error if:
/// - Forward references are detected (alias before anchor definition)
/// - Invalid anchor names are found
/// - Anchor redefinitions occur
pub fn scan_yaml_anchors(yaml_text: &str) -> Result<AnchorRegistry, YamlError> {
    let mut registry = AnchorRegistry::new();
    let mut anchor_contexts: HashMap<String, Vec<String>> = HashMap::new();

    // Patterns for matching anchors and aliases
    // Anchor: &name (preceded by whitespace or start of line, capture the name)
    // Alias: *name (preceded by whitespace, colon, or start of line, capture the name)
    // The lookbehind ensures we only match in proper YAML contexts, not inside words like "(*_"
    let anchor_pattern = Regex::new(r"(?:^|[\s:,\[\{])&([a-zA-Z_][a-zA-Z0-9_-]*)").unwrap();
    let alias_pattern = Regex::new(r"(?:^|[\s:,\[\{])\*([a-zA-Z_][a-zA-Z0-9_-]*)").unwrap();

    // Single pass: process anchors and aliases in document order
    // This ensures forward references are detected (alias before anchor definition)
    let mut current_anchor: Option<String> = None;
    let mut brace_depth = 0;

    for (line_num, line) in yaml_text.lines().enumerate() {
        let line_number = line_num + 1;

        // Mask out quoted strings to avoid matching anchors/aliases inside them
        let masked_line = mask_quoted_strings(line);

        // Collect all matches with their positions to process in order
        let mut events: Vec<(usize, bool, String)> = Vec::new(); // (position, is_anchor, name)

        for cap in anchor_pattern.captures_iter(&masked_line) {
            let m = cap.get(1).unwrap();
            events.push((m.start(), true, m.as_str().to_string()));
        }

        for cap in alias_pattern.captures_iter(&masked_line) {
            let m = cap.get(1).unwrap();
            events.push((m.start(), false, m.as_str().to_string()));
        }

        // Sort by position to process in order they appear
        events.sort_by_key(|(pos, _, _)| *pos);

        // Process events in order
        for (_, is_anchor, name) in events {
            if is_anchor {
                // Validate anchor name
                validate_anchor_name(&name)?;

                // Extract the rest of the line after the anchor as "content"
                let content = line
                    .split(&format!("&{name}"))
                    .nth(1)
                    .unwrap_or("")
                    .to_string();

                // Add to registry (fails on redefinition)
                registry.add_anchor(name.clone(), content, line_number)?;

                // Initialize context tracking and set as current anchor
                anchor_contexts.insert(name.clone(), Vec::new());
                current_anchor = Some(name);
                brace_depth = 0;
            } else {
                // This is an alias - validate it references an already-defined anchor
                // This enforces no forward references
                registry.add_alias(&name, line_number)?;

                // If we're inside an anchor definition, record the dependency
                if let Some(ref from_anchor) = current_anchor {
                    if from_anchor != &name {
                        // Record that from_anchor references this alias
                        registry.add_dependency(from_anchor, &name);

                        // Track this in our context map
                        if let Some(refs) = anchor_contexts.get_mut(from_anchor) {
                            refs.push(name.clone());
                        }
                    }
                }
            }
        }

        // Track brace depth to know when we exit an anchor's scope
        brace_depth += line.matches('{').count() as i32;
        brace_depth -= line.matches('}').count() as i32;

        // Simple heuristic: if we're back to depth 0 and see a new top-level key, we've exited
        if brace_depth <= 0 && !line.trim().is_empty() && !line.trim().starts_with('-') {
            // Check if this line starts a new top-level key without an anchor
            if !anchor_pattern.is_match(line)
                && !line.trim().starts_with('#')
                && line.contains(':')
                && !line.starts_with(' ')
                && !line.starts_with('\t')
            {
                current_anchor = None;
            }
        }
    }

    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_simple_anchor_and_alias() {
        let yaml = r"
defaults: &defaults
  timeout: 30
  retries: 3

production:
  config: *defaults
";

        let registry = scan_yaml_anchors(yaml).unwrap();

        assert!(registry.has_anchor("defaults"));
        assert!(registry.has_aliases());
        assert_eq!(registry.anchor_count(), 1);
    }

    #[test]
    fn test_scan_multiple_aliases() {
        let yaml = r"
shared: &shared
  value: 42

a: { ref: *shared }
b: { ref: *shared }
c: { ref: *shared }
";

        let registry = scan_yaml_anchors(yaml).unwrap();

        assert!(registry.has_anchor("shared"));
        assert_eq!(registry.anchor_count(), 1);
    }

    #[test]
    fn test_scan_forward_reference_error() {
        let yaml = r"
before:
  ref: *undefined

after: &undefined
  value: 123
";

        let result = scan_yaml_anchors(yaml);
        assert!(result.is_err());

        match result.unwrap_err() {
            YamlError::ForwardReference { alias, line } => {
                assert_eq!(alias, "undefined");
                assert_eq!(line, 3);
            }
            _ => panic!("Expected ForwardReference error"),
        }
    }

    #[test]
    fn test_scan_nested_anchors() {
        let yaml = r"
inner: &inner
  value: 42

outer: &outer
  nested: *inner
  data: xyz
";

        let registry = scan_yaml_anchors(yaml).unwrap();

        assert_eq!(registry.anchor_count(), 2);
        assert!(registry.has_anchor("inner"));
        assert!(registry.has_anchor("outer"));
    }

    #[test]
    fn test_scan_invalid_anchor_name() {
        let yaml = r"
data: &__reserved
  value: 1
";

        let result = scan_yaml_anchors(yaml);
        assert!(result.is_err());

        match result.unwrap_err() {
            YamlError::InvalidAnchorName { name, .. } => {
                assert_eq!(name, "__reserved");
            }
            _ => panic!("Expected InvalidAnchorName error"),
        }
    }

    #[test]
    fn test_scan_anchor_redefinition() {
        let yaml = r"
config: &config
  value: 1

config: &config
  value: 2
";

        let result = scan_yaml_anchors(yaml);
        assert!(result.is_err());

        match result.unwrap_err() {
            YamlError::AnchorRedefinition { name, .. } => {
                assert_eq!(name, "config");
            }
            _ => panic!("Expected AnchorRedefinition error"),
        }
    }

    #[test]
    fn test_scan_self_reference() {
        let yaml = r"
node: &self
  child: *self
";

        let registry = scan_yaml_anchors(yaml).unwrap();

        assert!(registry.has_anchor("self"));
        // Dependency should be recorded (will be caught by cycle detection)
    }

    #[test]
    fn test_scan_no_anchors() {
        let yaml = r"
name: test
value: 123
nested:
  key: value
";

        let registry = scan_yaml_anchors(yaml).unwrap();

        assert_eq!(registry.anchor_count(), 0);
        assert!(!registry.has_aliases());
    }

    #[test]
    fn test_scan_anchor_in_sequence() {
        let yaml = r"
items:
  - &item1
    id: 1
    name: first
  - *item1
  - *item1
";

        let registry = scan_yaml_anchors(yaml).unwrap();

        assert!(registry.has_anchor("item1"));
        assert!(registry.has_aliases());
    }
}

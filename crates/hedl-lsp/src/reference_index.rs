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

//! Reference index for fast reference lookups.
//!
//! This module implements an O(1) reference lookup system to replace the
//! previous O(n) linear search bottleneck. The index tracks both definitions
//! and references with precise location information.
//!
//! # Performance
//!
//! - **Building**: O(n) where n is document size
//! - **Lookup**: O(1) hash map access
//! - **Memory**: O(r) where r is number of references (minimal overhead)
//!
//! # Architecture
//!
//! The index maintains two primary data structures:
//!
//! 1. **Definition Index**: Maps entity IDs to their definition locations
//!    - Format: `(type_name, id) -> Location`
//!    - Used for: Go to definition
//!
//! 2. **Reference Index**: Maps reference strings to all usage locations
//!    - Format: `reference_string -> Vec<Location>`
//!    - Used for: Find all references
//!
//! # Incremental Updates
//!
//! The index supports incremental updates for efficient document editing:
//!
//! - Full rebuild: Re-parse entire document
//! - Line-based update: Re-index only changed lines (future optimization)
//! - Range-based update: Re-index only changed ranges (future optimization)

use std::collections::HashMap;
use tower_lsp::lsp_types::{Position, Range};

/// Precise location of a reference or definition.
///
/// This structure provides more detail than just line numbers,
/// enabling precise highlighting and navigation in the editor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RefLocation {
    /// Line number (0-indexed for LSP)
    pub line: u32,
    /// Starting character position
    pub start_char: u32,
    /// Ending character position
    pub end_char: u32,
}

impl RefLocation {
    /// Create a new reference location.
    pub fn new(line: u32, start_char: u32, end_char: u32) -> Self {
        Self {
            line,
            start_char,
            end_char,
        }
    }

    /// Convert to LSP Range.
    pub fn to_range(&self) -> Range {
        Range {
            start: Position {
                line: self.line,
                character: self.start_char,
            },
            end: Position {
                line: self.line,
                character: self.end_char,
            },
        }
    }

    /// Create from LSP Position with estimated end position.
    ///
    /// Used when exact character positions are not available.
    /// Assumes a reasonable default width for the reference.
    pub fn from_position(position: Position, estimated_width: u32) -> Self {
        Self {
            line: position.line,
            start_char: position.character,
            end_char: position.character + estimated_width,
        }
    }
}

/// Reference index for O(1) lookups.
///
/// This index provides fast access to both entity definitions and
/// all reference usages throughout the document.
#[derive(Debug, Clone, Default)]
pub struct ReferenceIndex {
    /// Definition locations: (type, id) -> location
    /// Used for "go to definition" feature
    definitions: HashMap<(String, String), RefLocation>,

    /// Reference locations: reference_string -> vec of locations
    /// Used for "find all references" feature
    /// Keys include both qualified (@Type:id) and unqualified (@id) forms
    references: HashMap<String, Vec<RefLocation>>,

    /// Reverse mapping: location -> reference string
    /// Used for finding what reference is at a given position
    location_to_ref: HashMap<u32, Vec<(String, RefLocation)>>,
}

impl ReferenceIndex {
    /// Create a new empty reference index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a definition to the index.
    ///
    /// # Arguments
    ///
    /// * `type_name` - The entity type (e.g., "User")
    /// * `id` - The entity ID (e.g., "alice")
    /// * `location` - The location where this entity is defined
    pub fn add_definition(&mut self, type_name: String, id: String, location: RefLocation) {
        self.definitions
            .insert((type_name.clone(), id.clone()), location.clone());

        // Also add to location_to_ref for reverse lookup
        let ref_str = format!("@{}:{}", type_name, id);
        self.location_to_ref
            .entry(location.line)
            .or_default()
            .push((ref_str, location));
    }

    /// Add a reference to the index.
    ///
    /// # Arguments
    ///
    /// * `type_name` - Optional entity type (Some for qualified refs, None for unqualified)
    /// * `id` - The entity ID being referenced
    /// * `location` - The location where this reference appears
    pub fn add_reference(&mut self, type_name: Option<String>, id: String, location: RefLocation) {
        // Build reference strings
        let ref_str = match &type_name {
            Some(t) => format!("@{}:{}", t, id),
            None => format!("@{}", id),
        };

        // Add to reference index (qualified form)
        self.references
            .entry(ref_str.clone())
            .or_default()
            .push(location.clone());

        // Also index by just the ID for flexible lookup
        let id_ref = format!("@{}", id);
        if id_ref != ref_str {
            self.references
                .entry(id_ref.clone())
                .or_default()
                .push(location.clone());
        }

        // Add to location_to_ref for reverse lookup
        self.location_to_ref
            .entry(location.line)
            .or_default()
            .push((ref_str, location));
    }

    /// Find the definition location for an entity.
    ///
    /// # Returns
    ///
    /// Returns `Some(location)` if the definition is found, `None` otherwise.
    ///
    /// # Performance
    ///
    /// O(1) hash map lookup
    pub fn find_definition(&self, type_name: &str, id: &str) -> Option<&RefLocation> {
        self.definitions
            .get(&(type_name.to_string(), id.to_string()))
    }

    /// Find all reference locations for a given reference string.
    ///
    /// # Arguments
    ///
    /// * `reference` - The reference string to search for (e.g., "@User:alice" or "@alice")
    ///
    /// # Returns
    ///
    /// A slice of all locations where this reference appears, or empty slice if not found.
    ///
    /// # Performance
    ///
    /// O(1) hash map lookup + O(k) where k is number of references (typically small)
    pub fn find_references(&self, reference: &str) -> &[RefLocation] {
        self.references
            .get(reference)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Find what reference is at a given position.
    ///
    /// # Returns
    ///
    /// Returns `Some((reference_string, location))` if a reference is found at the position.
    ///
    /// # Performance
    ///
    /// O(1) hash map lookup + O(k) where k is number of references on that line (typically 1-2)
    pub fn find_reference_at(&self, position: Position) -> Option<(&str, &RefLocation)> {
        let line_refs = self.location_to_ref.get(&position.line)?;

        for (ref_str, loc) in line_refs {
            if position.character >= loc.start_char && position.character <= loc.end_char {
                return Some((ref_str.as_str(), loc));
            }
        }

        None
    }

    /// Get all definitions in the index.
    ///
    /// Useful for providing a complete list of navigable definitions.
    pub fn all_definitions(&self) -> impl Iterator<Item = ((&str, &str), &RefLocation)> {
        self.definitions
            .iter()
            .map(|((t, id), loc)| ((t.as_str(), id.as_str()), loc))
    }

    /// Get all reference strings with their usage counts.
    ///
    /// Useful for analytics and debugging.
    pub fn reference_counts(&self) -> impl Iterator<Item = (&str, usize)> {
        self.references
            .iter()
            .map(|(ref_str, locs)| (ref_str.as_str(), locs.len()))
    }

    /// Clear the entire index.
    ///
    /// Used when performing a full rebuild.
    pub fn clear(&mut self) {
        self.definitions.clear();
        self.references.clear();
        self.location_to_ref.clear();
    }

    /// Get the number of definitions in the index.
    pub fn definition_count(&self) -> usize {
        self.definitions.len()
    }

    /// Get the number of unique reference strings in the index.
    pub fn reference_string_count(&self) -> usize {
        self.references.len()
    }

    /// Get the total number of reference usages in the index.
    pub fn total_reference_count(&self) -> usize {
        self.references.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_find_definition() {
        let mut index = ReferenceIndex::new();

        let loc = RefLocation::new(5, 10, 15);
        index.add_definition("User".to_string(), "alice".to_string(), loc.clone());

        let found = index.find_definition("User", "alice");
        assert_eq!(found, Some(&loc));

        // Non-existent definition
        assert_eq!(index.find_definition("User", "bob"), None);
        assert_eq!(index.find_definition("Product", "alice"), None);
    }

    #[test]
    fn test_add_and_find_references() {
        let mut index = ReferenceIndex::new();

        let loc1 = RefLocation::new(10, 5, 16);
        let loc2 = RefLocation::new(15, 8, 19);

        index.add_reference(Some("User".to_string()), "alice".to_string(), loc1.clone());
        index.add_reference(Some("User".to_string()), "alice".to_string(), loc2.clone());

        // Find by qualified reference
        let refs = index.find_references("@User:alice");
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&loc1));
        assert!(refs.contains(&loc2));

        // Find by unqualified reference (ID only)
        let refs_by_id = index.find_references("@alice");
        assert_eq!(refs_by_id.len(), 2);
    }

    #[test]
    fn test_find_reference_at_position() {
        let mut index = ReferenceIndex::new();

        let loc = RefLocation::new(5, 10, 21); // "@User:alice" spans chars 10-21
        index.add_reference(Some("User".to_string()), "alice".to_string(), loc);

        // Position within reference
        let pos = Position {
            line: 5,
            character: 15,
        };
        let found = index.find_reference_at(pos);
        assert!(found.is_some());
        let (ref_str, _) = found.unwrap();
        assert_eq!(ref_str, "@User:alice");

        // Position before reference
        let pos_before = Position {
            line: 5,
            character: 5,
        };
        assert!(index.find_reference_at(pos_before).is_none());

        // Position after reference
        let pos_after = Position {
            line: 5,
            character: 25,
        };
        assert!(index.find_reference_at(pos_after).is_none());

        // Different line
        let pos_other_line = Position {
            line: 6,
            character: 15,
        };
        assert!(index.find_reference_at(pos_other_line).is_none());
    }

    #[test]
    fn test_unqualified_reference() {
        let mut index = ReferenceIndex::new();

        let loc = RefLocation::new(8, 5, 11);
        index.add_reference(None, "alice".to_string(), loc.clone());

        // Should be findable by @alice
        let refs = index.find_references("@alice");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0], loc);
    }

    #[test]
    fn test_clear() {
        let mut index = ReferenceIndex::new();

        index.add_definition(
            "User".to_string(),
            "alice".to_string(),
            RefLocation::new(1, 0, 5),
        );
        index.add_reference(
            Some("User".to_string()),
            "alice".to_string(),
            RefLocation::new(2, 0, 11),
        );

        assert_eq!(index.definition_count(), 1);
        assert!(index.total_reference_count() > 0);

        index.clear();

        assert_eq!(index.definition_count(), 0);
        assert_eq!(index.total_reference_count(), 0);
    }

    #[test]
    fn test_statistics() {
        let mut index = ReferenceIndex::new();

        index.add_definition(
            "User".to_string(),
            "alice".to_string(),
            RefLocation::new(1, 0, 5),
        );
        index.add_definition(
            "User".to_string(),
            "bob".to_string(),
            RefLocation::new(2, 0, 3),
        );

        index.add_reference(
            Some("User".to_string()),
            "alice".to_string(),
            RefLocation::new(5, 0, 11),
        );
        index.add_reference(
            Some("User".to_string()),
            "alice".to_string(),
            RefLocation::new(6, 0, 11),
        );
        index.add_reference(
            Some("User".to_string()),
            "bob".to_string(),
            RefLocation::new(7, 0, 9),
        );

        assert_eq!(index.definition_count(), 2);
        // Each reference is indexed twice: by qualified and unqualified form
        assert!(index.reference_string_count() >= 2);
        assert_eq!(index.total_reference_count(), 6); // 3 refs * 2 (qualified + unqualified)
    }

    #[test]
    fn test_reflocation_to_range() {
        let loc = RefLocation::new(5, 10, 20);
        let range = loc.to_range();

        assert_eq!(range.start.line, 5);
        assert_eq!(range.start.character, 10);
        assert_eq!(range.end.line, 5);
        assert_eq!(range.end.character, 20);
    }

    #[test]
    fn test_reflocation_from_position() {
        let pos = Position {
            line: 10,
            character: 15,
        };
        let loc = RefLocation::from_position(pos, 8);

        assert_eq!(loc.line, 10);
        assert_eq!(loc.start_char, 15);
        assert_eq!(loc.end_char, 23);
    }

    #[test]
    fn test_all_definitions() {
        let mut index = ReferenceIndex::new();

        index.add_definition(
            "User".to_string(),
            "alice".to_string(),
            RefLocation::new(1, 0, 5),
        );
        index.add_definition(
            "User".to_string(),
            "bob".to_string(),
            RefLocation::new(2, 0, 3),
        );
        index.add_definition(
            "Product".to_string(),
            "widget".to_string(),
            RefLocation::new(3, 0, 6),
        );

        let all_defs: Vec<_> = index.all_definitions().collect();
        assert_eq!(all_defs.len(), 3);

        // Check that all expected definitions are present
        let def_ids: Vec<_> = all_defs.iter().map(|((t, id), _)| (*t, *id)).collect();
        assert!(def_ids.contains(&("User", "alice")));
        assert!(def_ids.contains(&("User", "bob")));
        assert!(def_ids.contains(&("Product", "widget")));
    }

    #[test]
    fn test_reference_counts() {
        let mut index = ReferenceIndex::new();

        // Add multiple references to the same entity
        index.add_reference(
            Some("User".to_string()),
            "alice".to_string(),
            RefLocation::new(5, 0, 11),
        );
        index.add_reference(
            Some("User".to_string()),
            "alice".to_string(),
            RefLocation::new(6, 0, 11),
        );
        index.add_reference(
            Some("User".to_string()),
            "alice".to_string(),
            RefLocation::new(7, 0, 11),
        );

        let counts: Vec<_> = index.reference_counts().collect();

        // Find the count for @User:alice
        let alice_count = counts.iter().find(|(ref_str, _)| *ref_str == "@User:alice");
        assert!(alice_count.is_some());
        assert_eq!(alice_count.unwrap().1, 3);
    }
}

// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Constants used throughout the hedl-neo4j library.

/// Prefix for Neo4j relationship types representing NEST hierarchies.
///
/// When exporting HEDL documents to Neo4j, NEST relationships (parent-child
/// hierarchies defined in `Document.nests`) are represented as Neo4j relationships
/// with types following the pattern `HAS_<CHILDTYPE>`.
///
/// For example:
/// - A NEST from `User` to `Post` becomes `HAS_POST` relationships
/// - A NEST from `Post` to `Comment` becomes `HAS_COMMENT` relationships
///
/// When importing from Neo4j, relationships matching this prefix pattern are
/// automatically inferred as NEST hierarchies (unless configured otherwise
/// via `FromNeo4jConfig::infer_nests_from_has_pattern`).
///
/// # Configuration Override
///
/// This prefix is used with `RelationshipNaming::PropertyName` (default).
/// Other naming strategies can be configured:
/// - `RelationshipNaming::Generic` → uses "`HAS_CHILD`" for all NESTs
/// - `RelationshipNaming::TargetType` → uses child type name directly
///
/// # Examples
///
/// ```rust
/// use hedl_neo4j::constants::NEST_RELATIONSHIP_PREFIX;
///
/// // Generating NEST relationship type
/// let child_type = "Post";
/// let rel_type = format!("{}{}", NEST_RELATIONSHIP_PREFIX, child_type.to_uppercase());
/// assert_eq!(rel_type, "HAS_POST");
///
/// // Detecting NEST relationship
/// let rel_type = "HAS_COMMENT";
/// assert!(rel_type.starts_with(NEST_RELATIONSHIP_PREFIX));
/// ```
pub const NEST_RELATIONSHIP_PREFIX: &str = "HAS_";

/// Generic NEST relationship type used with `RelationshipNaming::Generic`.
///
/// When `ToCypherConfig::nest_naming` is set to `RelationshipNaming::Generic`,
/// all NEST relationships use this single type instead of `HAS_<CHILDTYPE>`.
///
/// # Examples
///
/// ```rust
/// use hedl_neo4j::constants::NEST_RELATIONSHIP_GENERIC;
///
/// assert_eq!(NEST_RELATIONSHIP_GENERIC, "HAS_CHILD");
/// ```
pub const NEST_RELATIONSHIP_GENERIC: &str = "HAS_CHILD";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nest_relationship_prefix_properties() {
        // Verify prefix value
        assert_eq!(NEST_RELATIONSHIP_PREFIX, "HAS_");

        // Verify it ends with underscore (important for formatting)
        assert!(
            NEST_RELATIONSHIP_PREFIX.ends_with('_'),
            "NEST_RELATIONSHIP_PREFIX must end with underscore for proper formatting"
        );

        // Verify it's all uppercase (convention)
        assert_eq!(
            NEST_RELATIONSHIP_PREFIX,
            NEST_RELATIONSHIP_PREFIX.to_uppercase(),
            "NEST_RELATIONSHIP_PREFIX should be uppercase"
        );
    }

    #[test]
    fn test_nest_relationship_generic_value() {
        assert_eq!(NEST_RELATIONSHIP_GENERIC, "HAS_CHILD");

        // Verify it matches the prefix pattern
        assert!(
            NEST_RELATIONSHIP_GENERIC.starts_with(NEST_RELATIONSHIP_PREFIX),
            "Generic NEST relationship should follow prefix convention"
        );
    }

    #[test]
    fn test_relationship_type_construction() {
        // Verify the constant works correctly in formatting
        let child_type = "Post";
        let rel_type = format!("{}{}", NEST_RELATIONSHIP_PREFIX, child_type.to_uppercase());
        assert_eq!(rel_type, "HAS_POST");

        // Verify detection pattern
        assert!(rel_type.starts_with(NEST_RELATIONSHIP_PREFIX));
    }
}

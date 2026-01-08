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

//! Configuration types for Neo4j conversion operations.

use serde::{Deserialize, Serialize};

/// Default maximum string length for property values: 100 MB.
///
/// This limit is set high to accommodate large text content commonly found in
/// graph databases (descriptions, articles, documentation, etc.), while still
/// providing protection against resource exhaustion attacks.
///
/// Cypher queries can contain large text properties including:
/// - Long-form content (articles, documentation, descriptions)
/// - Serialized JSON or XML data
/// - Large text fields from data imports
///
/// For stricter security requirements, use `ToCypherConfig::for_untrusted_input()`
/// which enforces a conservative 1MB limit.
pub const DEFAULT_MAX_STRING_LENGTH: usize = 100 * 1024 * 1024; // 100 MB

/// How to name relationships generated from references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RelationshipNaming {
    /// Use the property name as relationship type (e.g., `author` -> `:AUTHOR`).
    #[default]
    PropertyName,
    /// Use a generic relationship type (e.g., `:REFERENCES`).
    Generic,
    /// Use the target type name (e.g., `@User:alice` -> `:USER`).
    TargetType,
}

/// How to handle nested objects in node properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ObjectHandling {
    /// Flatten nested objects into dot-notation properties (e.g., `address.city`).
    #[default]
    Flatten,
    /// Serialize nested objects as JSON strings.
    JsonString,
}

/// Configuration for converting HEDL documents to Cypher queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToCypherConfig {
    /// Use MERGE instead of CREATE for idempotent imports (default: true).
    pub use_merge: bool,

    /// Generate uniqueness constraints for ID properties (default: true).
    pub create_constraints: bool,

    /// How to name relationships from references.
    pub reference_naming: RelationshipNaming,

    /// How to name relationships from NEST hierarchies.
    pub nest_naming: RelationshipNaming,

    /// How to handle nested objects in properties.
    pub object_handling: ObjectHandling,

    /// Property name to use for HEDL node IDs (default: "_hedl_id").
    pub id_property: String,

    /// Batch size for UNWIND statements (default: 1000).
    pub batch_size: usize,

    /// Include type metadata property (default: false).
    pub include_type_metadata: bool,

    /// Property name for type metadata (default: "_hedl_type").
    pub type_property: String,

    /// Generate comments in output (default: true).
    pub include_comments: bool,

    /// Maximum string length for property values (default: 100MB, None = unlimited).
    ///
    /// Use this to prevent resource exhaustion attacks from malicious input.
    ///
    /// The default limit of 100MB accommodates large text content commonly found
    /// in graph databases while providing reasonable protection against resource
    /// exhaustion. Cypher queries often contain large text properties such as:
    /// - Long-form content (articles, documentation, descriptions)
    /// - Serialized JSON or XML data
    /// - Large text fields from data imports
    ///
    /// Recommended values:
    /// - Production (trusted data): Some(100_000_000) (100MB, default)
    /// - Production (untrusted data): Some(1_000_000) (1MB, use `for_untrusted_input()`)
    /// - Strict: Some(1_000_000) (1MB)
    /// - Development: None (unlimited)
    pub max_string_length: Option<usize>,

    /// Maximum number of nodes to process (default: None = unlimited).
    ///
    /// Use this to prevent memory exhaustion from large documents.
    /// Recommended for untrusted input: Some(1_000_000)
    pub max_nodes: Option<usize>,
}

impl Default for ToCypherConfig {
    fn default() -> Self {
        Self {
            use_merge: true,
            create_constraints: true,
            reference_naming: RelationshipNaming::PropertyName,
            nest_naming: RelationshipNaming::PropertyName,
            object_handling: ObjectHandling::Flatten,
            id_property: "_hedl_id".to_string(),
            batch_size: 1000,
            include_type_metadata: false,
            type_property: "_hedl_type".to_string(),
            include_comments: true,
            max_string_length: Some(DEFAULT_MAX_STRING_LENGTH), // 100MB default
            max_nodes: None,
        }
    }
}

/// Builder for ToCypherConfig.
///
/// Provides a fluent API for constructing ToCypherConfig instances with custom settings.
///
/// # Examples
///
/// ```
/// # use hedl_neo4j::ToCypherConfig;
/// let config = ToCypherConfig::builder()
///     .use_merge(true)
///     .create_constraints(false)
///     .batch_size(500)
///     .build();
/// ```
#[derive(Default)]
pub struct ToCypherConfigBuilder {
    use_merge: Option<bool>,
    create_constraints: Option<bool>,
    reference_naming: Option<RelationshipNaming>,
    nest_naming: Option<RelationshipNaming>,
    object_handling: Option<ObjectHandling>,
    id_property: Option<String>,
    batch_size: Option<usize>,
    include_type_metadata: Option<bool>,
    type_property: Option<String>,
    include_comments: Option<bool>,
    max_string_length: Option<Option<usize>>,
    max_nodes: Option<Option<usize>>,
}

impl ToCypherConfigBuilder {
    /// Create a new builder with no values set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to use MERGE instead of CREATE for idempotent imports.
    pub fn use_merge(mut self, use_merge: bool) -> Self {
        self.use_merge = Some(use_merge);
        self
    }

    /// Set whether to generate uniqueness constraints for ID properties.
    pub fn create_constraints(mut self, create: bool) -> Self {
        self.create_constraints = Some(create);
        self
    }

    /// Set how to name relationships from references.
    pub fn reference_naming(mut self, naming: RelationshipNaming) -> Self {
        self.reference_naming = Some(naming);
        self
    }

    /// Set how to name relationships from NEST hierarchies.
    pub fn nest_naming(mut self, naming: RelationshipNaming) -> Self {
        self.nest_naming = Some(naming);
        self
    }

    /// Set how to handle nested objects in properties.
    pub fn object_handling(mut self, handling: ObjectHandling) -> Self {
        self.object_handling = Some(handling);
        self
    }

    /// Set the property name to use for HEDL node IDs.
    pub fn id_property(mut self, name: impl Into<String>) -> Self {
        self.id_property = Some(name.into());
        self
    }

    /// Set the batch size for UNWIND statements.
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = Some(size);
        self
    }

    /// Set whether to include type metadata property.
    pub fn include_type_metadata(mut self, include: bool) -> Self {
        self.include_type_metadata = Some(include);
        self
    }

    /// Set the property name for type metadata.
    pub fn type_property(mut self, name: impl Into<String>) -> Self {
        self.type_property = Some(name.into());
        self
    }

    /// Set whether to generate comments in output.
    pub fn include_comments(mut self, include: bool) -> Self {
        self.include_comments = Some(include);
        self
    }

    /// Set maximum string length for property values.
    ///
    /// Use this to protect against resource exhaustion attacks.
    pub fn max_string_length(mut self, max: usize) -> Self {
        self.max_string_length = Some(Some(max));
        self
    }

    /// Remove string length limit (use with caution).
    ///
    /// Disabling the string length limit removes protection against resource
    /// exhaustion attacks. Only use this for trusted data sources.
    pub fn no_string_length_limit(mut self) -> Self {
        self.max_string_length = Some(None);
        self
    }

    /// Set maximum number of nodes to process.
    pub fn max_nodes(mut self, max: usize) -> Self {
        self.max_nodes = Some(Some(max));
        self
    }

    /// Build the ToCypherConfig instance.
    ///
    /// All unset fields will use their default values.
    pub fn build(self) -> ToCypherConfig {
        let defaults = ToCypherConfig::default();
        ToCypherConfig {
            use_merge: self.use_merge.unwrap_or(defaults.use_merge),
            create_constraints: self.create_constraints.unwrap_or(defaults.create_constraints),
            reference_naming: self.reference_naming.unwrap_or(defaults.reference_naming),
            nest_naming: self.nest_naming.unwrap_or(defaults.nest_naming),
            object_handling: self.object_handling.unwrap_or(defaults.object_handling),
            id_property: self.id_property.unwrap_or(defaults.id_property),
            batch_size: self.batch_size.unwrap_or(defaults.batch_size),
            include_type_metadata: self.include_type_metadata.unwrap_or(defaults.include_type_metadata),
            type_property: self.type_property.unwrap_or(defaults.type_property),
            include_comments: self.include_comments.unwrap_or(defaults.include_comments),
            max_string_length: self.max_string_length.unwrap_or(defaults.max_string_length),
            max_nodes: self.max_nodes.unwrap_or(defaults.max_nodes),
        }
    }
}

impl ToCypherConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder for ToCypherConfig.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// let config = ToCypherConfig::builder()
    ///     .use_merge(false)
    ///     .batch_size(500)
    ///     .build();
    /// ```
    pub fn builder() -> ToCypherConfigBuilder {
        ToCypherConfigBuilder::default()
    }

    /// Use CREATE instead of MERGE.
    pub fn with_create(mut self) -> Self {
        self.use_merge = false;
        self
    }

    /// Disable constraint generation.
    pub fn without_constraints(mut self) -> Self {
        self.create_constraints = false;
        self
    }

    /// Set the ID property name.
    pub fn with_id_property(mut self, name: impl Into<String>) -> Self {
        self.id_property = name.into();
        self
    }

    /// Set the batch size for UNWIND statements.
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Use JSON strings for nested objects.
    pub fn with_json_objects(mut self) -> Self {
        self.object_handling = ObjectHandling::JsonString;
        self
    }

    /// Include type metadata in nodes.
    pub fn with_type_metadata(mut self) -> Self {
        self.include_type_metadata = true;
        self
    }

    /// Disable comments in output.
    pub fn without_comments(mut self) -> Self {
        self.include_comments = false;
        self
    }

    /// Set maximum string length for property values.
    ///
    /// Use this to protect against resource exhaustion attacks.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// // Custom 50MB limit
    /// let config = ToCypherConfig::new()
    ///     .with_max_string_length(50 * 1024 * 1024);
    /// ```
    pub fn with_max_string_length(mut self, max: usize) -> Self {
        self.max_string_length = Some(max);
        self
    }

    /// Remove string length limit (use with caution).
    ///
    /// Disabling the string length limit removes protection against resource
    /// exhaustion attacks. Only use this for trusted data sources.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// // No limit for trusted data
    /// let config = ToCypherConfig::new()
    ///     .without_string_length_limit();
    /// ```
    pub fn without_string_length_limit(mut self) -> Self {
        self.max_string_length = None;
        self
    }

    /// Set maximum number of nodes to process.
    pub fn with_max_nodes(mut self, max: usize) -> Self {
        self.max_nodes = Some(max);
        self
    }

    /// Create a configuration suitable for untrusted input.
    ///
    /// Sets conservative limits for processing data from untrusted sources.
    /// This provides strong protection against resource exhaustion attacks.
    ///
    /// The default configuration uses a 100MB string limit which is appropriate
    /// for trusted data. For untrusted input, this method enforces much stricter
    /// limits:
    /// - 1MB max string length (vs 100MB default)
    /// - 100K max nodes
    /// - No comments (reduce output size)
    /// - Batch size: 100 (smaller batches for better control)
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// // For processing user-uploaded data
    /// let config = ToCypherConfig::for_untrusted_input();
    /// ```
    pub fn for_untrusted_input() -> Self {
        Self {
            max_string_length: Some(1_000_000),
            max_nodes: Some(100_000),
            batch_size: 100,
            include_comments: false,
            ..Default::default()
        }
    }
}

/// Configuration for converting Neo4j records to HEDL documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FromNeo4jConfig {
    /// HEDL version for the output document (default: (1, 0)).
    pub version: (u32, u32),

    /// Property name to use as HEDL node ID (default: "_hedl_id").
    pub id_property: String,

    /// Infer NEST relationships from `HAS_*` relationship patterns (default: true).
    pub infer_nests: bool,

    /// Property name for type metadata (default: "_hedl_type").
    pub type_property: String,

    /// Properties to exclude from HEDL output.
    pub exclude_properties: Vec<String>,

    /// Labels to exclude from HEDL output.
    pub exclude_labels: Vec<String>,

    /// Relationship types to treat as references (not NEST).
    pub reference_relationships: Vec<String>,

    /// Use the first property as ID if `id_property` is not found (default: true).
    pub fallback_id: bool,
}

impl Default for FromNeo4jConfig {
    fn default() -> Self {
        Self {
            version: (1, 0),
            id_property: "_hedl_id".to_string(),
            infer_nests: true,
            type_property: "_hedl_type".to_string(),
            exclude_properties: vec![],
            exclude_labels: vec![],
            reference_relationships: vec![],
            fallback_id: true,
        }
    }
}

/// Builder for FromNeo4jConfig.
///
/// Provides a fluent API for constructing FromNeo4jConfig instances with custom settings.
///
/// # Examples
///
/// ```
/// # use hedl_neo4j::FromNeo4jConfig;
/// let config = FromNeo4jConfig::builder()
///     .version(2, 0)
///     .id_property("nodeId")
///     .infer_nests(false)
///     .build();
/// ```
#[derive(Default)]
pub struct FromNeo4jConfigBuilder {
    version: Option<(u32, u32)>,
    id_property: Option<String>,
    infer_nests: Option<bool>,
    type_property: Option<String>,
    exclude_properties: Option<Vec<String>>,
    exclude_labels: Option<Vec<String>>,
    reference_relationships: Option<Vec<String>>,
    fallback_id: Option<bool>,
}

impl FromNeo4jConfigBuilder {
    /// Create a new builder with no values set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the HEDL version for the output document.
    pub fn version(mut self, major: u32, minor: u32) -> Self {
        self.version = Some((major, minor));
        self
    }

    /// Set the property name to use as HEDL node ID.
    pub fn id_property(mut self, name: impl Into<String>) -> Self {
        self.id_property = Some(name.into());
        self
    }

    /// Set whether to infer NEST relationships from `HAS_*` relationship patterns.
    pub fn infer_nests(mut self, infer: bool) -> Self {
        self.infer_nests = Some(infer);
        self
    }

    /// Set the property name for type metadata.
    pub fn type_property(mut self, name: impl Into<String>) -> Self {
        self.type_property = Some(name.into());
        self
    }

    /// Set properties to exclude from HEDL output.
    pub fn exclude_properties(mut self, properties: Vec<String>) -> Self {
        self.exclude_properties = Some(properties);
        self
    }

    /// Add a property to exclude from HEDL output.
    pub fn exclude_property(mut self, name: impl Into<String>) -> Self {
        let mut props = self.exclude_properties.unwrap_or_default();
        props.push(name.into());
        self.exclude_properties = Some(props);
        self
    }

    /// Set labels to exclude from HEDL output.
    pub fn exclude_labels(mut self, labels: Vec<String>) -> Self {
        self.exclude_labels = Some(labels);
        self
    }

    /// Add a label to exclude from HEDL output.
    pub fn exclude_label(mut self, name: impl Into<String>) -> Self {
        let mut labels = self.exclude_labels.unwrap_or_default();
        labels.push(name.into());
        self.exclude_labels = Some(labels);
        self
    }

    /// Set relationship types to treat as references (not NEST).
    pub fn reference_relationships(mut self, relationships: Vec<String>) -> Self {
        self.reference_relationships = Some(relationships);
        self
    }

    /// Add a relationship type to treat as a reference.
    pub fn reference_relationship(mut self, rel_type: impl Into<String>) -> Self {
        let mut rels = self.reference_relationships.unwrap_or_default();
        rels.push(rel_type.into());
        self.reference_relationships = Some(rels);
        self
    }

    /// Set whether to use the first property as ID if `id_property` is not found.
    pub fn fallback_id(mut self, fallback: bool) -> Self {
        self.fallback_id = Some(fallback);
        self
    }

    /// Build the FromNeo4jConfig instance.
    ///
    /// All unset fields will use their default values.
    pub fn build(self) -> FromNeo4jConfig {
        let defaults = FromNeo4jConfig::default();
        FromNeo4jConfig {
            version: self.version.unwrap_or(defaults.version),
            id_property: self.id_property.unwrap_or(defaults.id_property),
            infer_nests: self.infer_nests.unwrap_or(defaults.infer_nests),
            type_property: self.type_property.unwrap_or(defaults.type_property),
            exclude_properties: self.exclude_properties.unwrap_or(defaults.exclude_properties),
            exclude_labels: self.exclude_labels.unwrap_or(defaults.exclude_labels),
            reference_relationships: self.reference_relationships.unwrap_or(defaults.reference_relationships),
            fallback_id: self.fallback_id.unwrap_or(defaults.fallback_id),
        }
    }
}

impl FromNeo4jConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder for FromNeo4jConfig.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::FromNeo4jConfig;
    /// let config = FromNeo4jConfig::builder()
    ///     .version(2, 0)
    ///     .infer_nests(false)
    ///     .build();
    /// ```
    pub fn builder() -> FromNeo4jConfigBuilder {
        FromNeo4jConfigBuilder::default()
    }

    /// Set the HEDL version.
    pub fn with_version(mut self, major: u32, minor: u32) -> Self {
        self.version = (major, minor);
        self
    }

    /// Set the ID property name.
    pub fn with_id_property(mut self, name: impl Into<String>) -> Self {
        self.id_property = name.into();
        self
    }

    /// Disable NEST inference.
    pub fn without_nest_inference(mut self) -> Self {
        self.infer_nests = false;
        self
    }

    /// Add properties to exclude.
    pub fn exclude_property(mut self, name: impl Into<String>) -> Self {
        self.exclude_properties.push(name.into());
        self
    }

    /// Add labels to exclude.
    pub fn exclude_label(mut self, name: impl Into<String>) -> Self {
        self.exclude_labels.push(name.into());
        self
    }

    /// Specify relationships that should be treated as references.
    pub fn reference_relationship(mut self, rel_type: impl Into<String>) -> Self {
        self.reference_relationships.push(rel_type.into());
        self
    }

    /// Disable fallback ID behavior.
    pub fn without_fallback_id(mut self) -> Self {
        self.fallback_id = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_cypher_config_default() {
        let config = ToCypherConfig::default();
        assert!(config.use_merge);
        assert!(config.create_constraints);
        assert_eq!(config.id_property, "_hedl_id");
        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.reference_naming, RelationshipNaming::PropertyName);
        assert_eq!(config.object_handling, ObjectHandling::Flatten);
    }

    #[test]
    fn test_to_cypher_config_builder() {
        let config = ToCypherConfig::new()
            .with_create()
            .without_constraints()
            .with_id_property("id")
            .with_batch_size(500)
            .with_json_objects()
            .with_type_metadata()
            .without_comments();

        assert!(!config.use_merge);
        assert!(!config.create_constraints);
        assert_eq!(config.id_property, "id");
        assert_eq!(config.batch_size, 500);
        assert_eq!(config.object_handling, ObjectHandling::JsonString);
        assert!(config.include_type_metadata);
        assert!(!config.include_comments);
    }

    #[test]
    fn test_from_neo4j_config_default() {
        let config = FromNeo4jConfig::default();
        assert_eq!(config.version, (1, 0));
        assert_eq!(config.id_property, "_hedl_id");
        assert!(config.infer_nests);
        assert!(config.fallback_id);
    }

    #[test]
    fn test_from_neo4j_config_builder() {
        let config = FromNeo4jConfig::new()
            .with_version(2, 0)
            .with_id_property("nodeId")
            .without_nest_inference()
            .exclude_property("internal")
            .exclude_label("System")
            .reference_relationship("AUTHORED_BY")
            .without_fallback_id();

        assert_eq!(config.version, (2, 0));
        assert_eq!(config.id_property, "nodeId");
        assert!(!config.infer_nests);
        assert!(!config.fallback_id);
        assert!(config.exclude_properties.contains(&"internal".to_string()));
        assert!(config.exclude_labels.contains(&"System".to_string()));
        assert!(config
            .reference_relationships
            .contains(&"AUTHORED_BY".to_string()));
    }

    #[test]
    fn test_relationship_naming_variants() {
        assert_eq!(
            RelationshipNaming::default(),
            RelationshipNaming::PropertyName
        );

        let naming = RelationshipNaming::Generic;
        assert_eq!(naming, RelationshipNaming::Generic);

        let naming = RelationshipNaming::TargetType;
        assert_eq!(naming, RelationshipNaming::TargetType);
    }

    #[test]
    fn test_object_handling_variants() {
        assert_eq!(ObjectHandling::default(), ObjectHandling::Flatten);

        let handling = ObjectHandling::JsonString;
        assert_eq!(handling, ObjectHandling::JsonString);
    }

    #[test]
    fn test_config_serialization() {
        let config = ToCypherConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: ToCypherConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.use_merge, parsed.use_merge);
        assert_eq!(config.id_property, parsed.id_property);
    }

    #[test]
    fn test_for_untrusted_input_config() {
        let config = ToCypherConfig::for_untrusted_input();
        assert_eq!(config.max_string_length, Some(1_000_000)); // 1MB limit
        assert_eq!(config.max_nodes, Some(100_000));
        assert_eq!(config.batch_size, 100);
        assert!(!config.include_comments);
    }

    #[test]
    fn test_max_string_length_config() {
        let config = ToCypherConfig::default();
        assert_eq!(config.max_string_length, Some(DEFAULT_MAX_STRING_LENGTH)); // 100MB default
        assert_eq!(config.max_string_length, Some(100 * 1024 * 1024)); // Verify actual value (104,857,600 bytes)

        let custom = config.with_max_string_length(5000);
        assert_eq!(custom.max_string_length, Some(5000));

        let unlimited = custom.without_string_length_limit();
        assert_eq!(unlimited.max_string_length, None);
    }

    #[test]
    fn test_default_max_string_length_constant() {
        // Verify the constant has the correct value: 100 MiB = 104,857,600 bytes
        assert_eq!(DEFAULT_MAX_STRING_LENGTH, 100 * 1024 * 1024);
        assert_eq!(DEFAULT_MAX_STRING_LENGTH, 104_857_600);

        // Verify it's approximately 100 million bytes (within 5% tolerance)
        assert!((DEFAULT_MAX_STRING_LENGTH as f64 - 100_000_000.0).abs() / 100_000_000.0 < 0.05);
    }

    // ToCypherConfigBuilder tests
    #[test]
    fn test_to_cypher_builder_defaults() {
        let config = ToCypherConfig::builder().build();
        assert_eq!(config.use_merge, true);
        assert_eq!(config.create_constraints, true);
        assert_eq!(config.id_property, "_hedl_id");
        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.include_comments, true);
        assert_eq!(config.max_string_length, Some(DEFAULT_MAX_STRING_LENGTH));
    }

    #[test]
    fn test_to_cypher_builder_custom() {
        let config = ToCypherConfig::builder()
            .use_merge(false)
            .create_constraints(false)
            .id_property("custom_id")
            .batch_size(500)
            .build();

        assert_eq!(config.use_merge, false);
        assert_eq!(config.create_constraints, false);
        assert_eq!(config.id_property, "custom_id");
        assert_eq!(config.batch_size, 500);
    }

    #[test]
    fn test_to_cypher_builder_chaining() {
        let config = ToCypherConfig::builder()
            .use_merge(true)
            .create_constraints(false)
            .reference_naming(RelationshipNaming::Generic)
            .nest_naming(RelationshipNaming::TargetType)
            .object_handling(ObjectHandling::JsonString)
            .include_type_metadata(true)
            .type_property("custom_type")
            .include_comments(false)
            .max_string_length(5000)
            .max_nodes(10000)
            .build();

        assert_eq!(config.use_merge, true);
        assert_eq!(config.create_constraints, false);
        assert_eq!(config.reference_naming, RelationshipNaming::Generic);
        assert_eq!(config.nest_naming, RelationshipNaming::TargetType);
        assert_eq!(config.object_handling, ObjectHandling::JsonString);
        assert_eq!(config.include_type_metadata, true);
        assert_eq!(config.type_property, "custom_type");
        assert_eq!(config.include_comments, false);
        assert_eq!(config.max_string_length, Some(5000));
        assert_eq!(config.max_nodes, Some(10000));
    }

    #[test]
    fn test_to_cypher_builder_string_limits() {
        let config = ToCypherConfig::builder()
            .max_string_length(1000000)
            .build();
        assert_eq!(config.max_string_length, Some(1000000));

        let config = ToCypherConfig::builder()
            .no_string_length_limit()
            .build();
        assert_eq!(config.max_string_length, None);
    }

    #[test]
    fn test_to_cypher_builder_new() {
        let builder = ToCypherConfigBuilder::new();
        let config = builder.build();
        assert_eq!(config.use_merge, true); // Default value
    }

    // FromNeo4jConfigBuilder tests
    #[test]
    fn test_from_neo4j_builder_defaults() {
        let config = FromNeo4jConfig::builder().build();
        assert_eq!(config.version, (1, 0));
        assert_eq!(config.id_property, "_hedl_id");
        assert_eq!(config.infer_nests, true);
        assert_eq!(config.fallback_id, true);
        assert!(config.exclude_properties.is_empty());
        assert!(config.exclude_labels.is_empty());
        assert!(config.reference_relationships.is_empty());
    }

    #[test]
    fn test_from_neo4j_builder_custom() {
        let config = FromNeo4jConfig::builder()
            .version(2, 0)
            .id_property("nodeId")
            .infer_nests(false)
            .fallback_id(false)
            .build();

        assert_eq!(config.version, (2, 0));
        assert_eq!(config.id_property, "nodeId");
        assert_eq!(config.infer_nests, false);
        assert_eq!(config.fallback_id, false);
    }

    #[test]
    fn test_from_neo4j_builder_chaining() {
        let config = FromNeo4jConfig::builder()
            .version(2, 1)
            .id_property("custom_id")
            .infer_nests(true)
            .type_property("custom_type")
            .exclude_property("internal")
            .exclude_property("temp")
            .exclude_label("System")
            .exclude_label("Internal")
            .reference_relationship("AUTHORED_BY")
            .reference_relationship("CREATED_BY")
            .fallback_id(true)
            .build();

        assert_eq!(config.version, (2, 1));
        assert_eq!(config.id_property, "custom_id");
        assert_eq!(config.infer_nests, true);
        assert_eq!(config.type_property, "custom_type");
        assert_eq!(config.exclude_properties.len(), 2);
        assert!(config.exclude_properties.contains(&"internal".to_string()));
        assert!(config.exclude_properties.contains(&"temp".to_string()));
        assert_eq!(config.exclude_labels.len(), 2);
        assert!(config.exclude_labels.contains(&"System".to_string()));
        assert!(config.exclude_labels.contains(&"Internal".to_string()));
        assert_eq!(config.reference_relationships.len(), 2);
        assert!(config.reference_relationships.contains(&"AUTHORED_BY".to_string()));
        assert!(config.reference_relationships.contains(&"CREATED_BY".to_string()));
        assert_eq!(config.fallback_id, true);
    }

    #[test]
    fn test_from_neo4j_builder_bulk_collections() {
        let config = FromNeo4jConfig::builder()
            .exclude_properties(vec!["prop1".to_string(), "prop2".to_string()])
            .exclude_labels(vec!["Label1".to_string(), "Label2".to_string()])
            .reference_relationships(vec!["REL1".to_string(), "REL2".to_string()])
            .build();

        assert_eq!(config.exclude_properties.len(), 2);
        assert_eq!(config.exclude_labels.len(), 2);
        assert_eq!(config.reference_relationships.len(), 2);
    }

    #[test]
    fn test_from_neo4j_builder_new() {
        let builder = FromNeo4jConfigBuilder::new();
        let config = builder.build();
        assert_eq!(config.version, (1, 0)); // Default value
    }

    #[test]
    fn test_from_neo4j_builder_mixed_collection_methods() {
        // Test mixing bulk and individual additions
        let config = FromNeo4jConfig::builder()
            .exclude_properties(vec!["prop1".to_string()])
            .exclude_property("prop2")
            .build();

        assert_eq!(config.exclude_properties.len(), 2);
        assert!(config.exclude_properties.contains(&"prop1".to_string()));
        assert!(config.exclude_properties.contains(&"prop2".to_string()));
    }
}

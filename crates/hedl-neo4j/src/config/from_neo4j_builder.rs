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

//! Builder for `FromNeo4jConfig`.

use super::from_neo4j_config::FromNeo4jConfig;

/// Builder for `FromNeo4jConfig`.
///
/// Provides a fluent API for constructing `FromNeo4jConfig` instances with custom settings.
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
    batch_size: Option<usize>,
}

impl FromNeo4jConfigBuilder {
    /// Create a new builder with no values set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the HEDL version for the output document.
    #[must_use]
    pub fn version(mut self, major: u32, minor: u32) -> Self {
        self.version = Some((major, minor));
        self
    }

    /// Set the property name to use as HEDL node ID.
    pub fn id_property(mut self, name: impl Into<String>) -> Self {
        self.id_property = Some(name.into());
        self
    }

    /// Set whether to infer NEST relationships from patterns matching
    /// [`crate::constants::NEST_RELATIONSHIP_PREFIX`].
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn fallback_id(mut self, fallback: bool) -> Self {
        self.fallback_id = Some(fallback);
        self
    }

    /// Set the batch size for streaming record processing.
    ///
    /// Controls how many records are processed before flushing to accumulators.
    /// Higher values improve throughput but increase memory usage.
    #[must_use]
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = Some(size);
        self
    }

    /// Build the `FromNeo4jConfig` instance.
    ///
    /// All unset fields will use their default values.
    #[must_use]
    pub fn build(self) -> FromNeo4jConfig {
        let defaults = FromNeo4jConfig::default();
        FromNeo4jConfig {
            version: self.version.unwrap_or(defaults.version),
            id_property: self.id_property.unwrap_or(defaults.id_property),
            infer_nests: self.infer_nests.unwrap_or(defaults.infer_nests),
            type_property: self.type_property.unwrap_or(defaults.type_property),
            exclude_properties: self
                .exclude_properties
                .unwrap_or(defaults.exclude_properties),
            exclude_labels: self.exclude_labels.unwrap_or(defaults.exclude_labels),
            reference_relationships: self
                .reference_relationships
                .unwrap_or(defaults.reference_relationships),
            fallback_id: self.fallback_id.unwrap_or(defaults.fallback_id),
            batch_size: self.batch_size.unwrap_or(defaults.batch_size),
        }
    }
}

impl FromNeo4jConfig {
    /// Create a builder for `FromNeo4jConfig`.
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
    #[must_use]
    pub fn builder() -> FromNeo4jConfigBuilder {
        FromNeo4jConfigBuilder::default()
    }
}

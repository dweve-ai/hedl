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

//! Configuration for converting Neo4j records to HEDL documents.

use super::types::DEFAULT_FROM_NEO4J_BATCH_SIZE;
use serde::{Deserialize, Serialize};

/// Configuration for converting Neo4j records to HEDL documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FromNeo4jConfig {
    /// HEDL version for the output document (default: (2, 0)).
    pub version: (u32, u32),

    /// Property name to use as HEDL node ID (default: "_`hedl_id`").
    pub id_property: String,

    /// Infer NEST relationships from patterns matching [`crate::constants::NEST_RELATIONSHIP_PREFIX`]
    /// (default: true).
    pub infer_nests: bool,

    /// Property name for type metadata (default: "_`hedl_type`").
    pub type_property: String,

    /// Properties to exclude from HEDL output.
    pub exclude_properties: Vec<String>,

    /// Labels to exclude from HEDL output.
    pub exclude_labels: Vec<String>,

    /// Relationship types to treat as references (not NEST).
    pub reference_relationships: Vec<String>,

    /// Use the first property as ID if `id_property` is not found (default: true).
    pub fallback_id: bool,

    /// Batch size for streaming record processing (default: 1000).
    ///
    /// Controls how many records are processed before flushing to accumulators.
    /// Higher values improve throughput but increase memory usage.
    ///
    /// Recommended values:
    /// - Default workloads: 1000
    /// - High throughput: 2000-5000
    /// - Memory constrained: 100-500
    pub batch_size: usize,
}

impl Default for FromNeo4jConfig {
    fn default() -> Self {
        Self {
            version: (2, 0),
            id_property: "_hedl_id".to_string(),
            infer_nests: true,
            type_property: "_hedl_type".to_string(),
            exclude_properties: vec![],
            exclude_labels: vec![],
            reference_relationships: vec![],
            fallback_id: true,
            batch_size: DEFAULT_FROM_NEO4J_BATCH_SIZE,
        }
    }
}

impl FromNeo4jConfig {
    /// Create a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the HEDL version.
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn without_fallback_id(mut self) -> Self {
        self.fallback_id = false;
        self
    }

    /// Set the batch size for streaming record processing.
    ///
    /// Controls how many records are processed before flushing to accumulators.
    /// Higher values improve throughput but increase memory usage.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::FromNeo4jConfig;
    /// // Higher batch size for throughput
    /// let config = FromNeo4jConfig::new()
    ///     .with_batch_size(2000);
    ///
    /// // Lower batch size for memory-constrained environments
    /// let config = FromNeo4jConfig::new()
    ///     .with_batch_size(100);
    /// ```
    #[must_use]
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }
}

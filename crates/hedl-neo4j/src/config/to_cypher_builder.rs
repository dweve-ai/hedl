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

//! Builder for `ToCypherConfig`.

use super::to_cypher_config::ToCypherConfig;
use super::types::{
    BatchSizeStrategy, IsolationLevel, ObjectHandling, RelationshipNaming, TransactionStrategy,
};
use crate::cypher::RenderMode;

/// Builder for `ToCypherConfig`.
///
/// Provides a fluent API for constructing `ToCypherConfig` instances with custom settings.
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
    create_indexes: Option<bool>,
    create_relationship_indexes: Option<bool>,
    create_composite_indexes: Option<bool>,
    indexed_properties: Option<Vec<String>>,
    reference_naming: Option<RelationshipNaming>,
    nest_naming: Option<RelationshipNaming>,
    object_handling: Option<ObjectHandling>,
    id_property: Option<String>,
    batch_size: Option<usize>,
    batch_size_strategy: Option<BatchSizeStrategy>,
    parallel_execution: Option<bool>,
    max_parallel_batches: Option<usize>,
    pipeline_depth: Option<usize>,
    include_type_metadata: Option<bool>,
    type_property: Option<String>,
    include_comments: Option<bool>,
    max_string_length: Option<Option<usize>>,
    max_nodes: Option<Option<usize>>,
    render_mode: Option<RenderMode>,
    streaming_children: Option<bool>,
    // Transaction batching fields
    transaction_batching_enabled: Option<bool>,
    transaction_batch_size: Option<usize>,
    transaction_row_limit: Option<usize>,
    transaction_strategy: Option<TransactionStrategy>,
    transaction_isolation: Option<IsolationLevel>,
    // Query optimization fields
    use_index_hints: Option<bool>,
    enable_template_caching: Option<bool>,
    enable_adaptive_tracking: Option<bool>,
}

impl ToCypherConfigBuilder {
    /// Create a new builder with no values set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to use MERGE instead of CREATE for idempotent imports.
    #[must_use]
    pub fn use_merge(mut self, use_merge: bool) -> Self {
        self.use_merge = Some(use_merge);
        self
    }

    /// Set whether to generate uniqueness constraints for ID properties.
    #[must_use]
    pub fn create_constraints(mut self, create: bool) -> Self {
        self.create_constraints = Some(create);
        self
    }

    /// Set whether to generate indexes for ID properties.
    #[must_use]
    pub fn create_indexes(mut self, create: bool) -> Self {
        self.create_indexes = Some(create);
        self
    }

    /// Set whether to generate indexes for relationship types.
    #[must_use]
    pub fn create_relationship_indexes(mut self, create: bool) -> Self {
        self.create_relationship_indexes = Some(create);
        self
    }

    /// Set whether to generate composite indexes.
    #[must_use]
    pub fn create_composite_indexes(mut self, create: bool) -> Self {
        self.create_composite_indexes = Some(create);
        self
    }

    /// Set properties to create indexes for.
    #[must_use]
    pub fn indexed_properties(mut self, properties: Vec<String>) -> Self {
        self.indexed_properties = Some(properties);
        self
    }

    /// Add a property to the indexed properties list.
    pub fn indexed_property(mut self, property: impl Into<String>) -> Self {
        let mut props = self.indexed_properties.unwrap_or_default();
        props.push(property.into());
        self.indexed_properties = Some(props);
        self
    }

    /// Set how to name relationships from references.
    #[must_use]
    pub fn reference_naming(mut self, naming: RelationshipNaming) -> Self {
        self.reference_naming = Some(naming);
        self
    }

    /// Set how to name relationships from NEST hierarchies.
    #[must_use]
    pub fn nest_naming(mut self, naming: RelationshipNaming) -> Self {
        self.nest_naming = Some(naming);
        self
    }

    /// Set how to handle nested objects in properties.
    #[must_use]
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
    #[must_use]
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = Some(size);
        self
    }

    /// Set the batch size strategy.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::{ToCypherConfig, BatchSizeStrategy};
    /// // Fixed batch size
    /// let config = ToCypherConfig::builder()
    ///     .batch_size_strategy(BatchSizeStrategy::Fixed(1000))
    ///     .build();
    ///
    /// // Adaptive batch size
    /// let config = ToCypherConfig::builder()
    ///     .batch_size_strategy(BatchSizeStrategy::Adaptive {
    ///         target_batch_bytes: 512_000,
    ///         min_batch_size: 100,
    ///         max_batch_size: 5000,
    ///     })
    ///     .build();
    /// ```
    #[must_use]
    pub fn batch_size_strategy(mut self, strategy: BatchSizeStrategy) -> Self {
        self.batch_size_strategy = Some(strategy);
        self
    }

    /// Enable parallel batch execution.
    ///
    /// When enabled, independent node type batches are executed concurrently.
    /// Requires async feature and provides 3-5x throughput improvements for large datasets.
    #[must_use]
    pub fn parallel_execution(mut self, enabled: bool) -> Self {
        self.parallel_execution = Some(enabled);
        self
    }

    /// Set maximum number of parallel batch tasks.
    ///
    /// Controls the degree of parallelism. Higher values increase throughput
    /// but require more Neo4j connections.
    #[must_use]
    pub fn max_parallel_batches(mut self, max: usize) -> Self {
        self.max_parallel_batches = Some(max);
        self
    }

    /// Set query pipeline depth for concurrent in-flight queries.
    ///
    /// Higher values reduce network idle time but increase memory usage.
    #[must_use]
    pub fn pipeline_depth(mut self, depth: usize) -> Self {
        self.pipeline_depth = Some(depth);
        self
    }

    /// Set whether to include type metadata property.
    #[must_use]
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
    #[must_use]
    pub fn include_comments(mut self, include: bool) -> Self {
        self.include_comments = Some(include);
        self
    }

    /// Set maximum string length for property values.
    ///
    /// Use this to protect against resource exhaustion attacks.
    #[must_use]
    pub fn max_string_length(mut self, max: usize) -> Self {
        self.max_string_length = Some(Some(max));
        self
    }

    /// Remove string length limit (use with caution).
    ///
    /// Disabling the string length limit removes protection against resource
    /// exhaustion attacks. Only use this for trusted data sources.
    #[must_use]
    pub fn no_string_length_limit(mut self) -> Self {
        self.max_string_length = Some(None);
        self
    }

    /// Set maximum number of nodes to process.
    #[must_use]
    pub fn max_nodes(mut self, max: usize) -> Self {
        self.max_nodes = Some(Some(max));
        self
    }

    /// Set the render mode for Cypher parameters.
    ///
    /// Use `RenderMode::Parameterized` for maximum security with untrusted input.
    #[must_use]
    pub fn render_mode(mut self, mode: RenderMode) -> Self {
        self.render_mode = Some(mode);
        self
    }

    /// Set whether to enable streaming child collection.
    ///
    /// When enabled, child nodes from NEST hierarchies are processed incrementally,
    /// reducing peak memory usage by ~99% for large hierarchies.
    #[must_use]
    pub fn streaming_children(mut self, enabled: bool) -> Self {
        self.streaming_children = Some(enabled);
        self
    }

    /// Enable or disable transaction batching.
    #[must_use]
    pub fn transaction_batching_enabled(mut self, enabled: bool) -> Self {
        self.transaction_batching_enabled = Some(enabled);
        self
    }

    /// Set transaction batch size.
    #[must_use]
    pub fn transaction_batch_size(mut self, size: usize) -> Self {
        self.transaction_batch_size = Some(size);
        self
    }

    /// Set transaction row limit.
    #[must_use]
    pub fn transaction_row_limit(mut self, limit: usize) -> Self {
        self.transaction_row_limit = Some(limit);
        self
    }

    /// Set transaction strategy.
    #[must_use]
    pub fn transaction_strategy(mut self, strategy: TransactionStrategy) -> Self {
        self.transaction_strategy = Some(strategy);
        self
    }

    /// Set transaction isolation level.
    #[must_use]
    pub fn transaction_isolation(mut self, isolation: IsolationLevel) -> Self {
        self.transaction_isolation = Some(isolation);
        self
    }

    /// Enable or disable index hints.
    #[must_use]
    pub fn use_index_hints(mut self, enabled: bool) -> Self {
        self.use_index_hints = Some(enabled);
        self
    }

    /// Enable or disable template caching.
    #[must_use]
    pub fn enable_template_caching(mut self, enabled: bool) -> Self {
        self.enable_template_caching = Some(enabled);
        self
    }

    /// Enable or disable adaptive tracking.
    #[must_use]
    pub fn enable_adaptive_tracking(mut self, enabled: bool) -> Self {
        self.enable_adaptive_tracking = Some(enabled);
        self
    }

    /// Build the `ToCypherConfig` instance.
    ///
    /// All unset fields will use their default values.
    #[must_use]
    pub fn build(self) -> ToCypherConfig {
        let defaults = ToCypherConfig::default();
        ToCypherConfig {
            use_merge: self.use_merge.unwrap_or(defaults.use_merge),
            create_constraints: self
                .create_constraints
                .unwrap_or(defaults.create_constraints),
            create_indexes: self.create_indexes.unwrap_or(defaults.create_indexes),
            create_relationship_indexes: self
                .create_relationship_indexes
                .unwrap_or(defaults.create_relationship_indexes),
            create_composite_indexes: self
                .create_composite_indexes
                .unwrap_or(defaults.create_composite_indexes),
            indexed_properties: self
                .indexed_properties
                .unwrap_or(defaults.indexed_properties),
            reference_naming: self.reference_naming.unwrap_or(defaults.reference_naming),
            nest_naming: self.nest_naming.unwrap_or(defaults.nest_naming),
            object_handling: self.object_handling.unwrap_or(defaults.object_handling),
            id_property: self.id_property.unwrap_or(defaults.id_property),
            batch_size: self.batch_size.unwrap_or(defaults.batch_size),
            batch_size_strategy: self
                .batch_size_strategy
                .unwrap_or(defaults.batch_size_strategy),
            parallel_execution: self
                .parallel_execution
                .unwrap_or(defaults.parallel_execution),
            max_parallel_batches: self
                .max_parallel_batches
                .unwrap_or(defaults.max_parallel_batches),
            pipeline_depth: self.pipeline_depth.unwrap_or(defaults.pipeline_depth),
            include_type_metadata: self
                .include_type_metadata
                .unwrap_or(defaults.include_type_metadata),
            type_property: self.type_property.unwrap_or(defaults.type_property),
            include_comments: self.include_comments.unwrap_or(defaults.include_comments),
            max_string_length: self.max_string_length.unwrap_or(defaults.max_string_length),
            max_nodes: self.max_nodes.unwrap_or(defaults.max_nodes),
            render_mode: self.render_mode.unwrap_or(defaults.render_mode),
            streaming_children: self
                .streaming_children
                .unwrap_or(defaults.streaming_children),
            // Transaction batching fields
            transaction_batching_enabled: self
                .transaction_batching_enabled
                .unwrap_or(defaults.transaction_batching_enabled),
            transaction_batch_size: self
                .transaction_batch_size
                .unwrap_or(defaults.transaction_batch_size),
            transaction_row_limit: self
                .transaction_row_limit
                .unwrap_or(defaults.transaction_row_limit),
            transaction_strategy: self
                .transaction_strategy
                .unwrap_or(defaults.transaction_strategy),
            transaction_isolation: self
                .transaction_isolation
                .unwrap_or(defaults.transaction_isolation),
            // Query optimization fields
            use_index_hints: self.use_index_hints.unwrap_or(defaults.use_index_hints),
            enable_template_caching: self
                .enable_template_caching
                .unwrap_or(defaults.enable_template_caching),
            enable_adaptive_tracking: self
                .enable_adaptive_tracking
                .unwrap_or(defaults.enable_adaptive_tracking),
        }
    }
}

impl ToCypherConfig {
    /// Create a builder for `ToCypherConfig`.
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
    #[must_use]
    pub fn builder() -> ToCypherConfigBuilder {
        ToCypherConfigBuilder::default()
    }
}

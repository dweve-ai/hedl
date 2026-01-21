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

use crate::cypher::RenderMode;
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

/// Default transaction batch size: 100 statements per transaction.
pub const DEFAULT_TRANSACTION_BATCH_SIZE: usize = 100;

/// Default transaction row limit: 10,000 rows per transaction.
pub const DEFAULT_TRANSACTION_ROW_LIMIT: usize = 10_000;

/// Strategy for batching multiple statements into transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TransactionStrategy {
    /// Batch by statement count (default).
    #[default]
    StatementCount,
    /// Batch by estimated row count.
    RowCount,
    /// Batch by statement type (nodes, relationships, indexes).
    StatementType,
    /// Adaptive batching based on execution time.
    Adaptive,
}

/// Transaction isolation level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// Default isolation level (database default).
    #[default]
    Default,
    /// Serializable isolation for strict consistency.
    Serializable,
}

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

/// Batch size strategy for determining optimal batch sizes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchSizeStrategy {
    /// Fixed batch size (legacy behavior).
    Fixed(usize),
    /// Adaptive based on node size estimation.
    ///
    /// Dynamically calculates optimal batch size based on:
    /// - Average node size (properties and their values)
    /// - Target batch memory footprint
    /// - Min/max batch size bounds
    Adaptive {
        /// Target batch size in bytes (default: 512KB).
        target_batch_bytes: usize,
        /// Minimum batch size regardless of node size (default: 100).
        min_batch_size: usize,
        /// Maximum batch size regardless of node size (default: 5000).
        max_batch_size: usize,
    },
}

impl Default for BatchSizeStrategy {
    fn default() -> Self {
        BatchSizeStrategy::Fixed(1000)
    }
}

/// Configuration for converting HEDL documents to Cypher queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToCypherConfig {
    /// Use MERGE instead of CREATE for idempotent imports (default: true).
    pub use_merge: bool,

    /// Generate uniqueness constraints for ID properties (default: true).
    pub create_constraints: bool,

    /// Generate indexes for ID properties (default: true).
    ///
    /// Creates RANGE indexes on `id_property` for all node types,
    /// dramatically improving MERGE and MATCH performance.
    ///
    /// Index creation uses the Neo4j 5.x `IF NOT EXISTS` syntax,
    /// making it safe to run multiple times.
    pub create_indexes: bool,

    /// Generate indexes for relationship types (default: false).
    ///
    /// Creates indexes on relationship types and NEST order properties.
    /// Enable for workloads with heavy relationship traversal.
    ///
    /// **Performance Impact:**
    /// - Relationship queries: 5-20x faster
    /// - NEST traversal: 10x faster with order index
    ///
    /// **Trade-off:** Slower write performance, increased storage.
    pub create_relationship_indexes: bool,

    /// Generate composite indexes for common patterns (default: false).
    ///
    /// Creates multi-property indexes for frequent query patterns.
    /// Enable for production deployments with known query patterns.
    ///
    /// **Performance Impact:**
    /// - Label+property queries: 20-100x faster
    ///
    /// **Trade-off:** More indexes = more maintenance overhead.
    pub create_composite_indexes: bool,

    /// Properties to create individual indexes for.
    ///
    /// Specify property names that should have dedicated indexes
    /// across all node types that contain them.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// let config = ToCypherConfig::new()
    ///     .with_indexed_property("email")
    ///     .with_indexed_property("username");
    /// ```
    pub indexed_properties: Vec<String>,

    /// How to name relationships from references.
    pub reference_naming: RelationshipNaming,

    /// How to name relationships from NEST hierarchies.
    pub nest_naming: RelationshipNaming,

    /// How to handle nested objects in properties.
    pub object_handling: ObjectHandling,

    /// Property name to use for HEDL node IDs (default: "_`hedl_id`").
    pub id_property: String,

    /// Batch size for UNWIND statements (default: 1000).
    ///
    /// **Deprecated**: Use `batch_size_strategy` for more control.
    /// This field is maintained for backward compatibility and will be used
    /// when `batch_size_strategy` is `Fixed`.
    pub batch_size: usize,

    /// Batch size strategy (default: Fixed(1000)).
    ///
    /// Controls how batch sizes are determined:
    /// - `Fixed`: Use a constant batch size (backward compatible)
    /// - `Adaptive`: Dynamically calculate based on node complexity
    ///
    /// For large-scale imports with varied node sizes, use `Adaptive` for
    /// 1.5-2x throughput improvements.
    pub batch_size_strategy: BatchSizeStrategy,

    /// Enable parallel batch execution (default: false for backward compatibility).
    ///
    /// When enabled, independent node type batches are executed in parallel
    /// using async tasks. This provides 3-5x throughput improvements for
    /// large datasets (>10K nodes) at the cost of higher connection pool usage.
    ///
    /// **Requirements**: Requires async feature and adequate Neo4j connection pool size.
    ///
    /// Recommended for:
    /// - Large datasets (>10,000 nodes)
    /// - Multiple node types that can be created independently
    /// - Systems with good network bandwidth to Neo4j
    pub parallel_execution: bool,

    /// Maximum number of parallel batch tasks (default: 10).
    ///
    /// Controls the degree of parallelism for batch execution.
    /// Higher values increase throughput but require more connections.
    ///
    /// Recommended values:
    /// - Local Neo4j: 10-20
    /// - Remote Neo4j with good network: 20-30
    /// - Limited connection pool: 5-10
    pub max_parallel_batches: usize,

    /// Query pipeline depth for concurrent in-flight queries (default: 10).
    ///
    /// Controls how many queries can be submitted concurrently without
    /// waiting for responses. Higher values reduce network idle time but
    /// increase memory usage.
    ///
    /// This provides 40-60% reduction in network overhead when RTT is
    /// significant relative to query execution time.
    ///
    /// Recommended values:
    /// - Local Neo4j (low RTT): 5-10
    /// - Remote Neo4j (high RTT): 20-50
    /// - Memory constrained: 5
    pub pipeline_depth: usize,

    /// Include type metadata property (default: false).
    pub include_type_metadata: bool,

    /// Property name for type metadata (default: "_`hedl_type`").
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
    /// - Production (trusted data): `Some(100_000_000)` (100MB, default)
    /// - Production (untrusted data): `Some(1_000_000)` (1MB, use `for_untrusted_input()`)
    /// - Strict: `Some(1_000_000)` (1MB)
    /// - Development: None (unlimited)
    pub max_string_length: Option<usize>,

    /// Maximum number of nodes to process (default: None = unlimited).
    ///
    /// **SECURITY**: This limit is enforced at the start of conversion.
    /// If exceeded, `Neo4jError::NodeCountExceeded` is returned before
    /// any memory allocation for node conversion occurs.
    ///
    /// The count includes:
    /// - All rows in `MatrixLists`
    /// - All nested children in NEST hierarchies
    ///
    /// Recommended values:
    /// - Production (trusted data): None (unlimited)
    /// - Production (untrusted data): `Some(100_000)` (use `for_untrusted_input()`)
    /// - API endpoint: `Some(10_000)` - `Some(100_000)` based on expected load
    pub max_nodes: Option<usize>,

    /// How to render Cypher parameters (default: Inline for backward compatibility).
    ///
    /// **SECURITY**: For maximum security when processing untrusted input,
    /// use `RenderMode::Parameterized`. This keeps query text and data
    /// completely separate, preventing any possibility of injection.
    ///
    /// Modes:
    /// - `RenderMode::Inline` (default): Parameters substituted into query text
    /// - `RenderMode::Parameterized`: Query uses `$param` placeholders, data separate
    ///
    /// **Recommendation**: Use `with_parameterized_mode()` for untrusted input.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::{ToCypherConfig, RenderMode};
    /// // Inline mode (backward compatible, all values escaped)
    /// let config = ToCypherConfig::default();  // Inline by default
    ///
    /// // Parameterized mode (most secure, recommended for untrusted input)
    /// let config = ToCypherConfig::default()
    ///     .with_parameterized_mode();
    /// ```
    pub render_mode: RenderMode,

    /// Enable streaming child collection (default: true).
    ///
    /// When enabled, child nodes from NEST hierarchies are processed
    /// incrementally using an iterator, reducing peak memory usage by ~99%
    /// for large hierarchical documents.
    ///
    /// When disabled, uses legacy eager collection (deprecated).
    ///
    /// # Memory Impact
    ///
    /// - Streaming (true): Peak memory = `O(batch_size)`
    /// - Eager (false): Peak memory = `O(total_children)`
    ///
    /// # Backward Compatibility
    ///
    /// Set to `false` to preserve legacy behavior during migration.
    /// Will be removed in a future major version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// // Default: streaming enabled for better performance
    /// let config = ToCypherConfig::default();
    /// assert!(config.streaming_children);
    ///
    /// // Disable streaming for backward compatibility
    /// let config = ToCypherConfig::default()
    ///     .without_streaming_children();
    /// ```
    pub streaming_children: bool,

    // ========== Transaction Batching Options ==========
    /// Enable transaction batching for grouped statement execution (default: false).
    ///
    /// When enabled, multiple Cypher statements are grouped into single transactions,
    /// reducing network round-trips and improving throughput for large imports.
    pub transaction_batching_enabled: bool,

    /// Number of statements per transaction batch (default: 100).
    pub transaction_batch_size: usize,

    /// Maximum rows per transaction (default: 10000).
    pub transaction_row_limit: usize,

    /// Strategy for batching transactions.
    pub transaction_strategy: TransactionStrategy,

    /// Isolation level for transactions.
    pub transaction_isolation: IsolationLevel,

    // ========== Query Optimization Options ==========
    /// Use index hints in generated queries (default: true).
    pub use_index_hints: bool,

    /// Enable query template caching (default: true).
    pub enable_template_caching: bool,

    /// Enable adaptive performance tracking (default: false).
    pub enable_adaptive_tracking: bool,
}

impl Default for ToCypherConfig {
    fn default() -> Self {
        Self {
            use_merge: true,
            create_constraints: true,
            create_indexes: true, // Safe default - significant perf improvement
            create_relationship_indexes: false, // Opt-in for specific workloads
            create_composite_indexes: false, // Opt-in for production
            indexed_properties: vec![],
            reference_naming: RelationshipNaming::PropertyName,
            nest_naming: RelationshipNaming::PropertyName,
            object_handling: ObjectHandling::Flatten,
            id_property: "_hedl_id".to_string(),
            batch_size: 1000,
            batch_size_strategy: BatchSizeStrategy::default(),
            parallel_execution: false,
            max_parallel_batches: 10,
            pipeline_depth: 10,
            include_type_metadata: false,
            type_property: "_hedl_type".to_string(),
            include_comments: true,
            max_string_length: Some(DEFAULT_MAX_STRING_LENGTH), // 100MB default
            max_nodes: None,
            render_mode: RenderMode::default(), // Inline by default for backward compatibility
            streaming_children: true,           // Default to streaming for better performance
            // Transaction batching defaults
            transaction_batching_enabled: false, // Opt-in for performance
            transaction_batch_size: DEFAULT_TRANSACTION_BATCH_SIZE,
            transaction_row_limit: DEFAULT_TRANSACTION_ROW_LIMIT,
            transaction_strategy: TransactionStrategy::default(),
            transaction_isolation: IsolationLevel::default(),
            // Query optimization defaults
            use_index_hints: true,           // Guaranteed index usage
            enable_template_caching: true,   // Query plan reuse
            enable_adaptive_tracking: false, // Opt-in for long imports
        }
    }
}

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
    /// Create a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

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

    /// Use CREATE instead of MERGE.
    #[must_use]
    pub fn with_create(mut self) -> Self {
        self.use_merge = false;
        self
    }

    /// Disable constraint generation.
    #[must_use]
    pub fn without_constraints(mut self) -> Self {
        self.create_constraints = false;
        self
    }

    /// Disable all index creation.
    #[must_use]
    pub fn without_indexes(mut self) -> Self {
        self.create_indexes = false;
        self.create_relationship_indexes = false;
        self.create_composite_indexes = false;
        self
    }

    /// Enable all index types.
    #[must_use]
    pub fn with_all_indexes(mut self) -> Self {
        self.create_indexes = true;
        self.create_relationship_indexes = true;
        self.create_composite_indexes = true;
        self
    }

    /// Add a property to the indexed properties list.
    pub fn with_indexed_property(mut self, property: impl Into<String>) -> Self {
        self.indexed_properties.push(property.into());
        self
    }

    /// Set the ID property name.
    pub fn with_id_property(mut self, name: impl Into<String>) -> Self {
        self.id_property = name.into();
        self
    }

    /// Set the batch size for UNWIND statements.
    #[must_use]
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self.batch_size_strategy = BatchSizeStrategy::Fixed(size);
        self
    }

    /// Set the batch size strategy.
    #[must_use]
    pub fn with_batch_size_strategy(mut self, strategy: BatchSizeStrategy) -> Self {
        self.batch_size_strategy = strategy;
        self
    }

    /// Enable adaptive batch sizing with default settings.
    ///
    /// Uses 512KB target batch size with min=100, max=5000.
    /// Provides 1.5-2x throughput improvements for varied node sizes.
    #[must_use]
    pub fn with_adaptive_batch_sizing(mut self) -> Self {
        self.batch_size_strategy = BatchSizeStrategy::Adaptive {
            target_batch_bytes: 524_288, // 512KB
            min_batch_size: 100,
            max_batch_size: 5000,
        };
        self
    }

    /// Enable parallel batch execution.
    ///
    /// Provides 3-5x throughput improvements for large datasets (>10K nodes).
    /// Requires async feature and adequate Neo4j connection pool.
    #[must_use]
    pub fn with_parallel_execution(mut self) -> Self {
        self.parallel_execution = true;
        self
    }

    /// Set maximum number of parallel batch tasks.
    #[must_use]
    pub fn with_max_parallel_batches(mut self, max: usize) -> Self {
        self.max_parallel_batches = max;
        self
    }

    /// Set query pipeline depth.
    #[must_use]
    pub fn with_pipeline_depth(mut self, depth: usize) -> Self {
        self.pipeline_depth = depth;
        self
    }

    /// Enable performance optimizations for large datasets.
    ///
    /// Combines:
    /// - Parallel execution (3-5x improvement)
    /// - Adaptive batch sizing (1.5-2x improvement)
    /// - Increased pipeline depth (40-60% network overhead reduction)
    ///
    /// Expected combined improvement: 5-10x for large datasets (>10K nodes).
    #[must_use]
    pub fn with_performance_optimizations(mut self) -> Self {
        self.parallel_execution = true;
        self.batch_size_strategy = BatchSizeStrategy::Adaptive {
            target_batch_bytes: 524_288, // 512KB
            min_batch_size: 100,
            max_batch_size: 5000,
        };
        self.pipeline_depth = 20;
        self.max_parallel_batches = 10;
        self
    }

    /// Use JSON strings for nested objects.
    #[must_use]
    pub fn with_json_objects(mut self) -> Self {
        self.object_handling = ObjectHandling::JsonString;
        self
    }

    /// Include type metadata in nodes.
    #[must_use]
    pub fn with_type_metadata(mut self) -> Self {
        self.include_type_metadata = true;
        self
    }

    /// Disable comments in output.
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn without_string_length_limit(mut self) -> Self {
        self.max_string_length = None;
        self
    }

    /// Set maximum number of nodes to process.
    #[must_use]
    pub fn with_max_nodes(mut self, max: usize) -> Self {
        self.max_nodes = Some(max);
        self
    }

    /// Enable parameterized query mode for maximum security.
    ///
    /// This configures queries to use `$param` placeholders with separate parameter binding.
    /// This is the most secure mode as data never touches the query text.
    ///
    /// **SECURITY:** Use this mode for untrusted input to prevent any possibility of
    /// Cypher injection, even in theoretical edge cases.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// let config = ToCypherConfig::new()
    ///     .with_parameterized_mode();
    ///
    /// // Statements will have query with placeholders and separate parameters
    /// // let (query, params) = statement.render_parameterized();
    /// ```
    #[must_use]
    pub fn with_parameterized_mode(mut self) -> Self {
        self.render_mode = RenderMode::Parameterized;
        self
    }

    /// Enable inline rendering mode (backward compatible).
    ///
    /// This configures queries to substitute parameter values directly into the query text.
    /// All values are properly escaped, but true parameterized queries are more secure.
    ///
    /// This is the default mode for backward compatibility.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// let config = ToCypherConfig::new()
    ///     .with_inline_mode();
    ///
    /// // Statements will render with values inlined (all escaped)
    /// // let cypher = statement.render_inline();
    /// ```
    #[must_use]
    pub fn with_inline_mode(mut self) -> Self {
        self.render_mode = RenderMode::Inline;
        self
    }

    /// Enable streaming child collection (default: true).
    ///
    /// When enabled, child nodes from NEST hierarchies are processed incrementally,
    /// reducing peak memory usage by ~99% for large hierarchies.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// let config = ToCypherConfig::new()
    ///     .with_streaming_children();
    /// ```
    #[must_use]
    pub fn with_streaming_children(mut self) -> Self {
        self.streaming_children = true;
        self
    }

    /// Disable streaming child collection.
    ///
    /// Uses legacy eager collection (deprecated). Only use for backward compatibility.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// // Disable streaming for backward compatibility
    /// let config = ToCypherConfig::new()
    ///     .without_streaming_children();
    /// ```
    #[must_use]
    pub fn without_streaming_children(mut self) -> Self {
        self.streaming_children = false;
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
    /// - Parameterized mode enabled (most secure)
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// // For processing user-uploaded data
    /// let config = ToCypherConfig::for_untrusted_input();
    /// ```
    #[must_use]
    pub fn for_untrusted_input() -> Self {
        Self {
            max_string_length: Some(1_000_000),
            max_nodes: Some(100_000),
            batch_size: 100,
            include_comments: false,
            render_mode: RenderMode::Parameterized, // Most secure for untrusted input
            ..Default::default()
        }
    }

    /// Create configuration optimized for production workloads.
    ///
    /// Enables all index types for maximum query performance.
    ///
    /// # Trade-offs
    ///
    /// - **Benefits:** 5-100x faster queries depending on workload
    /// - **Costs:** Slower writes, increased storage, index maintenance
    ///
    /// # What's Enabled
    ///
    /// - ID property indexes (100x faster lookups)
    /// - Relationship indexes (5-20x faster traversals)
    /// - Composite indexes (20-100x faster complex queries)
    /// - Property indexes for common fields (name, email)
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// let config = ToCypherConfig::for_production();
    /// ```
    #[must_use]
    pub fn for_production() -> Self {
        Self {
            create_indexes: true,
            create_relationship_indexes: true,
            create_composite_indexes: true,
            indexed_properties: vec!["name".to_string(), "email".to_string()],
            ..Default::default()
        }
    }

    /// Create configuration optimized for bulk import.
    ///
    /// Minimal indexes during import, can be added later.
    ///
    /// # Strategy
    ///
    /// For large datasets (>100K nodes), it's faster to:
    /// 1. Import with constraints only (this config)
    /// 2. Add indexes after import completes
    ///
    /// # Trade-offs
    ///
    /// - **Benefits:** Fast imports, no index maintenance during load
    /// - **Costs:** Slower queries until indexes added post-import
    ///
    /// # Post-Import Steps
    ///
    /// After import, re-run with `for_production()` config to add indexes:
    ///
    /// ```ignore
    /// // During import
    /// let import_config = ToCypherConfig::for_bulk_import();
    /// // ... import data ...
    ///
    /// // After import (to add indexes)
    /// let index_config = ToCypherConfig::for_production();
    /// // Generate and execute index statements only
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// let config = ToCypherConfig::for_bulk_import();
    /// ```
    #[must_use]
    pub fn for_bulk_import() -> Self {
        Self {
            create_constraints: true,
            create_indexes: false, // Add indexes post-import
            create_relationship_indexes: false,
            create_composite_indexes: false,
            ..Default::default()
        }
    }

    /// Create configuration optimized for high-throughput imports.
    ///
    /// Enables transaction batching with aggressive settings for maximum
    /// network efficiency. Best for large datasets (>100K nodes) on network-bound
    /// connections.
    ///
    /// # What's Enabled
    ///
    /// - Transaction batching with 200 statements per batch
    /// - Row-based batching for even transaction sizes
    /// - 20,000 row limit per transaction
    /// - Minimal indexes (added post-import)
    /// - Large UNWIND batches (5,000)
    ///
    /// # Performance Characteristics
    ///
    /// - **30-50% faster** than default for large imports
    /// - **60% fewer network round-trips**
    /// - **Higher memory usage** during import
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// let config = ToCypherConfig::for_high_throughput();
    /// ```
    #[must_use]
    pub fn for_high_throughput() -> Self {
        Self {
            // Transaction batching for network efficiency
            transaction_batching_enabled: true,
            transaction_batch_size: 200,
            transaction_row_limit: 20_000,
            transaction_strategy: TransactionStrategy::RowCount,
            // Large UNWIND batches
            batch_size: 5000,
            // Minimal indexes during import
            create_indexes: false,
            create_relationship_indexes: false,
            create_composite_indexes: false,
            ..Default::default()
        }
    }

    /// Enable or disable transaction batching.
    ///
    /// When enabled, multiple Cypher statements are grouped into single transactions,
    /// reducing network round-trips and improving throughput.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// let config = ToCypherConfig::new()
    ///     .with_transaction_batching(true);
    /// ```
    #[must_use]
    pub fn with_transaction_batching(mut self, enabled: bool) -> Self {
        self.transaction_batching_enabled = enabled;
        self
    }

    /// Set the number of statements per transaction batch.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// let config = ToCypherConfig::new()
    ///     .with_transaction_batching(true)
    ///     .with_transaction_batch_size(50);
    /// ```
    #[must_use]
    pub fn with_transaction_batch_size(mut self, size: usize) -> Self {
        self.transaction_batch_size = size;
        self
    }

    /// Set the transaction batching strategy.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::{ToCypherConfig, TransactionStrategy};
    /// let config = ToCypherConfig::new()
    ///     .with_transaction_batching(true)
    ///     .with_transaction_strategy(TransactionStrategy::RowCount);
    /// ```
    #[must_use]
    pub fn with_transaction_strategy(mut self, strategy: TransactionStrategy) -> Self {
        self.transaction_strategy = strategy;
        self
    }

    /// Set the maximum rows per transaction.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// let config = ToCypherConfig::new()
    ///     .with_transaction_batching(true)
    ///     .with_transaction_row_limit(500);
    /// ```
    #[must_use]
    pub fn with_transaction_row_limit(mut self, limit: usize) -> Self {
        self.transaction_row_limit = limit;
        self
    }

    /// Enable or disable index hints in generated Cypher.
    ///
    /// When enabled, generates USING INDEX hints for optimized query plans.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// let config = ToCypherConfig::new()
    ///     .with_index_hints(false); // Disable index hints
    /// ```
    #[must_use]
    pub fn with_index_hints(mut self, enabled: bool) -> Self {
        self.use_index_hints = enabled;
        self
    }

    /// Enable or disable template caching for query optimization.
    ///
    /// When enabled, query templates are cached for reuse, improving performance.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_neo4j::ToCypherConfig;
    /// let config = ToCypherConfig::new()
    ///     .with_template_caching(true);
    /// ```
    #[must_use]
    pub fn with_template_caching(mut self, enabled: bool) -> Self {
        self.enable_template_caching = enabled;
        self
    }
}

/// Default batch size for streaming record processing: 1000 records.
pub const DEFAULT_FROM_NEO4J_BATCH_SIZE: usize = 1000;

/// Configuration for converting Neo4j records to HEDL documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FromNeo4jConfig {
    /// HEDL version for the output document (default: (1, 0)).
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
            version: (1, 0),
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
    /// Create a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

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
        assert_eq!(config.render_mode, RenderMode::Parameterized); // Most secure
    }

    #[test]
    fn test_render_mode_config() {
        // Default is Inline
        let config = ToCypherConfig::default();
        assert_eq!(config.render_mode, RenderMode::Inline);

        // Can set to Parameterized
        let config = ToCypherConfig::new().with_parameterized_mode();
        assert_eq!(config.render_mode, RenderMode::Parameterized);

        // Can explicitly set to Inline
        let config = ToCypherConfig::new().with_inline_mode();
        assert_eq!(config.render_mode, RenderMode::Inline);
    }

    #[test]
    fn test_builder_render_mode() {
        let config = ToCypherConfig::builder()
            .render_mode(RenderMode::Parameterized)
            .build();

        assert_eq!(config.render_mode, RenderMode::Parameterized);
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
        assert!(config.use_merge);
        assert!(config.create_constraints);
        assert_eq!(config.id_property, "_hedl_id");
        assert_eq!(config.batch_size, 1000);
        assert!(config.include_comments);
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

        assert!(!config.use_merge);
        assert!(!config.create_constraints);
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

        assert!(config.use_merge);
        assert!(!config.create_constraints);
        assert_eq!(config.reference_naming, RelationshipNaming::Generic);
        assert_eq!(config.nest_naming, RelationshipNaming::TargetType);
        assert_eq!(config.object_handling, ObjectHandling::JsonString);
        assert!(config.include_type_metadata);
        assert_eq!(config.type_property, "custom_type");
        assert!(!config.include_comments);
        assert_eq!(config.max_string_length, Some(5000));
        assert_eq!(config.max_nodes, Some(10000));
    }

    #[test]
    fn test_to_cypher_builder_string_limits() {
        let config = ToCypherConfig::builder().max_string_length(1000000).build();
        assert_eq!(config.max_string_length, Some(1000000));

        let config = ToCypherConfig::builder().no_string_length_limit().build();
        assert_eq!(config.max_string_length, None);
    }

    #[test]
    fn test_to_cypher_builder_new() {
        let builder = ToCypherConfigBuilder::new();
        let config = builder.build();
        assert!(config.use_merge); // Default value
    }

    // FromNeo4jConfigBuilder tests
    #[test]
    fn test_from_neo4j_builder_defaults() {
        let config = FromNeo4jConfig::builder().build();
        assert_eq!(config.version, (1, 0));
        assert_eq!(config.id_property, "_hedl_id");
        assert!(config.infer_nests);
        assert!(config.fallback_id);
        assert!(config.exclude_properties.is_empty());
        assert!(config.exclude_labels.is_empty());
        assert!(config.reference_relationships.is_empty());
        assert_eq!(config.batch_size, DEFAULT_FROM_NEO4J_BATCH_SIZE);
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
        assert!(!config.infer_nests);
        assert!(!config.fallback_id);
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
        assert!(config.infer_nests);
        assert_eq!(config.type_property, "custom_type");
        assert_eq!(config.exclude_properties.len(), 2);
        assert!(config.exclude_properties.contains(&"internal".to_string()));
        assert!(config.exclude_properties.contains(&"temp".to_string()));
        assert_eq!(config.exclude_labels.len(), 2);
        assert!(config.exclude_labels.contains(&"System".to_string()));
        assert!(config.exclude_labels.contains(&"Internal".to_string()));
        assert_eq!(config.reference_relationships.len(), 2);
        assert!(config
            .reference_relationships
            .contains(&"AUTHORED_BY".to_string()));
        assert!(config
            .reference_relationships
            .contains(&"CREATED_BY".to_string()));
        assert!(config.fallback_id);
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

    #[test]
    fn test_from_neo4j_batch_size() {
        // Default batch size
        let config = FromNeo4jConfig::default();
        assert_eq!(config.batch_size, DEFAULT_FROM_NEO4J_BATCH_SIZE);
        assert_eq!(config.batch_size, 1000);

        // Fluent method
        let config = FromNeo4jConfig::new().with_batch_size(500);
        assert_eq!(config.batch_size, 500);

        // Higher batch size for throughput
        let config = FromNeo4jConfig::new().with_batch_size(2000);
        assert_eq!(config.batch_size, 2000);

        // Builder pattern
        let config = FromNeo4jConfig::builder().batch_size(3000).build();
        assert_eq!(config.batch_size, 3000);
    }

    // Performance optimization configuration tests

    #[test]
    fn test_batch_size_strategy_default() {
        let config = ToCypherConfig::default();
        assert_eq!(config.batch_size_strategy, BatchSizeStrategy::Fixed(1000));
    }

    #[test]
    fn test_batch_size_strategy_adaptive() {
        let config = ToCypherConfig::default().with_adaptive_batch_sizing();
        assert_eq!(
            config.batch_size_strategy,
            BatchSizeStrategy::Adaptive {
                target_batch_bytes: 524_288,
                min_batch_size: 100,
                max_batch_size: 5000,
            }
        );
    }

    #[test]
    fn test_batch_size_strategy_custom() {
        let config = ToCypherConfig::builder()
            .batch_size_strategy(BatchSizeStrategy::Adaptive {
                target_batch_bytes: 1_048_576,
                min_batch_size: 50,
                max_batch_size: 10000,
            })
            .build();

        match config.batch_size_strategy {
            BatchSizeStrategy::Adaptive {
                target_batch_bytes,
                min_batch_size,
                max_batch_size,
            } => {
                assert_eq!(target_batch_bytes, 1_048_576);
                assert_eq!(min_batch_size, 50);
                assert_eq!(max_batch_size, 10000);
            }
            _ => panic!("Expected Adaptive strategy"),
        }
    }

    #[test]
    fn test_parallel_execution_default() {
        let config = ToCypherConfig::default();
        assert!(!config.parallel_execution);
        assert_eq!(config.max_parallel_batches, 10);
        assert_eq!(config.pipeline_depth, 10);
    }

    #[test]
    fn test_parallel_execution_enabled() {
        let config = ToCypherConfig::default().with_parallel_execution();
        assert!(config.parallel_execution);
    }

    #[test]
    fn test_parallel_execution_custom() {
        let config = ToCypherConfig::builder()
            .parallel_execution(true)
            .max_parallel_batches(20)
            .pipeline_depth(30)
            .build();

        assert!(config.parallel_execution);
        assert_eq!(config.max_parallel_batches, 20);
        assert_eq!(config.pipeline_depth, 30);
    }

    #[test]
    fn test_performance_optimizations() {
        let config = ToCypherConfig::default().with_performance_optimizations();

        assert!(config.parallel_execution);
        assert_eq!(config.max_parallel_batches, 10);
        assert_eq!(config.pipeline_depth, 20);

        match config.batch_size_strategy {
            BatchSizeStrategy::Adaptive {
                target_batch_bytes,
                min_batch_size,
                max_batch_size,
            } => {
                assert_eq!(target_batch_bytes, 524_288);
                assert_eq!(min_batch_size, 100);
                assert_eq!(max_batch_size, 5000);
            }
            _ => panic!("Expected Adaptive strategy"),
        }
    }

    #[test]
    fn test_builder_all_new_fields() {
        let config = ToCypherConfig::builder()
            .batch_size_strategy(BatchSizeStrategy::Fixed(2000))
            .parallel_execution(true)
            .max_parallel_batches(15)
            .pipeline_depth(25)
            .build();

        assert_eq!(config.batch_size_strategy, BatchSizeStrategy::Fixed(2000));
        assert!(config.parallel_execution);
        assert_eq!(config.max_parallel_batches, 15);
        assert_eq!(config.pipeline_depth, 25);
    }

    #[test]
    fn test_backward_compatibility_batch_size() {
        let config = ToCypherConfig::default();
        // batch_size should still be available for backward compatibility
        assert_eq!(config.batch_size, 1000);

        let config = ToCypherConfig::builder().batch_size(500).build();
        assert_eq!(config.batch_size, 500);
    }

    // Transaction batching and query optimization tests

    #[test]
    fn test_transaction_batching_defaults() {
        let config = ToCypherConfig::default();
        assert!(!config.transaction_batching_enabled);
        assert_eq!(
            config.transaction_batch_size,
            DEFAULT_TRANSACTION_BATCH_SIZE
        );
        assert_eq!(config.transaction_row_limit, DEFAULT_TRANSACTION_ROW_LIMIT);
        assert_eq!(
            config.transaction_strategy,
            TransactionStrategy::StatementCount
        );
        assert_eq!(config.transaction_isolation, IsolationLevel::Default);
    }

    #[test]
    fn test_query_optimization_defaults() {
        let config = ToCypherConfig::default();
        assert!(config.use_index_hints);
        assert!(config.enable_template_caching);
        assert!(!config.enable_adaptive_tracking);
    }

    #[test]
    fn test_for_high_throughput() {
        let config = ToCypherConfig::for_high_throughput();
        assert!(config.transaction_batching_enabled);
        assert_eq!(config.transaction_batch_size, 200);
        assert_eq!(config.transaction_row_limit, 20_000);
        assert_eq!(config.transaction_strategy, TransactionStrategy::RowCount);
        assert_eq!(config.batch_size, 5000);
        assert!(!config.create_indexes);
    }
}

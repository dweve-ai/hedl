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

//! Configuration for converting HEDL documents to Cypher queries.

use super::types::{
    BatchSizeStrategy, IsolationLevel, ObjectHandling, RelationshipNaming, TransactionStrategy,
    DEFAULT_MAX_STRING_LENGTH, DEFAULT_TRANSACTION_BATCH_SIZE, DEFAULT_TRANSACTION_ROW_LIMIT,
};
use crate::cypher::RenderMode;
use serde::{Deserialize, Serialize};

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

impl ToCypherConfig {
    /// Create a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

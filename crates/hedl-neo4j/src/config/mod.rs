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

mod from_neo4j_builder;
mod from_neo4j_config;
mod to_cypher_builder;
mod to_cypher_config;
mod types;

// Re-export all public types
pub use from_neo4j_builder::FromNeo4jConfigBuilder;
pub use from_neo4j_config::FromNeo4jConfig;
pub use to_cypher_builder::ToCypherConfigBuilder;
pub use to_cypher_config::ToCypherConfig;
pub use types::{
    BatchSizeStrategy, IsolationLevel, ObjectHandling, RelationshipNaming, TransactionStrategy,
    DEFAULT_FROM_NEO4J_BATCH_SIZE, DEFAULT_MAX_STRING_LENGTH, DEFAULT_TRANSACTION_BATCH_SIZE,
    DEFAULT_TRANSACTION_ROW_LIMIT,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cypher::RenderMode;

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
        assert_eq!(config.version, (2, 0));
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
        assert_eq!(config.version, (2, 0));
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
        assert_eq!(config.version, (2, 0)); // Default value
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

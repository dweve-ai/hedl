// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for configuration types.

use hedl_neo4j::config::{
    BatchSizeStrategy, FromNeo4jConfig, IsolationLevel, ObjectHandling, RelationshipNaming,
    ToCypherConfig, TransactionStrategy, DEFAULT_FROM_NEO4J_BATCH_SIZE, DEFAULT_MAX_STRING_LENGTH,
    DEFAULT_TRANSACTION_BATCH_SIZE, DEFAULT_TRANSACTION_ROW_LIMIT,
};
use hedl_neo4j::cypher::RenderMode;

#[test]
fn test_transaction_strategy_variants() {
    let statement_count = TransactionStrategy::StatementCount;
    let row_count = TransactionStrategy::RowCount;
    let statement_type = TransactionStrategy::StatementType;
    let adaptive = TransactionStrategy::Adaptive;

    assert_eq!(statement_count, TransactionStrategy::StatementCount);
    assert_eq!(row_count, TransactionStrategy::RowCount);
    assert_eq!(statement_type, TransactionStrategy::StatementType);
    assert_eq!(adaptive, TransactionStrategy::Adaptive);

    assert_ne!(statement_count, row_count);
    assert_ne!(row_count, adaptive);

    // Test default
    assert_eq!(
        TransactionStrategy::default(),
        TransactionStrategy::StatementCount
    );
}

#[test]
fn test_isolation_level_variants() {
    let default = IsolationLevel::Default;
    let serializable = IsolationLevel::Serializable;

    assert_eq!(default, IsolationLevel::Default);
    assert_eq!(serializable, IsolationLevel::Serializable);
    assert_ne!(default, serializable);

    // Test default
    assert_eq!(IsolationLevel::default(), IsolationLevel::Default);
}

#[test]
fn test_relationship_naming_variants() {
    assert_eq!(
        RelationshipNaming::default(),
        RelationshipNaming::PropertyName
    );
    assert_ne!(
        RelationshipNaming::PropertyName,
        RelationshipNaming::Generic
    );
    assert_ne!(RelationshipNaming::Generic, RelationshipNaming::TargetType);
}

#[test]
fn test_object_handling_variants() {
    assert_eq!(ObjectHandling::default(), ObjectHandling::Flatten);
    assert_ne!(ObjectHandling::Flatten, ObjectHandling::JsonString);
}

#[test]
fn test_batch_size_strategy_fixed() {
    let strategy = BatchSizeStrategy::Fixed(1000);
    assert_eq!(strategy, BatchSizeStrategy::Fixed(1000));
    assert_ne!(strategy, BatchSizeStrategy::Fixed(500));

    // Test default
    assert_eq!(BatchSizeStrategy::default(), BatchSizeStrategy::Fixed(1000));
}

#[test]
fn test_batch_size_strategy_adaptive() {
    let strategy = BatchSizeStrategy::Adaptive {
        target_batch_bytes: 512_000,
        min_batch_size: 100,
        max_batch_size: 5000,
    };

    match strategy {
        BatchSizeStrategy::Adaptive {
            target_batch_bytes,
            min_batch_size,
            max_batch_size,
        } => {
            assert_eq!(target_batch_bytes, 512_000);
            assert_eq!(min_batch_size, 100);
            assert_eq!(max_batch_size, 5000);
        }
        _ => panic!("Expected Adaptive strategy"),
    }
}

#[test]
fn test_to_cypher_config_constants() {
    assert_eq!(DEFAULT_MAX_STRING_LENGTH, 100 * 1024 * 1024);
    assert_eq!(DEFAULT_TRANSACTION_BATCH_SIZE, 100);
    assert_eq!(DEFAULT_TRANSACTION_ROW_LIMIT, 10_000);
}

#[test]
fn test_to_cypher_config_with_create_and_merge() {
    let create_config = ToCypherConfig::new().with_create();
    assert!(!create_config.use_merge);

    let merge_config = ToCypherConfig::default();
    assert!(merge_config.use_merge);
}

#[test]
fn test_to_cypher_config_constraints() {
    let no_constraints = ToCypherConfig::new().without_constraints();
    assert!(!no_constraints.create_constraints);

    let with_constraints = ToCypherConfig::default();
    assert!(with_constraints.create_constraints);
}

#[test]
fn test_to_cypher_config_indexes() {
    let no_indexes = ToCypherConfig::new().without_indexes();
    assert!(!no_indexes.create_indexes);
    assert!(!no_indexes.create_relationship_indexes);
    assert!(!no_indexes.create_composite_indexes);

    let all_indexes = ToCypherConfig::new().with_all_indexes();
    assert!(all_indexes.create_indexes);
    assert!(all_indexes.create_relationship_indexes);
    assert!(all_indexes.create_composite_indexes);
}

#[test]
fn test_to_cypher_config_indexed_properties() {
    let config = ToCypherConfig::new()
        .with_indexed_property("email")
        .with_indexed_property("username");

    assert_eq!(config.indexed_properties.len(), 2);
    assert!(config.indexed_properties.contains(&"email".to_string()));
    assert!(config.indexed_properties.contains(&"username".to_string()));
}

#[test]
fn test_to_cypher_config_id_property() {
    let config = ToCypherConfig::new().with_id_property("nodeId");
    assert_eq!(config.id_property, "nodeId");
}

#[test]
fn test_to_cypher_config_batch_size() {
    let config = ToCypherConfig::new().with_batch_size(500);
    assert_eq!(config.batch_size, 500);
    assert_eq!(config.batch_size_strategy, BatchSizeStrategy::Fixed(500));
}

#[test]
fn test_to_cypher_config_adaptive_batching() {
    let config = ToCypherConfig::new().with_adaptive_batch_sizing();

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
fn test_to_cypher_config_parallel_execution() {
    let config = ToCypherConfig::new().with_parallel_execution();
    assert!(config.parallel_execution);

    let with_max = config.with_max_parallel_batches(20);
    assert_eq!(with_max.max_parallel_batches, 20);
}

#[test]
fn test_to_cypher_config_pipeline_depth() {
    let config = ToCypherConfig::new().with_pipeline_depth(30);
    assert_eq!(config.pipeline_depth, 30);
}

#[test]
fn test_to_cypher_config_performance_optimizations() {
    let config = ToCypherConfig::new().with_performance_optimizations();

    assert!(config.parallel_execution);
    assert_eq!(config.max_parallel_batches, 10);
    assert_eq!(config.pipeline_depth, 20);

    match config.batch_size_strategy {
        BatchSizeStrategy::Adaptive { .. } => {}
        _ => panic!("Expected Adaptive strategy"),
    }
}

#[test]
fn test_to_cypher_config_object_handling() {
    let json_config = ToCypherConfig::new().with_json_objects();
    assert_eq!(json_config.object_handling, ObjectHandling::JsonString);

    let flatten_config = ToCypherConfig::default();
    assert_eq!(flatten_config.object_handling, ObjectHandling::Flatten);
}

#[test]
fn test_to_cypher_config_type_metadata() {
    let with_metadata = ToCypherConfig::new().with_type_metadata();
    assert!(with_metadata.include_type_metadata);

    let without_metadata = ToCypherConfig::default();
    assert!(!without_metadata.include_type_metadata);
}

#[test]
fn test_to_cypher_config_comments() {
    let no_comments = ToCypherConfig::new().without_comments();
    assert!(!no_comments.include_comments);

    let with_comments = ToCypherConfig::default();
    assert!(with_comments.include_comments);
}

#[test]
fn test_to_cypher_config_string_length_limits() {
    let limited = ToCypherConfig::new().with_max_string_length(50_000);
    assert_eq!(limited.max_string_length, Some(50_000));

    let unlimited = ToCypherConfig::new().without_string_length_limit();
    assert_eq!(unlimited.max_string_length, None);
}

#[test]
fn test_to_cypher_config_node_limits() {
    let limited = ToCypherConfig::new().with_max_nodes(10_000);
    assert_eq!(limited.max_nodes, Some(10_000));

    let unlimited = ToCypherConfig::default();
    assert_eq!(unlimited.max_nodes, None);
}

#[test]
fn test_to_cypher_config_render_modes() {
    let inline = ToCypherConfig::new().with_inline_mode();
    assert_eq!(inline.render_mode, RenderMode::Inline);

    let parameterized = ToCypherConfig::new().with_parameterized_mode();
    assert_eq!(parameterized.render_mode, RenderMode::Parameterized);
}

#[test]
fn test_to_cypher_config_streaming_children() {
    let streaming = ToCypherConfig::new().with_streaming_children();
    assert!(streaming.streaming_children);

    let no_streaming = ToCypherConfig::new().without_streaming_children();
    assert!(!no_streaming.streaming_children);

    let default = ToCypherConfig::default();
    assert!(default.streaming_children);
}

#[test]
fn test_to_cypher_config_for_untrusted_input() {
    let config = ToCypherConfig::for_untrusted_input();

    assert_eq!(config.max_string_length, Some(1_000_000));
    assert_eq!(config.max_nodes, Some(100_000));
    assert_eq!(config.batch_size, 100);
    assert!(!config.include_comments);
    assert_eq!(config.render_mode, RenderMode::Parameterized);
}

#[test]
fn test_to_cypher_config_for_production() {
    let config = ToCypherConfig::for_production();

    assert!(config.create_indexes);
    assert!(config.create_relationship_indexes);
    assert!(config.create_composite_indexes);
    assert!(config.indexed_properties.contains(&"name".to_string()));
    assert!(config.indexed_properties.contains(&"email".to_string()));
}

#[test]
fn test_to_cypher_config_for_bulk_import() {
    let config = ToCypherConfig::for_bulk_import();

    assert!(config.create_constraints);
    assert!(!config.create_indexes);
    assert!(!config.create_relationship_indexes);
    assert!(!config.create_composite_indexes);
}

#[test]
fn test_to_cypher_config_for_high_throughput() {
    let config = ToCypherConfig::for_high_throughput();

    assert!(config.transaction_batching_enabled);
    assert_eq!(config.transaction_batch_size, 200);
    assert_eq!(config.transaction_row_limit, 20_000);
    assert_eq!(config.transaction_strategy, TransactionStrategy::RowCount);
    assert_eq!(config.batch_size, 5000);
    assert!(!config.create_indexes);
}

#[test]
fn test_to_cypher_config_builder_all_fields() {
    let config = ToCypherConfig::builder()
        .use_merge(false)
        .create_constraints(false)
        .create_indexes(true)
        .create_relationship_indexes(true)
        .create_composite_indexes(true)
        .indexed_property("email")
        .reference_naming(RelationshipNaming::Generic)
        .nest_naming(RelationshipNaming::TargetType)
        .object_handling(ObjectHandling::JsonString)
        .id_property("custom_id")
        .batch_size(750)
        .batch_size_strategy(BatchSizeStrategy::Fixed(750))
        .parallel_execution(true)
        .max_parallel_batches(15)
        .pipeline_depth(25)
        .include_type_metadata(true)
        .type_property("custom_type")
        .include_comments(false)
        .max_string_length(2_000_000)
        .max_nodes(50_000)
        .render_mode(RenderMode::Parameterized)
        .streaming_children(false)
        .transaction_batching_enabled(true)
        .transaction_batch_size(150)
        .transaction_row_limit(15_000)
        .transaction_strategy(TransactionStrategy::Adaptive)
        .transaction_isolation(IsolationLevel::Serializable)
        .use_index_hints(false)
        .enable_template_caching(false)
        .enable_adaptive_tracking(true)
        .build();

    assert!(!config.use_merge);
    assert!(!config.create_constraints);
    assert!(config.create_indexes);
    assert!(config.create_relationship_indexes);
    assert!(config.create_composite_indexes);
    assert!(config.indexed_properties.contains(&"email".to_string()));
    assert_eq!(config.reference_naming, RelationshipNaming::Generic);
    assert_eq!(config.nest_naming, RelationshipNaming::TargetType);
    assert_eq!(config.object_handling, ObjectHandling::JsonString);
    assert_eq!(config.id_property, "custom_id");
    assert_eq!(config.batch_size, 750);
    assert!(config.parallel_execution);
    assert_eq!(config.max_parallel_batches, 15);
    assert_eq!(config.pipeline_depth, 25);
    assert!(config.include_type_metadata);
    assert_eq!(config.type_property, "custom_type");
    assert!(!config.include_comments);
    assert_eq!(config.max_string_length, Some(2_000_000));
    assert_eq!(config.max_nodes, Some(50_000));
    assert_eq!(config.render_mode, RenderMode::Parameterized);
    assert!(!config.streaming_children);
    assert!(config.transaction_batching_enabled);
    assert_eq!(config.transaction_batch_size, 150);
    assert_eq!(config.transaction_row_limit, 15_000);
    assert_eq!(config.transaction_strategy, TransactionStrategy::Adaptive);
    assert_eq!(config.transaction_isolation, IsolationLevel::Serializable);
    assert!(!config.use_index_hints);
    assert!(!config.enable_template_caching);
    assert!(config.enable_adaptive_tracking);
}

#[test]
fn test_from_neo4j_config_default() {
    let config = FromNeo4jConfig::default();

    assert_eq!(config.version, (1, 0));
    assert_eq!(config.id_property, "_hedl_id");
    assert!(config.infer_nests);
    assert_eq!(config.type_property, "_hedl_type");
    assert!(config.exclude_properties.is_empty());
    assert!(config.exclude_labels.is_empty());
    assert!(config.reference_relationships.is_empty());
    assert!(config.fallback_id);
    assert_eq!(config.batch_size, DEFAULT_FROM_NEO4J_BATCH_SIZE);
}

#[test]
fn test_from_neo4j_config_fluent_api() {
    let config = FromNeo4jConfig::new()
        .with_version(2, 1)
        .with_id_property("nodeId")
        .without_nest_inference()
        .exclude_property("internal")
        .exclude_property("temp")
        .exclude_label("System")
        .reference_relationship("AUTHORED_BY")
        .without_fallback_id()
        .with_batch_size(2000);

    assert_eq!(config.version, (2, 1));
    assert_eq!(config.id_property, "nodeId");
    assert!(!config.infer_nests);
    assert_eq!(config.exclude_properties.len(), 2);
    assert!(config.exclude_properties.contains(&"internal".to_string()));
    assert!(config.exclude_properties.contains(&"temp".to_string()));
    assert_eq!(config.exclude_labels.len(), 1);
    assert!(config.exclude_labels.contains(&"System".to_string()));
    assert_eq!(config.reference_relationships.len(), 1);
    assert!(config
        .reference_relationships
        .contains(&"AUTHORED_BY".to_string()));
    assert!(!config.fallback_id);
    assert_eq!(config.batch_size, 2000);
}

#[test]
fn test_from_neo4j_config_builder() {
    let config = FromNeo4jConfig::builder()
        .version(3, 0)
        .id_property("customId")
        .infer_nests(false)
        .type_property("customType")
        .exclude_properties(vec!["prop1".to_string(), "prop2".to_string()])
        .exclude_labels(vec!["Label1".to_string(), "Label2".to_string()])
        .reference_relationships(vec!["REL1".to_string(), "REL2".to_string()])
        .fallback_id(false)
        .batch_size(3000)
        .build();

    assert_eq!(config.version, (3, 0));
    assert_eq!(config.id_property, "customId");
    assert!(!config.infer_nests);
    assert_eq!(config.type_property, "customType");
    assert_eq!(config.exclude_properties.len(), 2);
    assert_eq!(config.exclude_labels.len(), 2);
    assert_eq!(config.reference_relationships.len(), 2);
    assert!(!config.fallback_id);
    assert_eq!(config.batch_size, 3000);
}

#[test]
fn test_config_serialization_roundtrip() {
    let original = ToCypherConfig::for_production();
    let json = serde_json::to_string(&original).unwrap();
    let parsed: ToCypherConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(original.use_merge, parsed.use_merge);
    assert_eq!(original.create_constraints, parsed.create_constraints);
    assert_eq!(original.create_indexes, parsed.create_indexes);
    assert_eq!(original.id_property, parsed.id_property);
    assert_eq!(original.batch_size, parsed.batch_size);
}

#[test]
fn test_from_neo4j_config_serialization_roundtrip() {
    let original = FromNeo4jConfig::builder()
        .version(2, 0)
        .id_property("nodeId")
        .infer_nests(false)
        .build();

    let json = serde_json::to_string(&original).unwrap();
    let parsed: FromNeo4jConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(original.version, parsed.version);
    assert_eq!(original.id_property, parsed.id_property);
    assert_eq!(original.infer_nests, parsed.infer_nests);
}

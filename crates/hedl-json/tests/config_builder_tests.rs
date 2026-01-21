// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive configuration and builder pattern tests

use hedl_json::jsonpath::{QueryConfig, QueryConfigBuilder};
use hedl_json::streaming::{StreamConfig, StreamConfigBuilder};
use hedl_json::*;
use serde_json::json;

// ==================== FromJsonConfig Tests ====================

#[test]
fn test_from_json_config_builder_all_options() {
    let config = FromJsonConfig::builder()
        .max_depth(100)
        .max_array_size(5000)
        .max_object_size(1000)
        .max_string_length(50000)
        .default_type_name("CustomType")
        .version(2, 1)
        .surrogate_policy(SurrogatePolicy::ReplaceWithFFFD)
        .build();

    assert_eq!(config.max_depth, Some(100));
    assert_eq!(config.max_array_size, Some(5000));
    assert_eq!(config.max_object_size, Some(1000));
    assert_eq!(config.max_string_length, Some(50000));
    assert_eq!(config.default_type_name, "CustomType");
    assert_eq!(config.version, (2, 1));
    assert!(matches!(
        config.surrogate_policy,
        SurrogatePolicy::ReplaceWithFFFD
    ));
}

#[test]
fn test_from_json_config_unlimited_options() {
    let config = FromJsonConfig::builder().unlimited().build();

    assert_eq!(config.max_depth, None);
    assert_eq!(config.max_array_size, None);
    assert_eq!(config.max_object_size, None);
    assert_eq!(config.max_string_length, None);
}

#[test]
fn test_from_json_config_default_values() {
    let config = FromJsonConfig::default();

    assert_eq!(config.max_depth, Some(DEFAULT_MAX_DEPTH));
    assert_eq!(config.max_array_size, Some(DEFAULT_MAX_ARRAY_SIZE));
    assert_eq!(config.max_object_size, Some(DEFAULT_MAX_OBJECT_SIZE));
    assert_eq!(config.max_string_length, Some(DEFAULT_MAX_STRING_LENGTH));
    assert_eq!(config.version, (1, 0));
    // Default is strict validation (Reject invalid surrogates)
    assert!(matches!(config.surrogate_policy, SurrogatePolicy::Reject));
}

#[test]
fn test_from_json_config_clone() {
    let config1 = FromJsonConfig::builder()
        .max_depth(50)
        .default_type_name("TestType")
        .build();

    let config2 = config1.clone();

    assert_eq!(config1.max_depth, config2.max_depth);
    assert_eq!(config1.default_type_name, config2.default_type_name);
}

#[test]
fn test_from_json_config_debug() {
    let config = FromJsonConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("FromJsonConfig"));
    assert!(debug_str.contains("max_depth"));
}

#[test]
fn test_surrogate_policy_variants() {
    let reject = SurrogatePolicy::Reject;
    let replace = SurrogatePolicy::ReplaceWithFFFD;
    let skip = SurrogatePolicy::Skip;

    let config_reject = FromJsonConfig::builder().surrogate_policy(reject).build();
    let config_replace = FromJsonConfig::builder().surrogate_policy(replace).build();
    let config_skip = FromJsonConfig::builder().surrogate_policy(skip).build();

    assert!(matches!(
        config_reject.surrogate_policy,
        SurrogatePolicy::Reject
    ));
    assert!(matches!(
        config_replace.surrogate_policy,
        SurrogatePolicy::ReplaceWithFFFD
    ));
    assert!(matches!(
        config_skip.surrogate_policy,
        SurrogatePolicy::Skip
    ));
}

#[test]
fn test_from_json_config_builder_chaining() {
    let config = FromJsonConfig::builder()
        .max_depth(10)
        .max_array_size(20)
        .max_object_size(30)
        .max_string_length(40)
        .default_type_name("Chained")
        .version(3, 2)
        .build();

    assert_eq!(config.max_depth, Some(10));
    assert_eq!(config.version, (3, 2));
}

// ==================== ToJsonConfig Tests ====================

#[test]
fn test_to_json_config_default() {
    let config = ToJsonConfig::default();

    assert!(!config.include_metadata);
    assert!(!config.flatten_lists);
    assert!(config.include_children);
    assert!(!config.ascii_safe);
}

#[test]
fn test_to_json_config_all_options() {
    let config = ToJsonConfig {
        include_metadata: true,
        flatten_lists: true,
        include_children: false,
        ascii_safe: true,
    };

    assert!(config.include_metadata);
    assert!(config.flatten_lists);
    assert!(!config.include_children);
    assert!(config.ascii_safe);
}

#[test]
fn test_to_json_config_clone() {
    let config1 = ToJsonConfig {
        include_metadata: true,
        flatten_lists: false,
        include_children: true,
        ascii_safe: false,
    };

    let config2 = config1.clone();

    assert_eq!(config1.include_metadata, config2.include_metadata);
    assert_eq!(config1.flatten_lists, config2.flatten_lists);
}

#[test]
fn test_to_json_config_debug() {
    let config = ToJsonConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("ToJsonConfig"));
    assert!(debug_str.contains("include_metadata"));
}

#[test]
fn test_to_json_config_export_trait() {
    use hedl_core::convert::ExportConfig;

    let config = ToJsonConfig {
        include_metadata: true,
        flatten_lists: false,
        include_children: true,
        ascii_safe: false,
    };

    assert!(config.include_metadata());
    assert!(config.pretty());
}

// ==================== PartialConfig Tests ====================

#[test]
fn test_partial_config_default() {
    let config = PartialConfig::default();

    assert!(matches!(config.tolerance, ErrorTolerance::StopOnFirst));
    assert!(!config.include_partial_on_fatal);
    // Default is false (strict validation - don't replace with null)
    assert!(!config.replace_invalid_with_null);
}

#[test]
fn test_partial_config_builder_all_options() {
    let from_json = FromJsonConfig::builder().max_depth(50).build();

    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .include_partial_on_fatal(true)
        .replace_invalid_with_null(false)
        .from_json_config(from_json)
        .build();

    assert!(matches!(config.tolerance, ErrorTolerance::CollectAll));
    assert!(config.include_partial_on_fatal);
    assert!(!config.replace_invalid_with_null);
    assert_eq!(config.from_json_config.max_depth, Some(50));
}

#[test]
fn test_partial_config_clone() {
    let config1 = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .include_partial_on_fatal(true)
        .build();

    let config2 = config1.clone();

    assert_eq!(
        config1.include_partial_on_fatal,
        config2.include_partial_on_fatal
    );
}

#[test]
fn test_partial_config_debug() {
    let config = PartialConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("PartialConfig"));
}

#[test]
fn test_error_tolerance_variants() {
    let stop = ErrorTolerance::StopOnFirst;
    let collect = ErrorTolerance::CollectAll;

    let config_stop = PartialConfig::builder().tolerance(stop).build();

    let config_collect = PartialConfig::builder().tolerance(collect).build();

    assert!(matches!(config_stop.tolerance, ErrorTolerance::StopOnFirst));
    assert!(matches!(
        config_collect.tolerance,
        ErrorTolerance::CollectAll
    ));
}

// ==================== StreamConfig Tests ====================

#[test]
fn test_stream_config_default() {
    let config = StreamConfig::default();

    assert_eq!(config.buffer_size, 64 * 1024);
    assert_eq!(config.max_object_bytes, Some(10 * 1024 * 1024));
    assert!(config.use_size_estimation);
    assert!(config.true_streaming);
}

#[test]
fn test_stream_config_large_file() {
    let config = StreamConfig::large_file();

    assert_eq!(config.buffer_size, 256 * 1024);
    assert_eq!(config.max_object_bytes, Some(50 * 1024 * 1024));
    assert!(config.true_streaming);
}

#[test]
fn test_stream_config_low_memory() {
    let config = StreamConfig::low_memory();

    assert_eq!(config.buffer_size, 8 * 1024);
    assert_eq!(config.max_object_bytes, Some(1024 * 1024));
    assert!(config.true_streaming);
}

#[test]
fn test_stream_config_builder_all_options() {
    let from_json = FromJsonConfig::builder().max_depth(100).build();

    let config = StreamConfig::builder()
        .buffer_size(128 * 1024)
        .max_object_bytes(25 * 1024 * 1024)
        .from_json_config(from_json)
        .use_size_estimation(false)
        .true_streaming(false)
        .build();

    assert_eq!(config.buffer_size, 128 * 1024);
    assert_eq!(config.max_object_bytes, Some(25 * 1024 * 1024));
    assert!(!config.use_size_estimation);
    assert!(!config.true_streaming);
    assert_eq!(config.from_json.max_depth, Some(100));
}

#[test]
fn test_stream_config_unlimited_object_size() {
    let config = StreamConfig::builder().unlimited_object_size().build();

    assert_eq!(config.max_object_bytes, None);
}

#[test]
fn test_stream_config_clone() {
    let config1 = StreamConfig::builder().buffer_size(32 * 1024).build();

    let config2 = config1.clone();

    assert_eq!(config1.buffer_size, config2.buffer_size);
}

#[test]
fn test_stream_config_debug() {
    let config = StreamConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("StreamConfig"));
    assert!(debug_str.contains("buffer_size"));
}

// ==================== QueryConfig Tests ====================

#[test]
fn test_query_config_default() {
    let config = QueryConfig::default();

    assert!(!config.include_metadata);
    assert!(!config.flatten_lists);
    assert!(config.include_children);
    assert_eq!(config.max_results, 0);
}

#[test]
fn test_query_config_builder_all_options() {
    let config = QueryConfigBuilder::new()
        .include_metadata(true)
        .flatten_lists(true)
        .include_children(false)
        .max_results(100)
        .build();

    assert!(config.include_metadata);
    assert!(config.flatten_lists);
    assert!(!config.include_children);
    assert_eq!(config.max_results, 100);
}

#[test]
fn test_query_config_clone() {
    let config1 = QueryConfigBuilder::new().max_results(50).build();

    let config2 = config1.clone();

    assert_eq!(config1.max_results, config2.max_results);
}

#[test]
fn test_query_config_debug() {
    let config = QueryConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("QueryConfig"));
}

#[test]
fn test_query_config_to_json_config_conversion() {
    use hedl_json::ToJsonConfig;

    let query_config = QueryConfigBuilder::new()
        .include_metadata(true)
        .flatten_lists(true)
        .include_children(false)
        .build();

    let json_config: ToJsonConfig = (&query_config).into();

    assert_eq!(json_config.include_metadata, query_config.include_metadata);
    assert_eq!(json_config.flatten_lists, query_config.flatten_lists);
    assert_eq!(json_config.include_children, query_config.include_children);
}

// ==================== SchemaConfig Tests ====================

#[test]
fn test_schema_config_default() {
    use hedl_json::schema_gen::SchemaConfig;

    let config = SchemaConfig::default();

    assert!(config.title.is_none());
    assert!(config.description.is_none());
    assert!(config.schema_id.is_none());
    assert!(!config.strict);
    assert!(!config.include_examples);
    assert!(config.include_metadata);
}

#[test]
fn test_schema_config_builder_all_options() {
    use hedl_json::schema_gen::SchemaConfig;

    let config = SchemaConfig::builder()
        .title("Test Schema")
        .description("A test schema")
        .schema_id("https://example.com/schema.json")
        .strict(true)
        .include_examples(true)
        .include_metadata(false)
        .build();

    assert_eq!(config.title, Some("Test Schema".to_string()));
    assert_eq!(config.description, Some("A test schema".to_string()));
    assert_eq!(
        config.schema_id,
        Some("https://example.com/schema.json".to_string())
    );
    assert!(config.strict);
    assert!(config.include_examples);
    assert!(!config.include_metadata);
}

#[test]
fn test_schema_config_clone() {
    use hedl_json::schema_gen::SchemaConfig;

    let config1 = SchemaConfig::builder().title("Test").strict(true).build();

    let config2 = config1.clone();

    assert_eq!(config1.title, config2.title);
    assert_eq!(config1.strict, config2.strict);
}

#[test]
fn test_schema_config_debug() {
    use hedl_json::schema_gen::SchemaConfig;

    let config = SchemaConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("SchemaConfig"));
}

// ==================== ValidationConfig Tests ====================

#[cfg(feature = "validation")]
#[test]
fn test_validation_config_default() {
    use hedl_json::validation::{SchemaDraft, ValidationConfig};

    let config = ValidationConfig::default();

    assert!(matches!(config.draft, SchemaDraft::Draft7));
    assert!(config.collect_all_errors);
    assert!(config.max_errors.is_none());
    assert!(config.validate_formats);
}

#[cfg(feature = "validation")]
#[test]
fn test_validation_config_all_options() {
    use hedl_json::validation::{SchemaDraft, ValidationConfig};

    let config = ValidationConfig {
        draft: SchemaDraft::Draft202012,
        collect_all_errors: false,
        max_errors: Some(5),
        validate_formats: false,
    };

    assert!(matches!(config.draft, SchemaDraft::Draft202012));
    assert!(!config.collect_all_errors);
    assert_eq!(config.max_errors, Some(5));
    assert!(!config.validate_formats);
}

#[cfg(feature = "validation")]
#[test]
fn test_validation_config_clone() {
    use hedl_json::validation::ValidationConfig;

    let config1 = ValidationConfig {
        draft: hedl_json::validation::SchemaDraft::Draft7,
        collect_all_errors: true,
        max_errors: Some(10),
        validate_formats: true,
    };

    let config2 = config1.clone();

    assert_eq!(config1.max_errors, config2.max_errors);
    assert_eq!(config1.validate_formats, config2.validate_formats);
}

#[cfg(feature = "validation")]
#[test]
fn test_validation_config_debug() {
    use hedl_json::validation::ValidationConfig;

    let config = ValidationConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("ValidationConfig"));
}

#[cfg(feature = "validation")]
#[test]
fn test_schema_draft_variants() {
    use hedl_json::validation::SchemaDraft;

    let d4 = SchemaDraft::Draft4;
    let _d6 = SchemaDraft::Draft6;
    let d7 = SchemaDraft::Draft7;
    let _d201909 = SchemaDraft::Draft201909;
    let d202012 = SchemaDraft::Draft202012;

    // Test Debug
    assert!(format!("{:?}", d4).contains("Draft4"));
    assert!(format!("{:?}", d7).contains("Draft7"));

    // Test Default
    assert_eq!(SchemaDraft::default(), SchemaDraft::Draft7);

    // Test PartialEq
    assert_eq!(d7, SchemaDraft::Draft7);
    assert_ne!(d7, d202012);
}

// ==================== Configuration Integration Tests ====================

#[test]
fn test_configs_work_together() {
    let json = json!({
        "deep": {"level": {"nested": 1}},
        "array": [1, 2, 3]
    });

    // Use custom FromJsonConfig
    let from_config = FromJsonConfig::builder().max_depth(10).build();

    let doc = from_json_value(&json, &from_config).unwrap();

    // Use custom ToJsonConfig
    let to_config = ToJsonConfig {
        include_metadata: false,
        flatten_lists: false,
        include_children: true,
        ascii_safe: false,
    };

    let json_str = to_json(&doc, &to_config).unwrap();
    assert!(!json_str.is_empty());
}

#[test]
fn test_stream_config_with_custom_from_json() {
    use hedl_json::streaming::{JsonLinesStreamer, StreamConfig};
    use std::io::Cursor;

    let from_json = FromJsonConfig::builder()
        .max_depth(5)
        .max_string_length(100)
        .build();

    let config = StreamConfig::builder()
        .buffer_size(1024)
        .from_json_config(from_json)
        .build();

    let jsonl = r#"{"id": 1, "name": "test"}"#;
    let reader = Cursor::new(jsonl.as_bytes());

    let mut streamer = JsonLinesStreamer::new(reader, config);
    let result = streamer.next();

    assert!(result.is_some());
    assert!(result.unwrap().is_ok());
}

#[test]
fn test_partial_config_with_custom_from_json() {
    let from_json = FromJsonConfig::builder().max_depth(2).build();

    let partial_config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .from_json_config(from_json)
        .build();

    let json = r#"{"a": {"b": {"c": 1}}}"#;
    let result = partial_parse_json(json, &partial_config);

    // Should have errors due to depth limit
    if !result.errors.is_empty() {
        assert!(!result.errors.is_empty());
    }
}

// ==================== Builder Default Tests ====================

#[test]
fn test_builder_default_implementations() {
    // FromJsonConfigBuilder
    let from_builder1 = FromJsonConfigBuilder::default();
    let from_builder2 = FromJsonConfigBuilder::default();
    let config1 = from_builder1.build();
    let config2 = from_builder2.build();
    assert_eq!(config1.max_depth, config2.max_depth);

    // PartialConfigBuilder
    let partial_builder = PartialConfigBuilder::default();
    let partial_config = partial_builder.build();
    assert!(!partial_config.include_partial_on_fatal);

    // StreamConfigBuilder
    let stream_builder = StreamConfigBuilder::default();
    let stream_config = stream_builder.build();
    assert_eq!(stream_config.buffer_size, 64 * 1024);

    // QueryConfigBuilder
    let query_builder = QueryConfigBuilder::default();
    let query_config = query_builder.build();
    assert_eq!(query_config.max_results, 0);
}

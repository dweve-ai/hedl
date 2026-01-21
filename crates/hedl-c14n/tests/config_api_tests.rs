// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Tests for `CanonicalConfig` fluent API and builder pattern.
//!
//! Ensures both builder and fluent APIs produce identical configurations.

use hedl_c14n::{CanonicalConfig, CanonicalConfigBuilder, QuotingStrategy};

// =============================================================================
// Fluent API Tests
// =============================================================================

#[test]
fn test_fluent_api_with_quoting() {
    let config = CanonicalConfig::new().with_quoting(QuotingStrategy::Always);
    assert_eq!(config.quoting, QuotingStrategy::Always);
    assert!(config.use_ditto); // Defaults unchanged
    assert!(config.sort_keys);
    assert!(!config.inline_schemas);
}

#[test]
fn test_fluent_api_with_ditto() {
    let config = CanonicalConfig::new().with_ditto(false);
    assert_eq!(config.quoting, QuotingStrategy::Minimal); // Default
    assert!(!config.use_ditto);
    assert!(config.sort_keys);
    assert!(!config.inline_schemas);
}

#[test]
fn test_fluent_api_with_sort_keys() {
    let config = CanonicalConfig::new().with_sort_keys(false);
    assert_eq!(config.quoting, QuotingStrategy::Minimal);
    assert!(config.use_ditto);
    assert!(!config.sort_keys);
    assert!(!config.inline_schemas);
}

#[test]
fn test_fluent_api_with_inline_schemas() {
    let config = CanonicalConfig::new().with_inline_schemas(true);
    assert_eq!(config.quoting, QuotingStrategy::Minimal);
    assert!(config.use_ditto);
    assert!(config.sort_keys);
    assert!(config.inline_schemas);
}

#[test]
fn test_fluent_api_chaining_all_options() {
    let config = CanonicalConfig::new()
        .with_quoting(QuotingStrategy::Always)
        .with_ditto(false)
        .with_sort_keys(false)
        .with_inline_schemas(true);

    assert_eq!(config.quoting, QuotingStrategy::Always);
    assert!(!config.use_ditto);
    assert!(!config.sort_keys);
    assert!(config.inline_schemas);
}

#[test]
fn test_fluent_api_chaining_order_independence() {
    let config1 = CanonicalConfig::new()
        .with_ditto(false)
        .with_quoting(QuotingStrategy::Always);

    let config2 = CanonicalConfig::new()
        .with_quoting(QuotingStrategy::Always)
        .with_ditto(false);

    assert_eq!(config1.quoting, config2.quoting);
    assert_eq!(config1.use_ditto, config2.use_ditto);
    assert_eq!(config1.sort_keys, config2.sort_keys);
    assert_eq!(config1.inline_schemas, config2.inline_schemas);
}

#[test]
fn test_fluent_api_overwrite_same_field() {
    let config = CanonicalConfig::new()
        .with_ditto(true)
        .with_ditto(false)
        .with_ditto(true);

    assert!(config.use_ditto); // Last value wins
}

// =============================================================================
// Builder vs Fluent API Equivalence Tests
// =============================================================================

#[test]
fn test_builder_fluent_equivalence_minimal() {
    let builder_config = CanonicalConfigBuilder::new()
        .quoting(QuotingStrategy::Minimal)
        .build();

    let fluent_config = CanonicalConfig::new().with_quoting(QuotingStrategy::Minimal);

    assert_eq!(builder_config.quoting, fluent_config.quoting);
    assert_eq!(builder_config.use_ditto, fluent_config.use_ditto);
    assert_eq!(builder_config.sort_keys, fluent_config.sort_keys);
    assert_eq!(builder_config.inline_schemas, fluent_config.inline_schemas);
}

#[test]
fn test_builder_fluent_equivalence_always() {
    let builder_config = CanonicalConfigBuilder::new()
        .quoting(QuotingStrategy::Always)
        .build();

    let fluent_config = CanonicalConfig::new().with_quoting(QuotingStrategy::Always);

    assert_eq!(builder_config.quoting, fluent_config.quoting);
}

#[test]
fn test_builder_fluent_equivalence_all_custom() {
    let builder_config = CanonicalConfigBuilder::new()
        .quoting(QuotingStrategy::Always)
        .use_ditto(false)
        .sort_keys(false)
        .inline_schemas(true)
        .build();

    let fluent_config = CanonicalConfig::new()
        .with_quoting(QuotingStrategy::Always)
        .with_ditto(false)
        .with_sort_keys(false)
        .with_inline_schemas(true);

    assert_eq!(builder_config.quoting, fluent_config.quoting);
    assert_eq!(builder_config.use_ditto, fluent_config.use_ditto);
    assert_eq!(builder_config.sort_keys, fluent_config.sort_keys);
    assert_eq!(builder_config.inline_schemas, fluent_config.inline_schemas);
}

// =============================================================================
// PartialEq Tests for CanonicalConfig
// =============================================================================

#[test]
fn test_canonical_config_equality() {
    let config1 = CanonicalConfig::new();
    let config2 = CanonicalConfig::new();
    assert_eq!(config1, config2);
}

#[test]
fn test_canonical_config_inequality_quoting() {
    let config1 = CanonicalConfig::new().with_quoting(QuotingStrategy::Minimal);
    let config2 = CanonicalConfig::new().with_quoting(QuotingStrategy::Always);
    assert_ne!(config1, config2);
}

#[test]
fn test_canonical_config_inequality_ditto() {
    let config1 = CanonicalConfig::new().with_ditto(true);
    let config2 = CanonicalConfig::new().with_ditto(false);
    assert_ne!(config1, config2);
}

#[test]
fn test_canonical_config_inequality_sort_keys() {
    let config1 = CanonicalConfig::new().with_sort_keys(true);
    let config2 = CanonicalConfig::new().with_sort_keys(false);
    assert_ne!(config1, config2);
}

#[test]
fn test_canonical_config_inequality_inline_schemas() {
    let config1 = CanonicalConfig::new().with_inline_schemas(true);
    let config2 = CanonicalConfig::new().with_inline_schemas(false);
    assert_ne!(config1, config2);
}

// =============================================================================
// CanonicalConfig::new() Tests
// =============================================================================

#[test]
fn test_canonical_config_new_equals_default() {
    let new_config = CanonicalConfig::new();
    let default_config = CanonicalConfig::default();
    assert_eq!(new_config, default_config);
}

#[test]
fn test_canonical_config_new_has_correct_defaults() {
    let config = CanonicalConfig::new();
    assert_eq!(config.quoting, QuotingStrategy::Minimal);
    assert!(config.use_ditto);
    assert!(config.sort_keys);
    assert!(!config.inline_schemas);
}

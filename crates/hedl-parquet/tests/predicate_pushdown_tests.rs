// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Predicate pushdown and filtering tests for hedl-parquet
//!
//! Tests row group pruning, in-memory filtering, and predicate evaluation.

use hedl_parquet::predicate::{Predicate, PredicateValue};
use hedl_parquet::FromParquetConfig;

// =============================================================================
// Predicate Construction Tests
// =============================================================================

#[test]
fn test_predicate_equal() {
    let pred = Predicate::equal("age", PredicateValue::Int(25));
    assert_eq!(pred.to_string(), "age = 25");
}

#[test]
fn test_predicate_not_equal() {
    let pred = Predicate::not_equal("status", PredicateValue::String("inactive".into()));
    assert_eq!(pred.to_string(), "status != 'inactive'");
}

#[test]
fn test_predicate_less_than() {
    let pred = Predicate::less_than("score", PredicateValue::Float(50.0));
    assert_eq!(pred.to_string(), "score < 50");
}

#[test]
fn test_predicate_less_than_or_equal() {
    let pred = Predicate::less_than_or_equal("count", PredicateValue::Int(100));
    assert_eq!(pred.to_string(), "count <= 100");
}

#[test]
fn test_predicate_greater_than() {
    let pred = Predicate::greater_than("price", PredicateValue::Float(9.99));
    assert_eq!(pred.to_string(), "price > 9.99");
}

#[test]
fn test_predicate_greater_than_or_equal() {
    let pred = Predicate::greater_than_or_equal("age", PredicateValue::Int(18));
    assert_eq!(pred.to_string(), "age >= 18");
}

#[test]
fn test_predicate_between() {
    let pred = Predicate::between("age", PredicateValue::Int(18), PredicateValue::Int(65));
    assert_eq!(pred.to_string(), "age BETWEEN 18 AND 65");
}

#[test]
fn test_predicate_in_set() {
    let pred = Predicate::in_set(
        "status",
        vec![
            PredicateValue::String("active".into()),
            PredicateValue::String("pending".into()),
        ],
    );
    assert_eq!(pred.to_string(), "status IN ('active', 'pending')");
}

#[test]
fn test_predicate_not_in_set() {
    let pred = Predicate::not_in_set(
        "category",
        vec![
            PredicateValue::String("archived".into()),
            PredicateValue::String("deleted".into()),
        ],
    );
    assert_eq!(pred.to_string(), "category NOT IN ('archived', 'deleted')");
}

#[test]
fn test_predicate_is_null() {
    let pred = Predicate::is_null("optional_field");
    assert_eq!(pred.to_string(), "optional_field IS NULL");
}

#[test]
fn test_predicate_is_not_null() {
    let pred = Predicate::is_not_null("required_field");
    assert_eq!(pred.to_string(), "required_field IS NOT NULL");
}

#[test]
fn test_predicate_and() {
    let pred = Predicate::and(vec![
        Predicate::equal("status", PredicateValue::String("active".into())),
        Predicate::greater_than("age", PredicateValue::Int(18)),
    ]);
    assert_eq!(pred.to_string(), "(status = 'active') AND (age > 18)");
}

#[test]
fn test_predicate_or() {
    let pred = Predicate::or(vec![
        Predicate::equal("role", PredicateValue::String("admin".into())),
        Predicate::equal("role", PredicateValue::String("moderator".into())),
    ]);
    assert_eq!(pred.to_string(), "(role = 'admin') OR (role = 'moderator')");
}

#[test]
fn test_predicate_not() {
    let pred = Predicate::not(Predicate::is_null("email"));
    assert_eq!(pred.to_string(), "NOT (email IS NULL)");
}

#[test]
fn test_predicate_complex_nested() {
    let pred = Predicate::and(vec![
        Predicate::or(vec![
            Predicate::equal("type", PredicateValue::String("user".into())),
            Predicate::equal("type", PredicateValue::String("admin".into())),
        ]),
        Predicate::between("age", PredicateValue::Int(18), PredicateValue::Int(100)),
        Predicate::not(Predicate::is_null("email")),
    ]);
    // Should produce nested expression
    assert!(pred.to_string().contains("AND"));
    assert!(pred.to_string().contains("OR"));
    assert!(pred.to_string().contains("NOT"));
}

// =============================================================================
// PredicateValue Comparison Tests
// =============================================================================

#[test]
fn test_predicate_value_int_comparison() {
    let a = PredicateValue::Int(10);
    let b = PredicateValue::Int(20);

    assert!(a.lt(&b).unwrap());
    assert!(a.le(&b).unwrap());
    assert!(!a.gt(&b).unwrap());
    assert!(!a.ge(&b).unwrap());
}

#[test]
fn test_predicate_value_float_comparison() {
    let a = PredicateValue::Float(1.5);
    let b = PredicateValue::Float(2.5);

    assert!(a.lt(&b).unwrap());
    assert!(a.le(&b).unwrap());
    assert!(!a.gt(&b).unwrap());
    assert!(!a.ge(&b).unwrap());
}

#[test]
fn test_predicate_value_string_comparison() {
    let a = PredicateValue::String("apple".into());
    let b = PredicateValue::String("banana".into());

    assert!(a.lt(&b).unwrap());
    assert!(a.le(&b).unwrap());
    assert!(!a.gt(&b).unwrap());
    assert!(!a.ge(&b).unwrap());
}

#[test]
fn test_predicate_value_bool_comparison() {
    let f = PredicateValue::Bool(false);
    let t = PredicateValue::Bool(true);

    // false < true in Arrow/Rust
    assert!(f.lt(&t).unwrap());
    assert!(f.le(&t).unwrap());
    assert!(!f.gt(&t).unwrap());
    assert!(!f.ge(&t).unwrap());
}

#[test]
fn test_predicate_value_cross_type_int_float() {
    let int_val = PredicateValue::Int(5);
    let float_val = PredicateValue::Float(5.5);

    assert!(int_val.lt(&float_val).unwrap());
    assert!(int_val.le(&float_val).unwrap());
}

#[test]
fn test_predicate_value_incompatible_types() {
    let int_val = PredicateValue::Int(5);
    let str_val = PredicateValue::String("hello".into());

    // Incompatible types return None
    assert!(int_val.lt(&str_val).is_none());
    assert!(int_val.le(&str_val).is_none());
    assert!(int_val.gt(&str_val).is_none());
    assert!(int_val.ge(&str_val).is_none());
}

// =============================================================================
// Config with Predicate Tests
// =============================================================================

#[test]
fn test_config_with_filter() {
    let config = FromParquetConfig::default()
        .with_filter(Predicate::greater_than("age", PredicateValue::Int(18)));

    assert!(config.filter.is_some());
}

#[test]
fn test_config_with_predicate_constructor() {
    let config = FromParquetConfig::with_predicate(Predicate::equal(
        "status",
        PredicateValue::String("active".into()),
    ));

    assert!(config.filter.is_some());
}

#[test]
fn test_config_with_columns_and_filter() {
    let config = FromParquetConfig::with_columns(vec!["id".into(), "name".into()])
        .with_filter(Predicate::is_not_null("name"));

    assert!(config.columns.is_some());
    assert!(config.filter.is_some());
}

// =============================================================================
// Predicate Display Tests
// =============================================================================

#[test]
fn test_predicate_display_null() {
    assert_eq!(PredicateValue::Null.to_string(), "NULL");
}

#[test]
fn test_predicate_display_bool() {
    assert_eq!(PredicateValue::Bool(true).to_string(), "true");
    assert_eq!(PredicateValue::Bool(false).to_string(), "false");
}

#[test]
fn test_predicate_display_int() {
    assert_eq!(PredicateValue::Int(42).to_string(), "42");
    assert_eq!(PredicateValue::Int(-100).to_string(), "-100");
}

#[test]
fn test_predicate_display_float() {
    assert_eq!(PredicateValue::Float(2.71).to_string(), "2.71");
}

#[test]
fn test_predicate_display_string() {
    assert_eq!(
        PredicateValue::String("hello".into()).to_string(),
        "'hello'"
    );
}

// =============================================================================
// Empty Predicate Lists Tests
// =============================================================================

#[test]
fn test_empty_and_predicate() {
    let pred = Predicate::and(vec![]);
    // Empty AND should match everything
    assert_eq!(pred.to_string(), "");
}

#[test]
fn test_empty_or_predicate() {
    let pred = Predicate::or(vec![]);
    // Empty OR should match nothing
    assert_eq!(pred.to_string(), "");
}

#[test]
fn test_empty_in_predicate() {
    let pred = Predicate::in_set("status", vec![]);
    // Empty IN should match nothing
    assert_eq!(pred.to_string(), "status IN ()");
}

#[test]
fn test_empty_not_in_predicate() {
    let pred = Predicate::not_in_set("status", vec![]);
    // Empty NOT IN should match everything
    assert_eq!(pred.to_string(), "status NOT IN ()");
}

// =============================================================================
// Predicate with Special Values Tests
// =============================================================================

#[test]
fn test_predicate_with_zero() {
    let pred = Predicate::equal("count", PredicateValue::Int(0));
    assert_eq!(pred.to_string(), "count = 0");
}

#[test]
fn test_predicate_with_negative_number() {
    let pred = Predicate::less_than("temperature", PredicateValue::Int(-10));
    assert_eq!(pred.to_string(), "temperature < -10");
}

#[test]
fn test_predicate_with_empty_string() {
    let pred = Predicate::not_equal("name", PredicateValue::String(String::new()));
    assert_eq!(pred.to_string(), "name != ''");
}

#[test]
fn test_predicate_with_special_characters_in_string() {
    let pred = Predicate::equal("message", PredicateValue::String("Hello, 'world'!".into()));
    // String should be quoted
    assert!(pred.to_string().contains("'Hello, 'world'!'"));
}

// =============================================================================
// NOT Predicate Edge Cases
// =============================================================================

#[test]
fn test_not_is_null_simplification() {
    // NOT (IS NULL) should behave like IS NOT NULL
    let pred = Predicate::not(Predicate::is_null("email"));
    assert_eq!(pred.to_string(), "NOT (email IS NULL)");
}

#[test]
fn test_not_is_not_null_simplification() {
    // NOT (IS NOT NULL) should behave like IS NULL
    let pred = Predicate::not(Predicate::is_not_null("phone"));
    assert_eq!(pred.to_string(), "NOT (phone IS NOT NULL)");
}

#[test]
fn test_double_negation() {
    let pred = Predicate::not(Predicate::not(Predicate::equal(
        "status",
        PredicateValue::String("active".into()),
    )));
    // Should preserve double NOT
    assert!(pred.to_string().contains("NOT"));
}

// =============================================================================
// Predicate Value Equality Tests
// =============================================================================

#[test]
fn test_predicate_value_equality() {
    assert_eq!(PredicateValue::Int(42), PredicateValue::Int(42));
    assert_ne!(PredicateValue::Int(42), PredicateValue::Int(43));

    assert_eq!(PredicateValue::Bool(true), PredicateValue::Bool(true));
    assert_ne!(PredicateValue::Bool(true), PredicateValue::Bool(false));

    assert_eq!(
        PredicateValue::String("hello".into()),
        PredicateValue::String("hello".into())
    );
    assert_ne!(
        PredicateValue::String("hello".into()),
        PredicateValue::String("world".into())
    );
}

// =============================================================================
// Predicate Equality Tests
// =============================================================================

#[test]
fn test_predicate_equality() {
    let pred1 = Predicate::equal("age", PredicateValue::Int(25));
    let pred2 = Predicate::equal("age", PredicateValue::Int(25));
    let pred3 = Predicate::equal("age", PredicateValue::Int(30));

    assert_eq!(pred1, pred2);
    assert_ne!(pred1, pred3);
}

#[test]
fn test_predicate_clone() {
    let pred1 = Predicate::and(vec![
        Predicate::equal("status", PredicateValue::String("active".into())),
        Predicate::greater_than("age", PredicateValue::Int(18)),
    ]);

    let pred2 = pred1.clone();
    assert_eq!(pred1, pred2);
}

// =============================================================================
// Extreme Value Tests
// =============================================================================

#[test]
fn test_predicate_with_max_i64() {
    let pred = Predicate::less_than("big_number", PredicateValue::Int(i64::MAX));
    assert!(pred.to_string().contains(&i64::MAX.to_string()));
}

#[test]
fn test_predicate_with_min_i64() {
    let pred = Predicate::greater_than("small_number", PredicateValue::Int(i64::MIN));
    assert!(pred.to_string().contains(&i64::MIN.to_string()));
}

#[test]
fn test_predicate_with_max_f64() {
    let pred = Predicate::less_than("huge_float", PredicateValue::Float(f64::MAX));
    // Just verify it doesn't panic
    let _ = pred.to_string();
}

#[test]
fn test_predicate_with_infinity() {
    let pred = Predicate::less_than("value", PredicateValue::Float(f64::INFINITY));
    let s = pred.to_string();
    assert!(s.contains("inf") || s.contains("INF"));
}

#[test]
fn test_predicate_with_nan() {
    let pred = Predicate::equal("value", PredicateValue::Float(f64::NAN));
    let s = pred.to_string();
    assert!(s.contains("NaN") || s.contains("nan"));
}

// =============================================================================
// Long Predicate Lists Tests
// =============================================================================

#[test]
fn test_many_and_predicates() {
    let mut preds = Vec::new();
    for i in 0..100 {
        preds.push(Predicate::equal(format!("col{i}"), PredicateValue::Int(i)));
    }

    let pred = Predicate::and(preds);
    let s = pred.to_string();
    // Should contain many AND clauses
    assert!(s.matches(" AND ").count() >= 99);
}

#[test]
fn test_many_or_predicates() {
    let mut preds = Vec::new();
    for i in 0..50 {
        preds.push(Predicate::equal("status", PredicateValue::Int(i)));
    }

    let pred = Predicate::or(preds);
    let s = pred.to_string();
    // Should contain many OR clauses
    assert!(s.matches(" OR ").count() >= 49);
}

#[test]
fn test_large_in_set() {
    let mut values = Vec::new();
    for i in 0..1000 {
        values.push(PredicateValue::Int(i));
    }

    let pred = Predicate::in_set("id", values);
    let s = pred.to_string();
    // Should contain many comma-separated values
    assert!(s.matches(", ").count() >= 999);
}

// =============================================================================
// Unicode in Predicates Tests
// =============================================================================

#[test]
fn test_predicate_with_unicode_string() {
    let pred = Predicate::equal("name", PredicateValue::String("你好世界".into()));
    assert!(pred.to_string().contains("你好世界"));
}

#[test]
fn test_predicate_with_emoji() {
    let pred = Predicate::equal("message", PredicateValue::String("Hello 🌍!".into()));
    assert!(pred.to_string().contains("🌍"));
}

#[test]
fn test_predicate_with_rtl_text() {
    let pred = Predicate::equal("arabic", PredicateValue::String("مرحبا".into()));
    assert!(pred.to_string().contains("مرحبا"));
}

// =============================================================================
// Predicate Debug Output Tests
// =============================================================================

#[test]
fn test_predicate_debug_output() {
    let pred = Predicate::equal("age", PredicateValue::Int(25));
    let debug_str = format!("{pred:?}");
    assert!(debug_str.contains("Equal"));
}

#[test]
fn test_predicate_value_debug_output() {
    let val = PredicateValue::Int(42);
    let debug_str = format!("{val:?}");
    assert!(debug_str.contains("Int"));
    assert!(debug_str.contains("42"));
}

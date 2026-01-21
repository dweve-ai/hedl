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

//! Integer overflow and boundary tests for HEDL JSON conversion.
//!
//! Tests the handling of integer values at various boundaries to ensure:
//! - i64 boundary values are correctly converted
//! - Large integers that overflow i64 fall back to f64
//! - Precision loss in f64 conversion is handled correctly
//! - Negative integer boundaries are handled
//! - Various numeric representations work correctly

// Allow approximate float constants in tests - these are intentional test values
#![allow(clippy::approx_constant)]

use hedl_core::{Item, Value};
use hedl_json::{from_json, hedl_to_json, json_to_hedl, FromJsonConfig};

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn get_int_value(item: &Item) -> Option<i64> {
    if let Item::Scalar(Value::Int(i)) = item {
        Some(*i)
    } else {
        None
    }
}

fn get_float_value(item: &Item) -> Option<f64> {
    if let Item::Scalar(Value::Float(f)) = item {
        Some(*f)
    } else {
        None
    }
}

// =============================================================================
// i64 BOUNDARY TESTS
// =============================================================================

#[test]
fn test_i64_max() {
    let max = i64::MAX;
    let json = format!(r#"{{"value": {max}}}"#);
    let doc = json_to_hedl(&json).unwrap();
    assert_eq!(get_int_value(&doc.root["value"]), Some(max));
}

#[test]
fn test_i64_min() {
    let min = i64::MIN;
    let json = format!(r#"{{"value": {min}}}"#);
    let doc = json_to_hedl(&json).unwrap();
    assert_eq!(get_int_value(&doc.root["value"]), Some(min));
}

#[test]
fn test_i64_max_minus_one() {
    let val = i64::MAX - 1;
    let json = format!(r#"{{"value": {val}}}"#);
    let doc = json_to_hedl(&json).unwrap();
    assert_eq!(get_int_value(&doc.root["value"]), Some(val));
}

#[test]
fn test_i64_min_plus_one() {
    let val = i64::MIN + 1;
    let json = format!(r#"{{"value": {val}}}"#);
    let doc = json_to_hedl(&json).unwrap();
    assert_eq!(get_int_value(&doc.root["value"]), Some(val));
}

#[test]
fn test_i32_max_as_i64() {
    let val = i64::from(i32::MAX);
    let json = format!(r#"{{"value": {val}}}"#);
    let doc = json_to_hedl(&json).unwrap();
    assert_eq!(get_int_value(&doc.root["value"]), Some(val));
}

#[test]
fn test_i32_min_as_i64() {
    let val = i64::from(i32::MIN);
    let json = format!(r#"{{"value": {val}}}"#);
    let doc = json_to_hedl(&json).unwrap();
    assert_eq!(get_int_value(&doc.root["value"]), Some(val));
}

// =============================================================================
// UNSIGNED INTEGER BOUNDARY TESTS
// =============================================================================

#[test]
fn test_u32_max_as_i64() {
    let val = i64::from(u32::MAX);
    let json = format!(r#"{{"value": {val}}}"#);
    let doc = json_to_hedl(&json).unwrap();
    assert_eq!(get_int_value(&doc.root["value"]), Some(val));
}

#[test]
fn test_u64_max_overflow() {
    // u64::MAX (18446744073709551615) cannot fit in i64
    // It should return an error for safe integer handling
    let val = u64::MAX;
    let json = format!(r#"{{"value": {val}}}"#);
    let result = json_to_hedl(&json);

    // Should return an error since it overflows i64
    assert!(result.is_err(), "u64::MAX should cause overflow error");
    let err = result.unwrap_err();
    assert!(
        err.clone().contains("Integer overflow"),
        "Error should mention integer overflow"
    );
}

#[test]
fn test_i64_max_plus_one_overflow() {
    // i64::MAX + 1 = 9223372036854775808 (which is u64)
    // It should return an error for safe integer handling
    let val = (i64::MAX as u64) + 1;
    let json = format!(r#"{{"value": {val}}}"#);
    let result = json_to_hedl(&json);

    // Should return an error since it overflows i64
    assert!(result.is_err(), "i64::MAX + 1 should cause overflow error");
    let err = result.unwrap_err();
    assert!(
        err.clone().contains("Integer overflow"),
        "Error should mention integer overflow"
    );
}

// =============================================================================
// ZERO AND SMALL VALUES
// =============================================================================

#[test]
fn test_zero() {
    let json = r#"{"value": 0}"#;
    let doc = json_to_hedl(json).unwrap();
    assert_eq!(get_int_value(&doc.root["value"]), Some(0));
}

#[test]
fn test_negative_one() {
    let json = r#"{"value": -1}"#;
    let doc = json_to_hedl(json).unwrap();
    assert_eq!(get_int_value(&doc.root["value"]), Some(-1));
}

#[test]
fn test_positive_one() {
    let json = r#"{"value": 1}"#;
    let doc = json_to_hedl(json).unwrap();
    assert_eq!(get_int_value(&doc.root["value"]), Some(1));
}

// =============================================================================
// FLOAT REPRESENTATION TESTS
// =============================================================================

#[test]
fn test_float_zero() {
    let json = r#"{"value": 0.0}"#;
    let doc = json_to_hedl(json).unwrap();
    assert_eq!(get_float_value(&doc.root["value"]), Some(0.0));
}

#[test]
fn test_float_with_decimal() {
    let json = r#"{"value": 1.5}"#;
    let doc = json_to_hedl(json).unwrap();
    let f = get_float_value(&doc.root["value"]).unwrap();
    assert!((f - 1.5).abs() < 0.001);
}

#[test]
fn test_integer_written_as_float() {
    // 42.0 should be parsed as float even though it's a whole number
    let json = r#"{"value": 42.0}"#;
    let doc = json_to_hedl(json).unwrap();
    let f = get_float_value(&doc.root["value"]).unwrap();
    assert!((f - 42.0).abs() < 0.001);
}

#[test]
fn test_scientific_notation_positive() {
    let json = r#"{"value": 1e10}"#;
    let doc = json_to_hedl(json).unwrap();
    let f = get_float_value(&doc.root["value"]).unwrap();
    assert!((f - 1e10).abs() < 1.0);
}

#[test]
fn test_scientific_notation_negative_exponent() {
    let json = r#"{"value": 1e-10}"#;
    let doc = json_to_hedl(json).unwrap();
    let f = get_float_value(&doc.root["value"]).unwrap();
    assert!((f - 1e-10).abs() < 1e-15);
}

#[test]
fn test_scientific_notation_large() {
    let json = r#"{"value": 1.23e308}"#;
    let doc = json_to_hedl(json).unwrap();
    let f = get_float_value(&doc.root["value"]).unwrap();
    assert!(f.is_finite());
    assert!(f > 1e307);
}

#[test]
fn test_f64_max() {
    let json = format!(r#"{{"value": {}}}"#, f64::MAX);
    // This may fail to parse due to representation limits
    let result = json_to_hedl(&json);
    // Just verify it doesn't panic
    let _ = result;
}

// =============================================================================
// NEGATIVE FLOAT TESTS
// =============================================================================

#[test]
fn test_negative_float() {
    let json = r#"{"value": -3.14159}"#;
    let doc = json_to_hedl(json).unwrap();
    let f = get_float_value(&doc.root["value"]).unwrap();
    assert!((f - (-3.14159)).abs() < 0.00001);
}

#[test]
fn test_negative_scientific_notation() {
    let json = r#"{"value": -1.5e10}"#;
    let doc = json_to_hedl(json).unwrap();
    let f = get_float_value(&doc.root["value"]).unwrap();
    assert!((f - (-1.5e10)).abs() < 1.0);
}

// =============================================================================
// SPECIAL FLOAT VALUES
// =============================================================================

#[test]
fn test_very_small_float() {
    let json = r#"{"value": 1e-300}"#;
    let doc = json_to_hedl(json).unwrap();
    let f = get_float_value(&doc.root["value"]).unwrap();
    assert!(f > 0.0 && f < 1e-299);
}

#[test]
fn test_subnormal_float() {
    // Subnormal (denormalized) numbers are very small
    let json = r#"{"value": 1e-320}"#;
    let doc = json_to_hedl(json).unwrap();
    let f = get_float_value(&doc.root["value"]).unwrap();
    assert!(f >= 0.0); // May be zero due to underflow
}

// =============================================================================
// ARRAY OF INTEGERS TESTS
// =============================================================================

#[test]
fn test_array_of_integers() {
    let json = r#"{"values": [1, 2, 3, 4, 5]}"#;
    let doc = json_to_hedl(json).unwrap();

    if let Item::Scalar(Value::Tensor(t)) = &doc.root["values"] {
        let values = t.flatten();
        assert_eq!(values.len(), 5);
        assert_eq!(values, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_array_with_boundary_integers() {
    // Mix of boundary values
    let json = format!(
        r#"{{"values": [{}, {}, 0, -1, 1]}}"#,
        i64::MAX / 2,
        i64::MIN / 2
    );
    let doc = json_to_hedl(&json).unwrap();

    if let Item::Scalar(Value::Tensor(t)) = &doc.root["values"] {
        let values = t.flatten();
        assert_eq!(values.len(), 5);
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_2d_array_of_integers() {
    let json = r#"{"matrix": [[1, 2], [3, 4], [5, 6]]}"#;
    let doc = json_to_hedl(json).unwrap();

    if let Item::Scalar(Value::Tensor(t)) = &doc.root["matrix"] {
        let values = t.flatten();
        assert_eq!(values.len(), 6);
        assert_eq!(values, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    } else {
        panic!("Expected tensor");
    }
}

// =============================================================================
// PRECISION TESTS
// =============================================================================

#[test]
fn test_integer_precision_53_bits() {
    // JavaScript (and JSON) numbers have 53-bit precision
    // 2^53 = 9007199254740992 is the largest integer that can be represented exactly
    let safe_int = 9007199254740992i64; // 2^53
    let json = format!(r#"{{"value": {safe_int}}}"#);
    let doc = json_to_hedl(&json).unwrap();

    // Should be stored as int
    let val = get_int_value(&doc.root["value"]);
    assert!(val.is_some(), "Should be stored as integer");
    assert_eq!(val.unwrap(), safe_int);
}

#[test]
fn test_integer_beyond_53_bits() {
    // 2^53 + 1 starts losing precision in float representation
    let val = 9007199254740993i64; // 2^53 + 1
    let json = format!(r#"{{"value": {val}}}"#);
    let doc = json_to_hedl(&json).unwrap();

    // Should still be stored as int in i64
    let int_val = get_int_value(&doc.root["value"]);
    assert!(int_val.is_some());
    assert_eq!(int_val.unwrap(), val);
}

#[test]
fn test_large_integer_in_safe_range() {
    // Large but within i64 range
    let val = 1234567890123456789i64;
    let json = format!(r#"{{"value": {val}}}"#);
    let doc = json_to_hedl(&json).unwrap();

    let int_val = get_int_value(&doc.root["value"]);
    assert!(int_val.is_some());
    assert_eq!(int_val.unwrap(), val);
}

// =============================================================================
// ROUNDTRIP TESTS
// =============================================================================

#[test]
fn test_roundtrip_small_integer() {
    let json = r#"{"value": 42}"#;
    let doc = json_to_hedl(json).unwrap();
    let output = hedl_to_json(&doc).unwrap();

    // Parse output and verify
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["value"].as_i64(), Some(42));
}

#[test]
fn test_roundtrip_large_integer() {
    let val = i64::MAX / 2;
    let json = format!(r#"{{"value": {val}}}"#);
    let doc = json_to_hedl(&json).unwrap();
    let output = hedl_to_json(&doc).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["value"].as_i64(), Some(val));
}

#[test]
fn test_roundtrip_negative_integer() {
    let val = i64::MIN / 2;
    let json = format!(r#"{{"value": {val}}}"#);
    let doc = json_to_hedl(&json).unwrap();
    let output = hedl_to_json(&doc).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["value"].as_i64(), Some(val));
}

#[test]
fn test_roundtrip_float() {
    let json = r#"{"value": 3.14159}"#;
    let doc = json_to_hedl(json).unwrap();
    let output = hedl_to_json(&doc).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let f = parsed["value"].as_f64().unwrap();
    assert!((f - 3.14159).abs() < 0.00001);
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_multiple_integer_fields() {
    let json = r#"{"a": 1, "b": 2, "c": 3}"#;
    let doc = json_to_hedl(json).unwrap();

    assert_eq!(get_int_value(&doc.root["a"]), Some(1));
    assert_eq!(get_int_value(&doc.root["b"]), Some(2));
    assert_eq!(get_int_value(&doc.root["c"]), Some(3));
}

#[test]
fn test_mixed_int_and_float() {
    let json = r#"{"int": 42, "float": 3.14}"#;
    let doc = json_to_hedl(json).unwrap();

    assert_eq!(get_int_value(&doc.root["int"]), Some(42));
    let f = get_float_value(&doc.root["float"]).unwrap();
    assert!((f - 3.14).abs() < 0.001);
}

#[test]
fn test_nested_integers() {
    let json = r#"{"outer": {"inner": 999}}"#;
    let doc = json_to_hedl(json).unwrap();

    if let Item::Object(obj) = &doc.root["outer"] {
        assert_eq!(get_int_value(&obj["inner"]), Some(999));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_integer_in_matrix_list() {
    let json = r#"{"users": [{"id": "1", "age": 30}, {"id": "2", "age": 25}]}"#;
    let doc = json_to_hedl(json).unwrap();

    if let Item::List(list) = &doc.root["users"] {
        assert_eq!(list.rows.len(), 2);
        // Age should be in the fields
        // Schema is typically ["id", "age"]
        let age_idx = list.schema.iter().position(|s| s == "age").unwrap();
        assert!(matches!(list.rows[0].fields[age_idx], Value::Int(30)));
        assert!(matches!(list.rows[1].fields[age_idx], Value::Int(25)));
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// ERROR HANDLING TESTS
// =============================================================================

#[test]
fn test_infinity_handling() {
    // JSON doesn't support infinity, but this tests what happens with very large values
    let json = r#"{"value": 1e999}"#;
    let result = json_to_hedl(json);
    // Should either fail to parse or result in infinity
    if let Ok(doc) = result {
        if let Some(f) = get_float_value(&doc.root["value"]) {
            assert!(f.is_infinite() || f.is_nan());
        }
    }
    // If it fails, that's also acceptable behavior
}

#[test]
fn test_negative_infinity_handling() {
    let json = r#"{"value": -1e999}"#;
    let result = json_to_hedl(json);
    if let Ok(doc) = result {
        if let Some(f) = get_float_value(&doc.root["value"]) {
            assert!(f.is_infinite() || f.is_nan());
        }
    }
}

// =============================================================================
// CONFIGURATION TESTS
// =============================================================================

#[test]
fn test_with_custom_config() {
    let json = r#"{"value": 42}"#;
    let config = FromJsonConfig::builder()
        .max_depth(100)
        .max_array_size(1000)
        .build();

    let doc = from_json(json, &config).unwrap();
    assert_eq!(get_int_value(&doc.root["value"]), Some(42));
}

#[test]
fn test_with_unlimited_config() {
    let json = r#"{"value": 42}"#;
    let config = FromJsonConfig::builder().unlimited().build();

    let doc = from_json(json, &config).unwrap();
    assert_eq!(get_int_value(&doc.root["value"]), Some(42));
}

// =============================================================================
// STRESS TESTS
// =============================================================================

#[test]
fn test_many_integers() {
    // Create JSON with many integer fields
    let mut fields = Vec::new();
    for i in 0..100 {
        fields.push(format!(r#""field{}": {}"#, i, i * 1000));
    }
    let json = format!("{{{}}}", fields.join(", "));

    let doc = json_to_hedl(&json).unwrap();

    for i in 0..100 {
        let key = format!("field{i}");
        assert_eq!(
            get_int_value(&doc.root[&key]),
            Some(i * 1000),
            "field{i} mismatch"
        );
    }
}

#[test]
fn test_deeply_nested_integers() {
    // Nested object with integers at various levels
    let json = r#"{"l1": {"l2": {"l3": {"value": 12345}}}}"#;
    let doc = json_to_hedl(json).unwrap();

    if let Item::Object(l1) = &doc.root["l1"] {
        if let Item::Object(l2) = &l1["l2"] {
            if let Item::Object(l3) = &l2["l3"] {
                assert_eq!(get_int_value(&l3["value"]), Some(12345));
            } else {
                panic!("l3 not object");
            }
        } else {
            panic!("l2 not object");
        }
    } else {
        panic!("l1 not object");
    }
}

// =============================================================================
// TENSOR PRECISION TESTS
// =============================================================================

#[test]
fn test_tensor_with_large_integers() {
    let json = format!(
        r#"{{"values": [{}, {}, {}]}}"#,
        i64::MAX / 4,
        0,
        i64::MIN / 4
    );
    let doc = json_to_hedl(&json).unwrap();

    if let Item::Scalar(Value::Tensor(t)) = &doc.root["values"] {
        let values = t.flatten();
        assert_eq!(values.len(), 3);
        // Note: Converting large i64 to f64 may lose precision
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_tensor_with_mixed_precision() {
    let json = r#"{"values": [1.0, 2, 3.5, 4, 5.999]}"#;
    let doc = json_to_hedl(json).unwrap();

    if let Item::Scalar(Value::Tensor(t)) = &doc.root["values"] {
        let values = t.flatten();
        assert_eq!(values.len(), 5);
        assert!((values[0] - 1.0).abs() < 0.001);
        assert!((values[1] - 2.0).abs() < 0.001);
        assert!((values[2] - 3.5).abs() < 0.001);
        assert!((values[3] - 4.0).abs() < 0.001);
        assert!((values[4] - 5.999).abs() < 0.001);
    } else {
        panic!("Expected tensor");
    }
}

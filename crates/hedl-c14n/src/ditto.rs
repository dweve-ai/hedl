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

//! Ditto operator optimization.
//!
//! The ditto operator (`^`) is a space-saving optimization for matrix lists
//! where consecutive rows often have repeated values. Instead of writing the
//! full value again, we use `^` to indicate "same as above".
//!
//! # Rules
//!
//! - Never used in first row (no previous row to reference)
//! - Never used in first column (ID column must always be explicit)
//! - Requires exact type and value equality (Int(42) ≠ Float(42.0))
//! - NaN equals NaN for ditto purposes (special case)
//!
//! # Examples
//!
//! ```text
//! Without ditto:
//! |alice,engineer,NYC,full-time
//! |bob,engineer,NYC,full-time
//! |carol,designer,SF,part-time
//!
//! With ditto:
//! |alice,engineer,NYC,full-time
//! |bob,^,^,^
//! |carol,designer,SF,part-time
//! ```

use hedl_core::Value;

/// Check if a value can use ditto marker from previous row.
///
/// Returns `true` if `current` equals `previous` with exact type matching.
/// This enables the ditto optimization where `^` replaces repeated values.
///
/// # Arguments
///
/// * `current` - The current row's value for a column
/// * `previous` - The previous row's value for the same column
///
/// # Returns
///
/// `true` if values are identical (type and value), `false` otherwise.
///
/// # Type Safety
///
/// Different types never match, even if semantically equivalent:
/// - `Int(42)` ≠ `Float(42.0)`
/// - `Bool(true)` ≠ `Int(1)`
/// - `String("42")` ≠ `Int(42)`
///
/// # Special Cases
///
/// - `NaN == NaN` returns `true` (differs from standard IEEE 754)
/// - `-0.0 == 0.0` returns `true` (follows IEEE 754)
/// - Empty strings match empty strings
/// - Null matches null
///
/// # Examples
///
/// ```
/// use hedl_c14n::can_use_ditto;
/// use hedl_core::Value;
///
/// // Same values and types
/// assert!(can_use_ditto(&Value::Int(42), &Value::Int(42)));
/// assert!(can_use_ditto(
///     &Value::String("hello".to_string()),
///     &Value::String("hello".to_string())
/// ));
///
/// // Different values
/// assert!(!can_use_ditto(&Value::Int(42), &Value::Int(43)));
///
/// // Different types (even if semantically similar)
/// assert!(!can_use_ditto(&Value::Int(42), &Value::Float(42.0)));
/// ```
pub fn can_use_ditto(current: &Value, previous: &Value) -> bool {
    // Ditto requires deep equality including type
    values_equal(current, previous)
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => {
            // Handle NaN and exact float comparison
            (x.is_nan() && y.is_nan()) || x == y
        }
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Tensor(x), Value::Tensor(y)) => x == y,
        (Value::Reference(x), Value::Reference(y)) => x == y,
        (Value::Expression(x), Value::Expression(y)) => x == y,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{Expression, Reference, Tensor};

    // ==================== Null tests ====================

    #[test]
    fn test_ditto_null_same() {
        assert!(can_use_ditto(&Value::Null, &Value::Null));
    }

    #[test]
    fn test_ditto_null_vs_other() {
        assert!(!can_use_ditto(&Value::Null, &Value::Bool(false)));
        assert!(!can_use_ditto(&Value::Null, &Value::Int(0)));
        assert!(!can_use_ditto(&Value::Null, &Value::String("".to_string())));
    }

    // ==================== Bool tests ====================

    #[test]
    fn test_ditto_bool_true_same() {
        assert!(can_use_ditto(&Value::Bool(true), &Value::Bool(true)));
    }

    #[test]
    fn test_ditto_bool_false_same() {
        assert!(can_use_ditto(&Value::Bool(false), &Value::Bool(false)));
    }

    #[test]
    fn test_ditto_bool_different() {
        assert!(!can_use_ditto(&Value::Bool(true), &Value::Bool(false)));
        assert!(!can_use_ditto(&Value::Bool(false), &Value::Bool(true)));
    }

    #[test]
    fn test_ditto_bool_vs_other() {
        assert!(!can_use_ditto(&Value::Bool(true), &Value::Int(1)));
        assert!(!can_use_ditto(&Value::Bool(false), &Value::Int(0)));
    }

    // ==================== Int tests ====================

    #[test]
    fn test_ditto_int_same() {
        assert!(can_use_ditto(&Value::Int(42), &Value::Int(42)));
    }

    #[test]
    fn test_ditto_int_different() {
        assert!(!can_use_ditto(&Value::Int(42), &Value::Int(43)));
    }

    #[test]
    fn test_ditto_int_zero() {
        assert!(can_use_ditto(&Value::Int(0), &Value::Int(0)));
    }

    #[test]
    fn test_ditto_int_negative() {
        assert!(can_use_ditto(&Value::Int(-100), &Value::Int(-100)));
        assert!(!can_use_ditto(&Value::Int(-100), &Value::Int(100)));
    }

    #[test]
    fn test_ditto_int_large() {
        assert!(can_use_ditto(&Value::Int(i64::MAX), &Value::Int(i64::MAX)));
        assert!(can_use_ditto(&Value::Int(i64::MIN), &Value::Int(i64::MIN)));
    }

    #[test]
    fn test_ditto_int_vs_float() {
        // Int 42 is NOT equal to Float 42.0 for ditto purposes
        assert!(!can_use_ditto(&Value::Int(42), &Value::Float(42.0)));
    }

    // ==================== Float tests ====================

    #[test]
    fn test_ditto_float_same() {
        assert!(can_use_ditto(&Value::Float(3.125), &Value::Float(3.125)));
    }

    #[test]
    fn test_ditto_float_different() {
        assert!(!can_use_ditto(&Value::Float(3.125), &Value::Float(2.75)));
    }

    #[test]
    fn test_ditto_float_zero() {
        assert!(can_use_ditto(&Value::Float(0.0), &Value::Float(0.0)));
    }

    #[test]
    fn test_ditto_float_negative_zero() {
        // -0.0 == 0.0 in IEEE 754
        assert!(can_use_ditto(&Value::Float(-0.0), &Value::Float(0.0)));
    }

    #[test]
    fn test_ditto_float_infinity() {
        assert!(can_use_ditto(
            &Value::Float(f64::INFINITY),
            &Value::Float(f64::INFINITY)
        ));
        assert!(can_use_ditto(
            &Value::Float(f64::NEG_INFINITY),
            &Value::Float(f64::NEG_INFINITY)
        ));
        assert!(!can_use_ditto(
            &Value::Float(f64::INFINITY),
            &Value::Float(f64::NEG_INFINITY)
        ));
    }

    #[test]
    fn test_ditto_float_nan() {
        // NaN should equal NaN for ditto purposes
        assert!(can_use_ditto(
            &Value::Float(f64::NAN),
            &Value::Float(f64::NAN)
        ));
    }

    #[test]
    fn test_ditto_float_nan_vs_number() {
        assert!(!can_use_ditto(&Value::Float(f64::NAN), &Value::Float(0.0)));
        assert!(!can_use_ditto(&Value::Float(f64::NAN), &Value::Float(1.0)));
    }

    #[test]
    fn test_ditto_float_very_small() {
        let small = 1e-308;
        assert!(can_use_ditto(&Value::Float(small), &Value::Float(small)));
    }

    #[test]
    fn test_ditto_float_very_large() {
        let large = 1e308;
        assert!(can_use_ditto(&Value::Float(large), &Value::Float(large)));
    }

    // ==================== String tests ====================

    #[test]
    fn test_ditto_string_same() {
        assert!(can_use_ditto(
            &Value::String("hello".to_string()),
            &Value::String("hello".to_string())
        ));
    }

    #[test]
    fn test_ditto_string_different() {
        assert!(!can_use_ditto(
            &Value::String("hello".to_string()),
            &Value::String("world".to_string())
        ));
    }

    #[test]
    fn test_ditto_string_empty() {
        assert!(can_use_ditto(
            &Value::String("".to_string()),
            &Value::String("".to_string())
        ));
    }

    #[test]
    fn test_ditto_string_case_sensitive() {
        assert!(!can_use_ditto(
            &Value::String("Hello".to_string()),
            &Value::String("hello".to_string())
        ));
    }

    #[test]
    fn test_ditto_string_whitespace() {
        assert!(!can_use_ditto(
            &Value::String(" hello".to_string()),
            &Value::String("hello".to_string())
        ));
        assert!(!can_use_ditto(
            &Value::String("hello ".to_string()),
            &Value::String("hello".to_string())
        ));
    }

    #[test]
    fn test_ditto_string_unicode() {
        assert!(can_use_ditto(
            &Value::String("héllo 世界".to_string()),
            &Value::String("héllo 世界".to_string())
        ));
    }

    #[test]
    fn test_ditto_string_with_special_chars() {
        assert!(can_use_ditto(
            &Value::String("line1\nline2".to_string()),
            &Value::String("line1\nline2".to_string())
        ));
    }

    // ==================== Tensor tests ====================

    #[test]
    fn test_ditto_tensor_scalar_same() {
        let t1 = Value::Tensor(Tensor::Scalar(1.0));
        let t2 = Value::Tensor(Tensor::Scalar(1.0));
        assert!(can_use_ditto(&t1, &t2));
    }

    #[test]
    fn test_ditto_tensor_scalar_different() {
        let t1 = Value::Tensor(Tensor::Scalar(1.0));
        let t2 = Value::Tensor(Tensor::Scalar(2.0));
        assert!(!can_use_ditto(&t1, &t2));
    }

    #[test]
    fn test_ditto_tensor_array_same() {
        let t1 = Value::Tensor(Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
        ]));
        let t2 = Value::Tensor(Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
        ]));
        assert!(can_use_ditto(&t1, &t2));
    }

    #[test]
    fn test_ditto_tensor_array_different_values() {
        let t1 = Value::Tensor(Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
        ]));
        let t2 = Value::Tensor(Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(3.0),
        ]));
        assert!(!can_use_ditto(&t1, &t2));
    }

    #[test]
    fn test_ditto_tensor_array_different_length() {
        let t1 = Value::Tensor(Tensor::Array(vec![Tensor::Scalar(1.0)]));
        let t2 = Value::Tensor(Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
        ]));
        assert!(!can_use_ditto(&t1, &t2));
    }

    // ==================== Reference tests ====================

    #[test]
    fn test_ditto_reference_same() {
        let r1 = Value::Reference(Reference::qualified("User", "name"));
        let r2 = Value::Reference(Reference::qualified("User", "name"));
        assert!(can_use_ditto(&r1, &r2));
    }

    #[test]
    fn test_ditto_reference_different_type() {
        let r1 = Value::Reference(Reference::qualified("User", "name"));
        let r2 = Value::Reference(Reference::qualified("Post", "name"));
        assert!(!can_use_ditto(&r1, &r2));
    }

    #[test]
    fn test_ditto_reference_different_field() {
        let r1 = Value::Reference(Reference::qualified("User", "name"));
        let r2 = Value::Reference(Reference::qualified("User", "email"));
        assert!(!can_use_ditto(&r1, &r2));
    }

    // ==================== Expression tests ====================

    #[test]
    fn test_ditto_expression_same() {
        let e1 = Value::Expression(Expression::Identifier {
            name: "foo".to_string(),
            span: Default::default(),
        });
        let e2 = Value::Expression(Expression::Identifier {
            name: "foo".to_string(),
            span: Default::default(),
        });
        assert!(can_use_ditto(&e1, &e2));
    }

    #[test]
    fn test_ditto_expression_different() {
        let e1 = Value::Expression(Expression::Identifier {
            name: "foo".to_string(),
            span: Default::default(),
        });
        let e2 = Value::Expression(Expression::Identifier {
            name: "bar".to_string(),
            span: Default::default(),
        });
        assert!(!can_use_ditto(&e1, &e2));
    }

    #[test]
    fn test_ditto_expression_different_variant() {
        let e1 = Value::Expression(Expression::Identifier {
            name: "foo".to_string(),
            span: Default::default(),
        });
        let e2 = Value::Expression(Expression::Call {
            name: "foo".to_string(),
            args: vec![],
            span: Default::default(),
        });
        assert!(!can_use_ditto(&e1, &e2));
    }

    // ==================== Cross-type tests ====================

    #[test]
    fn test_ditto_different_types() {
        // None of these should be equal despite potentially similar semantic meaning
        assert!(!can_use_ditto(&Value::Null, &Value::Bool(false)));
        assert!(!can_use_ditto(&Value::Int(0), &Value::Bool(false)));
        assert!(!can_use_ditto(&Value::Int(1), &Value::Bool(true)));
        assert!(!can_use_ditto(&Value::Int(42), &Value::Float(42.0)));
        assert!(!can_use_ditto(
            &Value::String("42".to_string()),
            &Value::Int(42)
        ));
        assert!(!can_use_ditto(
            &Value::String("true".to_string()),
            &Value::Bool(true)
        ));
    }

    #[test]
    fn test_ditto_all_types_self_equal() {
        // Every type should be equal to itself
        let values = vec![
            Value::Null,
            Value::Bool(true),
            Value::Bool(false),
            Value::Int(42),
            Value::Float(3.125),
            Value::String("hello".to_string()),
            Value::Tensor(Tensor::Scalar(1.0)),
            Value::Reference(Reference::qualified("Type", "field")),
            Value::Expression(Expression::Identifier {
                name: "expr".to_string(),
                span: Default::default(),
            }),
        ];

        for v in &values {
            assert!(can_use_ditto(v, v), "Value {:?} should equal itself", v);
        }
    }
}

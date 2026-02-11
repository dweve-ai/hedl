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

//! Type system for bidirectional type checking in HEDL.
//!
//! This module provides the infrastructure for expected type representation,
//! type matching, and type coercion used in the bidirectional type inference system.

use crate::value::Value;

/// Tensor data type hint for type expectations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TensorDtype {
    /// Integer tensor
    Int,
    /// Float tensor
    Float,
}

/// Expected type for a value during inference.
///
/// This enum represents type expectations that can be propagated from context
/// (such as schema definitions) to guide type inference and validation.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpectedType {
    /// Any type is acceptable (default, preserves current behavior)
    Any,
    /// Null is expected
    Null,
    /// Boolean expected
    Bool,
    /// Integer expected (may accept numeric strings in lenient mode)
    Int,
    /// Float expected (may accept int or numeric strings)
    Float,
    /// Any numeric type (int or float)
    Numeric,
    /// String expected
    String,
    /// Tensor with optional shape/dtype constraints
    Tensor {
        /// Expected tensor shape (if known)
        shape: Option<Vec<usize>>,
        /// Expected tensor data type (if known)
        dtype: Option<TensorDtype>,
    },
    /// Reference to specific type or any type
    Reference {
        /// Expected target type name (if known)
        target_type: Option<String>,
    },
    /// Expression expected
    Expression,
    /// One of several acceptable types
    Union(Vec<ExpectedType>),
    /// List with element type constraint
    ///
    /// Matches `Value::List` where all elements match the inner type.
    List(Box<ExpectedType>),
}

impl ExpectedType {
    /// Check if a Value matches this expected type.
    ///
    /// This performs strict type matching without coercion.
    pub fn matches(&self, value: &Value) -> bool {
        match self {
            ExpectedType::Any => true,
            ExpectedType::Null => value.is_null(),
            ExpectedType::Bool => matches!(value, Value::Bool(_)),
            ExpectedType::Int => matches!(value, Value::Int(_)),
            ExpectedType::Float => matches!(value, Value::Float(_)),
            ExpectedType::Numeric => matches!(value, Value::Int(_) | Value::Float(_)),
            ExpectedType::String => matches!(value, Value::String(_)),
            ExpectedType::Tensor { shape, dtype } => {
                if let Value::Tensor(tensor) = value {
                    // Check shape constraint if specified
                    if let Some(expected_shape) = shape {
                        let actual_shape = tensor.shape();
                        if &actual_shape != expected_shape {
                            return false;
                        }
                    }
                    // Check dtype constraint if specified
                    if let Some(expected_dtype) = dtype {
                        let is_int_tensor = tensor.is_integer();
                        match expected_dtype {
                            TensorDtype::Float => {
                                if is_int_tensor {
                                    return false;
                                }
                            }
                            TensorDtype::Int => {
                                if !is_int_tensor {
                                    return false;
                                }
                            }
                        }
                    }
                    true
                } else {
                    false
                }
            }
            ExpectedType::Reference { target_type } => {
                if let Value::Reference(r) = value {
                    if let Some(expected_type) = target_type {
                        // Check if reference type matches
                        match &r.type_name {
                            Some(actual_type) => &**actual_type == expected_type,
                            None => false, // Unqualified reference doesn't match specific type
                        }
                    } else {
                        // Any reference is acceptable
                        true
                    }
                } else {
                    false
                }
            }
            ExpectedType::Expression => matches!(value, Value::Expression(_)),
            ExpectedType::Union(types) => types.iter().any(|t| t.matches(value)),
            ExpectedType::List(element_type) => {
                if let Value::List(items) = value {
                    // All elements must match the element type
                    items.iter().all(|item| element_type.matches(item))
                } else {
                    false
                }
            }
        }
    }

    /// Get a human-readable description of this type.
    ///
    /// Used for error messages and documentation.
    pub fn describe(&self) -> String {
        match self {
            ExpectedType::Any => "Any".to_string(),
            ExpectedType::Null => "Null".to_string(),
            ExpectedType::Bool => "Bool".to_string(),
            ExpectedType::Int => "Int".to_string(),
            ExpectedType::Float => "Float".to_string(),
            ExpectedType::Numeric => "Numeric (Int or Float)".to_string(),
            ExpectedType::String => "String".to_string(),
            ExpectedType::Tensor { shape, dtype } => {
                let mut desc = "Tensor".to_string();
                if let Some(s) = shape {
                    desc.push_str(&format!(" (shape: {:?})", s));
                }
                if let Some(dt) = dtype {
                    desc.push_str(&format!(" (dtype: {:?})", dt));
                }
                desc
            }
            ExpectedType::Reference { target_type } => {
                if let Some(t) = target_type {
                    format!("Reference({})", t)
                } else {
                    "Reference".to_string()
                }
            }
            ExpectedType::Expression => "Expression".to_string(),
            ExpectedType::Union(types) => {
                let type_names: Vec<String> = types.iter().map(|t| t.describe()).collect();
                format!("Union({})", type_names.join(" | "))
            }
            ExpectedType::List(element_type) => {
                format!("List({})", element_type.describe())
            }
        }
    }

    /// Check if value can be coerced to this type.
    ///
    /// This checks whether safe type coercion is possible in the given mode.
    ///
    /// # Parameters
    ///
    /// - `value`: The value to check for coercion
    /// - `strict`: If true, only allow very safe coercions (Int→Float).
    ///   If false, allow parsing from strings.
    pub fn can_coerce(&self, value: &Value, strict: bool) -> bool {
        // If value already matches, no coercion needed
        if self.matches(value) {
            return true;
        }

        match self {
            ExpectedType::Any => true,

            ExpectedType::Float => {
                match value {
                    // Int → Float is always safe
                    Value::Int(_) => true,
                    // String → Float only in lenient mode
                    Value::String(s) if !strict => s.trim().parse::<f64>().is_ok(),
                    _ => false,
                }
            }

            ExpectedType::Int => {
                match value {
                    // String → Int only in lenient mode
                    Value::String(s) if !strict => s.trim().parse::<i64>().is_ok(),
                    _ => false,
                }
            }

            ExpectedType::Bool => {
                match value {
                    // String → Bool only in lenient mode for "true"/"false"
                    Value::String(s) if !strict => {
                        let trimmed = s.trim();
                        trimmed == "true" || trimmed == "false"
                    }
                    _ => false,
                }
            }

            ExpectedType::String => {
                // In lenient mode, most values can be converted to string
                !strict
            }

            ExpectedType::Numeric => {
                match value {
                    // Int is already numeric
                    Value::Int(_) => true,
                    // Float is already numeric
                    Value::Float(_) => true,
                    // String → Numeric in lenient mode
                    Value::String(s) if !strict => {
                        let trimmed = s.trim();
                        trimmed.parse::<i64>().is_ok() || trimmed.parse::<f64>().is_ok()
                    }
                    _ => false,
                }
            }

            ExpectedType::Union(types) => {
                // Can coerce if any union member accepts the value
                types.iter().any(|t| t.can_coerce(value, strict))
            }

            ExpectedType::List(element_type) => {
                // Check if a list can be coerced by checking all elements
                if let Value::List(items) = value {
                    items
                        .iter()
                        .all(|item| element_type.can_coerce(item, strict))
                } else {
                    false
                }
            }

            // Other types don't support coercion
            _ => false,
        }
    }
}

/// Get the inferred ExpectedType from a Value.
///
/// This is useful for converting runtime values back to type expectations.
pub fn value_to_expected_type(value: &Value) -> ExpectedType {
    match value {
        Value::Null => ExpectedType::Null,
        Value::Bool(_) => ExpectedType::Bool,
        Value::Int(_) => ExpectedType::Int,
        Value::Float(_) => ExpectedType::Float,
        Value::String(_) => ExpectedType::String,
        Value::Tensor(t) => {
            let shape = t.shape();
            let dtype = if t.is_integer() {
                Some(TensorDtype::Int)
            } else {
                Some(TensorDtype::Float)
            };
            ExpectedType::Tensor {
                shape: Some(shape),
                dtype,
            }
        }
        Value::Reference(r) => ExpectedType::Reference {
            target_type: r.type_name.as_ref().map(|s| s.to_string()),
        },
        Value::Expression(_) => ExpectedType::Expression,
        // Lists infer their element type from the first non-null element
        Value::List(items) => {
            let element_type = items
                .iter()
                .find(|v| !matches!(v, Value::Null))
                .map(value_to_expected_type)
                .unwrap_or(ExpectedType::Any);
            ExpectedType::List(Box::new(element_type))
        }
    }
}

/// Get a type description from a Value (for error messages).
pub fn describe_value_type(value: &Value) -> String {
    match value {
        Value::Null => "Null".to_string(),
        Value::Bool(_) => "Bool".to_string(),
        Value::Int(_) => "Int".to_string(),
        Value::Float(_) => "Float".to_string(),
        Value::String(_) => "String".to_string(),
        Value::Tensor(t) => {
            let dtype = if t.is_integer() { "Int" } else { "Float" };
            format!("Tensor({})", dtype)
        }
        Value::Reference(r) => {
            if let Some(t) = &r.type_name {
                format!("Reference({})", t)
            } else {
                "Reference".to_string()
            }
        }
        Value::Expression(_) => "Expression".to_string(),
        Value::List(items) => {
            if items.is_empty() {
                "List".to_string()
            } else {
                let element_type = items
                    .iter()
                    .find(|v| !matches!(v, Value::Null))
                    .map(describe_value_type)
                    .unwrap_or_else(|| "Null".to_string());
                format!("List({})", element_type)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Reference;

    // ==================== ExpectedType::matches tests ====================

    #[test]
    fn test_expected_type_any_matches_all() {
        let any = ExpectedType::Any;
        assert!(any.matches(&Value::Null));
        assert!(any.matches(&Value::Bool(true)));
        assert!(any.matches(&Value::Int(42)));
        assert!(any.matches(&Value::Float(3.5)));
        assert!(any.matches(&Value::String("test".to_string().into())));
    }

    #[test]
    fn test_expected_type_null_matches() {
        let null_type = ExpectedType::Null;
        assert!(null_type.matches(&Value::Null));
        assert!(!null_type.matches(&Value::Bool(false)));
        assert!(!null_type.matches(&Value::Int(0)));
    }

    #[test]
    fn test_expected_type_bool_matches() {
        let bool_type = ExpectedType::Bool;
        assert!(bool_type.matches(&Value::Bool(true)));
        assert!(bool_type.matches(&Value::Bool(false)));
        assert!(!bool_type.matches(&Value::Int(1)));
        assert!(!bool_type.matches(&Value::String("true".to_string().into())));
    }

    #[test]
    fn test_expected_type_int_matches() {
        let int_type = ExpectedType::Int;
        assert!(int_type.matches(&Value::Int(42)));
        assert!(int_type.matches(&Value::Int(-100)));
        assert!(!int_type.matches(&Value::Float(42.0)));
        assert!(!int_type.matches(&Value::String("42".to_string().into())));
    }

    #[test]
    fn test_expected_type_float_matches() {
        let float_type = ExpectedType::Float;
        assert!(float_type.matches(&Value::Float(3.5)));
        assert!(float_type.matches(&Value::Float(0.0)));
        assert!(!float_type.matches(&Value::Int(3)));
        assert!(!float_type.matches(&Value::String("3.5".to_string().into())));
    }

    #[test]
    fn test_expected_type_numeric_matches() {
        let numeric_type = ExpectedType::Numeric;
        assert!(numeric_type.matches(&Value::Int(42)));
        assert!(numeric_type.matches(&Value::Float(3.5)));
        assert!(!numeric_type.matches(&Value::String("42".to_string().into())));
        assert!(!numeric_type.matches(&Value::Bool(true)));
    }

    #[test]
    fn test_expected_type_string_matches() {
        let string_type = ExpectedType::String;
        assert!(string_type.matches(&Value::String("test".to_string().into())));
        assert!(string_type.matches(&Value::String(String::new().into())));
        assert!(!string_type.matches(&Value::Int(42)));
        assert!(!string_type.matches(&Value::Bool(true)));
    }

    #[test]
    fn test_expected_type_reference_unqualified_matches() {
        let ref_type = ExpectedType::Reference { target_type: None };
        assert!(ref_type.matches(&Value::Reference(Reference::local("id"))));
        assert!(ref_type.matches(&Value::Reference(Reference::qualified("User", "id"))));
        assert!(!ref_type.matches(&Value::String("@id".to_string().into())));
    }

    #[test]
    fn test_expected_type_reference_qualified_matches() {
        let ref_type = ExpectedType::Reference {
            target_type: Some("User".to_string()),
        };
        assert!(ref_type.matches(&Value::Reference(Reference::qualified("User", "id"))));
        assert!(!ref_type.matches(&Value::Reference(Reference::qualified("Post", "id"))));
        assert!(!ref_type.matches(&Value::Reference(Reference::local("id"))));
    }

    #[test]
    fn test_expected_type_expression_matches() {
        use crate::lex::{Expression, Span};
        let expr_type = ExpectedType::Expression;
        let expr = Expression::Identifier {
            name: "x".to_string(),
            span: Span::synthetic(),
        };
        assert!(expr_type.matches(&Value::Expression(Box::new(expr))));
        assert!(!expr_type.matches(&Value::String("$(x)".to_string().into())));
    }

    #[test]
    fn test_expected_type_union_matches() {
        let union = ExpectedType::Union(vec![ExpectedType::Int, ExpectedType::String]);
        assert!(union.matches(&Value::Int(42)));
        assert!(union.matches(&Value::String("test".to_string().into())));
        assert!(!union.matches(&Value::Bool(true)));
        assert!(!union.matches(&Value::Float(3.5)));
    }

    // ==================== ExpectedType::describe tests ====================

    #[test]
    fn test_describe_any() {
        assert_eq!(ExpectedType::Any.describe(), "Any");
    }

    #[test]
    fn test_describe_null() {
        assert_eq!(ExpectedType::Null.describe(), "Null");
    }

    #[test]
    fn test_describe_bool() {
        assert_eq!(ExpectedType::Bool.describe(), "Bool");
    }

    #[test]
    fn test_describe_int() {
        assert_eq!(ExpectedType::Int.describe(), "Int");
    }

    #[test]
    fn test_describe_float() {
        assert_eq!(ExpectedType::Float.describe(), "Float");
    }

    #[test]
    fn test_describe_numeric() {
        assert_eq!(ExpectedType::Numeric.describe(), "Numeric (Int or Float)");
    }

    #[test]
    fn test_describe_string() {
        assert_eq!(ExpectedType::String.describe(), "String");
    }

    #[test]
    fn test_describe_tensor_basic() {
        let tensor = ExpectedType::Tensor {
            shape: None,
            dtype: None,
        };
        assert_eq!(tensor.describe(), "Tensor");
    }

    #[test]
    fn test_describe_tensor_with_shape() {
        let tensor = ExpectedType::Tensor {
            shape: Some(vec![2, 3]),
            dtype: None,
        };
        assert!(tensor.describe().contains("shape"));
        assert!(tensor.describe().contains("[2, 3]"));
    }

    #[test]
    fn test_describe_tensor_with_dtype() {
        let tensor = ExpectedType::Tensor {
            shape: None,
            dtype: Some(TensorDtype::Float),
        };
        assert!(tensor.describe().contains("dtype"));
        assert!(tensor.describe().contains("Float"));
    }

    #[test]
    fn test_describe_reference_unqualified() {
        let ref_type = ExpectedType::Reference { target_type: None };
        assert_eq!(ref_type.describe(), "Reference");
    }

    #[test]
    fn test_describe_reference_qualified() {
        let ref_type = ExpectedType::Reference {
            target_type: Some("User".to_string()),
        };
        assert_eq!(ref_type.describe(), "Reference(User)");
    }

    #[test]
    fn test_describe_expression() {
        assert_eq!(ExpectedType::Expression.describe(), "Expression");
    }

    #[test]
    fn test_describe_union() {
        let union = ExpectedType::Union(vec![ExpectedType::Int, ExpectedType::String]);
        let desc = union.describe();
        assert!(desc.contains("Union"));
        assert!(desc.contains("Int"));
        assert!(desc.contains("String"));
    }

    // ==================== ExpectedType::can_coerce tests ====================

    #[test]
    fn test_can_coerce_exact_match() {
        let int_type = ExpectedType::Int;
        assert!(int_type.can_coerce(&Value::Int(42), true));
    }

    #[test]
    fn test_can_coerce_int_to_float_strict() {
        let float_type = ExpectedType::Float;
        assert!(float_type.can_coerce(&Value::Int(42), true));
    }

    #[test]
    fn test_can_coerce_string_to_int_lenient() {
        let int_type = ExpectedType::Int;
        assert!(int_type.can_coerce(&Value::String("42".to_string().into()), false));
        assert!(int_type.can_coerce(&Value::String("  -100  ".to_string().into()), false));
    }

    #[test]
    fn test_cannot_coerce_string_to_int_strict() {
        let int_type = ExpectedType::Int;
        assert!(!int_type.can_coerce(&Value::String("42".to_string().into()), true));
    }

    #[test]
    fn test_can_coerce_string_to_float_lenient() {
        let float_type = ExpectedType::Float;
        assert!(float_type.can_coerce(&Value::String("3.5".to_string().into()), false));
        assert!(float_type.can_coerce(&Value::String("42".to_string().into()), false));
    }

    #[test]
    fn test_cannot_coerce_string_to_float_strict() {
        let float_type = ExpectedType::Float;
        assert!(!float_type.can_coerce(&Value::String("3.5".to_string().into()), true));
    }

    #[test]
    fn test_can_coerce_string_to_bool_lenient() {
        let bool_type = ExpectedType::Bool;
        assert!(bool_type.can_coerce(&Value::String("true".to_string().into()), false));
        assert!(bool_type.can_coerce(&Value::String("false".to_string().into()), false));
        assert!(bool_type.can_coerce(&Value::String("  true  ".to_string().into()), false));
    }

    #[test]
    fn test_cannot_coerce_string_to_bool_strict() {
        let bool_type = ExpectedType::Bool;
        assert!(!bool_type.can_coerce(&Value::String("true".to_string().into()), true));
    }

    #[test]
    fn test_cannot_coerce_invalid_string_to_number() {
        let int_type = ExpectedType::Int;
        assert!(!int_type.can_coerce(&Value::String("not_a_number".to_string().into()), false));

        let float_type = ExpectedType::Float;
        assert!(!float_type.can_coerce(&Value::String("not_a_number".to_string().into()), false));
    }

    #[test]
    fn test_can_coerce_to_string_lenient() {
        let string_type = ExpectedType::String;
        assert!(string_type.can_coerce(&Value::Int(42), false));
        assert!(string_type.can_coerce(&Value::Bool(true), false));
        assert!(string_type.can_coerce(&Value::Float(3.5), false));
    }

    #[test]
    fn test_cannot_coerce_to_string_strict() {
        let string_type = ExpectedType::String;
        assert!(!string_type.can_coerce(&Value::Int(42), true));
        assert!(!string_type.can_coerce(&Value::Bool(true), true));
    }

    #[test]
    fn test_can_coerce_numeric_lenient() {
        let numeric_type = ExpectedType::Numeric;
        assert!(numeric_type.can_coerce(&Value::String("42".to_string().into()), false));
        assert!(numeric_type.can_coerce(&Value::String("3.5".to_string().into()), false));
    }

    #[test]
    fn test_can_coerce_union() {
        let union = ExpectedType::Union(vec![ExpectedType::Int, ExpectedType::Bool]);
        // Can coerce string to int in lenient mode
        assert!(union.can_coerce(&Value::String("42".to_string().into()), false));
        // Can coerce string to bool in lenient mode
        assert!(union.can_coerce(&Value::String("true".to_string().into()), false));
    }

    #[test]
    fn test_any_accepts_all_coercion() {
        let any = ExpectedType::Any;
        assert!(any.can_coerce(&Value::Int(42), true));
        assert!(any.can_coerce(&Value::String("test".to_string().into()), true));
        assert!(any.can_coerce(&Value::Bool(true), true));
    }

    // ==================== value_to_expected_type tests ====================

    #[test]
    fn test_value_to_expected_type_null() {
        assert_eq!(value_to_expected_type(&Value::Null), ExpectedType::Null);
    }

    #[test]
    fn test_value_to_expected_type_bool() {
        assert_eq!(
            value_to_expected_type(&Value::Bool(true)),
            ExpectedType::Bool
        );
    }

    #[test]
    fn test_value_to_expected_type_int() {
        assert_eq!(value_to_expected_type(&Value::Int(42)), ExpectedType::Int);
    }

    #[test]
    fn test_value_to_expected_type_float() {
        assert_eq!(
            value_to_expected_type(&Value::Float(3.5)),
            ExpectedType::Float
        );
    }

    #[test]
    fn test_value_to_expected_type_string() {
        assert_eq!(
            value_to_expected_type(&Value::String("test".to_string().into())),
            ExpectedType::String
        );
    }

    #[test]
    fn test_value_to_expected_type_reference_local() {
        let ref_type = value_to_expected_type(&Value::Reference(Reference::local("id")));
        assert_eq!(ref_type, ExpectedType::Reference { target_type: None });
    }

    #[test]
    fn test_value_to_expected_type_reference_qualified() {
        let ref_type =
            value_to_expected_type(&Value::Reference(Reference::qualified("User", "id")));
        assert_eq!(
            ref_type,
            ExpectedType::Reference {
                target_type: Some("User".to_string())
            }
        );
    }

    // ==================== describe_value_type tests ====================

    #[test]
    fn test_describe_value_type_null() {
        assert_eq!(describe_value_type(&Value::Null), "Null");
    }

    #[test]
    fn test_describe_value_type_bool() {
        assert_eq!(describe_value_type(&Value::Bool(true)), "Bool");
    }

    #[test]
    fn test_describe_value_type_int() {
        assert_eq!(describe_value_type(&Value::Int(42)), "Int");
    }

    #[test]
    fn test_describe_value_type_float() {
        assert_eq!(describe_value_type(&Value::Float(3.5)), "Float");
    }

    #[test]
    fn test_describe_value_type_string() {
        assert_eq!(
            describe_value_type(&Value::String("test".to_string().into())),
            "String"
        );
    }

    #[test]
    fn test_describe_value_type_reference_local() {
        assert_eq!(
            describe_value_type(&Value::Reference(Reference::local("id"))),
            "Reference"
        );
    }

    #[test]
    fn test_describe_value_type_reference_qualified() {
        assert_eq!(
            describe_value_type(&Value::Reference(Reference::qualified("User", "id"))),
            "Reference(User)"
        );
    }

    #[test]
    fn test_describe_value_type_expression() {
        use crate::lex::{Expression, Span};
        let expr = Expression::Identifier {
            name: "x".to_string(),
            span: Span::synthetic(),
        };
        assert_eq!(
            describe_value_type(&Value::Expression(Box::new(expr))),
            "Expression"
        );
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_union_empty() {
        let union = ExpectedType::Union(vec![]);
        assert!(!union.matches(&Value::Int(42)));
        assert_eq!(union.describe(), "Union()");
    }

    #[test]
    fn test_union_single_type() {
        let union = ExpectedType::Union(vec![ExpectedType::Int]);
        assert!(union.matches(&Value::Int(42)));
        assert!(!union.matches(&Value::String("test".to_string().into())));
    }

    #[test]
    fn test_numeric_accepts_both_int_and_float() {
        let numeric = ExpectedType::Numeric;
        assert!(numeric.matches(&Value::Int(42)));
        assert!(numeric.matches(&Value::Float(3.5)));
        assert!(!numeric.matches(&Value::String("42".to_string().into())));
    }

    #[test]
    fn test_coerce_invalid_bool_string() {
        let bool_type = ExpectedType::Bool;
        assert!(!bool_type.can_coerce(&Value::String("yes".to_string().into()), false));
        assert!(!bool_type.can_coerce(&Value::String("1".to_string().into()), false));
    }

    #[test]
    fn test_coerce_whitespace_handling() {
        let int_type = ExpectedType::Int;
        assert!(int_type.can_coerce(&Value::String("  42  ".to_string().into()), false));

        let bool_type = ExpectedType::Bool;
        assert!(bool_type.can_coerce(&Value::String("  true  ".to_string().into()), false));
    }
}

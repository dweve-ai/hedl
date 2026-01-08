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

//! Value types for HEDL scalars.

use crate::lex::{Expression, Tensor};

/// A reference to another node.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Reference {
    /// Optional type qualifier (e.g., "User" in "@User:id").
    pub type_name: Option<String>,
    /// The ID being referenced.
    pub id: String,
}

impl Reference {
    /// Create a local reference (no type qualifier).
    pub fn local(id: impl Into<String>) -> Self {
        Self {
            type_name: None,
            id: id.into(),
        }
    }

    /// Create a qualified reference with type name.
    pub fn qualified(type_name: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            type_name: Some(type_name.into()),
            id: id.into(),
        }
    }

    /// Format as a reference string (with @).
    pub fn to_ref_string(&self) -> String {
        match &self.type_name {
            Some(t) => format!("@{}:{}", t, self.id),
            None => format!("@{}", self.id),
        }
    }
}

/// A scalar value in HEDL.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Null value (~).
    Null,
    /// Boolean value (true/false).
    Bool(bool),
    /// Integer value.
    Int(i64),
    /// Floating-point value.
    Float(f64),
    /// String value.
    String(String),
    /// Tensor (multi-dimensional array).
    Tensor(Tensor),
    /// Reference to another node.
    Reference(Reference),
    /// Parsed expression from $(...).
    Expression(Expression),
}

impl Value {
    /// Returns true if this value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns true if this value is a reference.
    pub fn is_reference(&self) -> bool {
        matches!(self, Self::Reference(_))
    }

    /// Try to get the value as a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get the value as an integer.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to get the value as a float.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(n) => Some(*n),
            Self::Int(n) => Some(*n as f64),
            _ => None,
        }
    }

    /// Try to get the value as a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get the value as a reference.
    pub fn as_reference(&self) -> Option<&Reference> {
        match self {
            Self::Reference(r) => Some(r),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "~"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Int(n) => write!(f, "{}", n),
            Self::Float(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::Tensor(_) => write!(f, "[tensor]"),
            Self::Reference(r) => write!(f, "{}", r.to_ref_string()),
            Self::Expression(e) => write!(f, "$({})", e),
        }
    }
}

impl Value {
    /// Try to get the expression if this is an Expression variant.
    pub fn as_expression(&self) -> Option<&Expression> {
        match self {
            Self::Expression(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Reference tests ====================

    #[test]
    fn test_reference_local() {
        let r = Reference::local("user-123");
        assert_eq!(r.type_name, None);
        assert_eq!(r.id, "user-123");
    }

    #[test]
    fn test_reference_qualified() {
        let r = Reference::qualified("User", "123");
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "123");
    }

    #[test]
    fn test_reference_to_ref_string_local() {
        let r = Reference::local("id-1");
        assert_eq!(r.to_ref_string(), "@id-1");
    }

    #[test]
    fn test_reference_to_ref_string_qualified() {
        let r = Reference::qualified("User", "id-1");
        assert_eq!(r.to_ref_string(), "@User:id-1");
    }

    #[test]
    fn test_reference_equality() {
        let a = Reference::qualified("User", "1");
        let b = Reference::qualified("User", "1");
        assert_eq!(a, b);
    }

    #[test]
    fn test_reference_inequality() {
        let a = Reference::qualified("User", "1");
        let b = Reference::qualified("Post", "1");
        assert_ne!(a, b);
    }

    #[test]
    fn test_reference_clone() {
        let original = Reference::qualified("Type", "id");
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_reference_debug() {
        let r = Reference::qualified("User", "abc");
        let debug = format!("{:?}", r);
        assert!(debug.contains("User"));
        assert!(debug.contains("abc"));
    }

    // ==================== Value::is_* tests ====================

    #[test]
    fn test_value_is_null() {
        assert!(Value::Null.is_null());
        assert!(!Value::Bool(true).is_null());
        assert!(!Value::Int(0).is_null());
    }

    #[test]
    fn test_value_is_reference() {
        let r = Reference::local("id");
        assert!(Value::Reference(r).is_reference());
        assert!(!Value::Null.is_reference());
        assert!(!Value::String("@ref".to_string()).is_reference());
    }

    // ==================== Value::as_* tests ====================

    #[test]
    fn test_value_as_str() {
        let v = Value::String("hello".to_string());
        assert_eq!(v.as_str(), Some("hello"));
        assert_eq!(Value::Null.as_str(), None);
        assert_eq!(Value::Int(42).as_str(), None);
    }

    #[test]
    fn test_value_as_int() {
        assert_eq!(Value::Int(42).as_int(), Some(42));
        assert_eq!(Value::Int(-100).as_int(), Some(-100));
        assert_eq!(Value::Float(3.5).as_int(), None);
        assert_eq!(Value::String("42".to_string()).as_int(), None);
    }

    #[test]
    fn test_value_as_float() {
        assert_eq!(Value::Float(3.5).as_float(), Some(3.5));
        // Int converts to float
        assert_eq!(Value::Int(42).as_float(), Some(42.0));
        assert_eq!(Value::String("3.5".to_string()).as_float(), None);
    }

    #[test]
    fn test_value_as_bool() {
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Bool(false).as_bool(), Some(false));
        assert_eq!(Value::Int(1).as_bool(), None);
        assert_eq!(Value::String("true".to_string()).as_bool(), None);
    }

    #[test]
    fn test_value_as_reference() {
        let r = Reference::local("id");
        let v = Value::Reference(r.clone());
        assert_eq!(v.as_reference(), Some(&r));
        assert_eq!(Value::Null.as_reference(), None);
    }

    #[test]
    fn test_value_as_expression() {
        use crate::lex::{Expression, Span};
        let expr = Expression::Identifier {
            name: "x".to_string(),
            span: Span::default(),
        };
        let v = Value::Expression(expr.clone());
        assert_eq!(v.as_expression(), Some(&expr));
        assert_eq!(Value::Null.as_expression(), None);
    }

    // ==================== Value Display tests ====================

    #[test]
    fn test_value_display_null() {
        assert_eq!(format!("{}", Value::Null), "~");
    }

    #[test]
    fn test_value_display_bool() {
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Bool(false)), "false");
    }

    #[test]
    fn test_value_display_int() {
        assert_eq!(format!("{}", Value::Int(42)), "42");
        assert_eq!(format!("{}", Value::Int(-100)), "-100");
        assert_eq!(format!("{}", Value::Int(0)), "0");
    }

    #[test]
    fn test_value_display_float() {
        let s = format!("{}", Value::Float(3.5));
        assert!(s.starts_with("3.5"));
    }

    #[test]
    fn test_value_display_string() {
        assert_eq!(format!("{}", Value::String("hello".to_string())), "hello");
    }

    #[test]
    fn test_value_display_reference() {
        let r = Reference::qualified("User", "123");
        assert_eq!(format!("{}", Value::Reference(r)), "@User:123");
    }

    #[test]
    fn test_value_display_expression() {
        use crate::lex::{Expression, Span};
        let expr = Expression::Identifier {
            name: "x".to_string(),
            span: Span::default(),
        };
        assert_eq!(format!("{}", Value::Expression(expr)), "$(x)");
    }

    #[test]
    fn test_value_display_tensor() {
        use crate::lex::Tensor;
        let t = Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]);
        assert_eq!(format!("{}", Value::Tensor(t)), "[tensor]");
    }

    // ==================== Value equality and clone ====================

    #[test]
    fn test_value_equality_null() {
        assert_eq!(Value::Null, Value::Null);
    }

    #[test]
    fn test_value_equality_bool() {
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_ne!(Value::Bool(true), Value::Bool(false));
    }

    #[test]
    fn test_value_equality_int() {
        assert_eq!(Value::Int(42), Value::Int(42));
        assert_ne!(Value::Int(42), Value::Int(43));
    }

    #[test]
    fn test_value_equality_string() {
        assert_eq!(
            Value::String("test".to_string()),
            Value::String("test".to_string())
        );
        assert_ne!(
            Value::String("a".to_string()),
            Value::String("b".to_string())
        );
    }

    #[test]
    fn test_value_inequality_different_types() {
        assert_ne!(Value::Int(1), Value::Bool(true));
        assert_ne!(Value::Null, Value::Bool(false));
        assert_ne!(Value::String("42".to_string()), Value::Int(42));
    }

    #[test]
    fn test_value_clone() {
        let values = vec![
            Value::Null,
            Value::Bool(true),
            Value::Int(42),
            Value::Float(3.5),
            Value::String("test".to_string()),
            Value::Reference(Reference::local("id")),
        ];

        for v in values {
            let cloned = v.clone();
            assert_eq!(v, cloned);
        }
    }

    #[test]
    fn test_value_debug() {
        let v = Value::Int(42);
        let debug = format!("{:?}", v);
        assert!(debug.contains("Int"));
        assert!(debug.contains("42"));
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_value_int_bounds() {
        assert_eq!(Value::Int(i64::MAX).as_int(), Some(i64::MAX));
        assert_eq!(Value::Int(i64::MIN).as_int(), Some(i64::MIN));
    }

    #[test]
    fn test_value_empty_string() {
        let v = Value::String(String::new());
        assert_eq!(v.as_str(), Some(""));
    }

    #[test]
    fn test_value_unicode_string() {
        let v = Value::String("日本語 🎉".to_string());
        assert_eq!(v.as_str(), Some("日本語 🎉"));
    }

    #[test]
    fn test_value_float_special() {
        let inf = Value::Float(f64::INFINITY);
        assert!(inf.as_float().unwrap().is_infinite());

        let nan = Value::Float(f64::NAN);
        assert!(nan.as_float().unwrap().is_nan());
    }

    #[test]
    fn test_reference_empty_id() {
        let r = Reference::local("");
        assert_eq!(r.id, "");
        assert_eq!(r.to_ref_string(), "@");
    }

    #[test]
    fn test_reference_with_special_chars() {
        let r = Reference::local("id-with-hyphens-123");
        assert_eq!(r.to_ref_string(), "@id-with-hyphens-123");
    }
}

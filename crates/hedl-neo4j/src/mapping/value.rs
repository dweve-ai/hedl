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

//! Mapping between HEDL values and Cypher values.

use hedl_core::Value;
use hedl_core::lex::Tensor;
use std::collections::BTreeMap;

use crate::config::ToCypherConfig;
use crate::cypher::{validate_string_length, CypherValue};
use crate::error::{Neo4jError, Result};

#[allow(unused_imports)]
use serde_json;

/// Convert a HEDL Value to a CypherValue.
///
/// # Arguments
///
/// * `value` - The HEDL value to convert
/// * `property_name` - The property name (for error reporting in string length validation)
/// * `config` - Configuration with limits and conversion options
///
/// # Errors
///
/// Returns `Neo4jError::StringLengthExceeded` if a string value exceeds the configured limit.
pub fn value_to_cypher(
    value: &Value,
    property_name: &str,
    config: &ToCypherConfig,
) -> Result<CypherValue> {
    match value {
        Value::Null => Ok(CypherValue::Null),
        Value::Bool(b) => Ok(CypherValue::Bool(*b)),
        Value::Int(i) => Ok(CypherValue::Int(*i)),
        Value::Float(f) => Ok(CypherValue::Float(*f)),
        Value::String(s) => {
            // Validate string length before conversion
            validate_string_length(s, property_name, config)?;
            Ok(CypherValue::String(s.clone()))
        }
        Value::Tensor(t) => tensor_to_cypher(t, property_name, config),
        Value::Reference(r) => {
            // References are handled separately in relationship generation
            // Here we just store the reference as a string for property storage
            let ref_string = if let Some(type_name) = &r.type_name {
                format!("@{}:{}", type_name, r.id)
            } else {
                format!("@{}", r.id)
            };
            validate_string_length(&ref_string, property_name, config)?;
            Ok(CypherValue::String(ref_string))
        }
        Value::Expression(e) => {
            // Expressions are stored as strings with $() preserved
            let expr_string = format!("$({})", e);
            validate_string_length(&expr_string, property_name, config)?;
            Ok(CypherValue::String(expr_string))
        }
    }
}

/// Convert a tensor to a Cypher value (as JSON string).
fn tensor_to_cypher(tensor: &Tensor, property_name: &str, config: &ToCypherConfig) -> Result<CypherValue> {
    let json = tensor_to_json(tensor)?;
    // Validate the serialized JSON string length
    validate_string_length(&json, property_name, config)?;
    Ok(CypherValue::String(json))
}

/// Convert a tensor to a JSON string.
fn tensor_to_json(tensor: &Tensor) -> Result<String> {
    fn tensor_to_value(t: &Tensor) -> serde_json::Value {
        match t {
            Tensor::Scalar(f) => serde_json::Value::Number(
                serde_json::Number::from_f64(*f).unwrap_or(serde_json::Number::from(0)),
            ),
            Tensor::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(tensor_to_value).collect())
            }
        }
    }

    let json_value = tensor_to_value(tensor);
    serde_json::to_string(&json_value).map_err(Neo4jError::JsonError)
}

/// Convert a CypherValue back to a HEDL Value.
///
/// Note: CypherValue::List and CypherValue::Map are converted to strings
/// since HEDL Value does not have List/Object variants.
pub fn cypher_to_value(value: &CypherValue) -> Result<Value> {
    Ok(match value {
        CypherValue::Null => Value::Null,
        CypherValue::Bool(b) => Value::Bool(*b),
        CypherValue::Int(i) => Value::Int(*i),
        CypherValue::Float(f) => Value::Float(*f),
        CypherValue::String(s) => {
            // Try to detect special string formats
            if s.starts_with('@') {
                // This might be a reference
                parse_reference_string(s)
            } else if s.starts_with('[') && s.ends_with(']') {
                // This might be a tensor
                parse_tensor_string(s)?
            } else {
                Value::String(s.clone())
            }
        }
        CypherValue::List(items) => {
            // Convert list to JSON string since Value doesn't have List variant
            let json = serde_json::to_string(items).map_err(Neo4jError::JsonError)?;
            Value::String(json)
        }
        CypherValue::Map(map) => {
            // Convert map to JSON string since Value doesn't have Object variant
            let json = serde_json::to_string(map).map_err(Neo4jError::JsonError)?;
            Value::String(json)
        }
    })
}

/// Parse a reference string like "@Type:id" or "@id".
fn parse_reference_string(s: &str) -> Value {
    let s = s.trim_start_matches('@');
    if let Some((type_name, id)) = s.split_once(':') {
        Value::Reference(hedl_core::Reference {
            type_name: Some(type_name.to_string()),
            id: id.to_string(),
        })
    } else {
        Value::Reference(hedl_core::Reference {
            type_name: None,
            id: s.to_string(),
        })
    }
}

/// Parse a tensor string like "[1, 2, 3]".
fn parse_tensor_string(s: &str) -> Result<Value> {
    match hedl_core::lex::parse_tensor(s) {
        Ok(tensor) => Ok(Value::Tensor(tensor)),
        Err(_) => Ok(Value::String(s.to_string())),
    }
}

/// Convert properties to HEDL values.
///
/// Dot-notation properties are kept as-is since HEDL Value doesn't support nesting.
/// Use the full key (e.g., "address.city") to preserve the original structure.
pub fn unflatten_properties(
    properties: &BTreeMap<String, CypherValue>,
) -> Result<BTreeMap<String, Value>> {
    let mut result: BTreeMap<String, Value> = BTreeMap::new();

    for (key, value) in properties {
        result.insert(key.clone(), cypher_to_value(value)?);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::Reference;

    #[test]
    fn test_value_to_cypher_primitives() {
        let config = ToCypherConfig::default();
        assert_eq!(
            value_to_cypher(&Value::Null, "field", &config).unwrap(),
            CypherValue::Null
        );
        assert_eq!(
            value_to_cypher(&Value::Bool(true), "field", &config).unwrap(),
            CypherValue::Bool(true)
        );
        assert_eq!(
            value_to_cypher(&Value::Int(42), "field", &config).unwrap(),
            CypherValue::Int(42)
        );
        assert_eq!(
            value_to_cypher(&Value::Float(3.25), "field", &config).unwrap(),
            CypherValue::Float(3.25)
        );
        assert_eq!(
            value_to_cypher(&Value::String("hello".to_string()), "field", &config).unwrap(),
            CypherValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_value_to_cypher_reference() {
        let config = ToCypherConfig::default();
        let ref_with_type = Value::Reference(Reference {
            type_name: Some("User".to_string()),
            id: "alice".to_string(),
        });
        assert_eq!(
            value_to_cypher(&ref_with_type, "author", &config).unwrap(),
            CypherValue::String("@User:alice".to_string())
        );

        let ref_without_type = Value::Reference(Reference {
            type_name: None,
            id: "bob".to_string(),
        });
        assert_eq!(
            value_to_cypher(&ref_without_type, "ref", &config).unwrap(),
            CypherValue::String("@bob".to_string())
        );
    }

    // TODO: Update this test to match the new Expression API structure
    // #[test]
    // fn test_value_to_cypher_expression() {
    //     use hedl_core::{ExprLiteral, Expression};
    //     let config = ToCypherConfig::default();
    //     let expr = Value::Expression(Expression::Call {
    //         name: "add".to_string(),
    //         args: vec![
    //             Expression::Literal(ExprLiteral::Int(1)),
    //             Expression::Literal(ExprLiteral::Int(2)),
    //         ],
    //     });
    //     let result = value_to_cypher(&expr, "calc", &config).unwrap();
    //     if let CypherValue::String(s) = result {
    //         assert!(s.starts_with("$("));
    //         assert!(s.ends_with(')'));
    //     } else {
    //         panic!("Expected string value for expression");
    //     }
    // }

    #[test]
    fn test_value_to_cypher_tensor() {
        let config = ToCypherConfig::default();
        let tensor = Value::Tensor(hedl_core::lex::parse_tensor("[1, 2, 3]").unwrap());
        let result = value_to_cypher(&tensor, "data", &config).unwrap();
        if let CypherValue::String(s) = result {
            assert!(s.contains('1'));
            assert!(s.contains('2'));
            assert!(s.contains('3'));
        } else {
            panic!("Expected string value for tensor");
        }
    }

    #[test]
    fn test_cypher_to_value_primitives() {
        assert_eq!(cypher_to_value(&CypherValue::Null).unwrap(), Value::Null);
        assert_eq!(
            cypher_to_value(&CypherValue::Bool(true)).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            cypher_to_value(&CypherValue::Int(42)).unwrap(),
            Value::Int(42)
        );
        assert_eq!(
            cypher_to_value(&CypherValue::Float(3.25)).unwrap(),
            Value::Float(3.25)
        );
        assert_eq!(
            cypher_to_value(&CypherValue::String("hello".to_string())).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_cypher_to_value_reference_string() {
        let result = cypher_to_value(&CypherValue::String("@User:alice".to_string())).unwrap();
        if let Value::Reference(r) = result {
            assert_eq!(r.type_name, Some("User".to_string()));
            assert_eq!(r.id, "alice");
        } else {
            panic!("Expected reference value");
        }
    }

    #[test]
    fn test_unflatten_properties() {
        let mut props = BTreeMap::new();
        props.insert("name".to_string(), CypherValue::String("Alice".to_string()));
        props.insert(
            "address.city".to_string(),
            CypherValue::String("NYC".to_string()),
        );
        props.insert(
            "address.zip".to_string(),
            CypherValue::String("10001".to_string()),
        );

        let result = unflatten_properties(&props).unwrap();

        assert_eq!(
            result.get("name"),
            Some(&Value::String("Alice".to_string()))
        );

        // Dot-notation properties are kept as-is
        assert_eq!(
            result.get("address.city"),
            Some(&Value::String("NYC".to_string()))
        );
        assert_eq!(
            result.get("address.zip"),
            Some(&Value::String("10001".to_string()))
        );
    }

    #[test]
    fn test_parse_reference_string() {
        let result = parse_reference_string("@User:alice");
        if let Value::Reference(r) = result {
            assert_eq!(r.type_name, Some("User".to_string()));
            assert_eq!(r.id, "alice");
        } else {
            panic!("Expected reference");
        }

        let result2 = parse_reference_string("@bob");
        if let Value::Reference(r) = result2 {
            assert_eq!(r.type_name, None);
            assert_eq!(r.id, "bob");
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_value_to_cypher_string_length_validation() {
        let config = ToCypherConfig::default().with_max_string_length(100);

        // String within limit
        let short_string = Value::String("short".to_string());
        assert!(value_to_cypher(&short_string, "name", &config).is_ok());

        // String exceeding limit
        let long_string = Value::String("x".repeat(101));
        let result = value_to_cypher(&long_string, "description", &config);
        assert!(result.is_err());
        if let Err(Neo4jError::StringLengthExceeded { length, max_length, property }) = result {
            assert_eq!(length, 101);
            assert_eq!(max_length, 100);
            assert_eq!(property, "description");
        } else {
            panic!("Expected StringLengthExceeded error");
        }
    }

    #[test]
    fn test_value_to_cypher_reference_length_validation() {
        let config = ToCypherConfig::default().with_max_string_length(20);

        // Short reference
        let short_ref = Value::Reference(Reference {
            type_name: Some("User".to_string()),
            id: "alice".to_string(),
        });
        assert!(value_to_cypher(&short_ref, "author", &config).is_ok());

        // Long reference that exceeds limit
        let long_id = "x".repeat(100);
        let long_ref = Value::Reference(Reference {
            type_name: Some("User".to_string()),
            id: long_id,
        });
        let result = value_to_cypher(&long_ref, "author", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_value_to_cypher_tensor_length_validation() {
        let config = ToCypherConfig::default().with_max_string_length(50);

        // Small tensor
        let small_tensor = Value::Tensor(hedl_core::lex::parse_tensor("[1, 2, 3]").unwrap());
        assert!(value_to_cypher(&small_tensor, "data", &config).is_ok());

        // Large tensor that exceeds limit when serialized
        let large_values: Vec<f64> = (0..1000).map(|i| i as f64).collect();
        let large_tensor = Tensor::Array(large_values.iter().map(|&v| Tensor::Scalar(v)).collect());
        let large_value = Value::Tensor(large_tensor);
        let result = value_to_cypher(&large_value, "bigdata", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_value_to_cypher_no_limit() {
        let config = ToCypherConfig::default().without_string_length_limit();

        // Very long string should be OK
        let huge_string = Value::String("x".repeat(1_000_000));
        assert!(value_to_cypher(&huge_string, "huge", &config).is_ok());
    }
}

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

//! JSON Schema validation for hedl-json
//!
//! This module provides JSON Schema validation capabilities using the `jsonschema` crate.
//! Validation is optional and enabled via the `validation` feature flag.
//!
//! # Features
//!
//! - Support for JSON Schema Draft-07, Draft 2019-09, and Draft 2020-12
//! - Schema compilation and caching for efficient repeated validation
//! - Comprehensive error reporting with JSON path locations
//! - Optional format validation (email, uri, date-time, uuid, etc.)
//!
//! # Examples
//!
//! ```ignore
//! use hedl_json::validation::{CompiledSchema, ValidationConfig, SchemaDraft};
//! use serde_json::json;
//!
//! let schema = json!({
//!     "$schema": "http://json-schema.org/draft-07/schema#",
//!     "type": "object",
//!     "properties": {
//!         "name": {"type": "string", "minLength": 1},
//!         "age": {"type": "integer", "minimum": 0}
//!     },
//!     "required": ["name", "age"]
//! });
//!
//! let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default())?;
//!
//! let valid_json = json!({"name": "Alice", "age": 30});
//! assert!(compiled.validate(&valid_json).is_valid);
//!
//! let invalid_json = json!({"name": "", "age": -5});
//! let result = compiled.validate(&invalid_json);
//! assert!(!result.is_valid);
//! for error in &result.errors {
//!     println!("Error at {}: {}", error.instance_path, error.message);
//! }
//! ```

use jsonschema::Validator;
use serde_json::Value as JsonValue;
use std::sync::Arc;

/// JSON Schema draft version
///
/// Specifies which JSON Schema draft specification to use for validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SchemaDraft {
    /// JSON Schema Draft 4
    Draft4,
    /// JSON Schema Draft 6
    Draft6,
    /// JSON Schema Draft 7 (recommended - most widely supported)
    #[default]
    Draft7,
    /// JSON Schema Draft 2019-09
    Draft201909,
    /// JSON Schema Draft 2020-12 (latest)
    Draft202012,
}

/// JSON Schema validation configuration
///
/// Controls how validation is performed, including which draft version to use
/// and how errors are collected.
///
/// # Examples
///
/// ```ignore
/// use hedl_json::validation::{ValidationConfig, SchemaDraft};
///
/// // Strict validation with all errors
/// let strict = ValidationConfig {
///     draft: SchemaDraft::Draft7,
///     collect_all_errors: true,
///     max_errors: None,
///     validate_formats: true,
/// };
///
/// // Lenient validation - stop at first error
/// let lenient = ValidationConfig {
///     draft: SchemaDraft::Draft7,
///     collect_all_errors: false,
///     max_errors: Some(1),
///     validate_formats: false,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// JSON Schema draft version to use
    ///
    /// Supported drafts:
    /// - `SchemaDraft::Draft4`: JSON Schema Draft 4
    /// - `SchemaDraft::Draft6`: JSON Schema Draft 6
    /// - `SchemaDraft::Draft7`: JSON Schema Draft 7 (recommended)
    /// - `SchemaDraft::Draft201909`: JSON Schema Draft 2019-09
    /// - `SchemaDraft::Draft202012`: JSON Schema Draft 2020-12
    pub draft: SchemaDraft,

    /// Continue validation after first error
    ///
    /// When `true`, collects all errors in the document.
    /// When `false`, stops at the first error (faster for invalid docs).
    pub collect_all_errors: bool,

    /// Maximum errors to collect before stopping
    ///
    /// `None` means collect all errors (when `collect_all_errors` is true).
    /// Useful for limiting memory usage with highly invalid documents.
    pub max_errors: Option<usize>,

    /// Enable format validation
    ///
    /// When `true`, validates format keywords like "email", "uri", "date-time".
    /// When `false`, format keywords are ignored (faster, more permissive).
    pub validate_formats: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            draft: SchemaDraft::Draft7,
            collect_all_errors: true,
            max_errors: None,
            validate_formats: true,
        }
    }
}

/// Validation error with location information
///
/// Represents a single validation failure with details about where
/// it occurred and what went wrong.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// JSON path to the error location (e.g., "/users/0/email")
    pub instance_path: String,

    /// Human-readable error message
    pub message: String,

    /// Schema path that failed (e.g., "/properties/email/format")
    pub schema_path: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Validation error at {}: {} (schema: {})",
            self.instance_path, self.message, self.schema_path
        )
    }
}

impl std::error::Error for ValidationError {}

/// Result of validating a JSON document against a schema
///
/// Contains the validation outcome and any errors found.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the document is valid according to the schema
    pub is_valid: bool,

    /// List of validation errors (empty if valid)
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    /// Create a successful validation result
    #[must_use]
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    /// Create a failed validation result with errors
    #[must_use]
    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }
}

/// Schema compilation error
#[derive(Debug, Clone, thiserror::Error)]
pub enum SchemaError {
    /// The schema itself is invalid
    #[error("Invalid JSON Schema: {0}")]
    InvalidSchema(String),

    /// Schema reference could not be resolved
    #[error("Unresolved schema reference: {0}")]
    UnresolvedReference(String),
}

/// Compiled JSON Schema for efficient repeated validation
///
/// Compiling a schema is relatively expensive, so this type caches
/// the compiled schema for efficient reuse.
///
/// # Thread Safety
///
/// `CompiledSchema` uses `Arc` internally and is safe to share
/// across threads.
///
/// # Examples
///
/// ```ignore
/// use hedl_json::validation::{CompiledSchema, ValidationConfig};
/// use serde_json::json;
///
/// let schema = json!({
///     "type": "object",
///     "properties": {
///         "id": {"type": "integer"}
///     }
/// });
///
/// let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default())?;
///
/// // Validate multiple documents efficiently
/// let doc1 = json!({"id": 1});
/// let doc2 = json!({"id": 2});
///
/// assert!(compiled.validate(&doc1).is_valid);
/// assert!(compiled.validate(&doc2).is_valid);
/// ```
#[derive(Clone)]
pub struct CompiledSchema {
    validator: Arc<Validator>,
    config: ValidationConfig,
}

impl std::fmt::Debug for CompiledSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledSchema")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl CompiledSchema {
    /// Compile a JSON Schema for validation
    ///
    /// # Arguments
    ///
    /// * `schema` - JSON Schema document as a `serde_json::Value`
    /// * `config` - Validation configuration
    ///
    /// # Returns
    ///
    /// * `Ok(CompiledSchema)` - Successfully compiled schema
    /// * `Err(SchemaError)` - Schema is invalid
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use hedl_json::validation::{CompiledSchema, ValidationConfig};
    /// use serde_json::json;
    ///
    /// let schema = json!({
    ///     "$schema": "http://json-schema.org/draft-07/schema#",
    ///     "type": "string",
    ///     "minLength": 1
    /// });
    ///
    /// let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default())?;
    /// ```
    pub fn compile(schema: &JsonValue, config: &ValidationConfig) -> Result<Self, SchemaError> {
        let validator = match config.draft {
            SchemaDraft::Draft4 => jsonschema::draft4::options()
                .should_validate_formats(config.validate_formats)
                .build(schema),
            SchemaDraft::Draft6 => jsonschema::draft6::options()
                .should_validate_formats(config.validate_formats)
                .build(schema),
            SchemaDraft::Draft7 => jsonschema::draft7::options()
                .should_validate_formats(config.validate_formats)
                .build(schema),
            SchemaDraft::Draft201909 => jsonschema::draft201909::options()
                .should_validate_formats(config.validate_formats)
                .build(schema),
            SchemaDraft::Draft202012 => jsonschema::draft202012::options()
                .should_validate_formats(config.validate_formats)
                .build(schema),
        }
        .map_err(|e| SchemaError::InvalidSchema(e.to_string()))?;

        Ok(Self {
            validator: Arc::new(validator),
            config: config.clone(),
        })
    }

    /// Validate a JSON value against the schema
    ///
    /// # Arguments
    ///
    /// * `instance` - JSON document to validate
    ///
    /// # Returns
    ///
    /// `ValidationResult` with validation outcome and any errors
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use hedl_json::validation::{CompiledSchema, ValidationConfig};
    /// use serde_json::json;
    ///
    /// let schema = json!({"type": "integer"});
    /// let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default())?;
    ///
    /// let result = compiled.validate(&json!(42));
    /// assert!(result.is_valid);
    ///
    /// let result = compiled.validate(&json!("not an integer"));
    /// assert!(!result.is_valid);
    /// ```
    #[must_use]
    pub fn validate(&self, instance: &JsonValue) -> ValidationResult {
        if self.validator.is_valid(instance) {
            return ValidationResult::valid();
        }

        let max = self.config.max_errors.unwrap_or(usize::MAX);
        let limit = if self.config.collect_all_errors {
            max
        } else {
            1
        };

        let collected: Vec<ValidationError> = self
            .validator
            .iter_errors(instance)
            .take(limit)
            .map(|e| ValidationError {
                instance_path: e.instance_path.to_string(),
                message: e.to_string(),
                schema_path: e.schema_path.to_string(),
            })
            .collect();

        ValidationResult::invalid(collected)
    }

    /// Check if a JSON value is valid without collecting errors
    ///
    /// This is faster than `validate()` when you only need to know
    /// whether the document is valid.
    ///
    /// # Arguments
    ///
    /// * `instance` - JSON document to validate
    ///
    /// # Returns
    ///
    /// `true` if valid, `false` otherwise
    #[must_use]
    pub fn is_valid(&self, instance: &JsonValue) -> bool {
        self.validator.is_valid(instance)
    }
}

/// Validate JSON against a schema (convenience function)
///
/// Compiles the schema and validates the instance in one call.
/// Use `CompiledSchema` for repeated validations to avoid recompilation.
///
/// # Arguments
///
/// * `schema` - JSON Schema document
/// * `instance` - JSON document to validate
/// * `config` - Validation configuration
///
/// # Returns
///
/// * `Ok(ValidationResult)` - Validation completed
/// * `Err(SchemaError)` - Schema is invalid
///
/// # Examples
///
/// ```ignore
/// use hedl_json::validation::{validate_json, ValidationConfig};
/// use serde_json::json;
///
/// let schema = json!({"type": "string"});
/// let instance = json!("hello");
///
/// let result = validate_json(&schema, &instance, &ValidationConfig::default())?;
/// assert!(result.is_valid);
/// ```
pub fn validate_json(
    schema: &JsonValue,
    instance: &JsonValue,
    config: &ValidationConfig,
) -> Result<ValidationResult, SchemaError> {
    let compiled = CompiledSchema::compile(schema, config)?;
    Ok(compiled.validate(instance))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        assert!(matches!(config.draft, SchemaDraft::Draft7));
        assert!(config.collect_all_errors);
        assert!(config.max_errors.is_none());
        assert!(config.validate_formats);
    }

    #[test]
    fn test_compile_valid_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        });

        let result = CompiledSchema::compile(&schema, &ValidationConfig::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_invalid_schema() {
        let schema = json!({
            "type": "invalid_type_that_does_not_exist"
        });

        let result = CompiledSchema::compile(&schema, &ValidationConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_document() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"}
            },
            "required": ["name"]
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        let valid = json!({"name": "Alice", "age": 30});
        let result = compiled.validate(&valid);

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_invalid_document() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"}
            },
            "required": ["name"]
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        let invalid = json!({"age": "not an integer"});
        let result = compiled.validate(&invalid);

        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_type_mismatch() {
        let schema = json!({"type": "string"});
        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(compiled.validate(&json!("hello")).is_valid);
        assert!(!compiled.validate(&json!(42)).is_valid);
        assert!(!compiled.validate(&json!(true)).is_valid);
        assert!(!compiled.validate(&json!(null)).is_valid);
    }

    #[test]
    fn test_validate_string_constraints() {
        let schema = json!({
            "type": "string",
            "minLength": 2,
            "maxLength": 5
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(!compiled.validate(&json!("a")).is_valid); // too short
        assert!(compiled.validate(&json!("ab")).is_valid);
        assert!(compiled.validate(&json!("abcde")).is_valid);
        assert!(!compiled.validate(&json!("abcdef")).is_valid); // too long
    }

    #[test]
    fn test_validate_number_constraints() {
        let schema = json!({
            "type": "integer",
            "minimum": 0,
            "maximum": 100
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(!compiled.validate(&json!(-1)).is_valid); // below minimum
        assert!(compiled.validate(&json!(0)).is_valid);
        assert!(compiled.validate(&json!(50)).is_valid);
        assert!(compiled.validate(&json!(100)).is_valid);
        assert!(!compiled.validate(&json!(101)).is_valid); // above maximum
    }

    #[test]
    fn test_validate_array_constraints() {
        let schema = json!({
            "type": "array",
            "items": {"type": "integer"},
            "minItems": 1,
            "maxItems": 3
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(!compiled.validate(&json!([])).is_valid); // empty
        assert!(compiled.validate(&json!([1])).is_valid);
        assert!(compiled.validate(&json!([1, 2, 3])).is_valid);
        assert!(!compiled.validate(&json!([1, 2, 3, 4])).is_valid); // too many
        assert!(!compiled.validate(&json!([1, "string", 3])).is_valid); // wrong type
    }

    #[test]
    fn test_validate_required_properties() {
        let schema = json!({
            "type": "object",
            "properties": {
                "id": {"type": "integer"},
                "name": {"type": "string"}
            },
            "required": ["id", "name"]
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(
            compiled
                .validate(&json!({"id": 1, "name": "test"}))
                .is_valid
        );
        assert!(!compiled.validate(&json!({"id": 1})).is_valid); // missing name
        assert!(!compiled.validate(&json!({"name": "test"})).is_valid); // missing id
        assert!(!compiled.validate(&json!({})).is_valid); // missing both
    }

    #[test]
    fn test_validate_ref_resolution() {
        let schema = json!({
            "$defs": {
                "User": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"}
                    },
                    "required": ["name"]
                }
            },
            "type": "object",
            "properties": {
                "user": {"$ref": "#/$defs/User"}
            }
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(
            compiled
                .validate(&json!({"user": {"name": "Alice"}}))
                .is_valid
        );
        assert!(!compiled.validate(&json!({"user": {}})).is_valid); // missing name
    }

    #[test]
    fn test_validate_any_of() {
        let schema = json!({
            "anyOf": [
                {"type": "string"},
                {"type": "integer"}
            ]
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(compiled.validate(&json!("hello")).is_valid);
        assert!(compiled.validate(&json!(42)).is_valid);
        assert!(!compiled.validate(&json!(true)).is_valid);
    }

    #[test]
    fn test_validate_all_of() {
        let schema = json!({
            "allOf": [
                {"type": "object"},
                {"required": ["name"]},
                {"properties": {"name": {"type": "string"}}}
            ]
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(compiled.validate(&json!({"name": "test"})).is_valid);
        assert!(!compiled.validate(&json!({})).is_valid); // missing name
        assert!(!compiled.validate(&json!({"name": 123})).is_valid); // wrong type
    }

    #[test]
    fn test_collect_all_errors() {
        let schema = json!({
            "type": "object",
            "properties": {
                "a": {"type": "integer"},
                "b": {"type": "integer"},
                "c": {"type": "integer"}
            }
        });

        let config = ValidationConfig {
            collect_all_errors: true,
            ..Default::default()
        };

        let compiled = CompiledSchema::compile(&schema, &config).unwrap();

        let invalid = json!({
            "a": "not int",
            "b": "not int",
            "c": "not int"
        });

        let result = compiled.validate(&invalid);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 3); // All three errors collected
    }

    #[test]
    fn test_stop_at_first_error() {
        let schema = json!({
            "type": "object",
            "properties": {
                "a": {"type": "integer"},
                "b": {"type": "integer"},
                "c": {"type": "integer"}
            }
        });

        let config = ValidationConfig {
            collect_all_errors: false,
            ..Default::default()
        };

        let compiled = CompiledSchema::compile(&schema, &config).unwrap();

        let invalid = json!({
            "a": "not int",
            "b": "not int",
            "c": "not int"
        });

        let result = compiled.validate(&invalid);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1); // Only first error
    }

    #[test]
    fn test_max_errors_limit() {
        let schema = json!({
            "type": "object",
            "properties": {
                "a": {"type": "integer"},
                "b": {"type": "integer"},
                "c": {"type": "integer"},
                "d": {"type": "integer"},
                "e": {"type": "integer"}
            }
        });

        let config = ValidationConfig {
            collect_all_errors: true,
            max_errors: Some(2),
            ..Default::default()
        };

        let compiled = CompiledSchema::compile(&schema, &config).unwrap();

        let invalid = json!({
            "a": "x",
            "b": "x",
            "c": "x",
            "d": "x",
            "e": "x"
        });

        let result = compiled.validate(&invalid);
        assert!(!result.is_valid);
        assert!(result.errors.len() <= 2); // Limited to max_errors
    }

    #[test]
    fn test_is_valid_method() {
        let schema = json!({"type": "string"});
        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(compiled.is_valid(&json!("hello")));
        assert!(!compiled.is_valid(&json!(42)));
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError {
            instance_path: "/users/0/email".to_string(),
            message: "Invalid email format".to_string(),
            schema_path: "/properties/email/format".to_string(),
        };

        let display = error.to_string();
        assert!(display.contains("/users/0/email"));
        assert!(display.contains("Invalid email format"));
    }

    #[test]
    fn test_validate_json_convenience() {
        let schema = json!({"type": "string"});
        let instance = json!("hello");

        let result = validate_json(&schema, &instance, &ValidationConfig::default()).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_compiled_schema_clone() {
        let schema = json!({"type": "string"});
        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        let cloned = compiled.clone();
        assert!(cloned.is_valid(&json!("test")));
    }

    #[test]
    fn test_nested_object_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "user": {
                    "type": "object",
                    "properties": {
                        "profile": {
                            "type": "object",
                            "properties": {
                                "age": {"type": "integer", "minimum": 0}
                            }
                        }
                    }
                }
            }
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(
            compiled
                .validate(&json!({"user": {"profile": {"age": 25}}}))
                .is_valid
        );
        assert!(
            !compiled
                .validate(&json!({"user": {"profile": {"age": -5}}}))
                .is_valid
        );
    }

    #[test]
    fn test_pattern_validation() {
        let schema = json!({
            "type": "string",
            "pattern": "^[a-z]+$"
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(compiled.validate(&json!("abc")).is_valid);
        assert!(!compiled.validate(&json!("ABC")).is_valid);
        assert!(!compiled.validate(&json!("abc123")).is_valid);
    }

    #[test]
    fn test_enum_validation() {
        let schema = json!({
            "enum": ["red", "green", "blue"]
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(compiled.validate(&json!("red")).is_valid);
        assert!(compiled.validate(&json!("green")).is_valid);
        assert!(compiled.validate(&json!("blue")).is_valid);
        assert!(!compiled.validate(&json!("yellow")).is_valid);
    }

    #[test]
    fn test_const_validation() {
        let schema = json!({
            "const": "fixed_value"
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(compiled.validate(&json!("fixed_value")).is_valid);
        assert!(!compiled.validate(&json!("other")).is_valid);
    }

    #[test]
    fn test_additional_properties_false() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "additionalProperties": false
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        assert!(compiled.validate(&json!({"name": "test"})).is_valid);
        assert!(
            !compiled
                .validate(&json!({"name": "test", "extra": "field"}))
                .is_valid
        );
    }

    #[test]
    fn test_error_paths() {
        let schema = json!({
            "type": "object",
            "properties": {
                "users": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "age": {"type": "integer"}
                        }
                    }
                }
            }
        });

        let compiled = CompiledSchema::compile(&schema, &ValidationConfig::default()).unwrap();

        let invalid = json!({
            "users": [
                {"age": 25},
                {"age": "not an integer"}
            ]
        });

        let result = compiled.validate(&invalid);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        // Error path should point to the nested location
        assert!(result.errors[0].instance_path.contains("users"));
    }

    #[test]
    fn test_draft_versions() {
        let schema = json!({"type": "string"});

        // Test Draft 4
        let config_d4 = ValidationConfig {
            draft: SchemaDraft::Draft4,
            ..Default::default()
        };
        assert!(CompiledSchema::compile(&schema, &config_d4).is_ok());

        // Test Draft 6
        let config_d6 = ValidationConfig {
            draft: SchemaDraft::Draft6,
            ..Default::default()
        };
        assert!(CompiledSchema::compile(&schema, &config_d6).is_ok());

        // Test Draft 7
        let config_d7 = ValidationConfig {
            draft: SchemaDraft::Draft7,
            ..Default::default()
        };
        assert!(CompiledSchema::compile(&schema, &config_d7).is_ok());

        // Test Draft 2019-09
        let config_d201909 = ValidationConfig {
            draft: SchemaDraft::Draft201909,
            ..Default::default()
        };
        assert!(CompiledSchema::compile(&schema, &config_d201909).is_ok());

        // Test Draft 2020-12
        let config_d202012 = ValidationConfig {
            draft: SchemaDraft::Draft202012,
            ..Default::default()
        };
        assert!(CompiledSchema::compile(&schema, &config_d202012).is_ok());
    }
}

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

//! Value coercion for HEDL type system.
//!
//! This module provides type coercion functionality that converts values between
//! compatible types. Coercion is used during validation and type checking to allow
//! flexible data handling while maintaining type safety.
//!
//! # Coercion Rules
//!
//! The coercion system supports two modes:
//!
//! - **Strict mode**: Only safe, lossless coercions (e.g., Int → Float)
//! - **Lenient mode**: Allows parsing from strings (e.g., "42" → Int)
//!
//! # Coercion Hierarchy
//!
//! ```text
//! String (lenient only)
//!    ↓
//! Int → Float (always safe)
//!    ↓
//! Bool ← String (lenient only, "true"/"false")
//! ```

use crate::types::ExpectedType;
use crate::value::Value;

/// Configuration for type coercion behavior.
///
/// This structure provides fine-grained control over how values are coerced
/// to expected types during parsing and validation.
///
/// # Examples
///
/// ```
/// use hedl_core::coercion::{CoercionConfig, CoercionLevel};
///
/// // Create a strict config (default)
/// let strict = CoercionConfig::default();
/// assert_eq!(strict.level, CoercionLevel::Strict);
///
/// // Create a permissive config
/// let permissive = CoercionConfig {
///     level: CoercionLevel::Permissive,
///     allow_lossy_float_to_int: true,
///     null_as_default: true,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct CoercionConfig {
    /// Strictness level for coercion.
    pub level: CoercionLevel,

    /// Allow string to number coercion (e.g., "42" → 42).
    ///
    /// Only applies when level is Standard or Permissive.
    pub allow_string_to_number: bool,

    /// Allow lossy float to int coercion (truncates decimal part).
    ///
    /// Only applies when level is Permissive.
    pub allow_lossy_float_to_int: bool,

    /// Custom true values for boolean coercion.
    ///
    /// Case-sensitive strings that should be treated as `true`.
    /// Default: ["true", "yes", "1"]
    pub bool_true_values: Vec<String>,

    /// Custom false values for boolean coercion.
    ///
    /// Case-sensitive strings that should be treated as `false`.
    /// Default: ["false", "no", "0"]
    pub bool_false_values: Vec<String>,

    /// Treat null as default value for the expected type.
    ///
    /// When true, null values are converted to type-specific defaults:
    /// - Int → 0
    /// - Float → 0.0
    /// - Bool → false
    /// - String → ""
    ///
    /// Only applies when level is Permissive.
    pub null_as_default: bool,
}

impl Default for CoercionConfig {
    fn default() -> Self {
        Self {
            level: CoercionLevel::Strict,
            allow_string_to_number: true,
            allow_lossy_float_to_int: false,
            bool_true_values: vec!["true".into(), "yes".into(), "1".into()],
            bool_false_values: vec!["false".into(), "no".into(), "0".into()],
            null_as_default: false,
        }
    }
}

impl CoercionConfig {
    /// Create a strict configuration (no coercion).
    pub fn none() -> Self {
        Self {
            level: CoercionLevel::None,
            ..Default::default()
        }
    }

    /// Create a strict configuration (only safe coercions).
    pub fn strict() -> Self {
        Self {
            level: CoercionLevel::Strict,
            ..Default::default()
        }
    }

    /// Create a standard configuration (allows string parsing).
    pub fn standard() -> Self {
        Self {
            level: CoercionLevel::Standard,
            allow_string_to_number: true,
            allow_lossy_float_to_int: false,
            // Only accept "true" and "false" for boolean coercion (not "yes", "no", etc.)
            bool_true_values: vec!["true".into()],
            bool_false_values: vec!["false".into()],
            null_as_default: false,
        }
    }

    /// Create a permissive configuration (allows all coercions).
    pub fn permissive() -> Self {
        Self {
            level: CoercionLevel::Permissive,
            allow_string_to_number: true,
            allow_lossy_float_to_int: true,
            null_as_default: true,
            ..Default::default()
        }
    }
}

/// Coercion mode controlling how strict type conversions are.
///
/// DEPRECATED: Use `CoercionLevel` instead. This enum is kept for backwards compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CoercionMode {
    /// Strict mode: Only safe, lossless coercions allowed.
    ///
    /// Safe coercions:
    /// - Int → Float (no precision loss for reasonable values)
    #[default]
    Strict,

    /// Lenient mode: Allows parsing from strings.
    ///
    /// Additional coercions:
    /// - String → Int (if string is valid integer)
    /// - String → Float (if string is valid float)
    /// - String → Bool (only "true" or "false")
    Lenient,
}

/// Coercion strictness level.
///
/// Defines four levels of type coercion strictness, from no coercion at all
/// to permissive coercions that may lose data or precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoercionLevel {
    /// No coercion allowed - types must match exactly.
    ///
    /// Only accepts values that exactly match the expected type.
    /// No conversions or parsing is performed.
    None,

    /// Only safe coercions (no data loss possible).
    ///
    /// Safe coercions:
    /// - Int → Float (no precision loss for reasonable values)
    #[default]
    Strict,

    /// Allow potentially lossy coercions with warnings.
    ///
    /// Standard coercions (in addition to Strict):
    /// - String → Int (if string is valid integer)
    /// - String → Float (if string is valid float)
    /// - String → Bool (if string matches configured values)
    /// - Most types → String
    Standard,

    /// Allow all coercions including potentially dangerous ones.
    ///
    /// Permissive coercions (in addition to Standard):
    /// - Float → Int (lossy, truncates decimal part)
    /// - Null → default value (if configured)
    Permissive,
}

/// Result of a coercion attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum CoercionResult {
    /// Value was already the expected type (no coercion needed)
    Matched(Value),
    /// Value was successfully coerced to the expected type
    Coerced(Value),
    /// Coercion is not possible
    Failed {
        /// The original value
        value: Value,
        /// The expected type
        expected: ExpectedType,
        /// Reason for failure
        reason: String,
    },
}

impl CoercionResult {
    /// Returns true if coercion succeeded (matched or coerced).
    pub fn is_ok(&self) -> bool {
        matches!(
            self,
            CoercionResult::Matched(_) | CoercionResult::Coerced(_)
        )
    }

    /// Returns true if coercion failed.
    pub fn is_err(&self) -> bool {
        matches!(self, CoercionResult::Failed { .. })
    }

    /// Get the resulting value if coercion succeeded.
    pub fn value(self) -> Option<Value> {
        match self {
            CoercionResult::Matched(v) | CoercionResult::Coerced(v) => Some(v),
            CoercionResult::Failed { .. } => None,
        }
    }

    /// Get the resulting value reference if coercion succeeded.
    pub fn value_ref(&self) -> Option<&Value> {
        match self {
            CoercionResult::Matched(v) | CoercionResult::Coerced(v) => Some(v),
            CoercionResult::Failed { .. } => None,
        }
    }
}

/// Attempt to coerce a value to match an expected type using configuration.
///
/// This is the main coercion function that supports all coercion levels and
/// configuration options.
///
/// # Arguments
///
/// * `value` - The value to coerce
/// * `expected` - The expected type
/// * `config` - Coercion configuration
///
/// # Returns
///
/// A `CoercionResult` indicating success or failure.
///
/// # Examples
///
/// ```
/// use hedl_core::coercion::{coerce_with_config, CoercionConfig, CoercionResult};
/// use hedl_core::types::ExpectedType;
/// use hedl_core::Value;
///
/// // Int to Float coercion (safe)
/// let result = coerce_with_config(
///     Value::Int(42),
///     &ExpectedType::Float,
///     &CoercionConfig::strict()
/// );
/// assert!(result.is_ok());
///
/// // String to Int with standard config
/// let result = coerce_with_config(
///     Value::String("42".to_string().into()),
///     &ExpectedType::Int,
///     &CoercionConfig::standard()
/// );
/// assert!(result.is_ok());
/// ```
pub fn coerce_with_config(
    value: Value,
    expected: &ExpectedType,
    config: &CoercionConfig,
) -> CoercionResult {
    // Check if value already matches
    if expected.matches(&value) {
        return CoercionResult::Matched(value);
    }

    // Level None: No coercion allowed
    if config.level == CoercionLevel::None {
        return CoercionResult::Failed {
            expected: expected.clone(),
            reason: format!(
                "cannot coerce {} to {} (coercion disabled)",
                describe_value_type(&value),
                expected.describe()
            ),
            value,
        };
    }

    // Attempt coercion based on expected type
    match expected {
        ExpectedType::Any => CoercionResult::Matched(value),

        ExpectedType::Float => coerce_to_float_with_config(value, config),

        ExpectedType::Int => coerce_to_int_with_config(value, config),

        ExpectedType::Bool => coerce_to_bool_with_config(value, config),

        ExpectedType::String => coerce_to_string_with_config(value, config),

        ExpectedType::Numeric => coerce_to_numeric_with_config(value, config),

        ExpectedType::Union(types) => coerce_to_union_with_config(value, types, config),

        // Null with null_as_default
        ExpectedType::Null => {
            if matches!(value, Value::Null) {
                CoercionResult::Matched(value)
            } else {
                CoercionResult::Failed {
                    expected: expected.clone(),
                    reason: format!("cannot coerce {} to Null", describe_value_type(&value)),
                    value,
                }
            }
        }

        // List coercion: coerce each element
        ExpectedType::List(element_type) => {
            if let Value::List(items) = &value {
                let mut coerced_items = Vec::with_capacity(items.len());
                for item in items.iter() {
                    match coerce_with_config(item.clone(), element_type, config) {
                        CoercionResult::Matched(v) | CoercionResult::Coerced(v) => {
                            coerced_items.push(v);
                        }
                        CoercionResult::Failed { reason, .. } => {
                            return CoercionResult::Failed {
                                expected: expected.clone(),
                                reason: format!("list element coercion failed: {}", reason),
                                value,
                            };
                        }
                    }
                }
                CoercionResult::Coerced(Value::List(Box::new(coerced_items)))
            } else {
                CoercionResult::Failed {
                    expected: expected.clone(),
                    reason: format!("cannot coerce {} to List", describe_value_type(&value)),
                    value,
                }
            }
        }

        // Types that don't support coercion
        ExpectedType::Tensor { .. } | ExpectedType::Reference { .. } | ExpectedType::Expression => {
            CoercionResult::Failed {
                expected: expected.clone(),
                reason: format!(
                    "cannot coerce {} to {}",
                    describe_value_type(&value),
                    expected.describe()
                ),
                value,
            }
        }
    }
}

/// Attempt to coerce a value to match an expected type.
///
/// # Arguments
///
/// * `value` - The value to coerce
/// * `expected` - The expected type
/// * `mode` - Coercion mode (strict or lenient)
///
/// # Returns
///
/// A `CoercionResult` indicating success or failure.
///
/// # Examples
///
/// ```
/// use hedl_core::coercion::{coerce, CoercionMode, CoercionResult};
/// use hedl_core::types::ExpectedType;
/// use hedl_core::Value;
///
/// // Int to Float coercion (always safe)
/// let result = coerce(Value::Int(42), &ExpectedType::Float, CoercionMode::Strict);
/// assert!(result.is_ok());
///
/// // String to Int (lenient mode only)
/// let result = coerce(
///     Value::String("42".to_string().into()),
///     &ExpectedType::Int,
///     CoercionMode::Lenient
/// );
/// assert!(result.is_ok());
/// ```
pub fn coerce(value: Value, expected: &ExpectedType, mode: CoercionMode) -> CoercionResult {
    // Convert CoercionMode to CoercionConfig
    let config = match mode {
        CoercionMode::Strict => CoercionConfig::strict(),
        CoercionMode::Lenient => CoercionConfig::standard(),
    };

    coerce_with_config(value, expected, &config)
}

/// Coerce a value to Float using configuration.
fn coerce_to_float_with_config(value: Value, config: &CoercionConfig) -> CoercionResult {
    match value {
        // Null to default (if permissive)
        Value::Null if config.null_as_default && config.level == CoercionLevel::Permissive => {
            CoercionResult::Coerced(Value::Float(0.0))
        }

        // Int to Float (safe, always allowed except in None mode)
        Value::Int(i) if config.level != CoercionLevel::None => {
            CoercionResult::Coerced(Value::Float(i as f64))
        }

        // String to Float (standard and permissive)
        Value::String(ref s)
            if config.allow_string_to_number
                && matches!(
                    config.level,
                    CoercionLevel::Standard | CoercionLevel::Permissive
                ) =>
        {
            match s.trim().parse::<f64>() {
                Ok(f) if f.is_finite() => CoercionResult::Coerced(Value::Float(f)),
                _ => CoercionResult::Failed {
                    expected: ExpectedType::Float,
                    reason: format!("cannot parse '{}' as float", s),
                    value,
                },
            }
        }

        _ => CoercionResult::Failed {
            expected: ExpectedType::Float,
            reason: format!("cannot coerce {} to Float", describe_value_type(&value)),
            value,
        },
    }
}

/// Coerce a value to Int using configuration.
fn coerce_to_int_with_config(value: Value, config: &CoercionConfig) -> CoercionResult {
    match value {
        // Null to default (if permissive)
        Value::Null if config.null_as_default && config.level == CoercionLevel::Permissive => {
            CoercionResult::Coerced(Value::Int(0))
        }

        // Float to Int (lossy, only in permissive mode)
        Value::Float(f)
            if config.allow_lossy_float_to_int && config.level == CoercionLevel::Permissive =>
        {
            if f.is_finite() {
                CoercionResult::Coerced(Value::Int(f.trunc() as i64))
            } else {
                CoercionResult::Failed {
                    expected: ExpectedType::Int,
                    reason: format!("cannot convert non-finite float {} to Int", f),
                    value,
                }
            }
        }

        // String to Int (standard and permissive)
        Value::String(ref s)
            if config.allow_string_to_number
                && matches!(
                    config.level,
                    CoercionLevel::Standard | CoercionLevel::Permissive
                ) =>
        {
            match s.trim().parse::<i64>() {
                Ok(i) => CoercionResult::Coerced(Value::Int(i)),
                Err(_) => CoercionResult::Failed {
                    expected: ExpectedType::Int,
                    reason: format!("cannot parse '{}' as integer", s),
                    value,
                },
            }
        }

        _ => CoercionResult::Failed {
            expected: ExpectedType::Int,
            reason: format!("cannot coerce {} to Int", describe_value_type(&value)),
            value,
        },
    }
}

/// Coerce a value to Bool using configuration.
fn coerce_to_bool_with_config(value: Value, config: &CoercionConfig) -> CoercionResult {
    match value {
        // Null to default (if permissive)
        Value::Null if config.null_as_default && config.level == CoercionLevel::Permissive => {
            CoercionResult::Coerced(Value::Bool(false))
        }

        // String to Bool (standard and permissive)
        Value::String(ref s)
            if matches!(
                config.level,
                CoercionLevel::Standard | CoercionLevel::Permissive
            ) =>
        {
            let trimmed = s.trim();

            // Check true values
            if config.bool_true_values.iter().any(|v| v == trimmed) {
                return CoercionResult::Coerced(Value::Bool(true));
            }

            // Check false values
            if config.bool_false_values.iter().any(|v| v == trimmed) {
                return CoercionResult::Coerced(Value::Bool(false));
            }

            CoercionResult::Failed {
                expected: ExpectedType::Bool,
                reason: format!(
                    "cannot parse '{}' as boolean (expected one of: {}, {})",
                    s,
                    config.bool_true_values.join(", "),
                    config.bool_false_values.join(", ")
                ),
                value,
            }
        }

        _ => CoercionResult::Failed {
            expected: ExpectedType::Bool,
            reason: format!("cannot coerce {} to Bool", describe_value_type(&value)),
            value,
        },
    }
}

/// Coerce a value to String using configuration.
fn coerce_to_string_with_config(value: Value, config: &CoercionConfig) -> CoercionResult {
    // Null to default (if permissive)
    if matches!(value, Value::Null)
        && config.null_as_default
        && config.level == CoercionLevel::Permissive
    {
        return CoercionResult::Coerced(Value::String("".into()));
    }

    // Standard and permissive modes allow conversion to string
    if matches!(
        config.level,
        CoercionLevel::Standard | CoercionLevel::Permissive
    ) {
        let s: Box<str> = match &value {
            Value::Null => "~".into(),
            Value::Bool(b) => b.to_string().into_boxed_str(),
            Value::Int(i) => i.to_string().into_boxed_str(),
            Value::Float(f) => f.to_string().into_boxed_str(),
            Value::String(s) => return CoercionResult::Matched(Value::String(s.clone())),
            Value::Reference(r) => r.to_ref_string().into_boxed_str(),
            Value::Expression(_) | Value::Tensor(_) | Value::List(_) => {
                return CoercionResult::Failed {
                    expected: ExpectedType::String,
                    reason: format!("cannot coerce {} to String", describe_value_type(&value)),
                    value,
                }
            }
        };
        CoercionResult::Coerced(Value::String(s))
    } else {
        CoercionResult::Failed {
            expected: ExpectedType::String,
            reason: format!(
                "cannot coerce {} to String (coercion level too strict)",
                describe_value_type(&value)
            ),
            value,
        }
    }
}

/// Coerce a value to Numeric (Int or Float) using configuration.
fn coerce_to_numeric_with_config(value: Value, config: &CoercionConfig) -> CoercionResult {
    match &value {
        // Null to default (if permissive)
        Value::Null if config.null_as_default && config.level == CoercionLevel::Permissive => {
            CoercionResult::Coerced(Value::Int(0))
        }

        // Already numeric
        Value::Int(_) | Value::Float(_) => CoercionResult::Matched(value),

        // String to numeric (standard and permissive)
        Value::String(s)
            if config.allow_string_to_number
                && matches!(
                    config.level,
                    CoercionLevel::Standard | CoercionLevel::Permissive
                ) =>
        {
            let trimmed = s.trim();
            // Try int first, then float
            if let Ok(i) = trimmed.parse::<i64>() {
                CoercionResult::Coerced(Value::Int(i))
            } else if let Ok(f) = trimmed.parse::<f64>() {
                if f.is_finite() {
                    CoercionResult::Coerced(Value::Float(f))
                } else {
                    CoercionResult::Failed {
                        expected: ExpectedType::Numeric,
                        reason: format!("'{}' is not a finite number", s),
                        value,
                    }
                }
            } else {
                CoercionResult::Failed {
                    expected: ExpectedType::Numeric,
                    reason: format!("cannot parse '{}' as number", s),
                    value,
                }
            }
        }

        _ => CoercionResult::Failed {
            expected: ExpectedType::Numeric,
            reason: format!("cannot coerce {} to Numeric", describe_value_type(&value)),
            value,
        },
    }
}

/// Coerce a value to one of the union types using configuration.
fn coerce_to_union_with_config(
    value: Value,
    types: &[ExpectedType],
    config: &CoercionConfig,
) -> CoercionResult {
    // Try each type in order
    for expected in types {
        // Check if value matches without coercion
        if expected.matches(&value) {
            return CoercionResult::Matched(value);
        }
    }

    // Try coercion for each type
    for expected in types {
        match coerce_with_config(value.clone(), expected, config) {
            result @ CoercionResult::Coerced(_) => return result,
            _ => continue,
        }
    }

    // No match found
    let type_names: Vec<String> = types.iter().map(|t| t.describe()).collect();
    CoercionResult::Failed {
        expected: ExpectedType::Union(types.to_vec()),
        reason: format!(
            "cannot coerce {} to any of: {}",
            describe_value_type(&value),
            type_names.join(", ")
        ),
        value,
    }
}

/// Get a type description from a Value (for error messages).
fn describe_value_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "Null",
        Value::Bool(_) => "Bool",
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::String(_) => "String",
        Value::Reference(_) => "Reference",
        Value::Expression(_) => "Expression",
        Value::Tensor(_) => "Tensor",
        Value::List(_) => "List",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== CoercionMode tests ====================

    #[test]
    fn test_coercion_mode_default() {
        assert_eq!(CoercionMode::default(), CoercionMode::Strict);
    }

    // ==================== CoercionResult tests ====================

    #[test]
    fn test_result_matched_is_ok() {
        let result = CoercionResult::Matched(Value::Int(42));
        assert!(result.is_ok());
        assert!(!result.is_err());
    }

    #[test]
    fn test_result_coerced_is_ok() {
        let result = CoercionResult::Coerced(Value::Float(42.0));
        assert!(result.is_ok());
        assert!(!result.is_err());
    }

    #[test]
    fn test_result_failed_is_err() {
        let result = CoercionResult::Failed {
            value: Value::String("test".to_string().into()),
            expected: ExpectedType::Int,
            reason: "test".to_string(),
        };
        assert!(result.is_err());
        assert!(!result.is_ok());
    }

    #[test]
    fn test_result_value() {
        let result = CoercionResult::Coerced(Value::Int(42));
        assert_eq!(result.value(), Some(Value::Int(42)));

        let result = CoercionResult::Failed {
            value: Value::String("test".to_string().into()),
            expected: ExpectedType::Int,
            reason: "test".to_string(),
        };
        assert!(result.value().is_none());
    }

    #[test]
    fn test_result_value_ref() {
        let result = CoercionResult::Matched(Value::Int(42));
        assert_eq!(result.value_ref(), Some(&Value::Int(42)));
    }

    // ==================== Int to Float coercion ====================

    #[test]
    fn test_int_to_float_strict() {
        let result = coerce(Value::Int(42), &ExpectedType::Float, CoercionMode::Strict);
        assert!(
            matches!(result, CoercionResult::Coerced(Value::Float(f)) if (f - 42.0).abs() < 0.001)
        );
    }

    #[test]
    fn test_int_to_float_negative() {
        let result = coerce(Value::Int(-100), &ExpectedType::Float, CoercionMode::Strict);
        assert!(
            matches!(result, CoercionResult::Coerced(Value::Float(f)) if (f + 100.0).abs() < 0.001)
        );
    }

    // ==================== String to Int coercion ====================

    #[test]
    fn test_string_to_int_lenient() {
        let result = coerce(
            Value::String("42".to_string().into()),
            &ExpectedType::Int,
            CoercionMode::Lenient,
        );
        assert!(matches!(result, CoercionResult::Coerced(Value::Int(42))));
    }

    #[test]
    fn test_string_to_int_with_whitespace() {
        let result = coerce(
            Value::String("  42  ".to_string().into()),
            &ExpectedType::Int,
            CoercionMode::Lenient,
        );
        assert!(matches!(result, CoercionResult::Coerced(Value::Int(42))));
    }

    #[test]
    fn test_string_to_int_negative() {
        let result = coerce(
            Value::String("-100".to_string().into()),
            &ExpectedType::Int,
            CoercionMode::Lenient,
        );
        assert!(matches!(result, CoercionResult::Coerced(Value::Int(-100))));
    }

    #[test]
    fn test_string_to_int_strict_fails() {
        let result = coerce(
            Value::String("42".to_string().into()),
            &ExpectedType::Int,
            CoercionMode::Strict,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_string_to_int_invalid() {
        let result = coerce(
            Value::String("not_a_number".to_string().into()),
            &ExpectedType::Int,
            CoercionMode::Lenient,
        );
        assert!(result.is_err());
    }

    // ==================== String to Float coercion ====================

    #[test]
    fn test_string_to_float_lenient() {
        let result = coerce(
            Value::String("3.25".to_string().into()),
            &ExpectedType::Float,
            CoercionMode::Lenient,
        );
        assert!(
            matches!(result, CoercionResult::Coerced(Value::Float(f)) if (f - 3.25).abs() < 0.001)
        );
    }

    #[test]
    fn test_string_to_float_integer_string() {
        let result = coerce(
            Value::String("42".to_string().into()),
            &ExpectedType::Float,
            CoercionMode::Lenient,
        );
        assert!(
            matches!(result, CoercionResult::Coerced(Value::Float(f)) if (f - 42.0).abs() < 0.001)
        );
    }

    #[test]
    fn test_string_to_float_strict_fails() {
        let result = coerce(
            Value::String("3.25".to_string().into()),
            &ExpectedType::Float,
            CoercionMode::Strict,
        );
        assert!(result.is_err());
    }

    // ==================== String to Bool coercion ====================

    #[test]
    fn test_string_to_bool_true() {
        let result = coerce(
            Value::String("true".to_string().into()),
            &ExpectedType::Bool,
            CoercionMode::Lenient,
        );
        assert!(matches!(result, CoercionResult::Coerced(Value::Bool(true))));
    }

    #[test]
    fn test_string_to_bool_false() {
        let result = coerce(
            Value::String("false".to_string().into()),
            &ExpectedType::Bool,
            CoercionMode::Lenient,
        );
        assert!(matches!(
            result,
            CoercionResult::Coerced(Value::Bool(false))
        ));
    }

    #[test]
    fn test_string_to_bool_with_whitespace() {
        let result = coerce(
            Value::String("  true  ".to_string().into()),
            &ExpectedType::Bool,
            CoercionMode::Lenient,
        );
        assert!(matches!(result, CoercionResult::Coerced(Value::Bool(true))));
    }

    #[test]
    fn test_string_to_bool_invalid() {
        let result = coerce(
            Value::String("maybe".to_string().into()),
            &ExpectedType::Bool,
            CoercionMode::Lenient,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_string_to_bool_strict_fails() {
        let result = coerce(
            Value::String("true".to_string().into()),
            &ExpectedType::Bool,
            CoercionMode::Strict,
        );
        assert!(result.is_err());
    }

    // ==================== Any type coercion ====================

    #[test]
    fn test_any_matches_everything() {
        let result = coerce(Value::Int(42), &ExpectedType::Any, CoercionMode::Strict);
        assert!(matches!(result, CoercionResult::Matched(Value::Int(42))));

        let result = coerce(
            Value::String("test".to_string().into()),
            &ExpectedType::Any,
            CoercionMode::Strict,
        );
        assert!(result.is_ok());
    }

    // ==================== Numeric coercion ====================

    #[test]
    fn test_numeric_int_matches() {
        let result = coerce(Value::Int(42), &ExpectedType::Numeric, CoercionMode::Strict);
        assert!(matches!(result, CoercionResult::Matched(Value::Int(42))));
    }

    #[test]
    fn test_numeric_float_matches() {
        let result = coerce(
            Value::Float(3.25),
            &ExpectedType::Numeric,
            CoercionMode::Strict,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_to_numeric_lenient() {
        // Integer string
        let result = coerce(
            Value::String("42".to_string().into()),
            &ExpectedType::Numeric,
            CoercionMode::Lenient,
        );
        assert!(matches!(result, CoercionResult::Coerced(Value::Int(42))));

        // Float string
        let result = coerce(
            Value::String("3.25".to_string().into()),
            &ExpectedType::Numeric,
            CoercionMode::Lenient,
        );
        assert!(
            matches!(result, CoercionResult::Coerced(Value::Float(f)) if (f - 3.25).abs() < 0.001)
        );
    }

    // ==================== Union coercion ====================

    #[test]
    fn test_union_exact_match() {
        let union = ExpectedType::Union(vec![ExpectedType::Int, ExpectedType::String]);
        let result = coerce(Value::Int(42), &union, CoercionMode::Strict);
        assert!(matches!(result, CoercionResult::Matched(Value::Int(42))));
    }

    #[test]
    fn test_union_coercion() {
        let union = ExpectedType::Union(vec![ExpectedType::Float, ExpectedType::String]);
        // Int should coerce to Float
        let result = coerce(Value::Int(42), &union, CoercionMode::Strict);
        assert!(matches!(result, CoercionResult::Coerced(Value::Float(_))));
    }

    #[test]
    fn test_union_no_match() {
        let union = ExpectedType::Union(vec![ExpectedType::Int, ExpectedType::Bool]);
        let result = coerce(
            Value::String("test".to_string().into()),
            &union,
            CoercionMode::Strict,
        );
        assert!(result.is_err());
    }

    // ==================== Non-coercible types ====================

    #[test]
    fn test_null_no_coercion() {
        let result = coerce(Value::Int(0), &ExpectedType::Null, CoercionMode::Lenient);
        assert!(result.is_err());
    }

    #[test]
    fn test_expression_no_coercion() {
        let result = coerce(
            Value::String("$(now())".to_string().into()),
            &ExpectedType::Expression,
            CoercionMode::Lenient,
        );
        assert!(result.is_err());
    }

    // ==================== String coercion (lenient) ====================

    #[test]
    fn test_int_to_string_lenient() {
        let result = coerce(Value::Int(42), &ExpectedType::String, CoercionMode::Lenient);
        assert!(matches!(result, CoercionResult::Coerced(Value::String(s)) if s.as_ref() == "42"));
    }

    #[test]
    fn test_bool_to_string_lenient() {
        let result = coerce(
            Value::Bool(true),
            &ExpectedType::String,
            CoercionMode::Lenient,
        );
        assert!(
            matches!(result, CoercionResult::Coerced(Value::String(s)) if s.as_ref() == "true")
        );
    }

    #[test]
    fn test_to_string_strict_fails() {
        let result = coerce(Value::Int(42), &ExpectedType::String, CoercionMode::Strict);
        assert!(result.is_err());
    }

    // ==================== Already matched ====================

    #[test]
    fn test_int_matches_int() {
        let result = coerce(Value::Int(42), &ExpectedType::Int, CoercionMode::Strict);
        assert!(matches!(result, CoercionResult::Matched(Value::Int(42))));
    }

    #[test]
    fn test_string_matches_string() {
        let result = coerce(
            Value::String("test".to_string().into()),
            &ExpectedType::String,
            CoercionMode::Strict,
        );
        assert!(result.is_ok());
    }

    // ==================== CoercionLevel tests ====================

    #[test]
    fn test_coercion_level_default() {
        assert_eq!(CoercionLevel::default(), CoercionLevel::Strict);
    }

    // ==================== CoercionConfig tests ====================

    #[test]
    fn test_coercion_config_default() {
        let config = CoercionConfig::default();
        assert_eq!(config.level, CoercionLevel::Strict);
        assert!(config.allow_string_to_number);
        assert!(!config.allow_lossy_float_to_int);
        assert!(!config.null_as_default);
        assert_eq!(config.bool_true_values, vec!["true", "yes", "1"]);
        assert_eq!(config.bool_false_values, vec!["false", "no", "0"]);
    }

    #[test]
    fn test_coercion_config_none() {
        let config = CoercionConfig::none();
        assert_eq!(config.level, CoercionLevel::None);
    }

    #[test]
    fn test_coercion_config_strict() {
        let config = CoercionConfig::strict();
        assert_eq!(config.level, CoercionLevel::Strict);
    }

    #[test]
    fn test_coercion_config_standard() {
        let config = CoercionConfig::standard();
        assert_eq!(config.level, CoercionLevel::Standard);
        assert!(config.allow_string_to_number);
    }

    #[test]
    fn test_coercion_config_permissive() {
        let config = CoercionConfig::permissive();
        assert_eq!(config.level, CoercionLevel::Permissive);
        assert!(config.allow_string_to_number);
        assert!(config.allow_lossy_float_to_int);
        assert!(config.null_as_default);
    }

    // ==================== CoercionLevel::None tests ====================

    #[test]
    fn test_none_level_no_coercion() {
        let config = CoercionConfig::none();

        // Int to Float is not allowed in None mode
        let result = coerce_with_config(Value::Int(42), &ExpectedType::Float, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_none_level_exact_match_ok() {
        let config = CoercionConfig::none();

        // Exact match is still OK
        let result = coerce_with_config(Value::Int(42), &ExpectedType::Int, &config);
        assert!(matches!(result, CoercionResult::Matched(_)));
    }

    // ==================== CoercionLevel::Strict tests ====================

    #[test]
    fn test_strict_int_to_float() {
        let config = CoercionConfig::strict();
        let result = coerce_with_config(Value::Int(42), &ExpectedType::Float, &config);
        assert!(
            matches!(result, CoercionResult::Coerced(Value::Float(f)) if (f - 42.0).abs() < 0.001)
        );
    }

    #[test]
    fn test_strict_no_string_parsing() {
        let config = CoercionConfig::strict();
        let result = coerce_with_config(
            Value::String("42".to_string().into()),
            &ExpectedType::Int,
            &config,
        );
        assert!(result.is_err());
    }

    // ==================== CoercionLevel::Standard tests ====================

    #[test]
    fn test_standard_string_to_int() {
        let config = CoercionConfig::standard();
        let result = coerce_with_config(
            Value::String("42".to_string().into()),
            &ExpectedType::Int,
            &config,
        );
        assert!(matches!(result, CoercionResult::Coerced(Value::Int(42))));
    }

    #[test]
    fn test_standard_string_to_float() {
        let config = CoercionConfig::standard();
        let result = coerce_with_config(
            Value::String("2.5".to_string().into()),
            &ExpectedType::Float,
            &config,
        );
        assert!(
            matches!(result, CoercionResult::Coerced(Value::Float(f)) if (f - 2.5).abs() < 0.001)
        );
    }

    #[test]
    fn test_standard_string_to_bool() {
        let config = CoercionConfig::standard();
        let result = coerce_with_config(
            Value::String("true".to_string().into()),
            &ExpectedType::Bool,
            &config,
        );
        assert!(matches!(result, CoercionResult::Coerced(Value::Bool(true))));
    }

    #[test]
    fn test_standard_to_string() {
        let config = CoercionConfig::standard();
        let result = coerce_with_config(Value::Int(42), &ExpectedType::String, &config);
        assert!(matches!(result, CoercionResult::Coerced(Value::String(s)) if s.as_ref() == "42"));
    }

    #[test]
    fn test_standard_no_float_to_int() {
        let config = CoercionConfig::standard();
        let result = coerce_with_config(Value::Float(2.5), &ExpectedType::Int, &config);
        assert!(result.is_err());
    }

    // ==================== CoercionLevel::Permissive tests ====================

    #[test]
    fn test_permissive_float_to_int() {
        let config = CoercionConfig::permissive();
        let result = coerce_with_config(Value::Float(2.5), &ExpectedType::Int, &config);
        assert!(matches!(result, CoercionResult::Coerced(Value::Int(2))));
    }

    #[test]
    fn test_permissive_float_to_int_negative() {
        let config = CoercionConfig::permissive();
        let result = coerce_with_config(Value::Float(-3.9), &ExpectedType::Int, &config);
        assert!(matches!(result, CoercionResult::Coerced(Value::Int(-3))));
    }

    #[test]
    fn test_permissive_null_to_int() {
        let config = CoercionConfig::permissive();
        let result = coerce_with_config(Value::Null, &ExpectedType::Int, &config);
        assert!(matches!(result, CoercionResult::Coerced(Value::Int(0))));
    }

    #[test]
    fn test_permissive_null_to_float() {
        let config = CoercionConfig::permissive();
        let result = coerce_with_config(Value::Null, &ExpectedType::Float, &config);
        assert!(matches!(result, CoercionResult::Coerced(Value::Float(f)) if f.abs() < 0.001));
    }

    #[test]
    fn test_permissive_null_to_bool() {
        let config = CoercionConfig::permissive();
        let result = coerce_with_config(Value::Null, &ExpectedType::Bool, &config);
        assert!(matches!(
            result,
            CoercionResult::Coerced(Value::Bool(false))
        ));
    }

    #[test]
    fn test_permissive_null_to_string() {
        let config = CoercionConfig::permissive();
        let result = coerce_with_config(Value::Null, &ExpectedType::String, &config);
        assert!(matches!(result, CoercionResult::Coerced(Value::String(s)) if s.as_ref() == ""));
    }

    #[test]
    fn test_permissive_infinity_to_int_fails() {
        let config = CoercionConfig::permissive();
        let result = coerce_with_config(Value::Float(f64::INFINITY), &ExpectedType::Int, &config);
        assert!(result.is_err());
    }

    // ==================== Custom bool values tests ====================

    #[test]
    fn test_custom_bool_true_values() {
        let config = CoercionConfig {
            level: CoercionLevel::Standard,
            bool_true_values: vec!["yes".into(), "on".into(), "enabled".into()],
            bool_false_values: vec!["no".into(), "off".into(), "disabled".into()],
            ..Default::default()
        };

        let result = coerce_with_config(
            Value::String("yes".to_string().into()),
            &ExpectedType::Bool,
            &config,
        );
        assert!(matches!(result, CoercionResult::Coerced(Value::Bool(true))));

        let result = coerce_with_config(
            Value::String("on".to_string().into()),
            &ExpectedType::Bool,
            &config,
        );
        assert!(matches!(result, CoercionResult::Coerced(Value::Bool(true))));

        let result = coerce_with_config(
            Value::String("enabled".to_string().into()),
            &ExpectedType::Bool,
            &config,
        );
        assert!(matches!(result, CoercionResult::Coerced(Value::Bool(true))));
    }

    #[test]
    fn test_custom_bool_false_values() {
        let config = CoercionConfig {
            level: CoercionLevel::Standard,
            bool_true_values: vec!["yes".into()],
            bool_false_values: vec!["no".into(), "nope".into()],
            ..Default::default()
        };

        let result = coerce_with_config(
            Value::String("no".to_string().into()),
            &ExpectedType::Bool,
            &config,
        );
        assert!(matches!(
            result,
            CoercionResult::Coerced(Value::Bool(false))
        ));

        let result = coerce_with_config(
            Value::String("nope".to_string().into()),
            &ExpectedType::Bool,
            &config,
        );
        assert!(matches!(
            result,
            CoercionResult::Coerced(Value::Bool(false))
        ));
    }

    #[test]
    fn test_custom_bool_unrecognized_value() {
        let config = CoercionConfig {
            level: CoercionLevel::Standard,
            bool_true_values: vec!["yes".into()],
            bool_false_values: vec!["no".into()],
            ..Default::default()
        };

        let result = coerce_with_config(
            Value::String("maybe".to_string().into()),
            &ExpectedType::Bool,
            &config,
        );
        assert!(result.is_err());
    }

    // ==================== allow_string_to_number tests ====================

    #[test]
    fn test_disable_string_to_number() {
        let config = CoercionConfig {
            level: CoercionLevel::Standard,
            allow_string_to_number: false,
            ..Default::default()
        };

        let result = coerce_with_config(
            Value::String("42".to_string().into()),
            &ExpectedType::Int,
            &config,
        );
        assert!(result.is_err());

        let result = coerce_with_config(
            Value::String("3.14".to_string().into()),
            &ExpectedType::Float,
            &config,
        );
        assert!(result.is_err());
    }

    // ==================== Numeric coercion tests ====================

    #[test]
    fn test_numeric_null_to_default() {
        let config = CoercionConfig::permissive();
        let result = coerce_with_config(Value::Null, &ExpectedType::Numeric, &config);
        assert!(matches!(result, CoercionResult::Coerced(Value::Int(0))));
    }

    #[test]
    fn test_numeric_string_to_int() {
        let config = CoercionConfig::standard();
        let result = coerce_with_config(
            Value::String("42".to_string().into()),
            &ExpectedType::Numeric,
            &config,
        );
        assert!(matches!(result, CoercionResult::Coerced(Value::Int(42))));
    }

    #[test]
    fn test_numeric_string_to_float() {
        let config = CoercionConfig::standard();
        let result = coerce_with_config(
            Value::String("2.5".to_string().into()),
            &ExpectedType::Numeric,
            &config,
        );
        assert!(
            matches!(result, CoercionResult::Coerced(Value::Float(f)) if (f - 2.5).abs() < 0.001)
        );
    }

    // ==================== Union coercion with config tests ====================

    #[test]
    fn test_union_with_permissive_config() {
        let union = ExpectedType::Union(vec![ExpectedType::Int, ExpectedType::String]);
        let config = CoercionConfig::permissive();

        // Float to Int should work with permissive config
        let result = coerce_with_config(Value::Float(42.7), &union, &config);
        assert!(matches!(result, CoercionResult::Coerced(Value::Int(42))));
    }

    #[test]
    fn test_union_with_standard_config() {
        let union = ExpectedType::Union(vec![ExpectedType::Int, ExpectedType::Bool]);
        let config = CoercionConfig::standard();

        // String to Int should work with standard config
        let result = coerce_with_config(Value::String("42".to_string().into()), &union, &config);
        assert!(matches!(result, CoercionResult::Coerced(Value::Int(42))));
    }

    // ==================== Edge cases with config ====================

    #[test]
    fn test_empty_string_to_int() {
        let config = CoercionConfig::standard();
        let result = coerce_with_config(
            Value::String("".to_string().into()),
            &ExpectedType::Int,
            &config,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_only_string_to_int() {
        let config = CoercionConfig::standard();
        let result = coerce_with_config(
            Value::String("   ".to_string().into()),
            &ExpectedType::Int,
            &config,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_nan_to_int_fails() {
        let config = CoercionConfig::permissive();
        let result = coerce_with_config(Value::Float(f64::NAN), &ExpectedType::Int, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_reference_to_string() {
        use crate::value::Reference;
        let config = CoercionConfig::standard();
        let ref_val = Value::Reference(Reference::qualified("User", "123"));
        let result = coerce_with_config(ref_val, &ExpectedType::String, &config);
        assert!(
            matches!(result, CoercionResult::Coerced(Value::String(s)) if s.as_ref() == "@User:123")
        );
    }

    #[test]
    fn test_coerce_mode_to_config_conversion() {
        // Test that the old coerce() function properly converts modes
        let result = coerce(Value::Int(42), &ExpectedType::Float, CoercionMode::Strict);
        assert!(result.is_ok());

        let result = coerce(
            Value::String("42".to_string().into()),
            &ExpectedType::Int,
            CoercionMode::Lenient,
        );
        assert!(result.is_ok());
    }
}

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

//! Configuration and error types for JSON to HEDL conversion

use hedl_core::Value;

/// Default maximum recursion depth for JSON parsing
///
/// Set to 10,000 levels to handle deeply nested JSON structures.
/// This is significantly higher than typical JSON depth but prevents
/// stack overflow from malicious or malformed inputs.
pub const DEFAULT_MAX_DEPTH: usize = 10_000;

/// Default maximum array size for JSON parsing
///
/// Set to 10,000,000 elements to handle large datasets, including
/// large arrays commonly found in data science and ML applications.
pub const DEFAULT_MAX_ARRAY_SIZE: usize = 10_000_000;

/// Default maximum string length for JSON parsing
///
/// Set to 100 MB to handle large strings including base64-encoded
/// binary data, large text fields, and embedded documents.
pub const DEFAULT_MAX_STRING_LENGTH: usize = 100 * 1024 * 1024;

/// Default maximum object size (number of keys)
///
/// Set to 100,000 keys to handle objects with many properties,
/// common in configuration files and metadata-rich documents.
pub const DEFAULT_MAX_OBJECT_SIZE: usize = 100_000;

/// Policy for handling unpaired UTF-16 surrogates in JSON input
///
/// JSON's `\uXXXX` escapes use UTF-16 encoding. Characters outside the
/// Basic Multilingual Plane (U+10000+, including emoji) require surrogate
/// pairs: a high surrogate (0xD800-0xDBFF) followed immediately by a low
/// surrogate (0xDC00-0xDFFF).
///
/// Some systems (e.g., JavaScript with truncated strings, legacy databases)
/// may emit unpaired surrogates, which are technically invalid Unicode but
/// may appear in real-world data.
///
/// # Example
///
/// ```text
/// use hedl_json::{FromJsonConfig, SurrogatePolicy};
///
/// // Default: reject unpaired surrogates
/// let strict = FromJsonConfig::default();
///
/// // Replace unpaired surrogates with U+FFFD
/// let lenient = FromJsonConfig::builder()
///     .surrogate_policy(SurrogatePolicy::ReplaceWithFFFD)
///     .build();
///
/// // Skip (remove) unpaired surrogates entirely
/// let skip = FromJsonConfig::builder()
///     .surrogate_policy(SurrogatePolicy::Skip)
///     .build();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SurrogatePolicy {
    /// Reject unpaired surrogates with an error (default, strict)
    ///
    /// This is the safest option and ensures all processed JSON contains
    /// valid Unicode. Use this for data integrity requirements.
    #[default]
    Reject,

    /// Replace unpaired surrogates with U+FFFD (replacement character)
    ///
    /// This allows processing of JSON with invalid Unicode while preserving
    /// string structure. The replacement character (�) signals data loss.
    ReplaceWithFFFD,

    /// Skip (remove) unpaired surrogates silently
    ///
    /// Use with caution: this modifies string content without indication.
    /// Suitable when the surrogates are known to be noise or artifacts.
    Skip,
}

/// Errors that can occur during JSON to HEDL conversion
#[derive(Debug, Clone, thiserror::Error)]
pub enum JsonConversionError {
    /// JSON parsing failed
    #[error("JSON parse error: {0}")]
    ParseError(String),

    /// Root value must be an object
    #[error("Root must be a JSON object, found {0}")]
    InvalidRoot(String),

    /// Invalid number value
    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    /// Invalid expression syntax
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),

    /// Invalid tensor element
    #[error("Invalid tensor element - must be number or array")]
    InvalidTensor,

    /// Nested objects not allowed in scalar context
    #[error("Nested objects not allowed in scalar context")]
    NestedObject,

    /// Reference parsing failed
    #[error("Invalid reference: {0}")]
    InvalidReference(String),

    /// Invalid Unicode encoding
    ///
    /// This error occurs when JSON contains invalid Unicode sequences, such as:
    /// - Unpaired UTF-16 surrogates (`\uD83D` without its low surrogate pair)
    /// - Invalid surrogate pairs (low surrogate before high surrogate)
    /// - Unescaped control characters in strings
    ///
    /// # UTF-16 Surrogate Background
    ///
    /// JSON's `\uXXXX` escapes use UTF-16 encoding. Characters outside the
    /// Basic Multilingual Plane (U+10000 and above, including emoji) require
    /// surrogate pairs: a high surrogate (0xD800-0xDBFF) followed by a low
    /// surrogate (0xDC00-0xDFFF).
    ///
    /// # Solutions
    ///
    /// 1. **Use the `SurrogatePolicy::ReplaceWithFFFD` option**:
    ///    Replace invalid surrogates with the Unicode replacement character.
    ///
    /// 2. **Preprocess the JSON** to fix or remove invalid sequences.
    ///
    /// 3. **Ensure the source system** produces valid UTF-8/UTF-16 pairs.
    #[error("Invalid Unicode: {0}")]
    InvalidUnicode(String),

    /// Maximum recursion depth exceeded
    #[error("Maximum recursion depth ({0}) exceeded - possible deeply nested structure")]
    MaxDepthExceeded(usize),

    /// Maximum array size exceeded
    #[error("Maximum array size ({0}) exceeded - array has {1} elements")]
    MaxArraySizeExceeded(usize, usize),

    /// Maximum string length exceeded
    #[error("Maximum string length ({0}) exceeded - string has {1} characters")]
    MaxStringLengthExceeded(usize, usize),

    /// Maximum object size exceeded
    #[error("Maximum object size ({0}) exceeded - object has {1} keys")]
    MaxObjectSizeExceeded(usize, usize),

    /// Integer value outside i64 range
    ///
    /// JSON supports arbitrary-precision numbers, but HEDL's `Value::Int`
    /// uses `i64` which has a fixed range: -9,223,372,036,854,775,808 to
    /// 9,223,372,036,854,775,807.
    ///
    /// # Common Causes
    ///
    /// - Twitter/Snowflake IDs (often exceed `i64::MAX`)
    /// - Unsigned 64-bit integers from other systems
    /// - Large database auto-increment IDs
    /// - Timestamps in nanoseconds beyond year 2262
    ///
    /// # Solutions
    ///
    /// 1. **Use strings for large IDs** (recommended):
    ///    ```json
    ///    {"tweet_id": "18446744073709551615"}
    ///    ```
    ///
    /// 2. **Use hex encoding**:
    ///    ```json
    ///    {"large_number": "0xFFFFFFFFFFFFFFFF"}
    ///    ```
    ///
    /// 3. **Split into high/low parts**:
    ///    ```json
    ///    {"value_high": 1844674407, "value_low": 3709551615}
    ///    ```
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_json::{from_json, FromJsonConfig};
    ///
    /// let json = r#"{"id": 18446744073709551615}"#;
    /// let result = from_json(json, &FromJsonConfig::default());
    ///
    /// assert!(result.is_err());
    /// assert!(result.unwrap_err().to_string().contains("Integer overflow"));
    /// ```
    #[error(
        "Integer overflow: {value} exceeds i64 range [{min}..{max}]. \
         Consider using a string for large IDs or timestamps."
    )]
    IntegerOverflow {
        /// String representation of the overflowing value.
        value: String,
        /// Maximum valid i64 value.
        max: i64,
        /// Minimum valid i64 value.
        min: i64,
    },
}

impl From<serde_json::Error> for JsonConversionError {
    fn from(err: serde_json::Error) -> Self {
        let msg = err.to_string();

        // Detect surrogate-related errors from serde_json
        if msg.contains("lone surrogate")
            || msg.contains("surrogate")
            || msg.contains("invalid unicode")
        {
            JsonConversionError::InvalidUnicode(format!(
                "Invalid UTF-16 surrogate sequence: {msg}. \
                 JSON contains unpaired surrogates which cannot be represented \
                 in Rust UTF-8 strings. Configure SurrogatePolicy::ReplaceWithFFFD \
                 to replace with the Unicode replacement character (U+FFFD)."
            ))
        } else if msg.contains("control character") {
            JsonConversionError::InvalidUnicode(format!(
                "Unescaped control character in JSON string: {msg}. \
                 Control characters (U+0000-U+001F) must be escaped as \\uXXXX \
                 per RFC 8259."
            ))
        } else {
            JsonConversionError::ParseError(msg)
        }
    }
}

/// Check if a `serde_json::Number` represents an integer outside i64 range
///
/// Returns `true` if the number is an integer (not a float) but cannot
/// fit in i64 range. This happens when:
/// - The value is larger than `i64::MAX` (9,223,372,036,854,775,807)
/// - The value is smaller than `i64::MIN` (-9,223,372,036,854,775,808)
///
/// # Implementation Note
///
/// `serde_json::Number::as_i64()` returns `None` for both:
/// 1. Numbers outside i64 range (overflow)
/// 2. Floating point numbers
///
/// We use `as_u64()` to detect case 1: if `as_i64()` fails but `as_u64()`
/// succeeds, the number is an unsigned integer too large for i64.
/// We also check `is_i64()` to catch negative overflow cases.
#[inline]
pub(super) fn is_integer_overflow(n: &serde_json::Number) -> bool {
    // If as_i64() fails but as_u64() succeeds, it's an unsigned int overflow
    // Or if is_i64() is true but as_i64() is None, it's a signed int overflow
    n.as_i64().is_none() && (n.as_u64().is_some() || n.is_i64())
}

/// Convert JSON number to HEDL Value with overflow detection
///
/// This function enforces strict integer validation to prevent silent
/// precision loss from i64 overflow converting to f64.
///
/// # Behavior
///
/// 1. **i64 range integers**: Convert to `Value::Int(i64)`
/// 2. **Overflow integers**: Return `IntegerOverflow` error
/// 3. **Floating point**: Convert to `Value::Float(f64)`
///
/// # Implementation Details
///
/// - Valid i64 values are converted to `Value::Int`
/// - Integer values outside i64 range trigger `IntegerOverflow` error
/// - Floating point values are converted to `Value::Float`
/// - Uses fast-path optimization for common i64 case
#[inline]
pub(super) fn json_number_to_value(n: &serde_json::Number) -> Result<Value, JsonConversionError> {
    // Try i64 first (most common case - fast path)
    if let Some(i) = n.as_i64() {
        return Ok(Value::Int(i));
    }

    // Check for integer overflow
    if is_integer_overflow(n) {
        return Err(JsonConversionError::IntegerOverflow {
            value: n.to_string(),
            max: i64::MAX,
            min: i64::MIN,
        });
    }

    // Must be a float
    if let Some(f) = n.as_f64() {
        Ok(Value::Float(f))
    } else {
        // Should never happen with valid JSON
        Err(JsonConversionError::InvalidNumber(n.to_string()))
    }
}

/// Configuration for JSON import
///
/// Controls how JSON is converted to HEDL, including security limits
/// to prevent denial-of-service attacks from malicious inputs.
///
/// # High Default Limits
///
/// The default limits are set intentionally high to handle large-scale
/// data processing scenarios common in ML/AI applications:
///
/// - **10,000 depth**: Deep nesting in complex hierarchical data
/// - **10,000,000 array size**: Large datasets and batches
/// - **100 MB string length**: Base64-encoded binary data, embeddings
/// - **100,000 object size**: Rich metadata and configuration objects
///
/// These defaults prioritize functionality over restrictiveness. For
/// untrusted input, consider using the builder pattern with custom limits.
///
/// # Examples
///
/// ```text
/// use hedl_json::FromJsonConfig;
///
/// // Default configuration with high limits for ML/data workloads
/// let config = FromJsonConfig::default();
///
/// // Custom configuration using builder pattern
/// let custom_config = FromJsonConfig::builder()
///     .max_depth(1_000)
///     .max_array_size(100_000)
///     .max_string_length(10 * 1024 * 1024) // 10 MB
///     .build();
///
/// // Strict configuration for untrusted input
/// let strict_config = FromJsonConfig::builder()
///     .max_depth(50)
///     .max_array_size(10_000)
///     .max_string_length(1_000_000)
///     .max_object_size(1_000)
///     .build();
///
/// // Unlimited configuration (use with caution)
/// let unlimited_config = FromJsonConfig::builder()
///     .unlimited()
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct FromJsonConfig {
    /// Default type name for arrays without metadata
    pub default_type_name: String,

    /// HEDL version to use
    pub version: (u32, u32),

    /// Maximum recursion depth (default: 10,000)
    ///
    /// Prevents stack overflow from deeply nested JSON structures.
    /// Set to `None` to disable (not recommended for untrusted input).
    pub max_depth: Option<usize>,

    /// Maximum array size (default: 10,000,000)
    ///
    /// Prevents memory exhaustion from extremely large arrays.
    /// JSON arrays can contain large datasets, batches, or embeddings.
    /// Set to `None` to disable (not recommended for untrusted input).
    pub max_array_size: Option<usize>,

    /// Maximum string length (default: 100 MB)
    ///
    /// Prevents memory exhaustion from extremely large strings.
    /// JSON strings often contain base64-encoded binary data, large
    /// text fields, or embedded documents requiring high limits.
    /// Set to `None` to disable (not recommended for untrusted input).
    pub max_string_length: Option<usize>,

    /// Maximum object size (default: 100,000)
    ///
    /// Prevents memory exhaustion from objects with many keys.
    /// Configuration files and metadata-rich objects can have many properties.
    /// Set to `None` to disable (not recommended for untrusted input).
    pub max_object_size: Option<usize>,

    /// Policy for handling unpaired UTF-16 surrogates
    ///
    /// Some systems emit JSON with unpaired surrogates (e.g., truncated
    /// JavaScript strings). This setting controls how to handle them.
    ///
    /// Default: `SurrogatePolicy::Reject` (strict validation)
    pub surrogate_policy: SurrogatePolicy,

    /// Enable lenient JSON parsing (JSON5-style trailing commas and comments)
    ///
    /// When enabled, the parser accepts:
    /// - Trailing commas in arrays and objects
    /// - Single-line (//) and multi-line (/* */) comments
    ///
    /// Requires the `lenient` feature flag.
    ///
    /// Default: false (strict RFC 8259 JSON)
    #[cfg(feature = "lenient")]
    pub lenient: bool,
}

impl Default for FromJsonConfig {
    fn default() -> Self {
        Self {
            default_type_name: "Item".to_string(),
            version: (2, 0),
            max_depth: Some(DEFAULT_MAX_DEPTH),
            max_array_size: Some(DEFAULT_MAX_ARRAY_SIZE),
            max_string_length: Some(DEFAULT_MAX_STRING_LENGTH),
            max_object_size: Some(DEFAULT_MAX_OBJECT_SIZE),
            surrogate_policy: SurrogatePolicy::default(),
            #[cfg(feature = "lenient")]
            lenient: false,
        }
    }
}

impl FromJsonConfig {
    /// Create a new builder for configuring JSON import
    ///
    /// # Examples
    ///
    /// ```text
    /// use hedl_json::FromJsonConfig;
    ///
    /// let config = FromJsonConfig::builder()
    ///     .max_depth(1_000)
    ///     .max_array_size(100_000)
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder() -> FromJsonConfigBuilder {
        FromJsonConfigBuilder::default()
    }
}

impl hedl_core::convert::ImportConfig for FromJsonConfig {
    fn default_type_name(&self) -> &str {
        &self.default_type_name
    }

    fn version(&self) -> (u32, u32) {
        self.version
    }
}

/// Builder for `FromJsonConfig`
///
/// Provides ergonomic configuration of JSON import limits and behavior.
///
/// # Examples
///
/// ```text
/// use hedl_json::FromJsonConfig;
///
/// // Custom limits
/// let config = FromJsonConfig::builder()
///     .max_depth(1_000)
///     .max_array_size(100_000)
///     .max_string_length(10 * 1024 * 1024)
///     .build();
///
/// // Strict limits for untrusted input
/// let strict = FromJsonConfig::builder()
///     .max_depth(50)
///     .max_array_size(10_000)
///     .max_string_length(1_000_000)
///     .max_object_size(1_000)
///     .build();
///
/// // Unlimited (use with caution!)
/// let unlimited = FromJsonConfig::builder()
///     .unlimited()
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct FromJsonConfigBuilder {
    default_type_name: String,
    version: (u32, u32),
    max_depth: Option<usize>,
    max_array_size: Option<usize>,
    max_string_length: Option<usize>,
    max_object_size: Option<usize>,
    surrogate_policy: SurrogatePolicy,
    #[cfg(feature = "lenient")]
    lenient: bool,
}

impl Default for FromJsonConfigBuilder {
    fn default() -> Self {
        Self {
            default_type_name: "Item".to_string(),
            version: (2, 0),
            max_depth: Some(DEFAULT_MAX_DEPTH),
            max_array_size: Some(DEFAULT_MAX_ARRAY_SIZE),
            max_string_length: Some(DEFAULT_MAX_STRING_LENGTH),
            max_object_size: Some(DEFAULT_MAX_OBJECT_SIZE),
            surrogate_policy: SurrogatePolicy::default(),
            #[cfg(feature = "lenient")]
            lenient: false,
        }
    }
}

impl FromJsonConfigBuilder {
    /// Set the default type name for arrays without metadata
    pub fn default_type_name(mut self, name: impl Into<String>) -> Self {
        self.default_type_name = name.into();
        self
    }

    /// Set the HEDL version to use
    #[must_use]
    pub fn version(mut self, major: u32, minor: u32) -> Self {
        self.version = (major, minor);
        self
    }

    /// Set the maximum recursion depth
    ///
    /// Use `None` to disable the limit (not recommended for untrusted input).
    #[must_use]
    pub fn max_depth(mut self, limit: usize) -> Self {
        self.max_depth = Some(limit);
        self
    }

    /// Set the maximum array size
    ///
    /// Use `None` to disable the limit (not recommended for untrusted input).
    #[must_use]
    pub fn max_array_size(mut self, limit: usize) -> Self {
        self.max_array_size = Some(limit);
        self
    }

    /// Set the maximum string length in bytes
    ///
    /// Use `None` to disable the limit (not recommended for untrusted input).
    #[must_use]
    pub fn max_string_length(mut self, limit: usize) -> Self {
        self.max_string_length = Some(limit);
        self
    }

    /// Set the maximum object size (number of keys)
    ///
    /// Use `None` to disable the limit (not recommended for untrusted input).
    #[must_use]
    pub fn max_object_size(mut self, limit: usize) -> Self {
        self.max_object_size = Some(limit);
        self
    }

    /// Set the policy for handling unpaired UTF-16 surrogates
    ///
    /// # Options
    ///
    /// - `SurrogatePolicy::Reject` (default): Error on invalid surrogates
    /// - `SurrogatePolicy::ReplaceWithFFFD`: Replace with U+FFFD
    /// - `SurrogatePolicy::Skip`: Remove invalid surrogates silently
    ///
    /// # Example
    ///
    /// ```text
    /// use hedl_json::{FromJsonConfig, SurrogatePolicy};
    ///
    /// let config = FromJsonConfig::builder()
    ///     .surrogate_policy(SurrogatePolicy::ReplaceWithFFFD)
    ///     .build();
    /// ```
    #[must_use]
    pub fn surrogate_policy(mut self, policy: SurrogatePolicy) -> Self {
        self.surrogate_policy = policy;
        self
    }

    /// Disable all limits (use with caution - only for trusted input)
    ///
    /// This removes all safety limits and can lead to memory exhaustion
    /// or stack overflow with malicious or malformed JSON.
    #[must_use]
    pub fn unlimited(mut self) -> Self {
        self.max_depth = None;
        self.max_array_size = None;
        self.max_string_length = None;
        self.max_object_size = None;
        self
    }

    /// Enable lenient JSON parsing (trailing commas, comments)
    ///
    /// When enabled, the parser accepts:
    /// - Trailing commas in arrays and objects
    /// - Single-line (//) and multi-line (/* */) comments
    ///
    /// Requires the `lenient` feature flag.
    ///
    /// # Examples
    ///
    /// ```text
    /// use hedl_json::FromJsonConfig;
    ///
    /// let config = FromJsonConfig::builder()
    ///     .lenient(true)
    ///     .build();
    ///
    /// // Now you can parse JSON with trailing commas
    /// let json = r#"{"name": "Alice", "age": 30,}"#;
    /// ```
    #[cfg(feature = "lenient")]
    #[must_use]
    pub fn lenient(mut self, lenient: bool) -> Self {
        self.lenient = lenient;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> FromJsonConfig {
        FromJsonConfig {
            default_type_name: self.default_type_name,
            version: self.version,
            max_depth: self.max_depth,
            max_array_size: self.max_array_size,
            max_string_length: self.max_string_length,
            max_object_size: self.max_object_size,
            surrogate_policy: self.surrogate_policy,
            #[cfg(feature = "lenient")]
            lenient: self.lenient,
        }
    }
}

/// Schema cache for avoiding redundant schema inference
///
/// When converting large JSON arrays to matrix lists, we often encounter the same
/// structure repeatedly. Caching the inferred schema significantly improves performance
/// by avoiding redundant key iteration and sorting.
///
/// # Performance Impact
///
/// - First schema inference: ~O(n*log(n)) where n is number of keys
/// - Cached lookup: ~O(1) hash map lookup
/// - Expected speedup: 30-50% for documents with repeated array structures
pub(super) type SchemaCache = std::collections::HashMap<Vec<String>, Vec<String>>;

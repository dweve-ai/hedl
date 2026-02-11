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

//! Partial parsing with error recovery

use super::array_conversion::{is_object_array, is_tensor_array};
use super::config::{
    is_integer_overflow, json_number_to_value, FromJsonConfig, JsonConversionError, SchemaCache,
};
use crate::DEFAULT_SCHEMA;
use hedl_core::convert::parse_reference;
use hedl_core::lex::{parse_expression_token, singularize_and_capitalize, Tensor};
use hedl_core::{Document, Item, MatrixList, Node, Value};
use serde_json::{Map, Value as JsonValue};
use std::collections::BTreeMap;

/// Error tolerance strategy for partial parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ErrorTolerance {
    /// Stop on the first error encountered
    #[default]
    StopOnFirst,

    /// Collect up to N errors before stopping
    MaxErrors(usize),

    /// Collect all errors and continue parsing
    CollectAll,

    /// Skip invalid items in arrays/objects and continue
    SkipInvalidItems,
}

/// Location information for an error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorLocation {
    /// JSON path to the error (e.g., `$.users[2].email`)
    pub path: String,

    /// Depth in the JSON structure
    pub depth: usize,
}

impl ErrorLocation {
    fn root() -> Self {
        Self {
            path: "$".to_string(),
            depth: 0,
        }
    }

    fn child(&self, key: &str) -> Self {
        Self {
            path: format!("{}.{}", self.path, key),
            depth: self.depth + 1,
        }
    }

    fn index(&self, idx: usize) -> Self {
        Self {
            path: format!("{}[{}]", self.path, idx),
            depth: self.depth + 1,
        }
    }
}

/// Captured error during partial parsing
#[derive(Debug, Clone)]
pub struct ParseError {
    /// The error that occurred
    pub error: JsonConversionError,

    /// Location where the error occurred
    pub location: ErrorLocation,

    /// Whether this error is fatal (prevents document creation)
    pub is_fatal: bool,
}

impl ParseError {
    fn new(error: JsonConversionError, location: ErrorLocation, is_fatal: bool) -> Self {
        Self {
            error,
            location,
            is_fatal,
        }
    }
}

/// Configuration for partial parsing
#[derive(Debug, Clone, Default)]
pub struct PartialConfig {
    /// Base configuration for JSON conversion
    pub from_json_config: FromJsonConfig,

    /// Error tolerance strategy
    pub tolerance: ErrorTolerance,

    /// Whether to include partial results even on fatal errors
    pub include_partial_on_fatal: bool,

    /// Replace invalid values with null instead of skipping
    pub replace_invalid_with_null: bool,
}

impl PartialConfig {
    /// Create a new builder for partial parsing configuration
    #[must_use]
    pub fn builder() -> PartialConfigBuilder {
        PartialConfigBuilder::default()
    }
}

/// Builder for `PartialConfig`
#[derive(Debug, Clone, Default)]
pub struct PartialConfigBuilder {
    from_json_config: FromJsonConfig,
    tolerance: ErrorTolerance,
    include_partial_on_fatal: bool,
    replace_invalid_with_null: bool,
}

impl PartialConfigBuilder {
    /// Set the base `FromJsonConfig`
    #[must_use]
    pub fn from_json_config(mut self, config: FromJsonConfig) -> Self {
        self.from_json_config = config;
        self
    }

    /// Set the error tolerance strategy
    #[must_use]
    pub fn tolerance(mut self, tolerance: ErrorTolerance) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Set whether to include partial results on fatal errors
    #[must_use]
    pub fn include_partial_on_fatal(mut self, value: bool) -> Self {
        self.include_partial_on_fatal = value;
        self
    }

    /// Set whether to replace invalid values with null
    #[must_use]
    pub fn replace_invalid_with_null(mut self, value: bool) -> Self {
        self.replace_invalid_with_null = value;
        self
    }

    /// Build the `PartialConfig`
    #[must_use]
    pub fn build(self) -> PartialConfig {
        PartialConfig {
            from_json_config: self.from_json_config,
            tolerance: self.tolerance,
            include_partial_on_fatal: self.include_partial_on_fatal,
            replace_invalid_with_null: self.replace_invalid_with_null,
        }
    }
}

/// Result of partial parsing
#[derive(Debug)]
pub struct PartialResult {
    /// Parsed document (if any)
    pub document: Option<Document>,

    /// All errors encountered during parsing
    pub errors: Vec<ParseError>,

    /// Whether parsing stopped early due to error limits
    pub stopped_early: bool,
}

impl PartialResult {
    /// Check if parsing completed successfully without errors
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.errors.is_empty() && self.document.is_some()
    }

    /// Check if parsing failed (fatal errors or no document)
    #[must_use]
    pub fn is_failed(&self) -> bool {
        self.errors.iter().any(|e| e.is_fatal) || self.document.is_none()
    }

    /// Convert to Result type for simpler error handling
    pub fn into_result(self) -> Result<Document, Vec<ParseError>> {
        if self.errors.is_empty() {
            self.document.ok_or_else(Vec::new)
        } else {
            Err(self.errors)
        }
    }
}

/// Error collection context for partial parsing
struct ErrorContext {
    errors: Vec<ParseError>,
    config: PartialConfig,
    stopped: bool,
}

impl ErrorContext {
    fn new(config: PartialConfig) -> Self {
        Self {
            errors: Vec::new(),
            config,
            stopped: false,
        }
    }

    /// Record an error and determine if parsing should continue
    fn record_error(
        &mut self,
        error: JsonConversionError,
        location: ErrorLocation,
        is_fatal: bool,
    ) -> bool {
        if self.stopped {
            return false;
        }

        let parse_error = ParseError::new(error, location, is_fatal);
        self.errors.push(parse_error);

        // Check if we should stop
        let should_stop = match self.config.tolerance {
            ErrorTolerance::StopOnFirst => true,
            ErrorTolerance::MaxErrors(max) => self.errors.len() >= max,
            ErrorTolerance::CollectAll => false,
            ErrorTolerance::SkipInvalidItems => is_fatal,
        };

        if should_stop {
            self.stopped = true;
        }

        !should_stop
    }

    fn should_continue(&self) -> bool {
        !self.stopped
    }
}

/// Bundled mutable state for partial parsing functions.
///
/// Groups the shared parameters that are threaded through all partial parsing
/// functions, reducing argument counts and making call sites cleaner.
struct PartialParseState<'a> {
    /// Base configuration for JSON conversion limits.
    config: &'a FromJsonConfig,
    /// Accumulated struct definitions (type_name -> schema columns).
    structs: &'a mut BTreeMap<String, Vec<String>>,
    /// Cache for inferred schemas to avoid redundant computation.
    schema_cache: &'a mut SchemaCache,
    /// Error collection and tolerance context.
    context: &'a mut ErrorContext,
}

/// Parse JSON string with partial error recovery
///
/// This function attempts to parse as much of the JSON as possible,
/// collecting errors instead of failing on the first error.
///
/// # Examples
///
/// ```text
/// use hedl_json::from_json::{partial_parse_json, PartialConfig, ErrorTolerance};
///
/// let json = r#"{"valid": "data", "invalid": ...}"#;
/// let config = PartialConfig::builder()
///     .tolerance(ErrorTolerance::CollectAll)
///     .build();
///
/// let result = partial_parse_json(json, &config);
/// assert!(result.document.is_some());
/// assert!(!result.errors.is_empty());
/// ```
#[must_use]
pub fn partial_parse_json(json: &str, config: &PartialConfig) -> PartialResult {
    // Try to parse JSON first
    let value = match serde_json::from_str::<JsonValue>(json) {
        Ok(v) => v,
        Err(e) => {
            // Fatal JSON parsing error
            return PartialResult {
                document: None,
                errors: vec![ParseError::new(
                    JsonConversionError::ParseError(e.to_string()),
                    ErrorLocation::root(),
                    true,
                )],
                stopped_early: false,
            };
        }
    };

    partial_parse_json_value(&value, config)
}

/// Parse `serde_json::Value` with partial error recovery
#[must_use]
pub fn partial_parse_json_value(value: &JsonValue, config: &PartialConfig) -> PartialResult {
    let mut context = ErrorContext::new(config.clone());
    let mut structs = BTreeMap::new();
    let mut schema_cache = SchemaCache::new();

    // Try to parse the root
    let root = if let JsonValue::Object(map) = value {
        let mut state = PartialParseState {
            config: &config.from_json_config,
            structs: &mut structs,
            schema_cache: &mut schema_cache,
            context: &mut context,
        };
        match partial_json_object_to_root(map, &mut state, 0, &ErrorLocation::root()) {
            Ok(root) => Some(root),
            Err(_) => {
                if config.include_partial_on_fatal {
                    Some(BTreeMap::new())
                } else {
                    None
                }
            }
        }
    } else {
        context.record_error(
            JsonConversionError::InvalidRoot(format!("{value:?}")),
            ErrorLocation::root(),
            true,
        );
        None
    };

    let document = root.map(|root| Document {
        version: config.from_json_config.version,
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs,
        nests: BTreeMap::new(),
        root,
    });

    PartialResult {
        document,
        errors: context.errors,
        stopped_early: context.stopped,
    }
}

/// Partial parsing version of `json_object_to_root`
fn partial_json_object_to_root(
    map: &Map<String, JsonValue>,
    state: &mut PartialParseState<'_>,
    depth: usize,
    location: &ErrorLocation,
) -> Result<BTreeMap<String, Item>, JsonConversionError> {
    // Check object size limit
    if let Some(max_size) = state.config.max_object_size {
        if map.len() > max_size {
            let err = JsonConversionError::MaxObjectSizeExceeded(max_size, map.len());
            state
                .context
                .record_error(err.clone(), location.clone(), false);
            return Err(err);
        }
    }

    let mut result = BTreeMap::new();

    for (key, value) in map {
        if !state.context.should_continue() {
            break;
        }

        // Skip metadata keys
        if key.starts_with("__") {
            continue;
        }

        let item_location = location.child(key);
        match partial_json_value_to_item(value, key, state, depth, &item_location) {
            Ok(item) => {
                result.insert(key.clone(), item);
            }
            Err(_) => {
                // Error already recorded in partial_json_value_to_item
                if state.context.config.replace_invalid_with_null {
                    result.insert(key.clone(), Item::Scalar(Value::Null));
                }
                // Otherwise skip this item
            }
        }
    }

    Ok(result)
}

/// Partial parsing version of `json_value_to_item`
fn partial_json_value_to_item(
    value: &JsonValue,
    key: &str,
    state: &mut PartialParseState<'_>,
    depth: usize,
    location: &ErrorLocation,
) -> Result<Item, JsonConversionError> {
    // Check recursion depth
    if let Some(max_depth) = state.config.max_depth {
        if depth >= max_depth {
            let err = JsonConversionError::MaxDepthExceeded(max_depth);
            state
                .context
                .record_error(err.clone(), location.clone(), false);
            return Err(err);
        }
    }

    match value {
        JsonValue::Null => Ok(Item::Scalar(Value::Null)),
        JsonValue::Bool(b) => Ok(Item::Scalar(Value::Bool(*b))),
        JsonValue::Number(n) => match json_number_to_value(n) {
            Ok(value) => Ok(Item::Scalar(value)),
            Err(err) => {
                state
                    .context
                    .record_error(err.clone(), location.clone(), false);
                Err(err)
            }
        },
        JsonValue::String(s) => {
            // Check string length limit
            if let Some(max_len) = state.config.max_string_length {
                if s.len() > max_len {
                    let err = JsonConversionError::MaxStringLengthExceeded(max_len, s.len());
                    state
                        .context
                        .record_error(err.clone(), location.clone(), false);
                    return Err(err);
                }
            }

            // Check for expression pattern $( ... )
            if s.starts_with("$(") && s.ends_with(')') {
                match parse_expression_token(s) {
                    Ok(expr) => Ok(Item::Scalar(Value::Expression(Box::new(expr)))),
                    Err(e) => {
                        let err = JsonConversionError::InvalidExpression(e.to_string());
                        state
                            .context
                            .record_error(err.clone(), location.clone(), false);
                        Err(err)
                    }
                }
            } else {
                Ok(Item::Scalar(Value::String(s.clone().into_boxed_str())))
            }
        }
        JsonValue::Array(arr) => {
            // Check array size limit
            if let Some(max_size) = state.config.max_array_size {
                if arr.len() > max_size {
                    let err = JsonConversionError::MaxArraySizeExceeded(max_size, arr.len());
                    state
                        .context
                        .record_error(err.clone(), location.clone(), false);
                    return Err(err);
                }
            }

            // Handle empty arrays
            if arr.is_empty() {
                let type_name = singularize_and_capitalize(key);
                let schema: Vec<String> = DEFAULT_SCHEMA.iter().map(|s| (*s).to_string()).collect();
                let mut list = MatrixList::new(type_name.clone(), schema.clone());
                list.count_hint = Some(0);
                state.structs.insert(type_name, schema);
                Ok(Item::List(list))
            } else if is_tensor_array(arr) {
                match partial_json_array_to_tensor(
                    arr,
                    state.config,
                    depth + 1,
                    location,
                    state.context,
                ) {
                    Ok(tensor) => Ok(Item::Scalar(Value::Tensor(Box::new(tensor)))),
                    Err(err) => Err(err),
                }
            } else if is_object_array(arr) {
                match partial_json_array_to_matrix_list(arr, key, state, depth + 1, location) {
                    Ok(list) => Ok(Item::List(list)),
                    Err(err) => Err(err),
                }
            } else {
                // Primitive/mixed array (strings, bools, nulls, or heterogeneous)
                // Per HEDL SPEC: Tensor is for numerical data only.
                // No native string array type exists in HEDL, so serialize as JSON string
                // for lossless roundtrip conversion.
                Ok(Item::Scalar(Value::String(
                    serde_json::to_string(&JsonValue::Array(arr.to_vec()))
                        .unwrap_or_else(|_| "[]".to_string())
                        .into_boxed_str(),
                )))
            }
        }
        JsonValue::Object(obj) => {
            // Check for special keys
            if let Some(JsonValue::String(r)) = obj.get("@ref") {
                match parse_reference(r) {
                    Ok(reference) => Ok(Item::Scalar(Value::Reference(reference))),
                    Err(e) => {
                        let err = JsonConversionError::InvalidReference(e);
                        state
                            .context
                            .record_error(err.clone(), location.clone(), false);
                        Err(err)
                    }
                }
            } else {
                // Regular object
                match partial_json_object_to_item_map(obj, state, depth + 1, location) {
                    Ok(item_map) => Ok(Item::Object(item_map)),
                    Err(err) => Err(err),
                }
            }
        }
    }
}

/// Partial parsing version of `json_object_to_item_map`
fn partial_json_object_to_item_map(
    map: &Map<String, JsonValue>,
    state: &mut PartialParseState<'_>,
    depth: usize,
    location: &ErrorLocation,
) -> Result<BTreeMap<String, Item>, JsonConversionError> {
    // Check object size limit
    if let Some(max_size) = state.config.max_object_size {
        if map.len() > max_size {
            let err = JsonConversionError::MaxObjectSizeExceeded(max_size, map.len());
            state
                .context
                .record_error(err.clone(), location.clone(), false);
            return Err(err);
        }
    }

    let mut result = BTreeMap::new();

    for (key, value) in map {
        if !state.context.should_continue() {
            break;
        }

        if key.starts_with("__") {
            continue;
        }

        let item_location = location.child(key);
        match partial_json_value_to_item(value, key, state, depth, &item_location) {
            Ok(item) => {
                result.insert(key.clone(), item);
            }
            Err(_) => {
                if state.context.config.replace_invalid_with_null {
                    result.insert(key.clone(), Item::Scalar(Value::Null));
                }
            }
        }
    }

    Ok(result)
}

/// Partial parsing version of `json_array_to_tensor`
fn partial_json_array_to_tensor(
    arr: &[JsonValue],
    config: &FromJsonConfig,
    depth: usize,
    location: &ErrorLocation,
    context: &mut ErrorContext,
) -> Result<Tensor, JsonConversionError> {
    // Check recursion depth
    if let Some(max_depth) = config.max_depth {
        if depth >= max_depth {
            let err = JsonConversionError::MaxDepthExceeded(max_depth);
            context.record_error(err.clone(), location.clone(), false);
            return Err(err);
        }
    }

    let mut items = Vec::with_capacity(arr.len());

    for (idx, v) in arr.iter().enumerate() {
        if !context.should_continue() {
            break;
        }

        let elem_location = location.index(idx);
        let tensor = match v {
            JsonValue::Number(n) => {
                // Tensors use f64, but we should detect integer overflow
                // Note: For tensors, overflow to float is acceptable but we still check
                if is_integer_overflow(n) {
                    // For tensors, overflow to float is acceptable but worth noting
                    // in future versions, could add a warning mechanism
                }

                if let Some(f) = n.as_f64() {
                    Ok(Tensor::Scalar(f))
                } else {
                    let err = JsonConversionError::InvalidNumber(n.to_string());
                    context.record_error(err.clone(), elem_location, false);
                    Err(err)
                }
            }
            JsonValue::Array(nested) => {
                partial_json_array_to_tensor(nested, config, depth + 1, &elem_location, context)
            }
            _ => {
                let err = JsonConversionError::InvalidTensor;
                context.record_error(err.clone(), elem_location, false);
                Err(err)
            }
        };

        match tensor {
            Ok(t) => items.push(t),
            Err(_) => {
                if context.config.replace_invalid_with_null {
                    items.push(Tensor::Scalar(0.0));
                }
                // Otherwise skip this item
            }
        }
    }

    Ok(Tensor::Array(items))
}

/// Partial parsing version of `json_array_to_matrix_list`
fn partial_json_array_to_matrix_list(
    arr: &[JsonValue],
    key: &str,
    state: &mut PartialParseState<'_>,
    depth: usize,
    location: &ErrorLocation,
) -> Result<MatrixList, JsonConversionError> {
    // Check recursion depth
    if let Some(max_depth) = state.config.max_depth {
        if depth >= max_depth {
            let err = JsonConversionError::MaxDepthExceeded(max_depth);
            state
                .context
                .record_error(err.clone(), location.clone(), false);
            return Err(err);
        }
    }

    let type_name = singularize_and_capitalize(key);

    // Infer schema from first object
    let schema: Vec<String> = if let Some(JsonValue::Object(first)) = arr.first() {
        if let Some(JsonValue::Array(schema_arr)) = first.get("__hedl_schema") {
            schema_arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        } else {
            let mut cache_key: Vec<String> = first
                .keys()
                .filter(|k| {
                    if k.starts_with("__") {
                        return false;
                    }
                    if let Some(JsonValue::Array(arr)) = first.get(*k) {
                        !is_object_array(arr)
                    } else {
                        true
                    }
                })
                .cloned()
                .collect();
            cache_key.sort();

            if let Some(cached_schema) = state.schema_cache.get(&cache_key) {
                cached_schema.clone()
            } else {
                let mut keys = cache_key.clone();
                if let Some(pos) = keys.iter().position(|k| k == "id") {
                    keys.remove(pos);
                    keys.insert(0, "id".to_string());
                }
                state.schema_cache.insert(cache_key, keys.clone());
                keys
            }
        }
    } else {
        DEFAULT_SCHEMA.iter().map(|s| (*s).to_string()).collect()
    };

    let schema = if schema.is_empty() {
        DEFAULT_SCHEMA.iter().map(|s| (*s).to_string()).collect()
    } else {
        schema
    };

    state.structs.insert(type_name.clone(), schema.clone());

    let mut rows = Vec::with_capacity(arr.len());

    for (idx, item) in arr.iter().enumerate() {
        if !state.context.should_continue() {
            break;
        }

        let row_location = location.index(idx);

        if let JsonValue::Object(obj) = item {
            let id = obj
                .get(&schema[0])
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let mut fields = Vec::with_capacity(schema.len());
            for col in &schema {
                match obj.get(col) {
                    Some(v) => {
                        match partial_json_to_value(
                            v,
                            state.config,
                            &row_location.child(col),
                            state.context,
                        ) {
                            Ok(value) => fields.push(value),
                            Err(_) => {
                                // Replace invalid values with null in partial mode
                                fields.push(Value::Null);
                            }
                        }
                    }
                    None => fields.push(Value::Null),
                }
            }

            // Handle nested children
            let mut children: BTreeMap<String, Vec<Node>> = BTreeMap::new();
            for (child_key, child_value) in obj {
                if !state.context.should_continue() {
                    break;
                }

                if let JsonValue::Array(child_arr) = child_value {
                    if is_object_array(child_arr) {
                        let child_location = row_location.child(child_key);
                        if let Ok(child_list) = partial_json_array_to_matrix_list(
                            child_arr,
                            child_key,
                            state,
                            depth + 1,
                            &child_location,
                        ) {
                            children.insert(child_key.clone(), child_list.rows);
                        } else {
                            // Error already recorded, skip this child
                        }
                    }
                }
            }

            let node = Node {
                type_name: type_name.clone(),
                id,
                fields: fields.into(),
                children: if children.is_empty() {
                    None
                } else {
                    Some(Box::new(children))
                },
                child_count: 0,
            };

            rows.push(node);
        } else {
            // Invalid item in array - record error
            let err = JsonConversionError::InvalidRoot("Expected object in array".to_string());
            state.context.record_error(err, row_location, false);

            // Skip this item based on tolerance
            if state.context.config.tolerance == ErrorTolerance::SkipInvalidItems {
                continue;
            }
        }
    }

    let count_hint = Some(rows.len());

    Ok(MatrixList {
        type_name,
        schema,
        rows,
        count_hint,
    })
}

/// Partial parsing version of `json_to_value`
fn partial_json_to_value(
    value: &JsonValue,
    config: &FromJsonConfig,
    location: &ErrorLocation,
    context: &mut ErrorContext,
) -> Result<Value, JsonConversionError> {
    match value {
        JsonValue::Null => Ok(Value::Null),
        JsonValue::Bool(b) => Ok(Value::Bool(*b)),
        JsonValue::Number(n) => match json_number_to_value(n) {
            Ok(value) => Ok(value),
            Err(err) => {
                context.record_error(err.clone(), location.clone(), false);
                Err(err)
            }
        },
        JsonValue::String(s) => {
            // Check string length limit
            if let Some(max_len) = config.max_string_length {
                if s.len() > max_len {
                    let err = JsonConversionError::MaxStringLengthExceeded(max_len, s.len());
                    context.record_error(err.clone(), location.clone(), false);
                    return Err(err);
                }
            }

            // Check for expression pattern
            if s.starts_with("$(") && s.ends_with(')') {
                match parse_expression_token(s) {
                    Ok(expr) => Ok(Value::Expression(Box::new(expr))),
                    Err(e) => {
                        let err = JsonConversionError::InvalidExpression(e.to_string());
                        context.record_error(err.clone(), location.clone(), false);
                        Err(err)
                    }
                }
            } else {
                Ok(Value::String(s.clone().into_boxed_str()))
            }
        }
        JsonValue::Array(arr) => {
            // Check array size limit
            if let Some(max_size) = config.max_array_size {
                if arr.len() > max_size {
                    let err = JsonConversionError::MaxArraySizeExceeded(max_size, arr.len());
                    context.record_error(err.clone(), location.clone(), false);
                    return Err(err);
                }
            }

            if is_object_array(arr) {
                Ok(Value::Null) // Children processed separately
            } else if is_tensor_array(arr) {
                match partial_json_array_to_tensor(arr, config, 0, location, context) {
                    Ok(tensor) => Ok(Value::Tensor(Box::new(tensor))),
                    Err(err) => Err(err),
                }
            } else if arr.is_empty() {
                Ok(Value::Tensor(Box::new(Tensor::Array(vec![]))))
            } else {
                // Mixed/primitive array within a field (strings, bools, etc.)
                // Convert to JSON string representation since Value doesn't have a list type
                Ok(Value::String(
                    serde_json::to_string(value)
                        .unwrap_or_else(|_| "[]".to_string())
                        .into_boxed_str(),
                ))
            }
        }
        JsonValue::Object(obj) => {
            if let Some(JsonValue::String(r)) = obj.get("@ref") {
                match parse_reference(r) {
                    Ok(reference) => Ok(Value::Reference(reference)),
                    Err(e) => {
                        let err = JsonConversionError::InvalidReference(e);
                        context.record_error(err.clone(), location.clone(), false);
                        Err(err)
                    }
                }
            } else {
                let err = JsonConversionError::NestedObject;
                context.record_error(err.clone(), location.clone(), false);
                Err(err)
            }
        }
    }
}

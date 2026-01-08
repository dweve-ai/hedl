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

//! YAML to HEDL conversion

use crate::error::YamlError;
use crate::DEFAULT_SCHEMA;
use hedl_core::convert::parse_reference;
use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_core::lex::{parse_expression_token, singularize_and_capitalize};
use hedl_core::lex::Tensor;
use serde_yaml::{Mapping, Value as YamlValue};
use std::collections::BTreeMap;

/// Default maximum document size: 500 MB
///
/// YAML documents can be very large, especially for configurations like Kubernetes manifests,
/// large datasets, or complex application configurations. This high default allows processing
/// substantial YAML files while still providing DoS protection.
pub const DEFAULT_MAX_DOCUMENT_SIZE: usize = 500 * 1024 * 1024; // 500 MB

/// Default maximum array length: 10 million elements
///
/// Large YAML arrays are common in data processing, configuration management, and
/// infrastructure-as-code scenarios. This high limit accommodates real-world use cases
/// while preventing unbounded memory allocation.
pub const DEFAULT_MAX_ARRAY_LENGTH: usize = 10_000_000; // 10 million elements

/// Default maximum nesting depth: 10,000 levels
///
/// Deep nesting can occur in complex hierarchical configurations, particularly in
/// infrastructure definitions and data transformations. This high limit supports
/// realistic nesting while preventing stack overflow attacks.
pub const DEFAULT_MAX_NESTING_DEPTH: usize = 10_000; // 10,000 levels

/// Configuration for YAML import
///
/// # Security Considerations
///
/// This configuration includes resource limits to prevent Denial of Service (DoS) attacks
/// through maliciously crafted YAML documents:
///
/// - `max_document_size`: Prevents memory exhaustion from extremely large documents
/// - `max_array_length`: Prevents excessive memory allocation from huge arrays
/// - `max_nesting_depth`: Prevents stack overflow from deeply nested structures
///
/// These limits are enforced during parsing and conversion. Exceeding any limit will
/// result in an error.
///
/// # Default Limits
///
/// The default limits are intentionally high to accommodate real-world YAML files:
/// - Document size: 500 MB (large K8s manifests, datasets)
/// - Array length: 10 million elements (large data arrays)
/// - Nesting depth: 10,000 levels (complex hierarchies)
///
/// # Examples
///
/// ## Using Default Configuration
///
/// ```rust
/// use hedl_yaml::FromYamlConfig;
///
/// let config = FromYamlConfig::default();
/// // Uses high default limits: 500MB / 10M / 10K
/// ```
///
/// ## Customizing Limits with Builder Pattern
///
/// ```rust
/// use hedl_yaml::FromYamlConfig;
///
/// let config = FromYamlConfig::builder()
///     .max_document_size(100 * 1024 * 1024) // 100 MB
///     .max_array_length(1_000_000)           // 1 million
///     .max_nesting_depth(1000)               // 1000 levels
///     .build();
/// ```
///
/// ## Using Struct Initialization
///
/// ```rust
/// use hedl_yaml::FromYamlConfig;
///
/// let config = FromYamlConfig {
///     max_document_size: 200 * 1024 * 1024, // 200 MB
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct FromYamlConfig {
    /// Default type name for arrays without metadata
    pub default_type_name: String,
    /// HEDL version to use
    pub version: (u32, u32),
    /// Maximum allowed document size in bytes (default: 500 MB)
    pub max_document_size: usize,
    /// Maximum allowed array length (default: 10 million elements)
    pub max_array_length: usize,
    /// Maximum allowed nesting depth (default: 10,000 levels)
    pub max_nesting_depth: usize,
}

impl Default for FromYamlConfig {
    fn default() -> Self {
        Self {
            default_type_name: "Item".to_string(),
            version: (1, 0),
            max_document_size: DEFAULT_MAX_DOCUMENT_SIZE,
            max_array_length: DEFAULT_MAX_ARRAY_LENGTH,
            max_nesting_depth: DEFAULT_MAX_NESTING_DEPTH,
        }
    }
}

impl FromYamlConfig {
    /// Creates a new builder for `FromYamlConfig`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_yaml::FromYamlConfig;
    ///
    /// let config = FromYamlConfig::builder()
    ///     .max_document_size(100 * 1024 * 1024)
    ///     .max_array_length(5_000_000)
    ///     .max_nesting_depth(5000)
    ///     .build();
    /// ```
    pub fn builder() -> FromYamlConfigBuilder {
        FromYamlConfigBuilder::new()
    }
}

/// Builder for `FromYamlConfig` providing an ergonomic way to customize configuration.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use hedl_yaml::FromYamlConfig;
///
/// let config = FromYamlConfig::builder()
///     .max_document_size(100 * 1024 * 1024)  // 100 MB
///     .build();
/// ```
///
/// ## Customizing All Limits
///
/// ```rust
/// use hedl_yaml::FromYamlConfig;
///
/// let config = FromYamlConfig::builder()
///     .default_type_name("Entity")
///     .version(2, 0)
///     .max_document_size(200 * 1024 * 1024)  // 200 MB
///     .max_array_length(20_000_000)          // 20 million
///     .max_nesting_depth(20_000)             // 20K levels
///     .build();
/// ```
///
/// ## Conservative Limits for Untrusted Input
///
/// ```rust
/// use hedl_yaml::FromYamlConfig;
///
/// // For processing untrusted YAML from external sources
/// let config = FromYamlConfig::builder()
///     .max_document_size(10 * 1024 * 1024)  // 10 MB only
///     .max_array_length(100_000)             // 100K elements
///     .max_nesting_depth(100)                // 100 levels
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct FromYamlConfigBuilder {
    default_type_name: String,
    version: (u32, u32),
    max_document_size: usize,
    max_array_length: usize,
    max_nesting_depth: usize,
}

impl FromYamlConfigBuilder {
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        Self {
            default_type_name: "Item".to_string(),
            version: (1, 0),
            max_document_size: DEFAULT_MAX_DOCUMENT_SIZE,
            max_array_length: DEFAULT_MAX_ARRAY_LENGTH,
            max_nesting_depth: DEFAULT_MAX_NESTING_DEPTH,
        }
    }

    /// Sets the default type name for arrays without metadata.
    pub fn default_type_name(mut self, name: impl Into<String>) -> Self {
        self.default_type_name = name.into();
        self
    }

    /// Sets the HEDL version to use.
    pub fn version(mut self, major: u32, minor: u32) -> Self {
        self.version = (major, minor);
        self
    }

    /// Sets the maximum allowed document size in bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_yaml::FromYamlConfig;
    ///
    /// // Set to 100 MB
    /// let config = FromYamlConfig::builder()
    ///     .max_document_size(100 * 1024 * 1024)
    ///     .build();
    /// ```
    pub fn max_document_size(mut self, size: usize) -> Self {
        self.max_document_size = size;
        self
    }

    /// Sets the maximum allowed array length.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_yaml::FromYamlConfig;
    ///
    /// // Set to 5 million elements
    /// let config = FromYamlConfig::builder()
    ///     .max_array_length(5_000_000)
    ///     .build();
    /// ```
    pub fn max_array_length(mut self, length: usize) -> Self {
        self.max_array_length = length;
        self
    }

    /// Sets the maximum allowed nesting depth.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_yaml::FromYamlConfig;
    ///
    /// // Set to 5000 levels
    /// let config = FromYamlConfig::builder()
    ///     .max_nesting_depth(5000)
    ///     .build();
    /// ```
    pub fn max_nesting_depth(mut self, depth: usize) -> Self {
        self.max_nesting_depth = depth;
        self
    }

    /// Builds the `FromYamlConfig`.
    pub fn build(self) -> FromYamlConfig {
        FromYamlConfig {
            default_type_name: self.default_type_name,
            version: self.version,
            max_document_size: self.max_document_size,
            max_array_length: self.max_array_length,
            max_nesting_depth: self.max_nesting_depth,
        }
    }
}

impl Default for FromYamlConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl hedl_core::convert::ImportConfig for FromYamlConfig {
    fn default_type_name(&self) -> &str {
        &self.default_type_name
    }

    fn version(&self) -> (u32, u32) {
        self.version
    }
}

/// Convert YAML string to HEDL Document
///
/// # Security
///
/// This function enforces resource limits to prevent DoS attacks:
/// - Checks document size against `max_document_size`
/// - Enforces array length limits during conversion
/// - Enforces nesting depth limits during conversion
///
/// # Errors
///
/// Returns an error if:
/// - The YAML document exceeds `max_document_size`
/// - Any array exceeds `max_array_length`
/// - Nesting depth exceeds `max_nesting_depth`
/// - The YAML is malformed or cannot be parsed
pub fn from_yaml(yaml: &str, config: &FromYamlConfig) -> Result<Document, String> {
    // Check document size before parsing
    if yaml.len() > config.max_document_size {
        return Err(YamlError::DocumentTooLarge {
            size: yaml.len(),
            max_size: config.max_document_size,
        }
        .to_string());
    }

    let value: YamlValue =
        serde_yaml::from_str(yaml).map_err(|e| format!("YAML parse error: {}", e))?;
    from_yaml_value(&value, config)
}

/// Convert serde_yaml::Value to HEDL Document
pub fn from_yaml_value(value: &YamlValue, config: &FromYamlConfig) -> Result<Document, String> {
    let mut structs = BTreeMap::new();
    let root = match value {
        YamlValue::Mapping(map) => yaml_mapping_to_root(map, config, &mut structs, 0)?,
        _ => return Err("Root must be a YAML mapping".into()),
    };

    Ok(Document {
        version: config.version,
        aliases: BTreeMap::new(),
        structs,
        nests: BTreeMap::new(),
        root,
    })
}

fn yaml_mapping_to_root(
    map: &Mapping,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<BTreeMap<String, Item>, String> {
    // Check nesting depth
    if depth > config.max_nesting_depth {
        return Err(YamlError::MaxDepthExceeded {
            max_depth: config.max_nesting_depth,
            actual_depth: depth,
        }
        .to_string());
    }

    let mut root = BTreeMap::new();

    for (key, value) in map {
        // Skip ONLY known HEDL metadata keys (not all keys starting with __)
        let key_str = key.as_str().ok_or("Non-string keys not supported")?;
        if key_str == "__type__" || key_str == "__schema__" {
            continue;
        }

        let item = yaml_value_to_item(value, key_str, config, structs, depth)?;
        root.insert(key_str.to_string(), item);
    }

    Ok(root)
}

fn yaml_mapping_to_item_map(
    map: &Mapping,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<BTreeMap<String, Item>, String> {
    // Check nesting depth
    if depth > config.max_nesting_depth {
        return Err(YamlError::MaxDepthExceeded {
            max_depth: config.max_nesting_depth,
            actual_depth: depth,
        }
        .to_string());
    }

    let mut result = BTreeMap::new();

    for (key, value) in map {
        // Skip ONLY known HEDL metadata keys (not all keys starting with __)
        let key_str = key.as_str().ok_or("Non-string keys not supported")?;
        if key_str == "__type__" || key_str == "__schema__" {
            continue;
        }

        let item = yaml_value_to_item(value, key_str, config, structs, depth)?;
        result.insert(key_str.to_string(), item);
    }

    Ok(result)
}

fn yaml_value_to_item(
    value: &YamlValue,
    key: &str,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<Item, String> {
    // Check nesting depth
    if depth > config.max_nesting_depth {
        return Err(YamlError::MaxDepthExceeded {
            max_depth: config.max_nesting_depth,
            actual_depth: depth,
        }
        .to_string());
    }

    match value {
        YamlValue::Null => Ok(Item::Scalar(Value::Null)),
        YamlValue::Bool(b) => Ok(Item::Scalar(Value::Bool(*b))),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Item::Scalar(Value::Int(i)))
            } else if let Some(f) = n.as_f64() {
                Ok(Item::Scalar(Value::Float(f)))
            } else {
                Err(format!("Invalid number: {:?}", n))
            }
        }
        YamlValue::String(s) => {
            // Check for expression pattern $( ... )
            if s.starts_with("$(") && s.ends_with(')') {
                let expr =
                    parse_expression_token(s).map_err(|e| format!("Invalid expression: {}", e))?;
                return Ok(Item::Scalar(Value::Expression(expr)));
            }
            // Note: We no longer auto-convert @... strings to references here.
            // References are now encoded as mappings with @ref key (like JSON).
            // This allows strings that happen to start with @ to round-trip correctly.
            Ok(Item::Scalar(Value::String(s.clone())))
        }
        YamlValue::Sequence(seq) => {
            // Check array length limit
            if seq.len() > config.max_array_length {
                return Err(YamlError::ArrayTooLong {
                    length: seq.len(),
                    max_length: config.max_array_length,
                    path: key.to_string(),
                }
                .to_string());
            }

            // Handle empty sequences as empty matrix lists
            if seq.is_empty() {
                let type_name = singularize_and_capitalize(key);
                let list = MatrixList::new(type_name.clone(), vec!["id".to_string()]);
                structs.insert(type_name, vec!["id".to_string()]);
                Ok(Item::List(list))
            } else if is_tensor_sequence(seq) {
                // Check if it's a tensor (array of numbers)
                let tensor = yaml_sequence_to_tensor(seq, config, key, depth)?;
                Ok(Item::Scalar(Value::Tensor(tensor)))
            } else if is_object_sequence(seq) {
                // Convert to matrix list
                let list = yaml_sequence_to_matrix_list(seq, key, config, structs, depth)?;
                Ok(Item::List(list))
            } else {
                // Mixed array - try to convert to tensor
                let tensor = yaml_sequence_to_tensor(seq, config, key, depth)?;
                Ok(Item::Scalar(Value::Tensor(tensor)))
            }
        }
        YamlValue::Mapping(map) => {
            // Check for reference marker (@ref key)
            if let Some(YamlValue::String(ref_str)) = map.get(YamlValue::String("@ref".to_string()))
            {
                return Ok(Item::Scalar(Value::Reference(parse_reference(ref_str)?)));
            }
            // Check for special metadata indicating a matrix list
            if map.contains_key(YamlValue::String("items".to_string())) {
                // Structured matrix list with metadata (__type__, __schema__, items)
                let items = map
                    .get(YamlValue::String("items".to_string()))
                    .ok_or("Missing items array")?;
                if let YamlValue::Sequence(seq) = items {
                    // Check array length
                    if seq.len() > config.max_array_length {
                        return Err(YamlError::ArrayTooLong {
                            length: seq.len(),
                            max_length: config.max_array_length,
                            path: format!("{}.items", key),
                        }
                        .to_string());
                    }

                    // Extract type_name from wrapper metadata if present
                    let mut list = yaml_sequence_to_matrix_list(
                        seq, key, config, structs, depth,
                    )?;

                    // Override type_name with wrapper metadata if present
                    if let Some(YamlValue::String(wrapper_type)) =
                        map.get(YamlValue::String("__type__".to_string())) {
                        list.type_name = wrapper_type.clone();
                        // Re-register with correct type name
                        structs.insert(wrapper_type.clone(), list.schema.clone());
                    }

                    return Ok(Item::List(list));
                }
            }
            // Regular object
            let item_map = yaml_mapping_to_item_map(map, config, structs, depth + 1)?;
            Ok(Item::Object(item_map))
        }
        YamlValue::Tagged(tagged) => {
            // Handle YAML tags (anchors/aliases)
            yaml_value_to_item(&tagged.value, key, config, structs, depth)
        }
    }
}

fn is_tensor_sequence(seq: &[YamlValue]) -> bool {
    // Empty sequences are not tensors - they're empty matrix lists
    !seq.is_empty()
        && seq
            .iter()
            .all(|v| matches!(v, YamlValue::Number(_) | YamlValue::Sequence(_)))
}

fn is_object_sequence(seq: &[YamlValue]) -> bool {
    !seq.is_empty() && seq.iter().all(|v| matches!(v, YamlValue::Mapping(_)))
}

/// Infers the row schema from the first mapping in a sequence.
///
/// This function extracts all non-metadata keys from the first mapping,
/// excluding nested sequence fields (which represent children). The resulting
/// schema is sorted alphabetically with "id" always positioned first.
///
/// # Arguments
///
/// * `seq` - The YAML sequence to infer schema from
///
/// # Returns
///
/// A vector of schema column names, or a default schema if the sequence is empty.
///
/// # Examples
///
/// ```text
/// First mapping: { id: "u1", name: "Alice", age: 30 }
/// Result: ["id", "age", "name"]  // sorted, with "id" first
/// ```
fn infer_row_schema(seq: &[YamlValue]) -> Vec<String> {
    if let Some(YamlValue::Mapping(first)) = seq.first() {
        // OPTIMIZATION: Pre-allocate with estimated capacity to reduce reallocations
        let mut keys: Vec<String> = Vec::with_capacity(first.len());

        for (k, v) in first.iter() {
            let key_str = match k.as_str() {
                Some(s) => s,
                None => continue,
            };

            // Skip ONLY known HEDL metadata keys (not all keys starting with __)
            // This allows columns named "__", "___foo", etc. while filtering metadata
            if key_str == "__type__" || key_str == "__schema__" {
                continue;
            }

            // Exclude sequences of mappings - they become children
            if let YamlValue::Sequence(child_seq) = v {
                if is_object_sequence(child_seq) {
                    continue;
                }
            }

            keys.push(key_str.to_string());
        }

        keys.sort_unstable();  // OPTIMIZATION: Use unstable sort (faster for strings)

        // Ensure "id" is first if present
        if let Some(pos) = keys.iter().position(|k| k == "id") {
            keys.remove(pos);
            keys.insert(0, "id".to_string());
        }
        keys
    } else {
        DEFAULT_SCHEMA.iter().map(|s| s.to_string()).collect()
    }
}

/// Extracts field values from a mapping for a given schema.
///
/// This function retrieves all field values in schema order, converting each
/// YAML value to a HEDL Value. Missing fields are represented as `Value::Null`.
///
/// # Arguments
///
/// * `map` - The YAML mapping containing the row data
/// * `schema` - The column schema defining which fields to extract
/// * `config` - Configuration with depth/size limits
/// * `depth` - Current nesting depth
///
/// # Returns
///
/// A vector of HEDL Values in schema column order, or an error if conversion fails.
///
/// # Examples
///
/// ```text
/// Schema: ["id", "name"]
/// Mapping: { id: "u1", name: "Alice" }
/// Result: [Value::String("u1"), Value::String("Alice")]
/// ```
fn build_matrix_columns(
    map: &Mapping,
    schema: &[String],
    config: &FromYamlConfig,
    depth: usize,
) -> Result<Vec<Value>, String> {
    // OPTIMIZATION: Pre-allocate exact capacity needed
    let mut fields = Vec::with_capacity(schema.len());
    for col in schema {
        let value = map
            .get(YamlValue::String(col.clone()))
            .map(|v| yaml_to_value(v, config, depth + 1))
            .transpose()?
            .unwrap_or(Value::Null);
        fields.push(value);
    }
    Ok(fields)
}

/// Extracts the ID value from a mapping using the first schema column.
///
/// The ID is always derived from the first column in the schema. If missing
/// or not a string, an empty string is used as the default ID.
///
/// # Arguments
///
/// * `map` - The YAML mapping containing the row data
/// * `schema` - The column schema (first element is used for ID extraction)
///
/// # Returns
///
/// The ID string, or empty string if not found or not a string type.
///
/// # Examples
///
/// ```text
/// Schema: ["id", "name"]
/// Mapping: { id: "u1", name: "Alice" }
/// Result: "u1"
/// ```
///
/// # Optimization
///
/// Efficiently extracts the ID field from a YAML mapping according to the provided schema.
fn extract_row_id(map: &Mapping, schema: &[String]) -> String {
    map.get(YamlValue::String(schema[0].clone()))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())  // Explicitly convert &str to String
        .unwrap_or_else(String::new)  // Use String::new() instead of "".to_string()
}

/// Processes nested child sequences within a mapping.
///
/// This function identifies and recursively converts sequences of mappings
/// into child matrix lists. The results are stored in a BTreeMap keyed by
/// child field name.
///
/// # Arguments
///
/// * `map` - The YAML mapping that may contain child sequences
/// * `key` - The parent key (for error messages)
/// * `config` - Configuration with depth/size limits
/// * `structs` - Structure registry for new child types
/// * `depth` - Current nesting depth
///
/// # Returns
///
/// A BTreeMap of child matrices, or an error if processing fails.
///
/// # Examples
///
/// ```text
/// Mapping contains: { posts: [{ id: "p1", ... }, { id: "p2", ... }] }
/// Result: { "posts": [node1, node2] }
/// ```
fn process_nested_children(
    map: &Mapping,
    key: &str,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<BTreeMap<String, Vec<Node>>, String> {
    let mut children: BTreeMap<String, Vec<Node>> = BTreeMap::new();

    for (child_key, child_value) in map.iter() {
        if let (Some(child_key_str), YamlValue::Sequence(child_seq)) =
            (child_key.as_str(), child_value)
        {
            if is_object_sequence(child_seq) {
                // Check child array length
                if child_seq.len() > config.max_array_length {
                    return Err(YamlError::ArrayTooLong {
                        length: child_seq.len(),
                        max_length: config.max_array_length,
                        path: format!("{}.{}", key, child_key_str),
                    }
                    .to_string());
                }
                // This is a nested child list
                let child_list = yaml_sequence_to_matrix_list(
                    child_seq,
                    child_key_str,
                    config,
                    structs,
                    depth + 1,
                )?;
                children.insert(child_key_str.to_string(), child_list.rows);
            }
        }
    }

    Ok(children)
}

/// Converts a single mapping in a sequence to a Node.
///
/// This function extracts all necessary data (ID, fields, children) from a
/// mapping and constructs a Node for inclusion in a matrix list.
///
/// # Arguments
///
/// * `map` - The YAML mapping to convert
/// * `type_name` - The HEDL type name for this node
/// * `schema` - The column schema defining fields
/// * `key` - The parent key (for error messages)
/// * `config` - Configuration with depth/size limits
/// * `structs` - Structure registry for child types
/// * `depth` - Current nesting depth
///
/// # Returns
///
/// A Node with all fields and children properly populated, or an error if conversion fails.
fn convert_sequence_item(
    map: &Mapping,
    type_name: &str,
    schema: &[String],
    key: &str,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<Node, String> {
    let id = extract_row_id(map, schema);
    let fields = build_matrix_columns(map, schema, config, depth)?;
    let children = process_nested_children(map, key, config, structs, depth)?;

    Ok(Node {
        type_name: type_name.to_string(),
        id,
        fields,
        children,
        child_count: None,
    })
}

/// Validates row structure and converts all sequence items to Nodes.
///
/// This function processes all mappings in a sequence, validating them and
/// converting to Nodes. It filters out non-mapping items and collects all rows.
///
/// # Arguments
///
/// * `seq` - The YAML sequence to process
/// * `type_name` - The HEDL type name for nodes
/// * `schema` - The column schema defining fields
/// * `key` - The parent key (for error messages)
/// * `config` - Configuration with depth/size limits
/// * `structs` - Structure registry for child types
/// * `depth` - Current nesting depth
///
/// # Returns
///
/// A vector of Nodes, or an error if any row conversion fails.
fn validate_row_structure(
    seq: &[YamlValue],
    type_name: &str,
    schema: &[String],
    key: &str,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<Vec<Node>, String> {
    // OPTIMIZATION: Pre-allocate capacity based on sequence length
    let mut rows = Vec::with_capacity(seq.len());
    for item in seq.iter() {
        if let YamlValue::Mapping(map) = item {
            let node = convert_sequence_item(map, type_name, schema, key, config, structs, depth)?;
            rows.push(node);
        }
    }
    Ok(rows)
}

fn yaml_sequence_to_tensor(
    seq: &[YamlValue],
    config: &FromYamlConfig,
    path: &str,
    depth: usize,
) -> Result<Tensor, String> {
    // Check nesting depth
    if depth > config.max_nesting_depth {
        return Err(YamlError::MaxDepthExceeded {
            max_depth: config.max_nesting_depth,
            actual_depth: depth,
        }
        .to_string());
    }

    // Check array length
    if seq.len() > config.max_array_length {
        return Err(YamlError::ArrayTooLong {
            length: seq.len(),
            max_length: config.max_array_length,
            path: path.to_string(),
        }
        .to_string());
    }

    // Convert YAML sequence to Tensor recursively
    let items: Result<Vec<Tensor>, String> = seq
        .iter()
        .map(|v| match v {
            YamlValue::Number(n) => n
                .as_f64()
                .map(Tensor::Scalar)
                .ok_or_else(|| format!("Invalid tensor number: {:?}", n)),
            YamlValue::Sequence(nested) => yaml_sequence_to_tensor(nested, config, path, depth + 1),
            _ => Err("Invalid tensor element - must be number or sequence".into()),
        })
        .collect();

    Ok(Tensor::Array(items?))
}

/// Converts a YAML sequence to a HEDL MatrixList.
///
/// This function transforms a sequence of YAML mappings into a structured HEDL
/// MatrixList, inferring the schema from the first element. Child sequences are
/// recursively converted to nested matrix lists.
///
/// # How it works
///
/// 1. Validates nesting depth and array length constraints
/// 2. Infers type name from the key (singularized and capitalized)
/// 3. Infers schema from the first mapping, excluding child sequences
/// 4. Converts each mapping to a Node with fields and nested children
/// 5. Returns a MatrixList with all rows
///
/// # Arguments
///
/// * `seq` - The YAML sequence of mappings to convert
/// * `key` - The field name (used for type name inference)
/// * `config` - Configuration with depth/size limits
/// * `structs` - Structure registry (updated with new type definitions)
/// * `depth` - Current nesting depth for validation
///
/// # Examples
///
/// ```text
/// YAML:
/// users:
///   - id: u1
///     name: Alice
///   - id: u2
///     name: Bob
///
/// Result: MatrixList {
///     type_name: "User",
///     schema: ["id", "name"],
///     rows: [Node{id: "u1", ...}, Node{id: "u2", ...}]
/// }
/// ```
#[allow(clippy::only_used_in_recursion)]
fn yaml_sequence_to_matrix_list(
    seq: &[YamlValue],
    key: &str,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<MatrixList, String> {
    // Validate nesting depth and array length
    if depth > config.max_nesting_depth {
        return Err(YamlError::MaxDepthExceeded {
            max_depth: config.max_nesting_depth,
            actual_depth: depth,
        }
        .to_string());
    }
    if seq.len() > config.max_array_length {
        return Err(YamlError::ArrayTooLong {
            length: seq.len(),
            max_length: config.max_array_length,
            path: key.to_string(),
        }
        .to_string());
    }

    // Try to extract type_name from metadata (__type__ field in first row)
    // This preserves type_name during YAML roundtrip when include_metadata is used
    let type_name = if let Some(YamlValue::Mapping(first_map)) = seq.first() {
        if let Some(YamlValue::String(type_str)) = first_map.get(YamlValue::String("__type__".to_string())) {
            type_str.clone()
        } else {
            // Fallback to inferring from key
            singularize_and_capitalize(key)
        }
    } else {
        // Empty sequence or non-mapping first element - infer from key
        singularize_and_capitalize(key)
    };

    let schema = infer_row_schema(seq);

    // Register the struct definition
    structs.insert(type_name.clone(), schema.clone());

    // Process all rows and collect
    let rows = validate_row_structure(seq, &type_name, &schema, key, config, structs, depth)?;

    Ok(MatrixList {
        type_name,
        schema,
        rows,
        count_hint: None,
    })
}

fn yaml_to_value(
    value: &YamlValue,
    config: &FromYamlConfig,
    depth: usize,
) -> Result<Value, String> {
    // Check nesting depth
    if depth > config.max_nesting_depth {
        return Err(YamlError::MaxDepthExceeded {
            max_depth: config.max_nesting_depth,
            actual_depth: depth,
        }
        .to_string());
    }

    Ok(match value {
        YamlValue::Null => Value::Null,
        YamlValue::Bool(b) => Value::Bool(*b),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                return Err(format!("Invalid number: {:?}", n));
            }
        }
        YamlValue::String(s) => {
            // Check for expression pattern $( ... )
            if s.starts_with("$(") && s.ends_with(')') {
                let expr =
                    parse_expression_token(s).map_err(|e| format!("Invalid expression: {}", e))?;
                Value::Expression(expr)
            } else {
                // Note: Strings that start with @ are just strings.
                // References use the @ref mapping format.
                Value::String(s.clone())
            }
        }
        YamlValue::Sequence(seq) => {
            // Check array length
            if seq.len() > config.max_array_length {
                return Err(YamlError::ArrayTooLong {
                    length: seq.len(),
                    max_length: config.max_array_length,
                    path: "value".to_string(),
                }
                .to_string());
            }

            // Check if this is a sequence of mappings (nested children) - skip as Null
            // Child sequences are handled separately in yaml_sequence_to_matrix_list
            if is_object_sequence(seq) {
                Value::Null // Children processed by yaml_sequence_to_matrix_list
            } else if is_tensor_sequence(seq) {
                let tensor = yaml_sequence_to_tensor(seq, config, "tensor", depth + 1)?;
                Value::Tensor(tensor)
            } else if seq.is_empty() {
                // Empty sequence → empty tensor
                Value::Tensor(Tensor::Array(vec![]))
            } else {
                // Mixed sequence - try as tensor
                let tensor = yaml_sequence_to_tensor(seq, config, "tensor", depth + 1)?;
                Value::Tensor(tensor)
            }
        }
        YamlValue::Mapping(map) => {
            // Check for reference marker (@ref key)
            if let Some(YamlValue::String(ref_str)) = map.get(YamlValue::String("@ref".to_string()))
            {
                Value::Reference(parse_reference(ref_str)?)
            } else {
                return Err("Nested objects not allowed in scalar context".into());
            }
        }
        YamlValue::Tagged(tagged) => {
            return yaml_to_value(&tagged.value, config, depth);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== FromYamlConfig tests ====================

    #[test]
    fn test_from_yaml_config_default() {
        let config = FromYamlConfig::default();
        assert_eq!(config.default_type_name, "Item");
        assert_eq!(config.version, (1, 0));
    }

    #[test]
    fn test_from_yaml_config_debug() {
        let config = FromYamlConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("FromYamlConfig"));
        assert!(debug.contains("default_type_name"));
        assert!(debug.contains("version"));
    }

    #[test]
    fn test_from_yaml_config_clone() {
        let config = FromYamlConfig {
            default_type_name: "Custom".to_string(),
            version: (2, 1),
            ..Default::default()
        };
        let cloned = config.clone();
        assert_eq!(cloned.default_type_name, "Custom");
        assert_eq!(cloned.version, (2, 1));
    }

    #[test]
    fn test_from_yaml_config_custom() {
        let config = FromYamlConfig {
            default_type_name: "MyType".to_string(),
            version: (3, 0),
            ..Default::default()
        };
        assert_eq!(config.default_type_name, "MyType");
        assert_eq!(config.version, (3, 0));
    }

    // ==================== parse_reference tests ====================

    #[test]
    fn test_parse_reference_local() {
        let local_ref = parse_reference("@user1").unwrap();
        assert_eq!(local_ref.type_name, None);
        assert_eq!(local_ref.id, "user1");
    }

    #[test]
    fn test_parse_reference_qualified() {
        let qual_ref = parse_reference("@User:user1").unwrap();
        assert_eq!(qual_ref.type_name, Some("User".to_string()));
        assert_eq!(qual_ref.id, "user1");
    }

    #[test]
    fn test_parse_reference_invalid_no_at() {
        let result = parse_reference("user1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid reference format"));
    }

    #[test]
    fn test_parse_reference_with_special_chars() {
        let ref_val = parse_reference("@my-item_123").unwrap();
        assert_eq!(ref_val.type_name, None);
        assert_eq!(ref_val.id, "my-item_123");
    }

    #[test]
    fn test_parse_reference_qualified_with_dashes() {
        let ref_val = parse_reference("@My-Type:item-123").unwrap();
        assert_eq!(ref_val.type_name, Some("My-Type".to_string()));
        assert_eq!(ref_val.id, "item-123");
    }

    #[test]
    fn test_parse_reference_empty_id() {
        // @: is parsed as type "" and id ""
        let ref_val = parse_reference("@:").unwrap();
        assert_eq!(ref_val.type_name, Some("".to_string()));
        assert_eq!(ref_val.id, "");
    }

    // ==================== is_tensor_sequence tests ====================

    #[test]
    fn test_is_tensor_sequence_numbers() {
        let numbers = vec![
            YamlValue::Number(1.into()),
            YamlValue::Number(2.into()),
            YamlValue::Number(3.into()),
        ];
        assert!(is_tensor_sequence(&numbers));
    }

    #[test]
    fn test_is_tensor_sequence_nested() {
        let nested = vec![
            YamlValue::Sequence(vec![YamlValue::Number(1.into())]),
            YamlValue::Sequence(vec![YamlValue::Number(2.into())]),
        ];
        assert!(is_tensor_sequence(&nested));
    }

    #[test]
    fn test_is_tensor_sequence_mixed_numbers_and_nested() {
        let mixed = vec![
            YamlValue::Number(1.into()),
            YamlValue::Sequence(vec![YamlValue::Number(2.into())]),
        ];
        assert!(is_tensor_sequence(&mixed));
    }

    #[test]
    fn test_is_tensor_sequence_with_strings() {
        let mixed = vec![
            YamlValue::Number(1.into()),
            YamlValue::String("test".to_string()),
        ];
        assert!(!is_tensor_sequence(&mixed));
    }

    #[test]
    fn test_is_tensor_sequence_empty() {
        let empty: Vec<YamlValue> = vec![];
        assert!(!is_tensor_sequence(&empty));
    }

    #[test]
    fn test_is_tensor_sequence_all_strings() {
        let strings = vec![
            YamlValue::String("a".to_string()),
            YamlValue::String("b".to_string()),
        ];
        assert!(!is_tensor_sequence(&strings));
    }

    #[test]
    fn test_is_tensor_sequence_with_mappings() {
        let with_mapping = vec![
            YamlValue::Number(1.into()),
            YamlValue::Mapping(Mapping::new()),
        ];
        assert!(!is_tensor_sequence(&with_mapping));
    }

    // ==================== is_object_sequence tests ====================

    #[test]
    fn test_is_object_sequence_mappings() {
        let objects = vec![
            YamlValue::Mapping(Mapping::new()),
            YamlValue::Mapping(Mapping::new()),
        ];
        assert!(is_object_sequence(&objects));
    }

    #[test]
    fn test_is_object_sequence_mixed() {
        let mixed = vec![
            YamlValue::Mapping(Mapping::new()),
            YamlValue::Number(1.into()),
        ];
        assert!(!is_object_sequence(&mixed));
    }

    #[test]
    fn test_is_object_sequence_empty() {
        let empty: Vec<YamlValue> = vec![];
        assert!(!is_object_sequence(&empty));
    }

    #[test]
    fn test_is_object_sequence_all_numbers() {
        let numbers = vec![YamlValue::Number(1.into()), YamlValue::Number(2.into())];
        assert!(!is_object_sequence(&numbers));
    }

    #[test]
    fn test_is_object_sequence_with_nested_sequences() {
        let mixed = vec![
            YamlValue::Mapping(Mapping::new()),
            YamlValue::Sequence(vec![]),
        ];
        assert!(!is_object_sequence(&mixed));
    }

    // ==================== yaml_value_to_item tests ====================

    #[test]
    fn test_yaml_value_to_item_null() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(&YamlValue::Null, "test", &config, &mut structs, 0).unwrap();
        assert_eq!(item, Item::Scalar(Value::Null));
    }

    #[test]
    fn test_yaml_value_to_item_bool_true() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item =
            yaml_value_to_item(&YamlValue::Bool(true), "test", &config, &mut structs, 0).unwrap();
        assert_eq!(item, Item::Scalar(Value::Bool(true)));
    }

    #[test]
    fn test_yaml_value_to_item_bool_false() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item =
            yaml_value_to_item(&YamlValue::Bool(false), "test", &config, &mut structs, 0).unwrap();
        assert_eq!(item, Item::Scalar(Value::Bool(false)));
    }

    #[test]
    fn test_yaml_value_to_item_int() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::Number(42.into()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        assert_eq!(item, Item::Scalar(Value::Int(42)));
    }

    #[test]
    fn test_yaml_value_to_item_int_negative() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::Number((-100).into()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        assert_eq!(item, Item::Scalar(Value::Int(-100)));
    }

    #[test]
    fn test_yaml_value_to_item_float() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let yaml_num = YamlValue::Number(serde_yaml::Number::from(3.5));
        let item = yaml_value_to_item(&yaml_num, "test", &config, &mut structs, 0).unwrap();
        if let Item::Scalar(Value::Float(f)) = item {
            assert!((f - 3.5).abs() < 0.001);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_yaml_value_to_item_string() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::String("hello".to_string()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        assert_eq!(item, Item::Scalar(Value::String("hello".to_string())));
    }

    #[test]
    fn test_yaml_value_to_item_string_empty() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::String("".to_string()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        assert_eq!(item, Item::Scalar(Value::String("".to_string())));
    }

    #[test]
    fn test_yaml_value_to_item_string_with_at() {
        // Strings starting with @ are just strings, not references
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::String("@not-a-ref".to_string()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        if let Item::Scalar(Value::String(s)) = item {
            assert_eq!(s, "@not-a-ref");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_yaml_value_to_item_expression() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::String("$(add(x, 1))".to_string()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        if let Item::Scalar(Value::Expression(e)) = item {
            assert_eq!(e.to_string(), "add(x, 1)");
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_yaml_value_to_item_expression_identifier() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::String("$(foo)".to_string()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        if let Item::Scalar(Value::Expression(e)) = item {
            assert_eq!(e.to_string(), "foo");
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_yaml_value_to_item_reference_local() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut ref_map = Mapping::new();
        ref_map.insert(
            YamlValue::String("@ref".to_string()),
            YamlValue::String("@user1".to_string()),
        );
        let item = yaml_value_to_item(
            &YamlValue::Mapping(ref_map),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        if let Item::Scalar(Value::Reference(r)) = item {
            assert_eq!(r.type_name, None);
            assert_eq!(r.id, "user1");
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_yaml_value_to_item_reference_qualified() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut ref_map = Mapping::new();
        ref_map.insert(
            YamlValue::String("@ref".to_string()),
            YamlValue::String("@User:user1".to_string()),
        );
        let item = yaml_value_to_item(
            &YamlValue::Mapping(ref_map),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        if let Item::Scalar(Value::Reference(r)) = item {
            assert_eq!(r.type_name, Some("User".to_string()));
            assert_eq!(r.id, "user1");
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_yaml_value_to_item_tensor_1d() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let seq = YamlValue::Sequence(vec![
            YamlValue::Number(1.into()),
            YamlValue::Number(2.into()),
            YamlValue::Number(3.into()),
        ]);
        let item = yaml_value_to_item(&seq, "test", &config, &mut structs, 0).unwrap();
        if let Item::Scalar(Value::Tensor(Tensor::Array(arr))) = item {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected tensor");
        }
    }

    #[test]
    fn test_yaml_value_to_item_empty_sequence() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let seq = YamlValue::Sequence(vec![]);
        let item = yaml_value_to_item(&seq, "items", &config, &mut structs, 0).unwrap();
        // Empty sequences become empty matrix lists
        if let Item::List(list) = item {
            assert!(list.rows.is_empty());
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_yaml_value_to_item_object_sequence() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut obj1 = Mapping::new();
        obj1.insert(
            YamlValue::String("id".to_string()),
            YamlValue::String("u1".to_string()),
        );
        obj1.insert(
            YamlValue::String("name".to_string()),
            YamlValue::String("Alice".to_string()),
        );

        let seq = YamlValue::Sequence(vec![YamlValue::Mapping(obj1)]);
        let item = yaml_value_to_item(&seq, "users", &config, &mut structs, 0).unwrap();
        if let Item::List(list) = item {
            assert_eq!(list.rows.len(), 1);
            assert_eq!(list.type_name, "User");
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_yaml_value_to_item_simple_object() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut obj = Mapping::new();
        obj.insert(
            YamlValue::String("name".to_string()),
            YamlValue::String("test".to_string()),
        );
        obj.insert(
            YamlValue::String("age".to_string()),
            YamlValue::Number(42.into()),
        );

        let item =
            yaml_value_to_item(&YamlValue::Mapping(obj), "test", &config, &mut structs, 0).unwrap();
        if let Item::Object(map) = item {
            assert_eq!(map.len(), 2);
            assert!(map.contains_key("name"));
            assert!(map.contains_key("age"));
        } else {
            panic!("Expected object");
        }
    }

    // ==================== yaml_to_value tests ====================

    #[test]
    fn test_yaml_to_value_null() {
        let value = yaml_to_value(&YamlValue::Null, &FromYamlConfig::default(), 0).unwrap();
        assert_eq!(value, Value::Null);
    }

    #[test]
    fn test_yaml_to_value_bool() {
        assert_eq!(
            yaml_to_value(&YamlValue::Bool(true), &FromYamlConfig::default(), 0).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            yaml_to_value(&YamlValue::Bool(false), &FromYamlConfig::default(), 0).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_yaml_to_value_int() {
        let value =
            yaml_to_value(&YamlValue::Number(42.into()), &FromYamlConfig::default(), 0).unwrap();
        assert_eq!(value, Value::Int(42));
    }

    #[test]
    fn test_yaml_to_value_float() {
        let yaml_num = YamlValue::Number(serde_yaml::Number::from(3.5));
        let value = yaml_to_value(&yaml_num, &FromYamlConfig::default(), 0).unwrap();
        if let Value::Float(f) = value {
            assert!((f - 3.5).abs() < 0.001);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_yaml_to_value_string() {
        let value = yaml_to_value(
            &YamlValue::String("hello".to_string()),
            &FromYamlConfig::default(),
            0,
        )
        .unwrap();
        assert_eq!(value, Value::String("hello".to_string()));
    }

    #[test]
    fn test_yaml_to_value_expression() {
        let value = yaml_to_value(
            &YamlValue::String("$(foo)".to_string()),
            &FromYamlConfig::default(),
            0,
        )
        .unwrap();
        if let Value::Expression(e) = value {
            assert_eq!(e.to_string(), "foo");
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_yaml_to_value_reference() {
        let mut ref_map = Mapping::new();
        ref_map.insert(
            YamlValue::String("@ref".to_string()),
            YamlValue::String("@user1".to_string()),
        );
        let value =
            yaml_to_value(&YamlValue::Mapping(ref_map), &FromYamlConfig::default(), 0).unwrap();
        if let Value::Reference(r) = value {
            assert_eq!(r.id, "user1");
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_yaml_to_value_tensor() {
        let seq = YamlValue::Sequence(vec![
            YamlValue::Number(1.into()),
            YamlValue::Number(2.into()),
        ]);
        let value = yaml_to_value(&seq, &FromYamlConfig::default(), 0).unwrap();
        if let Value::Tensor(Tensor::Array(arr)) = value {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected tensor");
        }
    }

    #[test]
    fn test_yaml_to_value_empty_sequence() {
        let seq = YamlValue::Sequence(vec![]);
        let value = yaml_to_value(&seq, &FromYamlConfig::default(), 0).unwrap();
        if let Value::Tensor(Tensor::Array(arr)) = value {
            assert!(arr.is_empty());
        } else {
            panic!("Expected empty tensor");
        }
    }

    #[test]
    fn test_yaml_to_value_nested_object_error() {
        // Regular nested objects are not allowed in scalar context
        let mut obj = Mapping::new();
        obj.insert(
            YamlValue::String("nested".to_string()),
            YamlValue::String("value".to_string()),
        );
        let result = yaml_to_value(&YamlValue::Mapping(obj), &FromYamlConfig::default(), 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Nested objects not allowed"));
    }

    // ==================== yaml_sequence_to_tensor tests ====================

    #[test]
    fn test_yaml_sequence_to_tensor_1d() {
        let seq = vec![
            YamlValue::Number(1.into()),
            YamlValue::Number(2.into()),
            YamlValue::Number(3.into()),
        ];
        let tensor = yaml_sequence_to_tensor(&seq, &FromYamlConfig::default(), "test", 0).unwrap();
        if let Tensor::Array(arr) = tensor {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Tensor::Scalar(1.0));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_yaml_sequence_to_tensor_2d() {
        let seq = vec![
            YamlValue::Sequence(vec![
                YamlValue::Number(1.into()),
                YamlValue::Number(2.into()),
            ]),
            YamlValue::Sequence(vec![
                YamlValue::Number(3.into()),
                YamlValue::Number(4.into()),
            ]),
        ];
        let tensor = yaml_sequence_to_tensor(&seq, &FromYamlConfig::default(), "test", 0).unwrap();
        if let Tensor::Array(outer) = tensor {
            assert_eq!(outer.len(), 2);
            if let Tensor::Array(inner) = &outer[0] {
                assert_eq!(inner.len(), 2);
            } else {
                panic!("Expected nested array");
            }
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_yaml_sequence_to_tensor_empty() {
        let seq: Vec<YamlValue> = vec![];
        let tensor = yaml_sequence_to_tensor(&seq, &FromYamlConfig::default(), "test", 0).unwrap();
        if let Tensor::Array(arr) = tensor {
            assert!(arr.is_empty());
        } else {
            panic!("Expected empty array");
        }
    }

    #[test]
    fn test_yaml_sequence_to_tensor_invalid_element() {
        let seq = vec![
            YamlValue::Number(1.into()),
            YamlValue::String("invalid".to_string()),
        ];
        let result = yaml_sequence_to_tensor(&seq, &FromYamlConfig::default(), "test", 0);
        assert!(result.is_err());
    }

    // ==================== yaml_sequence_to_matrix_list tests ====================

    #[test]
    fn test_yaml_sequence_to_matrix_list_simple() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut obj = Mapping::new();
        obj.insert(
            YamlValue::String("id".to_string()),
            YamlValue::String("u1".to_string()),
        );
        obj.insert(
            YamlValue::String("name".to_string()),
            YamlValue::String("Alice".to_string()),
        );

        let seq = vec![YamlValue::Mapping(obj)];
        let list = yaml_sequence_to_matrix_list(&seq, "users", &config, &mut structs, 0).unwrap();

        assert_eq!(list.type_name, "User");
        assert_eq!(list.rows.len(), 1);
        assert_eq!(list.rows[0].id, "u1");
    }

    #[test]
    fn test_yaml_sequence_to_matrix_list_schema_inference() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut obj = Mapping::new();
        obj.insert(
            YamlValue::String("id".to_string()),
            YamlValue::String("u1".to_string()),
        );
        obj.insert(
            YamlValue::String("name".to_string()),
            YamlValue::String("Alice".to_string()),
        );
        obj.insert(
            YamlValue::String("age".to_string()),
            YamlValue::Number(30.into()),
        );

        let seq = vec![YamlValue::Mapping(obj)];
        let list = yaml_sequence_to_matrix_list(&seq, "users", &config, &mut structs, 0).unwrap();

        // Schema should be sorted with id first
        assert_eq!(list.schema[0], "id");
        assert!(list.schema.contains(&"name".to_string()));
        assert!(list.schema.contains(&"age".to_string()));
    }

    #[test]
    fn test_yaml_sequence_to_matrix_list_empty() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let seq: Vec<YamlValue> = vec![];
        let list = yaml_sequence_to_matrix_list(&seq, "users", &config, &mut structs, 0).unwrap();

        assert_eq!(list.type_name, "User");
        assert!(list.rows.is_empty());
        // Default schema for empty list
        assert!(list.schema.contains(&"id".to_string()));
    }

    #[test]
    fn test_yaml_sequence_to_matrix_list_type_name_singularization() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut obj = Mapping::new();
        obj.insert(
            YamlValue::String("id".to_string()),
            YamlValue::String("1".to_string()),
        );

        let seq = vec![YamlValue::Mapping(obj)];

        // Test various pluralizations
        let list = yaml_sequence_to_matrix_list(&seq, "users", &config, &mut structs, 0).unwrap();
        assert_eq!(list.type_name, "User");

        let list =
            yaml_sequence_to_matrix_list(&seq, "companies", &config, &mut structs, 0).unwrap();
        assert_eq!(list.type_name, "Company");

        // "people" uses standard singularization (just removes 's' and capitalizes)
        let list = yaml_sequence_to_matrix_list(&seq, "people", &config, &mut structs, 0).unwrap();
        assert_eq!(list.type_name, "People");

        let list = yaml_sequence_to_matrix_list(&seq, "items", &config, &mut structs, 0).unwrap();
        assert_eq!(list.type_name, "Item");
    }

    // ==================== from_yaml integration tests ====================

    #[test]
    fn test_from_yaml_simple() {
        let yaml = "name: test\ncount: 42\n";
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        assert_eq!(doc.version, (1, 0));
        assert_eq!(doc.root.len(), 2);
    }

    #[test]
    fn test_from_yaml_invalid() {
        let yaml = "{ invalid yaml: [";
        let config = FromYamlConfig::default();
        let result = from_yaml(yaml, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("YAML parse error"));
    }

    #[test]
    fn test_from_yaml_non_mapping_root() {
        let yaml = "- item1\n- item2\n";
        let config = FromYamlConfig::default();
        let result = from_yaml(yaml, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Root must be a YAML mapping"));
    }

    #[test]
    fn test_from_yaml_with_list() {
        let yaml = r#"
users:
  - id: u1
    name: Alice
  - id: u2
    name: Bob
"#;
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        if let Item::List(list) = &doc.root["users"] {
            assert_eq!(list.rows.len(), 2);
            assert_eq!(list.type_name, "User");
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_from_yaml_with_nested_object() {
        let yaml = r#"
config:
  server:
    host: localhost
    port: 8080
"#;
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        if let Item::Object(config_obj) = &doc.root["config"] {
            if let Item::Object(server) = &config_obj["server"] {
                assert!(server.contains_key("host"));
                assert!(server.contains_key("port"));
            } else {
                panic!("Expected server object");
            }
        } else {
            panic!("Expected config object");
        }
    }

    #[test]
    fn test_from_yaml_with_tensor() {
        let yaml = r#"
matrix:
  - [1, 2, 3]
  - [4, 5, 6]
"#;
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        if let Item::Scalar(Value::Tensor(Tensor::Array(outer))) = &doc.root["matrix"] {
            assert_eq!(outer.len(), 2);
        } else {
            panic!("Expected tensor");
        }
    }

    #[test]
    fn test_from_yaml_skips_metadata_keys() {
        let yaml = r#"
__type__: "MyType"
__schema__: ["id", "name"]
name: test
__other__: notskipped
"#;
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        // Only KNOWN HEDL metadata keys (__type__, __schema__) are skipped
        assert!(!doc.root.contains_key("__type__"));
        assert!(!doc.root.contains_key("__schema__"));
        // Regular keys and other __ keys are NOT skipped
        assert!(doc.root.contains_key("name"));
        assert!(doc.root.contains_key("__other__")); // Other __ keys are preserved
    }

    #[test]
    fn test_from_yaml_custom_version() {
        let yaml = "name: test\n";
        let config = FromYamlConfig {
            default_type_name: "Item".to_string(),
            version: (2, 5),
            ..Default::default()
        };
        let doc = from_yaml(yaml, &config).unwrap();
        assert_eq!(doc.version, (2, 5));
    }

    // ==================== from_yaml_value tests ====================

    #[test]
    fn test_from_yaml_value_mapping() {
        let mut map = Mapping::new();
        map.insert(
            YamlValue::String("key".to_string()),
            YamlValue::String("value".to_string()),
        );

        let config = FromYamlConfig::default();
        let doc = from_yaml_value(&YamlValue::Mapping(map), &config).unwrap();

        assert_eq!(doc.root.len(), 1);
        assert!(doc.root.contains_key("key"));
    }

    #[test]
    fn test_from_yaml_value_non_mapping() {
        let config = FromYamlConfig::default();
        let result = from_yaml_value(&YamlValue::Number(42.into()), &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Root must be a YAML mapping"));
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_yaml_unicode_keys_and_values() {
        let yaml = "名前: テスト\nцена: 100\n";
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        assert!(doc.root.contains_key("名前"));
        assert!(doc.root.contains_key("цена"));
    }

    #[test]
    fn test_yaml_multiline_string() {
        let yaml = r#"
description: |
  This is a
  multiline string
"#;
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        if let Item::Scalar(Value::String(s)) = &doc.root["description"] {
            assert!(s.contains('\n'));
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_yaml_anchors_and_aliases() {
        // Simple anchor/alias reference (not merge key)
        let yaml = r#"
defaults: &defaults
  timeout: 30
  retries: 3
production:
  config: *defaults
  host: prod.example.com
"#;
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        // The alias reference should be resolved as nested object
        if let Item::Object(prod) = &doc.root["production"] {
            assert!(prod.contains_key("config"));
            assert!(prod.contains_key("host"));
            // config should be an object with timeout and retries
            if let Item::Object(config_obj) = &prod["config"] {
                assert!(config_obj.contains_key("timeout"));
                assert!(config_obj.contains_key("retries"));
            } else {
                panic!("Expected config object");
            }
        } else {
            panic!("Expected object");
        }
    }

    // ==================== Resource Limit Tests (DoS Protection) ====================

    #[test]
    fn test_max_document_size_exceeded() {
        // Create a document larger than the limit
        let config = FromYamlConfig {
            max_document_size: 100, // Very small limit for testing
            ..Default::default()
        };

        let yaml = "a".repeat(200); // 200 bytes, exceeds 100 byte limit
        let result = from_yaml(&yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Document size"));
        assert!(err.contains("exceeds maximum"));
    }

    #[test]
    fn test_max_document_size_within_limit() {
        let config = FromYamlConfig {
            max_document_size: 1000,
            ..Default::default()
        };

        let yaml = "name: test\nvalue: 123\n";
        let result = from_yaml(yaml, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_array_length_exceeded() {
        let config = FromYamlConfig {
            max_array_length: 5, // Very small limit for testing
            ..Default::default()
        };

        // Create YAML with array longer than limit
        let yaml = r#"
numbers:
  - 1
  - 2
  - 3
  - 4
  - 5
  - 6
"#;
        let result = from_yaml(yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Array length"));
        assert!(err.contains("exceeds maximum"));
    }

    #[test]
    fn test_max_array_length_within_limit() {
        let config = FromYamlConfig {
            max_array_length: 10,
            ..Default::default()
        };

        let yaml = r#"
numbers:
  - 1
  - 2
  - 3
"#;
        let result = from_yaml(yaml, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_array_length_exceeded_in_matrix_list() {
        let config = FromYamlConfig {
            max_array_length: 2, // Very small limit
            ..Default::default()
        };

        let yaml = r#"
users:
  - id: u1
    name: Alice
  - id: u2
    name: Bob
  - id: u3
    name: Charlie
"#;
        let result = from_yaml(yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Array length"));
        assert!(err.contains("exceeds maximum"));
    }

    #[test]
    fn test_max_nesting_depth_exceeded() {
        let config = FromYamlConfig {
            max_nesting_depth: 3, // Very shallow for testing
            ..Default::default()
        };

        // Create deeply nested structure
        let yaml = r#"
level1:
  level2:
    level3:
      level4:
        level5: value
"#;
        let result = from_yaml(yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Maximum nesting depth"));
        assert!(err.contains("exceeded"));
    }

    #[test]
    fn test_max_nesting_depth_within_limit() {
        let config = FromYamlConfig {
            max_nesting_depth: 10,
            ..Default::default()
        };

        let yaml = r#"
level1:
  level2:
    level3: value
"#;
        let result = from_yaml(yaml, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_nesting_depth_exceeded_in_tensor() {
        let config = FromYamlConfig {
            max_nesting_depth: 2, // Very shallow
            ..Default::default()
        };

        // Nested tensor that's too deep
        let yaml = r#"
matrix:
  - - - [1, 2]
"#;
        let result = from_yaml(yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Maximum nesting depth"));
    }

    #[test]
    fn test_default_limits_are_reasonable() {
        let config = FromYamlConfig::default();

        // Verify default limits are set to high values
        assert_eq!(config.max_document_size, 500 * 1024 * 1024); // 500MB
        assert_eq!(config.max_array_length, 10_000_000); // 10 million
        assert_eq!(config.max_nesting_depth, 10_000); // 10,000 levels

        // Verify constants match defaults
        assert_eq!(config.max_document_size, DEFAULT_MAX_DOCUMENT_SIZE);
        assert_eq!(config.max_array_length, DEFAULT_MAX_ARRAY_LENGTH);
        assert_eq!(config.max_nesting_depth, DEFAULT_MAX_NESTING_DEPTH);
    }

    #[test]
    fn test_custom_limits_configuration() {
        let config = FromYamlConfig {
            default_type_name: "Custom".to_string(),
            version: (2, 0),
            max_document_size: 50_000_000,
            max_array_length: 500_000,
            max_nesting_depth: 500,
        };

        assert_eq!(config.max_document_size, 50_000_000);
        assert_eq!(config.max_array_length, 500_000);
        assert_eq!(config.max_nesting_depth, 500);
    }

    #[test]
    fn test_nested_children_array_length_limit() {
        let config = FromYamlConfig {
            max_array_length: 2,
            ..Default::default()
        };

        let yaml = r#"
users:
  - id: u1
    name: Alice
    posts:
      - id: p1
        title: First
      - id: p2
        title: Second
      - id: p3
        title: Third
"#;
        let result = from_yaml(yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Array length"));
        assert!(err.contains("exceeds maximum"));
    }

    #[test]
    fn test_tensor_array_length_limit() {
        let config = FromYamlConfig {
            max_array_length: 3,
            ..Default::default()
        };

        let yaml = r#"
matrix:
  - [1, 2, 3, 4, 5]
"#;
        let result = from_yaml(yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Array length"));
    }

    #[test]
    fn test_zero_limits_blocks_everything() {
        let config = FromYamlConfig {
            max_document_size: 0,
            max_array_length: 0,
            max_nesting_depth: 0,
            ..Default::default()
        };

        let yaml = "name: test\n";
        let result = from_yaml(yaml, &config);

        // Should fail on document size
        assert!(result.is_err());
    }

    #[test]
    fn test_large_valid_document_within_limits() {
        let config = FromYamlConfig::default();

        // Create a reasonably large document that's still within limits
        let mut items = Vec::new();
        for i in 0..1000 {
            items.push(format!("  - id: item{}\n    value: {}", i, i * 2));
        }
        let yaml = format!("items:\n{}", items.join("\n"));

        let result = from_yaml(&yaml, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_combined_limits_enforcement() {
        let config = FromYamlConfig {
            max_document_size: 500,
            max_array_length: 5,
            max_nesting_depth: 3,
            ..Default::default()
        };

        // Test each limit independently

        // Document size exceeded
        let large_doc = "a".repeat(600);
        assert!(from_yaml(&large_doc, &config).is_err());

        // Array length exceeded
        let long_array = r#"
items:
  - 1
  - 2
  - 3
  - 4
  - 5
  - 6
"#;
        assert!(from_yaml(long_array, &config).is_err());

        // Nesting depth exceeded
        let deep_nesting = r#"
a:
  b:
    c:
      d:
        e: value
"#;
        assert!(from_yaml(deep_nesting, &config).is_err());
    }

    // ==================== FromYamlConfigBuilder tests ====================

    #[test]
    fn test_builder_default() {
        let config = FromYamlConfigBuilder::new().build();
        assert_eq!(config.default_type_name, "Item");
        assert_eq!(config.version, (1, 0));
        assert_eq!(config.max_document_size, DEFAULT_MAX_DOCUMENT_SIZE);
        assert_eq!(config.max_array_length, DEFAULT_MAX_ARRAY_LENGTH);
        assert_eq!(config.max_nesting_depth, DEFAULT_MAX_NESTING_DEPTH);
    }

    #[test]
    fn test_builder_from_default() {
        let config = FromYamlConfigBuilder::default().build();
        assert_eq!(config.default_type_name, "Item");
        assert_eq!(config.version, (1, 0));
        assert_eq!(config.max_document_size, DEFAULT_MAX_DOCUMENT_SIZE);
    }

    #[test]
    fn test_builder_via_config() {
        let config = FromYamlConfig::builder().build();
        assert_eq!(config.default_type_name, "Item");
        assert_eq!(config.version, (1, 0));
        assert_eq!(config.max_document_size, DEFAULT_MAX_DOCUMENT_SIZE);
    }

    #[test]
    fn test_builder_custom_document_size() {
        let config = FromYamlConfig::builder()
            .max_document_size(100 * 1024 * 1024)
            .build();
        assert_eq!(config.max_document_size, 100 * 1024 * 1024);
        // Other values should be defaults
        assert_eq!(config.max_array_length, DEFAULT_MAX_ARRAY_LENGTH);
        assert_eq!(config.max_nesting_depth, DEFAULT_MAX_NESTING_DEPTH);
    }

    #[test]
    fn test_builder_custom_array_length() {
        let config = FromYamlConfig::builder().max_array_length(5_000_000).build();
        assert_eq!(config.max_array_length, 5_000_000);
        // Other values should be defaults
        assert_eq!(config.max_document_size, DEFAULT_MAX_DOCUMENT_SIZE);
        assert_eq!(config.max_nesting_depth, DEFAULT_MAX_NESTING_DEPTH);
    }

    #[test]
    fn test_builder_custom_nesting_depth() {
        let config = FromYamlConfig::builder().max_nesting_depth(5000).build();
        assert_eq!(config.max_nesting_depth, 5000);
        // Other values should be defaults
        assert_eq!(config.max_document_size, DEFAULT_MAX_DOCUMENT_SIZE);
        assert_eq!(config.max_array_length, DEFAULT_MAX_ARRAY_LENGTH);
    }

    #[test]
    fn test_builder_all_custom() {
        let config = FromYamlConfig::builder()
            .default_type_name("Entity")
            .version(2, 0)
            .max_document_size(200 * 1024 * 1024)
            .max_array_length(20_000_000)
            .max_nesting_depth(20_000)
            .build();

        assert_eq!(config.default_type_name, "Entity");
        assert_eq!(config.version, (2, 0));
        assert_eq!(config.max_document_size, 200 * 1024 * 1024);
        assert_eq!(config.max_array_length, 20_000_000);
        assert_eq!(config.max_nesting_depth, 20_000);
    }

    #[test]
    fn test_builder_conservative_limits() {
        // Conservative limits for untrusted input
        let config = FromYamlConfig::builder()
            .max_document_size(10 * 1024 * 1024) // 10 MB
            .max_array_length(100_000)
            .max_nesting_depth(100)
            .build();

        assert_eq!(config.max_document_size, 10 * 1024 * 1024);
        assert_eq!(config.max_array_length, 100_000);
        assert_eq!(config.max_nesting_depth, 100);
    }

    #[test]
    fn test_builder_type_name_from_string() {
        let config = FromYamlConfig::builder()
            .default_type_name("CustomType".to_string())
            .build();
        assert_eq!(config.default_type_name, "CustomType");
    }

    #[test]
    fn test_builder_type_name_from_str() {
        let config = FromYamlConfig::builder()
            .default_type_name("CustomType")
            .build();
        assert_eq!(config.default_type_name, "CustomType");
    }

    #[test]
    fn test_builder_chaining() {
        // Test that builder methods can be chained in any order
        let config1 = FromYamlConfig::builder()
            .max_document_size(100_000_000)
            .max_array_length(1_000_000)
            .max_nesting_depth(1000)
            .build();

        let config2 = FromYamlConfig::builder()
            .max_nesting_depth(1000)
            .max_array_length(1_000_000)
            .max_document_size(100_000_000)
            .build();

        assert_eq!(config1.max_document_size, config2.max_document_size);
        assert_eq!(config1.max_array_length, config2.max_array_length);
        assert_eq!(config1.max_nesting_depth, config2.max_nesting_depth);
    }

    #[test]
    fn test_builder_debug() {
        let builder = FromYamlConfig::builder();
        let debug_str = format!("{:?}", builder);
        assert!(debug_str.contains("FromYamlConfigBuilder"));
    }

    #[test]
    fn test_builder_clone() {
        let builder1 = FromYamlConfig::builder().max_document_size(100_000_000);
        let builder2 = builder1.clone();
        let config1 = builder1.build();
        let config2 = builder2.build();
        assert_eq!(config1.max_document_size, config2.max_document_size);
    }

    #[test]
    fn test_builder_with_yaml_parsing() {
        // Test that builder-configured limits work in actual parsing
        let config = FromYamlConfig::builder()
            .max_document_size(1000)
            .max_array_length(5)
            .build();

        let yaml = r#"
numbers:
  - 1
  - 2
  - 3
"#;
        // Should succeed - within limits
        let result = from_yaml(yaml, &config);
        assert!(result.is_ok());

        // Test exceeding array length
        let yaml_long = r#"
numbers:
  - 1
  - 2
  - 3
  - 4
  - 5
  - 6
"#;
        let result = from_yaml(yaml_long, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_constants_match_defaults() {
        // Verify that the constants have the expected values
        assert_eq!(DEFAULT_MAX_DOCUMENT_SIZE, 500 * 1024 * 1024);
        assert_eq!(DEFAULT_MAX_ARRAY_LENGTH, 10_000_000);
        assert_eq!(DEFAULT_MAX_NESTING_DEPTH, 10_000);
    }
}

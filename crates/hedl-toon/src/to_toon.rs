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

//! HEDL to TOON conversion
//!
//! Implements TOON v3.0 encoding following the spec at <https://github.com/toon-format/spec>
//!
//! This implementation converts directly from HEDL Document structure to TOON,
//! preserving references as `@Type:id` strings (primitives in TOON) rather than
//! going through JSON which would corrupt them into objects.
//!
//! # Architecture
//!
//! The conversion uses a recursive visitor pattern with depth tracking for security:
//!
//! 1. **Depth Limit Protection**: Every recursive call checks depth against [`MAX_NESTING_DEPTH`]
//! 2. **Format Selection**: Automatically chooses tabular vs expanded format based on content
//! 3. **String Safety**: Comprehensive quoting and escaping prevents injection attacks
//! 4. **Float Normalization**: Implements all TOON v3.0 float normalization rules
//!
//! # TOON v3.0 Compliance
//!
//! This implementation follows the TOON v3.0 specification exactly:
//!
//! - Tabular format for arrays of primitives
//! - Expanded format for nested/complex structures
//! - Proper quoting for special characters and delimiters
//! - Float normalization (NaN/Infinity → null, -0 → 0, no trailing zeros)
//! - Reference preservation as `@Type:id` strings
//!
//! # Security
//!
//! - **Stack Overflow Protection**: Depth limit prevents malicious deeply-nested documents
//! - **Injection Prevention**: Comprehensive string escaping and quoting
//! - **Resource Limits**: Bounded recursion prevents DoS attacks

use hedl_core::{Document, Item, MatrixList, Node, Value};
use crate::error::{ToonError, Result, MAX_NESTING_DEPTH};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Irregular English plurals lookup table
///
/// This comprehensive table handles common English irregular plurals where
/// simple "-s" suffixing would be incorrect. The table is lazily initialized
/// and cached for performance.
///
/// # Coverage
///
/// Includes ~30 common irregular plural forms across several categories:
/// - **People/Animals**: person→people, child→children, man→men, woman→women
/// - **Body Parts**: tooth→teeth, foot→feet, goose→geese
/// - **Small Animals**: mouse→mice, louse→lice
/// - **Livestock**: ox→oxen, sheep→sheep (unchanged)
/// - **Marine Life**: fish→fish (can also be fishes)
/// - **Scientific Terms**: phenomenon→phenomena, criterion→criteria
/// - **Classical Plurals**: cactus→cacti, fungus→fungi, nucleus→nuclei
/// - **Unchanged Forms**: deer, moose, species, series, etc.
///
/// # Performance
///
/// - Lazy initialization with `OnceLock` ensures one-time setup cost
/// - HashMap provides O(1) average lookup time
/// - Case-insensitive matching for robustness
///
/// # Examples
///
/// ```text
/// pluralize("child") → "children"
/// pluralize("person") → "people"
/// pluralize("tooth") → "teeth"
/// pluralize("cactus") → "cacti"
/// pluralize("user") → "users" // regular plural
/// ```
fn irregular_plurals() -> &'static HashMap<&'static str, &'static str> {
    static IRREGULAR_PLURALS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

    IRREGULAR_PLURALS.get_or_init(|| {
        let mut map = HashMap::new();

        // People and family
        map.insert("child", "children");
        map.insert("person", "people");
        map.insert("man", "men");
        map.insert("woman", "women");

        // Body parts
        map.insert("foot", "feet");
        map.insert("tooth", "teeth");
        map.insert("goose", "geese");

        // Small animals
        map.insert("mouse", "mice");
        map.insert("louse", "lice");

        // Livestock
        map.insert("ox", "oxen");
        map.insert("sheep", "sheep");
        map.insert("deer", "deer");
        map.insert("moose", "moose");
        map.insert("fish", "fish");

        // Scientific/academic terms
        map.insert("phenomenon", "phenomena");
        map.insert("criterion", "criteria");
        map.insert("datum", "data");
        map.insert("analysis", "analyses");
        map.insert("thesis", "theses");
        map.insert("hypothesis", "hypotheses");
        map.insert("crisis", "crises");

        // Classical/Latin plurals
        map.insert("cactus", "cacti");
        map.insert("fungus", "fungi");
        map.insert("nucleus", "nuclei");
        map.insert("radius", "radii");
        map.insert("alumnus", "alumni");
        map.insert("stimulus", "stimuli");

        // Unchanged forms
        map.insert("species", "species");
        map.insert("series", "series");
        map.insert("aircraft", "aircraft");
        map.insert("spacecraft", "spacecraft");

        map
    })
}

/// Pluralize an English word with support for irregular forms
///
/// This function implements comprehensive English pluralization rules:
/// 1. Checks irregular plural table first (O(1) lookup)
/// 2. Falls back to standard "-s" suffix for regular nouns
///
/// # Arguments
///
/// * `singular` - The singular form of the word
///
/// # Returns
///
/// The pluralized form of the word.
///
/// # Case Handling
///
/// The function preserves the original case:
/// - Lowercase input → lowercase output
/// - Capitalized input → capitalized output
/// - UPPERCASE input → UPPERCASE output
///
/// # Examples
///
/// ```text
/// // Irregular plurals
/// pluralize("child") → "children"
/// pluralize("person") → "people"
/// pluralize("tooth") → "teeth"
/// pluralize("mouse") → "mice"
/// pluralize("cactus") → "cacti"
///
/// // Regular plurals
/// pluralize("user") → "users"
/// pluralize("order") → "orders"
/// pluralize("team") → "teams"
///
/// // Unchanged forms
/// pluralize("sheep") → "sheep"
/// pluralize("deer") → "deer"
/// pluralize("fish") → "fish"
///
/// // Case preservation
/// pluralize("Child") → "Children"
/// pluralize("PERSON") → "PEOPLE"
/// ```
///
/// # Performance
///
/// - O(1) for irregular forms (HashMap lookup)
/// - O(n) for regular forms (string allocation, where n = word length)
/// - Case checking adds negligible overhead
fn pluralize(singular: &str) -> String {
    if singular.is_empty() {
        return String::new();
    }

    // Detect case pattern
    let is_all_upper = singular.chars().all(|c| !c.is_alphabetic() || c.is_uppercase());
    let is_capitalized = singular.chars().next().is_some_and(|c| c.is_uppercase());

    // Check irregular plurals (case-insensitive)
    let lowercase = singular.to_lowercase();
    if let Some(&irregular_plural) = irregular_plurals().get(lowercase.as_str()) {
        // Apply case pattern to irregular plural
        if is_all_upper {
            return irregular_plural.to_uppercase();
        } else if is_capitalized {
            // Capitalize first letter
            let mut chars = irregular_plural.chars();
            if let Some(first) = chars.next() {
                return first.to_uppercase().chain(chars).collect();
            }
        }
        return irregular_plural.to_string();
    }

    // Regular plural: just add 's' (preserve case)
    if is_all_upper {
        format!("{}S", singular)
    } else {
        format!("{}s", singular)
    }
}

/// Configuration for TOON output
///
/// Controls how the TOON format is generated, including indentation and delimiters.
///
/// # Examples
///
/// ```rust
/// use hedl_toon::{ToToonConfig, Delimiter};
///
/// // Use defaults
/// let config = ToToonConfig::default();
/// assert_eq!(config.indent, 2);
/// assert_eq!(config.delimiter, Delimiter::Comma);
///
/// // Custom configuration
/// let config = ToToonConfig {
///     indent: 4,
///     delimiter: Delimiter::Tab,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ToToonConfig {
    /// Number of spaces per indentation level (default: 2)
    ///
    /// Must be at least 1. Values of 2-4 are typical.
    pub indent: usize,

    /// Delimiter to use: comma (default), tab, or pipe
    ///
    /// The delimiter is used in tabular array format to separate field values.
    /// Different delimiters may be more suitable for different use cases:
    /// - Comma: Standard, human-readable
    /// - Tab: Better for TSV-like data
    /// - Pipe: Useful when data contains commas
    pub delimiter: Delimiter,
}

impl Default for ToToonConfig {
    fn default() -> Self {
        Self {
            indent: 2,
            delimiter: Delimiter::Comma,
        }
    }
}

impl ToToonConfig {
    /// Create a new configuration builder with default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_toon::{ToToonConfig, Delimiter};
    ///
    /// let config = ToToonConfig::builder()
    ///     .indent(4)
    ///     .delimiter(Delimiter::Tab)
    ///     .build();
    ///
    /// assert_eq!(config.indent, 4);
    /// assert_eq!(config.delimiter, Delimiter::Tab);
    /// ```
    #[inline]
    pub fn builder() -> ToToonConfigBuilder {
        ToToonConfigBuilder::new()
    }

    /// Validate the configuration
    ///
    /// Returns an error if the configuration is invalid.
    ///
    /// # Errors
    ///
    /// Returns [`ToonError::InvalidIndent`] if indent is 0.
    fn validate(&self) -> Result<()> {
        if self.indent == 0 {
            return Err(ToonError::InvalidIndent(0));
        }
        Ok(())
    }
}

/// Builder for creating ToToonConfig with a fluent API.
///
/// # Examples
///
/// ```
/// use hedl_toon::{ToToonConfig, Delimiter};
///
/// // Use builder pattern
/// let config = ToToonConfig::builder()
///     .indent(4)
///     .delimiter(Delimiter::Pipe)
///     .build();
///
/// assert_eq!(config.indent, 4);
/// assert_eq!(config.delimiter, Delimiter::Pipe);
/// ```
#[derive(Debug, Clone)]
pub struct ToToonConfigBuilder {
    indent: usize,
    delimiter: Delimiter,
}

impl Default for ToToonConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ToToonConfigBuilder {
    /// Create a new builder with default values.
    ///
    /// Defaults:
    /// - `indent`: 2
    /// - `delimiter`: Comma
    #[inline]
    pub fn new() -> Self {
        Self {
            indent: 2,
            delimiter: Delimiter::Comma,
        }
    }

    /// Set the indentation level.
    ///
    /// # Arguments
    ///
    /// * `indent` - Number of spaces per indentation level (must be >= 1)
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_toon::ToToonConfig;
    ///
    /// let config = ToToonConfig::builder()
    ///     .indent(4)
    ///     .build();
    ///
    /// assert_eq!(config.indent, 4);
    /// ```
    #[inline]
    pub fn indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    /// Set the delimiter to use.
    ///
    /// # Arguments
    ///
    /// * `delimiter` - The delimiter type (Comma, Tab, or Pipe)
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_toon::{ToToonConfig, Delimiter};
    ///
    /// let config = ToToonConfig::builder()
    ///     .delimiter(Delimiter::Tab)
    ///     .build();
    ///
    /// assert_eq!(config.delimiter, Delimiter::Tab);
    /// ```
    #[inline]
    pub fn delimiter(mut self, delimiter: Delimiter) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Build the configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_toon::ToToonConfig;
    ///
    /// let config = ToToonConfig::builder()
    ///     .indent(2)
    ///     .build();
    ///
    /// assert_eq!(config.indent, 2);
    /// ```
    #[inline]
    pub fn build(self) -> ToToonConfig {
        ToToonConfig {
            indent: self.indent,
            delimiter: self.delimiter,
        }
    }
}

/// TOON delimiter types
///
/// Delimiters are used in tabular array format to separate field values.
///
/// # TOON Specification
///
/// Per the TOON v3.0 spec, non-comma delimiters require a suffix in the
/// array bracket notation. For example:
/// - Comma: `users[2]{id,name}:`
/// - Tab: `users[2\t]{id\tname}:`
/// - Pipe: `users[2|]{id|name}:`
///
/// # Examples
///
/// ```rust
/// use hedl_toon::Delimiter;
///
/// let delim = Delimiter::Comma;
/// assert_eq!(delim, Delimiter::Comma);
///
/// // Delimiters are Copy
/// let delim2 = delim;
/// assert_eq!(delim, delim2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    /// Comma delimiter (`,`)
    ///
    /// This is the default and most common delimiter. Provides the most
    /// human-readable output.
    Comma,

    /// Tab delimiter (`\t`)
    ///
    /// Useful for TSV-like output or when field values commonly contain commas.
    Tab,

    /// Pipe delimiter (`|`)
    ///
    /// Useful when both commas and tabs might appear in field values.
    Pipe,
}

impl Delimiter {
    fn char(&self) -> char {
        match self {
            Delimiter::Comma => ',',
            Delimiter::Tab => '\t',
            Delimiter::Pipe => '|',
        }
    }

    fn str(&self) -> &'static str {
        match self {
            Delimiter::Comma => ",",
            Delimiter::Tab => "\t",
            Delimiter::Pipe => "|",
        }
    }

    /// Returns the bracket suffix for non-comma delimiters
    fn bracket_suffix(&self) -> &'static str {
        match self {
            Delimiter::Comma => "",
            Delimiter::Tab => "\t",
            Delimiter::Pipe => "|",
        }
    }
}

/// Convert Document directly to TOON string.
///
/// This conversion preserves HEDL semantics:
/// - References remain as `@Type:id` strings (primitives)
/// - Matrix lists use tabular format when all values are primitives
/// - Nested children are properly indented
///
/// # Arguments
///
/// * `doc` - The HEDL document to convert
/// * `config` - Configuration controlling output format
///
/// # Returns
///
/// A TOON-formatted string, or a [`ToonError`] if conversion fails.
///
/// # Errors
///
/// - [`ToonError::MaxDepthExceeded`] - Document nesting exceeds [`MAX_NESTING_DEPTH`]
/// - [`ToonError::InvalidIndent`] - Configuration has invalid indent value
///
/// # Examples
///
/// ```rust
/// use hedl_toon::{to_toon, ToToonConfig, Delimiter};
/// use hedl_core::Document;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let doc = Document::new((1, 0));
/// let config = ToToonConfig {
///     indent: 4,
///     delimiter: Delimiter::Tab,
/// };
/// let toon = to_toon(&doc, &config)?;
/// # Ok(())
/// # }
/// ```
///
/// # Performance
///
/// - Time Complexity: O(n) where n is total nodes in document
/// - Space Complexity: O(n) for output lines vector
///
/// # Security
///
/// This function enforces a maximum nesting depth of [`MAX_NESTING_DEPTH`] (100 levels)
/// to prevent stack overflow attacks from malicious documents.
pub fn to_toon(doc: &Document, config: &ToToonConfig) -> Result<String> {
    // Validate configuration
    config.validate()?;

    // Pre-allocate lines vector with estimated capacity
    let estimated_capacity = doc.root.len() * 5; // Estimate ~5 lines per root item
    let mut lines = Vec::with_capacity(estimated_capacity);

    // Encode all root items
    for (key, item) in &doc.root {
        encode_item(key, item, doc, &mut lines, 0, config)?;
    }

    Ok(lines.join("\n"))
}

/// Encode an Item (Scalar, Object, or List)
///
/// This is the main dispatch function that routes items to the appropriate
/// encoding function based on their type.
///
/// # Arguments
///
/// * `key` - The field name/key for this item
/// * `item` - The item to encode (Scalar, Object, or List)
/// * `doc` - The full document (for schema lookups)
/// * `lines` - Accumulator for output lines
/// * `depth` - Current nesting depth (for depth limit checking)
/// * `config` - Output configuration
///
/// # Errors
///
/// Returns [`ToonError::MaxDepthExceeded`] if depth exceeds [`MAX_NESTING_DEPTH`].
///
/// # Security
///
/// This function enforces depth limits on every recursive call to prevent
/// stack overflow from malicious deeply-nested documents.
fn encode_item(
    key: &str,
    item: &Item,
    doc: &Document,
    lines: &mut Vec<String>,
    depth: usize,
    config: &ToToonConfig,
) -> Result<()> {
    // Check depth limit to prevent stack overflow
    if depth > MAX_NESTING_DEPTH {
        return Err(ToonError::MaxDepthExceeded {
            depth,
            max: MAX_NESTING_DEPTH,
        });
    }

    match item {
        Item::Scalar(value) => {
            let encoded = encode_value(value, config.delimiter);
            lines.push(indented(depth, &format!("{}: {}", encode_key(key), encoded), config));
            Ok(())
        }
        Item::Object(map) => {
            lines.push(indented(depth, &format!("{}:", encode_key(key)), config));
            for (k, v) in map {
                encode_item(k, v, doc, lines, depth + 1, config)?;
            }
            Ok(())
        }
        Item::List(matrix_list) => {
            encode_matrix_list(key, matrix_list, doc, lines, depth, config)
        }
    }
}

/// Encode a MatrixList as TOON format
///
/// This function implements intelligent format selection:
/// - **Tabular format** when all values are primitives and no children exist
/// - **Expanded format** when items have children or complex values
///
/// # TOON v3.0 Specification
///
/// Per the spec:
/// - Tabular format requires ALL values to be primitives (no nested arrays/objects)
/// - When items have children, use expanded format with key-value pairs
/// - References (`@Type:id`) are treated as primitives
///
/// # Arguments
///
/// * `key` - The field name for this list
/// * `list` - The matrix list to encode
/// * `doc` - The full document (for schema lookups)
/// * `lines` - Accumulator for output lines
/// * `depth` - Current nesting depth
/// * `config` - Output configuration
///
/// # Errors
///
/// Returns [`ToonError::MaxDepthExceeded`] if depth exceeds [`MAX_NESTING_DEPTH`]
/// when encoding nested structures.
fn encode_matrix_list(
    key: &str,
    list: &MatrixList,
    doc: &Document,
    lines: &mut Vec<String>,
    depth: usize,
    config: &ToToonConfig,
) -> Result<()> {
    // Determine count to use: prefer count_hint if present, otherwise use actual rows length
    let count = list.count_hint.unwrap_or(list.rows.len());

    if list.rows.is_empty() {
        // Empty list - simple header without field names
        let header = format!("{}[0{}]:", encode_key(key), config.delimiter.bracket_suffix());
        lines.push(indented(depth, &header, config));
        return Ok(());
    }

    // Check if all values in all rows are primitives AND no children (can use tabular format)
    // Per TOON spec: tabular requires "All values are primitives (no nested arrays/objects)"
    let all_primitive = list.rows.iter().all(|node| {
        node.fields.iter().all(is_primitive_value) && node.children.is_empty()
    });

    if all_primitive {
        // Pure tabular format - all values are primitives with no children
        let field_names: Vec<String> = list.schema.iter().map(|s| encode_key(s)).collect();
        let header = format!(
            "{}[{}{}]{{{}}}:",
            encode_key(key),
            count,
            config.delimiter.bracket_suffix(),
            field_names.join(config.delimiter.str())
        );
        lines.push(indented(depth, &header, config));

        for node in &list.rows {
            let values: Vec<String> = node
                .fields
                .iter()
                .map(|v| encode_value(v, config.delimiter))
                .collect();
            lines.push(indented(depth + 1, &values.join(config.delimiter.str()), config));
        }
        Ok(())
    } else {
        // Expanded format - items have children or complex values
        // Per TOON spec: use list item format with key-value pairs
        let header = format!(
            "{}[{}{}]:",
            encode_key(key),
            count,
            config.delimiter.bracket_suffix()
        );
        lines.push(indented(depth, &header, config));

        for node in &list.rows {
            encode_node_expanded(&list.schema, node, doc, lines, depth + 1, config)?;
        }
        Ok(())
    }
}

/// Encode a single Node in expanded format (with key-value pairs)
///
/// This implements the TOON expanded list item format where each item
/// is represented as a set of key-value pairs.
///
/// # TOON v3.0 Specification
///
/// Per the spec for list items:
/// - First line has "- " prefix followed by first key: value
/// - Subsequent fields at same depth without prefix
/// - Nested arrays as named fields
///
/// # Arguments
///
/// * `schema` - Field names from the type schema
/// * `node` - The node to encode
/// * `doc` - The full document (for schema lookups)
/// * `lines` - Accumulator for output lines
/// * `depth` - Current nesting depth
/// * `config` - Output configuration
///
/// # Errors
///
/// Returns [`ToonError::MaxDepthExceeded`] if depth exceeds [`MAX_NESTING_DEPTH`]
/// when encoding nested children.
fn encode_node_expanded(
    schema: &[String],
    node: &Node,
    doc: &Document,
    lines: &mut Vec<String>,
    depth: usize,
    config: &ToToonConfig,
) -> Result<()> {
    // Output fields as key-value pairs
    // First field gets "- " prefix (list item marker)
    for (i, (field_name, value)) in schema.iter().zip(node.fields.iter()).enumerate() {
        let encoded_value = encode_value(value, config.delimiter);
        if i == 0 {
            // First field with list item prefix
            lines.push(indented(depth, &format!("- {}: {}", encode_key(field_name), encoded_value), config));
        } else {
            // Subsequent fields without prefix
            lines.push(indented(depth + 1, &format!("{}: {}", encode_key(field_name), encoded_value), config));
        }
    }

    // Output children as named array fields
    for (child_type, children) in &node.children {
        // Use pluralized lowercase as field name (e.g., "Child" -> "children", "Person" -> "people")
        let field_name = pluralize(&child_type.to_lowercase());
        encode_child_nodes_as_field(&field_name, child_type, children, doc, lines, depth + 1, config)?;
    }

    Ok(())
}

/// Encode child nodes as a named field (for expanded format)
///
/// This function handles encoding of nested child arrays within expanded format.
/// It uses the plural field name (e.g., "items") rather than the type name.
///
/// # Arguments
///
/// * `field_name` - The field name to use (e.g., "items")
/// * `type_name` - The type name for schema lookup
/// * `nodes` - The child nodes to encode
/// * `doc` - The full document (for schema lookups)
/// * `lines` - Accumulator for output lines
/// * `depth` - Current nesting depth
/// * `config` - Output configuration
///
/// # Notes
///
/// Child nodes don't have count_hint metadata - we use actual length.
///
/// # Errors
///
/// Returns [`ToonError::MaxDepthExceeded`] if depth exceeds [`MAX_NESTING_DEPTH`]
/// when encoding nested children.
fn encode_child_nodes_as_field(
    field_name: &str,
    type_name: &str,
    nodes: &[Node],
    doc: &Document,
    lines: &mut Vec<String>,
    depth: usize,
    config: &ToToonConfig,
) -> Result<()> {
    if nodes.is_empty() {
        let header = format!("{}[0{}]:", encode_key(field_name), config.delimiter.bracket_suffix());
        lines.push(indented(depth, &header, config));
        return Ok(());
    }

    // Get schema from document for this type
    let schema = doc.get_schema(type_name);
    let schema_vec: Vec<String> = if let Some(s) = schema {
        s.to_vec()
    } else if let Some(first) = nodes.first() {
        if let Some(s) = doc.get_schema(&first.type_name) {
            s.to_vec()
        } else {
            (0..first.fields.len())
                .map(|i| format!("field_{}", i))
                .collect()
        }
    } else {
        vec![]
    };
    let field_names: Vec<String> = schema_vec.iter().map(|f| encode_key(f)).collect();

    // Check if all children are primitive (can use tabular)
    let all_primitive = nodes.iter().all(|n| {
        n.fields.iter().all(is_primitive_value) && n.children.is_empty()
    });

    // For child nodes, always use actual length (no count_hint available in Vec<Node>)
    let count = nodes.len();

    if all_primitive && !field_names.is_empty() {
        // Tabular format for children
        let header = format!(
            "{}[{}{}]{{{}}}:",
            encode_key(field_name),
            count,
            config.delimiter.bracket_suffix(),
            field_names.join(config.delimiter.str())
        );
        lines.push(indented(depth, &header, config));

        for node in nodes {
            let values: Vec<String> = node
                .fields
                .iter()
                .map(|v| encode_value(v, config.delimiter))
                .collect();
            lines.push(indented(depth + 1, &values.join(config.delimiter.str()), config));
        }
        Ok(())
    } else {
        // Non-tabular format - use expanded list item format
        let header = format!(
            "{}[{}{}]:",
            encode_key(field_name),
            count,
            config.delimiter.bracket_suffix()
        );
        lines.push(indented(depth, &header, config));

        for node in nodes {
            encode_node_expanded(&schema_vec, node, doc, lines, depth + 1, config)?;
        }
        Ok(())
    }
}

/// Check if a Value is primitive (can be used in tabular format)
///
/// In TOON, primitives are types that can be represented inline without
/// nested structure. This includes:
/// - `null` - The null value
/// - `bool` - Boolean values (true/false)
/// - `int` - Integer numbers
/// - `float` - Floating-point numbers
/// - `string` - Text strings
/// - `reference` - References to other entities (`@Type:id`)
///
/// # Non-Primitives
///
/// Tensors and Expressions are NOT primitives for tabular purposes because
/// they have complex internal structure that cannot be represented inline
/// in a single cell.
///
/// # TOON Specification
///
/// This implements the TOON v3.0 rule that tabular format requires
/// "All values are primitives (no nested arrays/objects)".
///
/// # Performance
///
/// This is an O(1) operation using pattern matching.
fn is_primitive_value(value: &Value) -> bool {
    matches!(
        value,
        Value::Null | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_) | Value::Reference(_)
    )
}

/// Encode a HEDL Value to TOON string
///
/// Converts a single HEDL value to its TOON string representation,
/// applying all necessary normalization rules from the TOON v3.0 spec.
///
/// # Float Normalization
///
/// Per TOON v3.0 specification, floats are normalized as follows:
/// - `NaN` → `null`
/// - `±Infinity` → `null`
/// - `-0` → `0`
/// - Whole numbers → Integer format (no `.0`)
/// - No trailing zeros in decimals
/// - No exponent notation
///
/// # String Handling
///
/// Strings are quoted and escaped as needed based on their content.
/// See [`encode_string`] for details.
///
/// # Reference Encoding
///
/// References are encoded as primitive strings in the format `@Type:id`
/// or `@id` for local references. This preserves reference semantics
/// without requiring complex object representation.
///
/// # Tensor Encoding
///
/// Tensors are flattened and encoded as inline arrays with the format:
/// `[length] value1,value2,value3`
///
/// # Expression Encoding
///
/// Expressions are wrapped in `$()` and encoded as strings.
///
/// # Arguments
///
/// * `value` - The HEDL value to encode
/// * `delimiter` - The active delimiter (affects string quoting)
///
/// # Performance
///
/// - O(1) for primitives (null, bool, int, simple floats)
/// - O(n) for strings (where n is string length)
/// - O(n) for tensors (where n is element count)
fn encode_value(value: &Value, delimiter: Delimiter) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                // Per TOON v3.0 spec: "Non-finite values (NaN, ±Infinity) normalize to null"
                "null".to_string()
            } else if *f == 0.0 && f.is_sign_negative() {
                // Per spec: "-0 becomes 0"
                "0".to_string()
            } else {
                // Per spec: "no exponent notation, no trailing fractional zeros"
                // "If the fractional part is zero after normalization, emit as an integer"
                let rounded = f.round();
                if (*f - rounded).abs() < f64::EPSILON {
                    // It's effectively an integer
                    format!("{}", rounded as i64)
                } else {
                    // Has fractional part - format without exponent notation
                    let s = format!("{:.15}", f);
                    // Trim trailing zeros after decimal point
                    let s = s.trim_end_matches('0');
                    let s = s.trim_end_matches('.');
                    s.to_string()
                }
            }
        }
        Value::String(s) => encode_string(s, delimiter),
        Value::Reference(r) => {
            // References are primitives in TOON - encode as @Type:id string
            let ref_str = r.to_ref_string();
            encode_string(&ref_str, delimiter)
        }
        Value::Tensor(t) => {
            // Tensors become inline arrays - flatten to get all values
            let values: Vec<String> = t.flatten().iter().map(|v| format!("{}", v)).collect();
            format!("[{}] {}", values.len(), values.join(delimiter.str()))
        }
        Value::Expression(e) => {
            // Expressions use their Display implementation, wrapped in $()
            encode_string(&format!("$({})", e), delimiter)
        }
    }
}

/// Encode a string, quoting if necessary
///
/// Applies TOON string encoding rules:
/// - Simple identifiers are unquoted
/// - Strings with special characters are quoted and escaped
///
/// # Arguments
///
/// * `s` - The string to encode
/// * `delimiter` - The active delimiter (affects quoting decision)
///
/// # Returns
///
/// Either the unquoted string or `"quoted and escaped"` string.
///
/// # Security
///
/// This function prevents injection attacks by:
/// - Detecting strings that could be confused with other types
/// - Detecting strings containing structural characters
/// - Escaping all control characters and quotes
///
/// # Performance
///
/// - O(n) for needs_quoting check
/// - O(n) for escape_string if quoting needed
fn encode_string(s: &str, delimiter: Delimiter) -> String {
    if needs_quoting(s, delimiter) {
        format!("\"{}\"", escape_string(s))
    } else {
        s.to_string()
    }
}

/// Check if a string needs quoting in TOON format (optimized single-pass).
///
/// Uses a single iteration through the string to check all quoting conditions.
/// This implementation fixes bugs in the previous version and handles all edge cases correctly.
///
/// # Quoting Rules
///
/// A string needs quoting if it:
/// - Is empty
/// - Has leading/trailing whitespace (including non-ASCII whitespace)
/// - Is a boolean or null literal ("true", "false", "null")
/// - Looks numeric (starts with digit or minus+digit)
/// - Contains structural characters (`:`, `[`, `]`, `{`, `}`)
/// - Contains escape sequences (`"`, `\`, newline, etc.)
/// - Contains the delimiter character
/// - Starts with `-` (list marker) or `@` (reference marker)
///
/// # Performance
///
/// - Time Complexity: O(n) single pass through string
/// - Space Complexity: O(1) no allocations
///
/// # Examples
///
/// ```text
/// assert!(needs_quoting("", Delimiter::Comma)); // empty
/// assert!(needs_quoting("true", Delimiter::Comma)); // boolean literal
/// assert!(needs_quoting("hello, world", Delimiter::Comma)); // contains delimiter
/// assert!(!needs_quoting("hello", Delimiter::Comma)); // simple string
/// ```
#[inline]
fn needs_quoting(s: &str, delimiter: Delimiter) -> bool {
    // Empty strings need quoting
    if s.is_empty() {
        return true;
    }

    // Fast path: Check for boolean/null literals (very common)
    if matches!(s, "true" | "false" | "null") {
        return true;
    }

    let delimiter_char = delimiter.char();
    let mut chars = s.chars();

    // Check first character (must exist since we checked is_empty)
    let first = chars.next().unwrap();

    // Check for special markers, whitespace, and escape sequences
    if first == '@' || first.is_whitespace() {
        return true;
    }

    // Check first char for structural/escape chars
    if matches!(first, ':' | '[' | ']' | '{' | '}' | '"' | '\\' | '\n' | '\r' | '\t') {
        return true;
    }

    // Check if first char is the delimiter
    if first == delimiter.char() {
        return true;
    }

    // Check if numeric-like (starts with digit or minus+digit)
    if first.is_ascii_digit() {
        return true;
    }

    // Check if starts with minus (could be list marker or negative number)
    if first == '-' {
        // Peek at next character if it exists
        if let Some(second) = chars.clone().next() {
            // If followed by digit, it looks like a number
            if second.is_ascii_digit() {
                return true;
            }
        }
        // Single '-' or '-' followed by non-digit needs quoting (list marker)
        return true;
    }

    // Single pass through remaining characters, tracking last char
    let mut last_char = first;
    for c in chars {
        // Check for structural characters
        if matches!(c, ':' | '[' | ']' | '{' | '}') {
            return true;
        }

        // Check for escape sequences that always need quoting
        if matches!(c, '"' | '\\' | '\n' | '\r' | '\t') {
            return true;
        }

        // Check for delimiter (includes tab if delimiter is Tab)
        if c == delimiter_char {
            return true;
        }

        last_char = c;
    }

    // Check if last character is whitespace (trailing whitespace)
    if last_char.is_whitespace() {
        return true;
    }

    false
}

/// Check if a string looks like a number
///
/// Detects strings that could be parsed as numbers, which need to be
/// quoted to preserve their string type.
///
/// # Detection Rules
///
/// A string is numeric-like if it:
/// - Starts with a digit (0-9) or minus sign followed by digit
/// - Contains only digits and numeric characters (`.`, `e`, `E`, `+`, `-`)
///
/// # Examples
///
/// Numeric-like strings:
/// - `"123"` → true
/// - `"-45.67"` → true
/// - `"1.5e10"` → true
/// - `"0"` → true
///
/// Non-numeric strings:
/// - `"abc"` → false
/// - `"-"` → false (no digits)
/// - `"123abc"` → false (contains letters after digits)
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// `true` if the string looks like a number, `false` otherwise.
///
/// # Performance
///
/// O(n) where n is the string length.
///
/// Note: This function is kept for potential future use and testing.
/// The inline numeric check in needs_quoting() is optimized for performance.
#[allow(dead_code)]
fn is_numeric_like(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let bytes = s.as_bytes();
    let start = if bytes[0] == b'-' { 1 } else { 0 };

    if start >= bytes.len() {
        return false;
    }

    // Must start with digit
    if !bytes[start].is_ascii_digit() {
        return false;
    }

    // Check if it's all digits, dots, e, E, +, -
    bytes[start..].iter().all(|&b| {
        b.is_ascii_digit() || b == b'.' || b == b'e' || b == b'E' || b == b'+' || b == b'-'
    })
}

/// Escape a string for TOON
///
/// Applies escape sequences to control characters and special characters
/// that would otherwise break the TOON format.
///
/// # Escape Sequences
///
/// - `\` → `\\` (backslash)
/// - `"` → `\"` (double quote)
/// - newline → `\n`
/// - carriage return → `\r`
/// - tab → `\t`
/// - All other characters → unchanged
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// A new string with all special characters escaped.
///
/// # Security
///
/// This function is critical for injection prevention. It ensures that
/// no unescaped control characters or quotes can break out of the
/// string context in the TOON output.
///
/// # Performance
///
/// - Pre-allocates buffer to source string length
/// - O(n) where n is the string length
/// - Worst case: O(2n) for strings with many special characters
fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}

/// Encode a key, quoting if necessary
///
/// Keys (field names) have different quoting rules than values.
/// Simple identifiers can be unquoted, while complex keys must be quoted.
///
/// # Arguments
///
/// * `key` - The key to encode
///
/// # Returns
///
/// Either the unquoted key or `"quoted and escaped"` key.
///
/// # Examples
///
/// ```text
/// encode_key("simple") → "simple"
/// encode_key("with_underscore") → "with_underscore"
/// encode_key("with.dot") → "with.dot"
/// encode_key("123") → "\"123\"" (starts with number)
/// encode_key("has space") → "\"has space\""
/// ```
fn encode_key(key: &str) -> String {
    if is_valid_unquoted_key(key) {
        key.to_string()
    } else {
        format!("\"{}\"", escape_string(key))
    }
}

/// Check if a key can be unquoted
///
/// Determines whether a key (field name) can appear without quotes.
///
/// # Validity Rules
///
/// A key is valid unquoted if:
/// - It is non-empty
/// - It starts with a letter (a-z, A-Z) or underscore (`_`)
/// - It contains only letters, digits, underscores, or dots (`.`)
///
/// # Arguments
///
/// * `key` - The key to check
///
/// # Returns
///
/// `true` if the key can be unquoted, `false` otherwise.
///
/// # Performance
///
/// O(n) where n is the key length.
fn is_valid_unquoted_key(key: &str) -> bool {
    if key.is_empty() {
        return false;
    }

    let bytes = key.as_bytes();
    let first = bytes[0];

    // Must start with letter or underscore
    if !first.is_ascii_alphabetic() && first != b'_' {
        return false;
    }

    // Rest can be alphanumeric, underscore, or dot
    bytes[1..].iter().all(|&b| {
        b.is_ascii_alphanumeric() || b == b'_' || b == b'.'
    })
}

/// Create indented string
///
/// Generates a string with the appropriate indentation for the given depth level.
///
/// # Arguments
///
/// * `depth` - The nesting depth (0 = no indentation)
/// * `content` - The content string to indent
/// * `config` - Configuration containing indent size
///
/// # Returns
///
/// A new string with `depth * config.indent` spaces prepended to `content`.
///
/// # Examples
///
/// ```text
/// // With indent=2
/// indented(0, "hello", &config) → "hello"
/// indented(1, "hello", &config) → "  hello"
/// indented(2, "hello", &config) → "    hello"
/// ```
///
/// # Performance
///
/// - O(depth * indent) for space string allocation
/// - O(n) where n is content length for concatenation
fn indented(depth: usize, content: &str, config: &ToToonConfig) -> String {
    let indent = " ".repeat(config.indent * depth);
    format!("{}{}", indent, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::Reference;
    use hedl_core::lex::Tensor;
    use std::collections::BTreeMap;

    fn default_config() -> ToToonConfig {
        ToToonConfig::default()
    }

    #[test]
    fn test_encode_primitives() {
        let config = default_config();
        assert_eq!(encode_value(&Value::Null, config.delimiter), "null");
        assert_eq!(encode_value(&Value::Bool(true), config.delimiter), "true");
        assert_eq!(encode_value(&Value::Bool(false), config.delimiter), "false");
        assert_eq!(encode_value(&Value::Int(42), config.delimiter), "42");
        assert_eq!(encode_value(&Value::Int(-123), config.delimiter), "-123");
        assert_eq!(encode_value(&Value::Float(3.15), config.delimiter), "3.15");
        assert_eq!(encode_value(&Value::String("hello".to_string()), config.delimiter), "hello");
    }

    #[test]
    fn test_float_normalization() {
        // TOON v3.0 spec compliance tests
        let config = default_config();

        // NaN becomes null
        assert_eq!(encode_value(&Value::Float(f64::NAN), config.delimiter), "null");

        // Infinity becomes null
        assert_eq!(encode_value(&Value::Float(f64::INFINITY), config.delimiter), "null");
        assert_eq!(encode_value(&Value::Float(f64::NEG_INFINITY), config.delimiter), "null");

        // -0 becomes 0
        assert_eq!(encode_value(&Value::Float(-0.0), config.delimiter), "0");

        // Whole numbers emit as integers
        assert_eq!(encode_value(&Value::Float(5.0), config.delimiter), "5");
        assert_eq!(encode_value(&Value::Float(100.0), config.delimiter), "100");

        // No trailing zeros
        assert_eq!(encode_value(&Value::Float(3.5), config.delimiter), "3.5");
    }

    #[test]
    fn test_encode_reference() {
        let config = default_config();

        // Qualified reference
        let ref_val = Value::Reference(Reference::qualified("User", "user123"));
        assert_eq!(encode_value(&ref_val, config.delimiter), "\"@User:user123\"");

        // Local reference
        let local_ref = Value::Reference(Reference::local("item1"));
        assert_eq!(encode_value(&local_ref, config.delimiter), "\"@item1\"");
    }

    #[test]
    fn test_encode_string_quoting() {
        let config = default_config();

        // Simple string - no quoting
        assert_eq!(encode_value(&Value::String("hello".to_string()), config.delimiter), "hello");

        // Empty string - needs quoting
        assert_eq!(encode_value(&Value::String("".to_string()), config.delimiter), "\"\"");

        // String with colon - needs quoting
        assert_eq!(encode_value(&Value::String("foo:bar".to_string()), config.delimiter), "\"foo:bar\"");

        // Boolean-like string - needs quoting
        assert_eq!(encode_value(&Value::String("true".to_string()), config.delimiter), "\"true\"");

        // Numeric-like string - needs quoting
        assert_eq!(encode_value(&Value::String("123".to_string()), config.delimiter), "\"123\"");
    }

    #[test]
    fn test_simple_document() {
        let mut doc = Document::new((1, 0));
        doc.root.insert("name".to_string(), Item::Scalar(Value::String("test".to_string())));
        doc.root.insert("count".to_string(), Item::Scalar(Value::Int(42)));

        let config = default_config();
        let result = to_toon(&doc, &config).unwrap();

        assert!(result.contains("count: 42"));
        assert!(result.contains("name: test"));
    }

    #[test]
    fn test_matrix_list_tabular() {
        let mut doc = Document::new((1, 0));
        doc.structs.insert("User".to_string(), vec!["id".to_string(), "name".to_string(), "age".to_string()]);

        let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string(), "age".to_string()]);
        list.add_row(Node::new("User", "u1", vec![
            Value::String("u1".to_string()),
            Value::String("Alice".to_string()),
            Value::Int(30),
        ]));
        list.add_row(Node::new("User", "u2", vec![
            Value::String("u2".to_string()),
            Value::String("Bob".to_string()),
            Value::Int(25),
        ]));

        doc.root.insert("users".to_string(), Item::List(list));

        let config = default_config();
        let result = to_toon(&doc, &config).unwrap();

        // Should have tabular header
        assert!(result.contains("users[2]{id,name,age}:"));
        // Should have data rows
        assert!(result.contains("u1,Alice,30"));
        assert!(result.contains("u2,Bob,25"));
    }

    #[test]
    fn test_matrix_list_with_count_hint() {
        let mut doc = Document::new((1, 0));
        doc.structs.insert("Team".to_string(), vec!["id".to_string(), "name".to_string()]);

        // Create list with count_hint of 5, but only 2 actual rows
        let mut list = MatrixList::with_count_hint(
            "Team",
            vec!["id".to_string(), "name".to_string()],
            5
        );
        list.add_row(Node::new("Team", "t1", vec![
            Value::String("t1".to_string()),
            Value::String("Alpha".to_string()),
        ]));
        list.add_row(Node::new("Team", "t2", vec![
            Value::String("t2".to_string()),
            Value::String("Beta".to_string()),
        ]));

        doc.root.insert("teams".to_string(), Item::List(list));

        let config = default_config();
        let result = to_toon(&doc, &config).unwrap();

        // Should use count_hint (5) instead of actual rows (2)
        assert!(result.contains("teams[5]{id,name}:"));
        assert!(result.contains("t1,Alpha"));
        assert!(result.contains("t2,Beta"));
    }

    #[test]
    fn test_matrix_list_with_references() {
        let mut doc = Document::new((1, 0));
        doc.structs.insert("Order".to_string(), vec!["id".to_string(), "user".to_string(), "amount".to_string()]);

        let mut list = MatrixList::new("Order", vec!["id".to_string(), "user".to_string(), "amount".to_string()]);
        list.add_row(Node::new("Order", "o1", vec![
            Value::String("o1".to_string()),
            Value::Reference(Reference::qualified("User", "u1")),
            Value::Float(99.99),
        ]));

        doc.root.insert("orders".to_string(), Item::List(list));

        let config = default_config();
        let result = to_toon(&doc, &config).unwrap();

        // References should be in tabular format as primitives!
        assert!(result.contains("orders[1]{id,user,amount}:"));
        assert!(result.contains("\"@User:u1\""));
    }

    #[test]
    fn test_nested_object() {
        let mut doc = Document::new((1, 0));

        let mut inner = BTreeMap::new();
        inner.insert("x".to_string(), Item::Scalar(Value::Int(10)));
        inner.insert("y".to_string(), Item::Scalar(Value::Int(20)));

        doc.root.insert("position".to_string(), Item::Object(inner));

        let config = default_config();
        let result = to_toon(&doc, &config).unwrap();

        assert!(result.contains("position:"));
        assert!(result.contains("x: 10"));
        assert!(result.contains("y: 20"));
    }

    #[test]
    fn test_is_primitive_value() {
        assert!(is_primitive_value(&Value::Null));
        assert!(is_primitive_value(&Value::Bool(true)));
        assert!(is_primitive_value(&Value::Int(42)));
        assert!(is_primitive_value(&Value::Float(3.15)));
        assert!(is_primitive_value(&Value::String("test".to_string())));
        assert!(is_primitive_value(&Value::Reference(Reference::local("x"))));

        // Tensors are NOT primitives
        assert!(!is_primitive_value(&Value::Tensor(Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]))));
    }

    #[test]
    fn test_empty_list() {
        let mut doc = Document::new((1, 0));
        let list = MatrixList::new("User", vec!["id".to_string()]);
        doc.root.insert("users".to_string(), Item::List(list));

        let config = default_config();
        let result = to_toon(&doc, &config).unwrap();

        assert!(result.contains("users[0]:"));
    }

    #[test]
    fn test_special_characters_in_strings() {
        let mut doc = Document::new((1, 0));
        doc.root.insert("message".to_string(), Item::Scalar(Value::String("Hello\nWorld".to_string())));

        let config = default_config();
        let result = to_toon(&doc, &config).unwrap();

        // Newline should be escaped
        assert!(result.contains("\"Hello\\nWorld\""));
    }

    #[test]
    fn test_key_quoting() {
        assert_eq!(encode_key("simple"), "simple");
        assert_eq!(encode_key("with_underscore"), "with_underscore");
        assert_eq!(encode_key("with.dot"), "with.dot");
        assert_eq!(encode_key("123"), "\"123\""); // Starts with number
        assert_eq!(encode_key("has space"), "\"has space\"");
        assert_eq!(encode_key(""), "\"\"");
    }

    #[test]
    fn test_depth_limit_protection() {
        // Create a deeply nested document (exceeding MAX_NESTING_DEPTH)
        let mut doc = Document::new((1, 0));

        // Create nested objects
        let mut current = BTreeMap::new();
        current.insert("value".to_string(), Item::Scalar(Value::Int(42)));

        // Nest 101 levels deep (exceeds MAX_NESTING_DEPTH of 100)
        for i in (0..101).rev() {
            let mut parent = BTreeMap::new();
            parent.insert(format!("level{}", i), Item::Object(current));
            current = parent;
        }

        doc.root.insert("root".to_string(), Item::Object(current));

        let config = default_config();
        let result = to_toon(&doc, &config);

        // Should error with MaxDepthExceeded
        assert!(result.is_err());
        match result {
            Err(ToonError::MaxDepthExceeded { depth, max }) => {
                assert!(depth > max);
                assert_eq!(max, MAX_NESTING_DEPTH);
            }
            _ => panic!("Expected MaxDepthExceeded error"),
        }
    }

    #[test]
    fn test_invalid_indent_config() {
        let doc = Document::new((1, 0));
        let config = ToToonConfig {
            indent: 0,  // Invalid!
            delimiter: Delimiter::Comma,
        };

        let result = to_toon(&doc, &config);
        assert!(result.is_err());
        match result {
            Err(ToonError::InvalidIndent(0)) => {}
            _ => panic!("Expected InvalidIndent error"),
        }
    }

    #[test]
    fn test_config_validation() {
        let config = ToToonConfig {
            indent: 0,
            delimiter: Delimiter::Comma,
        };
        assert!(config.validate().is_err());

        let config = ToToonConfig {
            indent: 1,
            delimiter: Delimiter::Comma,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_needs_quoting_comprehensive() {
        // Empty string
        assert!(needs_quoting("", Delimiter::Comma));

        // Boolean literals
        assert!(needs_quoting("true", Delimiter::Comma));
        assert!(needs_quoting("false", Delimiter::Comma));
        assert!(needs_quoting("null", Delimiter::Comma));

        // Simple strings - no quoting needed
        assert!(!needs_quoting("hello", Delimiter::Comma));
        assert!(!needs_quoting("world", Delimiter::Comma));
        assert!(!needs_quoting("simple", Delimiter::Comma));

        // Leading/trailing whitespace
        assert!(needs_quoting(" hello", Delimiter::Comma));
        assert!(needs_quoting("hello ", Delimiter::Comma));
        assert!(needs_quoting(" hello ", Delimiter::Comma));
        assert!(needs_quoting("\thello", Delimiter::Comma));
        assert!(needs_quoting("hello\t", Delimiter::Comma));

        // Numeric-like strings
        assert!(needs_quoting("123", Delimiter::Comma));
        assert!(needs_quoting("0", Delimiter::Comma));
        assert!(needs_quoting("-123", Delimiter::Comma));
        assert!(needs_quoting("-0", Delimiter::Comma));
        assert!(needs_quoting("42", Delimiter::Comma));
        assert!(needs_quoting("3.14", Delimiter::Comma));

        // Structural characters
        assert!(needs_quoting("foo:bar", Delimiter::Comma));
        assert!(needs_quoting("a[b]", Delimiter::Comma));
        assert!(needs_quoting("a{b}", Delimiter::Comma));
        assert!(needs_quoting("has:colon", Delimiter::Comma));

        // Escape sequences
        assert!(needs_quoting("hello\"world", Delimiter::Comma));
        assert!(needs_quoting("path\\to\\file", Delimiter::Comma));
        assert!(needs_quoting("line1\nline2", Delimiter::Comma));
        assert!(needs_quoting("with\ttab", Delimiter::Comma));
        assert!(needs_quoting("carriage\rreturn", Delimiter::Comma));

        // Delimiter-specific
        assert!(needs_quoting("hello,world", Delimiter::Comma));
        assert!(!needs_quoting("hello,world", Delimiter::Tab));
        assert!(!needs_quoting("hello,world", Delimiter::Pipe));

        assert!(needs_quoting("hello\tworld", Delimiter::Tab));
        assert!(needs_quoting("hello\tworld", Delimiter::Comma)); // tab is escape sequence

        assert!(needs_quoting("hello|world", Delimiter::Pipe));
        assert!(!needs_quoting("hello|world", Delimiter::Comma));
        assert!(!needs_quoting("hello|world", Delimiter::Tab));

        // Special markers
        assert!(needs_quoting("@User:123", Delimiter::Comma));
        assert!(needs_quoting("@ref", Delimiter::Comma));
        assert!(needs_quoting("-item", Delimiter::Comma));
        assert!(needs_quoting("-", Delimiter::Comma));

        // Non-numeric strings starting with minus
        assert!(needs_quoting("-abc", Delimiter::Comma));
        assert!(needs_quoting("-", Delimiter::Comma));

        // Unicode whitespace
        assert!(needs_quoting("\u{00A0}hello", Delimiter::Comma)); // non-breaking space
        assert!(needs_quoting("hello\u{00A0}", Delimiter::Comma)); // non-breaking space at end

        // Complex but valid unquoted strings
        assert!(!needs_quoting("HelloWorld", Delimiter::Comma));
        assert!(!needs_quoting("hello_world", Delimiter::Comma));
        assert!(!needs_quoting("CONSTANT_NAME", Delimiter::Comma));
    }

    #[test]
    fn test_needs_quoting_edge_cases() {
        // Single character strings
        assert!(!needs_quoting("a", Delimiter::Comma));
        assert!(!needs_quoting("z", Delimiter::Comma));
        assert!(needs_quoting("0", Delimiter::Comma)); // numeric
        assert!(needs_quoting("-", Delimiter::Comma)); // special marker
        assert!(needs_quoting("@", Delimiter::Comma)); // reference marker
        assert!(needs_quoting(" ", Delimiter::Comma)); // whitespace

        // Mixed content
        assert!(!needs_quoting("abc123", Delimiter::Comma));
        assert!(!needs_quoting("test_123", Delimiter::Comma));
        assert!(!needs_quoting("camelCase", Delimiter::Comma));

        // Similar to literals but different
        assert!(!needs_quoting("truthy", Delimiter::Comma));
        assert!(!needs_quoting("falsey", Delimiter::Comma));
        assert!(!needs_quoting("nullable", Delimiter::Comma));
        assert!(!needs_quoting("True", Delimiter::Comma)); // capital T
        assert!(!needs_quoting("FALSE", Delimiter::Comma)); // all caps
        assert!(!needs_quoting("Null", Delimiter::Comma)); // capital N
    }

    // ========================================================================
    // Pluralization Tests
    // ========================================================================

    #[test]
    fn test_pluralize_irregular_people_and_family() {
        // People and family
        assert_eq!(pluralize("child"), "children");
        assert_eq!(pluralize("person"), "people");
        assert_eq!(pluralize("man"), "men");
        assert_eq!(pluralize("woman"), "women");
    }

    #[test]
    fn test_pluralize_irregular_body_parts() {
        // Body parts
        assert_eq!(pluralize("foot"), "feet");
        assert_eq!(pluralize("tooth"), "teeth");
        assert_eq!(pluralize("goose"), "geese");
    }

    #[test]
    fn test_pluralize_irregular_animals() {
        // Small animals
        assert_eq!(pluralize("mouse"), "mice");
        assert_eq!(pluralize("louse"), "lice");

        // Livestock
        assert_eq!(pluralize("ox"), "oxen");
        assert_eq!(pluralize("sheep"), "sheep");
        assert_eq!(pluralize("deer"), "deer");
        assert_eq!(pluralize("moose"), "moose");
        assert_eq!(pluralize("fish"), "fish");
    }

    #[test]
    fn test_pluralize_irregular_scientific() {
        // Scientific/academic terms
        assert_eq!(pluralize("phenomenon"), "phenomena");
        assert_eq!(pluralize("criterion"), "criteria");
        assert_eq!(pluralize("datum"), "data");
        assert_eq!(pluralize("analysis"), "analyses");
        assert_eq!(pluralize("thesis"), "theses");
        assert_eq!(pluralize("hypothesis"), "hypotheses");
        assert_eq!(pluralize("crisis"), "crises");
    }

    #[test]
    fn test_pluralize_irregular_classical() {
        // Classical/Latin plurals
        assert_eq!(pluralize("cactus"), "cacti");
        assert_eq!(pluralize("fungus"), "fungi");
        assert_eq!(pluralize("nucleus"), "nuclei");
        assert_eq!(pluralize("radius"), "radii");
        assert_eq!(pluralize("alumnus"), "alumni");
        assert_eq!(pluralize("stimulus"), "stimuli");
    }

    #[test]
    fn test_pluralize_unchanged_forms() {
        // Unchanged forms
        assert_eq!(pluralize("species"), "species");
        assert_eq!(pluralize("series"), "series");
        assert_eq!(pluralize("aircraft"), "aircraft");
        assert_eq!(pluralize("spacecraft"), "spacecraft");
    }

    #[test]
    fn test_pluralize_regular() {
        // Regular plurals (just add 's')
        assert_eq!(pluralize("user"), "users");
        assert_eq!(pluralize("order"), "orders");
        assert_eq!(pluralize("team"), "teams");
        assert_eq!(pluralize("item"), "items");
        assert_eq!(pluralize("product"), "products");
        assert_eq!(pluralize("account"), "accounts");
        assert_eq!(pluralize("invoice"), "invoices");
    }

    #[test]
    fn test_pluralize_case_preservation() {
        // Lowercase
        assert_eq!(pluralize("child"), "children");
        assert_eq!(pluralize("person"), "people");

        // Capitalized
        assert_eq!(pluralize("Child"), "Children");
        assert_eq!(pluralize("Person"), "People");
        assert_eq!(pluralize("Mouse"), "Mice");
        assert_eq!(pluralize("Tooth"), "Teeth");

        // All uppercase
        assert_eq!(pluralize("CHILD"), "CHILDREN");
        assert_eq!(pluralize("PERSON"), "PEOPLE");
        assert_eq!(pluralize("MOUSE"), "MICE");

        // Regular plurals with case
        assert_eq!(pluralize("User"), "Users");
        assert_eq!(pluralize("USER"), "USERS");
    }

    #[test]
    fn test_pluralize_edge_cases() {
        // Empty string
        assert_eq!(pluralize(""), "");

        // Single character
        assert_eq!(pluralize("a"), "as");
        assert_eq!(pluralize("x"), "xs");
    }

    #[test]
    fn test_pluralize_mixed_case() {
        // Mixed case detection (treat as regular)
        assert_eq!(pluralize("CamelCase"), "CamelCases");
        assert_eq!(pluralize("mixedCase"), "mixedCases");
    }

    #[test]
    fn test_pluralize_in_document_context() {
        // Test pluralization in actual TOON conversion
        let mut doc = Document::new((1, 0));
        doc.structs.insert("Child".to_string(), vec!["id".to_string(), "name".to_string()]);

        // Create parent with children
        let mut parent_list = MatrixList::new("Parent", vec!["id".to_string(), "name".to_string()]);
        let mut parent_node = Node::new("Parent", "p1", vec![
            Value::String("p1".to_string()),
            Value::String("John".to_string()),
        ]);

        // Add children to parent
        parent_node.children.insert("Child".to_string(), vec![
            Node::new("Child", "c1", vec![
                Value::String("c1".to_string()),
                Value::String("Alice".to_string()),
            ]),
            Node::new("Child", "c2", vec![
                Value::String("c2".to_string()),
                Value::String("Bob".to_string()),
            ]),
        ]);

        parent_list.add_row(parent_node);
        doc.root.insert("parents".to_string(), Item::List(parent_list));

        let config = default_config();
        let result = to_toon(&doc, &config).unwrap();

        // Should use "children" not "childs"
        assert!(result.contains("children[2]{id,name}:"));
        assert!(!result.contains("childs"));
    }

    #[test]
    fn test_pluralize_person_in_document() {
        let mut doc = Document::new((1, 0));
        doc.structs.insert("Person".to_string(), vec!["id".to_string(), "name".to_string()]);

        let mut team_list = MatrixList::new("Team", vec!["id".to_string(), "name".to_string()]);
        let mut team_node = Node::new("Team", "t1", vec![
            Value::String("t1".to_string()),
            Value::String("Alpha Team".to_string()),
        ]);

        // Add people to team
        team_node.children.insert("Person".to_string(), vec![
            Node::new("Person", "p1", vec![
                Value::String("p1".to_string()),
                Value::String("Alice".to_string()),
            ]),
            Node::new("Person", "p2", vec![
                Value::String("p2".to_string()),
                Value::String("Bob".to_string()),
            ]),
        ]);

        team_list.add_row(team_node);
        doc.root.insert("teams".to_string(), Item::List(team_list));

        let config = default_config();
        let result = to_toon(&doc, &config).unwrap();

        // Should use "people" not "persons"
        assert!(result.contains("people[2]{id,name}:"));
        assert!(!result.contains("persons"));
    }

    #[test]
    fn test_pluralize_comprehensive_irregular_list() {
        // Test all irregular forms to ensure comprehensive coverage
        let irregular_tests = vec![
            // People/family
            ("child", "children"),
            ("person", "people"),
            ("man", "men"),
            ("woman", "women"),
            // Body parts
            ("foot", "feet"),
            ("tooth", "teeth"),
            ("goose", "geese"),
            // Animals
            ("mouse", "mice"),
            ("louse", "lice"),
            ("ox", "oxen"),
            ("sheep", "sheep"),
            ("deer", "deer"),
            ("moose", "moose"),
            ("fish", "fish"),
            // Scientific
            ("phenomenon", "phenomena"),
            ("criterion", "criteria"),
            ("datum", "data"),
            ("analysis", "analyses"),
            ("thesis", "theses"),
            ("hypothesis", "hypotheses"),
            ("crisis", "crises"),
            // Classical
            ("cactus", "cacti"),
            ("fungus", "fungi"),
            ("nucleus", "nuclei"),
            ("radius", "radii"),
            ("alumnus", "alumni"),
            ("stimulus", "stimuli"),
            // Unchanged
            ("species", "species"),
            ("series", "series"),
            ("aircraft", "aircraft"),
            ("spacecraft", "spacecraft"),
        ];

        for (singular, expected_plural) in irregular_tests {
            assert_eq!(
                pluralize(singular),
                expected_plural,
                "Failed to pluralize '{}' correctly",
                singular
            );
        }
    }
}

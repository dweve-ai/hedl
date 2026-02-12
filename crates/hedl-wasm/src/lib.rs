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

//! HEDL WebAssembly Bindings
//!
//! This crate provides WebAssembly bindings for HEDL, enabling HEDL parsing
//! and manipulation in browsers and other JavaScript/TypeScript environments.
//!
//! # Usage (JavaScript/TypeScript)
//!
//! ```typescript
//! import init, { parse, toJson, fromJson, format, validate, getStats } from 'hedl-wasm';
//!
//! await init();
//!
//! // Parse HEDL
//! const doc = parse(`
//! %VERSION 1.0
//! %STRUCT User[id, name, email]
//! ---
//! users:@User
//!  | alice | Alice Smith | alice@example.com |
//!  | bob   | Bob Jones   | bob@example.com   |
//! `);
//!
//! // Convert to JSON
//! const json = toJson(doc);
//!
//! // Convert JSON to HEDL
//! const hedl = fromJson(jsonData);
//!
//! // Format HEDL
//! const formatted = format(hedlString);
//!
//! // Validate HEDL
//! const result = validate(hedlString);
//! if (!result.valid) {
//!     console.error(result.errors);
//! }
//!
//! // Get token statistics
//! const stats = getStats(hedlString);
//! console.log(`Token savings: ${stats.savingsPercent}%`);
//! ```

#![cfg_attr(not(test), warn(missing_docs))]
use hedl_c14n::CanonicalConfig;
use hedl_core::{parse as core_parse, Document};
use std::sync::atomic::{AtomicUsize, Ordering};
use wasm_bindgen::prelude::*;

#[cfg(feature = "full-validation")]
use hedl_lint::lint;

// Modules
mod document;
mod stats;
mod validation;

#[cfg(test)]
mod tests;

// Re-exports for internal use
use document::count_item_entities;
#[cfg(feature = "query-api")]
use document::find_entities;
#[cfg(any(feature = "statistics", feature = "token-tools"))]
use stats::{estimate_tokens, TokenStats};
#[cfg(feature = "full-validation")]
use validation::ValidationWarning;
use validation::{ValidationError, ValidationResult};

// TypeScript custom type definitions for better type inference
#[wasm_bindgen(typescript_custom_section)]
const TS_CUSTOM_TYPES: &'static str = r#"
/**
 * Represents a JSON primitive value.
 */
export type JsonPrimitive = string | number | boolean | null;

/**
 * Represents a JSON array (recursive).
 */
export type JsonArray = JsonValue[];

/**
 * Represents a JSON object (recursive).
 */
export type JsonObject = { [key: string]: JsonValue };

/**
 * Represents any valid JSON value.
 */
export type JsonValue = JsonPrimitive | JsonObject | JsonArray;
"#;

/// Default maximum input size: 500 MB
/// This is a conservative default that balances memory safety with practical use cases.
/// Can be customized using `setMaxInputSize()` for larger documents.
pub const DEFAULT_MAX_INPUT_SIZE: usize = 500 * 1024 * 1024; // 500 MB

/// Global maximum input size configuration
/// Uses atomic for thread-safe access in WASM context
static MAX_INPUT_SIZE: AtomicUsize = AtomicUsize::new(DEFAULT_MAX_INPUT_SIZE);

// Conditional imports for JSON feature
#[cfg(feature = "json")]
use hedl_json::{from_json_value, to_json_value, FromJsonConfig, ToJsonConfig};

/// Initialize panic hook for error handling.
///
/// In debug builds, show full panic messages for debugging.
/// In release builds, show generic message to avoid information disclosure.
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    #[cfg(not(debug_assertions))]
    std::panic::set_hook(Box::new(|_| {
        // Generic error message - avoids disclosing internal paths/state
        web_sys::console::error_1(&"HEDL: An internal error occurred".into());
    }));
}

/// HEDL version constant.
#[wasm_bindgen]
#[must_use]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Set the maximum input size in bytes.
///
/// This controls the maximum size of HEDL/JSON input strings that can be processed.
/// Default is 500 MB. Set to a higher value if you need to process larger documents.
///
/// # Arguments
/// * `size` - Maximum input size in bytes
///
/// # Example (JavaScript)
/// ```javascript
/// import { setMaxInputSize } from 'hedl-wasm';
///
/// // Allow processing up to 1 GB documents
/// setMaxInputSize(1024 * 1024 * 1024);
/// ```
#[wasm_bindgen(js_name = setMaxInputSize)]
pub fn set_max_input_size(size: usize) {
    MAX_INPUT_SIZE.store(size, Ordering::Relaxed);
}

/// Get the current maximum input size in bytes.
///
/// # Returns
/// Current maximum input size setting
///
/// # Example (JavaScript)
/// ```javascript
/// import { getMaxInputSize } from 'hedl-wasm';
///
/// const currentLimit = getMaxInputSize();
/// console.log(`Current limit: ${currentLimit / (1024 * 1024)} MB`);
/// ```
#[wasm_bindgen(js_name = getMaxInputSize)]
pub fn get_max_input_size() -> usize {
    MAX_INPUT_SIZE.load(Ordering::Relaxed)
}

/// Validate input size against the configured limit.
#[inline(always)]
fn check_input_size(input: &str) -> Result<(), JsError> {
    let max_size = MAX_INPUT_SIZE.load(Ordering::Relaxed);
    let input_size = input.len();

    if input_size > max_size {
        return Err(JsError::new(&format!(
            "Input size ({} bytes, {} MB) exceeds maximum allowed size ({} bytes, {} MB). \
             Use setMaxInputSize() to increase the limit if needed.",
            input_size,
            input_size / (1024 * 1024),
            max_size,
            max_size / (1024 * 1024)
        )));
    }

    Ok(())
}

// --- Parse Result Types ---

/// Parsed HEDL document wrapper.
#[wasm_bindgen]
pub struct HedlDocument {
    inner: Document,
}

#[wasm_bindgen]
impl HedlDocument {
    /// Get the HEDL version as a string (e.g., "1.0").
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn version(&self) -> String {
        format!("{}.{}", self.inner.version.0, self.inner.version.1)
    }

    /// Get the number of schema definitions.
    #[wasm_bindgen(getter, js_name = schemaCount)]
    #[must_use]
    pub fn schema_count(&self) -> usize {
        self.inner.structs.len()
    }

    /// Get the number of alias definitions.
    #[wasm_bindgen(getter, js_name = aliasCount)]
    #[must_use]
    pub fn alias_count(&self) -> usize {
        self.inner.aliases.len()
    }

    /// Get the number of nest relationships.
    #[wasm_bindgen(getter, js_name = nestCount)]
    #[must_use]
    pub fn nest_count(&self) -> usize {
        self.inner.nests.len()
    }

    /// Get the number of root items.
    #[wasm_bindgen(getter, js_name = rootItemCount)]
    #[must_use]
    pub fn root_item_count(&self) -> usize {
        self.inner.root.len()
    }

    /// Get all schema names.
    #[wasm_bindgen(js_name = getSchemaNames)]
    #[must_use]
    pub fn get_schema_names(&self) -> Vec<String> {
        self.inner.structs.keys().cloned().collect()
    }

    /// Get schema columns for a type.
    #[wasm_bindgen(js_name = getSchema)]
    #[must_use]
    pub fn get_schema(&self, type_name: &str) -> Option<Vec<String>> {
        self.inner.structs.get(type_name).cloned()
    }

    /// Get all aliases as a JSON object.
    ///
    /// Returns a JavaScript object mapping alias names to their resolved values.
    /// Returns an empty object if there are no aliases.
    #[wasm_bindgen(js_name = getAliases)]
    #[must_use]
    pub fn get_aliases(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.aliases).unwrap_or(JsValue::NULL)
    }

    /// Get all nest relationships as a JSON object.
    ///
    /// Returns a JavaScript object mapping parent type names to arrays of child type names.
    /// Returns an empty object if there are no nest relationships.
    #[wasm_bindgen(js_name = getNests)]
    #[must_use]
    pub fn get_nests(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.nests).unwrap_or(JsValue::NULL)
    }

    /// Convert to JSON object.
    ///
    /// Returns the HEDL document as a structured JSON value that can be used
    /// directly in JavaScript. The returned value conforms to the `JsonValue` type,
    /// which is a recursive union of JSON primitives, objects, and arrays.
    ///
    /// # Feature
    /// Requires the "json" feature to be enabled.
    ///
    /// # Returns
    /// A `JsonValue` representing the complete document structure.
    #[cfg(feature = "json")]
    #[wasm_bindgen(js_name = toJson)]
    #[must_use]
    pub fn to_json(&self) -> JsValue {
        let config = ToJsonConfig::default();
        match to_json_value(&self.inner, &config) {
            Ok(json) => serde_wasm_bindgen::to_value(&json).unwrap_or(JsValue::NULL),
            Err(_) => JsValue::NULL,
        }
    }

    /// Convert to JSON string.
    ///
    /// # Feature
    /// Requires the "json" feature to be enabled.
    #[cfg(feature = "json")]
    #[wasm_bindgen(js_name = toJsonString)]
    pub fn to_json_string(&self, pretty: Option<bool>) -> Result<String, JsError> {
        let config = ToJsonConfig::default();
        let json = to_json_value(&self.inner, &config).map_err(|e| JsError::new(&e))?;

        if pretty.unwrap_or(true) {
            serde_json::to_string_pretty(&json).map_err(|e| JsError::new(&e.to_string()))
        } else {
            serde_json::to_string(&json).map_err(|e| JsError::new(&e.to_string()))
        }
    }

    /// Canonicalize to HEDL string.
    ///
    /// Uses v2.0 canonical format (no ditto optimization).
    #[wasm_bindgen(js_name = toHedl)]
    pub fn to_hedl(&self) -> Result<String, JsError> {
        let config = CanonicalConfig::default();
        hedl_c14n::canonicalize_with_config(&self.inner, &config)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Count entities by type.
    #[wasm_bindgen(js_name = countEntities)]
    #[must_use]
    pub fn count_entities(&self) -> JsValue {
        let mut counts: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();

        for item in self.inner.root.values() {
            count_item_entities(item, &mut counts);
        }

        serde_wasm_bindgen::to_value(&counts).unwrap_or(JsValue::NULL)
    }

    /// Query entities by type and optional ID.
    ///
    /// Returns an array of `EntityResult` objects matching the specified criteria.
    /// Each result contains the entity type, ID, and field values as a `JsonValue` map.
    ///
    /// # Arguments
    /// * `type_name` - Optional type filter (e.g., "User"). If None, matches all types.
    /// * `id` - Optional ID filter. If None, matches all IDs.
    ///
    /// # Returns
    /// Array of `EntityResult` objects with properly typed fields (`JsonValue` instead of any).
    ///
    /// # Feature
    /// Requires the "query-api" feature to be enabled.
    #[cfg(feature = "query-api")]
    #[wasm_bindgen]
    #[must_use]
    pub fn query(&self, type_name: Option<String>, id: Option<String>) -> JsValue {
        let mut results = Vec::new();

        for item in self.inner.root.values() {
            find_entities(item, &type_name, &id, &mut results);
        }

        serde_wasm_bindgen::to_value(&results).unwrap_or(JsValue::NULL)
    }
}

// --- Main API Functions ---

/// Parse a HEDL string and return a document.
///
/// # Arguments
/// * `input` - HEDL document string
///
/// # Errors
/// Returns an error if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - Parsing fails due to syntax errors
/// - Inline child list syntax errors:
///   - Count mismatch between declared and actual children
///   - Invalid count format (non-numeric)
///   - Missing required separators (`#` or `:|`)
///   - Invalid type names in inline children
///   - Undefined child types or missing NEST relationships
///
/// Use `setMaxInputSize()` to increase the size limit for larger documents.
///
/// # Inline Child Lists
/// Inline child list syntax: `@TypeName#count:|child1|child2|...`
/// Example: `@Comment#2:|c1,Good|c2,Bad`
///
/// This feature allows compact representation of child entities directly within
/// a parent row, useful for small numbers of children (typically ≤3).
#[wasm_bindgen]
pub fn parse(input: &str) -> Result<HedlDocument, JsError> {
    check_input_size(input)?;
    core_parse(input.as_bytes())
        .map(|doc| HedlDocument { inner: doc })
        .map_err(|e| JsError::new(&format!("Parse error at line {}: {}", e.line, e.message)))
}

/// Convert HEDL string to JSON.
///
/// # Arguments
/// * `hedl` - HEDL document string
/// * `pretty` - Whether to pretty-print the JSON (default: true)
///
/// # Errors
/// Returns an error if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - Parsing or conversion fails
///
/// # Feature
/// Requires the "json" feature to be enabled.
#[cfg(feature = "json")]
#[wasm_bindgen(js_name = toJson)]
pub fn to_json(hedl: &str, pretty: Option<bool>) -> Result<String, JsError> {
    check_input_size(hedl)?;
    let doc = core_parse(hedl.as_bytes())
        .map_err(|e| JsError::new(&format!("Parse error: {}", e.message)))?;

    let config = ToJsonConfig::default();
    let json = to_json_value(&doc, &config).map_err(|e| JsError::new(&e))?;

    if pretty.unwrap_or(true) {
        serde_json::to_string_pretty(&json).map_err(|e| JsError::new(&e.to_string()))
    } else {
        serde_json::to_string(&json).map_err(|e| JsError::new(&e.to_string()))
    }
}

/// Convert JSON string to HEDL.
///
/// # Arguments
/// * `json` - JSON string to convert
///
/// # Errors
/// Returns an error if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - JSON parsing or conversion fails
///
/// # Feature
/// Requires the "json" feature to be enabled.
#[cfg(feature = "json")]
#[wasm_bindgen(js_name = fromJson)]
pub fn from_json(json: &str) -> Result<String, JsError> {
    check_input_size(json)?;
    let json_value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| JsError::new(&format!("Invalid JSON: {e}")))?;

    let config = FromJsonConfig::default();
    let doc = from_json_value(&json_value, &config)
        .map_err(|e| JsError::new(&format!("Conversion error: {e}")))?;

    let c14n_config = CanonicalConfig::default();
    hedl_c14n::canonicalize_with_config(&doc, &c14n_config)
        .map_err(|e| JsError::new(&format!("Format error: {e}")))
}

// --- YAML Conversion ---

/// Convert HEDL string to YAML.
///
/// # Arguments
/// * `hedl` - HEDL document string
///
/// # Errors
/// Returns an error if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - Parsing or conversion fails
///
/// # Feature
/// Requires the "yaml" feature to be enabled.
#[cfg(feature = "yaml")]
#[wasm_bindgen(js_name = toYaml)]
pub fn to_yaml(hedl: &str) -> Result<String, JsError> {
    check_input_size(hedl)?;
    let doc = core_parse(hedl.as_bytes())
        .map_err(|e| JsError::new(&format!("Parse error: {}", e.message)))?;

    let config = hedl_yaml::ToYamlConfig::default();
    hedl_yaml::to_yaml(&doc, &config)
        .map_err(|e| JsError::new(&format!("YAML conversion error: {e}")))
}

/// Convert YAML string to HEDL.
///
/// # Arguments
/// * `yaml` - YAML string to convert
///
/// # Errors
/// Returns an error if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - YAML parsing or conversion fails
///
/// # Feature
/// Requires the "yaml" feature to be enabled.
#[cfg(feature = "yaml")]
#[wasm_bindgen(js_name = fromYaml)]
pub fn from_yaml(yaml: &str) -> Result<String, JsError> {
    check_input_size(yaml)?;

    let config = hedl_yaml::FromYamlConfig::default();
    let doc = hedl_yaml::from_yaml(yaml, &config)
        .map_err(|e| JsError::new(&format!("YAML parse error: {e}")))?;

    let c14n_config = CanonicalConfig::default();
    hedl_c14n::canonicalize_with_config(&doc, &c14n_config)
        .map_err(|e| JsError::new(&format!("Format error: {e}")))
}

// --- XML Conversion ---

/// Convert HEDL string to XML.
///
/// # Arguments
/// * `hedl` - HEDL document string
///
/// # Errors
/// Returns an error if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - Parsing or conversion fails
///
/// # Feature
/// Requires the "xml" feature to be enabled.
#[cfg(feature = "xml")]
#[wasm_bindgen(js_name = toXml)]
pub fn to_xml(hedl: &str) -> Result<String, JsError> {
    check_input_size(hedl)?;
    let doc = core_parse(hedl.as_bytes())
        .map_err(|e| JsError::new(&format!("Parse error: {}", e.message)))?;

    let config = hedl_xml::ToXmlConfig::default();
    hedl_xml::to_xml(&doc, &config).map_err(|e| JsError::new(&format!("XML conversion error: {e}")))
}

/// Convert XML string to HEDL.
///
/// # Arguments
/// * `xml` - XML string to convert
///
/// # Errors
/// Returns an error if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - XML parsing or conversion fails
///
/// # Feature
/// Requires the "xml" feature to be enabled.
#[cfg(feature = "xml")]
#[wasm_bindgen(js_name = fromXml)]
pub fn from_xml(xml: &str) -> Result<String, JsError> {
    check_input_size(xml)?;

    let config = hedl_xml::FromXmlConfig::default();
    let doc = hedl_xml::from_xml(xml, &config)
        .map_err(|e| JsError::new(&format!("XML parse error: {e}")))?;

    let c14n_config = CanonicalConfig::default();
    hedl_c14n::canonicalize_with_config(&doc, &c14n_config)
        .map_err(|e| JsError::new(&format!("Format error: {e}")))
}

// --- CSV Conversion ---

/// Convert HEDL string to CSV.
///
/// # Arguments
/// * `hedl` - HEDL document string
///
/// # Errors
/// Returns an error if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - Parsing or conversion fails
///
/// # Feature
/// Requires the "csv" feature to be enabled.
#[cfg(feature = "csv")]
#[wasm_bindgen(js_name = toCsv)]
pub fn to_csv(hedl: &str) -> Result<String, JsError> {
    check_input_size(hedl)?;
    let doc = core_parse(hedl.as_bytes())
        .map_err(|e| JsError::new(&format!("Parse error: {}", e.message)))?;

    hedl_csv::to_csv(&doc).map_err(|e| JsError::new(&format!("CSV conversion error: {e}")))
}

/// Convert CSV string to HEDL.
///
/// The CSV must have a header row. Column names from the header become the schema.
/// The type name defaults to "Row" but can be customized.
///
/// # Arguments
/// * `csv` - CSV string to convert (must have header row)
/// * `type_name` - Optional type name for entities (default: "Row")
///
/// # Errors
/// Returns an error if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - CSV parsing or conversion fails
/// - CSV has no header row
///
/// # Feature
/// Requires the "csv" feature to be enabled.
#[cfg(feature = "csv")]
#[wasm_bindgen(js_name = fromCsv)]
pub fn from_csv(csv: &str, type_name: Option<String>) -> Result<String, JsError> {
    check_input_size(csv)?;

    // Parse header row to get schema (excluding 'id' column which is added automatically)
    let mut lines = csv.lines();
    let header = lines
        .next()
        .ok_or_else(|| JsError::new("CSV must have a header row"))?;
    let all_columns: Vec<&str> = header.split(',').map(str::trim).collect();

    // hedl_csv::from_csv expects schema WITHOUT the 'id' column (it prepends 'id' automatically)
    // Skip the first column if it's named 'id'
    let schema: Vec<&str> =
        if all_columns.first().map(|s| s.to_lowercase()) == Some("id".to_string()) {
            all_columns[1..].to_vec()
        } else {
            all_columns
        };

    let type_name = type_name.unwrap_or_else(|| "Row".to_string());

    let doc = hedl_csv::from_csv(csv, &type_name, &schema)
        .map_err(|e| JsError::new(&format!("CSV parse error: {e}")))?;

    let c14n_config = CanonicalConfig::default();
    hedl_c14n::canonicalize_with_config(&doc, &c14n_config)
        .map_err(|e| JsError::new(&format!("Format error: {e}")))
}

// --- TOON Conversion ---

/// Convert HEDL string to TOON.
///
/// TOON (Typed Object Outline Notation) is an external format specification
/// for human-readable data serialization.
///
/// # Arguments
/// * `hedl` - HEDL document string
///
/// # Errors
/// Returns an error if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - Parsing or conversion fails
///
/// # Feature
/// Requires the "toon" feature to be enabled.
#[cfg(feature = "toon")]
#[wasm_bindgen(js_name = toToon)]
pub fn to_toon(hedl: &str) -> Result<String, JsError> {
    check_input_size(hedl)?;
    let doc = core_parse(hedl.as_bytes())
        .map_err(|e| JsError::new(&format!("Parse error: {}", e.message)))?;

    hedl_toon::hedl_to_toon(&doc).map_err(|e| JsError::new(&format!("TOON conversion error: {e}")))
}

/// Convert TOON string to HEDL.
///
/// # Arguments
/// * `toon` - TOON string to convert
///
/// # Errors
/// Returns an error if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - TOON parsing or conversion fails
///
/// # Feature
/// Requires the "toon" feature to be enabled.
#[cfg(feature = "toon")]
#[wasm_bindgen(js_name = fromToon)]
pub fn from_toon(toon: &str) -> Result<String, JsError> {
    check_input_size(toon)?;

    let doc = hedl_toon::toon_to_hedl(toon)
        .map_err(|e| JsError::new(&format!("TOON parse error: {e}")))?;

    let c14n_config = CanonicalConfig::default();
    hedl_c14n::canonicalize_with_config(&doc, &c14n_config)
        .map_err(|e| JsError::new(&format!("Format error: {e}")))
}

/// Format HEDL to canonical form.
///
/// # Arguments
/// * `hedl` - HEDL document string
///
/// # Errors
/// Returns an error if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - Parsing or formatting fails
#[wasm_bindgen]
pub fn format(hedl: &str) -> Result<String, JsError> {
    check_input_size(hedl)?;
    let doc = core_parse(hedl.as_bytes())
        .map_err(|e| JsError::new(&format!("Parse error: {}", e.message)))?;

    let config = CanonicalConfig::default();
    hedl_c14n::canonicalize_with_config(&doc, &config)
        .map_err(|e| JsError::new(&format!("Format error: {e}")))
}

// --- Validation ---

/// Validate HEDL and return detailed diagnostics.
///
/// # Arguments
/// * `hedl` - HEDL document string
/// * `run_lint` - Run linting rules (default: true, only available with full-validation feature)
///
/// # Errors
/// Returns validation result with errors if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - Parsing fails due to syntax errors
/// - Linting detects errors (if enabled and full-validation feature is active)
#[wasm_bindgen]
#[must_use]
pub fn validate(hedl: &str, run_lint: Option<bool>) -> JsValue {
    // Check input size first
    if let Err(e) = check_input_size(hedl) {
        let result =
            ValidationResult::with_error(0, format!("{e:?}"), "InputSizeError".to_string());
        return serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL);
    }

    let mut result = ValidationResult::new();

    match core_parse(hedl.as_bytes()) {
        Ok(_doc) => {
            #[cfg(feature = "full-validation")]
            {
                if run_lint.unwrap_or(true) {
                    let diagnostics = lint(&_doc);

                    for diag in diagnostics {
                        match diag.severity() {
                            hedl_lint::Severity::Error => {
                                result.valid = false;
                                result.errors.push(ValidationError {
                                    line: diag.line().unwrap_or(0),
                                    message: diag.message().to_string(),
                                    error_type: diag.rule_id().to_string(),
                                });
                            }
                            hedl_lint::Severity::Warning | hedl_lint::Severity::Hint => {
                                result.warnings.push(ValidationWarning {
                                    line: diag.line().unwrap_or(0),
                                    message: diag.message().to_string(),
                                    rule: diag.rule_id().to_string(),
                                });
                            }
                        }
                    }
                }
            }

            #[cfg(not(feature = "full-validation"))]
            {
                let _ = run_lint; // Suppress unused variable warning
                                  // Minimal validation - syntax only (already done by parsing)
            }
        }
        Err(e) => {
            result.valid = false;
            result.errors.push(ValidationError {
                line: e.line,
                message: e.message,
                error_type: format!("{:?}", e.kind),
            });
        }
    }

    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

// --- Statistics ---

/// Get token usage statistics.
///
/// # Arguments
/// * `hedl` - HEDL document string
///
/// # Errors
/// Returns an error if:
/// - Input exceeds the configured maximum size (default: 500 MB)
/// - Parsing fails
///
/// # Feature
/// Requires the "statistics" feature to be enabled.
#[cfg(feature = "statistics")]
#[wasm_bindgen(js_name = getStats)]
pub fn get_stats(hedl: &str) -> Result<JsValue, JsError> {
    check_input_size(hedl)?;
    let doc = core_parse(hedl.as_bytes())
        .map_err(|e| JsError::new(&format!("Parse error: {}", e.message)))?;

    let config = ToJsonConfig::default();
    let json_value = to_json_value(&doc, &config).map_err(|e| JsError::new(&e))?;
    let json_str = serde_json::to_string(&json_value).map_err(|e| JsError::new(&e.to_string()))?;

    let hedl_tokens = estimate_tokens(hedl);
    let json_tokens = estimate_tokens(&json_str);

    let savings_percent = if json_tokens > 0 {
        ((json_tokens as i64 - hedl_tokens as i64) * 100 / json_tokens as i64) as i32
    } else {
        0
    };

    let stats = TokenStats {
        hedl_bytes: hedl.len(),
        hedl_tokens,
        hedl_lines: hedl.lines().count(),
        json_bytes: json_str.len(),
        json_tokens,
        savings_percent,
        tokens_saved: (json_tokens as i32) - (hedl_tokens as i32),
    };

    serde_wasm_bindgen::to_value(&stats).map_err(|e| JsError::new(&e.to_string()))
}

// --- Live Token Counter ---

/// Compare HEDL and JSON token counts in real-time.
///
/// # Feature
/// Requires the "token-tools" feature to be enabled.
#[cfg(feature = "token-tools")]
#[wasm_bindgen(js_name = compareTokens)]
#[must_use]
pub fn compare_tokens(hedl: &str, json: &str) -> JsValue {
    let hedl_tokens = estimate_tokens(hedl);
    let json_tokens = estimate_tokens(json);

    let savings = if json_tokens > 0 {
        ((json_tokens as i64 - hedl_tokens as i64) * 100 / json_tokens as i64) as i32
    } else {
        0
    };

    let result = serde_json::json!({
        "hedl": {
            "bytes": hedl.len(),
            "tokens": hedl_tokens,
            "lines": hedl.lines().count()
        },
        "json": {
            "bytes": json.len(),
            "tokens": json_tokens
        },
        "savings": {
            "percent": savings,
            "tokens": json_tokens as i32 - hedl_tokens as i32
        }
    });

    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

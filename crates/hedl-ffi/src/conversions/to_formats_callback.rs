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

//! Zero-copy export functions using callback pattern for large outputs.
//!
//! These functions avoid allocating memory for large string outputs (>1MB)
//! by using a callback mechanism that allows the caller to process the data
//! in-place without copying.
//!
//! # Callback Pattern
//!
//! The callback signature is:
//! ```c
//! typedef void (*hedl_output_callback)(const char* data, size_t len, void* user_data);
//! ```
//!
//! The callback receives:
//! - `data`: Pointer to the output data (valid only during the callback)
//! - `len`: Length of the data in bytes
//! - `user_data`: User-provided context pointer
//!
//! # Memory Management
//!
//! **CRITICAL**: The data pointer is only valid during the callback execution.
//! Do NOT store the pointer for later use. If you need to keep the data,
//! copy it within the callback.
//!
//! # Usage Example (C)
//!
//! ```c
//! void my_callback(const char* data, size_t len, void* user_data) {
//!     FILE* f = (FILE*)user_data;
//!     fwrite(data, 1, len, f);
//! }
//!
//! FILE* output = fopen("output.json", "w");
//! int result = hedl_to_json_callback(doc, 0, my_callback, output);
//! fclose(output);
//! ```

use crate::error::{clear_error, set_error};
use crate::memory::is_valid_document_ptr;
use crate::types::{
    HedlDocument, HEDL_ERR_CSV, HEDL_ERR_JSON, HEDL_ERR_NEO4J, HEDL_ERR_NULL_PTR, HEDL_ERR_XML,
    HEDL_ERR_YAML, HEDL_OK,
};
use std::os::raw::{c_char, c_int, c_void};

// =============================================================================
// Callback Type Definition
// =============================================================================

/// Output callback function type for zero-copy string return.
///
/// # Safety
/// - The `data` pointer is only valid during the callback execution
/// - Do NOT store the pointer for later use
/// - The data is not null-terminated
/// - The callback MUST NOT call back into HEDL functions
pub type HedlOutputCallback = unsafe extern "C" fn(data: *const c_char, len: usize, user_data: *mut c_void);

// =============================================================================
// Helper Functions
// =============================================================================

/// Helper to invoke callback with output data
#[inline]
unsafe fn invoke_callback(
    output: &str,
    callback: HedlOutputCallback,
    user_data: *mut c_void,
) {
    let data = output.as_ptr() as *const c_char;
    let len = output.len();
    callback(data, len, user_data);
}

// =============================================================================
// JSON Conversion with Callback
// =============================================================================

/// Convert a HEDL document to JSON using zero-copy callback pattern.
///
/// For outputs >1MB, this avoids memory allocation by passing data directly
/// to the callback. For smaller outputs, consider using `hedl_to_json`.
///
/// # Arguments
/// * `doc` - Document handle from hedl_parse
/// * `include_metadata` - Non-zero to include HEDL metadata (__type__, __schema__)
/// * `callback` - Function to receive the output data
/// * `user_data` - User context pointer passed to callback
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// - All pointers must be valid
/// - The callback MUST NOT call back into HEDL functions
/// - The data pointer passed to callback is only valid during the callback
///
/// # Feature
/// Requires the "json" feature to be enabled.
#[cfg(feature = "json")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_json_callback(
    doc: *const HedlDocument,
    include_metadata: c_int,
    callback: HedlOutputCallback,
    user_data: *mut c_void,
) -> c_int {
    use crate::audit::{audit_call_failure, audit_call_start, audit_call_success, sanitize_pointer};
    use std::time::Instant;
    let start = Instant::now();
    let doc_ptr_str = sanitize_pointer(doc);
    let include_metadata_str = include_metadata.to_string();
    audit_call_start("hedl_to_json_callback", &[
        ("doc_ptr", &doc_ptr_str),
        ("include_metadata", &include_metadata_str),
    ]);

    clear_error();

    if !is_valid_document_ptr(doc) {
        set_error("Null or invalid document pointer");
        let duration = start.elapsed();
        audit_call_failure("hedl_to_json_callback", HEDL_ERR_NULL_PTR, "NULL or invalid pointer", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let doc_ref = &(*doc).inner;
    let config = hedl_json::ToJsonConfig {
        include_metadata: include_metadata != 0,
        ..Default::default()
    };

    match hedl_json::to_json(doc_ref, &config) {
        Ok(json) => {
            invoke_callback(&json, callback, user_data);
            audit_call_success("hedl_to_json_callback", start.elapsed());
            HEDL_OK
        }
        Err(e) => {
            set_error(&format!("JSON conversion error: {}", e));
            let duration = start.elapsed();
            let msg = e.to_string();
            audit_call_failure("hedl_to_json_callback", HEDL_ERR_JSON, &msg, duration);
            HEDL_ERR_JSON
        }
    }
}

// =============================================================================
// YAML Conversion with Callback
// =============================================================================

/// Convert a HEDL document to YAML using zero-copy callback pattern.
///
/// For outputs >1MB, this avoids memory allocation by passing data directly
/// to the callback. For smaller outputs, consider using `hedl_to_yaml`.
///
/// # Arguments
/// * `doc` - Document handle from hedl_parse
/// * `include_metadata` - Non-zero to include HEDL metadata
/// * `callback` - Function to receive the output data
/// * `user_data` - User context pointer passed to callback
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// - All pointers must be valid
/// - The callback MUST NOT call back into HEDL functions
/// - The data pointer passed to callback is only valid during the callback
///
/// # Feature
/// Requires the "yaml" feature to be enabled.
#[cfg(feature = "yaml")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_yaml_callback(
    doc: *const HedlDocument,
    include_metadata: c_int,
    callback: HedlOutputCallback,
    user_data: *mut c_void,
) -> c_int {
    use crate::audit::{audit_call_failure, audit_call_start, audit_call_success, sanitize_pointer};
    use std::time::Instant;
    let start = Instant::now();
    let doc_ptr_str = sanitize_pointer(doc);
    let include_metadata_str = include_metadata.to_string();
    audit_call_start("hedl_to_yaml_callback", &[
        ("doc_ptr", &doc_ptr_str),
        ("include_metadata", &include_metadata_str),
    ]);

    clear_error();

    if !is_valid_document_ptr(doc) {
        set_error("Null or invalid document pointer");
        let duration = start.elapsed();
        audit_call_failure("hedl_to_yaml_callback", HEDL_ERR_NULL_PTR, "NULL or invalid pointer", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let doc_ref = &(*doc).inner;
    let config = hedl_yaml::ToYamlConfig {
        include_metadata: include_metadata != 0,
        ..Default::default()
    };

    match hedl_yaml::to_yaml(doc_ref, &config) {
        Ok(yaml) => {
            invoke_callback(&yaml, callback, user_data);
            audit_call_success("hedl_to_yaml_callback", start.elapsed());
            HEDL_OK
        }
        Err(e) => {
            set_error(&format!("YAML conversion error: {}", e));
            let duration = start.elapsed();
            let msg = e.to_string();
            audit_call_failure("hedl_to_yaml_callback", HEDL_ERR_JSON, &msg, duration);
            HEDL_ERR_YAML
        }
    }
}

// =============================================================================
// XML Conversion with Callback
// =============================================================================

/// Convert a HEDL document to XML using zero-copy callback pattern.
///
/// For outputs >1MB, this avoids memory allocation by passing data directly
/// to the callback. For smaller outputs, consider using `hedl_to_xml`.
///
/// # Arguments
/// * `doc` - Document handle from hedl_parse
/// * `callback` - Function to receive the output data
/// * `user_data` - User context pointer passed to callback
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// - All pointers must be valid
/// - The callback MUST NOT call back into HEDL functions
/// - The data pointer passed to callback is only valid during the callback
///
/// # Feature
/// Requires the "xml" feature to be enabled.
#[cfg(feature = "xml")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_xml_callback(
    doc: *const HedlDocument,
    callback: HedlOutputCallback,
    user_data: *mut c_void,
) -> c_int {
    use crate::audit::{audit_call_failure, audit_call_start, audit_call_success, sanitize_pointer};
    use std::time::Instant;
    let start = Instant::now();
    let doc_ptr_str = sanitize_pointer(doc);
    audit_call_start("hedl_to_xml_callback", &[("doc_ptr", &doc_ptr_str)]);

    clear_error();

    if !is_valid_document_ptr(doc) {
        set_error("Null or invalid document pointer");
        let duration = start.elapsed();
        audit_call_failure("hedl_to_xml_callback", HEDL_ERR_NULL_PTR, "NULL or invalid pointer", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let doc_ref = &(*doc).inner;

    match hedl_xml::hedl_to_xml(doc_ref) {
        Ok(xml) => {
            invoke_callback(&xml, callback, user_data);
            audit_call_success("hedl_to_xml_callback", start.elapsed());
            HEDL_OK
        }
        Err(e) => {
            set_error(&format!("XML conversion error: {}", e));
            let duration = start.elapsed();
            let msg = e.to_string();
            audit_call_failure("hedl_to_xml_callback", HEDL_ERR_JSON, &msg, duration);
            HEDL_ERR_XML
        }
    }
}

// =============================================================================
// CSV Conversion with Callback
// =============================================================================

/// Convert a HEDL document to CSV using zero-copy callback pattern.
///
/// Note: Only works for documents with matrix lists.
///
/// For outputs >1MB, this avoids memory allocation by passing data directly
/// to the callback. For smaller outputs, consider using `hedl_to_csv`.
///
/// # Arguments
/// * `doc` - Document handle from hedl_parse
/// * `callback` - Function to receive the output data
/// * `user_data` - User context pointer passed to callback
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// - All pointers must be valid
/// - The callback MUST NOT call back into HEDL functions
/// - The data pointer passed to callback is only valid during the callback
///
/// # Feature
/// Requires the "csv" feature to be enabled.
#[cfg(feature = "csv")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_csv_callback(
    doc: *const HedlDocument,
    callback: HedlOutputCallback,
    user_data: *mut c_void,
) -> c_int {
    use crate::audit::{audit_call_failure, audit_call_start, audit_call_success, sanitize_pointer};
    use std::time::Instant;
    let start = Instant::now();
    let doc_ptr_str = sanitize_pointer(doc);
    audit_call_start("hedl_to_csv_callback", &[("doc_ptr", &doc_ptr_str)]);

    clear_error();

    if !is_valid_document_ptr(doc) {
        set_error("Null or invalid document pointer");
        let duration = start.elapsed();
        audit_call_failure("hedl_to_csv_callback", HEDL_ERR_NULL_PTR, "NULL or invalid pointer", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let doc_ref = &(*doc).inner;

    match hedl_csv::to_csv(doc_ref) {
        Ok(csv) => {
            invoke_callback(&csv, callback, user_data);
            audit_call_success("hedl_to_csv_callback", start.elapsed());
            HEDL_OK
        }
        Err(e) => {
            set_error(&format!("CSV conversion error: {}", e));
            let duration = start.elapsed();
            let msg = e.to_string();
            audit_call_failure("hedl_to_csv_callback", HEDL_ERR_JSON, &msg, duration);
            HEDL_ERR_CSV
        }
    }
}

// =============================================================================
// Neo4j/Cypher Conversion with Callback
// =============================================================================

/// Convert a HEDL document to Cypher queries using zero-copy callback pattern.
///
/// Generates CREATE/MERGE statements, constraints, and relationships.
///
/// For outputs >1MB, this avoids memory allocation by passing data directly
/// to the callback. For smaller outputs, consider using `hedl_to_neo4j_cypher`.
///
/// # Arguments
/// * `doc` - Document handle from hedl_parse
/// * `use_merge` - Non-zero to use MERGE (idempotent), zero for CREATE
/// * `callback` - Function to receive the output data
/// * `user_data` - User context pointer passed to callback
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// - All pointers must be valid
/// - The callback MUST NOT call back into HEDL functions
/// - The data pointer passed to callback is only valid during the callback
///
/// # Feature
/// Requires the "neo4j" feature to be enabled.
#[cfg(feature = "neo4j")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_neo4j_cypher_callback(
    doc: *const HedlDocument,
    use_merge: c_int,
    callback: HedlOutputCallback,
    user_data: *mut c_void,
) -> c_int {
    use crate::audit::{audit_call_failure, audit_call_start, audit_call_success, sanitize_pointer};
    use std::time::Instant;
    let start = Instant::now();
    let doc_ptr_str = sanitize_pointer(doc);
    let use_merge_str = use_merge.to_string();
    audit_call_start("hedl_to_neo4j_cypher_callback", &[
        ("doc_ptr", &doc_ptr_str),
        ("use_merge", &use_merge_str),
    ]);

    clear_error();

    if !is_valid_document_ptr(doc) {
        set_error("Null or invalid document pointer");
        let duration = start.elapsed();
        audit_call_failure("hedl_to_neo4j_cypher_callback", HEDL_ERR_NULL_PTR, "NULL or invalid pointer", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let doc_ref = &(*doc).inner;
    let config = if use_merge != 0 {
        hedl_neo4j::ToCypherConfig::default()
    } else {
        hedl_neo4j::ToCypherConfig::new().with_create()
    };

    match hedl_neo4j::to_cypher(doc_ref, &config) {
        Ok(cypher) => {
            let cypher_str = cypher.to_string();
            invoke_callback(&cypher_str, callback, user_data);
            audit_call_success("hedl_to_neo4j_cypher_callback", start.elapsed());
            HEDL_OK
        }
        Err(e) => {
            set_error(&format!("Neo4j conversion error: {}", e));
            let duration = start.elapsed();
            let msg = e.to_string();
            audit_call_failure("hedl_to_neo4j_cypher_callback", HEDL_ERR_JSON, &msg, duration);
            HEDL_ERR_NEO4J
        }
    }
}

// =============================================================================
// Canonicalize with Callback
// =============================================================================

/// Canonicalize a HEDL document using zero-copy callback pattern.
///
/// For outputs >1MB, this avoids memory allocation by passing data directly
/// to the callback. For smaller outputs, consider using `hedl_canonicalize`.
///
/// # Arguments
/// * `doc` - Document handle from hedl_parse
/// * `callback` - Function to receive the output data
/// * `user_data` - User context pointer passed to callback
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// - All pointers must be valid
/// - The callback MUST NOT call back into HEDL functions
/// - The data pointer passed to callback is only valid during the callback
#[no_mangle]
pub unsafe extern "C" fn hedl_canonicalize_callback(
    doc: *const HedlDocument,
    callback: HedlOutputCallback,
    user_data: *mut c_void,
) -> c_int {
    use crate::audit::{audit_call_failure, audit_call_start, audit_call_success, sanitize_pointer};
    use std::time::Instant;
    let start = Instant::now();
    let doc_ptr_str = sanitize_pointer(doc);
    audit_call_start("hedl_canonicalize_callback", &[("doc_ptr", &doc_ptr_str)]);

    clear_error();

    if !is_valid_document_ptr(doc) {
        set_error("Null or invalid document pointer");
        let duration = start.elapsed();
        audit_call_failure("hedl_canonicalize_callback", HEDL_ERR_NULL_PTR, "NULL or invalid pointer", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let doc_ref = &(*doc).inner;

    match hedl_c14n::canonicalize(doc_ref) {
        Ok(canonical) => {
            invoke_callback(&canonical, callback, user_data);
            audit_call_success("hedl_canonicalize_callback", start.elapsed());
            HEDL_OK
        }
        Err(e) => {
            set_error(&format!("Canonicalization error: {}", e));
            let duration = start.elapsed();
            let msg = e.to_string();
            audit_call_failure("hedl_canonicalize_callback", HEDL_ERR_JSON, &msg, duration);
            crate::types::HEDL_ERR_CANONICALIZE
        }
    }
}

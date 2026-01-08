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

//! Import functions (from_*) for FFI.

use crate::error::{clear_error, set_error};
use crate::types::{
    HedlDocument, HEDL_ERR_JSON, HEDL_ERR_NULL_PTR, HEDL_ERR_PARQUET, HEDL_ERR_XML, HEDL_ERR_YAML,
    HEDL_OK,
};
use crate::utils::get_input_string;
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::slice;

// =============================================================================
// JSON Conversion (requires "json" feature)
// =============================================================================

/// Parse JSON into a HEDL document.
///
/// # Arguments
/// * `json` - UTF-8 encoded JSON string
/// * `json_len` - Length of input in bytes, or -1 for null-terminated
/// * `out_doc` - Pointer to store document handle
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// All pointers must be valid.
///
/// # Feature
/// Requires the "json" feature to be enabled.
#[cfg(feature = "json")]
#[no_mangle]
pub unsafe extern "C" fn hedl_from_json(
    json: *const c_char,
    json_len: c_int,
    out_doc: *mut *mut HedlDocument,
) -> c_int {
    use crate::audit::{audit_call_failure, audit_call_start, audit_call_success, sanitize_pointer};
    use std::time::Instant;

    let start = Instant::now();
    let json_ptr_str = sanitize_pointer(json);
    let json_len_str = json_len.to_string();
    audit_call_start("hedl_from_json", &[
        ("json_ptr", &json_ptr_str),
        ("json_len", &json_len_str),
    ]);

    clear_error();

    if json.is_null() || out_doc.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure("hedl_from_json", HEDL_ERR_NULL_PTR, "NULL pointer", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let json_str = match get_input_string(json, json_len) {
        Ok(s) => s,
        Err(code) => {
            let duration = start.elapsed();
            let msg = crate::error::get_thread_local_error();
            audit_call_failure("hedl_from_json", code, &msg, duration);
            return code;
        }
    };

    match hedl_json::json_to_hedl(&json_str) {
        Ok(doc) => {
            let handle = Box::new(HedlDocument { inner: doc });
            *out_doc = Box::into_raw(handle);
            audit_call_success("hedl_from_json", start.elapsed());
            HEDL_OK
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("JSON parse error: {}", e);
            set_error(&msg);
            *out_doc = ptr::null_mut();
            audit_call_failure("hedl_from_json", HEDL_ERR_JSON, &msg, duration);
            HEDL_ERR_JSON
        }
    }
}

// =============================================================================
// YAML Conversion (requires "yaml" feature)
// =============================================================================

/// Parse YAML into a HEDL document.
///
/// # Arguments
/// * `yaml` - UTF-8 encoded YAML string
/// * `yaml_len` - Length of input in bytes, or -1 for null-terminated
/// * `out_doc` - Pointer to store document handle
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// All pointers must be valid.
///
/// # Feature
/// Requires the "yaml" feature to be enabled.
#[cfg(feature = "yaml")]
#[no_mangle]
pub unsafe extern "C" fn hedl_from_yaml(
    yaml: *const c_char,
    yaml_len: c_int,
    out_doc: *mut *mut HedlDocument,
) -> c_int {
    use crate::audit::{audit_call_failure, audit_call_start, audit_call_success, sanitize_pointer};
    use std::time::Instant;

    let start = Instant::now();
    let yaml_ptr_str = sanitize_pointer(yaml);
    let yaml_len_str = yaml_len.to_string();
    audit_call_start("hedl_from_yaml", &[("yaml_ptr", &yaml_ptr_str), ("yaml_len", &yaml_len_str)]);

    clear_error();

    if yaml.is_null() || out_doc.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure("hedl_from_yaml", HEDL_ERR_NULL_PTR, "NULL pointer", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let yaml_str = match get_input_string(yaml, yaml_len) {
        Ok(s) => s,
        Err(code) => {
            let duration = start.elapsed();
            let msg = crate::error::get_thread_local_error();
            audit_call_failure("hedl_from_yaml", code, &msg, duration);
            return code;
        }
    };

    match hedl_yaml::yaml_to_hedl(&yaml_str) {
        Ok(doc) => {
            let handle = Box::new(HedlDocument { inner: doc });
            *out_doc = Box::into_raw(handle);
            audit_call_success("hedl_from_yaml", start.elapsed());
            HEDL_OK
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("YAML parse error: {}", e);
            set_error(&msg);
            *out_doc = ptr::null_mut();
            audit_call_failure("hedl_from_yaml", HEDL_ERR_YAML, &msg, duration);
            HEDL_ERR_YAML
        }
    }
}

// =============================================================================
// XML Conversion (requires "xml" feature)
// =============================================================================

/// Parse XML into a HEDL document.
///
/// # Arguments
/// * `xml` - UTF-8 encoded XML string
/// * `xml_len` - Length of input in bytes, or -1 for null-terminated
/// * `out_doc` - Pointer to store document handle
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// All pointers must be valid.
///
/// # Feature
/// Requires the "xml" feature to be enabled.
#[cfg(feature = "xml")]
#[no_mangle]
pub unsafe extern "C" fn hedl_from_xml(
    xml: *const c_char,
    xml_len: c_int,
    out_doc: *mut *mut HedlDocument,
) -> c_int {
    use crate::audit::{audit_call_failure, audit_call_start, audit_call_success, sanitize_pointer};
    use std::time::Instant;

    let start = Instant::now();
    let xml_ptr_str = sanitize_pointer(xml);
    let xml_len_str = xml_len.to_string();
    audit_call_start("hedl_from_xml", &[("xml_ptr", &xml_ptr_str), ("xml_len", &xml_len_str)]);

    clear_error();

    if xml.is_null() || out_doc.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure("hedl_from_xml", HEDL_ERR_NULL_PTR, "NULL pointer", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let xml_str = match get_input_string(xml, xml_len) {
        Ok(s) => s,
        Err(code) => {
            let duration = start.elapsed();
            let msg = crate::error::get_thread_local_error();
            audit_call_failure("hedl_from_xml", code, &msg, duration);
            return code;
        }
    };

    match hedl_xml::xml_to_hedl(&xml_str) {
        Ok(doc) => {
            let handle = Box::new(HedlDocument { inner: doc });
            *out_doc = Box::into_raw(handle);
            audit_call_success("hedl_from_xml", start.elapsed());
            HEDL_OK
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("XML parse error: {}", e);
            set_error(&msg);
            *out_doc = ptr::null_mut();
            audit_call_failure("hedl_from_xml", HEDL_ERR_XML, &msg, duration);
            HEDL_ERR_XML
        }
    }
}

// =============================================================================
// Parquet Conversion (requires "parquet" feature)
// =============================================================================

/// Parse Parquet bytes into a HEDL document.
///
/// # Arguments
/// * `data` - Parquet file bytes
/// * `len` - Length of data
/// * `out_doc` - Pointer to store document handle
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// All pointers must be valid.
///
/// # Feature
/// Requires the "parquet" feature to be enabled.
#[cfg(feature = "parquet")]
#[no_mangle]
pub unsafe extern "C" fn hedl_from_parquet(
    data: *const u8,
    len: usize,
    out_doc: *mut *mut HedlDocument,
) -> c_int {
    use crate::audit::{audit_call_failure, audit_call_start, audit_call_success, sanitize_pointer};
    use std::time::Instant;

    let start = Instant::now();
    let data_ptr_str = sanitize_pointer(data);
    let len_str = len.to_string();
    audit_call_start("hedl_from_parquet", &[("data_ptr", &data_ptr_str), ("len", &len_str)]);

    clear_error();

    if data.is_null() || out_doc.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure("hedl_from_parquet", HEDL_ERR_NULL_PTR, "NULL pointer", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let bytes = slice::from_raw_parts(data, len);

    match hedl_parquet::from_parquet_bytes(bytes) {
        Ok(doc) => {
            let handle = Box::new(HedlDocument { inner: doc });
            *out_doc = Box::into_raw(handle);
            audit_call_success("hedl_from_parquet", start.elapsed());
            HEDL_OK
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("Parquet parse error: {}", e);
            set_error(&msg);
            *out_doc = ptr::null_mut();
            audit_call_failure("hedl_from_parquet", HEDL_ERR_PARQUET, &msg, duration);
            HEDL_ERR_PARQUET
        }
    }
}

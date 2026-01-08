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

//! Utility functions for FFI.

use crate::error::set_error;
use crate::types::{HEDL_ERR_INVALID_UTF8, HEDL_OK};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::slice;

// =============================================================================
// Helper Functions
// =============================================================================

/// Maximum input length accepted by FFI functions (1GB).
/// This prevents extreme values that could cause memory issues.
const MAX_FFI_INPUT_LEN: usize = 1024 * 1024 * 1024;

/// Helper to get input string from C pointer.
///
/// # Arguments
/// * `input` - Pointer to input string
/// * `input_len` - Length in bytes, or -1 for null-terminated
///
/// # Safety
/// The caller MUST ensure `input_len` matches the actual buffer size.
/// Passing an incorrect length causes undefined behavior.
pub(crate) unsafe fn get_input_string(
    input: *const c_char,
    input_len: c_int,
) -> Result<String, c_int> {
    if input_len < 0 {
        match CStr::from_ptr(input).to_str() {
            Ok(s) => Ok(s.to_string()),
            Err(e) => {
                set_error(&format!("Invalid UTF-8: {}", e));
                Err(HEDL_ERR_INVALID_UTF8)
            }
        }
    } else {
        let len = input_len as usize;

        // Sanity check: reject extremely large inputs
        if len > MAX_FFI_INPUT_LEN {
            set_error(&format!(
                "Input length {} exceeds maximum allowed {}",
                len, MAX_FFI_INPUT_LEN
            ));
            return Err(HEDL_ERR_INVALID_UTF8);
        }

        let bytes = slice::from_raw_parts(input as *const u8, len);
        match std::str::from_utf8(bytes) {
            Ok(s) => Ok(s.to_string()),
            Err(e) => {
                set_error(&format!("Invalid UTF-8: {}", e));
                Err(HEDL_ERR_INVALID_UTF8)
            }
        }
    }
}

/// Helper to allocate output string
pub(crate) unsafe fn allocate_output_string(
    s: &str,
    out_str: *mut *mut c_char,
    err_code: c_int,
) -> c_int {
    match CString::new(s) {
        Ok(cstr) => {
            *out_str = cstr.into_raw();
            HEDL_OK
        }
        Err(e) => {
            set_error(&format!("String allocation failed: {}", e));
            *out_str = ptr::null_mut();
            err_code
        }
    }
}

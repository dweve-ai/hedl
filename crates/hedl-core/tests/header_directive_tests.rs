// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Comprehensive tests for HEDL header directives.
//!
//! Tests coverage for:
//! - %MODE directive (strict/lenient)
//! - %PROMPT directive (LLM hints)
//! - %X-* extension directives (experimental features)
//! - Removed directives (%ENUM, %DICT, %CONSTRAINT) produce clear errors

use hedl_core::parse;

// ==================== Helper functions ====================

// For mode and prompt tests, we parse the full document but only check that it doesn't error.
// The mode and prompt are used internally during parsing, but not stored in Document.
// To properly test these, we need to verify they parse without errors.
fn parses_successfully(input: &str) -> bool {
    parse(input.as_bytes()).is_ok()
}

fn parses_with_error_containing(input: &str, expected_msg: &str) -> bool {
    match parse(input.as_bytes()) {
        Ok(_) => false,
        Err(e) => e.message.contains(expected_msg),
    }
}

// ==================== %MODE directive tests ====================

#[test]
fn test_mode_directive_strict() {
    let input = "%VERSION: 1.1
%MODE: strict
---";
    assert!(parses_successfully(input));
}

#[test]
fn test_mode_directive_lenient() {
    let input = "%VERSION: 1.1
%MODE: lenient
---";
    assert!(parses_successfully(input));
}

#[test]
fn test_mode_directive_case_insensitive() {
    let input = "%VERSION: 1.1
%MODE: STRICT
---";
    assert!(parses_successfully(input));
}

#[test]
fn test_mode_directive_case_insensitive_lenient() {
    let input = "%VERSION: 1.1
%MODE: Lenient
---";
    assert!(parses_successfully(input));
}

#[test]
fn test_mode_directive_invalid_value() {
    let input = "%VERSION: 1.1
%MODE: invalid
---";
    assert!(parses_with_error_containing(input, "invalid"));
}

#[test]
fn test_mode_directive_duplicate_error() {
    let input = "%VERSION: 1.1
%MODE: strict
%MODE: lenient
---";
    assert!(parses_with_error_containing(input, "already defined"));
}

#[test]
fn test_mode_directive_default_when_not_specified() {
    let input = "%VERSION: 1.1
---";
    // Default mode is strict, document should parse successfully
    assert!(parses_successfully(input));
}

#[test]
fn test_mode_directive_with_struct() {
    let input = "%VERSION: 1.1
%MODE: strict
%STRUCT: User: [id, name]
---";
    let doc = parse(input.as_bytes()).unwrap();
    // Mode affects parsing but isn't stored in Document
    assert!(doc.structs.contains_key("User"));
}

// ==================== Removed directive rejection tests ====================
// %ENUM, %DICT, and %CONSTRAINT were removed in v2.0. They must produce clear errors.

#[test]
fn test_enum_directive_rejected() {
    let input = b"%VERSION: 1.1
%ENUM: status: {a:\"active\", i:\"inactive\"}
---
";
    let result = parse(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("removed"));
    assert!(err.message.contains("ENUM"));
}

#[test]
fn test_dict_directive_rejected() {
    let input = b"%VERSION: 1.1
%DICT: abbreviations: {NY:\"New York\"}
---
";
    let result = parse(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("removed"));
    assert!(err.message.contains("DICT"));
}

#[test]
fn test_constraint_directive_rejected() {
    let input = b"%VERSION: 1.1
%CONSTRAINT: salary: range(0, 500000)
---
";
    let result = parse(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("removed"));
    assert!(err.message.contains("CONSTRAINT"));
}

#[test]
fn test_enum_directive_rejected_v20() {
    // v2.0 requires compact version syntax (%V:2.0)
    let input = b"%V:2.0
%NULL:~
%QUOTE:'
%ENUM: status: {a:\"active\"}
---
";
    let result = parse(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("removed"));
}

#[test]
fn test_removed_directives_rejected_regardless_of_version() {
    for version in &["1.0", "1.1", "1.2", "2.0"] {
        for directive in &["%ENUM", "%DICT", "%CONSTRAINT"] {
            let input = format!("%VERSION: {}\n{}: test: value\n---", version, directive);
            let result = parse(input.as_bytes());
            assert!(
                result.is_err(),
                "{} should be rejected for version {}",
                directive,
                version
            );
        }
    }
}

// ==================== %PROMPT directive tests ====================

#[test]
fn test_prompt_directive_simple() {
    let input = "%VERSION: 1.1
%PROMPT: \"Use IDs for references.\"
---";
    assert!(parses_successfully(input));
}

#[test]
fn test_prompt_directive_with_special_characters() {
    let input = "%VERSION: 1.1
%PROMPT: \"This document contains user data: names, emails, and addresses.\"
---";
    assert!(parses_successfully(input));
}

#[test]
fn test_prompt_directive_empty_string() {
    let input = "%VERSION: 1.1
%PROMPT: \"\"
---";
    assert!(parses_successfully(input));
}

#[test]
fn test_prompt_directive_duplicate_error() {
    let input = "%VERSION: 1.1
%PROMPT: \"first\"
%PROMPT: \"second\"
---";
    assert!(parses_with_error_containing(input, "already defined"));
}

#[test]
fn test_prompt_directive_not_quoted_error() {
    let input = "%VERSION: 1.1
%PROMPT: This is not quoted
---";
    assert!(parses_with_error_containing(input, "quoted"));
}

// ==================== %X-* extension directive tests ====================

#[test]
fn test_extension_directive_does_not_error() {
    let input = b"%VERSION: 1.1
%X-custom-key: some value
---
";
    let result = parse(input);
    assert!(result.is_ok());
}

#[test]
fn test_extension_directive_multiple_extensions() {
    let input = b"%VERSION: 1.1
%X-author: John Doe
%X-created: 2025-01-22
%X-department: Engineering
---
";
    let result = parse(input);
    assert!(result.is_ok());
}

#[test]
fn test_extension_directive_with_colon_in_value() {
    let input = b"%VERSION: 1.1
%X-note: This is a note: with colon
---
";
    let result = parse(input);
    assert!(result.is_ok());
}

#[test]
fn test_extension_directive_with_other_directives() {
    let input = b"%VERSION: 1.1
%MODE: strict
%X-custom: experimental
%STRUCT: User: [id, name]
---";
    let doc = parse(input).unwrap();
    // Mode affects parsing but isn't stored in Document
    assert!(doc.structs.contains_key("User"));
}

// ==================== Combined directives tests ====================

#[test]
fn test_combined_directives() {
    let input = b"%VERSION: 1.1
%MODE: strict
%STRUCT: Employee: [id, name, status, salary, email]
%PROMPT: \"Reference employees by ID.\"
%X-author: System
---";
    let doc = parse(input).unwrap();
    assert!(doc.structs.contains_key("Employee"));
}

#[test]
fn test_combined_rejects_removed_directives() {
    // Including a removed directive alongside valid ones should error
    let input = b"%VERSION: 1.1
%MODE: strict
%STRUCT: Employee: [id, name]
%ENUM: Employee.status: {a:\"active\"}
---";
    let result = parse(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("removed"));
}

// ==================== Error cases tests ====================

#[test]
fn test_unknown_directive_not_extension() {
    let input = b"%VERSION: 1.1
%UNKNOWN: foo
---
";
    let result = parse(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("unknown directive") || err.message.contains("UNKNOWN"));
}

#[test]
fn test_malformed_directive_missing_colon() {
    let input = b"%VERSION: 1.1
%MODE strict
---
";
    let result = parse(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("missing ':'") || err.message.contains("colon"));
}

#[test]
fn test_malformed_directive_no_space_after_colon() {
    let input = b"%VERSION: 1.1
%MODE:strict
---
";
    let result = parse(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("space"));
}

#[test]
fn test_directive_after_separator_error() {
    let input = b"%VERSION: 1.1
---
%MODE: strict
";
    let result = parse(input);
    // This should parse the header successfully but not see the directive after separator
    // The directive after separator is part of the body and should error
    assert!(result.is_err());
}

// Removed: enum/dict/constraint-specific error tests (directives no longer parsed)

// ==================== Integration tests with body content ====================

#[test]
fn test_directives_with_body_content() {
    let input = b"%VERSION: 1.1
%MODE: strict
%STRUCT: User: [id, name]
---
user1:
 id: u1
 name: Alice
";
    let doc = parse(input).unwrap();
    assert!(doc.root.contains_key("user1"));
}

// ==================== Edge cases ====================

#[test]
fn test_version_with_mode_directive() {
    // v1.0 can still use %MODE
    let input = b"%VERSION: 1.0
%MODE: strict
---
";
    let result = parse(input);
    // At minimum, it should not panic
    let _ = result;
}

#[test]
fn test_comment_after_directive() {
    let input = "%VERSION: 1.1
%MODE: strict # This is a comment
---";
    assert!(parses_successfully(input));
}

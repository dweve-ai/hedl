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

//! Integration tests for benchmark name validation.
//!
//! Tests path safety, security boundaries, and edge cases for:
//! - Benchmark name validation
//! - Version string validation
//! - Path traversal prevention
//! - Unicode security issues

use hedl_bench::core::name_validation::{
    validate_benchmark_name, validate_version_string, NameValidationError, ValidationResult,
    MAX_NAME_LENGTH, MIN_NAME_LENGTH,
};

// =============================================================================
// VALID NAME TESTS
// =============================================================================

#[test]
fn test_valid_simple_names() {
    assert!(validate_benchmark_name("parse").is_ok());
    assert!(validate_benchmark_name("test").is_ok());
    assert!(validate_benchmark_name("benchmark").is_ok());
}

#[test]
fn test_valid_names_with_underscores() {
    assert!(validate_benchmark_name("parse_users").is_ok());
    assert!(validate_benchmark_name("convert_to_json").is_ok());
    assert!(validate_benchmark_name("test_case_1").is_ok());
    assert!(validate_benchmark_name("a_b_c_d_e").is_ok());
}

#[test]
fn test_valid_names_with_hyphens() {
    assert!(validate_benchmark_name("parse-users").is_ok());
    assert!(validate_benchmark_name("convert-to-json").is_ok());
    assert!(validate_benchmark_name("test-case-1").is_ok());
    assert!(validate_benchmark_name("a-b-c-d-e").is_ok());
}

#[test]
fn test_valid_names_mixed_separators() {
    assert!(validate_benchmark_name("parse_users-v1").is_ok());
    assert!(validate_benchmark_name("test-case_1").is_ok());
    assert!(validate_benchmark_name("a_b-c_d-e").is_ok());
}

#[test]
fn test_valid_names_with_numbers() {
    assert!(validate_benchmark_name("benchmark123").is_ok());
    assert!(validate_benchmark_name("test1").is_ok());
    assert!(validate_benchmark_name("parse_1000_users").is_ok());
    assert!(validate_benchmark_name("v2_benchmark").is_ok());
}

#[test]
fn test_valid_names_uppercase() {
    assert!(validate_benchmark_name("UPPERCASE").is_ok());
    assert!(validate_benchmark_name("ALL_CAPS").is_ok());
    assert!(validate_benchmark_name("MixedCase").is_ok());
    assert!(validate_benchmark_name("CamelCase").is_ok());
}

#[test]
fn test_valid_single_char_names() {
    assert!(validate_benchmark_name("a").is_ok());
    assert!(validate_benchmark_name("Z").is_ok());
    assert!(validate_benchmark_name("1").is_ok());
    assert!(validate_benchmark_name("_").is_ok());
}

#[test]
fn test_valid_boundary_length_name() {
    let max_length_name = "a".repeat(MAX_NAME_LENGTH);
    assert!(validate_benchmark_name(&max_length_name).is_ok());
}

// =============================================================================
// EMPTY AND LENGTH TESTS
// =============================================================================

#[test]
fn test_empty_name() {
    let result = validate_benchmark_name("");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{err}").contains("empty"));
}

#[test]
fn test_too_long_name() {
    let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
    let result = validate_benchmark_name(&long_name);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{err}").contains("too long"));
}

#[test]
fn test_length_boundary() {
    // Exactly at max length
    let max_name = "a".repeat(MAX_NAME_LENGTH);
    assert!(validate_benchmark_name(&max_name).is_ok());

    // One over max length
    let over_name = "a".repeat(MAX_NAME_LENGTH + 1);
    assert!(validate_benchmark_name(&over_name).is_err());
}

#[test]
fn test_very_long_name() {
    let very_long = "a".repeat(10000);
    assert!(validate_benchmark_name(&very_long).is_err());
}

// =============================================================================
// PATH TRAVERSAL ATTACK TESTS
// =============================================================================

#[test]
fn test_path_traversal_double_dot() {
    assert!(validate_benchmark_name("..").is_err());
    assert!(validate_benchmark_name("...").is_err());
    assert!(validate_benchmark_name("....").is_err());
}

#[test]
fn test_path_traversal_unix_style() {
    assert!(validate_benchmark_name("../etc/passwd").is_err());
    assert!(validate_benchmark_name("../../root").is_err());
    assert!(validate_benchmark_name("test/../other").is_err());
    assert!(validate_benchmark_name("a/b/../c").is_err());
}

#[test]
fn test_path_traversal_windows_style() {
    assert!(validate_benchmark_name("..\\windows").is_err());
    assert!(validate_benchmark_name("..\\..\\system32").is_err());
    assert!(validate_benchmark_name("test\\..\\other").is_err());
}

#[test]
fn test_path_traversal_mixed_style() {
    assert!(validate_benchmark_name("../..\\mixed").is_err());
    assert!(validate_benchmark_name("..\\../mixed").is_err());
}

#[test]
fn test_path_traversal_in_middle() {
    assert!(validate_benchmark_name("prefix..suffix").is_err());
    assert!(validate_benchmark_name("test..name").is_err());
}

// =============================================================================
// PATH SEPARATOR TESTS
// =============================================================================

#[test]
fn test_forward_slash() {
    assert!(validate_benchmark_name("test/name").is_err());
    assert!(validate_benchmark_name("a/b/c").is_err());
    assert!(validate_benchmark_name("/absolute").is_err());
    assert!(validate_benchmark_name("relative/").is_err());
}

#[test]
fn test_backslash() {
    assert!(validate_benchmark_name("test\\name").is_err());
    assert!(validate_benchmark_name("a\\b\\c").is_err());
    assert!(validate_benchmark_name("\\unc\\path").is_err());
    assert!(validate_benchmark_name("relative\\").is_err());
}

#[test]
fn test_mixed_separators() {
    assert!(validate_benchmark_name("test/other\\mixed").is_err());
    assert!(validate_benchmark_name("a\\b/c").is_err());
}

#[test]
fn test_absolute_paths() {
    assert!(validate_benchmark_name("/etc/passwd").is_err());
    assert!(validate_benchmark_name("\\\\unc\\share").is_err());
    assert!(validate_benchmark_name("/").is_err());
    assert!(validate_benchmark_name("\\").is_err());
}

// =============================================================================
// RESERVED CHARACTER TESTS
// =============================================================================

#[test]
fn test_angle_brackets() {
    assert!(validate_benchmark_name("test<name").is_err());
    assert!(validate_benchmark_name("test>name").is_err());
    assert!(validate_benchmark_name("<>").is_err());
    assert!(validate_benchmark_name("test<>name").is_err());
}

#[test]
fn test_colon() {
    assert!(validate_benchmark_name("test:name").is_err());
    assert!(validate_benchmark_name("C:").is_err());
    assert!(validate_benchmark_name("drive:path").is_err());
}

#[test]
fn test_quotes() {
    assert!(validate_benchmark_name("test\"name").is_err());
    assert!(validate_benchmark_name("test'name").is_err());
    assert!(validate_benchmark_name("\"quoted\"").is_err());
    assert!(validate_benchmark_name("'single'").is_err());
}

#[test]
fn test_pipe() {
    assert!(validate_benchmark_name("test|name").is_err());
    assert!(validate_benchmark_name("|").is_err());
    assert!(validate_benchmark_name("cmd|inject").is_err());
}

#[test]
fn test_question_mark() {
    assert!(validate_benchmark_name("test?name").is_err());
    assert!(validate_benchmark_name("query?param=1").is_err());
    assert!(validate_benchmark_name("?").is_err());
}

#[test]
fn test_asterisk() {
    assert!(validate_benchmark_name("test*name").is_err());
    assert!(validate_benchmark_name("*").is_err());
    assert!(validate_benchmark_name("glob*pattern").is_err());
}

#[test]
fn test_space() {
    assert!(validate_benchmark_name("test name").is_err());
    assert!(validate_benchmark_name(" leading").is_err());
    assert!(validate_benchmark_name("trailing ").is_err());
    assert!(validate_benchmark_name("   ").is_err());
}

#[test]
fn test_dot_alone() {
    assert!(validate_benchmark_name(".").is_err());
    assert!(validate_benchmark_name("test.name").is_err());
}

#[test]
fn test_special_shell_chars() {
    assert!(validate_benchmark_name("test;name").is_err());
    assert!(validate_benchmark_name("test&name").is_err());
    assert!(validate_benchmark_name("test$name").is_err());
    assert!(validate_benchmark_name("test`name").is_err());
    assert!(validate_benchmark_name("test(name").is_err());
    assert!(validate_benchmark_name("test)name").is_err());
    assert!(validate_benchmark_name("test[name").is_err());
    assert!(validate_benchmark_name("test]name").is_err());
    assert!(validate_benchmark_name("test{name").is_err());
    assert!(validate_benchmark_name("test}name").is_err());
    assert!(validate_benchmark_name("test!name").is_err());
    assert!(validate_benchmark_name("test#name").is_err());
    assert!(validate_benchmark_name("test@name").is_err());
    assert!(validate_benchmark_name("test%name").is_err());
    assert!(validate_benchmark_name("test^name").is_err());
    assert!(validate_benchmark_name("test=name").is_err());
    assert!(validate_benchmark_name("test+name").is_err());
    assert!(validate_benchmark_name("test~name").is_err());
}

// =============================================================================
// CONTROL CHARACTER TESTS
// =============================================================================

#[test]
fn test_null_byte() {
    assert!(validate_benchmark_name("test\0name").is_err());
    assert!(validate_benchmark_name("\0").is_err());
    assert!(validate_benchmark_name("prefix\0").is_err());
}

#[test]
fn test_newline() {
    assert!(validate_benchmark_name("test\nname").is_err());
    assert!(validate_benchmark_name("\n").is_err());
    assert!(validate_benchmark_name("test\r\nname").is_err());
}

#[test]
fn test_tab() {
    assert!(validate_benchmark_name("test\tname").is_err());
    assert!(validate_benchmark_name("\t").is_err());
}

#[test]
fn test_other_control_chars() {
    // Test common control characters
    for byte in 0u8..32 {
        let s = format!("test{}name", char::from(byte));
        let result = validate_benchmark_name(&s);
        assert!(
            result.is_err(),
            "Control char 0x{byte:02X} should be rejected"
        );
    }
}

#[test]
fn test_del_control_char() {
    assert!(validate_benchmark_name("test\x7Fname").is_err());
}

// =============================================================================
// UNICODE SECURITY TESTS
// =============================================================================

#[test]
fn test_unicode_basic() {
    // Basic Unicode letters
    assert!(validate_benchmark_name("test\u{00E9}name").is_err()); // e with acute
    assert!(validate_benchmark_name("caf\u{00E9}").is_err()); // cafe with accent
    assert!(validate_benchmark_name("\u{00F1}").is_err()); // Spanish n with tilde
}

#[test]
fn test_unicode_zero_width() {
    // Zero-width characters can hide content
    assert!(validate_benchmark_name("test\u{200B}name").is_err()); // Zero-width space
    assert!(validate_benchmark_name("test\u{200C}name").is_err()); // Zero-width non-joiner
    assert!(validate_benchmark_name("test\u{200D}name").is_err()); // Zero-width joiner
    assert!(validate_benchmark_name("test\u{FEFF}name").is_err()); // BOM
}

#[test]
fn test_unicode_directional() {
    // Bidirectional text attacks
    assert!(validate_benchmark_name("test\u{202E}name").is_err()); // RTL override
    assert!(validate_benchmark_name("test\u{202D}name").is_err()); // LTR override
    assert!(validate_benchmark_name("test\u{202A}name").is_err()); // LTR embedding
    assert!(validate_benchmark_name("test\u{202B}name").is_err()); // RTL embedding
}

#[test]
fn test_unicode_homoglyphs() {
    // Characters that look like ASCII but aren't
    assert!(validate_benchmark_name("test\u{0430}").is_err()); // Cyrillic 'a'
    assert!(validate_benchmark_name("test\u{03B1}").is_err()); // Greek alpha
    assert!(validate_benchmark_name("test\u{0435}").is_err()); // Cyrillic 'e'
    assert!(validate_benchmark_name("test\u{043E}").is_err()); // Cyrillic 'o'
}

#[test]
fn test_unicode_combining() {
    // Combining characters
    assert!(validate_benchmark_name("cafe\u{0301}").is_err()); // Combining acute accent
    assert!(validate_benchmark_name("test\u{0308}").is_err()); // Combining diaeresis
}

#[test]
fn test_unicode_emoji() {
    assert!(validate_benchmark_name("test\u{1F600}").is_err()); // Grinning face
    assert!(validate_benchmark_name("test\u{2764}").is_err()); // Red heart
    assert!(validate_benchmark_name("\u{1F4A9}").is_err()); // Poop emoji
}

#[test]
fn test_unicode_full_width() {
    // Full-width ASCII variants
    assert!(validate_benchmark_name("test\u{FF10}").is_err()); // Full-width 0
    assert!(validate_benchmark_name("test\u{FF41}").is_err()); // Full-width a
}

#[test]
fn test_unicode_math() {
    // Mathematical alphanumeric symbols
    assert!(validate_benchmark_name("test\u{1D400}").is_err()); // Mathematical bold A
    assert!(validate_benchmark_name("test\u{1D7CE}").is_err()); // Mathematical bold 0
}

// =============================================================================
// WINDOWS RESERVED NAME TESTS
// =============================================================================

#[test]
fn test_windows_reserved_con() {
    assert!(validate_benchmark_name("CON").is_err());
    assert!(validate_benchmark_name("con").is_err());
    assert!(validate_benchmark_name("Con").is_err());
    assert!(validate_benchmark_name("cOn").is_err());
}

#[test]
fn test_windows_reserved_prn() {
    assert!(validate_benchmark_name("PRN").is_err());
    assert!(validate_benchmark_name("prn").is_err());
}

#[test]
fn test_windows_reserved_aux() {
    assert!(validate_benchmark_name("AUX").is_err());
    assert!(validate_benchmark_name("aux").is_err());
}

#[test]
fn test_windows_reserved_nul() {
    assert!(validate_benchmark_name("NUL").is_err());
    assert!(validate_benchmark_name("nul").is_err());
}

#[test]
fn test_windows_reserved_com_ports() {
    for i in 1..=9 {
        let name = format!("COM{i}");
        assert!(
            validate_benchmark_name(&name).is_err(),
            "{name} should be rejected"
        );
        assert!(
            validate_benchmark_name(&name.to_lowercase()).is_err(),
            "{} should be rejected",
            name.to_lowercase()
        );
    }
}

#[test]
fn test_windows_reserved_lpt_ports() {
    for i in 1..=9 {
        let name = format!("LPT{i}");
        assert!(
            validate_benchmark_name(&name).is_err(),
            "{name} should be rejected"
        );
        assert!(
            validate_benchmark_name(&name.to_lowercase()).is_err(),
            "{} should be rejected",
            name.to_lowercase()
        );
    }
}

// =============================================================================
// PROBLEMATIC PREFIX/SUFFIX TESTS
// =============================================================================

#[test]
fn test_leading_hyphen() {
    assert!(validate_benchmark_name("-test").is_err());
    assert!(validate_benchmark_name("-").is_err());
    assert!(validate_benchmark_name("--test").is_err());
}

#[test]
fn test_leading_dot() {
    assert!(validate_benchmark_name(".hidden").is_err());
    assert!(validate_benchmark_name(".git").is_err());
    assert!(validate_benchmark_name("..hidden").is_err());
}

#[test]
fn test_trailing_dot() {
    assert!(validate_benchmark_name("test.").is_err());
    assert!(validate_benchmark_name("name..").is_err());
}

// =============================================================================
// VERSION STRING TESTS
// =============================================================================

#[test]
fn test_valid_version_strings() {
    assert!(validate_version_string("1.0.0").is_ok());
    assert!(validate_version_string("v2.3.4-beta").is_ok());
    assert!(validate_version_string("current").is_ok());
    assert!(validate_version_string("latest").is_ok());
    assert!(validate_version_string("main").is_ok());
}

#[test]
fn test_valid_version_with_subdirectories() {
    assert!(validate_version_string("2024/q1").is_ok());
    assert!(validate_version_string("2024/06/release").is_ok());
    assert!(validate_version_string("releases/v1.0").is_ok());
}

#[test]
fn test_valid_semver_versions() {
    assert!(validate_version_string("1.0.0").is_ok());
    assert!(validate_version_string("1.0.0-alpha").is_ok());
    assert!(validate_version_string("1.0.0-alpha.1").is_ok());
    assert!(validate_version_string("1.0.0-beta.2").is_ok());
    assert!(validate_version_string("1.0.0-rc.1").is_ok());
    assert!(validate_version_string("1.0.0+build.123").is_err()); // + not allowed
}

#[test]
fn test_empty_version_string() {
    assert!(validate_version_string("").is_err());
}

#[test]
fn test_version_string_path_traversal() {
    assert!(validate_version_string("../../../etc/passwd").is_err());
    assert!(validate_version_string("..").is_err());
    assert!(validate_version_string("test/../other").is_err());
}

#[test]
fn test_version_string_backslash() {
    assert!(validate_version_string("test\\backslash").is_err());
    assert!(validate_version_string("2024\\q1").is_err());
}

#[test]
fn test_version_string_absolute_path() {
    assert!(validate_version_string("/absolute/path").is_err());
    assert!(validate_version_string("/").is_err());
}

#[test]
fn test_version_string_control_chars() {
    assert!(validate_version_string("1.0.0\0evil").is_err());
    assert!(validate_version_string("1.0.0\n").is_err());
    assert!(validate_version_string("1.0.0\t").is_err());
}

#[test]
fn test_version_string_unicode() {
    assert!(validate_version_string("v1.0.0\u{200B}").is_err());
    assert!(validate_version_string("v1.0.0\u{202E}").is_err());
}

#[test]
fn test_version_string_too_long() {
    let long_version = "a".repeat(MAX_NAME_LENGTH + 1);
    assert!(validate_version_string(&long_version).is_err());
}

#[test]
fn test_version_string_special_chars() {
    assert!(validate_version_string("1.0.0<script>").is_err());
    assert!(validate_version_string("1.0.0|cmd").is_err());
    assert!(validate_version_string("1.0.0;injection").is_err());
}

// =============================================================================
// ERROR MESSAGE TESTS
// =============================================================================

#[test]
fn test_error_message_empty() {
    let result = validate_benchmark_name("");
    let err = result.unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("empty"), "Error should mention empty: {msg}");
}

#[test]
fn test_error_message_too_long() {
    let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
    let result = validate_benchmark_name(&long_name);
    let err = result.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("too long") || msg.contains("length"),
        "Error should mention length: {msg}"
    );
}

#[test]
fn test_error_message_path_separator() {
    let result = validate_benchmark_name("test/name");
    let err = result.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("path separator") || msg.contains('/'),
        "Error should mention path separator: {msg}"
    );
}

#[test]
fn test_error_message_path_traversal() {
    let result = validate_benchmark_name("..");
    let err = result.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("traversal") || msg.contains(".."),
        "Error should mention traversal: {msg}"
    );
}

#[test]
fn test_error_message_windows_reserved() {
    let result = validate_benchmark_name("CON");
    let err = result.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("Windows") || msg.contains("reserved"),
        "Error should mention Windows reserved: {msg}"
    );
}

// =============================================================================
// VALIDATION RESULT TYPE TESTS
// =============================================================================

#[test]
fn test_validation_result_valid() {
    let result = ValidationResult::Valid;
    assert_eq!(result, ValidationResult::Valid);
}

#[test]
fn test_validation_result_invalid() {
    let result = ValidationResult::Invalid(NameValidationError::Empty);
    assert!(matches!(
        result,
        ValidationResult::Invalid(NameValidationError::Empty)
    ));
}

#[test]
fn test_name_validation_error_display() {
    let errors = vec![
        NameValidationError::Empty,
        NameValidationError::TooLong {
            length: 200,
            max: 128,
        },
        NameValidationError::ContainsPathSeparator {
            char: '/',
            position: 5,
        },
        NameValidationError::ContainsPathTraversal,
        NameValidationError::ContainsReservedChar {
            char: '<',
            position: 3,
        },
        NameValidationError::ContainsControlChar {
            byte: 0x00,
            position: 2,
        },
        NameValidationError::ContainsNonAscii {
            char: '\u{00E9}',
            position: 4,
        },
        NameValidationError::WindowsReservedName {
            name: "CON".to_string(),
        },
        NameValidationError::ProblematicPrefixSuffix {
            issue: "starts with hyphen".to_string(),
        },
    ];

    for error in errors {
        let msg = format!("{error}");
        assert!(!msg.is_empty(), "Error display should not be empty");
    }
}

#[test]
fn test_name_validation_error_eq() {
    assert_eq!(NameValidationError::Empty, NameValidationError::Empty);
    assert_ne!(
        NameValidationError::Empty,
        NameValidationError::ContainsPathTraversal
    );
}

#[test]
fn test_name_validation_error_clone() {
    let error = NameValidationError::TooLong {
        length: 200,
        max: 128,
    };
    let cloned = error.clone();
    assert_eq!(error, cloned);
}

// =============================================================================
// STRESS TESTS
// =============================================================================

#[test]
fn test_many_valid_names() {
    for i in 0..1000 {
        let name = format!("benchmark_{i}");
        assert!(
            validate_benchmark_name(&name).is_ok(),
            "Name {name} should be valid"
        );
    }
}

#[test]
fn test_many_invalid_names() {
    let invalid_chars = vec![
        '/', '\\', '<', '>', ':', '"', '|', '?', '*', ' ', '.', ';', '&', '$', '`', '(', ')', '[',
        ']', '{', '}', '!', '#', '@', '%', '^', '=', '+', '~', '\'',
    ];

    for c in invalid_chars {
        let name = format!("test{c}name");
        assert!(
            validate_benchmark_name(&name).is_err(),
            "Name with '{c}' should be invalid"
        );
    }
}

// =============================================================================
// CONCURRENCY TESTS
// =============================================================================

#[test]
fn test_validation_thread_safe() {
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..100 {
                    let name = format!("benchmark_{i}_{j}");
                    assert!(validate_benchmark_name(&name).is_ok());
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

// =============================================================================
// CONSTANTS TESTS
// =============================================================================

#[test]
fn test_max_name_length_constant() {
    assert_eq!(MAX_NAME_LENGTH, 128);
}

#[test]
fn test_min_name_length_constant() {
    assert_eq!(MIN_NAME_LENGTH, 1);
}

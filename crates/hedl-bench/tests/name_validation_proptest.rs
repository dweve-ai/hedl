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

//! Property-based tests for benchmark name validation.
//!
//! Tests invariants and properties of name validation using proptest.

use hedl_bench::core::name_validation::{
    validate_benchmark_name, validate_version_string, MAX_NAME_LENGTH,
};
use proptest::prelude::*;

// =============================================================================
// VALID NAME GENERATOR
// =============================================================================

/// Generate valid benchmark names
fn valid_name_strategy() -> impl Strategy<Value = String> {
    // Valid chars: [a-zA-Z0-9_-] but not starting with - and not . or ..
    "[a-zA-Z0-9_][a-zA-Z0-9_-]{0,126}".prop_filter("Avoid Windows reserved names", |s| {
        let upper = s.to_uppercase();
        !matches!(
            upper.as_str(),
            "CON"
                | "PRN"
                | "AUX"
                | "NUL"
                | "COM1"
                | "COM2"
                | "COM3"
                | "COM4"
                | "COM5"
                | "COM6"
                | "COM7"
                | "COM8"
                | "COM9"
                | "LPT1"
                | "LPT2"
                | "LPT3"
                | "LPT4"
                | "LPT5"
                | "LPT6"
                | "LPT7"
                | "LPT8"
                | "LPT9"
        )
    })
}

/// Generate invalid characters
fn invalid_char_strategy() -> impl Strategy<Value = char> {
    prop_oneof![
        Just('/'),
        Just('\\'),
        Just('<'),
        Just('>'),
        Just(':'),
        Just('"'),
        Just('|'),
        Just('?'),
        Just('*'),
        Just(' '),
        Just(';'),
        Just('&'),
        Just('$'),
        Just('`'),
        Just('('),
        Just(')'),
        Just('['),
        Just(']'),
        Just('{'),
        Just('}'),
        Just('!'),
        Just('#'),
        Just('@'),
        Just('%'),
        Just('^'),
        Just('='),
        Just('+'),
        Just('~'),
        Just('\''),
        Just('.'),
    ]
}

// =============================================================================
// PROPERTY: Valid names are accepted
// =============================================================================

proptest! {
    /// Property: All valid benchmark names should be accepted
    #[test]
    fn prop_valid_names_accepted(name in valid_name_strategy()) {
        prop_assert!(
            validate_benchmark_name(&name).is_ok(),
            "Valid name '{}' should be accepted",
            name
        );
    }

    /// Property: Length at boundary should be accepted
    #[test]
    fn prop_max_length_accepted(len in 1usize..=MAX_NAME_LENGTH) {
        // Generate a valid name of exact length (using 'a')
        let name = "a".repeat(len);
        prop_assert!(
            validate_benchmark_name(&name).is_ok(),
            "Name of length {} should be accepted",
            len
        );
    }

    /// Property: Names over max length are rejected
    #[test]
    fn prop_over_max_length_rejected(extra in 1usize..100) {
        let name = "a".repeat(MAX_NAME_LENGTH + extra);
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Name of length {} should be rejected",
            name.len()
        );
    }
}

// =============================================================================
// PROPERTY: Invalid characters are rejected
// =============================================================================

proptest! {
    /// Property: Names with invalid characters are rejected
    #[test]
    fn prop_invalid_char_rejected(
        prefix in "[a-zA-Z0-9_]{1,10}",
        invalid in invalid_char_strategy(),
        suffix in "[a-zA-Z0-9_]{0,10}"
    ) {
        let name = format!("{prefix}{invalid}{suffix}");
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Name '{}' with invalid char '{}' should be rejected",
            name,
            invalid
        );
    }

    /// Property: Names with control characters are rejected
    #[test]
    fn prop_control_char_rejected(
        prefix in "[a-zA-Z0-9_]{1,10}",
        control in 0u8..32u8,
        suffix in "[a-zA-Z0-9_]{0,10}"
    ) {
        let name = format!("{}{}{}", prefix, char::from(control), suffix);
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Name with control char 0x{:02X} should be rejected",
            control
        );
    }

    /// Property: Names with non-ASCII are rejected
    #[test]
    fn prop_non_ascii_rejected(
        prefix in "[a-zA-Z0-9_]{1,10}",
        non_ascii in 0x80u32..0x10FFFFu32,
        suffix in "[a-zA-Z0-9_]{0,10}"
    ) {
        if let Some(c) = char::from_u32(non_ascii) {
            let name = format!("{prefix}{c}{suffix}");
            prop_assert!(
                validate_benchmark_name(&name).is_err(),
                "Name '{}' with non-ASCII char U+{:04X} should be rejected",
                name,
                non_ascii
            );
        }
    }
}

// =============================================================================
// PROPERTY: Path traversal is always rejected
// =============================================================================

proptest! {
    /// Property: Path traversal .. is always rejected
    #[test]
    fn prop_path_traversal_rejected(
        prefix in "[a-zA-Z0-9_]{0,10}",
        suffix in "[a-zA-Z0-9_]{0,10}"
    ) {
        let name = format!("{prefix}..{suffix}");
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Name '{}' with path traversal should be rejected",
            name
        );
    }

    /// Property: Forward slashes are rejected
    #[test]
    fn prop_forward_slash_rejected(
        prefix in "[a-zA-Z0-9_]{1,10}",
        suffix in "[a-zA-Z0-9_]{0,10}"
    ) {
        let name = format!("{prefix}/{suffix}");
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Name '{}' with forward slash should be rejected",
            name
        );
    }

    /// Property: Backslashes are rejected
    #[test]
    fn prop_backslash_rejected(
        prefix in "[a-zA-Z0-9_]{1,10}",
        suffix in "[a-zA-Z0-9_]{0,10}"
    ) {
        let name = format!("{prefix}\\{suffix}");
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Name '{}' with backslash should be rejected",
            name
        );
    }
}

// =============================================================================
// PROPERTY: Windows reserved names are rejected
// =============================================================================

proptest! {
    /// Property: CON in any case is rejected
    #[test]
    fn prop_con_rejected(
        c1 in prop_oneof![Just('C'), Just('c')],
        c2 in prop_oneof![Just('O'), Just('o')],
        c3 in prop_oneof![Just('N'), Just('n')]
    ) {
        let name = format!("{c1}{c2}{c3}");
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Windows reserved name '{}' should be rejected",
            name
        );
    }

    /// Property: PRN in any case is rejected
    #[test]
    fn prop_prn_rejected(
        c1 in prop_oneof![Just('P'), Just('p')],
        c2 in prop_oneof![Just('R'), Just('r')],
        c3 in prop_oneof![Just('N'), Just('n')]
    ) {
        let name = format!("{c1}{c2}{c3}");
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Windows reserved name '{}' should be rejected",
            name
        );
    }

    /// Property: AUX in any case is rejected
    #[test]
    fn prop_aux_rejected(
        c1 in prop_oneof![Just('A'), Just('a')],
        c2 in prop_oneof![Just('U'), Just('u')],
        c3 in prop_oneof![Just('X'), Just('x')]
    ) {
        let name = format!("{c1}{c2}{c3}");
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Windows reserved name '{}' should be rejected",
            name
        );
    }

    /// Property: NUL in any case is rejected
    #[test]
    fn prop_nul_rejected(
        c1 in prop_oneof![Just('N'), Just('n')],
        c2 in prop_oneof![Just('U'), Just('u')],
        c3 in prop_oneof![Just('L'), Just('l')]
    ) {
        let name = format!("{c1}{c2}{c3}");
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Windows reserved name '{}' should be rejected",
            name
        );
    }

    /// Property: COM ports in any case are rejected
    #[test]
    fn prop_com_rejected(
        c1 in prop_oneof![Just('C'), Just('c')],
        c2 in prop_oneof![Just('O'), Just('o')],
        c3 in prop_oneof![Just('M'), Just('m')],
        num in 1u8..=9u8
    ) {
        let name = format!("{c1}{c2}{c3}{num}");
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Windows reserved name '{}' should be rejected",
            name
        );
    }

    /// Property: LPT ports in any case are rejected
    #[test]
    fn prop_lpt_rejected(
        c1 in prop_oneof![Just('L'), Just('l')],
        c2 in prop_oneof![Just('P'), Just('p')],
        c3 in prop_oneof![Just('T'), Just('t')],
        num in 1u8..=9u8
    ) {
        let name = format!("{c1}{c2}{c3}{num}");
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Windows reserved name '{}' should be rejected",
            name
        );
    }
}

// =============================================================================
// PROPERTY: Prefix/suffix issues are rejected
// =============================================================================

proptest! {
    /// Property: Leading hyphen is rejected
    #[test]
    fn prop_leading_hyphen_rejected(suffix in "[a-zA-Z0-9_]{0,20}") {
        let name = format!("-{suffix}");
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Name '{}' with leading hyphen should be rejected",
            name
        );
    }

    /// Property: Leading dot is rejected
    #[test]
    fn prop_leading_dot_rejected(suffix in "[a-zA-Z0-9_]{0,20}") {
        let name = format!(".{suffix}");
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Name '{}' with leading dot should be rejected",
            name
        );
    }

    /// Property: Trailing dot is rejected
    #[test]
    fn prop_trailing_dot_rejected(prefix in "[a-zA-Z0-9_]{1,20}") {
        let name = format!("{prefix}.");
        prop_assert!(
            validate_benchmark_name(&name).is_err(),
            "Name '{}' with trailing dot should be rejected",
            name
        );
    }
}

// =============================================================================
// VERSION STRING PROPERTIES
// =============================================================================

proptest! {
    /// Property: Valid semver versions are accepted
    #[test]
    fn prop_valid_semver_accepted(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100
    ) {
        let version = format!("{major}.{minor}.{patch}");
        prop_assert!(
            validate_version_string(&version).is_ok(),
            "Semver '{}' should be accepted",
            version
        );
    }

    /// Property: Valid subdirectory versions are accepted
    #[test]
    fn prop_valid_subdir_version_accepted(
        year in 2020u32..2030,
        month in 1u32..=12
    ) {
        let version = format!("{year}/{month:02}");
        prop_assert!(
            validate_version_string(&version).is_ok(),
            "Subdirectory version '{}' should be accepted",
            version
        );
    }

    /// Property: Version strings with path traversal are rejected
    #[test]
    fn prop_version_path_traversal_rejected(
        prefix in "[a-zA-Z0-9_]{0,10}",
        suffix in "[a-zA-Z0-9_]{0,10}"
    ) {
        let version = format!("{prefix}..{suffix}");
        prop_assert!(
            validate_version_string(&version).is_err(),
            "Version '{}' with path traversal should be rejected",
            version
        );
    }

    /// Property: Version strings with backslash are rejected
    #[test]
    fn prop_version_backslash_rejected(
        prefix in "[a-zA-Z0-9_]{1,10}",
        suffix in "[a-zA-Z0-9_]{0,10}"
    ) {
        let version = format!("{prefix}\\{suffix}");
        prop_assert!(
            validate_version_string(&version).is_err(),
            "Version '{}' with backslash should be rejected",
            version
        );
    }

    /// Property: Absolute path versions are rejected
    #[test]
    fn prop_version_absolute_rejected(suffix in "[a-zA-Z0-9_]{0,20}") {
        let version = format!("/{suffix}");
        prop_assert!(
            validate_version_string(&version).is_err(),
            "Absolute path version '{}' should be rejected",
            version
        );
    }
}

// =============================================================================
// DETERMINISM PROPERTIES
// =============================================================================

proptest! {
    /// Property: Validation is deterministic
    #[test]
    fn prop_validation_deterministic(name in ".*") {
        let result1 = validate_benchmark_name(&name).is_ok();
        let result2 = validate_benchmark_name(&name).is_ok();
        let result3 = validate_benchmark_name(&name).is_ok();

        prop_assert_eq!(result1, result2, "Validation should be deterministic");
        prop_assert_eq!(result2, result3, "Validation should be deterministic");
    }

    /// Property: Version validation is deterministic
    #[test]
    fn prop_version_validation_deterministic(version in ".*") {
        let result1 = validate_version_string(&version).is_ok();
        let result2 = validate_version_string(&version).is_ok();

        prop_assert_eq!(result1, result2, "Version validation should be deterministic");
    }
}

// =============================================================================
// REGRESSION TESTS
// =============================================================================

#[test]
fn test_regression_empty_string() {
    assert!(validate_benchmark_name("").is_err());
    assert!(validate_version_string("").is_err());
}

#[test]
fn test_regression_single_char() {
    assert!(validate_benchmark_name("a").is_ok());
    assert!(validate_benchmark_name("_").is_ok());
    assert!(validate_benchmark_name("1").is_ok());
    assert!(validate_benchmark_name("-").is_err()); // Leading hyphen
    assert!(validate_benchmark_name(".").is_err()); // Leading dot
}

#[test]
fn test_regression_double_dot() {
    assert!(validate_benchmark_name("..").is_err());
    assert!(validate_benchmark_name("...").is_err());
    assert!(validate_benchmark_name("test..name").is_err());
    assert!(validate_benchmark_name("..test").is_err());
    assert!(validate_benchmark_name("test..").is_err());
}

#[test]
fn test_regression_unicode_attacks() {
    // Zero-width space
    assert!(validate_benchmark_name("test\u{200B}name").is_err());
    // RTL override
    assert!(validate_benchmark_name("test\u{202E}name").is_err());
    // Cyrillic 'a' (looks like 'a')
    assert!(validate_benchmark_name("test\u{0430}name").is_err());
    // Combining accent
    assert!(validate_benchmark_name("cafe\u{0301}").is_err());
}

#[test]
fn test_regression_shell_injection() {
    assert!(validate_benchmark_name("test;id").is_err());
    assert!(validate_benchmark_name("test&&whoami").is_err());
    assert!(validate_benchmark_name("test|cat").is_err());
    assert!(validate_benchmark_name("test`whoami`").is_err());
    assert!(validate_benchmark_name("test$(id)").is_err());
}

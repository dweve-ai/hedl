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

//! Centralized error message constants for HEDL parsing.
//!
//! This module provides a single source of truth for all error messages,
//! improving consistency and making internationalization easier in the future.

use crate::error::HedlError;

// ==================== Preprocessing Errors ====================

/// File exceeds maximum size limit.
pub fn file_too_large(size: usize, limit: usize, line: usize) -> HedlError {
    HedlError::security(
        format!("file too large: {} bytes exceeds limit of {} bytes", size, limit),
        line,
    )
}

/// Line exceeds maximum length limit.
pub fn line_too_long(length: usize, limit: usize, line: usize) -> HedlError {
    HedlError::security(
        format!("line too long: {} bytes exceeds limit of {} bytes", length, limit),
        line,
    )
}

/// Invalid UTF-8 encoding detected.
pub fn invalid_utf8(line: usize) -> HedlError {
    HedlError::syntax("file is not valid UTF-8", line)
}

/// Control character not allowed in document.
pub fn control_character(char_code: u8, line: usize) -> HedlError {
    HedlError::syntax(
        format!("control character U+{:04X} not allowed", char_code),
        line,
    )
}

// ==================== Header Errors ====================

/// Missing required VERSION directive.
pub fn missing_version(line: usize) -> HedlError {
    HedlError::version("missing %VERSION directive", line)
}

/// Missing VERSION directive before separator.
pub fn missing_version_before_separator(line: usize) -> HedlError {
    HedlError::syntax("missing %VERSION directive before separator", line)
}

/// Invalid version format.
pub fn invalid_version_format(payload: &str, line: usize) -> HedlError {
    HedlError::version(
        format!("invalid version format '{}', expected major.minor", payload),
        line,
    )
}

/// Invalid major version number.
pub fn invalid_major_version(value: &str, line: usize) -> HedlError {
    HedlError::version(format!("invalid major version: {}", value), line)
}

/// Invalid minor version number.
pub fn invalid_minor_version(value: &str, line: usize) -> HedlError {
    HedlError::version(format!("invalid minor version: {}", value), line)
}

/// Version has leading zeros.
pub fn version_leading_zeros(line: usize) -> HedlError {
    HedlError::version("leading zeros not allowed in version", line)
}

/// VERSION must be first directive.
pub fn version_not_first(line: usize) -> HedlError {
    HedlError::syntax("%VERSION must be the first directive", line)
}

/// Unsupported HEDL version.
pub fn unsupported_version(major: u32, minor: u32, line: usize) -> HedlError {
    HedlError::version(
        format!("unsupported version {}.{}, only 1.0 is supported", major, minor),
        line,
    )
}

/// Missing required separator.
pub fn missing_separator(line: usize) -> HedlError {
    HedlError::syntax("missing separator '---'", line)
}

/// Separator must not have leading whitespace.
pub fn invalid_separator_whitespace(line: usize) -> HedlError {
    HedlError::syntax("separator '---' must not have leading whitespace", line)
}

/// Expected directive starting with %.
pub fn expected_directive(content: &str, line: usize) -> HedlError {
    HedlError::syntax(
        format!("expected directive starting with '%', got: {}", content),
        line,
    )
}

/// Directive missing colon.
pub fn directive_missing_colon(line: usize) -> HedlError {
    HedlError::syntax("directive missing ':'", line)
}

/// Directive colon must be followed by space.
pub fn directive_missing_space_after_colon(line: usize) -> HedlError {
    HedlError::syntax("directive ':' must be followed by space", line)
}

/// Unknown directive.
pub fn unknown_directive(directive: &str, line: usize) -> HedlError {
    HedlError::syntax(format!("unknown directive: {}", directive), line)
}

/// Duplicate directive.
pub fn duplicate_directive(directive: &str, line: usize) -> HedlError {
    HedlError::syntax(
        format!("duplicate {} directive", directive),
        line,
    )
}

/// Invalid STRUCT definition format.
pub fn invalid_struct_format(line: usize) -> HedlError {
    HedlError::schema("invalid %STRUCT format, expected %STRUCT: TypeName: [col1, col2, ...]", line)
}

/// STRUCT directive missing colon after type name.
pub fn struct_missing_colon(line: usize) -> HedlError {
    HedlError::syntax("STRUCT directive missing ':' after type name", line)
}

/// Invalid type name.
pub fn invalid_type_name(name: &str, line: usize) -> HedlError {
    HedlError::syntax(
        format!("invalid type name: {}", name),
        line,
    )
}

/// Struct redefined with different columns.
pub fn struct_redefined(type_name: &str, line: usize) -> HedlError {
    HedlError::schema(
        format!("struct '{}' redefined with different columns", type_name),
        line,
    )
}

/// Unexpected content after count in STRUCT directive.
pub fn struct_count_unexpected_content(remaining: &str, line: usize) -> HedlError {
    HedlError::syntax(
        format!("unexpected content after count: {}", remaining),
        line,
    )
}

/// Invalid count value in STRUCT directive.
pub fn struct_count_invalid(value: &str, line: usize) -> HedlError {
    HedlError::syntax(format!("invalid count value: {}", value), line)
}

/// Leading zeros not allowed in count.
pub fn struct_count_leading_zeros(line: usize) -> HedlError {
    HedlError::syntax("leading zeros not allowed in count", line)
}

/// Column list must be enclosed in brackets.
pub fn column_list_not_bracketed(line: usize) -> HedlError {
    HedlError::syntax("column list must be enclosed in []", line)
}

/// Column list cannot be empty.
pub fn column_list_empty(line: usize) -> HedlError {
    HedlError::syntax("column list cannot be empty", line)
}

/// Invalid column name in schema.
pub fn invalid_column_name(name: &str, line: usize) -> HedlError {
    HedlError::syntax(
        format!("invalid column name: {}", name),
        line,
    )
}

/// Duplicate column name in struct.
pub fn duplicate_column_name(name: &str, line: usize) -> HedlError {
    HedlError::schema(
        format!("duplicate column name: {}", name),
        line,
    )
}

/// Duplicate struct definition.
pub fn duplicate_struct(type_name: &str, line: usize) -> HedlError {
    HedlError::schema(
        format!("duplicate struct definition for type '{}'", type_name),
        line,
    )
}

/// Empty schema not allowed.
pub fn empty_schema(type_name: &str, line: usize) -> HedlError {
    HedlError::schema(
        format!("struct '{}' must have at least one column", type_name),
        line,
    )
}

/// Too many columns in schema.
pub fn too_many_columns(count: usize, limit: usize, line: usize) -> HedlError {
    HedlError::security(
        format!("too many columns: {} exceeds limit of {}", count, limit),
        line,
    )
}

/// ALIAS directive missing colon after key.
pub fn alias_missing_colon(line: usize) -> HedlError {
    HedlError::syntax("ALIAS directive missing ':' after key", line)
}

/// Alias key must start with percent sign.
pub fn alias_key_missing_percent(line: usize) -> HedlError {
    HedlError::syntax("alias key must start with '%'", line)
}

/// Invalid alias key.
pub fn invalid_alias_key(key: &str, line: usize) -> HedlError {
    HedlError::syntax(
        format!("invalid alias key: {}", key),
        line,
    )
}

/// Alias value must be a quoted string.
pub fn alias_value_not_quoted(line: usize) -> HedlError {
    HedlError::syntax("alias value must be a quoted string", line)
}

/// Alias already defined.
pub fn alias_already_defined(key: &str, line: usize) -> HedlError {
    HedlError::alias(
        format!("alias '%{}' already defined", key),
        line,
    )
}

/// Duplicate alias definition.
pub fn duplicate_alias(key: &str, line: usize) -> HedlError {
    HedlError::alias(
        format!("duplicate alias definition for '{}'", key),
        line,
    )
}

/// Too many aliases.
pub fn too_many_aliases(_count: usize, limit: usize, line: usize) -> HedlError {
    HedlError::security(
        format!("too many aliases: exceeds limit of {}", limit),
        line,
    )
}

/// Invalid NEST format.
pub fn invalid_nest_format(line: usize) -> HedlError {
    HedlError::syntax("invalid %NEST format, expected %NEST: ParentType: ChildType", line)
}

/// NEST directive must have format 'Parent > Child'.
pub fn nest_invalid_syntax(line: usize) -> HedlError {
    HedlError::syntax("NEST directive must have format 'Parent > Child'", line)
}

/// Invalid parent type name in NEST.
pub fn nest_invalid_parent_type(parent: &str, line: usize) -> HedlError {
    HedlError::syntax(
        format!("invalid parent type name: {}", parent),
        line,
    )
}

/// Invalid child type name in NEST.
pub fn nest_invalid_child_type(child: &str, line: usize) -> HedlError {
    HedlError::syntax(
        format!("invalid child type name: {}", child),
        line,
    )
}

/// NEST parent type not defined.
pub fn nest_parent_not_defined(parent: &str, line: usize) -> HedlError {
    HedlError::schema(
        format!("NEST parent type '{}' not defined", parent),
        line,
    )
}

/// NEST child type not defined.
pub fn nest_child_not_defined(child: &str, line: usize) -> HedlError {
    HedlError::schema(
        format!("NEST child type '{}' not defined", child),
        line,
    )
}

/// Multiple NEST rules for parent type.
pub fn nest_multiple_rules(parent_type: &str, line: usize) -> HedlError {
    HedlError::schema(
        format!("multiple NEST rules for parent type '{}'", parent_type),
        line,
    )
}

/// NEST references undefined type.
pub fn nest_undefined_type(type_name: &str, line: usize) -> HedlError {
    HedlError::schema(
        format!("NEST references undefined type '{}'", type_name),
        line,
    )
}

/// Duplicate NEST definition.
pub fn duplicate_nest(parent_type: &str, line: usize) -> HedlError {
    HedlError::schema(
        format!("duplicate NEST definition for parent type '{}'", parent_type),
        line,
    )
}

// ==================== Parser Errors ====================

/// Unexpected content after block string closing.
pub fn block_string_trailing_content(line: usize) -> HedlError {
    HedlError::syntax("unexpected content after closing \"\"\"", line)
}

/// Block string size overflow.
pub fn block_string_size_overflow(line: usize) -> HedlError {
    HedlError::security("block string size overflow", line)
}

/// Block string exceeds size limit.
pub fn block_string_too_large(size: usize, limit: usize, line: usize) -> HedlError {
    HedlError::security(
        format!("block string size {} exceeds limit of {}", size, limit),
        line,
    )
}

/// Invalid indentation (not multiple of base).
pub fn invalid_indent(line: usize) -> HedlError {
    HedlError::syntax("indentation is not a consistent multiple of the base indent", line)
}

/// Indent depth exceeds maximum.
pub fn indent_depth_exceeded(depth: usize, limit: usize, line: usize) -> HedlError {
    HedlError::security(
        format!("indent depth {} exceeds limit {}", depth, limit),
        line,
    )
}

/// Invalid key-value format.
pub fn invalid_key_value_format(line: usize) -> HedlError {
    HedlError::syntax("invalid key: value format", line)
}

/// Invalid key name.
pub fn invalid_key(key: &str, line: usize) -> HedlError {
    HedlError::syntax(
        format!("invalid key name: '{}'", key),
        line,
    )
}

/// Duplicate key in object.
pub fn duplicate_key(key: &str, line: usize) -> HedlError {
    HedlError::syntax(
        format!("duplicate key '{}'", key),
        line,
    )
}

/// Too many keys in object.
pub fn too_many_object_keys(count: usize, limit: usize, line: usize) -> HedlError {
    HedlError::security(
        format!("object has too many keys: {} (max: {})", count, limit),
        line,
    )
}

/// Too many total keys across all objects.
pub fn too_many_total_keys(count: usize, limit: usize, line: usize) -> HedlError {
    HedlError::security(
        format!("too many total keys: {} exceeds limit {}", count, limit),
        line,
    )
}

/// List type not found in structs.
pub fn list_type_not_found(type_name: &str, line: usize) -> HedlError {
    HedlError::schema(
        format!("type '{}' not found in STRUCT definitions", type_name),
        line,
    )
}

/// Matrix row has wrong number of cells.
pub fn row_cell_count_mismatch(expected: usize, actual: usize, line: usize) -> HedlError {
    HedlError::shape(
        format!("row has {} cells, expected {}", actual, expected),
        line,
    )
}

/// ID column must be a string.
pub fn id_must_be_string(line: usize) -> HedlError {
    HedlError::semantic("ID column must be a string", line)
}

/// ID column cannot be null.
pub fn id_cannot_be_null(line: usize) -> HedlError {
    HedlError::semantic("ID column cannot be null", line)
}

/// ID column cannot be ditto.
pub fn id_cannot_be_ditto(line: usize) -> HedlError {
    HedlError::semantic("ID column cannot be ditto (^)", line)
}

/// Too many nodes.
pub fn too_many_nodes(count: usize, limit: usize, line: usize) -> HedlError {
    HedlError::security(
        format!("too many nodes: {} exceeds limit of {}", count, limit),
        line,
    )
}

/// Node count overflow.
pub fn node_count_overflow(line: usize) -> HedlError {
    HedlError::security("node count overflow", line)
}

/// NEST hierarchy depth exceeded.
pub fn nest_depth_exceeded(depth: usize, limit: usize, line: usize) -> HedlError {
    HedlError::security(
        format!("NEST hierarchy depth {} exceeds maximum allowed depth {}", depth, limit),
        line,
    )
}

/// Orphan row (child without NEST).
pub fn orphan_row(type_name: &str, line: usize) -> HedlError {
    HedlError::orphan_row(
        format!("row of type '{}' has no parent (missing NEST rule)", type_name),
        line,
    )
}

/// Truncated object (key without value at end of file).
pub fn truncated_object(key: &str, line: usize) -> HedlError {
    HedlError::syntax(
        format!("truncated object: key '{}' has no value", key),
        line,
    )
}

// ==================== Reference Errors ====================

/// Duplicate ID within type.
pub fn duplicate_id(type_name: &str, id: &str, prev_line: usize, line: usize) -> HedlError {
    HedlError::collision(
        format!(
            "duplicate ID '{}' in type '{}' (previously defined at line {})",
            id, type_name, prev_line
        ),
        line,
    )
}

/// Unresolved reference.
pub fn unresolved_reference(reference: &str, line: usize) -> HedlError {
    HedlError::reference(
        format!("unresolved reference: {}", reference),
        line,
    )
}

/// Ambiguous unqualified reference.
pub fn ambiguous_reference(id: &str, types: &[String], line: usize) -> HedlError {
    HedlError::reference(
        format!(
            "ambiguous reference @{}: found in types [{}]",
            id,
            types.join(", ")
        ),
        line,
    )
}

// ==================== Inference Errors ====================

/// Unknown alias.
pub fn unknown_alias(key: &str, line: usize) -> HedlError {
    HedlError::alias(
        format!("unknown alias: %{}", key),
        line,
    )
}

/// Ditto not allowed in non-matrix context.
pub fn ditto_not_in_matrix(line: usize) -> HedlError {
    HedlError::semantic("ditto (^) only allowed in matrix cells", line)
}

/// Ditto not allowed in first row.
pub fn ditto_in_first_row(line: usize) -> HedlError {
    HedlError::semantic("ditto (^) not allowed in first row", line)
}

// ==================== Test Helpers ====================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::HedlErrorKind;

    #[test]
    fn test_file_too_large() {
        let err = file_too_large(2000, 1000, 0);
        assert_eq!(err.kind, HedlErrorKind::Security);
        assert!(err.message.contains("2000"));
        assert!(err.message.contains("1000"));
    }

    #[test]
    fn test_invalid_type_name() {
        let err = invalid_type_name("123invalid", 5);
        assert_eq!(err.kind, HedlErrorKind::Syntax);
        assert_eq!(err.line, 5);
    }

    #[test]
    fn test_duplicate_key() {
        let err = duplicate_key("mykey", 10);
        assert_eq!(err.kind, HedlErrorKind::Syntax);
        assert!(err.message.contains("mykey"));
    }

    #[test]
    fn test_unresolved_reference() {
        let err = unresolved_reference("@User:123", 42);
        assert_eq!(err.kind, HedlErrorKind::Reference);
        assert_eq!(err.line, 42);
    }

    #[test]
    fn test_nest_depth_exceeded() {
        let err = nest_depth_exceeded(101, 100, 50);
        assert_eq!(err.kind, HedlErrorKind::Security);
        assert!(err.message.contains("101"));
        assert!(err.message.contains("100"));
    }

    #[test]
    fn test_ambiguous_reference() {
        let types = vec!["User".to_string(), "Admin".to_string()];
        let err = ambiguous_reference("id123", &types, 25);
        assert_eq!(err.kind, HedlErrorKind::Reference);
        assert!(err.message.contains("User"));
        assert!(err.message.contains("Admin"));
    }

    #[test]
    fn test_invalid_version_format() {
        let err = invalid_version_format("1.0.0", 1);
        assert_eq!(err.kind, HedlErrorKind::Version);
        assert!(err.message.contains("1.0.0"));
    }

    #[test]
    fn test_struct_missing_colon() {
        let err = struct_missing_colon(10);
        assert_eq!(err.kind, HedlErrorKind::Syntax);
        assert!(err.message.contains("missing ':'"));
    }

    #[test]
    fn test_alias_already_defined() {
        let err = alias_already_defined("key", 5);
        assert_eq!(err.kind, HedlErrorKind::Alias);
        assert!(err.message.contains("key"));
    }

    #[test]
    fn test_nest_invalid_syntax() {
        let err = nest_invalid_syntax(15);
        assert_eq!(err.kind, HedlErrorKind::Syntax);
        assert!(err.message.contains("Parent > Child"));
    }
}

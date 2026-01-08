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

//! Directive parsing for HEDL header section.
//!
//! Handles parsing of:
//! - %STRUCT: TypeName: [col1, col2, ...]
//! - %ALIAS: %key: "expansion value"
//! - %NEST: ParentType > ChildType

use super::error::LexError;
use super::span::{SourcePos, Span};
use super::tokens::{is_valid_key_token, is_valid_type_name};
use std::collections::HashMap;

/// A parsed %STRUCT directive defining a schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDirective {
    /// The type name (PascalCase).
    pub type_name: String,
    /// Ordered list of column names (Key tokens).
    pub columns: Vec<String>,
    /// Source span for this directive.
    pub span: Span,
}

/// A parsed %ALIAS directive defining a constant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasDirective {
    /// The alias key (without the % prefix).
    pub key: String,
    /// The expansion value (unquoted string).
    pub value: String,
    /// Source span for this directive.
    pub span: Span,
}

/// A parsed %NEST directive defining parent-child relationships.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestDirective {
    /// The parent type name.
    pub parent_type: String,
    /// The child type name.
    pub child_type: String,
    /// Source span for this directive.
    pub span: Span,
}

/// Parse a %STRUCT directive.
///
/// Syntax: `%STRUCT: TypeName: [col1, col2, ...]`
///
/// # Errors
///
/// Returns error if:
/// - TypeName is not valid PascalCase
/// - Column list is malformed
/// - Column names are not valid Key tokens
/// - Duplicate column names
/// - No columns provided
pub fn parse_struct(payload: &str) -> Result<StructDirective, LexError> {
    // Split on first colon to get TypeName and column list
    let parts: Vec<&str> = payload.splitn(2, ':').map(|s| s.trim()).collect();

    if parts.len() != 2 {
        return Err(LexError::InvalidToken {
            message: "STRUCT directive must be: TypeName: [col1, col2, ...]".to_string(),
            pos: SourcePos::new(1, 1),
        });
    }

    let type_name = parts[0];
    let column_part = parts[1];

    // Validate TypeName
    if !is_valid_type_name(type_name) {
        return Err(LexError::InvalidToken {
            message: format!(
                "invalid type name '{}': must be PascalCase [A-Z][A-Za-z0-9]*",
                type_name
            ),
            pos: SourcePos::new(1, 1),
        });
    }

    // Parse column list
    let columns = parse_column_list(column_part)?;

    if columns.is_empty() {
        return Err(LexError::InvalidToken {
            message: "STRUCT directive must have at least one column".to_string(),
            pos: SourcePos::new(1, 1),
        });
    }

    // Check for duplicate columns
    let mut seen = HashMap::new();
    for col in &columns {
        if seen.insert(col, ()).is_some() {
            return Err(LexError::InvalidToken {
                message: format!("duplicate column name '{}' in STRUCT", col),
                pos: SourcePos::new(1, 1),
            });
        }
    }

    Ok(StructDirective {
        type_name: type_name.to_string(),
        columns,
        span: Span::default(), // TODO: Caller should provide actual span
    })
}

/// Parse a %ALIAS directive.
///
/// Syntax: `%ALIAS: %key: "expansion value"`
///
/// # Errors
///
/// Returns error if:
/// - Key doesn't start with %
/// - Key (after %) is not valid Key token
/// - Value is not a quoted string
pub fn parse_alias(payload: &str) -> Result<AliasDirective, LexError> {
    // Split on first colon to get key and value
    let parts: Vec<&str> = payload.splitn(2, ':').map(|s| s.trim()).collect();

    if parts.len() != 2 {
        return Err(LexError::InvalidToken {
            message: "ALIAS directive must be: %key: \"value\"".to_string(),
            pos: SourcePos::new(1, 1),
        });
    }

    let key_part = parts[0];
    let value_part = parts[1];

    // Validate key starts with %
    if !key_part.starts_with('%') {
        return Err(LexError::InvalidToken {
            message: format!("ALIAS key must start with %: got '{}'", key_part),
            pos: SourcePos::new(1, 1),
        });
    }

    let key = &key_part[1..]; // Remove % prefix

    // Validate key is valid Key token
    if !is_valid_key_token(key) {
        return Err(LexError::InvalidToken {
            message: format!("invalid ALIAS key '{}': must be [a-z_][a-z0-9_]*", key),
            pos: SourcePos::new(1, 1),
        });
    }

    // Parse quoted value
    let value = parse_quoted_string(value_part)?;

    Ok(AliasDirective {
        key: key.to_string(),
        value,
        span: Span::default(), // TODO: Caller should provide actual span
    })
}

/// Parse a %NEST directive.
///
/// Syntax: `%NEST: ParentType > ChildType`
///
/// # Errors
///
/// Returns error if:
/// - Missing > separator
/// - ParentType or ChildType are not valid TypeNames
pub fn parse_nest(payload: &str) -> Result<NestDirective, LexError> {
    // Split on >
    let parts: Vec<&str> = payload.split('>').map(|s| s.trim()).collect();

    if parts.len() != 2 {
        return Err(LexError::InvalidToken {
            message: "NEST directive must be: ParentType > ChildType".to_string(),
            pos: SourcePos::new(1, 1),
        });
    }

    let parent_type = parts[0];
    let child_type = parts[1];

    // Validate both are TypeNames
    if !is_valid_type_name(parent_type) {
        return Err(LexError::InvalidToken {
            message: format!(
                "invalid parent type name '{}': must be PascalCase",
                parent_type
            ),
            pos: SourcePos::new(1, 1),
        });
    }

    if !is_valid_type_name(child_type) {
        return Err(LexError::InvalidToken {
            message: format!("invalid child type name '{}': must be PascalCase", child_type),
            pos: SourcePos::new(1, 1),
        });
    }

    Ok(NestDirective {
        parent_type: parent_type.to_string(),
        child_type: child_type.to_string(),
        span: Span::default(), // TODO: Caller should provide actual span
    })
}

/// Parse a column list: `[col1, col2, ...]`
fn parse_column_list(s: &str) -> Result<Vec<String>, LexError> {
    let s = s.trim();

    // Must start with [ and end with ]
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err(LexError::InvalidToken {
            message: "column list must be enclosed in []".to_string(),
            pos: SourcePos::new(1, 1),
        });
    }

    let content = &s[1..s.len() - 1].trim();

    if content.is_empty() {
        return Ok(Vec::new());
    }

    // Split by comma and validate each column
    let mut columns = Vec::new();
    for part in content.split(',') {
        let col = part.trim();

        // Check for trailing comma (empty part)
        if col.is_empty() {
            return Err(LexError::InvalidToken {
                message: "trailing comma or empty column name in column list".to_string(),
                pos: SourcePos::new(1, 1),
            });
        }

        if !is_valid_key_token(col) {
            return Err(LexError::InvalidToken {
                message: format!("invalid column name '{}': must be [a-z_][a-z0-9_]*", col),
                pos: SourcePos::new(1, 1),
            });
        }

        columns.push(col.to_string());
    }

    Ok(columns)
}

/// Parse a quoted string value.
///
/// Handles "" escaping for literal quotes.
fn parse_quoted_string(s: &str) -> Result<String, LexError> {
    let s = s.trim();

    if !s.starts_with('"') || !s.ends_with('"') {
        return Err(LexError::InvalidToken {
            message: "value must be a quoted string".to_string(),
            pos: SourcePos::new(1, 1),
        });
    }

    let content = &s[1..s.len() - 1];

    // Handle "" escaping
    let mut result = String::new();
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '"' {
            if chars.peek() == Some(&'"') {
                // Escaped quote
                chars.next();
                result.push('"');
            } else {
                // Unescaped quote in middle of string is an error
                return Err(LexError::UnclosedQuote {
                    pos: SourcePos::new(1, 1),
                });
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_struct_basic() {
        let s = parse_struct("User: [id, name, email]").unwrap();
        assert_eq!(s.type_name, "User");
        assert_eq!(s.columns, vec!["id", "name", "email"]);
    }

    #[test]
    fn test_parse_struct_with_spaces() {
        let s = parse_struct("User: [ id , name , email ]").unwrap();
        assert_eq!(s.type_name, "User");
        assert_eq!(s.columns, vec!["id", "name", "email"]);
    }

    #[test]
    fn test_parse_struct_no_spaces() {
        let s = parse_struct("User: [id,name,email]").unwrap();
        assert_eq!(s.type_name, "User");
        assert_eq!(s.columns, vec!["id", "name", "email"]);
    }

    #[test]
    fn test_parse_struct_invalid_type_name() {
        let result = parse_struct("user: [id, name]");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_struct_invalid_column_name() {
        let result = parse_struct("User: [id, Name]");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_struct_duplicate_columns() {
        let result = parse_struct("User: [id, name, id]");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_struct_empty() {
        let result = parse_struct("User: []");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_struct_trailing_comma() {
        let result = parse_struct("User: [id, name,]");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_alias_basic() {
        let a = parse_alias("%active: \"true\"").unwrap();
        assert_eq!(a.key, "active");
        assert_eq!(a.value, "true");
    }

    #[test]
    fn test_parse_alias_empty_value() {
        let a = parse_alias("%empty: \"\"").unwrap();
        assert_eq!(a.key, "empty");
        assert_eq!(a.value, "");
    }

    #[test]
    fn test_parse_alias_escaped_quotes() {
        let a = parse_alias("%name: \"John \"\"Doc\"\" Doe\"").unwrap();
        assert_eq!(a.key, "name");
        assert_eq!(a.value, "John \"Doc\" Doe");
    }

    #[test]
    fn test_parse_alias_no_percent() {
        let result = parse_alias("active: \"true\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_alias_not_quoted() {
        let result = parse_alias("%active: true");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_alias_invalid_key() {
        let result = parse_alias("%Active: \"true\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_nest_basic() {
        let n = parse_nest("User > Post").unwrap();
        assert_eq!(n.parent_type, "User");
        assert_eq!(n.child_type, "Post");
    }

    #[test]
    fn test_parse_nest_with_spaces() {
        let n = parse_nest("  User  >  Post  ").unwrap();
        assert_eq!(n.parent_type, "User");
        assert_eq!(n.child_type, "Post");
    }

    #[test]
    fn test_parse_nest_invalid_parent() {
        let result = parse_nest("user > Post");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_nest_invalid_child() {
        let result = parse_nest("User > post");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_nest_no_separator() {
        let result = parse_nest("User Post");
        assert!(result.is_err());
    }

    // ==================== Additional STRUCT tests ====================

    #[test]
    fn test_parse_struct_single_column() {
        let s = parse_struct("Item: [value]").unwrap();
        assert_eq!(s.type_name, "Item");
        assert_eq!(s.columns, vec!["value"]);
    }

    #[test]
    fn test_parse_struct_many_columns() {
        let s = parse_struct("Record: [a, b, c, d, e, f, g, h, i, j]").unwrap();
        assert_eq!(s.type_name, "Record");
        assert_eq!(s.columns.len(), 10);
    }

    #[test]
    fn test_parse_struct_column_with_numbers() {
        let s = parse_struct("Data: [col1, col2, value123]").unwrap();
        assert_eq!(s.columns, vec!["col1", "col2", "value123"]);
    }

    #[test]
    fn test_parse_struct_column_with_underscore() {
        let s = parse_struct("Data: [first_name, last_name, _private]").unwrap();
        assert_eq!(s.columns, vec!["first_name", "last_name", "_private"]);
    }

    #[test]
    fn test_parse_struct_pascal_case_with_numbers() {
        let s = parse_struct("User2: [id]").unwrap();
        assert_eq!(s.type_name, "User2");
    }

    #[test]
    fn test_parse_struct_type_name_lowercase_error() {
        let result = parse_struct("user: [id]");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("PascalCase"));
    }

    #[test]
    fn test_parse_struct_type_name_with_underscore_error() {
        let result = parse_struct("User_Type: [id]");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_struct_column_uppercase_error() {
        let result = parse_struct("User: [Id, Name]");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("column name"));
    }

    #[test]
    fn test_parse_struct_column_with_hyphen_error() {
        let result = parse_struct("User: [first-name]");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_struct_missing_brackets() {
        let result = parse_struct("User: id, name");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("enclosed in []"));
    }

    #[test]
    fn test_parse_struct_missing_colon() {
        let result = parse_struct("User [id, name]");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_struct_double_comma() {
        let result = parse_struct("User: [id,, name]");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("empty column"));
    }

    #[test]
    fn test_parse_struct_equality() {
        let a = StructDirective {
            type_name: "User".to_string(),
            columns: vec!["id".to_string(), "name".to_string()],
            span: Span::default(),
        };
        let b = StructDirective {
            type_name: "User".to_string(),
            columns: vec!["id".to_string(), "name".to_string()],
            span: Span::default(),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_parse_struct_debug() {
        let s = StructDirective {
            type_name: "User".to_string(),
            columns: vec!["id".to_string()],
            span: Span::default(),
        };
        let debug = format!("{:?}", s);
        assert!(debug.contains("User"));
        assert!(debug.contains("id"));
    }

    // ==================== Additional ALIAS tests ====================

    #[test]
    fn test_parse_alias_with_spaces_in_value() {
        let a = parse_alias("%greeting: \"Hello, World!\"").unwrap();
        assert_eq!(a.key, "greeting");
        assert_eq!(a.value, "Hello, World!");
    }

    #[test]
    fn test_parse_alias_with_unicode() {
        let a = parse_alias("%emoji: \"Hello 🌍\"").unwrap();
        assert_eq!(a.value, "Hello 🌍");
    }

    #[test]
    fn test_parse_alias_with_numbers_in_key() {
        let a = parse_alias("%config123: \"value\"").unwrap();
        assert_eq!(a.key, "config123");
    }

    #[test]
    fn test_parse_alias_with_underscore_in_key() {
        let a = parse_alias("%my_config: \"value\"").unwrap();
        assert_eq!(a.key, "my_config");
    }

    #[test]
    fn test_parse_alias_key_starting_with_number_error() {
        let result = parse_alias("%123key: \"value\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_alias_key_with_hyphen_error() {
        let result = parse_alias("%my-key: \"value\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_alias_value_with_colons() {
        let a = parse_alias("%url: \"https://example.com:8080\"").unwrap();
        assert_eq!(a.value, "https://example.com:8080");
    }

    #[test]
    fn test_parse_alias_missing_key_percent() {
        let result = parse_alias("key: \"value\"");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("must start with %"));
    }

    #[test]
    fn test_parse_alias_unquoted_value_error() {
        let result = parse_alias("%key: value");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("quoted string"));
    }

    #[test]
    fn test_parse_alias_single_quote_error() {
        let result = parse_alias("%key: 'value'");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_alias_unclosed_quote_error() {
        let result = parse_alias("%key: \"unclosed");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_alias_unescaped_quote_in_value_error() {
        let result = parse_alias("%key: \"say \"hello\"\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_alias_equality() {
        let a = AliasDirective {
            key: "test".to_string(),
            value: "value".to_string(),
            span: Span::default(),
        };
        let b = AliasDirective {
            key: "test".to_string(),
            value: "value".to_string(),
            span: Span::default(),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_parse_alias_debug() {
        let a = AliasDirective {
            key: "test".to_string(),
            value: "value".to_string(),
            span: Span::default(),
        };
        let debug = format!("{:?}", a);
        assert!(debug.contains("test"));
        assert!(debug.contains("value"));
    }

    // ==================== Additional NEST tests ====================

    #[test]
    fn test_parse_nest_long_type_names() {
        let n = parse_nest("VeryLongParentTypeName > VeryLongChildTypeName").unwrap();
        assert_eq!(n.parent_type, "VeryLongParentTypeName");
        assert_eq!(n.child_type, "VeryLongChildTypeName");
    }

    #[test]
    fn test_parse_nest_type_with_numbers() {
        let n = parse_nest("User2 > Post2").unwrap();
        assert_eq!(n.parent_type, "User2");
        assert_eq!(n.child_type, "Post2");
    }

    #[test]
    fn test_parse_nest_no_spaces() {
        let n = parse_nest("User>Post").unwrap();
        assert_eq!(n.parent_type, "User");
        assert_eq!(n.child_type, "Post");
    }

    #[test]
    fn test_parse_nest_extra_spaces() {
        let n = parse_nest("   User   >   Post   ").unwrap();
        assert_eq!(n.parent_type, "User");
        assert_eq!(n.child_type, "Post");
    }

    #[test]
    fn test_parse_nest_multiple_arrows_error() {
        let result = parse_nest("A > B > C");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_nest_empty_parent_error() {
        let result = parse_nest("> Child");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_nest_empty_child_error() {
        let result = parse_nest("Parent >");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_nest_lowercase_parent_error() {
        let result = parse_nest("parent > Child");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("parent type"));
    }

    #[test]
    fn test_parse_nest_lowercase_child_error() {
        let result = parse_nest("Parent > child");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("child type"));
    }

    #[test]
    fn test_parse_nest_snake_case_error() {
        let result = parse_nest("User_Type > Post_Type");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_nest_equality() {
        let a = NestDirective {
            parent_type: "User".to_string(),
            child_type: "Post".to_string(),
            span: Span::default(),
        };
        let b = NestDirective {
            parent_type: "User".to_string(),
            child_type: "Post".to_string(),
            span: Span::default(),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_parse_nest_debug() {
        let n = NestDirective {
            parent_type: "User".to_string(),
            child_type: "Post".to_string(),
            span: Span::default(),
        };
        let debug = format!("{:?}", n);
        assert!(debug.contains("User"));
        assert!(debug.contains("Post"));
    }

    #[test]
    fn test_parse_nest_clone() {
        let original = NestDirective {
            parent_type: "User".to_string(),
            child_type: "Post".to_string(),
            span: Span::default(),
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    // ==================== Column list edge cases ====================

    #[test]
    fn test_parse_column_list_single_underscore() {
        let s = parse_struct("Data: [_]").unwrap();
        assert_eq!(s.columns, vec!["_"]);
    }

    #[test]
    fn test_parse_column_list_multiple_underscores() {
        let s = parse_struct("Data: [_, __, ___]").unwrap();
        assert_eq!(s.columns, vec!["_", "__", "___"]);
    }

    #[test]
    fn test_parse_column_list_leading_underscore() {
        let s = parse_struct("Data: [_private, __internal]").unwrap();
        assert_eq!(s.columns, vec!["_private", "__internal"]);
    }
}

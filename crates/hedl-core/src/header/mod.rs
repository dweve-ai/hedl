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

//! Header directive parsing for HEDL.

mod parse;
mod types;

// Re-export public types and functions
pub use parse::parse_header;
pub use types::{CountValue, Header, ParseMode};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limits::{Limits, TimeoutContext};

    fn make_lines(s: &str) -> Vec<(usize, &str)> {
        s.lines().enumerate().map(|(i, l)| (i + 1, l)).collect()
    }

    fn default_limits() -> Limits {
        Limits::default()
    }

    // ==================== Minimal header tests ====================

    #[test]
    fn test_parse_minimal_header() {
        let input = "%V:2.0\n%NULL:~\n%QUOTE:\"\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.version, (2, 0));
    }

    #[test]
    fn test_header_returns_body_start_index() {
        let input = "%V:2.0\n%NULL:~\n%QUOTE:\"\n---";
        let lines = make_lines(input);
        let (_, body_idx) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(body_idx, 4); // Index after separator
    }

    #[test]
    fn test_header_with_comment() {
        let input = "%V:2.0\n# This is a comment\n%NULL:~\n%QUOTE:\"\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.version, (2, 0));
    }

    #[test]
    fn test_header_with_blank_lines() {
        let input = "%V:2.0\n\n  \n%NULL:~\n%QUOTE:\"\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.version, (2, 0));
    }

    #[test]
    fn test_separator_with_comment() {
        let input = "%V:2.0\n%NULL:~\n%QUOTE:\"\n---# comment after separator";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.version, (2, 0));
    }

    #[test]
    fn test_separator_with_space_comment() {
        let input = "%V:2.0\n%NULL:~\n%QUOTE:\"\n--- # comment";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.version, (2, 0));
    }

    // ==================== %VERSION tests ====================

    #[test]
    fn test_version_zero_zero() {
        let input = "%VERSION: 0.0\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.version, (0, 0));
    }

    #[test]
    fn test_version_high_numbers() {
        // Use pre-v2.0 version to avoid v2.0 syntax restrictions
        let input = "%VERSION: 1.2\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.version, (1, 2));
    }

    #[test]
    fn test_version_leading_zero_error() {
        let input = "%VERSION: 01.0\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("leading zeros"));
    }

    #[test]
    fn test_version_minor_leading_zero_error() {
        let input = "%VERSION: 1.01\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
    }

    #[test]
    fn test_version_invalid_format_error() {
        let input = "%VERSION: 1\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("invalid version format"));
    }

    #[test]
    fn test_version_three_parts_error() {
        let input = "%VERSION: 1.0.0\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
    }

    #[test]
    fn test_version_non_numeric_error() {
        let input = "%VERSION: a.b\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid major"));
    }

    #[test]
    fn test_version_not_first_error() {
        let input = "%STRUCT: User: [id,name]\n%VERSION: 1.0\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("must be the first"));
    }

    // ==================== %STRUCT tests ====================

    #[test]
    fn test_parse_struct() {
        let input = "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:User:[id,name,email]\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(
            header.structs.get("User"),
            Some(&vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string()
            ])
        );
    }

    #[test]
    fn test_parse_struct_single_column() {
        let input = "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:Point:[x]\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.structs.get("Point"), Some(&vec!["x".to_string()]));
    }

    #[test]
    fn test_parse_multiple_structs() {
        let input = "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:User:[id,name]\n%S:Post:[id,title]\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert!(header.structs.contains_key("User"));
        assert!(header.structs.contains_key("Post"));
    }

    #[test]
    fn test_struct_identical_redefinition_ok() {
        let input = "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:User:[id,name]\n%S:User:[id,name]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_different_redefinition_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: User: [id, email]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("redefined with different"));
    }

    #[test]
    fn test_struct_invalid_type_name_error() {
        let input = "%VERSION: 1.0\n%STRUCT: user: [id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid type name"));
    }

    #[test]
    fn test_struct_invalid_column_name_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [Id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid column name"));
    }

    #[test]
    fn test_struct_duplicate_column_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id, name, id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("duplicate column"));
    }

    #[test]
    fn test_struct_empty_columns_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: []\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("cannot be empty"));
    }

    #[test]
    fn test_struct_missing_brackets_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: id, name\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("enclosed in []"));
    }

    #[test]
    fn test_struct_too_many_columns_error() {
        let limits = Limits {
            max_columns: 2,
            ..Limits::default()
        };
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name,email]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &limits, &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("too many columns"));
    }

    // ==================== %ALIAS tests ====================

    #[test]
    fn test_parse_alias() {
        let input = "%VERSION: 1.0\n%ALIAS: %active: \"true\"\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.aliases.get("active"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_alias_empty_value() {
        let input = "%VERSION: 1.0\n%ALIAS: %empty: \"\"\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.aliases.get("empty"), Some(&String::new()));
    }

    #[test]
    fn test_parse_alias_escaped_quotes() {
        let input = "%VERSION: 1.0\n%ALIAS: %quote: \"say \"\"hello\"\"\"\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(
            header.aliases.get("quote"),
            Some(&"say \"hello\"".to_string())
        );
    }

    #[test]
    fn test_parse_multiple_aliases() {
        let input = "%VERSION: 1.0\n%ALIAS: %a: \"1\"\n%ALIAS: %b: \"2\"\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.aliases.get("a"), Some(&"1".to_string()));
        assert_eq!(header.aliases.get("b"), Some(&"2".to_string()));
    }

    #[test]
    fn test_alias_duplicate_error() {
        let input = "%VERSION: 1.0\n%ALIAS: %key: \"a\"\n%ALIAS: %key: \"b\"\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("already defined"));
    }

    #[test]
    fn test_alias_missing_percent_error() {
        let input = "%VERSION: 1.0\n%ALIAS: key: \"value\"\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("must start with '%'"));
    }

    #[test]
    fn test_alias_unquoted_value_error() {
        let input = "%VERSION: 1.0\n%ALIAS: %key: value\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("quoted string"));
    }

    #[test]
    fn test_alias_too_many_error() {
        let limits = Limits {
            max_aliases: 1,
            ..Limits::default()
        };
        let input = "%VERSION: 1.0\n%ALIAS: %a: \"1\"\n%ALIAS: %b: \"2\"\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &limits, &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("too many aliases"));
    }

    // ==================== %NEST tests ====================

    #[test]
    fn test_parse_nest() {
        let input =
            "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: Post: [id,title]\n%NEST: User > Post\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.nests.get("User"), Some(&vec!["Post".to_string()]));
    }

    #[test]
    fn test_nest_undefined_parent_error() {
        let input = "%VERSION: 1.0\n%STRUCT: Post: [id,title]\n%NEST: User > Post\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not defined"));
    }

    #[test]
    fn test_nest_undefined_child_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%NEST: User > Post\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not defined"));
    }

    #[test]
    fn test_nest_multiple_children_for_parent_allowed() {
        // Multiple NEST rules for same parent with DIFFERENT children is allowed
        let input = "%VERSION: 1.0\n%STRUCT: A: [id]\n%STRUCT: B: [id]\n%STRUCT: C: [id]\n%NEST: A > B\n%NEST: A > C\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(
            result.is_ok(),
            "Multiple NEST children should be allowed: {:?}",
            result.err()
        );
        let header = result.unwrap().0;
        let children = header.nests.get("A").unwrap();
        assert_eq!(children.len(), 2);
        assert!(children.contains(&"B".to_string()));
        assert!(children.contains(&"C".to_string()));
    }

    #[test]
    fn test_nest_duplicate_pair_error() {
        // Duplicate (parent, child) pair is NOT allowed
        let input =
            "%VERSION: 1.0\n%STRUCT: A: [id]\n%STRUCT: B: [id]\n%NEST: A > B\n%NEST: A > B\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("duplicate NEST rule"));
    }

    #[test]
    fn test_nest_invalid_format_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%NEST: User\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Parent > Child"));
    }

    #[test]
    fn test_nest_invalid_parent_type_name_error() {
        let input =
            "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: Post: [id,title]\n%NEST: user > Post\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid parent type"));
    }

    // ==================== General error cases ====================

    #[test]
    fn test_missing_version_error() {
        let input = "%STRUCT: User: [id,name]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("VERSION"));
    }

    #[test]
    fn test_missing_separator_error() {
        let input = "%VERSION: 1.0\na: 1";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        // Error is "expected directive starting with '%'" since 'a: 1' is not a directive
        assert!(result.unwrap_err().message.contains("directive"));
    }

    #[test]
    fn test_indented_separator_error() {
        let input = "%VERSION: 1.0\n  ---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("leading whitespace"));
    }

    #[test]
    fn test_unknown_directive_error() {
        let input = "%VERSION: 1.0\n%UNKNOWN: foo\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("unknown directive"));
    }

    #[test]
    fn test_directive_missing_colon_error() {
        let input = "%VERSION 1.0\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("missing ':'"));
    }

    #[test]
    fn test_directive_missing_space_after_colon_error() {
        let input = "%VERSION:1.0\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("followed by space"));
    }

    #[test]
    fn test_non_directive_in_header_error() {
        let input = "%VERSION: 1.0\nsome text\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("expected directive"));
    }

    // ==================== Header struct tests ====================

    #[test]
    fn test_header_clone() {
        let input = "%VERSION: 1.0\n%ALIAS: %x: \"1\"\n%STRUCT: User: [id,name]\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        let cloned = header.clone();
        assert_eq!(cloned.version, header.version);
        assert_eq!(cloned.aliases, header.aliases);
        assert_eq!(cloned.structs, header.structs);
    }

    #[test]
    fn test_header_debug() {
        let input = "%VERSION: 1.0\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        let debug = format!("{:?}", header);
        assert!(debug.contains("version"));
        assert!(debug.contains("aliases"));
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_empty_input() {
        let lines: Vec<(usize, &str)> = vec![];
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
    }

    #[test]
    fn test_comment_with_directive() {
        let input = "%VERSION: 1.0 # version comment\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.version, (1, 0));
    }

    #[test]
    fn test_struct_with_comment() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name] # columns\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert!(header.structs.contains_key("User"));
    }

    #[test]
    fn test_all_directives_combined() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: Post: [id,title]\n%ALIAS: %active: \"true\"\n%NEST: User > Post\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.version, (1, 0));
        assert_eq!(header.structs.len(), 2);
        assert_eq!(header.aliases.len(), 1);
        assert_eq!(header.nests.len(), 1);
    }

    // ==================== %STRUCT with count tests ====================

    #[test]
    fn test_struct_with_count() {
        let input = "%VERSION: 1.0\n%STRUCT: Company (1): [id, name, founded, industry]\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(
            header.structs.get("Company"),
            Some(&vec![
                "id".to_string(),
                "name".to_string(),
                "founded".to_string(),
                "industry".to_string()
            ])
        );
    }

    #[test]
    fn test_struct_with_higher_count() {
        let input = "%VERSION: 1.0\n%STRUCT: Division (3): [id, name, head, budget]\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(
            header.structs.get("Division"),
            Some(&vec![
                "id".to_string(),
                "name".to_string(),
                "head".to_string(),
                "budget".to_string()
            ])
        );
    }

    #[test]
    fn test_struct_with_zero_count() {
        let input = "%VERSION: 1.0\n%STRUCT: Empty (0): [id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_ok());
        let (header, _) = result.unwrap();
        assert_eq!(header.structs.get("Empty"), Some(&vec!["id".to_string()]));
    }

    #[test]
    fn test_struct_without_count() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(
            header.structs.get("User"),
            Some(&vec!["id".to_string(), "name".to_string()])
        );
        assert_eq!(header.struct_counts.get("User"), None);
    }

    #[test]
    fn test_struct_mixed_with_and_without_count() {
        let input = "%VERSION: 1.0\n%STRUCT: User (5): [id, name]\n%STRUCT: Post: [id,title]\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.struct_counts.get("User"), Some(&5));
        assert_eq!(header.struct_counts.get("Post"), None);
    }

    #[test]
    fn test_struct_count_leading_zero_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User (01): [id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("leading zeros"));
    }

    #[test]
    fn test_struct_count_invalid_number_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User (abc): [id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid count"));
    }

    #[test]
    fn test_struct_count_negative_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User (-1): [id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid count"));
    }

    #[test]
    fn test_struct_count_extra_content_after_paren_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User (5) extra: [id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("unexpected content after count"));
    }

    #[test]
    fn test_struct_count_whitespace_before_paren() {
        let input = "%VERSION: 1.0\n%STRUCT: Company (10): [id, name]\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.struct_counts.get("Company"), Some(&10));
    }

    #[test]
    fn test_struct_count_whitespace_inside_paren() {
        let input = "%VERSION: 1.0\n%STRUCT: Company ( 10 ): [id, name]\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.struct_counts.get("Company"), Some(&10));
    }

    #[test]
    fn test_struct_count_large_number() {
        let input = "%VERSION: 1.0\n%STRUCT: BigList (999999): [id]\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.struct_counts.get("BigList"), Some(&999999));
    }

    // ==================== %MODE tests ====================

    #[test]
    fn test_parse_mode_strict() {
        let input = "%VERSION: 1.1\n%MODE: strict\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.mode, ParseMode::Strict);
    }

    #[test]
    fn test_parse_mode_lenient() {
        let input = "%VERSION: 1.1\n%MODE: lenient\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.mode, ParseMode::Lenient);
    }

    #[test]
    fn test_parse_mode_case_insensitive() {
        let input = "%VERSION: 1.1\n%MODE: STRICT\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.mode, ParseMode::Strict);
    }

    #[test]
    fn test_parse_mode_invalid_returns_error() {
        let input = "%VERSION: 1.1\n%MODE: invalid\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid"));
    }

    #[test]
    fn test_parse_mode_duplicate_returns_error() {
        let input = "%VERSION: 1.1\n%MODE: strict\n%MODE: lenient\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("already defined"));
    }

    #[test]
    fn test_parse_mode_default_when_not_specified() {
        let input = "%VERSION: 1.1\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.mode, ParseMode::default());
    }

    // ==================== Removed directive tests (%ENUM/%DICT/%CONSTRAINT) ====================

    #[test]
    fn test_enum_directive_rejected() {
        let input = "%VERSION: 1.1\n%ENUM: roles: {a:\"admin\"}\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("removed"));
    }

    #[test]
    fn test_dict_directive_rejected() {
        let input = "%VERSION: 1.1\n%DICT: codes: {a:\"apple\"}\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("removed"));
    }

    #[test]
    fn test_constraint_directive_rejected() {
        let input = "%VERSION: 1.1\n%CONSTRAINT: salary: range(0, 500000)\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("removed"));
    }

    // ==================== %PROMPT tests ====================

    #[test]
    fn test_parse_prompt() {
        let input = "%VERSION: 1.1\n%PROMPT: \"Use IDs for references.\"\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.prompt.as_deref(), Some("Use IDs for references."));
    }

    #[test]
    fn test_parse_prompt_duplicate_returns_error() {
        let input = "%VERSION: 1.1\n%PROMPT: \"first\"\n%PROMPT: \"second\"\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("already defined"));
    }

    // ==================== %X-* experimental tests ====================

    #[test]
    fn test_parse_experimental_directive_does_not_error() {
        let input = "%VERSION: 1.1\n%X-CUSTOM: some value\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_ok());
    }

    // ==================== Mode and prompt directive tests ====================

    #[test]
    fn test_parse_mode_and_prompt_directives() {
        let input = "%VERSION: 1.1\n%MODE: strict\n%STRUCT: Employee: [id, name, status, salary]\n%PROMPT: \"Reference employees by ID.\"\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.mode, ParseMode::Strict);
        assert!(header.structs.contains_key("Employee"));
        assert!(header.prompt.is_some());
    }

    // ==================== Compact Syntax tests ====================

    #[test]
    fn test_compact_version() {
        let input = "%V:1.2\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.version, (1, 2));
    }

    #[test]
    fn test_compact_struct() {
        let input = "%V:1.2\n%S:User:[id,name,email]\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(
            header.structs.get("User"),
            Some(&vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string()
            ])
        );
    }

    #[test]
    fn test_compact_nest() {
        let input = "%V:1.2\n%S:User:[id,name]\n%S:Post:[id,title]\n%N:User>Post\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.nests.get("User"), Some(&vec!["Post".to_string()]));
    }

    #[test]
    fn test_compact_alias() {
        let input = "%V:1.2\n%A:%admin:\"Administrator\"\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(
            header.aliases.get("admin"),
            Some(&"Administrator".to_string())
        );
    }

    #[test]
    fn test_null_directive() {
        let input = "%V:1.2\n%NULL:~\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.null_char, '~');
    }

    #[test]
    fn test_null_directive_custom_char() {
        let input = "%V:1.2\n%NULL:-\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.null_char, '-');
    }

    #[test]
    fn test_quote_directive() {
        let input = "%V:1.2\n%QUOTE:\"\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.quote_char, '"');
    }

    #[test]
    fn test_quote_directive_custom_char() {
        let input = "%V:1.2\n%QUOTE:'\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.quote_char, '\'');
    }

    #[test]
    fn test_count_total() {
        let input = "%V:1.2\n%C:Product.total=15\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(
            header.counts.get("Product.total"),
            Some(&CountValue::Total(15))
        );
    }

    #[test]
    fn test_count_distribution() {
        let input = "%V:1.2\n%C:Order.status:delivered=7,shipped=3,pending=2\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        match header.counts.get("Order.status") {
            Some(CountValue::Distribution(dist)) => {
                assert_eq!(dist.get("delivered"), Some(&7));
                assert_eq!(dist.get("shipped"), Some(&3));
                assert_eq!(dist.get("pending"), Some(&2));
            }
            _ => panic!("Expected Distribution count"),
        }
    }

    #[test]
    fn test_null_directive_duplicate_error() {
        let input = "%V:1.2\n%NULL:~\n%NULL:-\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("already defined"));
    }

    #[test]
    fn test_quote_directive_duplicate_error() {
        let input = "%V:1.2\n%QUOTE:\"\n%QUOTE:'\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("already defined"));
    }

    #[test]
    fn test_null_empty_error() {
        let input = "%V:1.2\n%NULL:\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("single character"));
    }

    #[test]
    fn test_null_multiple_chars_error() {
        let input = "%V:1.2\n%NULL:~~\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("single character"));
    }

    #[test]
    fn test_count_missing_equals_error() {
        let input = "%V:1.2\n%C:Product.total\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("missing '='"));
    }

    #[test]
    fn test_count_invalid_number_error() {
        let input = "%V:1.2\n%C:Product.total=abc\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("non-negative integer"));
    }

    #[test]
    fn test_full_header_with_all_directives() {
        let input = "%V:1.2\n%NULL:~\n%QUOTE:\"\n%S:Product:[id,sku,name,category,price]\n%S:Review:[id,rating,text]\n%N:Product>Review\n%C:Product.total=15\n%C:Product.category:electronics=9,clothing=3,sports=3\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.version, (1, 2));
        assert_eq!(header.null_char, '~');
        assert_eq!(header.quote_char, '"');
        assert!(header.structs.contains_key("Product"));
        assert!(header.structs.contains_key("Review"));
        assert_eq!(
            header.nests.get("Product"),
            Some(&vec!["Review".to_string()])
        );
        assert_eq!(
            header.counts.get("Product.total"),
            Some(&CountValue::Total(15))
        );
        assert!(header.counts.contains_key("Product.category"));
    }

    #[test]
    fn test_mixed_compact_and_verbose() {
        // Mixing compact and verbose directives is allowed
        let input =
            "%V:1.2\n%STRUCT: User: [id, name]\n%S:Post:[id,title]\n%NEST: User > Post\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert!(header.structs.contains_key("User"));
        assert!(header.structs.contains_key("Post"));
        assert_eq!(header.nests.get("User"), Some(&vec!["Post".to_string()]));
    }

    #[test]
    fn test_defaults_without_null_quote() {
        // When %NULL and %QUOTE are not specified, defaults apply
        let input = "%V:1.2\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.null_char, '~');
        assert_eq!(header.quote_char, '"');
    }

    #[test]
    fn test_header_new_defaults() {
        let header = Header::new((1, 2));
        assert_eq!(header.version, (1, 2));
        assert_eq!(header.null_char, '~');
        assert_eq!(header.quote_char, '"');
        assert!(header.counts.is_empty());
    }

    // ==================== v2.0 Compliance tests ====================

    #[test]
    fn test_v20_requires_null() {
        let input = "%V:2.0\n%QUOTE:\"\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("v2.0"));
        assert!(err.message.contains("%NULL"));
    }

    #[test]
    fn test_v20_requires_quote() {
        let input = "%V:2.0\n%NULL:~\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("v2.0"));
        assert!(err.message.contains("%QUOTE"));
    }

    #[test]
    fn test_v20_valid_with_required_directives() {
        let input = "%V:2.0\n%NULL:~\n%QUOTE:\"\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.version, (2, 0));
        assert_eq!(header.null_char, '~');
        assert_eq!(header.quote_char, '"');
    }

    #[test]
    fn test_v20_rejects_verbose_version() {
        let input = "%VERSION: 2.0\n%NULL:~\n%QUOTE:\"\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("v2.0"));
        assert!(err.message.contains("%V"));
    }

    #[test]
    fn test_v20_rejects_verbose_struct() {
        let input = "%V:2.0\n%NULL:~\n%QUOTE:\"\n%STRUCT: User: [id, name]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("v2.0"));
        assert!(err.message.contains("%S"));
    }

    #[test]
    fn test_v20_rejects_verbose_nest() {
        let input =
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:User:[id,name]\n%S:Post:[id,title]\n%NEST: User > Post\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("v2.0"));
        assert!(err.message.contains("%N"));
    }

    #[test]
    fn test_v20_rejects_enum_directive() {
        let input =
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%ENUM: status: {a:\"active\", i:\"inactive\"}\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("removed"));
        assert!(err.message.contains("%ENUM"));
    }

    #[test]
    fn test_v20_rejects_dict_directive() {
        let input = "%V:2.0\n%NULL:~\n%QUOTE:\"\n%DICT: codes: {A:\"Apple\", B:\"Banana\"}\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("removed"));
        assert!(err.message.contains("%DICT"));
    }

    #[test]
    fn test_rejects_enum_directive() {
        let input = "%VERSION: 1.2\n%ENUM: status: {a:\"active\", i:\"inactive\"}\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err(), "%ENUM was removed in v2.0");
        assert!(result.unwrap_err().message.contains("removed"));
    }

    #[test]
    fn test_rejects_dict_directive() {
        let input = "%VERSION: 1.2\n%DICT: codes: {A:\"Apple\", B:\"Banana\"}\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_err(), "%DICT was removed in v2.0");
        assert!(result.unwrap_err().message.contains("removed"));
    }

    #[test]
    fn test_v20_accepts_compact_syntax() {
        let input =
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:User:[id,name]\n%S:Post:[id,title]\n%N:User>Post\n---";
        let lines = make_lines(input);
        let (header, _) =
            parse_header(&lines, &default_limits(), &TimeoutContext::new(None)).unwrap();
        assert_eq!(header.version, (2, 0));
        assert!(header.structs.contains_key("User"));
        assert!(header.structs.contains_key("Post"));
        assert_eq!(header.nests.get("User"), Some(&vec!["Post".to_string()]));
    }

    #[test]
    fn test_allows_verbose_syntax() {
        // Pre-v2.0 versions allow verbose syntax
        let input = "%V:1.2\n%STRUCT: User: [id, name]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits(), &TimeoutContext::new(None));
        assert!(result.is_ok());
    }
}

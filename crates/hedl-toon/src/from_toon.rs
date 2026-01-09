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

//! TOON to HEDL conversion
//!
//! Parses TOON (Token-Oriented Object Notation) format back to HEDL Document.
//!
//! # TOON Format Overview
//!
//! TOON is a line-based format with indentation-based nesting:
//!
//! - **Key-value**: `key: value`
//! - **Object**: `key:` followed by indented children
//! - **Tabular array**: `key[count]{field1,field2}:` followed by comma-separated rows
//! - **Expanded array**: `key[count]:` followed by `- field: value` items
//!
//! # Security
//!
//! - Depth limit protection via [`MAX_NESTING_DEPTH`]
//! - Input validation for all parsed values

use crate::error::{Result, ToonError, MAX_NESTING_DEPTH};
use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use std::collections::BTreeMap;

/// Configuration for TOON parsing
#[derive(Debug, Clone)]
pub struct FromToonConfig {
    /// Expected indentation width (default: 0 for auto-detection)
    pub indent_width: usize,
}

impl Default for FromToonConfig {
    fn default() -> Self {
        Self { indent_width: 0 }
    }
}

/// Parse TOON string to HEDL Document
///
/// # Arguments
///
/// * `input` - TOON formatted string
///
/// # Returns
///
/// A HEDL Document, or a [`ToonError`] if parsing fails.
///
/// # Examples
///
/// ```rust
/// use hedl_toon::from_toon;
///
/// let toon = r#"name: MyApp
/// version: 1
/// users[2]{id,name}:
///   u1,Alice
///   u2,Bob
/// "#;
///
/// let doc = from_toon(toon).unwrap();
/// ```
pub fn from_toon(input: &str) -> Result<Document> {
    from_toon_with_config(input, &FromToonConfig::default())
}

/// Parse TOON string to HEDL Document with custom configuration
pub fn from_toon_with_config(input: &str, config: &FromToonConfig) -> Result<Document> {
    let mut parser = ToonParser::new(input, config);
    parser.parse()
}

/// A parsed line with metadata
#[derive(Debug, Clone)]
struct Line {
    number: usize,
    indent: usize,
    content: String,
}

/// Internal parser state
struct ToonParser {
    lines: Vec<Line>,
    pos: usize,
    indent_width: usize,
    auto_detect_indent: bool,
}

impl ToonParser {
    fn new(input: &str, config: &FromToonConfig) -> Self {
        let lines: Vec<Line> = input
            .lines()
            .enumerate()
            .filter_map(|(i, line)| {
                let trimmed = line.trim_start();
                if trimmed.is_empty() {
                    None
                } else {
                    let indent = line.len() - trimmed.len();
                    Some(Line {
                        number: i + 1,
                        indent,
                        content: trimmed.to_string(),
                    })
                }
            })
            .collect();

        Self {
            lines,
            pos: 0,
            indent_width: config.indent_width,
            auto_detect_indent: config.indent_width == 0,
        }
    }

    fn parse(&mut self) -> Result<Document> {
        let mut doc = Document::new((1, 0));

        while self.pos < self.lines.len() {
            let indent = self.lines[self.pos].indent;
            if indent > 0 {
                let line_num = self.lines[self.pos].number;
                return Err(ToonError::IndentationError {
                    line: line_num,
                    message: "Unexpected indentation at root level".to_string(),
                });
            }

            let (key, item) = self.parse_item(0, 0)?;
            doc.root.insert(key, item);
        }

        Ok(doc)
    }

    fn parse_item(&mut self, _base_indent: usize, depth: usize) -> Result<(String, Item)> {
        if depth > MAX_NESTING_DEPTH {
            return Err(ToonError::MaxDepthExceeded {
                depth,
                max: MAX_NESTING_DEPTH,
            });
        }

        let line_num = self.lines[self.pos].number;
        let line_indent = self.lines[self.pos].indent;
        let content = self.lines[self.pos].content.clone();

        // Try to parse as array header
        if let Some((key, count, schema, delimiter)) = self.try_parse_array_header(&content) {
            self.pos += 1;

            if count == 0 {
                let list = MatrixList::new("Item", schema);
                return Ok((key, Item::List(list)));
            }

            // Use the header's indent as the base for child rows
            if schema.is_empty() {
                let list = self.parse_expanded_array(line_indent, count, depth)?;
                return Ok((key, Item::List(list)));
            } else {
                let list = self.parse_tabular_array(line_indent, count, schema, delimiter)?;
                return Ok((key, Item::List(list)));
            }
        }

        // Parse as key: value or key: (object)
        let (key, value_part) = self.parse_key_value_str(&content, line_num)?;
        self.pos += 1;

        if value_part.is_empty() {
            // Use the current line's indent as base for children
            let children = self.parse_object_children(line_indent, depth)?;
            Ok((key, Item::Object(children)))
        } else {
            let value = self.parse_value(&value_part, line_num)?;
            Ok((key, Item::Scalar(value)))
        }
    }

    fn try_parse_array_header(&self, content: &str) -> Option<(String, usize, Vec<String>, char)> {
        let bracket_start = content.find('[')?;
        let key = self.unquote_key(content[..bracket_start].trim());

        let rest = &content[bracket_start + 1..];

        // Find count and delimiter
        let (count_str, rest, delimiter) = if let Some(tab_pos) = rest.find('\t') {
            let count = rest[..tab_pos].trim();
            (count, &rest[tab_pos + 1..], '\t')
        } else if let Some(pipe_pos) = rest.find('|') {
            let bracket_pos = rest.find(']')?;
            if pipe_pos < bracket_pos && rest.get(pipe_pos + 1..pipe_pos + 2) == Some("]") {
                let count = rest[..pipe_pos].trim();
                (count, &rest[pipe_pos + 1..], '|')
            } else {
                let count = rest[..bracket_pos].trim();
                (count, &rest[bracket_pos..], ',')
            }
        } else {
            let bracket_pos = rest.find(']')?;
            let count = rest[..bracket_pos].trim();
            (count, &rest[bracket_pos..], ',')
        };

        let count: usize = count_str.parse().ok()?;
        let close_bracket = rest.find(']')?;
        let after_bracket = &rest[close_bracket + 1..];

        // Check for schema
        let (schema, final_rest) = if after_bracket.starts_with('{') {
            let schema_end = after_bracket.find('}')?;
            let schema_str = &after_bracket[1..schema_end];
            let schema: Vec<String> = schema_str
                .split(delimiter)
                .map(|s| self.unquote_key(s.trim()))
                .collect();
            (schema, &after_bracket[schema_end + 1..])
        } else {
            (vec![], after_bracket)
        };

        if !final_rest.trim().ends_with(':') {
            return None;
        }

        Some((key, count, schema, delimiter))
    }

    fn parse_tabular_array(
        &mut self,
        base_indent: usize,
        count: usize,
        schema: Vec<String>,
        delimiter: char,
    ) -> Result<MatrixList> {
        let mut list = MatrixList::with_count_hint("Item", schema.clone(), count);

        while self.pos < self.lines.len() {
            let line_indent = self.lines[self.pos].indent;

            if line_indent <= base_indent {
                break;
            }

            if self.auto_detect_indent && self.indent_width == 0 && line_indent > base_indent {
                self.indent_width = line_indent - base_indent;
            }

            let content = self.lines[self.pos].content.clone();
            let line_num = self.lines[self.pos].number;
            let values = self.parse_delimited_row(&content, delimiter, line_num)?;

            if values.len() != schema.len() {
                return Err(ToonError::SchemaMismatch {
                    type_name: "Item".to_string(),
                    expected: schema.len(),
                    actual: values.len(),
                });
            }

            let node = Node::new("Item", "", values);
            list.add_row(node);
            self.pos += 1;
        }

        Ok(list)
    }

    fn parse_expanded_array(
        &mut self,
        base_indent: usize,
        count: usize,
        depth: usize,
    ) -> Result<MatrixList> {
        let mut list = MatrixList::with_count_hint("Item", vec![], count);
        let mut schema_detected = false;

        while self.pos < self.lines.len() {
            let line_indent = self.lines[self.pos].indent;

            if line_indent <= base_indent {
                break;
            }

            if self.auto_detect_indent && self.indent_width == 0 && line_indent > base_indent {
                self.indent_width = line_indent - base_indent;
            }

            let content = &self.lines[self.pos].content;
            if !content.starts_with("- ") {
                let line_num = self.lines[self.pos].number;
                return Err(ToonError::ParseError {
                    line: line_num,
                    message: "Expected list item marker '- '".to_string(),
                });
            }

            let (node, detected_schema) = self.parse_expanded_item(line_indent, depth + 1)?;

            if !schema_detected && !detected_schema.is_empty() {
                list.schema = detected_schema;
                schema_detected = true;
            }

            list.add_row(node);
        }

        Ok(list)
    }

    fn parse_expanded_item(&mut self, item_indent: usize, depth: usize) -> Result<(Node, Vec<String>)> {
        if depth > MAX_NESTING_DEPTH {
            return Err(ToonError::MaxDepthExceeded {
                depth,
                max: MAX_NESTING_DEPTH,
            });
        }

        let mut fields = Vec::new();
        let mut schema = Vec::new();
        let children: BTreeMap<String, Vec<Node>> = BTreeMap::new();
        let mut first_item = true;

        while self.pos < self.lines.len() {
            let line_indent = self.lines[self.pos].indent;
            let content = self.lines[self.pos].content.clone();

            if first_item {
                if line_indent != item_indent || !content.starts_with("- ") {
                    break;
                }

                let after_dash = &content[2..];
                let line_num = self.lines[self.pos].number;
                let (key, value_str) = self.parse_key_value_str(after_dash, line_num)?;

                schema.push(key);
                if value_str.is_empty() {
                    fields.push(Value::String(String::new()));
                } else {
                    fields.push(self.parse_value(&value_str, line_num)?);
                }
                self.pos += 1;
                first_item = false;
            } else {
                let expected_indent = if self.indent_width > 0 {
                    item_indent + self.indent_width
                } else {
                    item_indent + 2
                };

                if line_indent < expected_indent {
                    break;
                }

                if line_indent != expected_indent {
                    if line_indent > expected_indent {
                        self.pos += 1;
                        continue;
                    }
                    break;
                }

                let line_num = self.lines[self.pos].number;
                let (key, value_str) = self.parse_key_value_str(&content, line_num)?;
                schema.push(key);
                fields.push(self.parse_value(&value_str, line_num)?);
                self.pos += 1;
            }
        }

        let mut node = Node::new("Item", "", fields);
        node.children = children;
        Ok((node, schema))
    }

    fn parse_object_children(&mut self, base_indent: usize, depth: usize) -> Result<BTreeMap<String, Item>> {
        let mut children = BTreeMap::new();

        while self.pos < self.lines.len() {
            let line_indent = self.lines[self.pos].indent;

            if line_indent <= base_indent {
                break;
            }

            if self.auto_detect_indent && self.indent_width == 0 && line_indent > base_indent {
                self.indent_width = line_indent - base_indent;
            }

            let (key, item) = self.parse_item(line_indent, depth + 1)?;
            children.insert(key, item);
        }

        Ok(children)
    }

    fn parse_key_value_str(&self, content: &str, line_num: usize) -> Result<(String, String)> {
        if content.starts_with('"') {
            let (key, rest) = self.parse_quoted_string(content, line_num)?;
            let rest = rest.trim_start();
            if !rest.starts_with(':') {
                return Err(ToonError::ParseError {
                    line: line_num,
                    message: "Expected ':' after key".to_string(),
                });
            }
            let value = rest[1..].trim().to_string();
            return Ok((key, value));
        }

        let colon_pos = content.find(':').ok_or_else(|| ToonError::ParseError {
            line: line_num,
            message: "Expected ':' in key-value pair".to_string(),
        })?;

        let key = self.unquote_key(content[..colon_pos].trim());
        let value = content[colon_pos + 1..].trim().to_string();

        Ok((key, value))
    }

    fn parse_delimited_row(&self, content: &str, delimiter: char, line_num: usize) -> Result<Vec<Value>> {
        let mut values = Vec::new();
        let mut pos = 0;
        let chars: Vec<char> = content.chars().collect();

        while pos < chars.len() {
            while pos < chars.len() && chars[pos].is_whitespace() {
                pos += 1;
            }

            if pos >= chars.len() {
                break;
            }

            let (value, end_pos) = self.parse_row_value(&chars, pos, delimiter, line_num)?;
            values.push(value);
            pos = end_pos;

            while pos < chars.len() && (chars[pos] == delimiter || chars[pos].is_whitespace()) {
                pos += 1;
            }
        }

        Ok(values)
    }

    fn parse_row_value(&self, chars: &[char], start: usize, delimiter: char, line_num: usize) -> Result<(Value, usize)> {
        if start >= chars.len() {
            return Ok((Value::Null, start));
        }

        if chars[start] == '"' {
            let mut pos = start + 1;
            let mut s = String::new();
            while pos < chars.len() {
                if chars[pos] == '\\' && pos + 1 < chars.len() {
                    pos += 1;
                    match chars[pos] {
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        c => s.push(c),
                    }
                    pos += 1;
                } else if chars[pos] == '"' {
                    pos += 1;
                    break;
                } else {
                    s.push(chars[pos]);
                    pos += 1;
                }
            }

            if s.starts_with('@') {
                return Ok((Value::Reference(parse_reference(&s)), pos));
            }

            return Ok((Value::String(s), pos));
        }

        let mut end = start;
        while end < chars.len() && chars[end] != delimiter {
            end += 1;
        }

        let value_str: String = chars[start..end].iter().collect();
        let value_str = value_str.trim();

        Ok((self.parse_value(value_str, line_num)?, end))
    }

    fn parse_value(&self, s: &str, _line_num: usize) -> Result<Value> {
        let s = s.trim();

        if s.is_empty() {
            return Ok(Value::String(String::new()));
        }

        if s == "null" {
            return Ok(Value::Null);
        }

        if s == "true" {
            return Ok(Value::Bool(true));
        }

        if s == "false" {
            return Ok(Value::Bool(false));
        }

        if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            let inner = &s[1..s.len() - 1];
            let unescaped = self.unescape_string(inner);
            if unescaped.starts_with('@') {
                return Ok(Value::Reference(parse_reference(&unescaped)));
            }
            return Ok(Value::String(unescaped));
        }

        if s.starts_with('@') {
            return Ok(Value::Reference(parse_reference(s)));
        }

        if let Ok(i) = s.parse::<i64>() {
            return Ok(Value::Int(i));
        }

        if let Ok(f) = s.parse::<f64>() {
            return Ok(Value::Float(f));
        }

        Ok(Value::String(s.to_string()))
    }

    fn parse_quoted_string(&self, content: &str, line_num: usize) -> Result<(String, String)> {
        if !content.starts_with('"') {
            return Err(ToonError::ParseError {
                line: line_num,
                message: "Expected quoted string".to_string(),
            });
        }

        let chars: Vec<char> = content.chars().collect();
        let mut pos = 1;
        let mut s = String::new();

        while pos < chars.len() {
            if chars[pos] == '\\' && pos + 1 < chars.len() {
                pos += 1;
                match chars[pos] {
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    '"' => s.push('"'),
                    '\\' => s.push('\\'),
                    c => s.push(c),
                }
                pos += 1;
            } else if chars[pos] == '"' {
                pos += 1;
                break;
            } else {
                s.push(chars[pos]);
                pos += 1;
            }
        }

        let rest: String = chars[pos..].iter().collect();
        Ok((s, rest))
    }

    fn unescape_string(&self, s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some(c) => result.push(c),
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    fn unquote_key(&self, key: &str) -> String {
        if key.starts_with('"') && key.ends_with('"') && key.len() >= 2 {
            self.unescape_string(&key[1..key.len() - 1])
        } else {
            key.to_string()
        }
    }
}

fn parse_reference(s: &str) -> Reference {
    let s = s.trim_start_matches('@');
    if let Some(colon_pos) = s.find(':') {
        Reference::qualified(&s[..colon_pos], &s[colon_pos + 1..])
    } else {
        Reference::local(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_key_value() {
        let toon = "name: test\ncount: 42";
        let doc = from_toon(toon).unwrap();

        assert!(doc.root.contains_key("name"));
        assert!(doc.root.contains_key("count"));

        if let Item::Scalar(Value::String(s)) = &doc.root["name"] {
            assert_eq!(s, "test");
        } else {
            panic!("Expected string");
        }

        if let Item::Scalar(Value::Int(n)) = &doc.root["count"] {
            assert_eq!(*n, 42);
        } else {
            panic!("Expected int");
        }
    }

    #[test]
    fn test_nested_object() {
        let toon = r#"config:
  name: MyApp
  version: 1
  settings:
    debug: true
    timeout: 30"#;

        let doc = from_toon(toon).unwrap();

        if let Item::Object(config) = &doc.root["config"] {
            if let Item::Scalar(Value::String(s)) = &config["name"] {
                assert_eq!(s, "MyApp");
            }
            if let Item::Object(settings) = &config["settings"] {
                if let Item::Scalar(Value::Bool(b)) = &settings["debug"] {
                    assert!(*b);
                }
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_tabular_array() {
        let toon = r#"users[2]{id,name,age}:
  u1,Alice,30
  u2,Bob,25"#;

        let doc = from_toon(toon).unwrap();

        if let Item::List(list) = &doc.root["users"] {
            assert_eq!(list.rows.len(), 2);
            assert_eq!(list.schema, vec!["id", "name", "age"]);

            assert_eq!(list.rows[0].fields.len(), 3);
            if let Value::String(s) = &list.rows[0].fields[0] {
                assert_eq!(s, "u1");
            }
            if let Value::String(s) = &list.rows[0].fields[1] {
                assert_eq!(s, "Alice");
            }
            if let Value::Int(n) = &list.rows[0].fields[2] {
                assert_eq!(*n, 30);
            }
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_empty_array() {
        let toon = "items[0]:";
        let doc = from_toon(toon).unwrap();

        if let Item::List(list) = &doc.root["items"] {
            assert_eq!(list.rows.len(), 0);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_quoted_strings() {
        let toon = r#"message: "Hello, world"
path: "C:\\Users\\test""#;

        let doc = from_toon(toon).unwrap();

        if let Item::Scalar(Value::String(s)) = &doc.root["message"] {
            assert_eq!(s, "Hello, world");
        }

        if let Item::Scalar(Value::String(s)) = &doc.root["path"] {
            assert_eq!(s, "C:\\Users\\test");
        }
    }

    #[test]
    fn test_reference() {
        let toon = r#"user: "@User:u1"
local: "@item1""#;

        let doc = from_toon(toon).unwrap();

        if let Item::Scalar(Value::Reference(r)) = &doc.root["user"] {
            assert_eq!(r.type_name.as_deref(), Some("User"));
            assert_eq!(r.id, "u1");
        } else {
            panic!("Expected reference");
        }

        if let Item::Scalar(Value::Reference(r)) = &doc.root["local"] {
            assert!(r.type_name.is_none());
            assert_eq!(r.id, "item1");
        }
    }

    #[test]
    fn test_null_and_booleans() {
        let toon = "empty: null\nactive: true\ndisabled: false";
        let doc = from_toon(toon).unwrap();

        assert!(matches!(&doc.root["empty"], Item::Scalar(Value::Null)));
        assert!(matches!(&doc.root["active"], Item::Scalar(Value::Bool(true))));
        assert!(matches!(&doc.root["disabled"], Item::Scalar(Value::Bool(false))));
    }

    #[test]
    fn test_floats() {
        let toon = "pi: 3.14159\nnegative: -2.5";
        let doc = from_toon(toon).unwrap();

        if let Item::Scalar(Value::Float(f)) = &doc.root["pi"] {
            assert!((*f - 3.14159).abs() < 0.00001);
        }

        if let Item::Scalar(Value::Float(f)) = &doc.root["negative"] {
            assert!((*f - -2.5).abs() < 0.00001);
        }
    }

    #[test]
    fn test_roundtrip_simple() {
        use crate::hedl_to_toon;

        let hedl = r#"%VERSION: 1.0
---
name: Test
count: 42
"#;
        let original_doc = hedl_core::parse(hedl.as_bytes()).unwrap();
        let toon = hedl_to_toon(&original_doc).unwrap();
        let roundtrip_doc = from_toon(&toon).unwrap();

        if let (Item::Scalar(Value::String(s1)), Item::Scalar(Value::String(s2))) =
            (&original_doc.root["name"], &roundtrip_doc.root["name"])
        {
            assert_eq!(s1, s2);
        }

        if let (Item::Scalar(Value::Int(n1)), Item::Scalar(Value::Int(n2))) =
            (&original_doc.root["count"], &roundtrip_doc.root["count"])
        {
            assert_eq!(n1, n2);
        }
    }
}

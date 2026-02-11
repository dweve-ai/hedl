// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Custom TOON encoder with correct indentation handling.
//!
//! This module implements a TOON encoder that correctly handles nested tabular arrays
//! in list items, fixing the indentation bug present in toon-format 0.4.x.

use crate::error::{Result, ToonError, MAX_NESTING_DEPTH};

/// TOON writer with correct indentation handling.
pub struct ToonWriter {
    buffer: String,
    indent_size: usize,
    delimiter: char,
}

impl ToonWriter {
    /// Create a new TOON writer.
    pub fn new(indent_size: usize, delimiter: char) -> Self {
        Self {
            buffer: String::new(),
            indent_size,
            delimiter,
        }
    }

    /// Consume the writer and return the output string.
    pub fn finish(self) -> String {
        self.buffer
    }

    /// Write a string.
    pub fn write_str(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    /// Write a character.
    pub fn write_char(&mut self, ch: char) {
        self.buffer.push(ch);
    }

    /// Write a newline.
    pub fn write_newline(&mut self) {
        self.buffer.push('\n');
    }

    /// Write indentation at the given depth.
    pub fn write_indent(&mut self, depth: usize) {
        if depth > 0 && self.indent_size > 0 {
            for _ in 0..(depth * self.indent_size) {
                self.buffer.push(' ');
            }
        }
    }

    /// Write the delimiter character.
    pub fn write_delimiter(&mut self) {
        self.buffer.push(self.delimiter);
    }

    /// Check if a string needs quoting in TOON.
    pub fn needs_quoting(&self, s: &str) -> bool {
        if s.is_empty() {
            return true;
        }

        // Reserved words
        if matches!(s, "true" | "false" | "null") {
            return true;
        }

        // Looks like a TOON number literal
        if looks_like_toon_number(s) {
            return true;
        }

        // Contains structural characters that always need quoting
        let always_quote_chars = [
            ':', '\t', '\n', '\r', '"', '\'', '\\', '[', ']', '{', '}', '-', '#',
        ];
        if s.chars().any(|c| always_quote_chars.contains(&c)) {
            return true;
        }

        // Contains the current delimiter
        if s.chars().any(|c| c == self.delimiter) {
            return true;
        }

        // Starts or ends with whitespace (including unicode whitespace)
        if let Some(first) = s.chars().next() {
            if first.is_whitespace() {
                return true;
            }
        }
        if let Some(last) = s.chars().last() {
            if last.is_whitespace() {
                return true;
            }
        }

        // Contains non-ASCII whitespace anywhere
        if s.chars().any(|c| c.is_whitespace() && !c.is_ascii()) {
            return true;
        }

        false
    }

    /// Write a key (with quoting if needed).
    pub fn write_key(&mut self, key: &str) {
        if self.needs_key_quoting(key) {
            self.write_quoted_string(key);
        } else {
            self.write_str(key);
        }
    }

    fn needs_key_quoting(&self, key: &str) -> bool {
        if key.is_empty() {
            return true;
        }

        // Keys that look like actual numeric literals need quoting
        // TOON numbers: digits with optional sign, decimal point, exponent
        // Does NOT include special strings like "nan", "inf", "infinity"
        if looks_like_toon_number(key) {
            return true;
        }

        // Keys with special characters need quoting
        let special_chars = [
            ':', ',', '|', '\t', '\n', '\r', '"', '\'', '[', ']', '{', '}', ' ',
        ];
        key.chars()
            .any(|c| special_chars.contains(&c) || c == self.delimiter)
    }

    /// Write a quoted string.
    pub fn write_quoted_string(&mut self, s: &str) {
        self.write_char('"');
        for ch in s.chars() {
            match ch {
                '"' => self.write_str("\\\""),
                '\\' => self.write_str("\\\\"),
                '\n' => self.write_str("\\n"),
                '\r' => self.write_str("\\r"),
                '\t' => self.write_str("\\t"),
                _ => self.write_char(ch),
            }
        }
        self.write_char('"');
    }

    /// Write a value (with quoting if needed).
    pub fn write_value(&mut self, s: &str) {
        if self.needs_quoting(s) {
            self.write_quoted_string(s);
        } else {
            self.write_str(s);
        }
    }

    /// Write an array header with optional field list.
    pub fn write_array_header(&mut self, key: Option<&str>, len: usize, fields: Option<&[String]>) {
        if let Some(k) = key {
            self.write_key(k);
        }
        self.write_char('[');
        self.write_str(&len.to_string());
        // Only write delimiter in header if it's not comma (comma is default/implied)
        if self.delimiter != ',' {
            self.write_delimiter();
        }
        self.write_char(']');
        if let Some(field_list) = fields {
            self.write_char('{');
            for (i, field) in field_list.iter().enumerate() {
                if i > 0 {
                    self.write_delimiter();
                }
                self.write_key(field);
            }
            self.write_char('}');
        }
        self.write_char(':');
    }
}

use serde_json::Value;

/// Encode a JSON value to TOON with correct indentation.
pub fn encode_to_toon(value: &Value, indent_size: usize, delimiter: char) -> Result<String> {
    let mut writer = ToonWriter::new(indent_size, delimiter);
    encode_value(&mut writer, value, 0)?;
    Ok(writer.finish())
}

fn encode_value(writer: &mut ToonWriter, value: &Value, depth: usize) -> Result<()> {
    if depth > MAX_NESTING_DEPTH {
        return Err(ToonError::MaxDepthExceeded {
            depth,
            max: MAX_NESTING_DEPTH,
        });
    }

    match value {
        Value::Object(obj) => encode_object(writer, obj, depth),
        Value::Array(arr) => encode_array(writer, None, arr, depth),
        _ => {
            write_primitive(writer, value);
            Ok(())
        }
    }
}

fn encode_object(
    writer: &mut ToonWriter,
    obj: &serde_json::Map<String, Value>,
    depth: usize,
) -> Result<()> {
    let keys: Vec<&String> = obj.keys().collect();

    for (i, key) in keys.iter().enumerate() {
        if i > 0 {
            writer.write_newline();
        }

        let value = &obj[*key];

        match value {
            Value::Array(arr) => {
                encode_array(writer, Some(key), arr, depth)?;
            }
            Value::Object(nested_obj) => {
                if depth > 0 {
                    writer.write_indent(depth);
                }
                writer.write_key(key);
                writer.write_char(':');
                if !nested_obj.is_empty() {
                    writer.write_newline();
                    encode_object(writer, nested_obj, depth + 1)?;
                }
            }
            _ => {
                if depth > 0 {
                    writer.write_indent(depth);
                }
                writer.write_key(key);
                writer.write_str(": ");
                write_primitive(writer, value);
            }
        }
    }

    Ok(())
}

fn encode_array(
    writer: &mut ToonWriter,
    key: Option<&str>,
    arr: &[Value],
    depth: usize,
) -> Result<()> {
    if arr.is_empty() {
        if depth > 0 {
            writer.write_indent(depth);
        }
        writer.write_array_header(key, 0, None);
        return Ok(());
    }

    // Determine array format
    if let Some(fields) = is_tabular_array(arr) {
        encode_tabular_array(writer, key, arr, &fields, depth)
    } else if is_primitive_array(arr) {
        encode_primitive_array(writer, key, arr, depth)
    } else {
        encode_nested_array(writer, key, arr, depth)
    }
}

/// Check if an array can be encoded as tabular format.
fn is_tabular_array(arr: &[Value]) -> Option<Vec<String>> {
    if arr.is_empty() {
        return None;
    }

    let first = arr.first()?;
    let first_obj = first.as_object()?;

    // First object must have only primitive values
    for value in first_obj.values() {
        if !is_primitive(value) {
            return None;
        }
    }

    let keys: Vec<String> = first_obj.keys().cloned().collect();

    // All remaining objects must match
    for val in arr.iter().skip(1) {
        if let Some(obj) = val.as_object() {
            if obj.len() != keys.len() {
                return None;
            }
            for key in &keys {
                if !obj.contains_key(key) {
                    return None;
                }
            }
            for value in obj.values() {
                if !is_primitive(value) {
                    return None;
                }
            }
        } else {
            return None;
        }
    }

    Some(keys)
}

fn is_primitive(value: &Value) -> bool {
    matches!(
        value,
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
    )
}

fn is_primitive_array(arr: &[Value]) -> bool {
    arr.iter().all(is_primitive)
}

fn encode_primitive_array(
    writer: &mut ToonWriter,
    key: Option<&str>,
    arr: &[Value],
    depth: usize,
) -> Result<()> {
    if depth > 0 {
        writer.write_indent(depth);
    }
    writer.write_array_header(key, arr.len(), None);
    writer.write_char(' ');

    for (i, val) in arr.iter().enumerate() {
        if i > 0 {
            writer.write_delimiter();
        }
        write_primitive(writer, val);
    }

    Ok(())
}

fn encode_tabular_array(
    writer: &mut ToonWriter,
    key: Option<&str>,
    arr: &[Value],
    fields: &[String],
    depth: usize,
) -> Result<()> {
    if depth > 0 {
        writer.write_indent(depth);
    }
    writer.write_array_header(key, arr.len(), Some(fields));
    writer.write_newline();

    // Write rows at depth + 1
    for (row_idx, obj_val) in arr.iter().enumerate() {
        if let Some(obj) = obj_val.as_object() {
            writer.write_indent(depth + 1);

            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    writer.write_delimiter();
                }

                if let Some(val) = obj.get(field) {
                    write_primitive(writer, val);
                } else {
                    writer.write_str("null");
                }
            }

            if row_idx < arr.len() - 1 {
                writer.write_newline();
            }
        }
    }

    Ok(())
}

fn encode_nested_array(
    writer: &mut ToonWriter,
    key: Option<&str>,
    arr: &[Value],
    depth: usize,
) -> Result<()> {
    if depth > 0 {
        writer.write_indent(depth);
    }
    writer.write_array_header(key, arr.len(), None);
    writer.write_newline();

    for (i, val) in arr.iter().enumerate() {
        writer.write_indent(depth + 1);
        writer.write_char('-');

        match val {
            Value::Array(inner_arr) => {
                writer.write_char(' ');
                // For nested arrays, write at current position (no key)
                encode_array_inline(writer, inner_arr, depth + 1)?;
            }
            Value::Object(obj) => {
                encode_list_item_object(writer, obj, depth + 1)?;
            }
            _ => {
                writer.write_char(' ');
                write_primitive(writer, val);
            }
        }

        if i < arr.len() - 1 {
            writer.write_newline();
        }
    }

    Ok(())
}

/// Encode an object inside a list item (- key: value format).
fn encode_list_item_object(
    writer: &mut ToonWriter,
    obj: &serde_json::Map<String, Value>,
    depth: usize,
) -> Result<()> {
    let keys: Vec<&String> = obj.keys().collect();

    if keys.is_empty() {
        return Ok(());
    }

    // First field on same line as hyphen
    let first_key = keys[0];
    let first_val = &obj[first_key];

    writer.write_char(' ');

    match first_val {
        Value::Array(arr) => {
            // Arrays as first field
            writer.write_key(first_key);
            if let Some(fields) = is_tabular_array(arr) {
                // Tabular array: write header inline, rows at depth + 2
                encode_list_item_first_field_tabular(writer, arr, &fields, depth)?;
            } else if is_primitive_array(arr) {
                // Inline primitive array
                writer.write_array_header(None, arr.len(), None);
                writer.write_char(' ');
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        writer.write_delimiter();
                    }
                    write_primitive(writer, v);
                }
            } else {
                // Non-uniform array
                encode_array_inline(writer, arr, depth + 2)?;
            }
        }
        Value::Object(nested_obj) => {
            writer.write_key(first_key);
            writer.write_char(':');
            if !nested_obj.is_empty() {
                writer.write_newline();
                encode_object(writer, nested_obj, depth + 2)?;
            }
        }
        _ => {
            writer.write_key(first_key);
            writer.write_str(": ");
            write_primitive(writer, first_val);
        }
    }

    // Remaining fields on separate lines at depth + 1
    for key in keys.iter().skip(1) {
        writer.write_newline();
        writer.write_indent(depth + 1);

        let value = &obj[*key];

        match value {
            Value::Array(arr) => {
                writer.write_key(key);
                if let Some(fields) = is_tabular_array(arr) {
                    // FIXED: Tabular array as non-first field, rows at depth + 2
                    encode_non_first_field_tabular(writer, arr, &fields, depth + 1)?;
                } else if is_primitive_array(arr) {
                    writer.write_array_header(None, arr.len(), None);
                    writer.write_char(' ');
                    for (i, v) in arr.iter().enumerate() {
                        if i > 0 {
                            writer.write_delimiter();
                        }
                        write_primitive(writer, v);
                    }
                } else {
                    encode_array_inline(writer, arr, depth + 2)?;
                }
            }
            Value::Object(nested_obj) => {
                writer.write_key(key);
                writer.write_char(':');
                if !nested_obj.is_empty() {
                    writer.write_newline();
                    encode_object(writer, nested_obj, depth + 2)?;
                }
            }
            _ => {
                writer.write_key(key);
                writer.write_str(": ");
                write_primitive(writer, value);
            }
        }
    }

    Ok(())
}

/// Encode tabular array as first field in a list item.
/// Header on same line as hyphen, rows at depth + 2.
fn encode_list_item_first_field_tabular(
    writer: &mut ToonWriter,
    arr: &[Value],
    fields: &[String],
    depth: usize,
) -> Result<()> {
    writer.write_array_header(None, arr.len(), Some(fields));
    writer.write_newline();

    for (row_idx, obj_val) in arr.iter().enumerate() {
        if let Some(obj) = obj_val.as_object() {
            writer.write_indent(depth + 2);

            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    writer.write_delimiter();
                }

                if let Some(val) = obj.get(field) {
                    write_primitive(writer, val);
                } else {
                    writer.write_str("null");
                }
            }

            if row_idx < arr.len() - 1 {
                writer.write_newline();
            }
        }
    }

    Ok(())
}

/// Encode tabular array as non-first field in a list item.
/// Header at current position, rows at depth + 1.
fn encode_non_first_field_tabular(
    writer: &mut ToonWriter,
    arr: &[Value],
    fields: &[String],
    depth: usize,
) -> Result<()> {
    writer.write_array_header(None, arr.len(), Some(fields));
    writer.write_newline();

    // FIXED: Rows at depth + 1 (which is depth + 2 from the hyphen line)
    for (row_idx, obj_val) in arr.iter().enumerate() {
        if let Some(obj) = obj_val.as_object() {
            writer.write_indent(depth + 1);

            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    writer.write_delimiter();
                }

                if let Some(val) = obj.get(field) {
                    write_primitive(writer, val);
                } else {
                    writer.write_str("null");
                }
            }

            if row_idx < arr.len() - 1 {
                writer.write_newline();
            }
        }
    }

    Ok(())
}

/// Encode an array inline (without leading indent).
fn encode_array_inline(writer: &mut ToonWriter, arr: &[Value], depth: usize) -> Result<()> {
    if arr.is_empty() {
        writer.write_array_header(None, 0, None);
        return Ok(());
    }

    if let Some(fields) = is_tabular_array(arr) {
        writer.write_array_header(None, arr.len(), Some(&fields));
        writer.write_newline();
        for (row_idx, obj_val) in arr.iter().enumerate() {
            if let Some(obj) = obj_val.as_object() {
                writer.write_indent(depth);
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        writer.write_delimiter();
                    }
                    if let Some(val) = obj.get(field) {
                        write_primitive(writer, val);
                    } else {
                        writer.write_str("null");
                    }
                }
                if row_idx < arr.len() - 1 {
                    writer.write_newline();
                }
            }
        }
    } else if is_primitive_array(arr) {
        writer.write_array_header(None, arr.len(), None);
        writer.write_char(' ');
        for (i, v) in arr.iter().enumerate() {
            if i > 0 {
                writer.write_delimiter();
            }
            write_primitive(writer, v);
        }
    } else {
        writer.write_array_header(None, arr.len(), None);
        writer.write_newline();
        for (i, val) in arr.iter().enumerate() {
            writer.write_indent(depth);
            writer.write_char('-');
            match val {
                Value::Object(obj) => {
                    encode_list_item_object(writer, obj, depth)?;
                }
                Value::Array(inner) => {
                    writer.write_char(' ');
                    encode_array_inline(writer, inner, depth + 1)?;
                }
                _ => {
                    writer.write_char(' ');
                    write_primitive(writer, val);
                }
            }
            if i < arr.len() - 1 {
                writer.write_newline();
            }
        }
    }

    Ok(())
}

fn write_primitive(writer: &mut ToonWriter, value: &Value) {
    match value {
        Value::Null => writer.write_str("null"),
        Value::Bool(b) => writer.write_str(if *b { "true" } else { "false" }),
        Value::Number(n) => writer.write_str(&format_number(n)),
        Value::String(s) => writer.write_value(s),
        _ => writer.write_str("null"), // Fallback for non-primitives
    }
}

/// Check if a string looks like a TOON number literal
/// (digits with optional sign, decimal point, exponent)
fn looks_like_toon_number(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars().peekable();

    // Optional leading sign
    if let Some(&c) = chars.peek() {
        if c == '+' || c == '-' {
            chars.next();
        }
    }

    // Must have at least one digit
    let mut has_digits = false;
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            has_digits = true;
            chars.next();
        } else {
            break;
        }
    }

    // Optional decimal point and more digits
    if let Some(&c) = chars.peek() {
        if c == '.' {
            chars.next();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    has_digits = true;
                    chars.next();
                } else {
                    break;
                }
            }
        }
    }

    // Optional exponent
    if let Some(&c) = chars.peek() {
        if c == 'e' || c == 'E' {
            chars.next();
            // Optional sign
            if let Some(&c) = chars.peek() {
                if c == '+' || c == '-' {
                    chars.next();
                }
            }
            // Must have exponent digits
            let mut has_exp_digits = false;
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    has_exp_digits = true;
                    chars.next();
                } else {
                    break;
                }
            }
            if !has_exp_digits {
                return false;
            }
        }
    }

    // Must have consumed all characters and have at least one digit
    has_digits && chars.peek().is_none()
}

fn format_number(n: &serde_json::Number) -> String {
    if let Some(i) = n.as_i64() {
        i.to_string()
    } else if let Some(u) = n.as_u64() {
        u.to_string()
    } else if let Some(f) = n.as_f64() {
        if f.is_nan() || f.is_infinite() {
            "null".to_string()
        } else if f == 0.0 {
            // Normalize -0.0 to 0 per TOON spec
            "0".to_string()
        } else if f.fract() == 0.0 && f.abs() < 1e15 {
            format!("{:.0}", f)
        } else {
            f.to_string()
        }
    } else {
        n.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_object() {
        let val = json!({"name": "Alice", "age": 30});
        let toon = encode_to_toon(&val, 2, ',').unwrap();
        assert!(toon.contains("name: Alice"));
        assert!(toon.contains("age: 30"));
    }

    #[test]
    fn test_primitive_array() {
        let val = json!({"tags": ["a", "b", "c"]});
        let toon = encode_to_toon(&val, 2, ',').unwrap();
        assert_eq!(toon, "tags[3]: a,b,c");
    }

    #[test]
    fn test_tabular_array() {
        let val = json!({"users": [{"id": 1, "name": "Ada"}]});
        let toon = encode_to_toon(&val, 2, ',').unwrap();
        assert!(toon.contains("users[1]{id,name}:"));
        assert!(toon.contains("1,Ada"));
    }

    #[test]
    fn test_nested_tabular_first_field() {
        let val = json!({
            "items": [{
                "users": [{"id": 1, "name": "Ada"}],
                "status": "active"
            }]
        });
        let toon = encode_to_toon(&val, 2, ',').unwrap();
        println!("TOON:\n{}", toon);

        // First field tabular: rows at 6 spaces
        assert!(toon.contains("  - users[1]{id,name}:"));
        assert!(toon.contains("      1,Ada"), "Rows should be at 6 spaces");
        assert!(toon.contains("    status: active"));
    }

    #[test]
    fn test_nested_tabular_non_first_field() {
        // This is the bug case in toon-format 0.4.x
        let val = json!({
            "items": [{
                "id": "acme",
                "users": [{"id": 1, "name": "Ada"}]
            }]
        });
        let toon = encode_to_toon(&val, 2, ',').unwrap();
        println!("TOON:\n{}", toon);

        // Check each line's indentation
        for (i, line) in toon.lines().enumerate() {
            let spaces = line.len() - line.trim_start().len();
            println!("Line {}: {} spaces: '{}'", i, spaces, line);
        }

        // Non-first field tabular: rows should ALSO be at 6 spaces
        assert!(toon.contains("  - id: acme"));
        assert!(toon.contains("    users[1]{id,name}:"));
        assert!(
            toon.contains("      1,Ada"),
            "Rows should be at 6 spaces (depth + 2 from hyphen line)"
        );
    }
}

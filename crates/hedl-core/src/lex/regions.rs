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

//! Protected region scanning for HEDL.
//!
//! Identifies quoted strings and expressions where special characters
//! like `#` and `,` lose their usual meaning.

/// Type of protected region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionType {
    Quote,
    Expression,
}

/// A protected region in a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub start: usize,
    pub end: usize,
    pub region_type: RegionType,
}

/// Scan a line for protected regions (quoted strings and expressions).
///
/// Returns a list of regions where special characters should not be interpreted.
pub fn scan_regions(line: &str) -> Vec<Region> {
    let mut regions = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'"' {
            // Start of quoted string
            let start = i;
            i += 1;

            while i < bytes.len() {
                if bytes[i] == b'"' {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                        // Escaped quote
                        i += 2;
                    } else {
                        // End of quoted string
                        regions.push(Region {
                            start,
                            end: i + 1,
                            region_type: RegionType::Quote,
                        });
                        i += 1;
                        break;
                    }
                } else {
                    i += 1;
                }
            }

            // If we hit end of line without closing quote, mark region to end
            if i >= bytes.len() && (regions.is_empty() || regions.last().unwrap().start != start) {
                regions.push(Region {
                    start,
                    end: bytes.len(),
                    region_type: RegionType::Quote,
                });
            }
        } else if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'(' {
            // Start of expression
            let start = i;
            i += 2;
            let mut depth = 1;
            let mut in_expr_quotes = false;

            while i < bytes.len() && depth > 0 {
                let b = bytes[i];

                if b == b'"' {
                    if in_expr_quotes {
                        if i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                            i += 2;
                            continue;
                        } else {
                            in_expr_quotes = false;
                        }
                    } else {
                        in_expr_quotes = true;
                    }
                }

                if !in_expr_quotes {
                    if b == b'(' {
                        depth += 1;
                    } else if b == b')' {
                        depth -= 1;
                    }
                }

                i += 1;
            }

            regions.push(Region {
                start,
                end: i,
                region_type: RegionType::Expression,
            });
        } else {
            i += 1;
        }
    }

    regions
}

/// Strip inline comment from a line, respecting protected regions.
///
/// Returns the line with comment removed (trimmed).
pub fn strip_comment(line: &str) -> &str {
    let bytes = line.as_bytes();

    // Fast path: no comment character at all
    let hash_pos = match memchr::memchr(b'#', bytes) {
        Some(pos) => pos,
        None => return line.trim_end(),
    };

    // Fast path: no protected regions possible (no quotes or expressions)
    // If # comes before any " or $( we can strip directly
    let has_quote = memchr::memchr(b'"', bytes).is_some_and(|p| p < hash_pos);
    let has_expr = bytes
        .windows(2)
        .position(|w| w == b"$(")
        .is_some_and(|p| p < hash_pos);

    if !has_quote && !has_expr {
        // No protected regions before the #, safe to strip
        return line[..hash_pos].trim_end();
    }

    // Slow path: need to scan regions to find unprotected #
    let regions = scan_regions(line);

    for (i, &b) in bytes.iter().enumerate() {
        if b == b'#' {
            // Check if this position is inside a protected region
            let in_region = regions.iter().any(|r| r.start <= i && i < r.end);
            if !in_region {
                return line[..i].trim_end();
            }
        }
    }

    line.trim_end()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_regions_quote() {
        let regions = scan_regions(r#"hello "world" there"#);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].region_type, RegionType::Quote);
        assert_eq!(regions[0].start, 6);
        assert_eq!(regions[0].end, 13);
    }

    #[test]
    fn test_scan_regions_expression() {
        let regions = scan_regions("value: $(x + 1)");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].region_type, RegionType::Expression);
    }

    #[test]
    fn test_strip_comment_simple() {
        assert_eq!(strip_comment("hello # comment"), "hello");
        assert_eq!(strip_comment("hello"), "hello");
    }

    #[test]
    fn test_strip_comment_in_quote() {
        assert_eq!(
            strip_comment(r#""hello # not comment" # comment"#),
            r#""hello # not comment""#
        );
    }

    #[test]
    fn test_strip_comment_in_expression() {
        assert_eq!(strip_comment("$(x # y) # comment"), "$(x # y)");
    }

    // ==================== Scan regions: multiple regions ====================

    #[test]
    fn test_scan_regions_multiple_quotes() {
        let regions = scan_regions(r#""a" and "b" and "c""#);
        assert_eq!(regions.len(), 3);
        assert!(regions.iter().all(|r| r.region_type == RegionType::Quote));
    }

    #[test]
    fn test_scan_regions_multiple_expressions() {
        let regions = scan_regions("$(a) and $(b) and $(c)");
        assert_eq!(regions.len(), 3);
        assert!(regions
            .iter()
            .all(|r| r.region_type == RegionType::Expression));
    }

    #[test]
    fn test_scan_regions_mixed() {
        let regions = scan_regions(r#""hello" $(x) "world""#);
        assert_eq!(regions.len(), 3);
        assert_eq!(regions[0].region_type, RegionType::Quote);
        assert_eq!(regions[1].region_type, RegionType::Expression);
        assert_eq!(regions[2].region_type, RegionType::Quote);
    }

    #[test]
    fn test_scan_regions_adjacent() {
        // "a""b""c" is ONE region with escaped quotes inside (CSV-style)
        // The "" in the middle are escaped quote characters
        let regions = scan_regions(r#""a""b""c""#);
        assert_eq!(regions.len(), 1);
    }

    #[test]
    fn test_scan_regions_truly_adjacent() {
        // To have truly adjacent strings, need space between them
        let regions = scan_regions(r#""a" "b" "c""#);
        assert_eq!(regions.len(), 3);
    }

    #[test]
    fn test_scan_regions_empty_quote() {
        let regions = scan_regions(r#""""#);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].start, 0);
        assert_eq!(regions[0].end, 2);
    }

    // ==================== Scan regions: nested parens in expressions ====================

    #[test]
    fn test_scan_regions_nested_parens() {
        let regions = scan_regions("$(outer(inner(x)))");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].region_type, RegionType::Expression);
        assert_eq!(regions[0].start, 0);
        assert_eq!(regions[0].end, 18);
    }

    #[test]
    fn test_scan_regions_deeply_nested() {
        let regions = scan_regions("$((((deep))))");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].end, 13);
    }

    // ==================== Scan regions: quotes in expressions ====================

    #[test]
    fn test_scan_regions_quote_in_expression() {
        let regions = scan_regions(r#"$(concat("a", "b"))"#);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].region_type, RegionType::Expression);
    }

    #[test]
    fn test_scan_regions_escaped_quote_in_expression() {
        let regions = scan_regions(r#"$(say("hello""world"))"#);
        assert_eq!(regions.len(), 1);
    }

    #[test]
    fn test_scan_regions_paren_in_quote_in_expression() {
        // Parentheses inside quotes should not affect depth
        let regions = scan_regions(r#"$(foo("()"))"#);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].end, 12);
    }

    // ==================== Scan regions: unclosed ====================

    #[test]
    fn test_scan_regions_unclosed_quote() {
        let regions = scan_regions(r#""unclosed"#);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].region_type, RegionType::Quote);
        assert_eq!(regions[0].end, 9); // extends to end of line
    }

    #[test]
    fn test_scan_regions_unclosed_expression() {
        let regions = scan_regions("$(unclosed");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].region_type, RegionType::Expression);
    }

    // ==================== Scan regions: escaped quotes ====================

    #[test]
    fn test_scan_regions_escaped_quote_at_end() {
        // "test"" - has escaped quote at the end
        let regions = scan_regions(r#""test""""#);
        assert_eq!(regions.len(), 1);
        // The entire thing is one region with escaped quotes
    }

    #[test]
    fn test_scan_regions_with_escaped_quotes() {
        // "a""b""c" is actually one string with escaped quotes inside
        // Content: a"b"c
        let regions = scan_regions(r#""a""b""c""#);
        assert_eq!(regions.len(), 1);
    }

    #[test]
    fn test_scan_regions_three_separate() {
        // To have three separate strings, need separator between them
        let regions = scan_regions(r#""a" "b" "c""#);
        assert_eq!(regions.len(), 3);
    }

    // ==================== Scan regions: edge cases ====================

    #[test]
    fn test_scan_regions_empty() {
        let regions = scan_regions("");
        assert!(regions.is_empty());
    }

    #[test]
    fn test_scan_regions_no_regions() {
        let regions = scan_regions("hello world");
        assert!(regions.is_empty());
    }

    #[test]
    fn test_scan_regions_dollar_alone() {
        // Just $ without ( is not an expression
        let regions = scan_regions("$100");
        assert!(regions.is_empty());
    }

    #[test]
    fn test_scan_regions_dollar_at_end() {
        let regions = scan_regions("price: $");
        assert!(regions.is_empty());
    }

    // ==================== Strip comment: more tests ====================

    #[test]
    fn test_strip_comment_no_hash() {
        assert_eq!(strip_comment("hello world"), "hello world");
    }

    #[test]
    fn test_strip_comment_hash_at_start() {
        assert_eq!(strip_comment("# full line comment"), "");
    }

    #[test]
    fn test_strip_comment_multiple_hashes() {
        assert_eq!(strip_comment("value # first # second"), "value");
    }

    #[test]
    fn test_strip_comment_hash_in_multiple_quotes() {
        // Two quoted strings containing #, then a comment
        assert_eq!(strip_comment("\"#a\" \"#b\" # comment"), "\"#a\" \"#b\"");
    }

    #[test]
    fn test_strip_comment_nested_expression() {
        assert_eq!(
            strip_comment("$(outer(inner(#))) # comment"),
            "$(outer(inner(#)))"
        );
    }

    #[test]
    fn test_strip_comment_expression_then_quote() {
        // Expression, then quoted #, then comment
        assert_eq!(strip_comment("$(x) \"#\" # comment"), "$(x) \"#\"");
    }

    #[test]
    fn test_strip_comment_empty() {
        assert_eq!(strip_comment(""), "");
    }

    #[test]
    fn test_strip_comment_only_hash() {
        assert_eq!(strip_comment("#"), "");
    }

    #[test]
    fn test_strip_comment_trailing_whitespace() {
        assert_eq!(strip_comment("hello   "), "hello");
    }

    #[test]
    fn test_strip_comment_whitespace_before_hash() {
        assert_eq!(strip_comment("hello   # comment"), "hello");
    }

    // ==================== Region struct tests ====================

    #[test]
    fn test_region_equality() {
        let a = Region {
            start: 0,
            end: 5,
            region_type: RegionType::Quote,
        };
        let b = Region {
            start: 0,
            end: 5,
            region_type: RegionType::Quote,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_region_inequality() {
        let a = Region {
            start: 0,
            end: 5,
            region_type: RegionType::Quote,
        };
        let b = Region {
            start: 0,
            end: 5,
            region_type: RegionType::Expression,
        };
        assert_ne!(a, b);
    }

    #[test]
    fn test_region_clone() {
        let original = Region {
            start: 10,
            end: 20,
            region_type: RegionType::Expression,
        };
        let cloned = original;
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_region_debug() {
        let region = Region {
            start: 0,
            end: 5,
            region_type: RegionType::Quote,
        };
        let debug = format!("{:?}", region);
        assert!(debug.contains("start"));
        assert!(debug.contains("end"));
        assert!(debug.contains("Quote"));
    }

    #[test]
    fn test_region_type_equality() {
        assert_eq!(RegionType::Quote, RegionType::Quote);
        assert_eq!(RegionType::Expression, RegionType::Expression);
        assert_ne!(RegionType::Quote, RegionType::Expression);
    }

    #[test]
    fn test_region_type_debug() {
        assert_eq!(format!("{:?}", RegionType::Quote), "Quote");
        assert_eq!(format!("{:?}", RegionType::Expression), "Expression");
    }

    // ==================== Complex scenarios ====================

    #[test]
    fn test_strip_comment_real_world() {
        // Real HEDL line examples
        assert_eq!(
            strip_comment("name: \"John Doe\" # User's name"),
            "name: \"John Doe\""
        );
        assert_eq!(
            strip_comment("value: $(calculate(x)) # Computed"),
            "value: $(calculate(x))"
        );
        assert_eq!(
            strip_comment(r#"msg: "Hello # World" # greeting"#),
            r#"msg: "Hello # World""#
        );
    }

    #[test]
    fn test_scan_regions_real_world() {
        // A complex HEDL value line
        let regions = scan_regions(r#"name: "John", age: $(years), city: "NYC""#);
        assert_eq!(regions.len(), 3);
        assert_eq!(regions[0].region_type, RegionType::Quote);
        assert_eq!(regions[1].region_type, RegionType::Expression);
        assert_eq!(regions[2].region_type, RegionType::Quote);
    }

    #[test]
    fn test_unicode_in_regions() {
        let regions = scan_regions(r#""日本語" $(émoji)"#);
        assert_eq!(regions.len(), 2);
    }

    #[test]
    fn test_very_long_line() {
        let long_str = format!("\"{}\"", "x".repeat(10000));
        let regions = scan_regions(&long_str);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].end, 10002);
    }
}

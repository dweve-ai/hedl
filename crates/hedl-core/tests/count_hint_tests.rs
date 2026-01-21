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

//! Integration tests for parenthesized count hint parsing across contexts.

use hedl_core::parse;

#[test]
fn test_struct_with_count_hint() {
    let input = b"%VERSION: 1.0
%STRUCT: Company (10): [id, name, founded]
---
";
    let doc = parse(input).unwrap();
    assert_eq!(doc.version, (1, 0));
    assert!(doc.structs.contains_key("Company"));
}

#[test]
fn test_struct_with_zero_count() {
    let input = b"%VERSION: 1.0
%STRUCT: Empty (0): [id]
---
";
    let result = parse(input);
    assert!(result.is_ok());
}

#[test]
fn test_struct_with_leading_zeros_rejected() {
    let input = b"%VERSION: 1.0
%STRUCT: Company (01): [id]
---
";
    let result = parse(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("leading zeros"));
}

#[test]
fn test_struct_with_trailing_content_rejected() {
    let input = b"%VERSION: 1.0
%STRUCT: Company (10) extra: [id]
---
";
    let result = parse(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("unexpected content"));
}

#[test]
fn test_list_count_hint_deprecated_syntax() {
    let input = b"%VERSION: 1.0
%STRUCT: Team: [id, name]
---
teams(3): @Team
  | t1, Alice
  | t2, Bob
  | t3, Carol
";
    let doc = parse(input).unwrap();
    // Count hint is parsed but not enforced (deprecated)
    assert!(doc.root.contains_key("teams"));
}

#[test]
fn test_list_count_hint_zero_rejected() {
    let input = b"%VERSION: 1.0
%STRUCT: Team: [id, name]
---
teams(0): @Team
";
    let result = parse(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("greater than zero"));
}

#[test]
fn test_both_contexts_in_same_document() {
    let input = b"%VERSION: 1.0
%STRUCT: Company (5): [id, name]
%STRUCT: Division: [id, name]
---
companies: @Company
  | c1, Acme
divisions(3): @Division
  | d1, Engineering
  | d2, Sales
  | d3, Marketing
";
    let doc = parse(input).unwrap();
    assert!(doc.structs.contains_key("Company"));
    assert!(doc.root.contains_key("companies"));
    assert!(doc.root.contains_key("divisions"));
}

#[test]
fn test_struct_without_count_hint() {
    let input = b"%VERSION: 1.0
%STRUCT: NormalStruct: [id, name]
---
";
    let doc = parse(input).unwrap();
    assert!(doc.structs.contains_key("NormalStruct"));
}

#[test]
fn test_struct_with_spaces_in_count() {
    let input = b"%VERSION: 1.0
%STRUCT: Company ( 10 ): [id, name]
---
";
    let doc = parse(input).unwrap();
    assert!(doc.structs.contains_key("Company"));
}

#[test]
fn test_struct_with_large_count() {
    let input = b"%VERSION: 1.0
%STRUCT: BigStruct (999999): [id]
---
";
    let doc = parse(input).unwrap();
    assert!(doc.structs.contains_key("BigStruct"));
}

#[test]
fn test_list_without_count_hint() {
    let input = b"%VERSION: 1.0
%STRUCT: Team: [id, name]
---
teams: @Team
  | t1, Alice
  | t2, Bob
";
    let doc = parse(input).unwrap();
    assert!(doc.root.contains_key("teams"));
}

#[test]
fn test_list_with_valid_count_hint() {
    let input = b"%VERSION: 1.0
%STRUCT: Team: [id, name]
---
teams(2): @Team
  | t1, Alice
  | t2, Bob
";
    let doc = parse(input).unwrap();
    assert!(doc.root.contains_key("teams"));
}

#[test]
fn test_struct_unclosed_parenthesis_rejected() {
    let input = b"%VERSION: 1.0
%STRUCT: Company (10: [id]
---
";
    let result = parse(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("unclosed"));
}

#[test]
fn test_struct_invalid_count_format_rejected() {
    let input = b"%VERSION: 1.0
%STRUCT: Company (abc): [id]
---
";
    let result = parse(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("invalid count"));
}

#[test]
fn test_list_unclosed_parenthesis_rejected() {
    let input = b"%VERSION: 1.0
%STRUCT: Team: [id, name]
---
teams(10: @Team
";
    let result = parse(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("unclosed"));
}

#[test]
fn test_list_invalid_count_format_rejected() {
    let input = b"%VERSION: 1.0
%STRUCT: Team: [id, name]
---
teams(xyz): @Team
";
    let result = parse(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("invalid count"));
}

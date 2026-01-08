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


use hedl_core::parse;

#[test]
fn test_count_hint_parsing() {
    let input = b"%VERSION: 1.0
%STRUCT: Team: [id,name]
---
teams(3): @Team
  | t1,Warriors
  | t2,Lakers
  | t3,Celtics

no_hint: @Team
  | t4,Heat
";

    let doc = parse(input).expect("parse should succeed");

    // Check teams list has count hint
    let teams_item = doc.get("teams").expect("teams should exist");
    let teams_list = teams_item.as_list().expect("teams should be a list");
    assert_eq!(teams_list.count_hint, Some(3), "teams should have count hint of 3");
    assert_eq!(teams_list.rows.len(), 3, "teams should have 3 rows");

    // Check no_hint list doesn't have count hint
    let no_hint_item = doc.get("no_hint").expect("no_hint should exist");
    let no_hint_list = no_hint_item.as_list().expect("no_hint should be a list");
    assert_eq!(no_hint_list.count_hint, None, "no_hint should have no count hint");
    assert_eq!(no_hint_list.rows.len(), 1, "no_hint should have 1 row");
}

#[test]
fn test_count_hint_with_inline_schema() {
    let input = b"%VERSION: 1.0
---
players(2): @Player[id,name,position]
  | p1,Curry,Guard
  | p2,James,Forward
";

    let doc = parse(input).expect("parse should succeed");

    let players_item = doc.get("players").expect("players should exist");
    let players_list = players_item.as_list().expect("players should be a list");
    assert_eq!(players_list.count_hint, Some(2), "players should have count hint of 2");
    assert_eq!(players_list.schema, vec!["id", "name", "position"]);
}

#[test]
fn test_invalid_count_hint_zero() {
    let input = b"%VERSION: 1.0
%STRUCT: Team: [id,name]
---
teams(0): @Team
  | t1,Warriors
";

    let result = parse(input);
    assert!(result.is_err(), "zero count hint should fail");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("count hint must be greater than zero"));
}

#[test]
fn test_invalid_count_hint_non_numeric() {
    let input = b"%VERSION: 1.0
%STRUCT: Team: [id,name]
---
teams(abc): @Team
  | t1,Warriors
";

    let result = parse(input);
    assert!(result.is_err(), "non-numeric count hint should fail");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("invalid count hint"));
}

#[test]
fn test_count_hint_not_allowed_on_objects() {
    let input = b"%VERSION: 1.0
---
config(5):
  key: value
";

    let result = parse(input);
    assert!(result.is_err(), "count hint on object should fail");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("count hint not allowed on object"));
}

#[test]
fn test_count_hint_not_allowed_on_scalars() {
    let input = b"%VERSION: 1.0
---
name(5): Alice
";

    let result = parse(input);
    assert!(result.is_err(), "count hint on scalar should fail");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("count hint not allowed on scalar"));
}

#[test]
fn test_unclosed_count_hint_parenthesis() {
    let input = b"%VERSION: 1.0
%STRUCT: Team: [id,name]
---
teams(3: @Team
  | t1,Warriors
";

    let result = parse(input);
    assert!(result.is_err(), "unclosed parenthesis should fail");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("unclosed count hint parenthesis"));
}

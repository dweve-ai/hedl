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

//! Cypher statement types and builders.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A Cypher parameter value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CypherValue {
    /// Null value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Integer value.
    Int(i64),
    /// Floating-point value.
    Float(f64),
    /// String value.
    String(String),
    /// List value.
    List(Vec<CypherValue>),
    /// Map/object value.
    Map(BTreeMap<String, CypherValue>),
}

impl From<bool> for CypherValue {
    fn from(v: bool) -> Self {
        CypherValue::Bool(v)
    }
}

impl From<i64> for CypherValue {
    fn from(v: i64) -> Self {
        CypherValue::Int(v)
    }
}

impl From<i32> for CypherValue {
    fn from(v: i32) -> Self {
        CypherValue::Int(v as i64)
    }
}

impl From<f64> for CypherValue {
    fn from(v: f64) -> Self {
        CypherValue::Float(v)
    }
}

impl From<String> for CypherValue {
    fn from(v: String) -> Self {
        CypherValue::String(v)
    }
}

impl From<&str> for CypherValue {
    fn from(v: &str) -> Self {
        CypherValue::String(v.to_string())
    }
}

impl<T: Into<CypherValue>> From<Vec<T>> for CypherValue {
    fn from(v: Vec<T>) -> Self {
        CypherValue::List(v.into_iter().map(|x| x.into()).collect())
    }
}

impl<T: Into<CypherValue>> From<Option<T>> for CypherValue {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(x) => x.into(),
            None => CypherValue::Null,
        }
    }
}

impl CypherValue {
    /// Convert to Cypher literal syntax.
    pub fn to_cypher_literal(&self) -> String {
        match self {
            CypherValue::Null => "null".to_string(),
            CypherValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            CypherValue::Int(i) => i.to_string(),
            CypherValue::Float(f) => {
                if f.is_nan() {
                    "0.0/0.0".to_string() // NaN in Cypher
                } else if f.is_infinite() {
                    if *f > 0.0 {
                        "1.0/0.0".to_string()
                    } else {
                        "-1.0/0.0".to_string()
                    }
                } else {
                    let s = f.to_string();
                    if s.contains('.') || s.contains('e') || s.contains('E') {
                        s
                    } else {
                        format!("{}.0", s)
                    }
                }
            }
            CypherValue::String(s) => super::escape::quote_string(s),
            CypherValue::List(items) => {
                let inner: Vec<String> = items.iter().map(|v| v.to_cypher_literal()).collect();
                format!("[{}]", inner.join(", "))
            }
            CypherValue::Map(map) => {
                let pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}: {}",
                            super::escape::escape_identifier(k),
                            v.to_cypher_literal()
                        )
                    })
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
        }
    }

    /// Check if this is a null value.
    pub fn is_null(&self) -> bool {
        matches!(self, CypherValue::Null)
    }

    /// Try to get as a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            CypherValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as an integer.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            CypherValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get as a float.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            CypherValue::Float(f) => Some(*f),
            CypherValue::Int(i) => Some(*i as f64),
            _ => None,
        }
    }
}

/// The type of Cypher statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatementType {
    /// Constraint creation.
    Constraint,
    /// Index creation.
    Index,
    /// Node creation (CREATE or MERGE).
    CreateNode,
    /// Relationship creation.
    CreateRelationship,
    /// Node property update.
    SetProperty,
    /// General query.
    Query,
}

/// A single Cypher statement with optional parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CypherStatement {
    /// The Cypher query text.
    pub query: String,
    /// Optional parameters for the query.
    pub parameters: BTreeMap<String, CypherValue>,
    /// Type of statement.
    pub statement_type: StatementType,
    /// Optional comment describing the statement.
    pub comment: Option<String>,
}

impl CypherStatement {
    /// Create a new Cypher statement.
    pub fn new(query: impl Into<String>, statement_type: StatementType) -> Self {
        Self {
            query: query.into(),
            parameters: BTreeMap::new(),
            statement_type,
            comment: None,
        }
    }

    /// Create a constraint statement.
    pub fn constraint(query: impl Into<String>) -> Self {
        Self::new(query, StatementType::Constraint)
    }

    /// Create an index statement.
    pub fn index(query: impl Into<String>) -> Self {
        Self::new(query, StatementType::Index)
    }

    /// Create a node creation statement.
    pub fn create_node(query: impl Into<String>) -> Self {
        Self::new(query, StatementType::CreateNode)
    }

    /// Create a relationship creation statement.
    pub fn create_relationship(query: impl Into<String>) -> Self {
        Self::new(query, StatementType::CreateRelationship)
    }

    /// Create a property update statement.
    pub fn set_property(query: impl Into<String>) -> Self {
        Self::new(query, StatementType::SetProperty)
    }

    /// Create a general query statement.
    pub fn query(query: impl Into<String>) -> Self {
        Self::new(query, StatementType::Query)
    }

    /// Add a parameter to this statement.
    pub fn with_param(mut self, name: impl Into<String>, value: impl Into<CypherValue>) -> Self {
        self.parameters.insert(name.into(), value.into());
        self
    }

    /// Add multiple parameters to this statement.
    pub fn with_params(mut self, params: impl IntoIterator<Item = (String, CypherValue)>) -> Self {
        self.parameters.extend(params);
        self
    }

    /// Add a comment to this statement.
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Check if this statement has parameters.
    pub fn has_parameters(&self) -> bool {
        !self.parameters.is_empty()
    }

    /// Render this statement as a string with embedded values.
    ///
    /// This inlines parameter values directly into the query. Use this when
    /// you can't use parameterized queries.
    pub fn render_inline(&self) -> String {
        let mut result = self.query.clone();
        for (name, value) in &self.parameters {
            let placeholder = format!("${}", name);
            result = result.replace(&placeholder, &value.to_cypher_literal());
        }
        result
    }

    /// Format this statement with optional comment prefix.
    ///
    /// This inlines all parameters so the result is directly executable.
    pub fn format(&self, include_comment: bool) -> String {
        let mut lines = Vec::new();

        if include_comment {
            if let Some(comment) = &self.comment {
                lines.push(format!("// {}", comment));
            }
        }

        // Use render_inline to substitute parameters
        lines.push(format!("{};", self.render_inline()));

        lines.join("\n")
    }
}

/// A collection of Cypher statements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CypherScript {
    /// The statements in this script.
    pub statements: Vec<CypherStatement>,
}

impl CypherScript {
    /// Create a new empty script.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a statement to the script.
    pub fn add(&mut self, statement: CypherStatement) {
        self.statements.push(statement);
    }

    /// Add multiple statements to the script.
    pub fn extend(&mut self, statements: impl IntoIterator<Item = CypherStatement>) {
        self.statements.extend(statements);
    }

    /// Get all statements of a specific type.
    pub fn statements_of_type(&self, statement_type: StatementType) -> Vec<&CypherStatement> {
        self.statements
            .iter()
            .filter(|s| s.statement_type == statement_type)
            .collect()
    }

    /// Render the script as a single string.
    pub fn render(&self, include_comments: bool) -> String {
        self.statements
            .iter()
            .map(|s| s.format(include_comments))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Get the number of statements.
    pub fn len(&self) -> usize {
        self.statements.len()
    }

    /// Check if the script is empty.
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }
}

impl IntoIterator for CypherScript {
    type Item = CypherStatement;
    type IntoIter = std::vec::IntoIter<CypherStatement>;

    fn into_iter(self) -> Self::IntoIter {
        self.statements.into_iter()
    }
}

impl<'a> IntoIterator for &'a CypherScript {
    type Item = &'a CypherStatement;
    type IntoIter = std::slice::Iter<'a, CypherStatement>;

    fn into_iter(self) -> Self::IntoIter {
        self.statements.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cypher_value_literals() {
        assert_eq!(CypherValue::Null.to_cypher_literal(), "null");
        assert_eq!(CypherValue::Bool(true).to_cypher_literal(), "true");
        assert_eq!(CypherValue::Bool(false).to_cypher_literal(), "false");
        assert_eq!(CypherValue::Int(42).to_cypher_literal(), "42");
        assert_eq!(CypherValue::Float(3.25).to_cypher_literal(), "3.25");
        assert_eq!(
            CypherValue::String("hello".to_string()).to_cypher_literal(),
            "'hello'"
        );
    }

    #[test]
    fn test_cypher_value_string_escaping() {
        assert_eq!(
            CypherValue::String("it's".to_string()).to_cypher_literal(),
            "'it\\'s'"
        );
    }

    #[test]
    fn test_cypher_value_list() {
        let list = CypherValue::List(vec![
            CypherValue::Int(1),
            CypherValue::Int(2),
            CypherValue::Int(3),
        ]);
        assert_eq!(list.to_cypher_literal(), "[1, 2, 3]");
    }

    #[test]
    fn test_cypher_value_map() {
        let mut map = BTreeMap::new();
        map.insert("name".to_string(), CypherValue::String("Alice".to_string()));
        map.insert("age".to_string(), CypherValue::Int(30));
        let value = CypherValue::Map(map);
        assert_eq!(value.to_cypher_literal(), "{age: 30, name: 'Alice'}");
    }

    #[test]
    fn test_cypher_value_from_impls() {
        assert_eq!(CypherValue::from(true), CypherValue::Bool(true));
        assert_eq!(CypherValue::from(42i64), CypherValue::Int(42));
        assert_eq!(CypherValue::from(42i32), CypherValue::Int(42));
        assert_eq!(CypherValue::from(3.25f64), CypherValue::Float(3.25));
        assert_eq!(
            CypherValue::from("hello"),
            CypherValue::String("hello".to_string())
        );
        assert_eq!(
            CypherValue::from("hello".to_string()),
            CypherValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_cypher_value_accessors() {
        assert!(CypherValue::Null.is_null());
        assert!(!CypherValue::Int(1).is_null());

        assert_eq!(CypherValue::String("hi".to_string()).as_str(), Some("hi"));
        assert_eq!(CypherValue::Int(42).as_str(), None);

        assert_eq!(CypherValue::Int(42).as_int(), Some(42));
        assert_eq!(CypherValue::Float(42.0).as_float(), Some(42.0));
        assert_eq!(CypherValue::Int(42).as_float(), Some(42.0));
    }

    #[test]
    fn test_statement_basic() {
        let stmt = CypherStatement::new("MATCH (n) RETURN n", StatementType::Query);
        assert_eq!(stmt.query, "MATCH (n) RETURN n");
        assert_eq!(stmt.statement_type, StatementType::Query);
        assert!(!stmt.has_parameters());
    }

    #[test]
    fn test_statement_with_params() {
        let stmt =
            CypherStatement::query("MATCH (n {name: $name}) RETURN n").with_param("name", "Alice");

        assert!(stmt.has_parameters());
        assert_eq!(
            stmt.parameters.get("name"),
            Some(&CypherValue::String("Alice".to_string()))
        );
    }

    #[test]
    fn test_statement_render_inline() {
        let stmt = CypherStatement::query("MATCH (n {name: $name, age: $age}) RETURN n")
            .with_param("name", "Alice")
            .with_param("age", 30i64);

        let rendered = stmt.render_inline();
        assert_eq!(rendered, "MATCH (n {name: 'Alice', age: 30}) RETURN n");
    }

    #[test]
    fn test_statement_format_with_comment() {
        let stmt = CypherStatement::constraint("CREATE CONSTRAINT user_id IF NOT EXISTS...")
            .with_comment("Ensure unique user IDs");

        let formatted = stmt.format(true);
        assert!(formatted.contains("// Ensure unique user IDs"));
        assert!(formatted.contains("CREATE CONSTRAINT"));
        assert!(formatted.ends_with(';'));
    }

    #[test]
    fn test_statement_format_without_comment() {
        let stmt = CypherStatement::constraint("CREATE CONSTRAINT user_id...")
            .with_comment("This won't show");

        let formatted = stmt.format(false);
        assert!(!formatted.contains("//"));
    }

    #[test]
    fn test_script_basic() {
        let mut script = CypherScript::new();
        assert!(script.is_empty());

        script.add(CypherStatement::constraint("CREATE CONSTRAINT..."));
        script.add(CypherStatement::create_node("CREATE (n:User)"));

        assert_eq!(script.len(), 2);
        assert!(!script.is_empty());
    }

    #[test]
    fn test_script_statements_of_type() {
        let mut script = CypherScript::new();
        script.add(CypherStatement::constraint("C1"));
        script.add(CypherStatement::create_node("N1"));
        script.add(CypherStatement::constraint("C2"));

        let constraints = script.statements_of_type(StatementType::Constraint);
        assert_eq!(constraints.len(), 2);
    }

    #[test]
    fn test_script_render() {
        let mut script = CypherScript::new();
        script.add(CypherStatement::constraint("CREATE CONSTRAINT c1").with_comment("First"));
        script.add(CypherStatement::create_node("CREATE (n:User)").with_comment("Second"));

        let rendered = script.render(true);
        assert!(rendered.contains("// First"));
        assert!(rendered.contains("// Second"));
        assert!(rendered.contains("CREATE CONSTRAINT c1;"));
        assert!(rendered.contains("CREATE (n:User);"));
    }

    #[test]
    fn test_cypher_value_float_edge_cases() {
        let nan = CypherValue::Float(f64::NAN);
        assert_eq!(nan.to_cypher_literal(), "0.0/0.0");

        let inf = CypherValue::Float(f64::INFINITY);
        assert_eq!(inf.to_cypher_literal(), "1.0/0.0");

        let neg_inf = CypherValue::Float(f64::NEG_INFINITY);
        assert_eq!(neg_inf.to_cypher_literal(), "-1.0/0.0");
    }
}

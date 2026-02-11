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

//! Fixtures for expression-based values.

use hedl_core::lex::Span;
use hedl_core::{Document, ExprLiteral, Expression, Item, Value};
use std::collections::BTreeMap;

/// Document with expression values.
///
/// Tests: simple expressions, expressions with variables.
#[must_use]
pub fn expressions() -> Document {
    let mut root = BTreeMap::new();

    // Simple function call: $(now())
    root.insert(
        "simple_expr".to_string(),
        Item::Scalar(Value::Expression(Box::new(Expression::Call {
            name: "now".to_string(),
            args: vec![],
            span: Span::synthetic(),
        }))),
    );

    // Field access: $(user.name)
    root.insert(
        "var_expr".to_string(),
        Item::Scalar(Value::Expression(Box::new(Expression::Access {
            target: Box::new(Expression::Identifier {
                name: "user".to_string(),
                span: Span::synthetic(),
            }),
            field: "name".to_string(),
            span: Span::synthetic(),
        }))),
    );

    // Function call with arguments: $(concat("hello", "world"))
    root.insert(
        "complex_expr".to_string(),
        Item::Scalar(Value::Expression(Box::new(Expression::Call {
            name: "concat".to_string(),
            args: vec![
                Expression::Literal {
                    value: ExprLiteral::String("hello".to_string()),
                    span: Span::synthetic(),
                },
                Expression::Literal {
                    value: ExprLiteral::String("world".to_string()),
                    span: Span::synthetic(),
                },
            ],
            span: Span::synthetic(),
        }))),
    );

    Document {
        version: (1, 2),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    }
}

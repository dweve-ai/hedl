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

//! Constraint generation for Neo4j.

use std::collections::BTreeMap;

use crate::config::ToCypherConfig;
use crate::cypher::{escape_identifier, escape_label, CypherStatement};
use crate::error::Result;

/// Generate uniqueness constraints for node types.
///
/// Creates `CREATE CONSTRAINT` statements for each node type to ensure
/// ID uniqueness. Constraints are named using the pattern: `{type}_{id_property}`.
///
/// # Arguments
///
/// * `node_types` - Map of type names to their schemas
/// * `config` - Configuration specifying the ID property name
///
/// # Returns
///
/// Vector of constraint creation statements.
pub(crate) fn generate_constraints(
    node_types: &BTreeMap<String, Vec<String>>,
    config: &ToCypherConfig,
) -> Result<Vec<CypherStatement>> {
    let mut statements = Vec::new();

    for type_name in node_types.keys() {
        let constraint_name = format!(
            "{}_{}",
            type_name.to_lowercase(),
            config.id_property.replace('.', "_")
        );

        let label = escape_label(type_name);
        let id_prop = escape_identifier(&config.id_property);

        let query = format!(
            "CREATE CONSTRAINT {} IF NOT EXISTS FOR (n{}) REQUIRE n.{} IS UNIQUE",
            escape_identifier(&constraint_name),
            label,
            id_prop
        );

        statements.push(
            CypherStatement::constraint(query)
                .with_comment(format!("Ensure unique {type_name} IDs")),
        );
    }

    Ok(statements)
}

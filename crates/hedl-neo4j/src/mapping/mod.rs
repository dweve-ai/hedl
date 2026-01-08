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

//! Mapping between HEDL types and Neo4j graph structures.
//!
//! This module provides types and functions for converting between
//! HEDL documents and Neo4j nodes and relationships.

pub mod node;
pub mod reference;
pub mod value;

pub use node::{
    extract_references, group_nodes_by_label, infer_schema_from_nodes, matrix_list_to_nodes,
    neo4j_to_node, node_to_neo4j, Neo4jNode, Neo4jRelationship,
};

pub use reference::{
    collect_node_ids, extract_relationships, group_relationships_by_source,
    group_relationships_by_type, infer_nests_from_relationships, validate_references, Nest,
};

pub use value::{cypher_to_value, unflatten_properties, value_to_cypher};

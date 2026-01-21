// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Example: Using relationship constants in hedl-neo4j
//!
//! This example demonstrates how to use the `NEST_RELATIONSHIP_PREFIX` constant
//! for working with Neo4j NEST relationships.

use hedl_neo4j::constants::{NEST_RELATIONSHIP_GENERIC, NEST_RELATIONSHIP_PREFIX};

fn main() {
    // Example 1: Generating NEST relationship types
    let child_types = ["Post", "Comment", "Like"];
    for child_type in &child_types {
        let rel_type = format!("{}{}", NEST_RELATIONSHIP_PREFIX, child_type.to_uppercase());
        println!("NEST relationship for {child_type}: {rel_type}");
    }

    // Example 2: Detecting NEST relationships
    let relationships = ["HAS_POST", "AUTHOR", "HAS_COMMENT", "TAG"];
    println!("\nNEST relationships:");
    for rel in &relationships {
        if rel.starts_with(NEST_RELATIONSHIP_PREFIX) {
            println!("  - {rel} (NEST)");
        }
    }

    // Example 3: Using generic NEST relationship
    println!("\nGeneric NEST relationship: {NEST_RELATIONSHIP_GENERIC}");
}

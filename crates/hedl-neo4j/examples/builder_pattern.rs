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


//! Example demonstrating the builder pattern for Neo4j configuration.

use hedl_neo4j::{FromNeo4jConfig, ObjectHandling, RelationshipNaming, ToCypherConfig};

fn main() {
    println!("=== Builder Pattern Examples ===\n");

    // Example 1: Simple ToCypherConfig with defaults
    println!("1. Simple ToCypherConfig with defaults:");
    let config = ToCypherConfig::builder().build();
    println!("   use_merge: {}", config.use_merge);
    println!("   batch_size: {}", config.batch_size);
    println!();

    // Example 2: Custom ToCypherConfig
    println!("2. Custom ToCypherConfig:");
    let config = ToCypherConfig::builder()
        .use_merge(false)
        .create_constraints(false)
        .id_property("custom_id")
        .batch_size(500)
        .build();
    println!("   use_merge: {}", config.use_merge);
    println!("   create_constraints: {}", config.create_constraints);
    println!("   id_property: {}", config.id_property);
    println!("   batch_size: {}", config.batch_size);
    println!();

    // Example 3: Full ToCypherConfig with all options
    println!("3. Full ToCypherConfig with all options:");
    let config = ToCypherConfig::builder()
        .use_merge(true)
        .create_constraints(false)
        .reference_naming(RelationshipNaming::Generic)
        .nest_naming(RelationshipNaming::TargetType)
        .object_handling(ObjectHandling::JsonString)
        .id_property("node_id")
        .batch_size(250)
        .include_type_metadata(true)
        .type_property("node_type")
        .include_comments(false)
        .max_string_length(1000000)
        .max_nodes(50000)
        .build();
    println!("   use_merge: {}", config.use_merge);
    println!("   reference_naming: {:?}", config.reference_naming);
    println!("   object_handling: {:?}", config.object_handling);
    println!("   batch_size: {}", config.batch_size);
    println!("   max_string_length: {:?}", config.max_string_length);
    println!();

    // Example 4: String length limits
    println!("4. String length limit variations:");
    let config_with_limit = ToCypherConfig::builder()
        .max_string_length(5000)
        .build();
    println!("   With limit: {:?}", config_with_limit.max_string_length);

    let config_no_limit = ToCypherConfig::builder().no_string_length_limit().build();
    println!("   No limit: {:?}", config_no_limit.max_string_length);
    println!();

    // Example 5: Simple FromNeo4jConfig
    println!("5. Simple FromNeo4jConfig:");
    let config = FromNeo4jConfig::builder()
        .version(2, 0)
        .id_property("nodeId")
        .build();
    println!("   version: {:?}", config.version);
    println!("   id_property: {}", config.id_property);
    println!();

    // Example 6: FromNeo4jConfig with collections
    println!("6. FromNeo4jConfig with collections:");
    let config = FromNeo4jConfig::builder()
        .version(2, 1)
        .id_property("custom_id")
        .infer_nests(false)
        .type_property("custom_type")
        .exclude_property("internal")
        .exclude_property("temp")
        .exclude_label("System")
        .exclude_label("Internal")
        .reference_relationship("AUTHORED_BY")
        .reference_relationship("CREATED_BY")
        .fallback_id(true)
        .build();
    println!("   version: {:?}", config.version);
    println!("   exclude_properties: {:?}", config.exclude_properties);
    println!("   exclude_labels: {:?}", config.exclude_labels);
    println!(
        "   reference_relationships: {:?}",
        config.reference_relationships
    );
    println!();

    // Example 7: Bulk collection configuration
    println!("7. Bulk collection configuration:");
    let config = FromNeo4jConfig::builder()
        .exclude_properties(vec!["prop1".to_string(), "prop2".to_string()])
        .exclude_labels(vec!["Label1".to_string(), "Label2".to_string()])
        .reference_relationships(vec!["REL1".to_string(), "REL2".to_string()])
        .build();
    println!("   exclude_properties: {:?}", config.exclude_properties);
    println!("   exclude_labels: {:?}", config.exclude_labels);
    println!(
        "   reference_relationships: {:?}",
        config.reference_relationships
    );
    println!();

    // Example 8: Mixed collection methods
    println!("8. Mixed collection methods:");
    let config = FromNeo4jConfig::builder()
        .exclude_properties(vec!["prop1".to_string()])
        .exclude_property("prop2")
        .exclude_property("prop3")
        .build();
    println!("   exclude_properties: {:?}", config.exclude_properties);
    println!();

    // Example 9: Comparison with fluent API
    println!("9. Builder vs Fluent API comparison:");

    let builder_config = ToCypherConfig::builder()
        .use_merge(false)
        .id_property("custom_id")
        .batch_size(500)
        .build();

    let fluent_config = ToCypherConfig::new()
        .with_create()
        .with_id_property("custom_id")
        .with_batch_size(500);

    println!("   Builder - use_merge: {}", builder_config.use_merge);
    println!("   Fluent  - use_merge: {}", fluent_config.use_merge);
    println!("   Both produce equivalent configurations!");
    println!();

    println!("=== All examples completed successfully! ===");
}

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

//! JSONPath query demonstration
//!
//! This example demonstrates how to use JSONPath queries to extract
//! specific data from HEDL documents efficiently.

use hedl_core::parse;
use hedl_json::jsonpath::{
    query, query_count, query_exists, query_first, query_single, QueryConfig, QueryConfigBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HEDL JSONPath Query Demo ===\n");

    // Sample HEDL document with user data
    let hedl = r#"
STRUCT User: id, name, email, age, active
STRUCT Post: id, title, author, views

users: @User
  u1, Alice, alice@example.com, 30, true
  u2, Bob, bob@example.com, 25, true
  u3, Charlie, charlie@example.com, 35, false

posts: @Post
  p1, "First Post", @User:u1, 1500
  p2, "Second Post", @User:u2, 2300
  p3, "Third Post", @User:u1, 800

config:
  database:
    host: "localhost"
    port: 5432
    max_connections: 100
  cache:
    enabled: true
    ttl: 300
"#;

    let doc = parse(hedl.as_bytes())?;
    let config = QueryConfig::default();

    // Example 1: Simple field access
    println!("1. Simple field access:");
    let results = query(&doc, "$.config.database.host", &config)?;
    if let Some(host) = results.first() {
        println!("   Database host: {}", host);
    }
    println!();

    // Example 2: Query first match
    println!("2. Query first match:");
    if let Some(port) = query_first(&doc, "$.config.database.port", &config)? {
        println!("   Database port: {}", port);
    }
    println!();

    // Example 3: Query single expected value
    println!("3. Query single value:");
    let ttl = query_single(&doc, "$.config.cache.ttl", &config)?;
    println!("   Cache TTL: {} seconds", ttl);
    println!();

    // Example 4: Check existence
    println!("4. Check field existence:");
    let has_cache = query_exists(&doc, "$.config.cache", &config)?;
    let has_redis = query_exists(&doc, "$.config.redis", &config)?;
    println!("   Has cache config: {}", has_cache);
    println!("   Has redis config: {}", has_redis);
    println!();

    // Example 5: Count matches
    println!("5. Count matches:");
    let field_count = query_count(&doc, "$.config.*", &config)?;
    println!("   Top-level config fields: {}", field_count);
    println!();

    // Example 6: Recursive descent
    println!("6. Recursive descent (find all 'enabled' fields):");
    let results = query(&doc, "$..enabled", &config)?;
    println!("   Found {} 'enabled' fields", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("   [{}] enabled = {}", i, result);
    }
    println!();

    // Example 7: Wildcard selection
    println!("7. Wildcard selection (all database config fields):");
    let results = query(&doc, "$.config.database.*", &config)?;
    println!("   Database configuration ({} fields):", results.len());
    for result in results {
        println!("     - {}", result);
    }
    println!();

    // Example 8: With max results limit
    println!("8. Limited results (max 2 config fields):");
    let limited_config = QueryConfigBuilder::new().max_results(2).build();
    let results = query(&doc, "$.config.*", &limited_config)?;
    println!("   Retrieved {} fields (limited from more):", results.len());
    for result in results {
        println!("     - {}", result);
    }
    println!();

    // Example 9: Complex nested query
    println!("9. Complex nested query:");
    let results = query(&doc, "$.config.database", &config)?;
    if let Some(db_config) = results.first() {
        println!("   Full database config:");
        println!("{}", serde_json::to_string_pretty(db_config)?);
    }
    println!();

    // Example 10: Query with array bracket notation
    println!("10. Array bracket notation:");
    let results = query(&doc, "$['config']['database']['host']", &config)?;
    if let Some(host) = results.first() {
        println!("    Database host (bracket notation): {}", host);
    }
    println!();

    // Example 11: Error handling
    println!("11. Error handling (invalid JSONPath):");
    match query(&doc, "$$invalid", &config) {
        Ok(_) => println!("    Unexpected success"),
        Err(e) => println!("    Caught error: {}", e),
    }
    println!();

    // Example 12: Multiple matches with filtering
    println!("12. Deep search for numeric values:");
    let results = query(&doc, "$..port", &config)?;
    println!("    Found {} 'port' fields:", results.len());
    for result in results {
        if let Some(port) = result.as_i64() {
            println!("      Port: {}", port);
        }
    }
    println!();

    // Performance demonstration
    println!("=== Performance Demo ===");
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = query(&doc, "$.config.database.host", &config)?;
    }
    let duration = start.elapsed();
    println!(
        "1000 queries executed in {:?} ({:.2} µs/query)",
        duration,
        duration.as_micros() as f64 / 1000.0
    );

    Ok(())
}

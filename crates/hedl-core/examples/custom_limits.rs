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

//! Example demonstrating custom security limits configuration.
//!
//! This example shows how to configure `max_total_keys` and other security
//! limits when parsing HEDL documents with different security requirements.

use hedl_core::{parse_with_limits, Limits, ParseOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("HEDL Custom Limits Example\n");

    // Example 1: Default limits (10M keys)
    println!("1. Default Limits (10M keys):");
    let default_limits = Limits::default();
    println!("   max_total_keys: {}", default_limits.max_total_keys);
    println!("   max_object_keys: {}", default_limits.max_object_keys);
    println!("   max_nodes: {}", default_limits.max_nodes);
    println!("   max_file_size: {} bytes\n", default_limits.max_file_size);

    // Example 2: Large dataset configuration (50M keys)
    println!("2. Large Dataset Configuration (50M keys):");
    let mut large_limits = Limits::default();
    large_limits.max_total_keys = 50_000_000;
    large_limits.max_nodes = 50_000_000;
    large_limits.max_file_size = 5 * 1024 * 1024 * 1024; // 5 GB

    let hedl_large = br#"%VERSION: 1.0
---
# Large dataset with many objects
users:
  alice: admin
  bob: user
  charlie: user
  # In a real large dataset, there would be millions of keys
"#;

    let options_large = ParseOptions {
        limits: large_limits.clone(),
        strict_refs: true,
    };

    match parse_with_limits(hedl_large, options_large) {
        Ok(doc) => println!("   Successfully parsed with large limits: {} aliases\n", doc.aliases.len()),
        Err(e) => println!("   Error: {}\n", e),
    }

    println!("   Configured limits:");
    println!("   max_total_keys: {}", large_limits.max_total_keys);
    println!("   max_nodes: {}", large_limits.max_nodes);
    println!("   max_file_size: {} GB\n", large_limits.max_file_size / (1024 * 1024 * 1024));

    // Example 3: Conservative limits for untrusted input (100k keys)
    println!("3. Conservative Limits for Untrusted Input (100k keys):");
    let conservative_limits = Limits {
        max_file_size: 10 * 1024 * 1024,     // 10 MB
        max_line_length: 100 * 1024,         // 100 KB
        max_indent_depth: 20,                // Shallow nesting
        max_nodes: 50_000,                   // 50k nodes
        max_aliases: 1_000,                  // 1k aliases
        max_columns: 50,                     // 50 columns
        max_nest_depth: 20,                  // Shallow NEST hierarchy
        max_block_string_size: 1024 * 1024, // 1 MB
        max_object_keys: 1_000,              // 1k keys per object
        max_total_keys: 100_000,             // 100k total keys
    };

    let hedl_small = br#"%VERSION: 1.0
---
user:
  name: Alice
  role: admin
"#;

    let options_conservative = ParseOptions {
        limits: conservative_limits.clone(),
        strict_refs: true,
    };

    match parse_with_limits(hedl_small, options_conservative) {
        Ok(doc) => println!("   Successfully parsed with conservative limits: {} items\n", doc.root.len()),
        Err(e) => println!("   Error: {}\n", e),
    }

    println!("   Configured limits:");
    println!("   max_total_keys: {}", conservative_limits.max_total_keys);
    println!("   max_file_size: {} MB", conservative_limits.max_file_size / (1024 * 1024));
    println!("   max_indent_depth: {}\n", conservative_limits.max_indent_depth);

    // Example 4: Demonstrating limit enforcement
    println!("4. Demonstrating Limit Enforcement:");
    let strict_limits = Limits {
        max_total_keys: 2, // Only allow 2 keys total
        ..Limits::default()
    };

    let hedl_too_many = br#"%VERSION: 1.0
---
key1: value1
key2: value2
key3: value3
"#;

    let options_strict = ParseOptions {
        limits: strict_limits,
        strict_refs: true,
    };

    match parse_with_limits(hedl_too_many, options_strict) {
        Ok(_) => println!("   Unexpected success!\n"),
        Err(e) => println!("   Expected error (too many keys): {}\n", e),
    }

    // Example 5: Memory estimation
    println!("5. Memory Estimation:");
    println!("   Approximate memory usage for max_total_keys:");
    println!("   - 1M keys ≈ 8 MB (key references only)");
    println!("   - 10M keys ≈ 80 MB (key references only)");
    println!("   - 50M keys ≈ 400 MB (key references only)");
    println!("\n   Note: Total memory usage will be higher due to:");
    println!("   - Key string storage");
    println!("   - Value storage");
    println!("   - Metadata and internal structures\n");

    // Example 6: Production recommendations
    println!("6. Production Recommendations:");
    println!("   - Web APIs (untrusted): max_total_keys = 100k - 1M");
    println!("   - Internal services: max_total_keys = 1M - 10M (default)");
    println!("   - Batch processing: max_total_keys = 10M - 100M");
    println!("   - Testing/development: Limits::unlimited()");
    println!("\n   Always profile your specific workload and adjust accordingly.");

    Ok(())
}

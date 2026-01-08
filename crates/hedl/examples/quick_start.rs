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

//! Quick Start Example
//!
//! This example demonstrates the core functionality of HEDL:
//! 1. Parsing HEDL documents
//! 2. Converting to JSON
//! 3. Basic validation
//! 4. Canonicalization
//!
//! Run with: cargo run --example quick_start

use hedl::{canonicalize, parse, to_json, validate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HEDL Quick Start Example ===\n");

    // 1. Define a simple HEDL document
    let hedl_text = r#"%VERSION: 1.0
%STRUCT: Product: [id, name, price, category]
---
# Product catalog
products: @Product
  | laptop, ThinkPad X1, 1299.99, electronics
  | mouse, Wireless Mouse, 29.99, accessories
  | keyboard, Mechanical Keyboard, 149.99, accessories

# Simple key-value pairs
store_name: Tech Paradise
location: San Francisco
"#;

    println!("Input HEDL document:");
    println!("{}", hedl_text);
    println!();

    // 2. Parse the document
    println!("--- Parsing ---");
    let doc = parse(hedl_text)?;
    println!("✓ Parsed successfully");
    println!("  Version: {}.{}", doc.version.0, doc.version.1);
    println!("  Structs: {} defined", doc.structs.len());
    println!("  Root items: {}", doc.root.len());
    println!();

    // 3. Validate the document
    println!("--- Validation ---");
    validate(hedl_text)?;
    println!("✓ Document is valid");
    println!();

    // 4. Convert to JSON
    println!("--- JSON Conversion ---");
    let json = to_json(&doc)?;
    println!("JSON output:");
    println!("{}", json);
    println!();

    // 5. Canonicalize (deterministic formatting)
    println!("--- Canonicalization ---");
    let canonical = canonicalize(&doc)?;
    println!("Canonical HEDL (sorted, deterministic):");
    println!("{}", canonical);

    Ok(())
}

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

//! Basic usage example for the HEDL library

use hedl::{canonicalize, lint, parse, to_json};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example HEDL document
    let hedl_text = r#"
%VERSION: 1.0
%STRUCT: User: [id,name,email,role]
---
users: @User
  | alice, Alice Smith, alice@example.com, admin
  | bob, Bob Jones, bob@example.com, user
  | charlie, Charlie Brown, charlie@example.com, user

config:
  max_connections: 100
  timeout_ms: 5000
  debug: true
"#;

    println!("=== Parsing HEDL ===");
    let doc = parse(hedl_text)?;
    println!("Version: {}.{}", doc.version.0, doc.version.1);
    println!("Structs: {:?}", doc.structs.keys().collect::<Vec<_>>());
    println!();

    println!("=== Canonicalization ===");
    let canonical = canonicalize(&doc)?;
    println!("{}", canonical);
    println!();

    println!("=== JSON Conversion ===");
    let json = to_json(&doc)?;
    println!("{}", json);
    println!();

    println!("=== Linting ===");
    let diagnostics = lint(&doc);
    if diagnostics.is_empty() {
        println!("No issues found!");
    } else {
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }
    }

    Ok(())
}

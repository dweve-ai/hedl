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


//! Compare hedl-toon output format with expected TOON v3.0 output

fn main() {
    println!("=== TOON Format Comparison ===\n");
    
    // Test 1: Simple tabular array
    println!("Test 1: Simple tabular array output:");
    let users = hedl_bench::generate_users(3);
    let doc = hedl_core::parse(users.as_bytes()).unwrap();
    let toon = hedl_toon::hedl_to_toon(&doc).unwrap();
    println!("{}", toon);
    println!();
    
    // Test 2: Nested objects
    println!("Test 2: Analytics with quoted strings:");
    let analytics = hedl_bench::generate_analytics(2);
    let doc = hedl_core::parse(analytics.as_bytes()).unwrap();
    let toon = hedl_toon::hedl_to_toon(&doc).unwrap();
    println!("{}", toon);
    println!();
    
    // Test 3: Compare with JSON
    println!("Test 3: JSON vs TOON comparison for products:");
    let products = hedl_bench::generate_products(3);
    let doc = hedl_core::parse(products.as_bytes()).unwrap();
    let json = hedl_json::to_json(&doc, &hedl_json::ToJsonConfig::default()).unwrap();
    let toon = hedl_toon::hedl_to_toon(&doc).unwrap();
    
    println!("JSON ({} bytes):", json.len());
    println!("{}", &json[..json.len().min(400)]);
    if json.len() > 400 { println!("..."); }
    println!();
    
    println!("TOON ({} bytes, {:.1}% of JSON):", toon.len(), toon.len() as f64 / json.len() as f64 * 100.0);
    println!("{}", toon);
    println!();
    
    // Test 4: Blog with nested comments
    println!("Test 4: Blog with nested comments structure:");
    let blog = hedl_bench::generate_blog(2, 2);
    let doc = hedl_core::parse(blog.as_bytes()).unwrap();
    let toon = hedl_toon::hedl_to_toon(&doc).unwrap();
    println!("{}", toon);
}

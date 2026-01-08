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

//! Demonstration of cache speedup for MCP operations.
//!
//! Run with: cargo run --example cache_speedup_demo --release

use hedl_mcp::cache::OperationCache;
use hedl_mcp::tools::{execute_hedl_query, execute_hedl_stats, execute_hedl_validate};
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;

const HEDL_SAMPLE: &str = r#"%VERSION: 1.0
%STRUCT: User: [id, name, email, age, city]
%STRUCT: Product: [id, title, price, category, stock]
%STRUCT: Order: [id, user_id, product_id, quantity, total]
---
users: @User
  | alice, Alice Smith, alice@example.com, 30, San Francisco
  | bob, Bob Jones, bob@example.com, 25, New York
  | charlie, Charlie Brown, charlie@example.com, 35, Los Angeles
  | diana, Diana Prince, diana@example.com, 28, Chicago
  | eve, Eve Anderson, eve@example.com, 32, Seattle
  | frank, Frank Miller, frank@example.com, 27, Boston
  | grace, Grace Hopper, grace@example.com, 31, Austin
  | henry, Henry Ford, henry@example.com, 29, Detroit
  | iris, Iris Chang, iris@example.com, 33, Portland
  | jack, Jack London, jack@example.com, 26, Sacramento

products: @Product
  | widget, Widget Pro, 99.99, electronics, 100
  | gadget, Gadget Plus, 149.99, electronics, 50
  | tool, Tool Master, 79.99, hardware, 75
  | device, Smart Device, 199.99, electronics, 25
  | accessory, Premium Accessory, 49.99, accessories, 200
  | component, Essential Component, 29.99, parts, 150
  | module, Power Module, 89.99, electronics, 60
  | sensor, Smart Sensor, 39.99, electronics, 120
  | controller, Main Controller, 129.99, electronics, 40
  | adapter, Universal Adapter, 19.99, accessories, 300

orders: @Order
  | order1, alice, widget, 2, 199.98
  | order2, bob, gadget, 1, 149.99
  | order3, charlie, tool, 3, 239.97
  | order4, diana, device, 1, 199.99
  | order5, eve, accessory, 5, 249.95
  | order6, frank, component, 10, 299.90
  | order7, grace, module, 2, 179.98
  | order8, henry, sensor, 5, 199.95
  | order9, iris, controller, 1, 129.99
  | order10, jack, adapter, 20, 399.80
"#;

fn main() {
    println!("=== MCP Operation Cache Speedup Demonstration ===\n");

    // Benchmark validation
    benchmark_validate();
    println!();

    // Benchmark query
    benchmark_query();
    println!();

    // Benchmark stats
    benchmark_stats();
    println!();

    // Mixed workload
    benchmark_mixed_workload();
}

fn benchmark_validate() {
    println!("--- VALIDATE Operation ---");

    let args = json!({ "hedl": HEDL_SAMPLE, "strict": true, "lint": true });
    let iterations = 100;

    // Uncached performance
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = execute_hedl_validate(Some(args.clone())).unwrap();
    }
    let uncached_duration = start.elapsed();
    let uncached_per_op = uncached_duration.as_micros() / iterations as u128;

    println!("  Uncached: {} ops in {:?} ({} µs/op)", iterations, uncached_duration, uncached_per_op);

    // Cached performance
    let cache = Arc::new(OperationCache::new(1000));
    let cache_key = format!("{}:{}:{}", HEDL_SAMPLE, true, true);

    // Warm up cache
    let result = execute_hedl_validate(Some(args.clone())).unwrap();
    let result_json = serde_json::to_value(&result).unwrap();
    cache.insert("validate", &cache_key, result_json);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = cache.get("validate", &cache_key);
    }
    let cached_duration = start.elapsed();
    let cached_per_op = cached_duration.as_micros() / iterations as u128;

    println!("  Cached:   {} ops in {:?} ({} µs/op)", iterations, cached_duration, cached_per_op);

    let speedup = uncached_per_op as f64 / cached_per_op.max(1) as f64;
    println!("  Speedup:  {:.1}x faster", speedup);

    let stats = cache.stats();
    println!("  Hit rate: {:.1}%", stats.hit_rate_percent());
}

fn benchmark_query() {
    println!("--- QUERY Operation ---");

    let args = json!({ "hedl": HEDL_SAMPLE, "type_name": "User" });
    let iterations = 100;

    // Uncached performance
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = execute_hedl_query(Some(args.clone())).unwrap();
    }
    let uncached_duration = start.elapsed();
    let uncached_per_op = uncached_duration.as_micros() / iterations as u128;

    println!("  Uncached: {} ops in {:?} ({} µs/op)", iterations, uncached_duration, uncached_per_op);

    // Cached performance
    let cache = Arc::new(OperationCache::new(1000));
    let cache_key = format!("{}:User::true", HEDL_SAMPLE);

    // Warm up cache
    let result = execute_hedl_query(Some(args.clone())).unwrap();
    let result_json = serde_json::to_value(&result).unwrap();
    cache.insert("query", &cache_key, result_json);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = cache.get("query", &cache_key);
    }
    let cached_duration = start.elapsed();
    let cached_per_op = cached_duration.as_micros() / iterations as u128;

    println!("  Cached:   {} ops in {:?} ({} µs/op)", iterations, cached_duration, cached_per_op);

    let speedup = uncached_per_op as f64 / cached_per_op.max(1) as f64;
    println!("  Speedup:  {:.1}x faster", speedup);

    let stats = cache.stats();
    println!("  Hit rate: {:.1}%", stats.hit_rate_percent());
}

fn benchmark_stats() {
    println!("--- STATS Operation ---");

    let args = json!({ "hedl": HEDL_SAMPLE, "tokenizer": "simple" });
    let iterations = 100;

    // Uncached performance
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = execute_hedl_stats(Some(args.clone())).unwrap();
    }
    let uncached_duration = start.elapsed();
    let uncached_per_op = uncached_duration.as_micros() / iterations as u128;

    println!("  Uncached: {} ops in {:?} ({} µs/op)", iterations, uncached_duration, uncached_per_op);

    // Cached performance
    let cache = Arc::new(OperationCache::new(1000));
    let cache_key = format!("{}:simple", HEDL_SAMPLE);

    // Warm up cache
    let result = execute_hedl_stats(Some(args.clone())).unwrap();
    let result_json = serde_json::to_value(&result).unwrap();
    cache.insert("stats", &cache_key, result_json);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = cache.get("stats", &cache_key);
    }
    let cached_duration = start.elapsed();
    let cached_per_op = cached_duration.as_micros() / iterations as u128;

    println!("  Cached:   {} ops in {:?} ({} µs/op)", iterations, cached_duration, cached_per_op);

    let speedup = uncached_per_op as f64 / cached_per_op.max(1) as f64;
    println!("  Speedup:  {:.1}x faster", speedup);

    let stats = cache.stats();
    println!("  Hit rate: {:.1}%", stats.hit_rate_percent());
}

fn benchmark_mixed_workload() {
    println!("--- MIXED Workload (80% cache hits) ---");

    let cache = Arc::new(OperationCache::new(1000));
    let iterations = 500;

    // Prepare 5 different documents (we'll hit 4 frequently, 1 rarely)
    let docs: Vec<String> = (0..5)
        .map(|i| {
            format!(
                "%VERSION: 1.0\n%STRUCT: Item: [id, value]\n---\nitems: @Item\n  | item{}, {}\n",
                i, i * 100
            )
        })
        .collect();

    // Pre-populate cache with 4 documents (80%)
    for (_i, doc) in docs.iter().enumerate().take(4) {
        let args = json!({ "hedl": doc, "strict": true, "lint": true });
        let result = execute_hedl_validate(Some(args)).unwrap();
        let result_json = serde_json::to_value(&result).unwrap();
        let cache_key = format!("{}:{}:{}", doc, true, true);
        cache.insert("validate", &cache_key, result_json);
    }

    // Mixed workload: 80% hits, 20% misses
    let start = Instant::now();
    for i in 0..iterations {
        // 80% of the time, access cached documents (0-3)
        // 20% of the time, access uncached document (4)
        let doc_idx = if i % 5 < 4 { i % 4 } else { 4 };
        let doc = &docs[doc_idx];
        let cache_key = format!("{}:{}:{}", doc, true, true);

        if cache.get("validate", &cache_key).is_none() {
            // Cache miss: execute and cache
            let args = json!({ "hedl": doc, "strict": true, "lint": true });
            let result = execute_hedl_validate(Some(args)).unwrap();
            let result_json = serde_json::to_value(&result).unwrap();
            cache.insert("validate", &cache_key, result_json);
        }
    }
    let mixed_duration = start.elapsed();
    let mixed_per_op = mixed_duration.as_micros() / iterations as u128;

    println!("  Mixed:    {} ops in {:?} ({} µs/op)", iterations, mixed_duration, mixed_per_op);

    // Compare to uncached baseline
    let start = Instant::now();
    for i in 0..iterations {
        let doc_idx = if i % 5 < 4 { i % 4 } else { 4 };
        let doc = &docs[doc_idx];
        let args = json!({ "hedl": doc, "strict": true, "lint": true });
        let _ = execute_hedl_validate(Some(args)).unwrap();
    }
    let uncached_duration = start.elapsed();
    let uncached_per_op = uncached_duration.as_micros() / iterations as u128;

    println!("  Uncached: {} ops in {:?} ({} µs/op)", iterations, uncached_duration, uncached_per_op);

    let speedup = uncached_per_op as f64 / mixed_per_op as f64;
    println!("  Speedup:  {:.1}x faster", speedup);

    let stats = cache.stats();
    println!("  Cache stats:");
    println!("    Hits:      {}", stats.hits);
    println!("    Misses:    {}", stats.misses);
    println!("    Hit rate:  {:.1}%", stats.hit_rate_percent());
    println!("    Size:      {}/{}", stats.size, stats.max_size);
}

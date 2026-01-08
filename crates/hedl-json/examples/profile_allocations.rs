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

//! Allocation profiling for hedl-json
//!
//! This example profiles allocation patterns during JSON conversion
//! to identify optimization opportunities.

use hedl_json::{from_json, to_json, FromJsonConfig, ToJsonConfig};
use serde_json::json;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Custom allocator that tracks allocation statistics
struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
            ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        DEALLOCATIONS.fetch_add(1, Ordering::SeqCst);
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

fn reset_stats() {
    ALLOCATED.store(0, Ordering::SeqCst);
    ALLOCATIONS.store(0, Ordering::SeqCst);
    DEALLOCATIONS.store(0, Ordering::SeqCst);
}

fn get_stats() -> (usize, usize, usize) {
    (
        ALLOCATED.load(Ordering::SeqCst),
        ALLOCATIONS.load(Ordering::SeqCst),
        DEALLOCATIONS.load(Ordering::SeqCst),
    )
}

fn main() {
    println!("=== HEDL-JSON Allocation Profiling ===\n");

    // Test 1: Small object array
    println!("Test 1: Small object array (10 items)");
    test_allocation("small", 10);

    // Test 2: Medium object array
    println!("\nTest 2: Medium object array (100 items)");
    test_allocation("medium", 100);

    // Test 3: Large object array
    println!("\nTest 3: Large object array (1000 items)");
    test_allocation("large", 1000);

    // Test 4: Deep nesting
    println!("\nTest 4: Deep nesting (depth=10)");
    test_deep_nesting(10);

    // Test 5: Wide objects
    println!("\nTest 5: Wide objects (50 fields)");
    test_wide_objects(50);
}

fn test_allocation(name: &str, count: usize) {
    // Generate test JSON
    let json = generate_users_json(count);
    let json_size = json.len();

    // Test from_json
    reset_stats();
    let config = FromJsonConfig::default();
    let doc = from_json(&json, &config).expect("Failed to parse JSON");
    let (allocated, allocs, deallocs) = get_stats();

    println!("  from_json:");
    println!("    Input size:      {:>10} bytes", json_size);
    println!("    Total allocated: {:>10} bytes ({:.2}x input)", allocated, allocated as f64 / json_size as f64);
    println!("    Allocations:     {:>10}", allocs);
    println!("    Deallocations:   {:>10}", deallocs);
    println!("    Allocs/item:     {:>10.2}", allocs as f64 / count as f64);

    // Test to_json
    reset_stats();
    let to_config = ToJsonConfig::default();
    let output = to_json(&doc, &to_config).expect("Failed to serialize JSON");
    let (allocated, allocs, deallocs) = get_stats();
    let output_size = output.len();

    println!("  to_json:");
    println!("    Output size:     {:>10} bytes", output_size);
    println!("    Total allocated: {:>10} bytes ({:.2}x output)", allocated, allocated as f64 / output_size as f64);
    println!("    Allocations:     {:>10}", allocs);
    println!("    Deallocations:   {:>10}", deallocs);
    println!("    Allocs/item:     {:>10.2}", allocs as f64 / count as f64);
}

fn test_deep_nesting(depth: usize) {
    let json = generate_nested_json(depth);
    let json_size = json.len();

    reset_stats();
    let config = FromJsonConfig::default();
    let _doc = from_json(&json, &config).expect("Failed to parse JSON");
    let (allocated, allocs, deallocs) = get_stats();

    println!("  Depth {}:", depth);
    println!("    Input size:      {:>10} bytes", json_size);
    println!("    Total allocated: {:>10} bytes ({:.2}x input)", allocated, allocated as f64 / json_size as f64);
    println!("    Allocations:     {:>10}", allocs);
    println!("    Allocs/level:    {:>10.2}", allocs as f64 / depth as f64);
}

fn test_wide_objects(field_count: usize) {
    let json = generate_wide_object_json(field_count);
    let json_size = json.len();

    reset_stats();
    let config = FromJsonConfig::default();
    let _doc = from_json(&json, &config).expect("Failed to parse JSON");
    let (allocated, allocs, deallocs) = get_stats();

    println!("  {} fields:", field_count);
    println!("    Input size:      {:>10} bytes", json_size);
    println!("    Total allocated: {:>10} bytes ({:.2}x input)", allocated, allocated as f64 / json_size as f64);
    println!("    Allocations:     {:>10}", allocs);
    println!("    Allocs/field:    {:>10.2}", allocs as f64 / field_count as f64);
}

fn generate_users_json(count: usize) -> String {
    let mut users = Vec::with_capacity(count);
    for i in 0..count {
        users.push(json!({
            "id": format!("user_{}", i),
            "name": format!("User {}", i),
            "email": format!("user{}@example.com", i),
            "age": 20 + (i % 50),
            "active": i % 2 == 0,
        }));
    }
    serde_json::to_string(&json!({
        "users": users
    })).unwrap()
}

fn generate_nested_json(depth: usize) -> String {
    let mut current = json!({"value": 42});
    for i in (0..depth).rev() {
        current = json!({
            "level": i,
            "child": current
        });
    }
    serde_json::to_string(&current).unwrap()
}

fn generate_wide_object_json(field_count: usize) -> String {
    let mut obj = serde_json::Map::new();
    for i in 0..field_count {
        obj.insert(format!("field_{}", i), json!(format!("value_{}", i)));
    }
    serde_json::to_string(&json!({
        "data": obj
    })).unwrap()
}

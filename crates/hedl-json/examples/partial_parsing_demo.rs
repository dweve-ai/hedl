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

//! Example demonstrating partial parsing with error recovery
//!
//! This example shows how to use partial parsing to continue parsing
//! despite errors and collect all errors encountered.

use hedl_json::{partial_parse_json, ErrorTolerance, FromJsonConfig, PartialConfig};

fn main() {
    println!("=== Partial Parsing Demo ===\n");

    // Example 1: Collect all errors
    demo_collect_all_errors();

    // Example 2: Skip invalid items
    demo_skip_invalid_items();

    // Example 3: Max errors limit
    demo_max_errors_limit();

    // Example 4: Replace invalid with null
    demo_replace_with_null();

    // Example 5: Data migration scenario
    demo_data_migration();
}

fn demo_collect_all_errors() {
    println!("--- Example 1: Collect All Errors ---");

    let json = r#"{
        "valid_field": "good data",
        "too_long": "this string will exceed the 20 char limit we set",
        "another_valid": 42,
        "also_too_long": "this one is also way too long for our limit"
    }"#;

    let config = PartialConfig::builder()
        .from_json_config(
            FromJsonConfig::builder()
                .max_string_length(20)
                .build(),
        )
        .tolerance(ErrorTolerance::CollectAll)
        .build();

    let result = partial_parse_json(json, &config);

    println!("Parsing completed: {}", result.is_complete());
    println!("Errors encountered: {}", result.errors.len());

    for (i, error) in result.errors.iter().enumerate() {
        println!(
            "  Error {}: {} at {}",
            i + 1,
            error.error,
            error.location.path
        );
    }

    if let Some(doc) = result.document {
        println!("Partial document created with {} root fields", doc.root.len());
    }

    println!();
}

fn demo_skip_invalid_items() {
    println!("--- Example 2: Skip Invalid Items ---");

    let json = r#"{
        "users": [
            {"id": "1", "name": "Alice"},
            "invalid entry",
            {"id": "2", "name": "Bob"},
            123,
            {"id": "3", "name": "Carol"}
        ]
    }"#;

    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::SkipInvalidItems)
        .build();

    let result = partial_parse_json(json, &config);

    println!("Parsing completed: {}", result.is_complete());
    println!("Errors encountered: {}", result.errors.len());

    if let Some(_doc) = result.document {
        println!("Successfully parsed document");
        // In a real application, you would process the valid users here
    }

    println!();
}

fn demo_max_errors_limit() {
    println!("--- Example 3: Max Errors Limit ---");

    let json = r#"{
        "field1": "way too long string 1",
        "field2": "way too long string 2",
        "field3": "way too long string 3",
        "field4": "way too long string 4",
        "field5": "way too long string 5"
    }"#;

    let config = PartialConfig::builder()
        .from_json_config(FromJsonConfig::builder().max_string_length(10).build())
        .tolerance(ErrorTolerance::MaxErrors(2))
        .build();

    let result = partial_parse_json(json, &config);

    println!("Parsing completed: {}", result.is_complete());
    println!("Stopped early: {}", result.stopped_early);
    println!("Errors collected: {}", result.errors.len());
    println!("Max errors reached after encountering {} errors", result.errors.len());

    println!();
}

fn demo_replace_with_null() {
    println!("--- Example 4: Replace Invalid with Null ---");

    let json = r#"{
        "name": "Alice",
        "age": 30,
        "bio": "This is a very long biography that exceeds our character limit"
    }"#;

    let config = PartialConfig::builder()
        .from_json_config(FromJsonConfig::builder().max_string_length(20).build())
        .tolerance(ErrorTolerance::CollectAll)
        .replace_invalid_with_null(true)
        .build();

    let result = partial_parse_json(json, &config);

    println!("Parsing completed: {}", result.is_complete());
    println!("Errors encountered: {}", result.errors.len());

    for error in &result.errors {
        println!(
            "  Replaced invalid value at {} with null: {}",
            error.location.path, error.error
        );
    }

    if let Some(doc) = result.document {
        println!("Document created with {} fields (bio replaced with null)", doc.root.len());
    }

    println!();
}

fn demo_data_migration() {
    println!("--- Example 5: Data Migration Scenario ---");

    let legacy_json = r#"{
        "users": [
            {"id": "1", "name": "Alice", "email": "alice@example.com"},
            {"id": "2", "name": "Bob", "email": "bob@example.com"},
            {"id": "3", "name": "Carol", "email": "carol@example.com"}
        ],
        "settings": {
            "theme": "dark",
            "language": "en"
        }
    }"#;

    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .replace_invalid_with_null(true)
        .include_partial_on_fatal(true)
        .build();

    let result = partial_parse_json(legacy_json, &config);

    println!("Migration completed: {}", result.is_complete());
    println!("Issues encountered: {}", result.errors.len());

    if result.errors.is_empty() {
        println!("  No migration issues!");
    } else {
        for error in &result.errors {
            println!("  Issue at {}: {}", error.location.path, error.error);
        }
    }

    match result.document {
        Some(doc) => {
            println!("Successfully migrated {} root fields", doc.root.len());
            // In a real application, you would save the document to the database
        }
        None => {
            println!("Migration failed - too many errors");
        }
    }

    println!();
}

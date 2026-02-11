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

//! Advanced Features Example
//!
//! Demonstrates advanced HEDL features:
//! - References between entities
//! - Tensor literals
//! - Nested structures
//! - Aliases
//!
//! Run with: cargo run --example `advanced_features`

use hedl::{parse, to_json};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HEDL Advanced Features Example ===\n");

    let hedl_text = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email,role]
%S:Post:[id,author_id,title,tags]
%S:Comment:[id,post_id,author_id,text]
%S:Meeting:[id,time,title,room]
%A:%conf_a:"conference-room-a"
---
# User database
users:@User
 |alice,Alice Smith,alice@example.com,admin
 |bob,Bob Jones,bob@example.com,user
 |charlie,Charlie Brown,charlie@example.com,moderator

# Blog posts with references to authors
posts:@Post
 |post1,@User:alice,"Getting Started with HEDL",[tutorial,beginner]
 |post2,@User:bob,"Advanced HEDL Patterns",[advanced,patterns]
 |post3,@User:alice,"HEDL vs JSON",[comparison,analysis]

# Comments with references to posts and authors
comments:@Comment
 |c1,@Post:post1,@User:bob,"Great introduction!"
 |c2,@Post:post1,@User:charlie,"Very helpful, thanks!"
 |c3,@Post:post2,@User:alice,"Excellent deep dive"

# Meeting schedules (using alias for repeated room)
monday_schedule:@Meeting
 |meeting1, 09:00 AM, Team standup, %conf_a
 |meeting2, 10:00 AM, Code review, %conf_a
 |meeting3, 02:00 PM, Sprint planning, %conf_a

tuesday_schedule:@Meeting
 |meeting4, 09:00 AM, Design meeting, %conf_a
 |meeting5, 11:00 AM, One-on-one, office-2

# Tensor literals
ml_config:
 embedding_dims: [768, 512, 256]
 learning_rates: [0.001, 0.0001, 0.00001]
 batch_sizes: [32, 64, 128]
"#;

    println!("Input HEDL document with advanced features:");
    println!("{hedl_text}");
    println!();

    // Parse the document
    let doc = parse(hedl_text)?;
    println!("--- Parsing Results ---");
    println!(
        "✓ Parsed {} structs: {:?}",
        doc.structs.len(),
        doc.structs.keys().collect::<Vec<_>>()
    );
    println!("✓ Parsed {} root items", doc.root.len());
    println!();

    // Convert to JSON to see the resolved structure
    let json = to_json(&doc)?;
    println!("--- JSON Output ---");
    println!("{json}");
    println!();

    println!("✓ All advanced features working correctly!");

    Ok(())
}

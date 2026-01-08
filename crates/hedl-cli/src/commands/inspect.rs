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

//! Inspect command - HEDL structure visualization

use super::read_file;
use colored::Colorize;
use hedl_core::{parse, Item, Value};
use std::collections::BTreeMap;

/// Inspect and visualize the internal structure of a HEDL file.
///
/// Parses a HEDL file and displays its internal structure in a human-readable,
/// tree-like format with color highlighting. Useful for debugging and understanding
/// HEDL document organization.
///
/// # Arguments
///
/// * `file` - Path to the HEDL file to inspect
/// * `verbose` - If `true`, shows detailed field values and schema information
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns `Err` if:
/// - The file cannot be read
/// - The file contains syntax errors
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::inspect;
///
/// # fn main() -> Result<(), String> {
/// // Inspect with basic output
/// inspect("example.hedl", false)?;
///
/// // Inspect with verbose details (shows all fields and schemas)
/// inspect("example.hedl", true)?;
/// # Ok(())
/// # }
/// ```
///
/// # Output
///
/// Displays:
/// - HEDL version
/// - Struct definitions (type names and columns)
/// - Alias definitions
/// - Nest relationships (parent > child)
/// - Root document structure (tree view)
/// - In verbose mode: field values, schemas, and row details
pub fn inspect(file: &str, verbose: bool) -> Result<(), String> {
    let content = read_file(file)?;

    let doc = parse(content.as_bytes()).map_err(|e| format!("Parse error: {}", e))?;

    println!("{}", "HEDL Document".bold().underline());
    println!();
    println!("{}  {}.{}", "Version:".cyan(), doc.version.0, doc.version.1);

    if !doc.structs.is_empty() {
        println!();
        println!("{}", "Structs:".cyan());
        for (name, cols) in &doc.structs {
            println!("  {}: [{}]", name.green(), cols.join(", "));
        }
    }

    if !doc.aliases.is_empty() {
        println!();
        println!("{}", "Aliases:".cyan());
        for (name, value) in &doc.aliases {
            println!("  %{}: \"{}\"", name.green(), value);
        }
    }

    if !doc.nests.is_empty() {
        println!();
        println!("{}", "Nests:".cyan());
        for (parent, child) in &doc.nests {
            println!("  {} > {}", parent.green(), child);
        }
    }

    println!();
    println!("{}", "Root:".cyan());
    print_items(&doc.root, 1, verbose);

    Ok(())
}

fn print_items(items: &BTreeMap<String, Item>, indent: usize, verbose: bool) {
    let prefix = "  ".repeat(indent);

    for (key, item) in items {
        match item {
            Item::Scalar(value) => {
                println!("{}{}: {}", prefix, key.yellow(), format_value(value));
            }
            Item::Object(child) => {
                println!("{}{}:", prefix, key.yellow());
                print_items(child, indent + 1, verbose);
            }
            Item::List(list) => {
                println!(
                    "{}{}: @{} ({} rows)",
                    prefix,
                    key.yellow(),
                    list.type_name.green(),
                    list.rows.len()
                );
                if verbose {
                    println!("{}  schema: [{}]", prefix, list.schema.join(", "));
                    for (i, row) in list.rows.iter().enumerate() {
                        println!("{}  [{}] id={}", prefix, i, row.id);
                        for (field_idx, col) in list.schema.iter().enumerate() {
                            if let Some(v) = row.fields.get(field_idx) {
                                println!("{}    {}: {}", prefix, col, format_value(v));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "~".dimmed().to_string(),
        Value::Bool(b) => b.to_string().magenta().to_string(),
        Value::Int(n) => n.to_string().cyan().to_string(),
        Value::Float(f) => f.to_string().cyan().to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::Tensor(t) => format!("{:?}", t).cyan().to_string(),
        Value::Reference(r) => r.to_ref_string().green().to_string(),
        Value::Expression(e) => format!("$({})", e).yellow().to_string(),
    }
}

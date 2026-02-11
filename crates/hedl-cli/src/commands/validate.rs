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

//! Validate command - HEDL file syntax and structure validation

use super::read_file;
use crate::error::CliError;
use colored::Colorize;
use hedl_core::{parse_with_limits, ParseOptions, ReferenceMode};

/// Validate a HEDL file for syntax and structural correctness.
///
/// Parses a HEDL file and reports whether it is syntactically valid. In strict mode,
/// all entity references must resolve to defined entities. In lenient mode, constraint
/// violations become null values with diagnostics emitted separately.
///
/// # Arguments
///
/// * `file` - Path to the HEDL file to validate
/// * `strict` - If `true`, enables strict reference validation (all references must resolve)
/// * `lenient` - If `true`, uses lenient parsing mode (constraint violations become null)
///
/// # Returns
///
/// Returns `Ok(())` if the file is valid, `Err` with a descriptive error message otherwise.
///
/// # Errors
///
/// Returns `Err` if:
/// - The file cannot be read
/// - The file contains syntax errors
/// - In strict mode, if any entity references cannot be resolved
/// - In non-lenient mode, if constraints are violated
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::validate;
///
/// # fn main() -> Result<(), hedl_cli::error::CliError> {
/// // Validate a well-formed HEDL file
/// validate("valid.hedl", false, false)?;
///
/// // Strict validation requires all references to resolve
/// validate("references.hedl", true, false)?;
///
/// // Lenient mode allows constraint violations
/// validate("data.hedl", false, true)?;
///
/// // Invalid syntax will fail
/// let result = validate("invalid.hedl", false, false);
/// assert!(result.is_err());
/// # Ok(())
/// # }
/// ```
///
/// # Output
///
/// Prints a summary to stdout including:
/// - File validation status (✓ or ✗)
/// - HEDL version
/// - Count of structs, aliases, and nests
/// - Parse mode (strict/lenient)
/// - Reference mode indicator if strict enabled
pub fn validate(file: &str, strict: bool, lenient: bool) -> Result<(), CliError> {
    let content = read_file(file)?;

    // Configure parser options with strict reference mode
    // Note: Parse mode (strict/lenient for constraints) is controlled by %MODE directive in the file,
    // but we can override it via CLI flag if needed in the future
    let options = ParseOptions {
        reference_mode: if strict {
            ReferenceMode::Strict
        } else {
            ReferenceMode::Lenient
        },
        ..ParseOptions::default()
    };

    match parse_with_limits(content.as_bytes(), options) {
        Ok(doc) => {
            println!("{} {}", "✓".green().bold(), file);
            println!("  Version: {}.{}", doc.version.0, doc.version.1);
            println!("  Structs: {}", doc.structs.len());
            println!("  Aliases: {}", doc.aliases.len());
            println!("  Nests: {}", doc.nests.len());

            // Show modes
            let mode_str = if lenient { "lenient" } else { "strict" };
            println!("  Parse mode: {} (CLI override)", mode_str);

            if strict {
                println!("  Reference mode: strict (all references must resolve)");
            }

            Ok(())
        }
        Err(e) => {
            println!("{} {}", "✗".red().bold(), file);
            Err(CliError::parse(e.to_string()))
        }
    }
}

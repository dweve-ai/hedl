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

//! Lint command - HEDL best practices and style checking

use super::read_file;
use colored::Colorize;
use hedl_core::parse;
use hedl_lint::{lint_with_config, LintConfig, Severity};

/// Lint a HEDL file for best practices and style issues.
///
/// Analyzes a HEDL file for potential issues, style violations, and best practice
/// deviations. Reports diagnostics with severity levels (error, warning, hint).
///
/// # Arguments
///
/// * `file` - Path to the HEDL file to lint
/// * `format` - Output format: "text" (default, colored) or "json" (machine-readable)
/// * `warn_error` - If `true`, treat warnings as errors (fail on any warning)
///
/// # Returns
///
/// Returns `Ok(())` if no issues are found (or only hints), `Err` if errors or warnings
/// (with `warn_error` enabled) are detected.
///
/// # Errors
///
/// Returns `Err` if:
/// - The file cannot be read
/// - The file contains syntax errors
/// - Lint errors are found
/// - Warnings are found and `warn_error` is `true`
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::lint;
///
/// # fn main() -> Result<(), String> {
/// // Lint with text output
/// lint("example.hedl", "text", false)?;
///
/// // Lint with JSON output for CI/CD integration
/// lint("example.hedl", "json", false)?;
///
/// // Treat warnings as errors (strict mode)
/// let result = lint("example.hedl", "text", true);
/// if result.is_err() {
///     eprintln!("Warnings or errors found!");
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Output
///
/// **Text format**: Colored, human-readable output with:
/// - File name and issue count
/// - Each diagnostic with severity, rule ID, message, and line number
/// - Suggestions for fixing issues
///
/// **JSON format**: Structured JSON with:
/// - File path
/// - Array of diagnostics with severity, rule, message, line, and suggestion
pub fn lint(file: &str, format: &str, warn_error: bool) -> Result<(), String> {
    let content = read_file(file)?;

    let doc = parse(content.as_bytes()).map_err(|e| format!("Parse error: {}", e))?;

    let config = LintConfig::default();
    let diagnostics = lint_with_config(&doc, config);

    match format {
        "json" => {
            let json = serde_json::json!({
                "file": file,
                "diagnostics": diagnostics.iter().map(|d| {
                    serde_json::json!({
                        "severity": format!("{:?}", d.severity()),
                        "rule": d.rule_id(),
                        "message": d.message(),
                        "line": d.line(),
                        "suggestion": d.suggestion()
                    })
                }).collect::<Vec<_>>()
            });
            let output = serde_json::to_string_pretty(&json)
                .map_err(|e| format!("JSON serialization error: {}", e))?;
            println!("{}", output);
        }
        _ => {
            if diagnostics.is_empty() {
                println!("{} {} - no issues found", "✓".green().bold(), file);
            } else {
                println!(
                    "{} {} - {} issue(s) found:",
                    "!".yellow().bold(),
                    file,
                    diagnostics.len()
                );
                for diag in &diagnostics {
                    let severity_str = match diag.severity() {
                        Severity::Error => "error".red(),
                        Severity::Warning => "warning".yellow(),
                        Severity::Hint => "hint".blue(),
                    };

                    if let Some(line) = diag.line() {
                        println!("  {}:{}: {}: {}", file, line, severity_str, diag.message());
                    } else {
                        println!("  {}: {}: {}", file, severity_str, diag.message());
                    }

                    if let Some(ref suggestion) = diag.suggestion() {
                        println!("    {} {}", "suggestion:".cyan(), suggestion);
                    }
                }
            }
        }
    }

    let has_errors = diagnostics.iter().any(|d| d.severity() == Severity::Error);
    let has_warnings = diagnostics.iter().any(|d| d.severity() == Severity::Warning);

    if has_errors || (warn_error && has_warnings) {
        Err("Lint errors found".to_string())
    } else {
        Ok(())
    }
}

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

//! Conversion commands - HEDL format interoperability
//!
//! This module provides bidirectional conversion between HEDL and popular data formats:
//! - JSON (compact and pretty-printed)
//! - YAML
//! - XML (compact and pretty-printed)
//! - CSV
//! - Parquet
//!
//! All conversions preserve data fidelity where possible, with format-specific
//! optimizations and configurations.

use super::{read_file, write_output};
use hedl_c14n::canonicalize;
use hedl_core::parse;
use hedl_csv::{from_csv as csv_to_hedl, to_csv as hedl_to_csv};
use hedl_json::{from_json as json_to_hedl, to_json_value, FromJsonConfig, ToJsonConfig};
use hedl_parquet::{from_parquet as parquet_to_hedl, to_parquet as hedl_to_parquet};
use hedl_xml::{from_xml as xml_to_hedl, to_xml as hedl_to_xml, FromXmlConfig, ToXmlConfig};
use hedl_yaml::{from_yaml as yaml_to_hedl, to_yaml as hedl_to_yaml, FromYamlConfig, ToYamlConfig};
use hedl_toon::{hedl_to_toon, toon_to_hedl};
use std::path::Path;

// JSON conversion

/// Convert a HEDL file to JSON format.
///
/// Parses a HEDL file and converts it to JSON, with options for metadata inclusion
/// and pretty-printing.
///
/// # Arguments
///
/// * `file` - Path to the HEDL file to convert
/// * `output` - Optional output file path. If `None`, writes to stdout
/// * `metadata` - If `true`, includes HEDL-specific metadata in the JSON output
/// * `pretty` - If `true`, pretty-prints the JSON with indentation
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
/// - JSON conversion fails
/// - Output writing fails
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::to_json;
///
/// # fn main() -> Result<(), String> {
/// // Convert to compact JSON on stdout
/// to_json("data.hedl", None, false, false)?;
///
/// // Convert to pretty JSON with metadata
/// to_json("data.hedl", Some("output.json"), true, true)?;
/// # Ok(())
/// # }
/// ```
pub fn to_json(
    file: &str,
    output: Option<&str>,
    metadata: bool,
    pretty: bool,
) -> Result<(), String> {
    let content = read_file(file)?;

    let doc = parse(content.as_bytes()).map_err(|e| format!("Parse error: {}", e))?;

    let config = ToJsonConfig {
        include_metadata: metadata,
        ..Default::default()
    };

    // P0 OPTIMIZATION: Direct conversion to Value (no double conversion)
    let value = to_json_value(&doc, &config).map_err(|e| format!("JSON conversion error: {}", e))?;
    let output_str = if pretty {
        serde_json::to_string_pretty(&value).map_err(|e| format!("JSON format error: {}", e))?
    } else {
        serde_json::to_string(&value).map_err(|e| format!("JSON format error: {}", e))?
    };

    write_output(&output_str, output)
}

/// Convert a JSON file to HEDL format.
///
/// Parses a JSON file and converts it to canonical HEDL format.
///
/// # Arguments
///
/// * `file` - Path to the JSON file to convert
/// * `output` - Optional output file path. If `None`, writes to stdout
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns `Err` if:
/// - The file cannot be read
/// - The JSON is malformed
/// - JSON-to-HEDL conversion fails
/// - HEDL canonicalization fails
/// - Output writing fails
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::from_json;
///
/// # fn main() -> Result<(), String> {
/// // Convert JSON to HEDL on stdout
/// from_json("data.json", None)?;
///
/// // Convert JSON to HEDL file
/// from_json("data.json", Some("output.hedl"))?;
/// # Ok(())
/// # }
/// ```
pub fn from_json(file: &str, output: Option<&str>) -> Result<(), String> {
    let content = read_file(file)?;

    let config = FromJsonConfig::default();
    let doc =
        json_to_hedl(&content, &config).map_err(|e| format!("JSON conversion error: {}", e))?;

    let hedl = canonicalize(&doc).map_err(|e| format!("HEDL generation error: {}", e))?;

    write_output(&hedl, output)
}

// YAML conversion

/// Convert a HEDL file to YAML format.
///
/// Parses a HEDL file and converts it to YAML format.
///
/// # Arguments
///
/// * `file` - Path to the HEDL file to convert
/// * `output` - Optional output file path. If `None`, writes to stdout
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
/// - YAML conversion fails
/// - Output writing fails
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::to_yaml;
///
/// # fn main() -> Result<(), String> {
/// // Convert to YAML on stdout
/// to_yaml("data.hedl", None)?;
///
/// // Convert to YAML file
/// to_yaml("data.hedl", Some("output.yaml"))?;
/// # Ok(())
/// # }
/// ```
pub fn to_yaml(file: &str, output: Option<&str>) -> Result<(), String> {
    let content = read_file(file)?;

    let doc = parse(content.as_bytes()).map_err(|e| format!("Parse error: {}", e))?;

    let config = ToYamlConfig::default();
    let yaml = hedl_to_yaml(&doc, &config).map_err(|e| format!("YAML conversion error: {}", e))?;

    write_output(&yaml, output)
}

/// Convert a YAML file to HEDL format.
///
/// Parses a YAML file and converts it to canonical HEDL format.
///
/// # Arguments
///
/// * `file` - Path to the YAML file to convert
/// * `output` - Optional output file path. If `None`, writes to stdout
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns `Err` if:
/// - The file cannot be read
/// - The YAML is malformed
/// - YAML-to-HEDL conversion fails
/// - HEDL canonicalization fails
/// - Output writing fails
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::from_yaml;
///
/// # fn main() -> Result<(), String> {
/// // Convert YAML to HEDL on stdout
/// from_yaml("data.yaml", None)?;
///
/// // Convert YAML to HEDL file
/// from_yaml("data.yml", Some("output.hedl"))?;
/// # Ok(())
/// # }
/// ```
pub fn from_yaml(file: &str, output: Option<&str>) -> Result<(), String> {
    let content = read_file(file)?;

    let config = FromYamlConfig::default();
    let doc =
        yaml_to_hedl(&content, &config).map_err(|e| format!("YAML conversion error: {}", e))?;

    let hedl = canonicalize(&doc).map_err(|e| format!("HEDL generation error: {}", e))?;

    write_output(&hedl, output)
}

// XML conversion

/// Convert a HEDL file to XML format.
///
/// Parses a HEDL file and converts it to XML, with optional pretty-printing.
///
/// # Arguments
///
/// * `file` - Path to the HEDL file to convert
/// * `output` - Optional output file path. If `None`, writes to stdout
/// * `pretty` - If `true`, pretty-prints the XML with indentation
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
/// - XML conversion fails
/// - Output writing fails
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::to_xml;
///
/// # fn main() -> Result<(), String> {
/// // Convert to compact XML on stdout
/// to_xml("data.hedl", None, false)?;
///
/// // Convert to pretty XML file
/// to_xml("data.hedl", Some("output.xml"), true)?;
/// # Ok(())
/// # }
/// ```
pub fn to_xml(file: &str, output: Option<&str>, pretty: bool) -> Result<(), String> {
    let content = read_file(file)?;

    let doc = parse(content.as_bytes()).map_err(|e| format!("Parse error: {}", e))?;

    let config = ToXmlConfig {
        pretty,
        ..Default::default()
    };
    let xml = hedl_to_xml(&doc, &config).map_err(|e| format!("XML conversion error: {}", e))?;

    write_output(&xml, output)
}

/// Convert an XML file to HEDL format.
///
/// Parses an XML file and converts it to canonical HEDL format.
///
/// # Arguments
///
/// * `file` - Path to the XML file to convert
/// * `output` - Optional output file path. If `None`, writes to stdout
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns `Err` if:
/// - The file cannot be read
/// - The XML is malformed
/// - XML-to-HEDL conversion fails
/// - HEDL canonicalization fails
/// - Output writing fails
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::from_xml;
///
/// # fn main() -> Result<(), String> {
/// // Convert XML to HEDL on stdout
/// from_xml("data.xml", None)?;
///
/// // Convert XML to HEDL file
/// from_xml("data.xml", Some("output.hedl"))?;
/// # Ok(())
/// # }
/// ```
pub fn from_xml(file: &str, output: Option<&str>) -> Result<(), String> {
    let content = read_file(file)?;

    let config = FromXmlConfig::default();
    let doc = xml_to_hedl(&content, &config).map_err(|e| format!("XML conversion error: {}", e))?;

    let hedl = canonicalize(&doc).map_err(|e| format!("HEDL generation error: {}", e))?;

    write_output(&hedl, output)
}

// CSV conversion

/// Convert a HEDL file to CSV format.
///
/// Parses a HEDL file and converts it to CSV format. Expects the HEDL file to contain
/// a matrix list that can be represented as a table.
///
/// # Arguments
///
/// * `file` - Path to the HEDL file to convert
/// * `output` - Optional output file path. If `None`, writes to stdout
/// * `_include_headers` - Reserved for future use (headers always included)
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
/// - The HEDL structure is not compatible with CSV (e.g., nested structures)
/// - CSV conversion fails
/// - Output writing fails
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::to_csv;
///
/// # fn main() -> Result<(), String> {
/// // Convert to CSV on stdout
/// to_csv("data.hedl", None, true)?;
///
/// // Convert to CSV file
/// to_csv("data.hedl", Some("output.csv"), true)?;
/// # Ok(())
/// # }
/// ```
pub fn to_csv(file: &str, output: Option<&str>, _include_headers: bool) -> Result<(), String> {
    let content = read_file(file)?;

    let doc = parse(content.as_bytes()).map_err(|e| format!("Parse error: {}", e))?;

    let csv = hedl_to_csv(&doc).map_err(|e| format!("CSV conversion error: {}", e))?;

    write_output(&csv, output)
}

/// Convert a CSV file to HEDL format.
///
/// Parses a CSV file and converts it to canonical HEDL format. The first row is assumed
/// to be the header row containing column names.
///
/// # Arguments
///
/// * `file` - Path to the CSV file to convert
/// * `output` - Optional output file path. If `None`, writes to stdout
/// * `type_name` - The type name to use for the HEDL matrix list (must be alphanumeric)
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns `Err` if:
/// - The file cannot be read
/// - The CSV is malformed or empty
/// - The type name is invalid (must be alphanumeric with underscores)
/// - CSV-to-HEDL conversion fails
/// - HEDL canonicalization fails
/// - Output writing fails
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::from_csv;
///
/// # fn main() -> Result<(), String> {
/// // Convert CSV to HEDL on stdout with type name "Person"
/// from_csv("people.csv", None, "Person")?;
///
/// // Convert CSV to HEDL file
/// from_csv("data.csv", Some("output.hedl"), "Record")?;
///
/// // Invalid type name will fail
/// let result = from_csv("data.csv", None, "Invalid-Name!");
/// assert!(result.is_err());
/// # Ok(())
/// # }
/// ```
///
/// # Security
///
/// The type name is validated to prevent injection attacks. Only alphanumeric
/// characters and underscores are allowed.
pub fn from_csv(file: &str, output: Option<&str>, type_name: &str) -> Result<(), String> {
    let content = read_file(file)?;

    // Validate type_name to prevent injection
    if !type_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Type name must be alphanumeric (with underscores allowed)".to_string());
    }

    // Infer column names from header row
    let first_line = content
        .lines()
        .next()
        .ok_or_else(|| "CSV file is empty or has no header row".to_string())?;
    let columns: Vec<&str> = first_line.split(',').skip(1).collect(); // Skip ID column

    let doc = csv_to_hedl(&content, type_name, &columns)
        .map_err(|e| format!("CSV conversion error: {}", e))?;

    let hedl = canonicalize(&doc).map_err(|e| format!("HEDL generation error: {}", e))?;

    write_output(&hedl, output)
}

// Parquet conversion

/// Convert a HEDL file to Parquet format.
///
/// Parses a HEDL file and converts it to Apache Parquet columnar format. This is ideal
/// for analytical workloads and integration with data processing frameworks.
///
/// # Arguments
///
/// * `file` - Path to the HEDL file to convert
/// * `output` - Output Parquet file path (required, cannot write to stdout)
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
/// - The HEDL structure is not compatible with Parquet
/// - Parquet conversion fails
/// - Output file cannot be written
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::to_parquet;
///
/// # fn main() -> Result<(), String> {
/// // Convert to Parquet file
/// to_parquet("data.hedl", "output.parquet")?;
/// # Ok(())
/// # }
/// ```
///
/// # Note
///
/// Parquet requires a file path for output; it cannot write to stdout due to
/// the binary columnar format.
pub fn to_parquet(file: &str, output: &str) -> Result<(), String> {
    let content = read_file(file)?;

    let doc = parse(content.as_bytes()).map_err(|e| format!("Parse error: {}", e))?;

    hedl_to_parquet(&doc, Path::new(output))
        .map_err(|e| format!("Parquet conversion error: {}", e))?;

    Ok(())
}

/// Convert a Parquet file to HEDL format.
///
/// Reads an Apache Parquet file and converts it to canonical HEDL format.
///
/// # Arguments
///
/// * `file` - Path to the Parquet file to convert
/// * `output` - Optional output file path. If `None`, writes to stdout
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns `Err` if:
/// - The file cannot be read
/// - The Parquet file is malformed or unsupported
/// - Parquet-to-HEDL conversion fails
/// - HEDL canonicalization fails
/// - Output writing fails
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::from_parquet;
///
/// # fn main() -> Result<(), String> {
/// // Convert Parquet to HEDL on stdout
/// from_parquet("data.parquet", None)?;
///
/// // Convert Parquet to HEDL file
/// from_parquet("data.parquet", Some("output.hedl"))?;
/// # Ok(())
/// # }
/// ```
pub fn from_parquet(file: &str, output: Option<&str>) -> Result<(), String> {
    let doc =
        parquet_to_hedl(Path::new(file)).map_err(|e| format!("Parquet conversion error: {}", e))?;

    let hedl = canonicalize(&doc).map_err(|e| format!("HEDL generation error: {}", e))?;

    write_output(&hedl, output)
}

// TOON conversion

/// Convert a HEDL file to TOON format.
///
/// Parses a HEDL file and converts it to TOON format.
///
/// # Arguments
///
/// * `file` - Path to the HEDL file to convert
/// * `output` - Optional output file path. If `None`, writes to stdout
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
/// - TOON conversion fails
/// - Output writing fails
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::to_toon;
///
/// # fn main() -> Result<(), String> {
/// // Convert to TOON on stdout
/// to_toon("data.hedl", None)?;
///
/// // Convert to TOON file
/// to_toon("data.hedl", Some("output.toon"))?;
/// # Ok(())
/// # }
/// ```
pub fn to_toon(file: &str, output: Option<&str>) -> Result<(), String> {
    let content = read_file(file)?;

    let doc = parse(content.as_bytes()).map_err(|e| format!("Parse error: {}", e))?;

    let toon = hedl_to_toon(&doc).map_err(|e| format!("TOON conversion error: {}", e))?;

    write_output(&toon, output)
}

/// Convert a TOON file to HEDL format.
///
/// Parses a TOON file and converts it to HEDL format.
///
/// # Arguments
///
/// * `file` - Path to the TOON file to convert
/// * `output` - Optional output file path. If `None`, writes to stdout
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
/// - HEDL generation fails
/// - Output writing fails
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::from_toon;
///
/// # fn main() -> Result<(), String> {
/// // Convert TOON to HEDL on stdout
/// from_toon("data.toon", None)?;
///
/// // Convert TOON to HEDL file
/// from_toon("data.toon", Some("output.hedl"))?;
/// # Ok(())
/// # }
/// ```
pub fn from_toon(file: &str, output: Option<&str>) -> Result<(), String> {
    let content = read_file(file)?;

    let doc = toon_to_hedl(&content).map_err(|e| format!("TOON parse error: {}", e))?;

    let hedl = canonicalize(&doc).map_err(|e| format!("HEDL generation error: {}", e))?;

    write_output(&hedl, output)
}

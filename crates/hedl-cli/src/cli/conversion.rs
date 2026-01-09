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

//! Format conversion commands for HEDL.
//!
//! This module provides bidirectional conversion between HEDL and various
//! popular data formats including JSON, YAML, XML, CSV, and Parquet.

use crate::commands;
use clap::Subcommand;

/// Format conversion commands.
///
/// These commands enable bidirectional conversion between HEDL and various
/// data formats, making HEDL interoperable with existing tools and workflows.
///
/// # Supported Formats
///
/// - **JSON**: Compact and pretty printing, optional metadata
/// - **YAML**: Standard YAML format
/// - **XML**: Compact and pretty printing
/// - **CSV**: Tabular data with optional headers
/// - **Parquet**: Apache Parquet columnar format
///
/// # Design
///
/// All conversion commands follow a consistent pattern:
/// - `to-<format>`: Convert HEDL to target format
/// - `from-<format>`: Convert target format to HEDL
#[derive(Subcommand)]
pub enum ConversionCommands {
    /// Convert HEDL to JSON
    ///
    /// Converts a HEDL file to JSON format with optional pretty printing
    /// and metadata inclusion.
    ToJson {
        /// Input HEDL file
        #[arg(value_name = "FILE")]
        file: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Include HEDL metadata in JSON
        #[arg(long)]
        metadata: bool,

        /// Pretty print JSON
        #[arg(short, long)]
        pretty: bool,
    },

    /// Convert JSON to HEDL
    ///
    /// Converts a JSON file to HEDL format.
    FromJson {
        /// Input JSON file
        #[arg(value_name = "FILE")]
        file: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Convert HEDL to YAML
    ///
    /// Converts a HEDL file to YAML format.
    ToYaml {
        /// Input HEDL file
        #[arg(value_name = "FILE")]
        file: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Convert YAML to HEDL
    ///
    /// Converts a YAML file to HEDL format.
    FromYaml {
        /// Input YAML file
        #[arg(value_name = "FILE")]
        file: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Convert HEDL to XML
    ///
    /// Converts a HEDL file to XML format with optional pretty printing.
    ToXml {
        /// Input HEDL file
        #[arg(value_name = "FILE")]
        file: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Pretty print XML
        #[arg(short, long)]
        pretty: bool,
    },

    /// Convert XML to HEDL
    ///
    /// Converts an XML file to HEDL format.
    FromXml {
        /// Input XML file
        #[arg(value_name = "FILE")]
        file: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Convert HEDL to CSV
    ///
    /// Converts a HEDL file containing tabular data to CSV format.
    /// Works best with HEDL matrix lists.
    ToCsv {
        /// Input HEDL file
        #[arg(value_name = "FILE")]
        file: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Include header row
        #[arg(long, default_value = "true")]
        headers: bool,
    },

    /// Convert CSV to HEDL
    ///
    /// Converts a CSV file to HEDL matrix list format.
    FromCsv {
        /// Input CSV file
        #[arg(value_name = "FILE")]
        file: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Type name for the matrix list
        #[arg(short, long, default_value = "Row")]
        type_name: String,
    },

    /// Convert HEDL to Parquet
    ///
    /// Converts a HEDL file to Apache Parquet columnar format.
    /// Requires an output file path (Parquet cannot write to stdout).
    ToParquet {
        /// Input HEDL file
        #[arg(value_name = "FILE")]
        file: String,

        /// Output Parquet file path (required)
        #[arg(short, long)]
        output: String,
    },

    /// Convert Parquet to HEDL
    ///
    /// Converts an Apache Parquet file to HEDL format.
    FromParquet {
        /// Input Parquet file
        #[arg(value_name = "FILE")]
        file: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Convert HEDL to TOON
    ///
    /// Converts a HEDL file to TOON format.
    ToToon {
        /// Input HEDL file
        #[arg(value_name = "FILE")]
        file: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Convert TOON to HEDL
    ///
    /// Converts a TOON file to HEDL format.
    FromToon {
        /// Input TOON file
        #[arg(value_name = "FILE")]
        file: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
}

impl ConversionCommands {
    /// Execute the conversion command.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error message on failure.
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    /// - File I/O fails
    /// - Parsing fails
    /// - Conversion fails
    /// - Output writing fails
    pub fn execute(self) -> Result<(), String> {
        match self {
            ConversionCommands::ToJson {
                file,
                output,
                metadata,
                pretty,
            } => commands::to_json(&file, output.as_deref(), metadata, pretty),
            ConversionCommands::FromJson { file, output } => {
                commands::from_json(&file, output.as_deref())
            }
            ConversionCommands::ToYaml { file, output } => {
                commands::to_yaml(&file, output.as_deref())
            }
            ConversionCommands::FromYaml { file, output } => {
                commands::from_yaml(&file, output.as_deref())
            }
            ConversionCommands::ToXml {
                file,
                output,
                pretty,
            } => commands::to_xml(&file, output.as_deref(), pretty),
            ConversionCommands::FromXml { file, output } => {
                commands::from_xml(&file, output.as_deref())
            }
            ConversionCommands::ToCsv {
                file,
                output,
                headers,
            } => commands::to_csv(&file, output.as_deref(), headers),
            ConversionCommands::FromCsv {
                file,
                output,
                type_name,
            } => commands::from_csv(&file, output.as_deref(), &type_name),
            ConversionCommands::ToParquet { file, output } => commands::to_parquet(&file, &output),
            ConversionCommands::FromParquet { file, output } => {
                commands::from_parquet(&file, output.as_deref())
            }
            ConversionCommands::ToToon { file, output } => {
                commands::to_toon(&file, output.as_deref())
            }
            ConversionCommands::FromToon { file, output } => {
                commands::from_toon(&file, output.as_deref())
            }
        }
    }
}

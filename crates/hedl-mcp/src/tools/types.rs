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

//! Shared types for MCP tools.

use serde::Deserialize;
use serde_json::Value as JsonValue;

/// Maximum input size in bytes for HEDL/JSON string arguments (10 MB).
pub const MAX_INPUT_SIZE: usize = 10 * 1024 * 1024;

// ============ Argument Structures ============

#[derive(Debug, Deserialize)]
pub struct ReadArgs {
    pub path: String,
    #[serde(default = "default_true")]
    pub recursive: bool,
    #[serde(default)]
    pub include_json: bool,
    /// Number of threads to use for parallel file reading.
    ///
    /// When set to None or 0, uses the default rayon thread pool (typically CPU core count).
    /// For single-file operations, parallelism is not used.
    /// For directory operations, files are read in parallel using the specified thread count.
    #[serde(default)]
    pub num_threads: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct QueryArgs {
    pub hedl: String,
    pub type_name: Option<String>,
    pub id: Option<String>,
    #[serde(default = "default_true")]
    pub include_children: bool,
}

#[derive(Debug, Deserialize)]
pub struct ValidateArgs {
    pub hedl: String,
    /// Enable strict validation mode.
    ///
    /// When enabled:
    /// - Lint warnings are treated as errors
    /// - Validation fails if any warnings are present
    #[serde(default = "default_true")]
    pub strict: bool,
    #[serde(default = "default_true")]
    pub lint: bool,
}

#[derive(Debug, Deserialize)]
pub struct OptimizeArgs {
    pub json: String,
    #[serde(default = "default_true")]
    pub ditto: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub compact: bool,
}

#[derive(Debug, Deserialize)]
pub struct StatsArgs {
    pub hedl: String,
    #[serde(default = "default_simple")]
    pub tokenizer: String,
}

#[derive(Debug, Deserialize)]
pub struct FormatArgs {
    pub hedl: String,
    #[serde(default = "default_true")]
    pub ditto: bool,
}

#[derive(Debug, Deserialize)]
pub struct WriteArgs {
    pub path: String,
    pub content: String,
    #[serde(default = "default_true")]
    pub validate: bool,
    #[serde(default)]
    pub format: bool,
    #[serde(default = "default_true")]
    pub backup: bool,
}

#[derive(Debug, Deserialize)]
pub struct ConvertToArgs {
    pub hedl: String,
    pub format: String,
    #[serde(default)]
    pub options: Option<JsonValue>,
}

#[derive(Debug, Deserialize)]
pub struct ConvertFromArgs {
    pub content: String,
    pub format: String,
    #[serde(default)]
    pub options: Option<JsonValue>,
}

#[derive(Debug, Deserialize)]
pub struct StreamArgs {
    pub hedl: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
    pub type_filter: Option<String>,
}

// ============ Default Value Functions ============

pub fn default_true() -> bool {
    true
}

pub fn default_simple() -> String {
    "simple".to_string()
}

pub fn default_limit() -> usize {
    100
}

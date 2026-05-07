// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Metrics types for HEDL analytics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Metrics for a single command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetrics {
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub filter_type: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

impl CommandMetrics {
    pub fn savings_percentage(&self) -> f64 {
        if self.input_tokens == 0 {
            return 0.0;
        }
        100.0 - (self.output_tokens as f64 / self.input_tokens as f64 * 100.0)
    }
}

/// Metrics for format conversions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatMetrics {
    pub timestamp: DateTime<Utc>,
    pub from_format: String,
    pub to_format: String,
    pub input_size: usize,
    pub output_size: usize,
    pub input_tokens: usize,
    pub output_tokens: usize,
}

impl FormatMetrics {
    pub fn savings_percentage(&self) -> f64 {
        if self.input_tokens == 0 {
            return 0.0;
        }
        100.0 - (self.output_tokens as f64 / self.input_tokens as f64 * 100.0)
    }
}

/// Aggregated token metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenMetrics {
    pub total_commands: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_duration_ms: u64,
    pub native_filtered: u64,
    pub toml_filtered: u64,
    pub passthrough: u64,
}

impl TokenMetrics {
    pub fn savings_percentage(&self) -> f64 {
        if self.total_input_tokens == 0 {
            return 0.0;
        }
        100.0 - (self.total_output_tokens as f64 / self.total_input_tokens as f64 * 100.0)
    }

    pub fn average_duration_ms(&self) -> u64 {
        if self.total_commands == 0 {
            return 0;
        }
        self.total_duration_ms / self.total_commands
    }
}

// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! In-memory storage for analytics data.

use crate::metrics::{CommandMetrics, FormatMetrics, TokenMetrics};
use std::collections::VecDeque;
use std::sync::Mutex;

const MAX_HISTORY: usize = 10000;

pub struct AnalyticsStorage {
    commands: Mutex<VecDeque<CommandMetrics>>,
    conversions: Mutex<VecDeque<FormatMetrics>>,
}

impl AnalyticsStorage {
    pub fn new() -> Self {
        Self {
            commands: Mutex::new(VecDeque::with_capacity(MAX_HISTORY)),
            conversions: Mutex::new(VecDeque::with_capacity(MAX_HISTORY)),
        }
    }

    pub fn record_command(&self, metrics: CommandMetrics) {
        if let Ok(mut commands) = self.commands.lock() {
            if commands.len() >= MAX_HISTORY {
                commands.pop_front();
            }
            commands.push_back(metrics);
        }
    }

    pub fn record_conversion(&self, metrics: FormatMetrics) {
        if let Ok(mut conversions) = self.conversions.lock() {
            if conversions.len() >= MAX_HISTORY {
                conversions.pop_front();
            }
            conversions.push_back(metrics);
        }
    }

    pub fn get_token_metrics(&self) -> TokenMetrics {
        let mut totals = TokenMetrics::default();

        if let Ok(commands) = self.commands.lock() {
            for cmd in commands.iter() {
                totals.total_commands += 1;
                totals.total_input_tokens += cmd.input_tokens as u64;
                totals.total_output_tokens += cmd.output_tokens as u64;
                totals.total_duration_ms += cmd.duration_ms;

                match cmd.filter_type.as_str() {
                    "native" => totals.native_filtered += 1,
                    "toml" => totals.toml_filtered += 1,
                    _ => totals.passthrough += 1,
                }
            }
        }

        totals
    }

    pub fn get_recent_commands(&self,
        limit: usize,
    ) -> Vec<CommandMetrics> {
        if let Ok(commands) = self.commands.lock() {
            commands.iter().rev().take(limit).cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_command_history(&self,
    ) -> Vec<CommandMetrics> {
        if let Ok(commands) = self.commands.lock() {
            commands.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for AnalyticsStorage {
    fn default() -> Self {
        Self::new()
    }
}

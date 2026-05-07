// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Analytics and tracking for token savings.

use std::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub struct SavingsStats {
    pub commands_processed: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub native_filtered: u64,
    pub toml_filtered: u64,
    pub passthrough: u64,
}

impl SavingsStats {
    pub fn savings_percentage(&self) -> f64 {
        if self.input_tokens == 0 { return 0.0; }
        100.0 - (self.output_tokens as f64 / self.input_tokens as f64 * 100.0)
    }

    pub fn format_report(&self) -> String {
        format!(
            "Token Savings Report:\n\
             - Commands processed: {}\n\
             - Input tokens: {}\n\
             - Output tokens: {}\n\
             - Savings: {:.1}%\n\
             - Native filters: {}\n\
             - TOML filters: {}\n\
             - Passthrough: {}",
            self.commands_processed,
            crate::utils::format_tokens(self.input_tokens as usize),
            crate::utils::format_tokens(self.output_tokens as usize),
            self.savings_percentage(),
            self.native_filtered,
            self.toml_filtered,
            self.passthrough
        )
    }
}

static GLOBAL_STATS: Mutex<SavingsStats> = Mutex::new(SavingsStats {
    commands_processed: 0,
    input_tokens: 0,
    output_tokens: 0,
    native_filtered: 0,
    toml_filtered: 0,
    passthrough: 0,
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    Native,
    Toml,
    Passthrough,
}

pub fn track_savings_raw(input_tokens: usize, output_tokens: usize, filter_type: FilterType) {
    if let Ok(mut stats) = GLOBAL_STATS.lock() {
        stats.commands_processed += 1;
        stats.input_tokens += input_tokens as u64;
        stats.output_tokens += output_tokens as u64;
        match filter_type {
            FilterType::Native => stats.native_filtered += 1,
            FilterType::Toml => stats.toml_filtered += 1,
            FilterType::Passthrough => stats.passthrough += 1,
        }
    }
}

pub fn get_stats() -> SavingsStats {
    GLOBAL_STATS.lock().map(|s| s.clone()).unwrap_or_default()
}

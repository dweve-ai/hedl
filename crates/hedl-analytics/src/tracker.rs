// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Global analytics tracker.

use crate::metrics::{CommandMetrics, FormatMetrics};
use crate::storage::AnalyticsStorage;
use chrono::Utc;
use once_cell::sync::Lazy;
use std::time::Instant;

static STORAGE: Lazy<AnalyticsStorage> = Lazy::new(AnalyticsStorage::new);

/// Report of analytics data.
#[derive(Debug, Clone)]
pub struct AnalyticsReport {
    pub total_commands: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub savings_percentage: f64,
    pub native_filtered: u64,
    pub toml_filtered: u64,
    pub passthrough: u64,
    pub average_duration_ms: u64,
}

impl AnalyticsReport {
    pub fn format(&self) -> String {
        format!(
            "HEDL Analytics Report\n\
             ====================\n\
             Commands processed: {}\n\
             Input tokens:       {}\n\
             Output tokens:      {}\n\
             Savings:            {:.1}%\n\
             Native filters:     {}\n\
             TOML filters:       {}\n\
             Passthrough:        {}\n\
             Avg duration:       {}ms",
            self.total_commands,
            format_number(self.total_input_tokens),
            format_number(self.total_output_tokens),
            self.savings_percentage,
            self.native_filtered,
            self.toml_filtered,
            self.passthrough,
            self.average_duration_ms,
        )
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

/// Track a command execution.
pub fn track_command(
    command: &str,
    input_tokens: usize,
    output_tokens: usize,
    filter_type: &str,
    exit_code: i32,
    duration_ms: u64,
) {
    STORAGE.record_command(CommandMetrics {
        timestamp: Utc::now(),
        command: command.to_string(),
        input_tokens,
        output_tokens,
        filter_type: filter_type.to_string(),
        exit_code,
        duration_ms,
    });
}

/// Track a format conversion.
pub fn track_format_conversion(
    from_format: &str,
    to_format: &str,
    input_size: usize,
    output_size: usize,
    input_tokens: usize,
    output_tokens: usize,
) {
    STORAGE.record_conversion(FormatMetrics {
        timestamp: Utc::now(),
        from_format: from_format.to_string(),
        to_format: to_format.to_string(),
        input_size,
        output_size,
        input_tokens,
        output_tokens,
    });
}

/// Get the current analytics report.
pub fn get_analytics_report() -> AnalyticsReport {
    let metrics = STORAGE.get_token_metrics();
    AnalyticsReport {
        total_commands: metrics.total_commands,
        total_input_tokens: metrics.total_input_tokens,
        total_output_tokens: metrics.total_output_tokens,
        savings_percentage: metrics.savings_percentage(),
        native_filtered: metrics.native_filtered,
        toml_filtered: metrics.toml_filtered,
        passthrough: metrics.passthrough,
        average_duration_ms: metrics.average_duration_ms(),
    }
}

/// Get recent command history.
pub fn get_recent_commands(limit: usize) -> Vec<CommandMetrics> {
    STORAGE.get_recent_commands(limit)
}

/// Get all command history.
pub fn get_command_history() -> Vec<CommandMetrics> {
    STORAGE.get_command_history()
}

/// Start a timer for measuring command duration.
pub fn start_timer() -> Instant {
    Instant::now()
}

/// Get elapsed milliseconds from a timer.
pub fn elapsed_ms(timer: Instant) -> u64 {
    timer.elapsed().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_command() {
        track_command("git status", 100, 30, "native", 0, 5);
        let report = get_analytics_report();
        assert_eq!(report.total_commands, 1);
        assert_eq!(report.total_input_tokens, 100);
        assert_eq!(report.total_output_tokens, 30);
        assert!((report.savings_percentage - 70.0).abs() < 0.1);
    }

    #[test]
    fn test_analytics_report_format() {
        let report = AnalyticsReport {
            total_commands: 10,
            total_input_tokens: 1000,
            total_output_tokens: 300,
            savings_percentage: 70.0,
            native_filtered: 5,
            toml_filtered: 3,
            passthrough: 2,
            average_duration_ms: 10,
        };
        let formatted = report.format();
        assert!(formatted.contains("Commands processed: 10"));
        assert!(formatted.contains("Savings:"));
        assert!(formatted.contains("70.0%"));
    }
}
